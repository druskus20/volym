use std::path::Path;

use tracing::info;

use crate::{
    demos::pipeline::{layout_from_unbound_entries, BaseDemoConfig},
    gpu_context::Context,
    gpu_resources::{
        transfer_function::GPUTransferFunction,
        volume::{FlipMode, GpuVolume},
        ToGpuResources,
    },
    state::State,
    transfer_function,
};

use super::{pipeline::bindgroup_from_resources, ComputeDemo};
use crate::Result;

#[derive(Debug)]
pub struct Simple {
    // Base demo
    base: super::pipeline::BaseDemo,

    // Resources for state
    _volume: GpuVolume,
    _transfer_function: GPUTransferFunction,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            //"{}/assets/boston_teapot_256x256x178_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));
        let volume = GpuVolume::init(volume_path.as_ref(), FlipMode::None, ctx)?;
        let transfer_function = transfer_function::TransferFunction::default();
        let gpu_transfer_function =
            GPUTransferFunction::new_texture_1d_rgbt(&transfer_function, &ctx.device, &ctx.queue);

        let shader_path =
            Path::new(&(format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"))))
                .to_path_buf();

        let extra_layout = layout_from_unbound_entries(
            ctx,
            "Extra Layout",
            &[
                GpuVolume::BIND_GROUP_LAYOUT_ENTRIES,
                GPUTransferFunction::BIND_GROUP_LAYOUT_ENTRIES,
            ],
        );
        let extra_bind_groups = bindgroup_from_resources(
            ctx,
            "Extra Bind Group",
            &extra_layout,
            &[
                volume.to_gpu_resources(),
                gpu_transfer_function.to_gpu_resources(),
            ],
        );

        let config = BaseDemoConfig {
            shader_path,
            output_texture,
            extra_bind_groups: vec![extra_bind_groups],
            extra_layouts: vec![extra_layout],
        };

        let base = super::pipeline::BaseDemo::init(ctx, state, config)?;

        Ok(Self {
            base,
            _volume: volume,
            _transfer_function: gpu_transfer_function,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.base.update_gpu_state(ctx, state)?;
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
        self.base.compute_pass(ctx)?;

        Ok(())
    }
}
