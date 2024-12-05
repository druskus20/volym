/// Render pipeline that displays the texture on the screen
use crate::Result;
use std::path::Path;

pub struct RenderPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

pub const DESC_RENDER: wgpu::BindGroupLayoutDescriptor<'static> = wgpu::BindGroupLayoutDescriptor {
    label: Some("Render Bind Group"),
    entries: &[
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        },
        wgpu::BindGroupLayoutEntry {
            binding: 1,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        },
    ],
};
impl RenderPipeline {
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
        let storage_texture_layout = device.create_bind_group_layout(&DESC_RENDER);

        // Create render pipeline
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&storage_texture_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
                    format: config.format,
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

        Ok(Self {
            pipeline,
            bind_group_layout: storage_texture_layout,
        })
    }
}

impl AsRef<wgpu::RenderPipeline> for RenderPipeline {
    fn as_ref(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}
