use tracing::debug;
use winit::event::{ElementState, MouseButton, WindowEvent};

use crate::camera::{Camera, CameraController};

#[derive(Debug)]
pub struct State {
    pub camera: Camera,
    pub camera_controller: CameraController,
    mouse_pressed: bool,
    last_mouse_position: Option<(f64, f64)>,
}

impl State {
    pub fn new(aspect: f32) -> Self {
        let camera = crate::camera::Camera::new(aspect);
        Self {
            camera,
            camera_controller: CameraController::new(0.2, 0.2),
            mouse_pressed: false,
            last_mouse_position: None,
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
