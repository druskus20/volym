use crate::gpu_context::Context;
use crate::state::State;

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};

/// Base struct for every compute pipeline
#[derive(Debug)]
pub struct GpuDebugMatrix {
    texture_view: wgpu::TextureView,
}

impl GpuDebugMatrix {
    pub const BIND_GROUP_LAYOUT_ENTRIES: &[BindGroupLayoutEntryUnbound] =
        &[BindGroupLayoutEntryUnbound {
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: wgpu::TextureFormat::Rgba8Unorm,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            count: None,
        }];
    pub fn new(ctx: &Context, state: &State) -> Self {
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
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
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&Default::default());

        Self { texture_view }
    }
}

impl ToGpuResources for GpuDebugMatrix {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![wgpu::BindingResource::TextureView(&self.texture_view)]
    }
}
