@group(0) @binding(0)
var xor_tex: texture_storage_3d<rgba16float, write>;
@group(0) @binding(1)
var normal_tex: texture_storage_3d<rgba16float, write>;


@compute @workgroup_size(4, 4, 4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    textureStore(xor_tex, global_id, vec4<f32>(0));
    textureStore(normal_tex, global_id, vec4<f32>(0));
}
