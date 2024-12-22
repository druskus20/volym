use bytemuck::{Pod, Zeroable};
use cgmath::{Matrix4, SquareMatrix};
use wgpu::util::DeviceExt;

use crate::gpu_context::Context;
use crate::Result;
use crate::{camera::Camera, state::State};

/// Base struct for every compute pipeline
#[derive(Debug)]
pub struct GpuCamera {
    pub layout: wgpu::BindGroupLayout,
    pub group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
}

pub const DESC_CAMERA_UNIFORMS: wgpu::BindGroupLayoutDescriptor<'static> =
    wgpu::BindGroupLayoutDescriptor {
        label: Some("Camera layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    };

impl GpuCamera {
    pub fn new(ctx: &Context, state: &State) -> Self {
        let camera_layout = ctx.device.create_bind_group_layout(&DESC_CAMERA_UNIFORMS);

        let uniforms: CameraUniforms = CameraUniforms::try_from(&state.camera).unwrap();
        let camera_buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_group = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        Self {
            layout: camera_layout,
            group: camera_group,
            camera_buffer,
        }
    }
    pub fn update(&self, ctx: &Context, state: &State) -> Result<()> {
        let uniforms = CameraUniforms::try_from(&state.camera)?;
        ctx.queue
            .write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[uniforms]));

        Ok(())
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
