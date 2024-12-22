use tracing::info;

use crate::Result;
use std::path::Path;

use super::context::Context;

#[derive(Debug)]
pub struct GpuVolume {
    pub group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

impl GpuVolume {
    pub const DESC_VOLUME: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Volume Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        };

    #[tracing::instrument(skip(ctx))]
    pub fn init(path: &Path, flip_mode: FlipMode, ctx: &Context) -> Result<Self> {
        info!("Loading volume");
        let data = {
            let mut data = std::fs::read(path)?;
            if flip_mode == FlipMode::Y {
                flip_y(&mut data, 256, 256, 256);
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

        let volume_layout = ctx.device.create_bind_group_layout(&Self::DESC_VOLUME);
        let volume_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volume Bind Group"),
            layout: &volume_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(GpuVolume {
            group: volume_group,
            layout: volume_layout,
        })
    }
}

fn flip_y(data: &mut [u8], width: usize, height: usize, depth: usize) {
    for z in 0..depth {
        for y in 0..(height / 2) {
            let top_row = y * width;
            let bottom_row = (height - y - 1) * width;
            for x in 0..width {
                let top_index = z * width * height + top_row + x;
                let bottom_index = z * width * height + bottom_row + x;
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
