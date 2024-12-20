use color_eyre::owo_colors::OwoColorize;
use gpu_volume::GPUVolume;
use tracing::info;

use crate::{rendering_context::Context, state::State, transfer_function};

use super::{compute_base, ComputeDemo};
use crate::Result;

pub mod compute_pipeline;
pub mod gpu_transfer_function;
pub mod gpu_volume;

#[derive(Debug)]
pub struct Simple {
    volume: gpu_volume::GPUVolume, // contains the bindgroup
    compute_pipeline: compute_pipeline::ComputePipeline,
    transfer_function: gpu_transfer_function::GPUTransferFunction,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));

        let volume = GPUVolume::init(volume_path.as_ref(), gpu_volume::FlipMode::None, ctx)?;
        info!("Volume loaded: {:?}", volume_path);

        let transfer_function = transfer_function::TransferFunction1D::default().normalized();
        info!("Transfer Function initialized");
        info!("TF value at 0: {:?}", transfer_function.get(0.0));
        info!("TF value at 0.5: {:?}", transfer_function.get(0.5));
        info!("TF value at 1: {:?}", transfer_function.get(1.0));
        info!("TF value at 128: {:?}", transfer_function.get(128.0));

        let transfer_function = gpu_transfer_function::GPUTransferFunction::new_texture_1d_rgbt(
            &transfer_function,
            &ctx.device,
            &ctx.queue,
        );

        let compute_pipeline = compute_pipeline::ComputePipeline::new(
            ctx,
            state,
            output_texture,
            &volume,
            &transfer_function,
        )?;

        Ok(Simple {
            volume,
            compute_pipeline,
            transfer_function,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.compute_pipeline.base.update(ctx, state)?;
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
        self.compute_pipeline
            .compute_pass(ctx, &self.volume, &self.transfer_function);
        Ok(())
    }
}
