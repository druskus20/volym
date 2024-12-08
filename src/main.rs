use std::path::Path;

use tracing::{info, level_filters::LevelFilter};
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod compute_pipeline;
mod context;
mod event_loop;
mod render_pipeline;
mod volume;

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
    let volume = volume::Volume::new(Path::new(&volume_path), &ctx.device, &ctx.queue)?;
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

struct SimpleRaycaster {
    volume: volume::Volume,                      // contains the bindgroup
    pipeline: compute_pipeline::ComputePipeline, // contains the bindgrouplayout
    compute_bind_group: wgpu::BindGroup,
}

pub trait RenderingAlgorithm: Sized {
    fn init(ctx: &mut context::Context, volume: volume::Volume) -> Result<Self>;
    fn validate(&self) -> Result<()>;
    fn compute(&self, ctx: &mut context::Context) -> Result<()>;
}

impl RenderingAlgorithm for SimpleRaycaster {
    fn init(ctx: &mut context::Context, volume: volume::Volume) -> Result<Self> {
        // todo: load volume

        let compute_path = format!(
            "{}/shaders/raycast_compute.wgsl",
            env!("CARGO_MANIFEST_DIR")
        );
        let pipeline =
            crate::compute_pipeline::ComputePipeline::new(&ctx.device, Path::new(&compute_path))?;
        let compute_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ctx.texture_view),
            }],
        });

        Ok(SimpleRaycaster {
            volume,
            pipeline,
            compute_bind_group,
        })
    }

    fn validate(&self) -> Result<()> {
        todo!()
    }

    fn compute(&self, ctx: &mut context::Context) -> Result<()> {
        let size = ctx.size;
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.pipeline.as_ref());

            compute_pass.set_bind_group(0, &self.volume.bind_group, &[]);
            compute_pass.set_bind_group(1, &self.compute_bind_group, &[]);

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgroup.
            compute_pass.dispatch_workgroups((size.width + 15) / 16, (size.height + 15) / 16, 1);
        }
        ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}
