use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, EuclideanSpace, Matrix4, Point3, Vector3};
use wgpu::{util::DeviceExt, Buffer, Device, Queue};
use winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

#[derive(Debug)]
#[repr(C)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub target: Vector3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub horizontal_angle: f32,
    pub vertical_angle: f32,
    pub distance: f32,
    uniforms: CameraUniforms,
    pub buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct CameraUniforms {
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: f32,
}

impl Camera {
    pub const DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

    pub fn new(aspect: f32, device: &Device) -> Self {
        let position = Vector3::new(0.5, 0.5, 0.5);
        let target = Vector3::new(0.5, 0.5, 0.5);
        let up = Vector3::new(0.0, 1.0, 0.0);
        let fovy: f32 = 90.0;
        let aspect: f32 = aspect;
        let znear: f32 = 0.001;
        let zfar: f32 = 1000000.0;

        let uniforms = CameraUniforms {
            view_matrix: view_matrix(position, target, up).into(),
            projection_matrix: projection_matrix(fovy, aspect, znear, zfar).into(),
            camera_position: position.into(),
            _padding: 0.0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &device.create_bind_group_layout(&Self::DESC),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

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
            uniforms,
            buffer,
            bind_group,
        }
    }

    pub fn orbit(&mut self, horizontal_delta: f32, vertical_delta: f32, zoom_delta: f32) {
        self.horizontal_angle += horizontal_delta;
        self.vertical_angle = (self.vertical_angle + vertical_delta).clamp(-89.0, 89.0); // Prevent gimbal lock

        self.distance = (self.distance + zoom_delta).clamp(1.0, 10.0);

        let h_rad = self.horizontal_angle.to_radians();
        let v_rad = self.vertical_angle.to_radians();

        self.position = Vector3::new(
            self.target.x + self.distance * h_rad.sin() * v_rad.cos(),
            self.target.y + self.distance * v_rad.sin(),
            self.target.z + self.distance * h_rad.cos() * v_rad.cos(),
        );
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        view_matrix(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Matrix4<f32> {
        projection_matrix(self.fovy, self.aspect, self.znear, self.zfar)
    }

    pub fn update_buffer(&self, queue: &Queue, buffer: &Buffer) {
        let uniforms = CameraUniforms {
            view_matrix: self.view_matrix().into(),
            projection_matrix: self.projection_matrix().into(),
            camera_position: self.position.into(),
            _padding: 0.0,
        };

        queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

pub fn view_matrix(position: Vector3<f32>, target: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    Matrix4::look_at_rh(Point3::from_vec(position), Point3::from_vec(target), up)
}

pub fn projection_matrix(fovy: f32, aspect: f32, znear: f32, zfar: f32) -> Matrix4<f32> {
    perspective(Deg(fovy), aspect, znear, zfar)
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

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        camera.orbit(self.rotate_horizontal, self.rotate_vertical, self.scroll);

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;
        self.scroll = 0.0;
    }
}
