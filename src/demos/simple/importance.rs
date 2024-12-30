use egui_wgpu::wgpu;
use std::path::Path;

use serde::Deserialize;
use tracing::info;

use crate::{
    gpu_context::GpuContext,
    gpu_resources::{flip_3d_texture_y, BindGroupLayoutEntryUnbound, FlipMode, ToGpuResources},
    Result,
};

#[derive(Debug, Deserialize)]
pub struct SegmentInfo {
    pub id: String,
    pub name: String,
    pub index: u8,
    pub label_value: u8,
    pub importance: u8,
}

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
    pub fn init(
        data_path: &Path,
        info_path: &Path,
        flip_mode: FlipMode,
        ctx: &GpuContext,
    ) -> Result<Self> {
        info!("Loading Importances");

        let data = {
            let data = std::fs::read(data_path)?;
            let info: Vec<SegmentInfo> = serde_json::from_slice(&std::fs::read(info_path)?)?;
            let mut data = map_segments_to_importance(data, info);

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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
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
fn map_segments_to_importance(data: Vec<u8>, info: Vec<SegmentInfo>) -> Vec<u8> {
    // map each byte of data - which corresponds to label_value - to it's importance

    data.into_iter()
        .map(|label_value| {
            info.iter()
                .find(|segment| segment.label_value == label_value)
                .map_or(0, |segment| segment.importance)
        })
        .collect()
}
