use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod context;
mod demos;
mod event_loop;
mod render_pipeline;

// Demos
use demos::simple::compute_pipeline;
use demos::simple::volume;
use demos::simple::Simple;

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;

fn main() -> Result<()> {
    setup_tracing()?;
    run::<Simple>()?;
    Ok(())
}

fn run<Demo: demos::RenderingDemo>() -> Result<()> {
    // Setup event loop and window.
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;

    // Create a rendering context
    let mut ctx = pollster::block_on(context::Context::new(&window))?;

    // Init and run the rendering demo
    let rendering_algorithm = Demo::init(&mut ctx)?;
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
