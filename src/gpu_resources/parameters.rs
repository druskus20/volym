use bytemuck::{Pod, Zeroable};
use cgmath::{perspective, Deg, Matrix4, Point3, Vector3};
use egui_wgpu::wgpu::{self, util::DeviceExt};
use egui_winit::winit::{dpi::PhysicalPosition, event::MouseScrollDelta};

use crate::{gpu_context::GpuContext, state::State};

use super::{BindGroupLayoutEntryUnbound, ToGpuResources};
use crate::Result;

#[derive(Debug)]
#[repr(C)]
pub struct GpuParameters {
    buffer: wgpu::Buffer,
}

impl GpuParameters {
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
        let uniforms: ParameterUniforms = ParameterUniforms::try_from(state).unwrap();
        let buffer = ctx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Parameters Buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self { buffer }
    }

    pub fn update(&self, ctx: &GpuContext, state: &State) -> Result<()> {
        let uniforms = ParameterUniforms::try_from(state)?;
        ctx.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniforms]));

        Ok(())
    }
}

impl ToGpuResources for GpuParameters {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource> {
        vec![self.buffer.as_entire_binding()]
    }
}

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C, align(16))]
pub struct ParameterUniforms {
    use_cone_importance_check: u32,
    use_importance_coloring: u32,
    use_opacity: u32,
    use_importance_rendering: u32,
}
impl ParameterUniforms {
    // Convenience methods to convert u8 to bool
    pub fn use_cone_importance_check(&self) -> bool {
        self.use_cone_importance_check != 0
    }

    pub fn use_importance_coloring(&self) -> bool {
        self.use_importance_coloring != 0
    }

    pub fn use_opacity(&self) -> bool {
        self.use_opacity != 0
    }

    pub fn use_importance_rendering(&self) -> bool {
        self.use_importance_rendering != 0
    }
}

impl TryFrom<&State> for ParameterUniforms {
    type Error = crate::Error;

    fn try_from(s: &State) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            use_cone_importance_check: if s.use_cone_importance_check { 1 } else { 0 },
            use_importance_coloring: if s.use_importance_coloring { 1 } else { 0 },
            use_opacity: if s.use_opacity { 1 } else { 0 },
            use_importance_rendering: if s.use_importance_rendering { 1 } else { 0 },
        })
    }
}
