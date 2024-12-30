use crate::gpu_context::GpuContext;
use crate::state::State;
use crate::Result;

use egui_wgpu::wgpu;

pub mod pipeline;
pub mod simple;

pub trait ComputeDemo: Sized {
    fn init(ctx: &GpuContext, state: &State, output_texture_view: &wgpu::Texture) -> Result<Self>;
    fn update_gpu_state(&self, ctx: &GpuContext, state: &State) -> Result<()>;
    fn compute_pass(&self, ctx: &GpuContext) -> Result<()>;
}
