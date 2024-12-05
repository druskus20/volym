/// Compute pipeline that does the heavy lifting and outputs to a texture
use std::path::Path;

use crate::Result;

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub const DESC_COMPUTE: wgpu::BindGroupLayoutDescriptor<'static> =
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

impl ComputePipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_path: &Path,
        _config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self> {
        let shader_contents = std::fs::read_to_string(shader_path)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_path.to_str().unwrap()),
            source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
        });

        let storage_texture_layout = device.create_bind_group_layout(&DESC_COMPUTE);
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&storage_texture_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: Default::default(),
        });

        Ok(ComputePipeline {
            pipeline,
            bind_group_layout: storage_texture_layout,
        })
    }
}

impl AsRef<wgpu::ComputePipeline> for ComputePipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
