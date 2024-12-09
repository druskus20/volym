use simple::volume;

use crate::context;
use crate::Result;

pub mod simple;

pub trait RenderingDemo: Sized {
    fn init(ctx: &mut context::Context, volume: volume::Volume) -> Result<Self>;
    fn compute(&self, ctx: &mut context::Context) -> Result<()>;
}
