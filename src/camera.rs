use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

#[derive(Debug)]
#[repr(C)]
pub struct Camera {
    pub position: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub horizontal_angle: f32,
    pub vertical_angle: f32,
    pub distance: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        let position = Point3::new(0.5, 0.5, 0.5);
        let target = Point3::new(0.5, 0.5, 0.5);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let fovy: f32 = 90.0;
        let aspect: f32 = aspect;
        let znear: f32 = 0.001;
        let zfar: f32 = 1000000.0;

        Self {
            position,
            aspect,
            fovy,
            znear,
            zfar,
            horizontal_angle: 0.0,
            vertical_angle: 0.0,
            distance: 2.0,
            target,
            up,
        }
    }

    pub fn orbit(&mut self, horizontal_delta: f32, vertical_delta: f32, zoom_delta: f32) {
        self.horizontal_angle += horizontal_delta;
        self.vertical_angle = (self.vertical_angle + vertical_delta).clamp(-89.0, 89.0); // Prevent gimbal lock

        self.distance = (self.distance + zoom_delta).clamp(1.0, 10.0);

        let h_rad = self.horizontal_angle.to_radians();
        let v_rad = self.vertical_angle.to_radians();

        self.position = Point3::new(
            self.target.x + self.distance * h_rad.sin() * v_rad.cos(),
            self.target.y + self.distance * v_rad.sin(),
            self.target.z + self.distance * h_rad.cos() * v_rad.cos(),
        );
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        {
            Matrix4::look_at_rh(self.position, self.target, self.up)
        }
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        {
            perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar)
        }
    }
}

#[derive(Debug)]
pub struct CameraController {
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    sensitivity: f32,
    zoom_sensitivity: f32,
}

impl CameraController {
    pub fn new(sensitivity: f32, zoom_sensitivity: f32) -> Self {
        Self {
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            sensitivity,
            zoom_sensitivity,
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = -mouse_dx as f32 * self.sensitivity;
        self.rotate_vertical = -mouse_dy as f32 * self.sensitivity;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = match delta {
            MouseScrollDelta::LineDelta(_, scroll) => -scroll * self.zoom_sensitivity,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                -*scroll as f32 * self.zoom_sensitivity
            }
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        camera.orbit(self.rotate_horizontal, self.rotate_vertical, self.scroll);

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        self.scroll = 0.0;
    }
}
