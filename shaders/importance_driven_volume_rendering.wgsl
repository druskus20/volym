
struct CameraUniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    inverse_view_proj: mat4x4<f32>,
    camera_position: vec3<f32>,
}

struct Parameters {
  density_threshold: f32,
  use_cone_importance_check: u32,
  use_importance_coloring: u32,
  use_opacity: u32,
  use_importance_rendering: u32,
  use_gaussian_smoothing: u32,
  importance_check_ahead_steps: u32,
  raymarching_step_size: f32,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;
@group(0) @binding(1) 
var<uniform> parameters: Parameters;

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
@group(2) @binding(4)
var importances_texture: texture_3d<f32>;
@group(2) @binding(5)
var importances_sampler: sampler;


fn gaussian_weight(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
}

// The smoothing works by sampling multiple points along the ray around each
// sample position, weighting these samples using a Gaussian function and
// computing a weighted average for the final density value

fn sample_volume_smoothed(pos: vec3<f32>, ray_dir: vec3<f32>, sigma: f32) -> f32 {
    let kernel_size = 2;  // higher is slower
    let step = 0.005;
    var sum = 0.0;
    var weight_sum = 0.0;

    for (var i = -kernel_size; i <= kernel_size; i++) {
        let offset = f32(i) * step;
        let sample_pos = pos + ray_dir * offset;
        
        // Skip samples outside the volume
        if any(sample_pos < vec3<f32>(0.0)) || any(sample_pos > vec3<f32>(1.0)) {
            continue;
        }

        let weight = gaussian_weight(offset, sigma);
        let sample = textureSampleLevel(volume_texture, volume_sampler, sample_pos, 0.0).r;

        sum += sample * weight;
        weight_sum += weight;
    }

    return sum / weight_sum;
}


fn has_non_zero_component(color: vec3<f32>) -> bool {
    let epsilon: f32 = 0.0001;
    return abs(color.x) > epsilon || abs(color.y) > epsilon || abs(color.z) > epsilon;
}

fn importance_to_color(importance: f32) -> vec4<f32> {
    let alpha = importance;

    return vec4<f32>(
        min(importance * 1.5, 1.0),
        (1.0 - importance) * 1.2,
        0.2,
        alpha
    );
}

fn sample_cone_directions(main_direction: vec3<f32>, cone_angle: f32, sample_index: i32, total_samples: i32) -> vec3<f32> {
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(main_direction, up));
    let new_up = cross(main_direction, right);

    let angle = (f32(sample_index) / f32(total_samples)) * 2.0 * 3.14159;
    let radius = cone_angle;

    let x_offset = cos(angle) * radius;
    let y_offset = sin(angle) * radius;

    return normalize(main_direction + right * x_offset + new_up * y_offset);
}

fn has_important_object_ahead_cone(current_pos: vec3<f32>, main_direction: vec3<f32>, max_distance: f32) -> bool {
    var pos = current_pos;
    let check_steps = i32(parameters.importance_check_ahead_steps);
    let step = (max_distance - length(current_pos)) / f32(check_steps);
    let cone_samples = 8;
    let cone_angle = 0.2;

    for (var s = 0; s < cone_samples; s++) {
        let sample_direction = sample_cone_directions(main_direction, cone_angle, s, cone_samples);
        pos = current_pos;

        for (var i = 0; i < check_steps; i++) {
            pos += sample_direction * step;

            if any(pos < vec3<f32>(0.0)) || any(pos > vec3<f32>(1.0)) {
                break;
            }

            let importance = textureSampleLevel(
                importances_texture,
                importances_sampler,
                pos,
                0.0
            ).r;

            if importance >= 0.5 {
                return true;
            }
        }
    }
    return false;
}

fn has_important_object_ahead_straight(current_pos: vec3<f32>, ray_direction: vec3<f32>, max_distance: f32) -> bool {
    var pos = current_pos;
    let check_steps = i32(parameters.importance_check_ahead_steps);
    let step = (max_distance - length(current_pos)) / f32(check_steps);

    for (var i = 0; i < check_steps; i++) {
        pos += ray_direction * step;
        let importance = textureSampleLevel(
            importances_texture,
            importances_sampler,
            pos,
            0.0
        ).r;

        if importance >= 0.5 {
            return true;
        }
    }
    return false;
}

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

        let ambient = 0.2;
        let diffuse = max(0.0, dot(gradient_normal, light_direction));
        let specular = pow(max(0.0, dot(halfway_vector, gradient_normal)), 24.0);

        return color * (ambient + 0.7 * diffuse) + vec3<f32>(1.0, 1.0, 1.0) * 0.4 * specular;
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

    let base_step_size = parameters.raymarching_step_size;
    let min_step_size = base_step_size * 0.25; // Minimum step size when in dense regions
    var current_step_size = base_step_size;
    var accumulated_color = vec3<f32>(0.0);
    var accumulated_alpha = 0.0;

    var current_distance = intersection.x;
    while current_distance < intersection.y && accumulated_alpha < 0.95 {
        let current_pos = ray_origin + ray_direction * current_distance;

        var density = 0.0;
        if parameters.use_gaussian_smoothing == 1 {
            let sigma = 1.5; // higher = more smoothing
            density = sample_volume_smoothed(current_pos, ray_direction, sigma);
        } else {
            density = textureSampleLevel(volume_texture, volume_sampler, current_pos, 0.0).r;
        }
        let importance = textureSampleLevel(importances_texture, importances_sampler, current_pos, 0.0).r;

         // Adapt step size based on density
        if density >= parameters.density_threshold {
            // When we hit something interesting, reduce step size
            current_step_size = min_step_size;
        } else {
            // Gradually return to base step size when in empty space
            current_step_size = min(base_step_size, current_step_size * 1.5);
        }

        if density < parameters.density_threshold {
            current_distance += current_step_size;
            continue;
        }

        var color_and_alpha: vec4<f32>;
        var use_alpha = parameters.use_opacity == 1;

        if parameters.use_importance_coloring == 1 {
            color_and_alpha = importance_to_color(importance);
            use_alpha = true;
        } else {
            if parameters.use_importance_rendering == 1 {
                var has_important_object_ahead = false;
                if parameters.use_cone_importance_check == 1 {
                    has_important_object_ahead = has_important_object_ahead_cone(current_pos, ray_direction, intersection.y);
                } else {
                    has_important_object_ahead = has_important_object_ahead_straight(current_pos, ray_direction, intersection.y);
                }

                if importance < 1.0 && has_important_object_ahead {
                    current_distance += current_step_size;
                continue;
                }
            }

            let transfer_color = textureSampleLevel(
                transfer_function_texture,
                transfer_function_sampler,
                density,
                0.0
            );
            color_and_alpha = transfer_color;
        }

        let shaded_color = blinn_phong_shade(
            current_pos,
            color_and_alpha.rgb,
            volume_texture,
            volume_sampler
        );

        if use_alpha {
            let alpha = 1.0 - pow(1.0 - color_and_alpha.a, current_step_size * 100.0);
            let opacity_contrib = (1.0 - accumulated_alpha) * alpha;

            accumulated_color += shaded_color * opacity_contrib;
            accumulated_alpha += opacity_contrib;
        } else {
            accumulated_color = shaded_color;
            accumulated_alpha = 1.0;
            break;
        }

        current_distance += current_step_size;
    }

    textureStore(output_texture, vec2<u32>(global_id.x, global_id.y),
        vec4<f32>(accumulated_color, accumulated_alpha));
}
