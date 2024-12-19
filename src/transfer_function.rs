use cgmath::Vector4;

// Transfer Control Point for defining color/opacity at specific values
#[derive(Debug, Clone, Copy)]
struct TransferControlPoint {
    color: Vector4<f32>,
    iso_value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct TransferFunction1D {
    pub max_density: u32,
    rgb_points: Vec<TransferControlPoint>,
    alpha_points: Vec<TransferControlPoint>,
    pub function_vec: Vec<Vector4<f32>>,
}

impl TransferFunction1D {
    pub fn new(max_density: u32) -> Self {
        Self {
            max_density,
            rgb_points: Vec::new(),
            alpha_points: Vec::new(),
            function_vec: vec![Vector4::new(0.0, 0.0, 0.0, 0.0); (max_density + 1) as usize],
        }
    }

    fn add_rgb_control_point(&mut self, point: TransferControlPoint) {
        self.rgb_points.push(point);
        self.rgb_points
            .sort_by(|a, b| a.iso_value.partial_cmp(&b.iso_value).unwrap());
    }

    fn add_alpha_control_point(&mut self, point: TransferControlPoint) {
        self.alpha_points.push(point);
        self.alpha_points
            .sort_by(|a, b| a.iso_value.partial_cmp(&b.iso_value).unwrap());
    }

    fn build_linear(&mut self) {
        // RGB interpolation
        for window in self.rgb_points.windows(2) {
            let start = &window[0];
            let end = &window[1];

            for x in start.iso_value as u32..=end.iso_value as u32 {
                let k = (x as f32 - start.iso_value) / (end.iso_value - start.iso_value);

                // Linear interpolation for RGB components
                self.function_vec[x as usize] = Vector4::new(
                    start.color.x + (end.color.x - start.color.x) * k,
                    start.color.y + (end.color.y - start.color.y) * k,
                    start.color.z + (end.color.z - start.color.z) * k,
                    self.function_vec[x as usize].w, // Preserve existing alpha
                );
            }
        }

        // Alpha interpolation
        for window in self.alpha_points.windows(2) {
            let start = &window[0];
            let end = &window[1];

            for x in start.iso_value as u32..=end.iso_value as u32 {
                let k = (x as f32 - start.iso_value) / (end.iso_value - start.iso_value);

                // Linear interpolation for alpha component
                self.function_vec[x as usize].w = start.color.w + (end.color.w - start.color.w) * k;
            }
        }
    }

    // Convert to GPU-compatible buffer
    fn to_gpu_buffer(&self) -> Vec<f32> {
        let mut buffer = Vec::with_capacity((self.max_density as usize + 1) * 4);
        for v in &self.function_vec {
            buffer.extend_from_slice(&[v.x, v.y, v.z, v.w]);
        }
        buffer
    }

    // Get interpolated value for a specific density
    fn get(&self, value: f32) -> Vector4<f32> {
        let idx = (value * self.max_density as f32).clamp(0.0, self.max_density as f32);
        let idx_floor = idx.floor() as usize;
        let idx_ceil = (idx_floor + 1).min(self.max_density as usize);

        let t = idx.fract();

        // Linear interpolation between neighboring values
        let v1 = self.function_vec[idx_floor];
        let v2 = self.function_vec[idx_ceil];

        Vector4::new(
            v1.x + (v2.x - v1.x) * t,
            v1.y + (v2.y - v1.y) * t,
            v1.z + (v2.z - v1.z) * t,
            v1.w + (v2.w - v1.w) * t,
        )
    }

    pub fn generate_texture_1d_rgbt(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<(wgpu::Texture, wgpu::TextureView)> {
        let tf_size = self.max_density + 1;

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

        let mut data: Vec<f32> = Vec::with_capacity((tf_size * 4) as usize);

        for i in 0..tf_size {
            let tf_value = self.function_vec[i as usize];
            data.push(tf_value.x); // r
            data.push(tf_value.y); // g
            data.push(tf_value.z); // b

            let mut alpha = tf_value.w;
            //if !self.extinction_coef_type {
            //    alpha = self.material_opacity_to_extinction(alpha);
            //}
            data.push(alpha);
        }

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Transfer Function 1D View"),
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });

        queue.write_texture(
            texture.as_image_copy(),
            bytemuck::cast_slice(&data),
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

        Some((texture, texture_view))
    }
}

// Example usage
fn create_example_transfer_function() -> TransferFunction1D {
    let mut tf = TransferFunction1D::new(255);

    // RGB Control Points
    tf.add_rgb_control_point(TransferControlPoint {
        color: Vector4::new(0.0, 0.5, 0.0, 1.0), // Green
        iso_value: 0.0,
    });
    tf.add_rgb_control_point(TransferControlPoint {
        color: Vector4::new(0.0, 0.0, 1.0, 1.0), // Blue
        iso_value: 51.0,
    });
    tf.add_rgb_control_point(TransferControlPoint {
        color: Vector4::new(0.0, 1.0, 1.0, 1.0), // Cyan
        iso_value: 102.0,
    });
    tf.add_rgb_control_point(TransferControlPoint {
        color: Vector4::new(1.0, 1.0, 0.0, 1.0), // Yellow
        iso_value: 153.0,
    });
    tf.add_rgb_control_point(TransferControlPoint {
        color: Vector4::new(1.0, 0.0, 0.0, 1.0), // Red
        iso_value: 255.0,
    });

    // Alpha Control Points
    tf.add_alpha_control_point(TransferControlPoint {
        color: Vector4::new(0.0, 0.0, 0.0, 0.0), // Transparent
        iso_value: 0.0,
    });
    tf.add_alpha_control_point(TransferControlPoint {
        color: Vector4::new(0.0, 0.0, 0.0, 1.0), // Opaque
        iso_value: 255.0,
    });

    tf.build_linear();
    tf
}

