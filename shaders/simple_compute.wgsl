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
@group(1) @binding(1)
var debug_texture: texture_storage_2d<rgba8unorm, write>;


@group(2) @binding(0)
var volume_texture: texture_3d<f32>;
@group(2) @binding(1)
var volume_sampler: sampler;
@group(2) @binding(2)
var transfer_function_texture: texture_1d<f32>;
@group(2) @binding(3)
var transfer_function_sampler: sampler;


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

fn compute_gradient(volume: texture_3d<f32>, s: sampler, pos: vec3<f32>) -> vec3<f32> {
    let offset = vec3<f32>(0.01, 0.01, 0.01);
    let grad_x = (textureSampleLevel(volume, s, pos + vec3<f32>(offset.x, 0.0, 0.0), 0.0).r - textureSampleLevel(volume, s, pos - vec3<f32>(offset.x, 0.0, 0.0), 0.0).r) / (2.0 * offset.x);
    let grad_y = (textureSampleLevel(volume, s, pos + vec3<f32>(0.0, offset.y, 0.0), 0.0).r - textureSampleLevel(volume, s, pos - vec3<f32>(0.0, offset.y, 0.0), 0.0).r) / (2.0 * offset.y);
    let grad_z = (textureSampleLevel(volume, s, pos + vec3<f32>(0.0, 0.0, offset.z), 0.0).r - textureSampleLevel(volume, s, pos - vec3<f32>(0.0, 0.0, offset.z), 0.0).r) / (2.0 * offset.z);

    return normalize(vec3<f32>(grad_x, grad_y, grad_z));
}

fn blinn_phong_shade(
    pos: vec3<f32>,
    color: vec3<f32>,
    volume: texture_3d<f32>,
    s: sampler
) -> vec3<f32> {
    let gradient_normal = compute_gradient(volume, s, pos);

    if length(gradient_normal) > 0.0 {
        let light_direction = normalize(vec3<f32>(1.0, 1.0, 1.0));
        let eye_direction = normalize(camera.camera_position - pos);
        let halfway_vector = normalize(eye_direction + light_direction);

        let ambient = 0.1;
        let diffuse = max(0.0, dot(gradient_normal, light_direction));
        let specular = pow(max(0.0, dot(halfway_vector, gradient_normal)), 32.0);

        return color * (ambient + 0.6 * diffuse) + vec3<f32>(1.0, 1.0, 1.0) * 0.3 * specular;
    }

    return color;
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

    if intersection.y <= intersection.x {
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }

    let step_size = 0.005;
    var accumulated_color = vec3<f32>(0.0);
    var accumulated_alpha = 0.0;

    var current_distance = intersection.x;
    while current_distance < intersection.y && accumulated_alpha < 0.95 {
        let current_pos = ray_origin + ray_direction * current_distance;

        let density = textureSampleLevel(
            volume_texture,
            volume_sampler,
            current_pos,
            0.0
        ).r;

    // Filter out low density noise
        if density < 0.12 { // Adjust this threshold value to find the right balance
            current_distance += step_size;
            continue;
        }

        // Sample transfer function
        let transfer_color = textureSampleLevel(
            transfer_function_texture,
            transfer_function_sampler,
            density,
            0.0
        );

        // Only process non-zero opacity samples
        if transfer_color.a > 0.0 {
            let shaded_color = blinn_phong_shade(
                current_pos,
                transfer_color.rgb,
                volume_texture,
                volume_sampler
            );
            
            // Opacity correction based on step size
            let alpha = 1.0 - pow(1.0 - transfer_color.a, step_size * 100.0);
            let opacity_contrib = (1.0 - accumulated_alpha) * alpha;

            accumulated_color += shaded_color * opacity_contrib;
            accumulated_alpha += opacity_contrib;
        }

        current_distance += step_size;
    }

    textureStore(output_texture, vec2<u32>(global_id.x, global_id.y),
        vec4<f32>(accumulated_color, accumulated_alpha));
    textureStore(debug_texture, vec2<u32>(global_id.x, global_id.y),
        vec4<f32>(ray_direction, 1.0));
}
