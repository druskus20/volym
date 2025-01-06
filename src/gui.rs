use egui::epaint::Shadow;
use egui::{Align2, Color32, Context, Pos2, Rect, Vec2, Visuals};
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

    pub fn process_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        let r = self.state.on_window_event(window, event);
        r.consumed
    }

    pub fn draw(
        &mut self,
        ctx: &GpuContext,
        state: &mut State,
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
            let raw_input = self.state.take_egui_input(window);
            let full_output = self.egui_ctx.run(raw_input, |ui| {
                egui::Window::new("Volym")
                    .vscroll(true)
                    .default_open(true)
                    .default_width(300.0)
                    .default_height(170.0)
                    .resizable(true)
                    .show(ui, |ui| {
                        show_ui(state, ui);
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
                        load: wgpu::LoadOp::Load,
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

fn show_ui(state: &mut State, ui: &mut egui::Ui) {
    ui.heading("Camera Position");
    let mut pos = [
        state.camera.position.x,
        state.camera.position.y,
        state.camera.position.z,
    ];
    if ui
        .add(egui::DragValue::new(&mut pos[0]).prefix("X: ").speed(0.1))
        .changed()
    {
        state.camera.position.x = pos[0];
    }
    if ui
        .add(egui::DragValue::new(&mut pos[1]).prefix("Y: ").speed(0.1))
        .changed()
    {
        state.camera.position.y = pos[1];
    }
    if ui
        .add(egui::DragValue::new(&mut pos[2]).prefix("Z: ").speed(0.1))
        .changed()
    {
        state.camera.position.z = pos[2];
    }
    ui.add_space(10.0);
    // Add copy button
    if ui.button("📋 Copy Position").clicked() {
        let position_text = format!(
            "{:.3}, {:.3}, {:.3}",
            state.camera.position.x, state.camera.position.y, state.camera.position.z
        );
        ui.output_mut(|o| o.copied_text = position_text);
    }

    ui.add_space(10.0);
    ui.checkbox(&mut state.show_importance, "Show Importance Coloring");
    ui.add_space(10.0);
    //ui.heading("Transfer Function");
    //// Transfer function editor
    //let transfer_response = ui.allocate_rect(
    //    Rect::from_min_size(ui.cursor().min, Vec2::new(280.0, 100.0)),
    //    egui::Sense::drag(),
    //);
    //if let Some(pointer_pos) = transfer_response.hover_pos() {
    //    let normalized_x = (pointer_pos.x - transfer_response.rect.min.x)
    //        / transfer_response.rect.width();
    //    let normalized_y = 1.0
    //        - (pointer_pos.y - transfer_response.rect.min.y)
    //            / transfer_response.rect.height();
    //    if transfer_response.dragged() {
    //        state.transfer_points.push((
    //            normalized_x.clamp(0.0, 1.0),
    //            Color32::from_rgb(
    //                (normalized_y * 255.0) as u8,
    //                (normalized_y * 255.0) as u8,
    //                (normalized_y * 255.0) as u8,
    //            ),
    //        ));
    //    }
    //}
    //// Draw transfer function
    //let painter = ui.painter();
    //let rect = transfer_response.rect;
    //// Background
    //painter.rect_filled(rect, 0.0, Color32::from_gray(20));
    //// Draw points and lines
    //if state.transfer_points.len() >= 2 {
    //    let mut points: Vec<Pos2> = state
    //        .transfer_points
    //        .iter()
    //        .map(|(x, _)| Pos2::new(rect.min.x + x * rect.width(), rect.max.y))
    //        .collect();
    //    points.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    //    // Draw lines between points
    //    for points in points.windows(2) {
    //        painter.line_segment([points[0], points[1]], (1.0, Color32::WHITE));
    //    }
    //    // Draw points
    //    for point in points {
    //        painter.circle_filled(point, 3.0, Color32::WHITE);
    //    }
    //}
}
