use cli::Command;
use cli::Demo;
use gpu_context::GpuContext;
use gpu_resources::texture::GpuWriteTexture2D;
use render_pipeline::RenderPipeline;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod camera;
mod cli;
mod demos;
mod event_loop;
mod gpu_context;
mod gpu_resources;
//mod gui_context;
mod render_pipeline;
mod state;
mod transfer_function;

// Demos
use demos::simple::Simple;

pub(crate) type Result<T> = color_eyre::eyre::Result<T>;
pub(crate) type Error = color_eyre::eyre::Report;

fn main() -> Result<()> {
    let args = cli::ParsedArgs::parse_args();
    setup_tracing(args.log_level.to_string())?;
    match args.command {
        Command::Run(Demo::Simple) => run::<Simple>(),
        Command::Benchmark => benchmark(),
    }
}

fn benchmark() -> Result<()> {
    Ok(())
}

fn run<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    // Hook window and event loop
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;

    // ctx needs to be independent to be moved into the event loop
    let ctx = pollster::block_on(GpuContext::new(&window))?;

    // state needs to be mutable - thus separate from ctx
    let mut state =
        state::State::new((ctx.surface_config.width / ctx.surface_config.height) as f32);

    // Setup render pipeline and compute demo.
    let compute_output_texture = GpuWriteTexture2D::new(&ctx);
    let compute_demo = ComputeDemo::init(&ctx, &state, &compute_output_texture)?;

    let render_input_texture = compute_output_texture.into_read_texture_2d(&ctx);
    let render_pipeline = RenderPipeline::init(&ctx, &render_input_texture)?;

    event_loop::run(
        event_loop,      // event loop of the window
        ctx,             // wgpu context
        &mut state,      // state for things like moving the camera
        render_pipeline, // render pipeline that draws a texture into the screen
        &compute_demo,   // compute based demo that draws into a texture
    )?;

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
