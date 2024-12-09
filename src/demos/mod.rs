use crate::context;
use crate::Result;

pub mod simple;

pub trait RenderingDemo: Sized {
    fn init(ctx: &mut context::Context) -> Result<Self>;
    fn compute(&self, ctx: &mut context::Context) -> Result<()>;
}
