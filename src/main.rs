use std::time::Duration;

use cgmath::Point3;
use cli::{Command, Demo};
use egui::scroll_area::State;
use egui_winit::winit::{
    self,
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
use event_loop::EventLoopEx;
use gpu_context::GpuContext;
use gpu_resources::{parameters, texture::GpuWriteTexture2D};
use render_pipeline::RenderPipeline;
use state::StateParameters;
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
        Command::Benchmark => benchmark_all(),
    }
}

#[derive(Debug, Clone, Copy)]
enum BenchmarkMsg {
    Stop,
}

#[derive(Debug, Clone, Copy)]
struct RunSettings {
    refresh_rate_sync: bool,
    secs_per_benchmark: u32,
}

impl Default for RunSettings {
    fn default() -> Self {
        Self {
            refresh_rate_sync: true,
            secs_per_benchmark: 5,
        }
    }
}

fn benchmark_all() -> Result<()> {
    // 3 different algorithms
    //
    // 0. base
    // 1. importance rendering
    // 2. imporatance rendering with cone projection
    //
    //
    // Paramters:
    // - density threshold
    // - opacity
    // - step-size
    // - gaussian smoothing
    // - check steps
    //
    // opacity is needed for importance rendering
    // importance rendering is neeed for importance_check_ahead_steps
    let parameters = StateParameters {
        camera_position: Point3::new(0.5, 0.5, 3.5),
        use_opacity: false,
        density_trheshold: 0.15,
        use_cone_importance_check: false,
        use_importance_coloring: false,
        use_importance_rendering: false,
        use_gaussian_smoothing: false,
        importance_check_ahead_steps: 15,
        raymarching_step_size: 0.020,
    };

    let parameters = StateParameters::default();

    benchmark::<Simple>(parameters)?;

    Ok(())
}
fn benchmark<ComputeDemo: demos::ComputeDemo>(parameters: StateParameters) -> Result<()> {
    let settings = RunSettings {
        refresh_rate_sync: false,
        ..RunSettings::default()
    };

    let event_loop = EventLoopBuilder::<BenchmarkMsg>::with_user_event().build()?;
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024, 768))
        .with_title("Volym")
        .build(&event_loop)?;

    let user_event_handler: fn(BenchmarkMsg, &EventLoopWindowTarget<BenchmarkMsg>) =
        |event, control_flow| {
            if let BenchmarkMsg::Stop = event {
                info!("Benchmark finished");
                control_flow.exit();
            }
        };

    let sleep_t = Duration::from_secs(settings.secs_per_benchmark as u64);
    std::thread::spawn(move || {
        std::thread::sleep(sleep_t);
        event_loop_proxy.send_event(BenchmarkMsg::Stop).unwrap();
    });

    run_with_event_loop::<Simple, BenchmarkMsg>(
        window,
        parameters,
        settings,
        event_loop,
        user_event_handler,
    )?;

    Ok(())
}

fn run<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    let event_loop = EventLoop::<()>::new()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .with_inner_size(winit::dpi::PhysicalSize::new(1400, 768))
        .build(&event_loop)?;

    run_with_event_loop::<ComputeDemo, ()>(
        window,
        StateParameters::default(),
        RunSettings::default(),
        event_loop,
        |_, _| {},
    )
}

fn run_with_event_loop<ComputeDemo: demos::ComputeDemo, UserEvent: std::fmt::Debug>(
    window: Window,
    state_parameters: StateParameters,
    settings: RunSettings,
    event_loop: EventLoop<UserEvent>,
    user_event_handler: impl FnMut(UserEvent, &EventLoopWindowTarget<UserEvent>),
) -> Result<()> {
    // ctx needs to be independent to be moved into the event loop
    let ctx = pollster::block_on(GpuContext::new(&window))?;

    // state needs to be mutable - thus separate from ctx
    let mut state = state::State::with_parameters(
        ctx.surface_config.width as f32 / ctx.surface_config.height as f32,
        state_parameters,
    );

    // Setup render pipeline and compute demo.
    let compute_output_texture = GpuWriteTexture2D::new(&ctx);
    let compute_demo = ComputeDemo::init(&ctx, &state, &compute_output_texture)?;

    let render_input_texture = compute_output_texture.into_read_texture_2d(&ctx);
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
        &render_input_texture,
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
