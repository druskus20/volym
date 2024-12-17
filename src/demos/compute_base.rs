use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{camera::Camera, rendering_context::Context, state::State};

/// Base struct for every compute pipeline
#[derive(Debug)]
pub struct ComputeBase {
    // Layouts are needed to create the pipeline
    pub output_texture_layout: wgpu::BindGroupLayout,
    pub camera_layout: wgpu::BindGroupLayout,
    pub debug_matrix_layout: wgpu::BindGroupLayout,

    // Groups are passed to the pipeline
    pub debug_matrix_group: wgpu::BindGroup,
    pub camera_group: wgpu::BindGroup,
    pub output_texture_group: wgpu::BindGroup,

    camera_buffer: wgpu::Buffer,
}

pub const DESC_OUTPUT_TEXTURE: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Storage Texture Layour"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    };

pub const DESC_DEBUG_MATRIX: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Storage Texture Layour"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }],
    };

pub const DESC_CAMERA_UNIFORMS: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
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

impl ComputeBase {
    pub fn new(ctx: &Context, state: &State, output_texture_view: &wgpu::TextureView) -> Self {
        let camera_layout = ctx.device.create_bind_group_layout(&DESC_CAMERA_UNIFORMS);
        let debug_matrix_layout = ctx.device.create_bind_group_layout(&DESC_DEBUG_MATRIX);
        let output_texture_layout = ctx.device.create_bind_group_layout(&DESC_OUTPUT_TEXTURE);

        let uniforms: CameraUniforms = CameraUniforms::from(&state.camera);

        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let debug_matrix_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Debug Matrix Texture"),
            size: wgpu::Extent3d {
                width: ctx.surface_config.width,
                height: ctx.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let debug_matrix_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug Matrix Bind Group"),
            layout: &debug_matrix_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &debug_matrix_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        let output_texture_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Output Texture Bind Group"),
            layout: &output_texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(output_texture_view),
            }],
        });
        Self {
            camera_layout,
            debug_matrix_group,
            output_texture_layout,
            camera_group,
            debug_matrix_layout,
            output_texture_group,
            camera_buffer,
        }
    }

    pub fn update(&self, ctx: &Context, state: &State) {
        let uniforms = CameraUniforms::from(&state.camera);
        ctx.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }
}

#[repr(C, align(16))]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniforms {
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: f32,
}

impl From<&Camera> for CameraUniforms {
    fn from(camera: &Camera) -> Self {
        CameraUniforms {
            view_matrix: camera.view_matrix().into(),
            projection_matrix: camera.projection_matrix().into(),
            camera_position: camera.position.into(),
            _padding: 0.0,
        }
    }
}
