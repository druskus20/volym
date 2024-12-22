use std::path::Path;

use tracing::info;

use crate::{
    demos::pipeline::{DemoPipeline, DemoPipelineConfig},
    gpu::{
        camera::GpuCamera,
        context::Context,
        debug_matrix::GpuDebugMatrix,
        output_texture::GpuOutputTexture,
        tf::GPUTransferFunction,
        volume::{FlipMode, GpuVolume},
    },
    state::State,
    transfer_function,
};

use super::ComputeDemo;
use crate::Result;

#[derive(Debug)]
pub struct Simple {
    compute_pipeline: super::pipeline::DemoPipeline,

    // Resources
    volume: GpuVolume,
    transfer_function: GPUTransferFunction,
    camera: GpuCamera,
    debug_matrix: GpuDebugMatrix,
    output_texture: GpuOutputTexture,
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

        let bind_group_layouts = &[
            &volume.layout,
            &output_texture.layout,
            &camera.layout,
            &debug_matrix.layout,
            &transfer_function.layout,
        ];
        let compute_pipeline = DemoPipeline::with_config(
            ctx,
            &DemoPipelineConfig {
                shader_path,
                bind_group_layouts,
            },
        )?;

        Ok(Simple {
            volume,
            compute_pipeline,
            transfer_function,
            camera,
            debug_matrix,
            output_texture,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.camera.update(ctx, state)?;
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
        let bind_groups = &[
            &self.volume.group,
            &self.output_texture.group,
            &self.camera.group,
            &self.debug_matrix.group,
            &self.transfer_function.group,
        ];
        self.compute_pipeline.compute_pass(ctx, bind_groups);

        Ok(())
    }
}
