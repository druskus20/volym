use std::time::Duration;

use cli::{Command, Demo};
use egui_winit::winit::{
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
use event_loop::EventLoopEx;
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
mod gui;
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
        Command::Benchmark => benchmark::<Simple>(),
    }
}

#[derive(Debug, Clone, Copy)]
enum BenchmarkMsg {
    Stop,
}

#[derive(Debug, Clone, Copy)]
struct Settings {
    refresh_rate_sync: bool,
    secs_per_benchmark: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_rate_sync: true,
            secs_per_benchmark: 10,
        }
    }
}

fn benchmark<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    let settings = Settings {
        refresh_rate_sync: false,
        ..Settings::default()
    };

    let t = Duration::from_secs(settings.secs_per_benchmark as u64);

    let event_loop = EventLoopBuilder::<BenchmarkMsg>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;

    // spawn a thread that will close the windo in `t` seconds
    std::thread::spawn(move || {
        std::thread::sleep(t);
        event_loop_proxy.send_event(BenchmarkMsg::Stop).unwrap();
    });

    let user_event_handler: fn(BenchmarkMsg, &EventLoopWindowTarget<BenchmarkMsg>) =
        |event, control_flow| match event {
            BenchmarkMsg::Stop => {
                info!("Benchmark finished");
                control_flow.exit();
            }
            _ => (),
        };

    run_with_event_loop::<Simple, BenchmarkMsg>(window, event_loop, user_event_handler, settings)?;

    Ok(())
}

fn run<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    let event_loop = EventLoop::<()>::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .build(&event_loop)?;
    run_with_event_loop::<ComputeDemo, ()>(window, event_loop, |_, _| {}, Settings::default())
}

fn run_with_event_loop<ComputeDemo: demos::ComputeDemo, UserEvent: std::fmt::Debug>(
    window: Window,
    event_loop: EventLoop<UserEvent>,
    user_event_handler: impl FnMut(UserEvent, &EventLoopWindowTarget<UserEvent>),
    settings: Settings,
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

    let mut egui = gui::GuiContext::new(
        &ctx.device,               // wgpu Device
        ctx.surface_config.format, // TextureFormat
        None,                      // this can be None
        1,                         // samples
        &window,                   // winit Window
    );

    event_loop.run_volym(
        settings,
        ctx,
        &mut state,
        &render_pipeline,
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
