use crate::gpu_context::GpuContext;
use crate::gpu_resources::texture::GpuWriteTexture2D;
use crate::state::State;
use crate::Result;

pub mod pipeline;
pub mod simple;

pub trait ComputeDemo: Sized {
    fn init(
        ctx: &GpuContext,
        state: &State,
        output_texture_view: &GpuWriteTexture2D,
    ) -> Result<Self>;
    fn update_gpu_state(&self, ctx: &GpuContext, state: &State) -> Result<()>;
    fn compute_pass(&self, ctx: &GpuContext) -> Result<()>;
}
