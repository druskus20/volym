use tracing::{debug, info};

use crate::{rendering_context::Context, state::State};

use super::Demo;
use crate::Result;

pub mod compute_pipeline;
pub mod volume;

#[derive(Debug)]
pub struct Simple {
    volume: volume::Volume, // contains the bindgroup
    compute_pipeline: compute_pipeline::ComputePipeline,
}

impl Demo for Simple {
    #[tracing::instrument()]
    fn init(ctx: &Context, state: &State, output_texture_view: &wgpu::TextureView) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));

        let (volume, input_volume_layout) = volume::Volume::init(
            volume_path.as_ref(),
            volume::FlipMode::None,
            &ctx.device,
            &ctx.queue,
        )?;
        info!("Volume loaded: {:?}", volume_path);

        // TODO maybe move this to the context. And make it an argument of compute()
        let compute_pipeline = compute_pipeline::ComputePipeline::new(
            ctx,
            state,
            output_texture_view,
            &input_volume_layout,
        )?;

        Ok(Simple {
            volume,
            compute_pipeline,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.compute_pipeline.base.update(ctx, state);
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
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
            compute_pass.set_pipeline(self.compute_pipeline.as_ref());

            let base = &self.compute_pipeline.base;
            // Get the volume inputs
            // TODO: consider moving the bind_group to the compute pipeline or something
            compute_pass.set_bind_group(0, &self.volume.bind_group, &[]);
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

        Ok(())
    }
}
