use std::path::Path;

use crate::context;

use super::RenderingAlgorithm;
use crate::Result;

pub mod compute_pipeline;
pub mod volume;

pub struct SimpleRaycaster {
    volume: volume::Volume,                      // contains the bindgroup
    pipeline: compute_pipeline::ComputePipeline, // contains the bindgrouplayout
    compute_bind_group: wgpu::BindGroup,
}

impl RenderingAlgorithm for SimpleRaycaster {
    fn init(ctx: &mut context::Context, volume: volume::Volume) -> Result<Self> {
        let compute_path = format!(
            "{}/shaders/raycast_compute.wgsl",
            env!("CARGO_MANIFEST_DIR")
        );
        let input_texture_layout = ctx
            .device
            .create_bind_group_layout(&crate::volume::Volume::DESC);
        let pipeline = crate::compute_pipeline::ComputePipeline::new(
            &ctx.device,
            Path::new(&compute_path),
            input_texture_layout,
        )?;
        let compute_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ctx.texture_view),
            }],
        });

        Ok(SimpleRaycaster {
            volume,
            pipeline,
            compute_bind_group,
        })
    }

    fn compute(&self, ctx: &mut context::Context) -> Result<()> {
        let size = ctx.size;
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

            compute_pass.set_pipeline(self.pipeline.as_ref());

            // Get the volume inputs
            compute_pass.set_bind_group(0, self.volume.bind_group(), &[]);
            // Get the pipeline inputs
            compute_pass.set_bind_group(1, &self.compute_bind_group, &[]);

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgroup.
            compute_pass.dispatch_workgroups((size.width + 15) / 16, (size.height + 15) / 16, 1);
        }
        ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}
