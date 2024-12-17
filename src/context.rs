/// Rendering context
use std::path::Path;

// lib.rs
use crate::{camera::Camera, render_pipeline};
use tracing::{debug, info};
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    window::Window,
};

use crate::Result;

#[derive(Debug)]
pub struct Context<'a> {
    pub surface: wgpu::Surface<'a>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    _texture: wgpu::Texture,
    pub computed_texture_view: wgpu::TextureView,

    window: &'a Window,

    render_pipeline: render_pipeline::RenderPipeline,
    render_bind_group: wgpu::BindGroup,

    pub camera: Camera,
    pub camera_controller: crate::camera::CameraController,

    mouse_pressed: bool,
    last_mouse_position: Option<(f64, f64)>,
}

impl<'a> Context<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'a Window) -> Result<Context<'a>> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let render_path = format!("{}/shaders/render.wgsl", env!("CARGO_MANIFEST_DIR"));
        let render_pipeline = crate::context::render_pipeline::RenderPipeline::new(
            &device,
            Path::new(&render_path),
            &config,
        )?;

        // TODO: maybe handle resizing?
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Compute Output Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[], // TODO
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_pipeline.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ],
        });

        let aspect = config.width as f32 / config.height as f32;
        let camera = Camera::new(aspect, &device);

        let camera_controller = crate::camera::CameraController::new(0.2, 0.2);

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            _texture: texture,
            computed_texture_view: texture_view,
            render_pipeline,
            render_bind_group,
            camera,
            camera_controller,
            mouse_pressed: false,
            last_mouse_position: None,
        })
    }

    pub fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
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

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera.update_buffer(&self.queue, &self.camera.buffer);
    }

    #[tracing::instrument(skip(self))]
    pub fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        debug!("Camera Position: {:?}", self.camera.position);
        debug!("Camera Target: {:?}", self.camera.target);
        debug!("Horizontal Angle: {}", self.camera.horizontal_angle);
        debug!("Vertical Angle: {}", self.camera.vertical_angle);
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(self.render_pipeline.as_ref());
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            debug!(target = "render_pass", "Render bind group set");
            render_pass.draw(0..6, 0..1); // Draw a quad (2*3 vertices)
            debug!(target = "render_pass", "Draw done");
        }

        self.queue.submit(Some(encoder.finish()));

        // Before presenting to the screen we need to let the compositor know - This effectively
        // syncs us to the monitor refresh rate.
        // https://docs.rs/winit/latest/winit/window/struct.Window.html#platform-specific-2
        self.window.pre_present_notify();

        output.present();

        Ok(())
    }
}
