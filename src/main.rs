use cli::Command;
use cli::Demo;
use tracing::debug;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod camera;
mod cli;
mod demos;
mod event_loop;
mod render_pipeline;
mod rendering_context;
mod state;

// Demos
use demos::simple::Simple;

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;

fn main() -> Result<()> {
    let args = cli::ParsedArgs::parse_args();
    debug!("Parsed arguments: {:?}", args);

    setup_tracing(args.log_level.to_string())?;

    match args.command {
        Command::Run(Demo::Simple) => run::<Simple>(),
    }
}

fn run<Demo: demos::Demo>() -> Result<()> {
    // Setup event loop and window.
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;

    let ctx = pollster::block_on(rendering_context::Context::new(&window))?;
    let aspect = ctx.surface_config.width as f32 / ctx.surface_config.height as f32;
    let mut state = state::State::new(aspect);
    let render_pipeline = render_pipeline::RenderPipeline::new(&ctx.device, &ctx.surface_config)?;
    let output_texture_view = render_pipeline
        .input_texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let demo = Demo::init(&ctx, &state, &output_texture_view)?;
    event_loop::run(event_loop, ctx, &mut state, render_pipeline, &demo)?;

    Ok(())
}

fn setup_tracing(log_level: String) -> Result<()> {
    color_eyre::install()?;
    let s = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or(EnvFilter::new(log_level)),
        )
        .compact()
        .finish()
        .with(ErrorLayer::default());
    tracing::subscriber::set_global_default(s)?;
    Ok(())
}
