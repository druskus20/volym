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
        // let tf_size = tf.max_density + 1;
        // // Calculate proper dimensions based on the actual texture data
        // let pixel_size = 4; // RGBA, 4 bytes per pixel
        // let bytes_per_row = tf_size * pixel_size;
        let tf_size = tf.max_density + 1;
        let bytes_per_color = 4;
        //let num_colors = texture_data.len() / bytes_per_color;

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
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // flatten the transfer function data
        let mut texture_data: Vec<u8> = Vec::with_capacity((tf_size * bytes_per_color) as usize);

        //// Fill the texture data based on transfer function values
        for i in 0..tf_size {
            let tf_value = tf.get(i as f32 / tf_size as f32);

            info!("i: {:3}, tf_value {:?}", i, tf_value);
            texture_data.push((tf_value.x * 255.0) as u8); // r
            texture_data.push((tf_value.y * 255.0) as u8); // g
            texture_data.push((tf_value.z * 255.0) as u8); // b

            let alpha = tf_value.w;
            //if !tf.extinction_coef_type {
            //    alpha = tf.material_opacity_to_extinction(alpha);
            //}
            texture_data.push((alpha * 255.0) as u8);
        }

        // Debug make a red -> blue texture half ahd half
        //for i in 0..tf_size {
        //    if i < tf_size / 2 {
        //        texture_data.push(255);
        //        texture_data.push(0);
        //        texture_data.push(0);
        //        texture_data.push(255);
        //    } else {
        //        texture_data.push(0);
        //        texture_data.push(0);
        //        texture_data.push(255);
        //        texture_data.push(255);
        //    }
        //}

        info!(
            "Texture len / bytes_per_color: {}",
            texture_data.len() / bytes_per_color as usize
        );
        info!("Texture size: {}", tf_size);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Transfer Function 1D View"),
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });

        // Calculate proper dimensions based on the actual texture data

        queue.write_texture(
            texture.as_image_copy(),
            &texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(tf_size * bytes_per_color),
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
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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
