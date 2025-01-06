use cgmath::Point3;
use egui_winit::winit::event::{ElementState, MouseButton, WindowEvent};
use tracing::debug;

use crate::camera::{Camera, CameraController};

#[derive(Debug)]
pub struct State {
    pub camera: Camera,
    pub camera_controller: CameraController,
    mouse_pressed: bool,
    last_mouse_position: Option<(f64, f64)>,
    pub transfer_points: Vec<(f32, egui::Color32)>,
    pub density_threshold: f32,
    pub use_importance_coloring: bool,
    pub use_cone_importance_check: bool,
    pub use_opacity: bool,
    pub use_importance_rendering: bool,
    pub use_gaussian_smoothing: bool,
}

#[derive(Debug)]
pub struct StateParameters {
    pub camera_position: Point3<f32>,
    pub density_trheshold: f32,
    pub use_cone_importance_check: bool,
    pub use_importance_coloring: bool,
    pub use_opacity: bool,
    pub use_importance_rendering: bool,
    pub use_gaussian_smoothing: bool,
}

impl Default for StateParameters {
    fn default() -> Self {
        Self {
            camera_position: Point3::new(0.5, 0.5, 0.5),
            use_cone_importance_check: false,
            use_importance_coloring: false,
            use_opacity: true,
            use_importance_rendering: true,
            density_trheshold: 0.12,
            use_gaussian_smoothing: true,
        }
    }
}

impl State {
    pub fn with_parameters(aspect: f32, parameters: StateParameters) -> Self {
        let camera =
            crate::camera::Camera::default_with_aspect_and_pos(aspect, parameters.camera_position);
        Self {
            camera,
            camera_controller: CameraController::new(0.2, 0.2),
            mouse_pressed: false,
            last_mouse_position: None,
            transfer_points: Vec::new(),
            density_threshold: parameters.density_trheshold,
            use_cone_importance_check: parameters.use_cone_importance_check,
            use_importance_coloring: parameters.use_importance_coloring,
            use_opacity: parameters.use_opacity,
            use_importance_rendering: parameters.use_importance_rendering,
            use_gaussian_smoothing: parameters.use_gaussian_smoothing,
        }
    }

    pub fn process_input(&mut self, event: &WindowEvent) -> bool {
        let r = match event {
            //WindowEvent::KeyboardInput {
            //    event:
            //        KeyEvent {
            //            physical_key: PhysicalKey::Code(key),
            //            state,
            //            ..
            //        },
            //    ..
            //} => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::CursorMoved { position, .. } => {
                if self.mouse_pressed {
                    let current_pos = (position.x, position.y);

                    // Calculate delta movement when mouse is pressed
                    if let Some(last_pos) = self.last_mouse_position {
                        let dx = current_pos.0 - last_pos.0;
                        let dy = current_pos.1 - last_pos.1;

                        // Use the existing process_mouse method
                        self.camera_controller.process_mouse(dx, dy);
                    }

                    // Update last mouse position
                    self.last_mouse_position = Some(current_pos);
                }
                true
            }
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        };

        if r {
            debug!(target = "input", "Processed event: {:?}", event);
        }
        r
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
    }
}
