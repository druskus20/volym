use cli::Command;
use cli::Demo;
use egui_winit::winit::{event_loop::EventLoop, window::WindowBuilder};
use gpu_context::GpuContext;
use gpu_resources::texture::GpuWriteTexture2D;
use render_pipeline::RenderPipeline;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

mod camera;
mod cli;
mod demos;
mod event_loop;
mod gpu_context;
mod gpu_resources;
mod gui_context;
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
    }
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
    let aspect = ctx.surface_config.width as f32 / ctx.surface_config.height as f32;
    let mut state = state::State::new(aspect);

    // Setup render pipeline and compute demo.
    let compute_output_texture = GpuWriteTexture2D::new(&ctx);
    let compute_demo = ComputeDemo::init(&ctx, &state, &compute_output_texture)?;

    let render_input_texture = compute_output_texture.into_write_texture_2d(&ctx);
    let render_pipeline = RenderPipeline::init(&ctx, &render_input_texture)?;

    let mut egui = gui_context::EguiContext::new(
        &ctx.device,               // wgpu Device
        ctx.surface_config.format, // TextureFormat
        None,                      // this can be None
        1,                         // samples
        &window,                   // winit Window
    );

    event_loop::run(
        event_loop,
        ctx,
        &mut state,
        render_pipeline,
        &compute_demo,
        &mut egui,
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
