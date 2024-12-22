
use crate::state::State;

use super::context::Context;

#[derive(Debug)]
pub struct GpuOutputTexture {
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,
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

impl GpuOutputTexture {
    pub fn new(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Self {
        let output_texture_layout = ctx.device.create_bind_group_layout(&DESC_OUTPUT_TEXTURE);

        let output_texture_view =
            output_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let output_texture_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Output Texture Bind Group"),
            layout: &output_texture_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&output_texture_view),
            }],
        });
        Self {
            layout: output_texture_layout,
            group: output_texture_group,
        }
    }
}
