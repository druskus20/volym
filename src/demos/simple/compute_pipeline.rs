/// Compute pipeline that does the heavy lifting and outputs to a texture
use std::path::Path;

use tracing::{debug, info};
use tracing_subscriber::filter::BadFieldName;

use crate::{
    demos::{compute_base, simple::gpu_transfer_function::GPUTransferFunction},
    rendering_context::Context,
    state::State,
    Result,
};

use super::gpu_volume::GPUVolume;

#[derive(Debug)]
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub base: compute_base::ComputeBase,
}

impl ComputePipeline {
    pub fn new(
        ctx: &Context,
        state: &State,
        output_texture: &wgpu::Texture,
        volume: &GPUVolume,
        transfer_function: &GPUTransferFunction,
        band_colors_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        let device = &ctx.device;
        let base = compute_base::ComputeBase::new(ctx, state, output_texture);

        let shader_path =
            Path::new(&(format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"))))
                .to_path_buf();
        let shader_contents = std::fs::read_to_string(&shader_path)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_path.to_str().unwrap()),
            source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
        });
        info!("Loaded shader module from {:?}", shader_path);

        info!("Creating compute pipeline");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    &volume.layout,
                    &base.output_texture_layout,
                    &base.camera_layout,
                    &base.debug_matrix_layout,
                    &transfer_function.layout,
                    &band_colors_layout,
                ],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Ok(ComputePipeline { pipeline, base })
    }

    pub fn compute_pass(
        &self,
        ctx: &Context,
        volume: &GPUVolume,
        transfer_function: &GPUTransferFunction,
        band_colors_group: &wgpu::BindGroup,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // move into compute_pipeline
        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(self.as_ref());

            let base = &self.base;

            // Bind the parameters to the shader
            compute_pass.set_bind_group(0, &volume.bind_group, &[]);
            compute_pass.set_bind_group(1, &base.output_texture_group, &[]);
            compute_pass.set_bind_group(2, &base.camera_group, &[]);
            compute_pass.set_bind_group(3, &base.debug_matrix_group, &[]);
            compute_pass.set_bind_group(4, &transfer_function.bind_group, &[]);
            compute_pass.set_bind_group(5, band_colors_group, &[]);

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

impl AsRef<wgpu::ComputePipeline> for ComputePipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
