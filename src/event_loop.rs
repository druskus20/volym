/// Event loop that handles window events and triggers rendering
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{
    demos::ComputeDemo, render_pipeline::RenderPipeline, rendering_context::Context, state::State,
    Result,
};

#[tracing::instrument(skip_all)]
pub fn run<T: std::fmt::Debug>(
    event_loop: EventLoop<T>,
    mut ctx: Context,
    state: &mut State,
    render_pipeline: RenderPipeline,
    demo: &impl ComputeDemo,
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
                if !state.process_input(event) {
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
                            // queue another redraw
                            ctx.window().request_redraw();

                            // update the state
                            {
                                state.update();
                                demo.update_gpu_state(&ctx, state).unwrap();
                            }

                            //  compute and render
                            let render_result = {
                                demo.compute_pass(&ctx).unwrap();
                                render_pipeline.render_pass(&ctx)
                            };

                            match render_result {
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
