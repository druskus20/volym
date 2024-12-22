/// Compute pipeline that does the he'a avy lifting and outputs to a texture
use std::path::{Path, PathBuf};

use crate::Result;
use tracing::{debug, info};

use crate::gpu_context::Context;

#[derive(Debug)]
pub struct DemoPipeline {
    pub pipeline: wgpu::ComputePipeline,
}

pub(crate) struct DemoPipelineConfig<'a> {
    pub shader_path: PathBuf,
    pub bind_group_layouts: &'a [&'a wgpu::BindGroupLayout],
}

impl DemoPipeline {
    pub fn with_config(ctx: &Context, config: &DemoPipelineConfig) -> Result<Self> {
        let shader_contents = std::fs::read_to_string(&config.shader_path)?;
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(config.shader_path.to_str().unwrap()),
                source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
            });
        info!("Loaded shader module from {:?}", config.shader_path);

        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: config.bind_group_layouts,
                push_constant_ranges: &[],
            });

        info!("Creating compute pipeline");
        let pipeline = ctx
            .device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: Default::default(),
            });

        Ok(DemoPipeline { pipeline })
    }

    pub fn compute_pass(&self, ctx: &Context, bind_groups: &[&wgpu::BindGroup]) {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // move into compute_pipeline
        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(self.as_ref());

            for (i, bind_group) in bind_groups.iter().enumerate() {
                compute_pass.set_bind_group(i as u32, *bind_group, &[]);
            }

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgro
            compute_pass.dispatch_workgroups(
                (ctx.size.width + 15) / 16,
                (ctx.size.height + 15) / 16,
                1,
            );
            debug!(
                target = "compute_pass",
                "dispatch_workgroups: {}, {}, {}",
                (ctx.size.width + 15) / 16,
                (ctx.size.height + 15) / 16,
                1
            );
        }

        ctx.queue.submit(Some(encoder.finish()));
        debug!(
            target = "compute_pass",
            "Compute task submitted to the queue"
        );
    }
}

impl AsRef<wgpu::ComputePipeline> for DemoPipeline {
    fn as_ref(&self) -> &wgpu::ComputePipeline {
        &self.pipeline
    }
}
