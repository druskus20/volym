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
    function_vec: Vec<Vector4<f32>>,
}

impl Default for TransferFunction1D {
    fn default() -> Self {
        let mut tf = Self::new(255);
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

    pub fn add_rgb_control_point(&mut self, point: TransferControlPoint) {
        self.rgb_points.push(point);
        self.rgb_points
            .sort_by(|a, b| a.iso_value.partial_cmp(&b.iso_value).unwrap());
    }

    pub fn add_alpha_control_point(&mut self, point: TransferControlPoint) {
        self.alpha_points.push(point);
        self.alpha_points
            .sort_by(|a, b| a.iso_value.partial_cmp(&b.iso_value).unwrap());
    }

    // Converts the denstity range from 0..max_density to 0..1, keeping the values intact
    pub fn normalized(mut self) -> Self {
        for point in &mut self.rgb_points {
            point.iso_value /= self.max_density as f32;
        }

        for point in &mut self.alpha_points {
            point.iso_value /= self.max_density as f32;
        }
        self
    }

    pub fn build_linear(&mut self) {
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

    // Get interpolated value for a specific density
    pub fn get(&self, value: f32) -> Vector4<f32> {
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
}
