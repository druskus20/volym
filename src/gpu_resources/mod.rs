use std::num::NonZeroU32;

use wgpu::{BindingType, ShaderStages};

pub mod camera;
pub mod debug_matrix;
pub mod output_texture;
pub mod transfer_function;
pub mod volume;

/// WGPU's BindGroupEntries and BindGroupLayoutEntry's force you to hardcode the binding index
/// This is a way to abstract that away.
/// - We let each resource define it's shape
/// - We merge many resources into a single bind group layout at the top level
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BindGroupLayoutEntryUnbound {
    /// Which shader stages can see this binding.
    pub visibility: ShaderStages,
    /// The type of the binding
    pub ty: BindingType,
    /// If this value is Some, indicates this entry is an array. Array size must be 1 or greater.
    ///
    /// If this value is Some and `ty` is `BindingType::Texture`, [`Features::TEXTURE_BINDING_ARRAY`] must be supported.
    ///
    /// If this value is Some and `ty` is any other variant, bind group creation will fail.
    pub count: Option<NonZeroU32>,
}

pub trait ToGpuResources {
    fn to_gpu_resources(&self) -> Vec<wgpu::BindingResource>;
}

// TODO: there are definitely better names and better ways to do this.
pub trait ToBindGroupEntries: Sized {
    fn to_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry>;
}

impl ToBindGroupEntries for Vec<wgpu::BindingResource<'_>> {
    fn to_bind_group_entries(&self) -> Vec<wgpu::BindGroupEntry> {
        self.iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupEntry {
                binding: i as u32,
                resource: r.clone(),
            })
            .collect()
    }
}

pub trait ToBindGroupLayoutEntries: Sized {
    fn to_bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry>;
}

impl ToBindGroupLayoutEntries for Vec<&BindGroupLayoutEntryUnbound> {
    fn to_bind_group_layout_entries(&self) -> Vec<wgpu::BindGroupLayoutEntry> {
        self.iter()
            .enumerate()
            .map(|(i, r)| wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: r.visibility,
                ty: r.ty,
                count: r.count,
            })
            .collect()
    }
}
