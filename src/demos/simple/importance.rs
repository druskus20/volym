use std::path::Path;

use tracing::info;

use crate::{
    gpu_context::Context,
    gpu_resources::{flip_3d_texture_y, BindGroupLayoutEntryUnbound, FlipMode, ToGpuResources},
    Result,
};

#[derive(Debug)]
pub struct GpuImportances {
    texture_view: wgpu::TextureView,
    sampler: wgpu::Sampler,
}

impl GpuImportances {
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
    pub fn init(path: &Path, flip_mode: FlipMode, ctx: &Context) -> Result<Self> {
        info!("Loading Importances");

        let data = {
            let mut data = std::fs::read(path)?;
            // center the volume to be 256x256x256
            let desired_len = 256 * 256 * 256;

            if data.len() < desired_len {
                info!(
                    "Importances' size is {}, which is less than 256x256x256, padding with zeros",
                    data.len()
                );
                data.resize(desired_len, 0);
            } else {
                info!(
                    "Importances' size is {}, which is greater than 256x256x256, truncating",
                    data.len()
                );
                data.truncate(desired_len);
            }

            if flip_mode == FlipMode::Y {
                flip_3d_texture_y(&mut data, (256, 256, 256));
            }
            data
        };

        // Segments are identified by an Id, this id refers to it's importance.
        // Count how many segments exist (i.e. how many different Id's are), and how many voxels each segment has.
        let diff_segments = data.iter().fold([0; 256], |mut acc, &id| {
            acc[id as usize] += 1;
            acc
        });
        for (id, count) in diff_segments.iter().enumerate() {
            if *count > 0 {
                info!("Segment {} has {} voxels", id, count);
            }
        }

        let size = wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 256,
        };

        let texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Importances Texture"),
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
            label: Some("Importances Sampler"),
            ..Default::default()
        });

        Ok(Self {
            texture_view,
            sampler,
        })
    }
}

impl ToGpuResources for GpuImportances {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![
            wgpu::BindingResource::TextureView(&self.texture_view),
            wgpu::BindingResource::Sampler(&self.sampler),
        ]
    }
}
