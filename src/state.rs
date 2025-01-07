use crate::Result;
use cgmath::Point3;
use egui_wgpu::wgpu;
use egui_winit::winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};
use tracing::{debug, info};

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
    pub importance_check_ahead_steps: u32,
    pub raymarching_step_size: f32,
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
    pub importance_check_ahead_steps: u32,
    pub raymarching_step_size: f32,
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
            importance_check_ahead_steps: 15,
            raymarching_step_size: 0.020,
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
            importance_check_ahead_steps: parameters.importance_check_ahead_steps,
            raymarching_step_size: parameters.raymarching_step_size,
        }
    }

    pub fn process_input(
        &mut self,
        ctx: &super::gpu_context::GpuContext,
        event: &WindowEvent,
    ) -> bool {
        let r = match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyP),
                        ..
                    },
                ..
            } => {
                let output = ctx.surface.get_current_texture().unwrap();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let screenshot_path = format!("screenshot_{}.png", now);

                pollster::block_on(capture_screenshot(
                    &ctx.device,
                    &ctx.queue,
                    &output.texture,
                    &screenshot_path,
                ));

                info!("Screenshot saved to {}", screenshot_path);
                true
            }
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

use image::{ImageBuffer, Rgba};
use wgpu::util::DeviceExt;
async fn capture_screenshot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    surface_texture: &wgpu::Texture,
    filename: &str,
) -> Result<()> {
    let format = surface_texture.format();
    let size = surface_texture.size();
    let width = size.width;
    let height = size.height;

    // Create an intermediate texture with COPY_SRC usage
    let intermediate_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Screenshot Intermediate Texture"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Create buffer to receive the screenshot data
    let buffer_size = (width * height * 4) as wgpu::BufferAddress;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Screenshot Buffer"),
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Screenshot Encoder"),
    });

    // Copy from surface texture to intermediate texture
    encoder.copy_texture_to_texture(
        wgpu::ImageCopyTexture {
            texture: surface_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyTexture {
            texture: &intermediate_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    // Copy from intermediate texture to buffer
    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            texture: &intermediate_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::ImageCopyBuffer {
            buffer: &buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(Some(encoder.finish()));

    // Read the buffer
    let buffer_slice = buffer.slice(..);
    let (sender, receiver) = futures_intrusive::channel::shared::oneshot_channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::Wait);
    receiver.receive().await.unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data.to_vec()).unwrap();
    buffer.save(filename).unwrap();

    drop(data);
    Ok(())
}
