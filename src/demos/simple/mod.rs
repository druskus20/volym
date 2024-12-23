use std::path::Path;

use tracing::info;
use wgpu::BindGroupLayoutEntry;

use crate::{
    demos::pipeline::{DemoPipeline, DemoPipelineConfig},
    gpu_context::Context,
    gpu_resources::{
        camera::GpuCamera,
        debug_matrix::GpuDebugMatrix,
        output_texture::GpuOutputTexture,
        transfer_function::GPUTransferFunction,
        volume::{FlipMode, GpuVolume},
        BindGroupLayoutEntryUnbound, ToBindGroupEntries, ToBindGroupLayoutEntries, ToGpuResources,
    },
    state::State,
    transfer_function,
};

use super::ComputeDemo;
use crate::Result;

#[derive(Debug)]
pub struct Simple {
    compute_pipeline: super::pipeline::DemoPipeline,

    // Resources for state
    volume: GpuVolume,
    transfer_function: GPUTransferFunction,
    camera: GpuCamera,
    debug_matrix: GpuDebugMatrix,
    output_texture: GpuOutputTexture,

    // Layouts of the pipeline
    base_inputs_layout: wgpu::BindGroupLayout,
    base_outputs_layout: wgpu::BindGroupLayout,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));
        let volume = GpuVolume::init(volume_path.as_ref(), FlipMode::None, ctx)?;
        let transfer_function = transfer_function::TransferFunction1D::default();
        let camera = GpuCamera::new(ctx, state);
        let debug_matrix = GpuDebugMatrix::new(ctx, state);
        let output_texture = GpuOutputTexture::new(ctx, state, output_texture);
        let transfer_function =
            GPUTransferFunction::new_texture_1d_rgbt(&transfer_function, &ctx.device, &ctx.queue);

        let shader_path =
            Path::new(&(format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"))))
                .to_path_buf();

        let base_inputs = [
            GpuVolume::BIND_GROUP_LAYOUT_ENTRIES,
            GpuCamera::BIND_GROUP_LAYOUT_ENTRIES,
            GPUTransferFunction::BIND_GROUP_LAYOUT_ENTRIES,
        ];

        let base_outputs = [
            GpuOutputTexture::BIND_GROUP_LAYOUT_ENTRIES,
            GpuDebugMatrix::BIND_GROUP_LAYOUT_ENTRIES,
        ];

        let base_inputs_layout =
            layout_from_unbound_entries(ctx, "Base Inputs Layout", &base_inputs);

        let base_outputs_layout =
            layout_from_unbound_entries(ctx, "Base Outputs Layout", &base_outputs);

        let compute_pipeline = DemoPipeline::with_config(
            ctx,
            &DemoPipelineConfig {
                shader_path,
                bind_group_layouts: &[&base_inputs_layout, &base_outputs_layout],
            },
        )?;

        Ok(Simple {
            volume,
            compute_pipeline,
            transfer_function,
            camera,
            debug_matrix,
            output_texture,
            base_inputs_layout,
            base_outputs_layout,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.camera.update(ctx, state)?;
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
        let base_inputs_resources = [
            self.volume.to_gpu_resources(),
            self.camera.to_gpu_resources(),
            self.transfer_function.to_gpu_resources(),
        ];

        let base_outputs_resources = [
            self.output_texture.to_gpu_resources(),
            self.debug_matrix.to_gpu_resources(),
        ];

        let base_inputs_group = bindgroup_from_resources(
            ctx,
            "Base Inputs Bind Group",
            &self.base_inputs_layout,
            &base_inputs_resources,
        );

        let base_outputs_group = bindgroup_from_resources(
            ctx,
            "Base Outputs Bind Group",
            &self.base_outputs_layout,
            &base_outputs_resources,
        );

        self.compute_pipeline
            .compute_pass(ctx, &[&base_inputs_group, &base_outputs_group]);

        Ok(())
    }
}

fn layout_from_unbound_entries(
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

fn bindgroup_from_resources(
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
