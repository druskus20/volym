
use super::context::Context;
use crate::state::State;

/// Base struct for every compute pipeline
#[derive(Debug)]
pub struct GpuDebugMatrix {
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,
}

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
impl GpuDebugMatrix {
    pub fn new(ctx: &Context, state: &State) -> Self {
        let debug_matrix_layout = ctx.device.create_bind_group_layout(&DESC_DEBUG_MATRIX);

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

        Self {
            layout: debug_matrix_layout,
            group: debug_matrix_group,
        }
    }
}
