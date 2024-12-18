use crate::rendering_context::Context;
use crate::state::State;
use crate::Result;

pub mod compute_base;
pub mod simple;

pub trait Demo: Sized {
    fn init(ctx: &Context, state: &State, output_texture_view: &wgpu::TextureView) -> Result<Self>;
    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()>;
    fn compute_pass(&self, ctx: &Context) -> Result<()>;
}
