/// Event loop that handles window events and triggers rendering
use tracing::{debug, error, info, warn};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::Result;

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut ctx = crate::context::Context::new(&window).await?;
    info!("Renderer initialized");

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
                            // This tells winit that we want another frame after this one
                            ctx.window().request_redraw();

                            ctx.update();
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
