/// Compute pipeline that does the he'a avy lifting and outputs to a texture
use std::path::PathBuf;

use crate::{
    gpu_resources::{
        camera::GpuCamera, debug_matrix::GpuDebugMatrix, output_texture::GpuOutputTexture,
        BindGroupLayoutEntryUnbound, ToBindGroupEntries, ToBindGroupLayoutEntries, ToGpuResources,
    },
    state::State,
    Result,
};
use tracing::{debug, info};
use wgpu::BindGroupLayout;

use crate::gpu_context::Context;

#[derive(Debug)]
pub struct DemoPipeline {
    pub pipeline: wgpu::ComputePipeline,
}

pub(crate) struct DemoPipelineConfig<'a> {
    pub shader_path: PathBuf,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl DemoPipeline {
    pub fn with_config(ctx: &Context, config: &DemoPipelineConfig) -> Result<Self> {
        let shader_contents = std::fs::read_to_string(&config.shader_path)?;
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(config.shader_path.to_str().unwrap()),
                source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
            });
        info!("Loaded shader module from {:?}", config.shader_path);

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: config.bind_group_layouts,
                push_constant_ranges: &[],
            });

        info!("Creating compute pipeline");
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: Default::default(),
            });

        Ok(DemoPipeline { pipeline })
    }

    pub fn compute_pass(&self, ctx: &Context, bind_groups: &[&wgpu::BindGroup]) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(self.as_ref());

            for (i, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(i as u32, *bind_group, &[]);
            }

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgro
            compute_pass.dispatch_workgroups(
                (ctx.size.width + 15) / 16,
                (ctx.size.height + 15) / 16,
                1,
            );
            debug!(
                target = "compute_pass",
                "dispatch_workgroups: {}, {}, {}",
                (ctx.size.width + 15) / 16,
                (ctx.size.height + 15) / 16,
                1
            );
        }

        ctx.queue.submit(Some(encoder.finish()));
        debug!(
            target = "compute_pass",
            "Compute task submitted to the queue"
        );
    }
}

impl AsRef<wgpu::ComputePipeline> for DemoPipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}

// TODO: Should this be a trait? - probably not
#[derive(Debug)]
pub struct BaseDemo {
    compute_pipeline: super::pipeline::DemoPipeline,

    // Resources for state
    camera: GpuCamera,
    _debug_matrix: GpuDebugMatrix,
    _output_texture: GpuOutputTexture,

    // Bind groups
    base_inputs_group: wgpu::BindGroup,
    base_outputs_group: wgpu::BindGroup,
    extra_bind_groups: Vec<wgpu::BindGroup>,
}

#[derive(Debug)]
pub struct BaseDemoConfig<'a> {
    pub shader_path: PathBuf,
    pub output_texture: &'a wgpu::Texture,
    pub extra_bind_groups: Vec<wgpu::BindGroup>,
    pub extra_layouts: Vec<BindGroupLayout>,
}

impl BaseDemo {
    pub fn init(ctx: &Context, state: &State, config: BaseDemoConfig) -> Result<Self> {
        info!("Initializing Simple Demo");

        let camera = GpuCamera::new(ctx, state);
        let debug_matrix = GpuDebugMatrix::new(ctx, state);
        let output_texture = GpuOutputTexture::new(ctx, state, config.output_texture);

        let base_inputs_layout = layout_from_unbound_entries(
            ctx,
            "Base Inputs Layout",
            &[
                //GpuVolume::BIND_GROUP_LAYOUT_ENTRIES,
                GpuCamera::BIND_GROUP_LAYOUT_ENTRIES,
                //GPUTransferFunction::BIND_GROUP_LAYOUT_ENTRIES,
            ],
        );

        let base_outputs_layout = layout_from_unbound_entries(
            ctx,
            "Base Outputs Layout",
            &[
                GpuOutputTexture::BIND_GROUP_LAYOUT_ENTRIES,
                GpuDebugMatrix::BIND_GROUP_LAYOUT_ENTRIES,
            ],
        );

        let mut bind_group_layouts = vec![&base_inputs_layout, &base_outputs_layout];
        let extra_layouts = &config.extra_layouts;
        for layout in extra_layouts {
            bind_group_layouts.push(layout);
        }

        // flat concat of all the bind group layouts
        // Should be created at the BaseDemo level, passing extra's
        let compute_pipeline = DemoPipeline::with_config(
            ctx,
            &DemoPipelineConfig {
                shader_path: config.shader_path.clone(),
                bind_group_layouts: bind_group_layouts.as_slice(),
            },
        )?;

        let base_inputs_group = bindgroup_from_resources(
            ctx,
            "Base Inputs Bind Group",
            &base_inputs_layout,
            &[camera.to_gpu_resources()],
        );
        let base_outputs_group = bindgroup_from_resources(
            ctx,
            "Base Outputs Bind Group",
            &base_outputs_layout,
            &[
                output_texture.to_gpu_resources(),
                debug_matrix.to_gpu_resources(),
            ],
        );

        let extra_bind_groups = config.extra_bind_groups;

        Ok(Self {
            compute_pipeline,
            camera,
            _debug_matrix: debug_matrix,
            _output_texture: output_texture,
            base_inputs_group,
            base_outputs_group,
            extra_bind_groups,
        })
    }

    pub fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.camera.update(ctx, state)?;
        Ok(())
    }

    pub fn compute_pass(&self, ctx: &Context) -> Result<()> {
        let mut bind_groups = vec![&self.base_inputs_group, &self.base_outputs_group];

        for bind_group in &self.extra_bind_groups {
            bind_groups.push(bind_group);
        }

        self.compute_pipeline
            .compute_pass(ctx, bind_groups.as_slice());

        Ok(())
    }
}

pub fn layout_from_unbound_entries(
    ctx: &Context,
    label: &str,
    base_inputs: &[&[BindGroupLayoutEntryUnbound]],
) -> wgpu::BindGroupLayout {
    let flat_base_inputs = base_inputs
        .iter()
        .flat_map(|x| x.iter())
        .collect::<Vec<&BindGroupLayoutEntryUnbound>>();

    ctx.device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: flat_base_inputs.to_bind_group_layout_entries().as_slice(),
        })
}

pub fn bindgroup_from_resources(
    ctx: &Context,
    label: &str,
    base_inputs_layout: &wgpu::BindGroupLayout,
    base_inputs_resources: &[Vec<wgpu::BindingResource>],
) -> wgpu::BindGroup {
    let flat_base_inputs_resources: Vec<wgpu::BindingResource> = base_inputs_resources
        .iter()
        .flat_map(|x| x.iter())
        .cloned()
        .collect();

    ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout: base_inputs_layout,
        entries: flat_base_inputs_resources
            .to_bind_group_entries()
            .as_slice(),
    })
}
