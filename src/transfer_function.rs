use crate::Result;
use cgmath::Vector4;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct TransferControlPoint {
    color: Vector4<f32>,
    iso_value: f32,
}

#[derive(Debug, Clone)]
pub(crate) struct TransferFunction {
    pub max_density: u32,
    rgb_points: Vec<TransferControlPoint>,
    alpha_points: Vec<TransferControlPoint>,
    function_vec: Vec<Vector4<f32>>,
}

impl Default for TransferFunction {
    fn default() -> Self {
        let mut tf = Self::new(255);
        // RGB Control Points - normalized iso_values (0.0 to 1.0)
        tf.add_rgb_control_point(TransferControlPoint {
            color: Vector4::new(0.0, 1.0, 0.0, 1.0), // Green
            iso_value: 0.0,
        });
        tf.add_rgb_control_point(TransferControlPoint {
            color: Vector4::new(0.0, 1.0, 1.0, 1.0), // Cyan
            iso_value: 0.4,                          // ~102/255
        });
        tf.add_rgb_control_point(TransferControlPoint {
            color: Vector4::new(1.0, 1.0, 0.0, 1.0), // Yellow
            iso_value: 0.6,                          // ~153/255
        });
        tf.add_rgb_control_point(TransferControlPoint {
            color: Vector4::new(1.0, 0.0, 0.0, 1.0), // Red
            iso_value: 1.0,
        });

        // Alpha Control Points
        tf.add_alpha_control_point(TransferControlPoint {
            color: Vector4::new(0.0, 0.0, 0.0, 0.0), // Transparent
            iso_value: 0.0,
        });
        tf.add_alpha_control_point(TransferControlPoint {
            color: Vector4::new(0.0, 0.0, 0.0, 1.0), // Opaque
            iso_value: 1.0,
        });
        tf.build_linear();
        tf
    }
}

impl TransferFunction {
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

    pub fn build_linear(&mut self) {
        // RGB interpolation
        for window in self.rgb_points.windows(2) {
            let start = &window[0];
            let end = &window[1];

            let start_idx = (start.iso_value * self.max_density as f32) as u32;
            let end_idx = (end.iso_value * self.max_density as f32) as u32;

            for x in start_idx..=end_idx {
                let k = if end_idx == start_idx {
                    0.0
                } else {
                    (x - start_idx) as f32 / (end_idx - start_idx) as f32
                };

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

            let start_idx = (start.iso_value * self.max_density as f32) as u32;
            let end_idx = (end.iso_value * self.max_density as f32) as u32;

            for x in start_idx..=end_idx {
                let k = if end_idx == start_idx {
                    0.0
                } else {
                    (x - start_idx) as f32 / (end_idx - start_idx) as f32
                };

                // Linear interpolation for alpha component
                self.function_vec[x as usize].w = start.color.w + (end.color.w - start.color.w) * k;
            }
        }
    }

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

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let mut imgbuf = image::ImageBuffer::new(self.max_density + 1, 1);
        for (x, _, pixel) in imgbuf.enumerate_pixels_mut() {
            let color = self.get(x as f32 / self.max_density as f32); // Normalize input value
            *pixel = image::Rgba([
                (color.x * 255.0) as u8,
                (color.y * 255.0) as u8,
                (color.z * 255.0) as u8,
                (color.w * 255.0) as u8,
            ]);
        }
        imgbuf.save(path)?;
        Ok(())
    }
}
