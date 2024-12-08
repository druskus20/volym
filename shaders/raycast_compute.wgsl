@group(0) @binding(0)
var volume_texture: texture_3d<f32>;

@group(0) @binding(1)
var volume_sampler: sampler;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dimensions = textureDimensions(volume_texture);
    let output_dims = textureDimensions(output_texture);
    
    // ensure we're within output texture bounds
    if global_id.x >= output_dims.x || global_id.y >= output_dims.y {
        return;
    }
    
    // normalize screen coordinates
    let screen_coord = vec2<f32>(
        f32(global_id.x) / f32(output_dims.x),
        f32(global_id.y) / f32(output_dims.y)
    );
    
    // Simple ray setup
    let ray_origin = vec3<f32>(0.5, 0.5, 2.0);
    let ray_direction = normalize(vec3<f32>(
        (screen_coord.x - 0.5) * 2.0,
        (screen_coord.y - 0.5) * 2.0,
        -1.0
    ));

    var current_pos = ray_origin;
    let step_size = 0.1;
    let max_steps = 50u;
    var found_non_zero = false;

    for (var i = 0u; i < max_steps; i++) {
        current_pos += ray_direction * step_size;
        
        // check if we're inside the volume
        if all(current_pos >= vec3<f32>(0.0)) && all(current_pos <= vec3<f32>(1.0)) {
            let sample_value = textureSampleLevel(
                volume_texture,
                volume_sampler,
                current_pos,
                0.0
            );
            
            // check if any channel is non-zero
            if sample_value.r != 0.0 || sample_value.g != 0.0 || sample_value.b != 0.0 {
                found_non_zero = true;
                break;
            }
        }
    }

    if found_non_zero {
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(1.0, 1.0, 1.0, 1.0));
    } else {
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 0.0, 0.0, 1.0));
    }
}
