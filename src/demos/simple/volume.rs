use tracing::info;

use crate::Result;
use std::path::Path;

pub struct Volume {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    sampler: wgpu::Sampler,
}

impl Volume {
    pub const DESC: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
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

    #[tracing::instrument]
    pub fn new(
        path: &Path,
        flip_mode: FlipMode,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Result<Self> {
        info!("Loading volume from {:?}", path);
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
        let texture = device.create_texture(&wgpu::TextureDescriptor {
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

        queue.write_texture(
            texture.as_image_copy(),
            &data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(256),
            },
            size,
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Volume Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&Self::DESC);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Volume Bind Group"),
            layout: &bind_group_layout,
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

        Ok(Volume {
            texture,
            bind_group,
            sampler,
        })
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
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
