/// Event loop that handles window events and triggers rendering
use tracing::{debug, error, warn};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{context::Context, RenderingAlgorithm, Result};

pub fn run<T: std::fmt::Debug>(
    event_loop: EventLoop<T>,
    ctx: &mut Context,
    rendering_algorithm: impl RenderingAlgorithm,
) -> Result<()> {
    event_loop.run(move |event, control_flow| {
        debug!(target = "render loop", "received event {:?}", event);
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
                            // This tells winit that we want another frame after this one
                            ctx.window().request_redraw(); // <-- TODO: This should probably be done
                                                           // somehow else - maybe everytime the
                                                           // model is updated?

                            ctx.update();
                            rendering_algorithm.compute(ctx).unwrap();
                            match ctx.render() {
                                Ok(_) => {}
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
