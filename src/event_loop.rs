use tracing::{error, info, warn};
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

    let mut ctx = crate::renderer::Context::new(&window).await?;
    info!("Renderer initialized");

    event_loop.run(move |event, control_flow| {
        info!(target = "render loop", "received event {:?}", event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == ctx.window().id() => {
                if !ctx.input(event) {
                    // UPDATED!
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
                                // Reconfigure the surface if it's lost or outdated
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    ctx.resize(ctx.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    error!("Out of memory");
                                    control_flow.exit();
                                }

                                // This happens when the a frame takes too long to present
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
