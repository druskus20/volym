use egui_winit::winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
};
/// Event loop that handles window events and triggers rendering
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use egui_wgpu::{wgpu, ScreenDescriptor};

use crate::{
    demos::ComputeDemo, gpu_context::GpuContext, gui_context::EguiContext,
    render_pipeline::RenderPipeline, state::State, Result,
};

#[tracing::instrument(skip_all)]
pub fn run<T: std::fmt::Debug>(
    event_loop: EventLoop<T>,
    mut ctx: GpuContext,
    state: &mut State,
    render_pipeline: RenderPipeline,
    demo: &impl ComputeDemo,
    egui: &mut EguiContext,
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

                                let mut encoder = ctx.device.create_command_encoder(
                                    &wgpu::CommandEncoderDescriptor {
                                        label: Some("Render Encoder"),
                                    },
                                );
                                let screen_descriptor = ScreenDescriptor {
                                    size_in_pixels: [
                                        ctx.surface_config.width,
                                        ctx.surface_config.height,
                                    ],
                                    pixels_per_point: ctx.window().scale_factor() as f32,
                                };

                                //egui.draw(
                                //    &ctx.device,
                                //    &ctx.queue,
                                //    &mut encoder,
                                //    &ctx.window,
                                //    &view,
                                //    screen_descriptor,
                                //    |ui| GUI(ui),
                                //);

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
