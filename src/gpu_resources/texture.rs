use crate::state::State;

use crate::gpu_context::GpuContext;
use egui_wgpu::wgpu;

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};

#[derive(Debug)]
pub struct GpuWriteTexture2D {
    texture_view: wgpu::TextureView,
    texture: wgpu::Texture,
}

impl GpuWriteTexture2D {
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

    pub fn from_write_texture_2d(read_texture: GpuReadTexture2D) -> Self {
        Self::from_wgpu_texture(read_texture.texture)
    }

    pub fn into_write_texture_2d(self, ctx: &GpuContext) -> GpuReadTexture2D {
        GpuReadTexture2D::from_wgpu_texture(ctx, self.texture)
    }

    fn from_wgpu_texture(texture: wgpu::Texture) -> Self {
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            texture_view,
            texture,
        }
    }

    pub fn new(ctx: &GpuContext, state: &State) -> Self {
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Input Texture"),
            size: wgpu::Extent3d {
                width: ctx.surface_config.width,
                height: ctx.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        Self::from_wgpu_texture(texture)
    }
}

impl ToGpuResources for GpuWriteTexture2D {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![wgpu::BindingResource::TextureView(&self.texture_view)]
    }
}

#[derive(Debug)]
pub struct GpuReadTexture2D {
    pub texture_view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    texture: wgpu::Texture,
}

impl GpuReadTexture2D {
    pub fn from_write_texture_2d(ctx: &GpuContext, write_texture: GpuWriteTexture2D) -> Self {
        Self::from_wgpu_texture(ctx, write_texture.texture)
    }

    pub fn into_write_texture_2d(self) -> GpuWriteTexture2D {
        GpuWriteTexture2D::from_wgpu_texture(self.texture)
    }

    fn from_wgpu_texture(ctx: &GpuContext, texture: wgpu::Texture) -> Self {
        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
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
        Self {
            texture_view,
            texture,
            sampler,
        }
    }
    pub fn new(ctx: &GpuContext) -> Self {
        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Render Input Texture"),
            size: wgpu::Extent3d {
                width: ctx.surface_config.width,
                height: ctx.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        Self::from_wgpu_texture(ctx, texture)
    }

    pub fn bind_group_layout_entries() -> Vec<BindGroupLayoutEntryUnbound> {
        Vec::from(&[
            BindGroupLayoutEntryUnbound {
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            BindGroupLayoutEntryUnbound {
                visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ])
    }
}

impl ToGpuResources for GpuReadTexture2D {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![
            wgpu::BindingResource::TextureView(&self.texture_view),
            wgpu::BindingResource::Sampler(&self.sampler),
        ]
    }
}
