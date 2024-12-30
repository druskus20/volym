use crate::state::State;

use crate::gpu_context::GpuContext;
use egui_wgpu::wgpu;

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};

#[derive(Debug)]
pub struct GpuOutputTexture {
    texture_view: wgpu::TextureView,
}

impl GpuOutputTexture {
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

    pub fn new(ctx: &GpuContext, state: &State, output_texture: &wgpu::Texture) -> Self {
        let texture_view = output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self { texture_view }
    }
}

impl ToGpuResources for GpuOutputTexture {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![wgpu::BindingResource::TextureView(&self.texture_view)]
    }
}
