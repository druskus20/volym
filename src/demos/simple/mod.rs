use std::path::Path;

use importance::GpuImportances;
use tracing::info;

use crate::{
    demos::pipeline::{layout_from_unbound_entries, BaseDemoConfig},
    gpu_context::Context,
    gpu_resources::{
        transfer_function::GPUTransferFunction, volume::GpuVolume, FlipMode, ToGpuResources,
    },
    state::State,
    transfer_function::TransferFunction,
    Result,
};

use super::{
    pipeline::{bindgroup_from_resources, BaseDemo},
    ComputeDemo,
};

mod importance;

#[derive(Debug)]
pub struct Simple {
    // Base demo
    base: BaseDemo,

    // Resources for state
    _volume: GpuVolume,
    _transfer_function: GPUTransferFunction,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        // Volume
        let volume_path = &(format!(
            //"{}/assets/bonsai_256x256x256_uint8.raw",
            "{}/assets/boston_teapot_256x256x178_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));
        let volume = GpuVolume::init(volume_path.as_ref(), FlipMode::Y, ctx)?;

        let importances_path = &(format!(
            "{}/assets/boston_teapot_256x256x178_uint8_importances.raw",
            env!("CARGO_MANIFEST_DIR")
        ));
        let importances = GpuImportances::init(importances_path.as_ref(), FlipMode::Y, ctx)?;

        // TF
        let transfer_function = TransferFunction::default();
        let gpu_transfer_function =
            GPUTransferFunction::new_texture_1d_rgbt(&transfer_function, &ctx.device, &ctx.queue);

        // Shader
        let shader_path =
            Path::new(&(format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"))))
                .to_path_buf();
        let extra_layout = layout_from_unbound_entries(
            ctx,
            "Extra Layout",
            &[
                GpuVolume::BIND_GROUP_LAYOUT_ENTRIES,
                GPUTransferFunction::BIND_GROUP_LAYOUT_ENTRIES,
                GpuImportances::BIND_GROUP_LAYOUT_ENTRIES,
            ],
        );
        let extra_bind_group = bindgroup_from_resources(
            ctx,
            "Extra Bind Group",
            &extra_layout,
            &[
                volume.to_gpu_resources(),
                gpu_transfer_function.to_gpu_resources(),
                importances.to_gpu_resources(),
            ],
        );

        let config = BaseDemoConfig {
            shader_path,
            output_texture,
            extra_bind_groups: vec![extra_bind_group],
            extra_layouts: vec![extra_layout],
        };

        let base = BaseDemo::init(ctx, state, config)?;

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
