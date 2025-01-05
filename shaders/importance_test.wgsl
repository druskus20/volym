struct CameraUniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    camera_position: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

@group(2) @binding(0)
var volume_texture: texture_3d<f32>;
@group(2) @binding(1)
var volume_sampler: sampler;
@group(2) @binding(4)
var importances_texture: texture_3d<f32>;
@group(2) @binding(5)
var importances_sampler: sampler;

fn ray_box_intersection(ray_origin: vec3<f32>, ray_direction: vec3<f32>) -> vec2<f32> {
    let box_min = vec3<f32>(0.0);
    let box_max = vec3<f32>(1.0);

    let t1 = (box_min - ray_origin) / ray_direction;
    let t2 = (box_max - ray_origin) / ray_direction;

    let tmin = min(t1, t2);
    let tmax = max(t1, t2);

    let entry_point = max(max(tmin.x, tmin.y), tmin.z);
    let exit_point = min(min(tmax.x, tmax.y), tmax.z);

    return vec2<f32>(
        max(entry_point, 0.0),
        max(exit_point, 0.0)
    );
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_dims = textureDimensions(output_texture);

    if global_id.x >= output_dims.x || global_id.y >= output_dims.y {
        return;
    }

    let screen_coord = vec2<f32>(
        f32(global_id.x) / f32(output_dims.x),
        f32(global_id.y) / f32(output_dims.y)
    );

    let ndc_coord = vec2<f32>(
        screen_coord.x * 2.0 - 1.0,
        1.0 - screen_coord.y * 2.0
    );

    let ray_origin = camera.camera_position;
    let inverse_view_proj = camera.inverse_view_proj;
    let world_pos = inverse_view_proj * vec4<f32>(ndc_coord, 0.0, 1.0);
    let ray_direction = normalize(world_pos.xyz / world_pos.w - camera.camera_position);

    let intersection = ray_box_intersection(ray_origin, ray_direction);

    // If no intersection, render black
    if intersection.y <= intersection.x {
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    // Sample at the middle of the volume
    let sample_pos = ray_origin + ray_direction * (intersection.x + intersection.y) * 0.5;

    let density = textureSampleLevel(
        volume_texture,
        volume_sampler,
        sample_pos,
        0.0
    ).r;

    // Only process if it's not air (density above threshold)
    if density > 0.1 {
        let importance = textureSampleLevel(
            importances_texture,
            importances_sampler,
            sample_pos,
            0.0
        ).r;

        // Red for high importance, blue for low importance
        let color = select(
            vec4<f32>(0.0, 0.0, 1.0, 1.0),  // blue for low importance
            vec4<f32>(1.0, 0.0, 0.0, 1.0),  // red for high importance
            importance > 0.5
        );

        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), color);
    } else {
        // Render black for air
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 0.0, 0.0, 1.0));
    }
}
