use std::path::Path;

use crate::Result;

pub struct VolymPipeline {
    pipeline: wgpu::ComputePipeline,
}

impl VolymPipeline {
    pub fn new(
        device: &wgpu::Device,
        shader_path: &Path,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<Self> {
        let shader_contents = std::fs::read_to_string(shader_path)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_path.to_str().unwrap()),
            source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&render_pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(VolymPipeline { pipeline })
    }
}

impl AsRef<wgpu::ComputePipeline> for VolymPipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
