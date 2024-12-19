use tracing::info;

use crate::transfer_function::TransferFunction1D;

#[derive(Debug)]
pub struct GPUTransferFunction {
    pub bind_group: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
}

impl GPUTransferFunction {
    const DESC_TRANSFER_FUNCTION: wgpu::BindGroupLayoutDescriptor<'static> =
        wgpu::BindGroupLayoutDescriptor {
            label: Some("Transfer Function Bind Group Layout"),
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
        };

    pub fn new_texture_1d_rgbt(
        tf: &TransferFunction1D,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let tf_size = tf.max_density + 1;

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Transfer Function 1D Texture"),
            size: wgpu::Extent3d {
                width: tf_size,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let mut texture_data: Vec<f32> = Vec::with_capacity((tf_size * 4) as usize);

        for i in 0..tf_size {
            let tf_value = tf.function_vec[i as usize];
            texture_data.push(tf_value.x); // r
            texture_data.push(tf_value.y); // g
            texture_data.push(tf_value.z); // b

            let alpha = tf_value.w;
            //if !tf.extinction_coef_type {
            //    alpha = tf.material_opacity_to_extinction(alpha);
            //}
            texture_data.push(alpha);
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Transfer Function 1D View"),
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&texture_data),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(tf_size * 4 * std::mem::size_of::<f32>() as u32),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: tf_size,
                height: 1,
                depth_or_array_layers: 1,
            },
        );

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Transfer Function Sampler"),
            ..Default::default()
        });

        info!("Sampler: {:?}", sampler);

        let tf_layout = device.create_bind_group_layout(&Self::DESC_TRANSFER_FUNCTION);

        let transfer_function_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Transfer Function Bind Group"),
            layout: &tf_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            bind_group: transfer_function_bind_group,
            layout: tf_layout,
        }
    }
}
