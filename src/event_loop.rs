use std::time::{Duration, Instant};
/// Event loop that handles window events and triggers rendering
use tracing::{debug, error, info, warn};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{context::Context, RenderingAlgorithm, Result};

#[tracing::instrument(skip(event_loop, ctx, rendering_algorithm))]
pub fn run<T: std::fmt::Debug>(
    event_loop: EventLoop<T>,
    ctx: &mut Context,
    rendering_algorithm: impl RenderingAlgorithm,
) -> Result<()> {
    let mut last_update = Instant::now();
    let mut frame_count = 0;

    event_loop.run(move |event, control_flow| {
        debug!(target = "Render loop", "received event {:?}", event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == ctx.window().id() => {
                if !ctx.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            ctx.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            debug!("Redraw requested");
                            ctx.window().request_redraw();

                            ctx.update();
                            rendering_algorithm.compute(ctx).unwrap();
                            match ctx.render() {
                                Ok(_) => {
                                    frame_count += 1;
                                    let now = Instant::now();
                                    if now.duration_since(last_update) >= Duration::from_secs(1) {
                                        info!("FPS: {}", frame_count);
                                        frame_count = 0;
                                        last_update = now;
                                    }
                                }
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    ctx.resize(ctx.size)
                                }
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    error!("Out of memory");
                                    control_flow.exit();
                                }
                                Err(wgpu::SurfaceError::Timeout) => {
                                    warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    })?;
    Ok(())
}
