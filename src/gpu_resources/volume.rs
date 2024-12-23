use tracing::info;

use crate::Result;
use std::path::Path;

use crate::gpu_context::Context;

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};

#[derive(Debug)]
pub struct GpuVolume {
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl GpuVolume {
    pub const BIND_GROUP_LAYOUT_ENTRIES: &[BindGroupLayoutEntryUnbound] = &[
        BindGroupLayoutEntryUnbound {
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D3,
                multisampled: false,
            },
            count: None,
        },
        BindGroupLayoutEntryUnbound {
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
    ];

    #[tracing::instrument(skip(ctx))]
    pub fn init(path: &Path, flip_mode: FlipMode, ctx: &Context) -> Result<Self> {
        info!("Loading volume");

        let data = {
            let mut data = std::fs::read(path)?;
            // center the volume to be 256x256x256
            let desired_len = 256 * 256 * 256;

            if data.len() < desired_len {
                info!("Volume's size is less than 256x256x256, padding with zeros");
                data.resize(desired_len, 0);
            } else {
                info!("Volume's size is greater than 256x256x256, truncating");
                data.truncate(desired_len);
            }

            if flip_mode == FlipMode::Y {
                flip_y(&mut data, (256, 256, 256));
            }
            data
        };

        let size = wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 256,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Volume Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[], // TODO
        });
        let texture_view = texture.create_view(&Default::default());

        ctx.queue.write_texture(
            texture.as_image_copy(),
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(256),
            },
            size,
        );

        let sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Volume Sampler"),
            ..Default::default()
        });

        Ok(GpuVolume {
            texture_view,
            sampler,
        })
    }
}

impl ToGpuResources for GpuVolume {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![
            wgpu::BindingResource::TextureView(&self.texture_view),
            wgpu::BindingResource::Sampler(&self.sampler),
        ]
    }
}

fn flip_y(data: &mut [u8], (x, y, z): (usize, usize, usize)) {
    for k in 0..z {
        for j in 0..(y / 2) {
            let top_row = j * x;
            let bottom_row = (y - j - 1) * x;
            for i in 0..x {
                let top_index = k * x * y + top_row + i;
                let bottom_index = k * x * y + bottom_row + i;
                data.swap(top_index, bottom_index);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FlipMode {
    None,
    Y,
}
