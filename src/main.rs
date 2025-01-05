use std::time::Duration;

use cli::Command;
use cli::Demo;
use egui::Event;
use egui_winit::winit::event_loop::EventLoopBuilder;
use egui_winit::winit::event_loop::EventLoopWindowTarget;
use egui_winit::winit::window::Window;
use egui_winit::winit::{event_loop::EventLoop, window::WindowBuilder};
use gpu_context::GpuContext;
use gpu_resources::texture::GpuWriteTexture2D;
use render_pipeline::RenderPipeline;
use tracing::info;
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
        Command::Benchmark => run_benchmarks(),
    }
}

#[derive(Debug, Clone, Copy)]
enum EventLoopMsg {
    Stop,
}

fn run_benchmarks() -> Result<()> {
    let t = Duration::from_secs(10);

    let event_loop = EventLoopBuilder::<EventLoopMsg>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;

    // spawn a thread and pass the proxy
    std::thread::spawn(move || {
        std::thread::sleep(t);
        event_loop_proxy.send_event(EventLoopMsg::Stop).unwrap();
    });

    let mut user_event_handler: fn(EventLoopMsg, &EventLoopWindowTarget<EventLoopMsg>) =
        |event, control_flow| {
            if let EventLoopMsg::Stop = event {
                info!("Benchmark finished");
                control_flow.exit();
            }
        };
    run_with_event_loop::<Simple, EventLoopMsg>(window, event_loop, user_event_handler)?;

    Ok(())
}

fn run<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    let event_loop = EventLoop::<()>::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;
    run_with_event_loop::<ComputeDemo, ()>(window, event_loop, |_, _| {})
}

fn run_with_event_loop<ComputeDemo: demos::ComputeDemo, UserEvent: std::fmt::Debug>(
    window: Window,
    event_loop: EventLoop<UserEvent>,
    mut user_event_handler: impl FnMut(UserEvent, &EventLoopWindowTarget<UserEvent>),
) -> Result<()> {
    // ctx needs to be independent to be moved into the event loop
    let ctx = pollster::block_on(GpuContext::new(&window))?;

    // state needs to be mutable - thus separate from ctx
    let mut state =
        state::State::new((ctx.surface_config.width / ctx.surface_config.height) as f32);

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
        user_event_handler,
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
