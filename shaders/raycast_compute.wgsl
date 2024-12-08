// Calcualte whatever you want in the compute shader

@group(0) @binding(0)
var in_texture: texture_3d<f32>;

@group(0) @binding(1)
var in_sampler: sampler;

@group(1) @binding(0)
var out_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {

    // get the middle slice the texture3d 
    let size = textureDimensions(in_texture);
    let slice = size.z / 2;
    let texture = in_texture;
    let coords = vec3<u32>(global_id.x, global_id.y, slice);
    let level = 0;
    let texel = textureLoad(texture, coords, level);

    if texel.r > 0.0 {
        textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(1.0, 0.0, 0.0, 1.0));
    } else {
        textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 1.0, 0.0, 1.0));
    }
}
