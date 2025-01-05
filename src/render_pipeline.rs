use tracing::debug;

use crate::demos::pipeline::layout_from_unbound_entries;
use crate::gpu_resources::texture::GpuReadTexture2D;
use crate::gpu_resources::{ToBindGroupEntries, ToGpuResources};
/// Render pipeline that displays the texture on the screen
use crate::Result;

use crate::gpu_context::GpuContext;

use wgpu::{self, TextureView};

#[derive(Debug)]
pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub input_texture_group: wgpu::BindGroup,
}

impl RenderPipeline {
    pub fn init(ctx: &GpuContext, input_texture: &GpuReadTexture2D) -> Result<Self> {
        let shader_path = format!("{}/shaders/render.wgsl", env!("CARGO_MANIFEST_DIR"));
        let shader_contents = std::fs::read_to_string(&shader_path)?;
        let shader = ctx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(shader_path.as_str()),
                source: wgpu::ShaderSource::Wgsl(shader_contents.into()),
            });

        let render_input_texture_layout = layout_from_unbound_entries(
            ctx,
            "Render Input Texture Group Layout",
            &[GpuReadTexture2D::bind_group_layout_entries().as_slice()],
        );
        // Create render pipeline
        let pipeline_layout = ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&render_input_texture_layout],
                push_constant_ranges: &[],
            });

        let pipeline = ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: ctx.surface_config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            });

        let render_pipeline_resources = input_texture.to_gpu_resources();

        let input_texture_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Render Bind Group"),
            layout: &render_input_texture_layout,
            entries: &render_pipeline_resources.to_bind_group_entries(),
        });

        Ok(Self {
            pipeline,
            input_texture_group,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn render_pass(
        &self,
        ctx: &GpuContext,
        texture_view: &TextureView,
    ) -> std::result::Result<(), wgpu::SurfaceError> {
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: texture_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::default()),
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.input_texture_group, &[]);
            debug!(target = "render_pass", "Render bind group set");
            render_pass.draw(0..6, 0..1); // Draw a quad (2*3 vertices)
            debug!(target = "render_pass", "Draw done");
        }

        ctx.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}

impl AsRef<wgpu::RenderPipeline> for RenderPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
