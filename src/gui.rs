use egui::epaint::Shadow;
use egui::{Align2, Context, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{CommandEncoder, Device, Queue, TextureFormat, TextureView};
use egui_winit::winit::event::WindowEvent;
use egui_winit::winit::window::Window;
use egui_winit::State as EguiState;

use crate::gpu_context::GpuContext;
use crate::state::State;

pub struct GuiContext {
    pub egui_ctx: Context,
    pub state: EguiState,
    renderer: Renderer,
}

impl GuiContext {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> GuiContext {
        let egui_context = Context::default();
        let id = egui_context.viewport_id();

        const BORDER_RADIUS: f32 = 2.0;

        let visuals = Visuals {
            window_rounding: egui::Rounding::same(BORDER_RADIUS),
            window_shadow: Shadow::NONE,
            // menu_rounding: todo!(),
            ..Default::default()
        };

        egui_context.set_visuals(visuals);

        let egui_state = EguiState::new(egui_context.clone(), id, &window, None, None);

        // egui_state.set_pixels_per_point(window.scale_factor() as f32);
        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
        );

        GuiContext {
            egui_ctx: egui_context,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let r = self.state.on_window_event(window, event);

        if r.consumed == true {
            dbg!(event);
        };

        // todo!()

        false
    }

    pub fn draw(
        &mut self,
        ctx: &GpuContext,
        state: &State,
        window_surface_view: &TextureView,
        screen_descriptor: ScreenDescriptor,
    ) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        let window = &ctx.window;
        let queue = &ctx.queue;
        let device = &ctx.device;
        {
            // self.state.set_pixels_per_point(window.scale_factor() as f32);
            let raw_input = self.state.take_egui_input(window);
            let full_output = self.egui_ctx.run(raw_input, |ui| {
                egui::Window::new("Streamline CFD")
                    //.vscroll(true)
                    .default_open(true)
                    .max_width(100.0)
                    .max_height(800.0)
                    .default_width(800.0)
                    .resizable(false)
                    .anchor(Align2::LEFT_TOP, [10.0, 10.0])
                    .show(ui, |ui| {
                        if ui.add(egui::Button::new("Click me")).clicked() {
                            println!("PRESSED")
                        }

                        ui.label("Slider");
                        ui.end_row();
                    });
            });

            self.state
                .handle_platform_output(window, full_output.platform_output);

            let tris = self
                .egui_ctx
                .tessellate(full_output.shapes, full_output.pixels_per_point);
            for (id, image_delta) in &full_output.textures_delta.set {
                self.renderer
                    .update_texture(device, queue, *id, image_delta);
            }
            self.renderer
                .update_buffers(device, queue, &mut encoder, &tris, &screen_descriptor);
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: window_surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),

                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            self.renderer.render(&mut rpass, &tris, &screen_descriptor);
            drop(rpass);
            for x in &full_output.textures_delta.free {
                self.renderer.free_texture(x)
            }
        }
        ctx.queue.submit(Some(encoder.finish()));
    }
}
