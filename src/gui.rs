use egui::epaint::Shadow;
use egui::{Context, Visuals};
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;

use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{Device, TextureFormat, TextureView};
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
                    .default_width(200.0)
                    .default_height(300.0)
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

use egui::{Color32, RichText, Ui, Vec2};

fn show_ui(state: &mut State, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        // Camera Controls Section
        ui.add_space(4.0);
        egui::CollapsingHeader::new(RichText::new("📷 Camera Controls").heading().size(18.0))
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(8.0);

                // Position controls in a grid
                egui::Grid::new("camera_position_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        let mut pos = [
                            state.camera.position.x,
                            state.camera.position.y,
                            state.camera.position.z,
                        ];

                        for (i, (axis, val)) in
                            ["X", "Y", "Z"].iter().zip(pos.iter_mut()).enumerate()
                        {
                            ui.label(RichText::new(*axis).strong());
                            if ui
                                .add(
                                    egui::DragValue::new(val)
                                        .speed(0.1)
                                        .prefix(format!("{}: ", axis))
                                        .clamp_range(-100.0..=100.0),
                                )
                                .changed()
                            {
                                state.camera.position[i] = *val;
                            }
                            ui.end_row();
                        }
                    });

                ui.add_space(4.0);

                // Copy button with improved styling
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("📋 Copy Position").text_style(egui::TextStyle::Button),
                        )
                        .min_size(Vec2::new(120.0, 24.0)),
                    )
                    .clicked()
                {
                    let position_text = format!(
                        "{:.3}, {:.3}, {:.3}",
                        state.camera.position.x, state.camera.position.y, state.camera.position.z
                    );
                    ui.output_mut(|o| o.copied_text = position_text);
                }
            });

        // Rendering Settings Section
        ui.add_space(8.0);
        egui::CollapsingHeader::new(RichText::new("🎨 Rendering Settings").heading().size(18.0))
            .default_open(true)
            .show(ui, |ui| {
                ui.add_space(8.0);

                add_setting_group(ui, "Primary Controls", |ui| {
                    ui.checkbox(
                        &mut state.use_importance_coloring,
                        RichText::new("Importance Coloring").strong(),
                    )
                    .on_hover_text("Enable coloring based on importance values");

                    let opacity_response = ui
                        .add_enabled(
                            !state.use_importance_rendering,
                            egui::Checkbox::new(
                                &mut state.use_opacity,
                                RichText::new("Opacity").strong(),
                            ),
                        )
                        .on_hover_text("Control opacity of rendered elements");

                    if opacity_response.changed() && state.use_importance_rendering {
                        state.use_opacity = true;
                    }

                    // Importance rendering with dependencies
                    let imp_render_response = ui
                        .checkbox(
                            &mut state.use_importance_rendering,
                            RichText::new("Importance Rendering").strong(),
                        )
                        .on_hover_text("Enable advanced importance-based rendering");

                    if imp_render_response.changed() && state.use_importance_rendering {
                        state.use_opacity = true;
                    }
                });

                ui.add_space(8.0);

                add_setting_group(ui, "Advanced Controls", |ui| {
                    ui.add_enabled(
                        state.use_importance_rendering,
                        egui::Checkbox::new(
                            &mut state.use_cone_importance_check,
                            RichText::new("Cone Importance Check").strong(),
                        ),
                    )
                    .on_hover_text(
                        "Enable cone-based importance checking (requires Importance Rendering)",
                    );

                    ui.checkbox(
                        &mut state.use_gaussian_smoothing,
                        RichText::new("Gaussian Smoothing").strong(),
                    )
                    .on_hover_text("Apply Gaussian smoothing to the rendered output");
                });

                ui.add_space(8.0);

                // Parameters section
                add_setting_group(ui, "Parameters", |ui| {
                    ui.add_enabled(
                        state.use_importance_rendering,
                        egui::Slider::new(&mut state.importance_check_ahead_steps, 2..=25)
                            .text(RichText::new("Look Ahead Steps").strong())
                            .clamp_to_range(true),
                    )
                    .on_hover_text("Number of steps to look ahead for importance rendering");

                    ui.add(
                        egui::Slider::new(&mut state.raymarching_step_size, 0.001..=0.1)
                            .text(RichText::new("Raymarching Step Size").strong())
                            .logarithmic(true),
                    )
                    .on_hover_text("Size of steps used in raymarching algorithm");

                    ui.add(
                        egui::Slider::new(&mut state.density_threshold, 0.005..=1.0)
                            .text(RichText::new("Density Threshold").strong()),
                    )
                    .on_hover_text("Minimum density threshold for rendering");
                });
            });
    });
}

fn add_setting_group(ui: &mut Ui, title: &str, add_contents: impl FnOnce(&mut Ui)) {
    ui.group(|ui| {
        ui.label(RichText::new(title).size(14.0).color(Color32::LIGHT_BLUE));
        ui.add_space(4.0);
        add_contents(ui);
    });
}
