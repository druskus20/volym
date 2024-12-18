/// Compute pipeline that does the heavy lifting and outputs to a texture
use std::path::Path;

use tracing::{debug, info};

use crate::{demos::compute_base, rendering_context::Context, state::State, Result};

use super::volume::Volume;

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
        input_volume_layout: &wgpu::BindGroupLayout,
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

        info!("Creating compute pipeline");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    input_volume_layout,
                    &base.output_texture_layout,
                    &base.camera_layout,
                    &base.debug_matrix_layout,
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

    pub fn compute_pass(&self, ctx: &Context, volume: &Volume) {
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
            // Get the volume inputs
            // TODO: consider moving the bind_group to the compute pipeline or something
            compute_pass.set_bind_group(0, &volume.bind_group, &[]);
            debug!(target = "compute_pass", "Volume inputs bind_group set");
            // Get the pipeline inputs
            compute_pass.set_bind_group(1, &base.output_texture_group, &[]);
            debug!(target = "compute_pass", "Output texture bind_group set");

            compute_pass.set_bind_group(2, &base.camera_group, &[]);
            debug!(target = "compute_pass", "Camera bind_group set");

            compute_pass.set_bind_group(3, &base.debug_matrix_group, &[]);
            debug!(target = "compute_pass", "Debug matrix bind_group set");

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
