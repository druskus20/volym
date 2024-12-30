use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::util::DeviceExt;

use crate::gpu_context::GpuContext;
use crate::Result;
use crate::{camera::Camera, state::State};

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};

/// Base struct for every compute pipeline
#[derive(Debug)]
pub struct GpuCamera {
    camera_buffer: wgpu::Buffer,
}

impl GpuCamera {
    pub const BIND_GROUP_LAYOUT_ENTRIES: &[BindGroupLayoutEntryUnbound] =
        &[BindGroupLayoutEntryUnbound {
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }];
    pub fn new(ctx: &GpuContext, state: &State) -> Self {
        let uniforms: CameraUniforms = CameraUniforms::try_from(&state.camera).unwrap();
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self { camera_buffer }
    }
    pub fn update(&self, ctx: &GpuContext, state: &State) -> Result<()> {
        let uniforms = CameraUniforms::try_from(&state.camera)?;
        ctx.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        Ok(())
    }
}

impl ToGpuResources for GpuCamera {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![self.camera_buffer.as_entire_binding()]
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct CameraUniforms {
    view_matrix: [[f32; 4]; 4],
    projection_matrix: [[f32; 4]; 4],
    inverse_view_proj: [[f32; 4]; 4],
    camera_position: [f32; 3],
    _padding: f32,
}

impl TryFrom<&Camera> for CameraUniforms {
    type Error = crate::Error;
    fn try_from(camera: &Camera) -> std::result::Result<Self, Self::Error> {
        let projection_matrix = camera.projection_matrix();
        let view_matrix = camera.view_matrix();

        let inverse_view_proj: Matrix4<f32> = view_matrix.invert().ok_or(
            color_eyre::eyre::eyre!("inverse_view_proj inversion failed"),
        )? * projection_matrix
            .invert()
            .ok_or(color_eyre::eyre::eyre!("view_matrix inversion failed"))?;
        Ok(CameraUniforms {
            view_matrix: view_matrix.into(),
            projection_matrix: projection_matrix.into(),
            inverse_view_proj: inverse_view_proj.into(),
            camera_position: camera.position.into(),
            _padding: 0.0,
        })
    }
}
