use egui_winit::winit::{
    event::*,
    event_loop::{EventLoop, EventLoopWindowTarget},
    keyboard::{KeyCode, PhysicalKey},
    platform::run_on_demand::EventLoopExtRunOnDemand,
};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use egui_wgpu::{wgpu, ScreenDescriptor};

use crate::{
    demos::ComputeDemo,
    gpu_context::GpuContext,
    gpu_resources::texture::{GpuReadTexture2D, GpuWriteTexture2D},
    gui::GuiContext,
    render_pipeline::RenderPipeline,
    state::State,
    Result, RunSettings,
};

pub trait EventLoopEx {
    type EventLoopUserMsg: std::fmt::Debug;
    fn run_volym(
        &mut self,
        settings: RunSettings,
        ctx: GpuContext,
        state: &mut State,
        render_pipeline: &RenderPipeline,
        demo: &impl ComputeDemo,
        egui: &mut GuiContext,
        user_event_handler: impl FnMut(
            Self::EventLoopUserMsg,
            &EventLoopWindowTarget<Self::EventLoopUserMsg>,
        ),
        render_iput_texture: &GpuReadTexture2D,
    ) -> Result<(u32, Duration)>;
}

impl<T: std::fmt::Debug> EventLoopEx for EventLoop<T> {
    type EventLoopUserMsg = T;
    #[tracing::instrument(skip_all)]
    fn run_volym(
        &mut self,
        settings: RunSettings,
        mut ctx: GpuContext,
        state: &mut State,
        render_pipeline: &RenderPipeline,
        demo: &impl ComputeDemo,
        egui: &mut GuiContext,
        mut user_event_handler: impl FnMut(
            Self::EventLoopUserMsg,
            &EventLoopWindowTarget<Self::EventLoopUserMsg>,
        ),
        render_input_texture: &GpuReadTexture2D,
    ) -> Result<(u32, Duration)> {
        let mut first_update = Instant::now();
        let mut last_update = first_update;
        let mut frame_count: u32 = 0;
        let mut total_frames: u32 = 0;

        self.run_on_demand(move |event, control_flow| {
            debug!(target = "Render loop", "received event {:?}", event);
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == ctx.window().id() => {
                    let is_egui_event = egui.process_input(ctx.window, event);
                    if is_egui_event {
                        return;
                    }
                    let is_volym_event =
                        state.process_input(&ctx, &render_input_texture.texture, event);
                    if is_volym_event {
                        return;
                    }

                    if !is_egui_event && !is_volym_event {
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
                                    let screen_descriptor = ScreenDescriptor {
                                        size_in_pixels: [
                                            ctx.surface_config.width,
                                            ctx.surface_config.height,
                                        ],
                                        pixels_per_point: ctx.window().scale_factor() as f32,
                                    };

                                    let output = ctx.surface.get_current_texture().unwrap();
                                    let view = output
                                        .texture
                                        .create_view(&wgpu::TextureViewDescriptor::default());

                                    demo.compute_pass(&ctx).unwrap();
                                    let r = render_pipeline.render_pass(&ctx, &view);
                                    egui.draw(&ctx, state, &view, screen_descriptor);

                                    // Before presenting to the screen we need to let the compositor know - This effectively
                                    // syncs us to the monitor refresh rate.
                                    // https://docs.rs/winit/latest/winit/window/struct.Window.html#platform-specific-2
                                    if settings.refresh_rate_sync {
                                        ctx.window.pre_present_notify();
                                    }

                                    output.present();
                                    r
                                };

                                match render_result {
                                    Ok(_) => {
                                        frame_count += 1;
                                        total_frames += 1;
                                        let now = Instant::now();
                                        if now.duration_since(last_update) >= Duration::from_secs(1)
                                        {
                                            info!("FPS: {}", frame_count);
                                            frame_count = 0;
                                            last_update = now;
                                        }
                                    }
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => ctx.resize(ctx.size),
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
                Event::UserEvent(e) => user_event_handler(e, control_flow),
                _ => {}
            }
        })?;

        return Ok((total_frames, Instant::now().duration_since(first_update)));
    }
}
