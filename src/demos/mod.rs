use crate::gpu_context::Context;
use crate::state::State;
use crate::Result;

pub mod pipeline;
pub mod simple;

pub trait ComputeDemo: Sized {
    fn init(ctx: &Context, state: &State, output_texture_view: &wgpu::Texture) -> Result<Self>;
    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()>;
    fn compute_pass(&self, ctx: &Context) -> Result<()>;
}

struct DemoBase {}
