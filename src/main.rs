use std::time::Duration;

use cgmath::Point3;
use cli::{Command, Demo};
use csv::Writer;
use egui::scroll_area::State;
use egui_winit::winit::{
    self, event,
    event_loop::{EventLoop, EventLoopBuilder, EventLoopWindowTarget},
    window::{Window, WindowBuilder},
};
use event_loop::EventLoopEx;
use gpu_context::GpuContext;
use gpu_resources::{parameters, texture::GpuWriteTexture2D};
use render_pipeline::RenderPipeline;
use serde::Serialize;
use state::StateParameters;
use std::fs::File;
use std::io::Write;
use std::time::Instant;
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
enum EventLoopUserMsg {
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
            secs_per_benchmark: 2,
        }
    }
}

#[derive(Serialize)]
struct BenchmarkResult {
    algorithm: String,
    step_size: f32,
    importance_steps: u32,
    use_cone: bool,
    avg_total_frames: f64,
    avg_total_time_ms: f64,
    avg_frame_time_ms: f64,
    avg_fps: f64,
    std_dev_total_frames: f64,
    std_dev_total_time_ms: f64,
    std_dev_frame_time_ms: f64,
    std_dev_fps: f64,
}

#[derive(Default)]
struct TrialResults {
    total_frames: Vec<u32>,
    total_times_ms: Vec<u64>,
    frame_times_ms: Vec<f64>,
    fps_values: Vec<f64>,
}

impl TrialResults {
    fn new() -> Self {
        Self::default()
    }

    fn add_trial(&mut self, frames: u32, duration: Duration) {
        let time_ms = duration.as_millis() as u64;
        let frame_time_ms = duration.as_secs_f64() * 1000.0 / frames as f64;
        let fps = frames as f64 / duration.as_secs_f64();

        self.total_frames.push(frames);
        self.total_times_ms.push(time_ms);
        self.frame_times_ms.push(frame_time_ms);
        self.fps_values.push(fps);
    }

    fn calculate_stats(&self) -> (f64, f64, f64, f64, f64, f64, f64, f64) {
        fn mean_u32(values: &[u32]) -> f64 {
            values.iter().map(|&x| f64::from(x)).sum::<f64>() / values.len() as f64
        }

        fn mean_u64(values: &[u64]) -> f64 {
            values.iter().map(|&x| x as f64).sum::<f64>() / values.len() as f64
        }

        fn mean_f64(values: &[f64]) -> f64 {
            values.iter().sum::<f64>() / values.len() as f64
        }

        fn std_dev_u32(values: &[u32], mean: f64) -> f64 {
            let variance = values
                .iter()
                .map(|&x| {
                    let diff = f64::from(x) - mean;
                    diff * diff
                })
                .sum::<f64>()
                / values.len() as f64;
            variance.sqrt()
        }

        fn std_dev_u64(values: &[u64], mean: f64) -> f64 {
            let variance = values
                .iter()
                .map(|&x| {
                    let diff = x as f64 - mean;
                    diff * diff
                })
                .sum::<f64>()
                / values.len() as f64;
            variance.sqrt()
        }

        fn std_dev_f64(values: &[f64], mean: f64) -> f64 {
            let variance = values
                .iter()
                .map(|&x| {
                    let diff = x - mean;
                    diff * diff
                })
                .sum::<f64>()
                / values.len() as f64;
            variance.sqrt()
        }

        let avg_frames = mean_u32(&self.total_frames);
        let avg_time = mean_u64(&self.total_times_ms);
        let avg_frame_time = mean_f64(&self.frame_times_ms);
        let avg_fps = mean_f64(&self.fps_values);

        (
            avg_frames,
            avg_time,
            avg_frame_time,
            avg_fps,
            std_dev_u32(&self.total_frames, avg_frames),
            std_dev_u64(&self.total_times_ms, avg_time),
            std_dev_f64(&self.frame_times_ms, avg_frame_time),
            std_dev_f64(&self.fps_values, avg_fps),
        )
    }
}

fn benchmark_all() -> Result<()> {
    const NUM_TRIALS: usize = 3;
    let base_parameters = StateParameters {
        camera_position: Point3::new(0.5, 0.5, 3.5),
        use_opacity: true,
        density_trheshold: 0.15,
        use_cone_importance_check: false,
        use_importance_coloring: false,
        use_importance_rendering: false,
        use_gaussian_smoothing: false,
        importance_check_ahead_steps: 15,
        raymarching_step_size: 0.020,
    };

    let step_sizes = [0.0030, 0.0050, 0.0100, 0.0200];
    let importance_steps = [10, 15, 20];
    let mut results = Vec::new();
    let mut event_loop = EventLoopBuilder::<EventLoopUserMsg>::with_user_event().build()?;

    info!("Running base algorithm benchmarks");
    for &step_size in &step_sizes {
        let mut trial_results = TrialResults::new();

        for trial in 0..NUM_TRIALS {
            info!(
                "Base algorithm trial {} with step_size {}",
                trial + 1,
                step_size
            );
            let mut params = base_parameters.clone();
            params.raymarching_step_size = step_size;
            let (total_frames, duration) = benchmark::<Simple>(&mut event_loop, params)?;
            trial_results.add_trial(total_frames, duration);
        }

        let (
            avg_frames,
            avg_time,
            avg_frame_time,
            avg_fps,
            std_frames,
            std_time,
            std_frame_time,
            std_fps,
        ) = trial_results.calculate_stats();

        results.push(BenchmarkResult {
            algorithm: "Base".to_string(),
            step_size,
            importance_steps: 0,
            use_cone: false,
            avg_total_frames: avg_frames,
            avg_total_time_ms: avg_time,
            avg_frame_time_ms: avg_frame_time,
            avg_fps: avg_fps,
            std_dev_total_frames: std_frames,
            std_dev_total_time_ms: std_time,
            std_dev_frame_time_ms: std_frame_time,
            std_dev_fps: std_fps,
        });
    }

    info!("Running importance rendering benchmarks");
    for &step_size in &step_sizes {
        for &importance_step in &importance_steps {
            let mut trial_results = TrialResults::new();

            for trial in 0..NUM_TRIALS {
                info!(
                    "Importance rendering trial {} with step_size {} and importance_step {}",
                    trial + 1,
                    step_size,
                    importance_step
                );
                let mut params = base_parameters.clone();
                params.raymarching_step_size = step_size;
                params.importance_check_ahead_steps = importance_step;
                params.use_importance_rendering = true;
                let (total_frames, duration) = benchmark::<Simple>(&mut event_loop, params)?;
                trial_results.add_trial(total_frames, duration);
            }

            let (
                avg_frames,
                avg_time,
                avg_frame_time,
                avg_fps,
                std_frames,
                std_time,
                std_frame_time,
                std_fps,
            ) = trial_results.calculate_stats();

            results.push(BenchmarkResult {
                algorithm: "Importance".to_string(),
                step_size,
                importance_steps: importance_step,
                use_cone: false,
                avg_total_frames: avg_frames,
                avg_total_time_ms: avg_time,
                avg_frame_time_ms: avg_frame_time,
                avg_fps: avg_fps,
                std_dev_total_frames: std_frames,
                std_dev_total_time_ms: std_time,
                std_dev_frame_time_ms: std_frame_time,
                std_dev_fps: std_fps,
            });
        }
    }

    info!("Running importance rendering with cone projection benchmarks");
    for &step_size in &step_sizes {
        for &importance_step in &importance_steps {
            let mut trial_results = TrialResults::new();

            for trial in 0..NUM_TRIALS {
                info!(
                    "Importance cone rendering trial {} with step_size {} and importance_step {}",
                    trial + 1,
                    step_size,
                    importance_step
                );
                let mut params = base_parameters.clone();
                params.raymarching_step_size = step_size;
                params.importance_check_ahead_steps = importance_step;
                params.use_importance_rendering = true;
                params.use_cone_importance_check = true;
                let (total_frames, duration) = benchmark::<Simple>(&mut event_loop, params)?;
                trial_results.add_trial(total_frames, duration);
            }

            let (
                avg_frames,
                avg_time,
                avg_frame_time,
                avg_fps,
                std_frames,
                std_time,
                std_frame_time,
                std_fps,
            ) = trial_results.calculate_stats();

            results.push(BenchmarkResult {
                algorithm: "ImportanceCone".to_string(),
                step_size,
                importance_steps: importance_step,
                use_cone: true,
                avg_total_frames: avg_frames,
                avg_total_time_ms: avg_time,
                avg_frame_time_ms: avg_frame_time,
                avg_fps: avg_fps,
                std_dev_total_frames: std_frames,
                std_dev_total_time_ms: std_time,
                std_dev_frame_time_ms: std_frame_time,
                std_dev_fps: std_fps,
            });
        }
    }

    // Write results to CSV
    let mut wtr = Writer::from_path("benchmark_results.csv")?;
    for result in results {
        wtr.serialize(result)?;
    }
    wtr.flush()?;

    Ok(())
}

fn benchmark<ComputeDemo: demos::ComputeDemo>(
    event_loop: &mut EventLoop<EventLoopUserMsg>,
    parameters: StateParameters,
) -> Result<(u32, Duration)> {
    let settings = RunSettings {
        refresh_rate_sync: false,
        ..RunSettings::default()
    };
    let event_loop_proxy = event_loop.create_proxy();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::PhysicalSize::new(1024, 768))
        .with_title("Volym")
        .build(&event_loop)?;

    let user_event_handler: fn(EventLoopUserMsg, &EventLoopWindowTarget<EventLoopUserMsg>) =
        |event, control_flow| {
            if let EventLoopUserMsg::Stop = event {
                info!("Benchmark finished");
                control_flow.exit();
            }
        };

    let sleep_t = Duration::from_secs(settings.secs_per_benchmark as u64);
    std::thread::spawn(move || {
        std::thread::sleep(sleep_t);
        event_loop_proxy.send_event(EventLoopUserMsg::Stop).unwrap();
    });

    let (total_frames, duration) = run_with_event_loop::<Simple>(
        window,
        parameters,
        settings,
        event_loop,
        user_event_handler,
    )?;

    Ok((total_frames, duration))
}

fn run<ComputeDemo: demos::ComputeDemo>() -> Result<()> {
    let mut event_loop = EventLoopBuilder::<EventLoopUserMsg>::with_user_event().build()?;
    let window = WindowBuilder::new()
        .with_title("Volym")
        .with_inner_size(winit::dpi::PhysicalSize::new(1200, 768))
        .build(&event_loop)?;

    let _ = run_with_event_loop::<ComputeDemo>(
        window,
        StateParameters::default(),
        RunSettings::default(),
        &mut event_loop,
        |_, _| {},
    )?;

    Ok(())
}

fn run_with_event_loop<ComputeDemo: demos::ComputeDemo>(
    window: Window,
    state_parameters: StateParameters,
    settings: RunSettings,
    event_loop: &mut EventLoop<EventLoopUserMsg>,
    user_event_handler: impl FnMut(EventLoopUserMsg, &EventLoopWindowTarget<EventLoopUserMsg>),
) -> Result<(u32, Duration)> {
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

    let (total_frames, duration) = event_loop.run_volym(
        settings,
        ctx,
        &mut state,
        &render_pipeline,
        &compute_demo,
        &mut egui,
        user_event_handler,
        &render_input_texture,
    )?;

    Ok((total_frames, duration))
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
