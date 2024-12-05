/// Rendering context
use std::path::Path;

// lib.rs
use crate::render_pipeline;
use winit::{event::WindowEvent, window::Window};

use crate::{compute_pipeline::ComputePipeline, Result};

pub struct Context<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,

    window: &'a Window,

    compute_pipeline: ComputePipeline,
    compute_bind_group: wgpu::BindGroup,

    render_pipeline: render_pipeline::RenderPipeline,
    render_bind_group: wgpu::BindGroup,
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
            .request_device(
                &wgpu::DeviceDescriptor {
                    //required_features: wgpu::Features::default()
                    //    | wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                    //// | wgpu::Features::POLYGON_MODE_LINE
                    ..Default::default()
                },
                None,
            )
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

        let compute_path = format!(
            "{}/shaders/raycast_compute.wgsl",
            env!("CARGO_MANIFEST_DIR")
        );
        let render_path = format!("{}/shaders/raycast_render.wgsl", env!("CARGO_MANIFEST_DIR"));
        let compute_pipeline = crate::compute_pipeline::ComputePipeline::new(
            &device,
            Path::new(&compute_path),
            &config,
        )?;
        let render_pipeline = crate::context::render_pipeline::RenderPipeline::new(
            &device,
            Path::new(&render_path),
            &config,
        )?;

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

        let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &compute_pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        Ok(Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            compute_pipeline,
            compute_bind_group,
            render_pipeline,
            render_bind_group,
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

    pub fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> std::result::Result<(), wgpu::SurfaceError> {
        let size = self.window.inner_size();
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        // compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.compute_pipeline.as_ref());
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgroup.
            compute_pass.dispatch_workgroups((size.width + 15) / 16, (size.height + 15) / 16, 1);
        }

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
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
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
            render_pass.draw(0..6, 0..1); // Draw a quad (2*3 vertices)
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
