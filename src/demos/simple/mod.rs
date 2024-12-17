use std::path::Path;

use tracing::{debug, info};

use crate::context;

use super::RenderingDemo;
use crate::Result;

pub mod compute_pipeline;
pub mod volume;

#[derive(Debug)]
pub struct Simple {
    volume: volume::Volume,                      // contains the bindgroup
    pipeline: compute_pipeline::ComputePipeline, // contains the bindgrouplayout
    compute_bind_group: wgpu::BindGroup,
    debug_matrxix_group: wgpu::BindGroup,
}

pub const DESC_DEBUG_MATRIX: wgpu::BindGroupLayoutDescriptor<'static> =
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

impl RenderingDemo for Simple {
    #[tracing::instrument(skip(ctx))]
    fn init(ctx: &mut context::Context) -> Result<Self> {
        info!("Initializing Simple Demo");

        let compute_path = format!("{}/shaders/simple_compute.wgsl", env!("CARGO_MANIFEST_DIR"));

        // Move?
        let input_texture_layout = ctx
            .device
            .create_bind_group_layout(&crate::volume::Volume::DESC);
        let camera_layout = ctx
            .device
            .create_bind_group_layout(&crate::camera::Camera::DESC);
        let debug_matrix_layout = ctx.device.create_bind_group_layout(&DESC_DEBUG_MATRIX);

        let pipeline = crate::compute_pipeline::ComputePipeline::new(
            &ctx.device,
            Path::new(&compute_path),
            &input_texture_layout,
            &camera_layout,
            &debug_matrix_layout,
        )?;

        // TODO move this to the compute pipeline
        // Abstract the compute pipeline away from the demo
        let compute_bind_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Group"),
            layout: &pipeline.bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&ctx.computed_texture_view),
            }],
        });

        let debug_matrix_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Debug Matrix Texture"),
            size: wgpu::Extent3d {
                width: ctx.size.width,
                height: ctx.size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            view_formats: &[],
        });

        let debug_matrxix_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Debug Matrix Bind Group"),
            layout: &debug_matrix_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &debug_matrix_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                ),
            }],
        });

        info!("Compute shader: {:?}", compute_path);
        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));

        let volume = volume::Volume::new(
            volume_path.as_ref(),
            volume::FlipMode::None,
            &ctx.device,
            &ctx.queue,
        )?;
        info!("Volume loaded: {:?}", volume_path);

        Ok(Simple {
            volume,
            pipeline,
            compute_bind_group,
            debug_matrxix_group,
        })
    }

    #[tracing::instrument(skip(self, ctx))]
    fn compute(&self, ctx: &mut context::Context) -> Result<()> {
        let size = ctx.size;
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Compute Encoder"),
            });

        // Compute pass
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Compute Pass"),
                timestamp_writes: None,
            });

            compute_pass.set_pipeline(self.pipeline.as_ref());

            // Get the volume inputs
            compute_pass.set_bind_group(0, &self.volume.bind_group, &[]);
            debug!(target = "compute_pass", "Volume inputs bind_group set");
            // Get the pipeline inputs
            compute_pass.set_bind_group(1, &self.compute_bind_group, &[]);
            debug!(target = "compute_pass", "Pipeline inputs bind_group set");

            compute_pass.set_bind_group(2, &ctx.camera.bind_group, &[]);
            debug!(target = "compute_pass", "Camera bind_group set");

            compute_pass.set_bind_group(3, &self.debug_matrxix_group, &[]);
            debug!(target = "compute_pass", "Debug matrix bind_group set");

            // size.width + 15 ensures that any leftover pixels (less than a full workgroup 16x16)
            // still require an additional workgroup.
            compute_pass.dispatch_workgroups((size.width + 15) / 16, (size.height + 15) / 16, 1);
            debug!(
                target = "compute_pass",
                "dispatch_workgroups: {}, {}, {}",
                (size.width + 15) / 16,
                (size.height + 15) / 16,
                1
            );
        }
        ctx.queue.submit(Some(encoder.finish()));
        debug!(
            target = "compute_pass",
            "Compute task submitted to the queue"
        );

        Ok(())
    }
}
