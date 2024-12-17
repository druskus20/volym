/// Compute pipeline that does the heavy lifting and outputs to a texture
use std::path::Path;

use tracing::info;

use crate::{demos::compute_base, rendering_context::Context, state::State, Result};

#[derive(Debug)]
pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub base: compute_base::ComputeBase,
}

impl ComputePipeline {
    pub fn new(
        ctx: &Context,
        state: &State,
        output_texture_view: &wgpu::TextureView,
        input_volume_layout: &wgpu::BindGroupLayout,
    ) -> Result<Self> {
        let device = &ctx.device;
        let base = compute_base::ComputeBase::new(ctx, state, output_texture_view);

        let shader_path =
            Path::new(&(format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"))))
                .to_path_buf();
        let shader_contents = std::fs::read_to_string(&shader_path)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(shader_path.to_str().unwrap()),
            source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
        });

        info!("Creating compute pipeline");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[
                    input_volume_layout,
                    &base.output_texture_layout,
                    &base.camera_layout,
                    &base.debug_matrix_layout,
                ],
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

        Ok(ComputePipeline { pipeline, base })
    }
}

impl AsRef<wgpu::ComputePipeline> for ComputePipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
