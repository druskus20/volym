use tracing::info;

use crate::{rendering_context::Context, state::State};

use super::ComputeDemo;
use crate::Result;

pub mod compute_pipeline;
pub mod volume;

#[derive(Debug)]
pub struct Simple {
    volume: volume::Volume, // contains the bindgroup
    compute_pipeline: compute_pipeline::ComputePipeline,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));

        let volume = volume::Volume::init(volume_path.as_ref(), volume::FlipMode::None, ctx)?;
        info!("Volume loaded: {:?}", volume_path);

        let compute_pipeline =
            compute_pipeline::ComputePipeline::new(ctx, state, output_texture, &volume.layout)?;

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
        self.compute_pipeline.compute_pass(ctx, &self.volume);
        Ok(())
    }
}
