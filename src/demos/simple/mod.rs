use color_eyre::owo_colors::OwoColorize;
use gpu_volume::GPUVolume;
use tracing::info;

use crate::{rendering_context::Context, state::State, transfer_function};

use super::{compute_base, ComputeDemo};
use crate::Result;

pub mod compute_pipeline;
pub mod gpu_transfer_function;
pub mod gpu_volume;

#[derive(Debug)]
pub struct Simple {
    volume: gpu_volume::GPUVolume, // contains the bindgroup
    compute_pipeline: compute_pipeline::ComputePipeline,
    transfer_function: gpu_transfer_function::GPUTransferFunction,
    band_colors_group: wgpu::BindGroup,
}

impl ComputeDemo for Simple {
    fn init(ctx: &Context, state: &State, output_texture: &wgpu::Texture) -> Result<Self> {
        info!("Initializing Simple Demo");

        let volume_path = &(format!(
            "{}/assets/bonsai_256x256x256_uint8.raw",
            env!("CARGO_MANIFEST_DIR")
        ));

        let volume = GPUVolume::init(volume_path.as_ref(), gpu_volume::FlipMode::None, ctx)?;
        info!("Volume loaded: {:?}", volume_path);

        let transfer_function = transfer_function::TransferFunction1D::default();
        dbg!(&transfer_function);
        info!("Transfer Function initialized");
        info!("TF value at 0: {:?}", transfer_function.get(0.0));
        info!("TF value at 0.5: {:?}", transfer_function.get(0.5));
        info!("TF value at 1: {:?}", transfer_function.get(1.0));

        transfer_function.save_to_file("transfer_function.png".as_ref())?;

        let transfer_function = gpu_transfer_function::GPUTransferFunction::new_texture_1d_rgbt(
            &transfer_function,
            &ctx.device,
            &ctx.queue,
        );

        // First, create the texture data
        let band_colors: Vec<u8> = vec![
            255, 0, 0, 255, // Red
            0, 255, 0, 255, // Green
            0, 0, 255, 255, // Blue
            255, 255, 0, 255, // Yellow
            255, 0, 255, 255, // Magenta
        ];

        // Calculate the number of colors (total bytes / 4 bytes per color)
        let num_colors = band_colors.len() / 4;

        // Create the texture
        let band_colors_texture = ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Band Colors Texture"),
            size: wgpu::Extent3d {
                width: num_colors as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // Write the color data to the texture
        ctx.queue.write_texture(
            band_colors_texture.as_image_copy(),
            bytemuck::cast_slice(&band_colors),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(num_colors as u32 * 4),

                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: num_colors as u32,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        // Create the texture view
        let band_colors_view =
            band_colors_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // Create the sampler
        let band_colors_sampler = ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let band_colors_layout =
            ctx.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Band Colors Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::COMPUTE,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D1,
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
                });
        // Create the bind group
        let band_colors_bindgroup = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Band Colors Bind Group"),
            layout: &band_colors_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&band_colors_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&band_colors_sampler),
                },
            ],
        });

        let compute_pipeline = compute_pipeline::ComputePipeline::new(
            ctx,
            state,
            output_texture,
            &volume,
            &transfer_function,
            &band_colors_layout,
        )?;

        Ok(Simple {
            volume,
            compute_pipeline,
            transfer_function,
            band_colors_group: band_colors_bindgroup,
        })
    }

    fn update_gpu_state(&self, ctx: &Context, state: &State) -> Result<()> {
        self.compute_pipeline.base.update(ctx, state)?;
        Ok(())
    }

    fn compute_pass(&self, ctx: &Context) -> Result<()> {
        self.compute_pipeline.compute_pass(
            ctx,
            &self.volume,
            &self.transfer_function,
            &self.band_colors_group,
        );

        Ok(())
    }
}
