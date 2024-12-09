use std::path::Path;

use demos::RenderingAlgorithm;
use tracing::{info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod context;
mod demos;
mod event_loop;
mod render_pipeline;

// Demo
use demos::simple::compute_pipeline;
use demos::simple::volume;
use demos::simple::SimpleRaycaster;

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;

fn main() -> Result<()> {
    setup_tracing()?;
    run::<SimpleRaycaster>()?;
    info!("Done");
    Ok(())
}

fn run<Algo: RenderingAlgorithm>() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;
    let mut ctx = pollster::block_on(context::Context::new(&window))?;
    let volume_path = format!(
        "{}/assets/bonsai_256x256x256_uint8.raw",
        env!("CARGO_MANIFEST_DIR")
    );
    let volume = volume::Volume::new(
        Path::new(&volume_path),
        volume::FlipMode::Y,
        &ctx.device,
        &ctx.queue,
    )?;
    let rendering_algorithm = Algo::init(&mut ctx, volume)?;
    event_loop::run(event_loop, &mut ctx, rendering_algorithm)?;
    Ok(())
}

fn setup_tracing() -> Result<()> {
    color_eyre::install()?;

    let s = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new(LevelFilter::INFO.to_string())),
        )
        .compact()
        .finish()
        .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(s)?;

    Ok(())
}
