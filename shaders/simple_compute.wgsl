@group(0) @binding(0)
var volume_texture: texture_3d<f32>;
@group(0) @binding(1)
var volume_sampler: sampler;
@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba8unorm, write>;

// New camera uniform buffer for camera parameters
struct CameraUniforms {
    view_matrix: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    camera_position: vec3<f32>,
}

@group(2) @binding(0)
var<uniform> camera: CameraUniforms;

// Debug matrix struct
struct DebugMatrix {
    matrix: array<vec4<f32>, 4>,
}

@group(3) @binding(0)
var debug_texture: texture_storage_2d<rgba8unorm, write>;


fn inverse(m: mat4x4<f32>) -> mat4x4<f32> {
    var inv: mat4x4<f32>;
    var det: f32;

    inv[0][0] = m[1][1] * m[2][2] * m[3][3] - m[1][1] * m[2][3] * m[3][2] - m[2][1] * m[1][2] * m[3][3] + m[2][1] * m[1][3] * m[3][2] + m[3][1] * m[1][2] * m[2][3] - m[3][1] * m[1][3] * m[2][2];

    inv[0][1] = -m[0][1] * m[2][2] * m[3][3] + m[0][1] * m[2][3] * m[3][2] + m[2][1] * m[0][2] * m[3][3] - m[2][1] * m[0][3] * m[3][2] - m[3][1] * m[0][2] * m[2][3] + m[3][1] * m[0][3] * m[2][2];

    inv[0][2] = m[0][1] * m[1][2] * m[3][3] - m[0][1] * m[1][3] * m[3][2] - m[1][1] * m[0][2] * m[3][3] + m[1][1] * m[0][3] * m[3][2] + m[3][1] * m[0][2] * m[1][3] - m[3][1] * m[0][3] * m[1][2];

    inv[0][3] = -m[0][1] * m[1][2] * m[2][3] + m[0][1] * m[1][3] * m[2][2] + m[1][1] * m[0][2] * m[2][3] - m[1][1] * m[0][3] * m[2][2] - m[2][1] * m[0][2] * m[1][3] + m[2][1] * m[0][3] * m[1][2];

    inv[1][0] = -m[1][0] * m[2][2] * m[3][3] + m[1][0] * m[2][3] * m[3][2] + m[2][0] * m[1][2] * m[3][3] - m[2][0] * m[1][3] * m[3][2] - m[3][0] * m[1][2] * m[2][3] + m[3][0] * m[1][3] * m[2][2];

    inv[1][1] = m[0][0] * m[2][2] * m[3][3] - m[0][0] * m[2][3] * m[3][2] - m[2][0] * m[0][2] * m[3][3] + m[2][0] * m[0][3] * m[3][2] + m[3][0] * m[0][2] * m[2][3] - m[3][0] * m[0][3] * m[2][2];

    inv[1][2] = -m[0][0] * m[1][2] * m[3][3] + m[0][0] * m[1][3] * m[3][2] + m[1][0] * m[0][2] * m[3][3] - m[1][0] * m[0][3] * m[3][2] - m[3][0] * m[0][2] * m[1][3] + m[3][0] * m[0][3] * m[1][2];

    inv[1][3] = m[0][0] * m[1][2] * m[2][3] - m[0][0] * m[1][3] * m[2][2] - m[1][0] * m[0][2] * m[2][3] + m[1][0] * m[0][3] * m[2][2] + m[2][0] * m[0][2] * m[1][3] - m[2][0] * m[0][3] * m[1][2];

    inv[2][0] = m[1][0] * m[2][1] * m[3][3] - m[1][0] * m[2][3] * m[3][1] - m[2][0] * m[1][1] * m[3][3] + m[2][0] * m[1][3] * m[3][1] + m[3][0] * m[1][1] * m[2][3] - m[3][0] * m[1][3] * m[2][1];

    inv[2][1] = -m[0][0] * m[2][1] * m[3][3] + m[0][0] * m[2][3] * m[3][1] + m[2][0] * m[0][1] * m[3][3] - m[2][0] * m[0][3] * m[3][1] - m[3][0] * m[0][1] * m[2][3] + m[3][0] * m[0][3] * m[2][1];

    inv[2][2] = m[0][0] * m[1][1] * m[3][3] - m[0][0] * m[1][3] * m[3][1] - m[1][0] * m[0][1] * m[3][3] + m[1][0] * m[0][3] * m[3][1] + m[3][0] * m[0][1] * m[1][3] - m[3][0] * m[0][3] * m[1][1];

    inv[2][3] = -m[0][0] * m[1][1] * m[2][3] + m[0][0] * m[1][3] * m[2][1] + m[1][0] * m[0][1] * m[2][3] - m[1][0] * m[0][3] * m[2][1] - m[2][0] * m[0][1] * m[1][3] + m[2][0] * m[0][3] * m[1][1];

    inv[3][0] = -m[1][0] * m[2][1] * m[3][2] + m[1][0] * m[2][2] * m[3][1] + m[2][0] * m[1][1] * m[3][2] - m[2][0] * m[1][2] * m[3][1] - m[3][0] * m[1][1] * m[2][2] + m[3][0] * m[1][2] * m[2][1];

    inv[3][1] = m[0][0] * m[2][1] * m[3][2] - m[0][0] * m[2][2] * m[3][1] - m[2][0] * m[0][1] * m[3][2] + m[2][0] * m[0][2] * m[3][1] + m[3][0] * m[0][1] * m[2][2] - m[3][0] * m[0][2] * m[2][1];

    inv[3][2] = -m[0][0] * m[1][1] * m[3][2] + m[0][0] * m[1][2] * m[3][1] + m[1][0] * m[0][1] * m[3][2] - m[1][0] * m[0][2] * m[3][1] - m[3][0] * m[0][1] * m[1][2] + m[3][0] * m[0][2] * m[1][1];

    inv[3][3] = m[0][0] * m[1][1] * m[2][2] - m[0][0] * m[1][2] * m[2][1] - m[1][0] * m[0][1] * m[2][2] + m[1][0] * m[0][2] * m[2][1] + m[2][0] * m[0][1] * m[1][2] - m[2][0] * m[0][2] * m[1][1];

    det = m[0][0] * inv[0][0] + m[0][1] * inv[0][1] + m[0][2] * inv[0][2] + m[0][3] * inv[0][3];

    if det == 0.0 {
        return mat4x4<f32>(); // Return zero matrix if not invertible
    }

    for (var i = 0; i < 4; i++) {
        for (var j = 0; j < 4; j++) {
            inv[i][j] = inv[i][j] / det;
        }
    }

    return inv;
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
// Transfer function to map density to color
fn transfer_function(density: f32) -> vec4<f32> {
    let intensity = clamp(density * 5.0, 0.0, 1.0);

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    if density < 0.2 {
        color = vec4<f32>(0.0, 0.5, 0, 1.0); // Green
    } else if density < 0.4 {
        color = vec4<f32>(0.0, 0.0, 1, 1.0); // Blue
    } else if density < 0.6 {
        color = vec4<f32>(0.0, 1.0, 1.0, 1.0); // Cyan
    } else if density < 0.8 {
        color = vec4<f32>(1.0, 1.0, 0.0, 1.0); // Yellow
    } else {
        color = vec4<f32>(1.0, 0, 0.0, 1.0);   // Red
    }

    color.a = intensity;
    return color;
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
        let eye_direction = normalize(vec3<f32>(0.0, 0.0, 1.0));
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
    
    // Ensure we're within output texture bounds
    if global_id.x >= output_dims.x || global_id.y >= output_dims.y {
        return;
    }
    
    // Normalize screen coordinates
    let screen_coord = vec2<f32>(
        f32(global_id.x) / f32(output_dims.x),
        f32(global_id.y) / f32(output_dims.y)
    );

    // Create ray using view and projection matrices
    let ndc_coord = vec2<f32>(
        screen_coord.x * 2.0 - 1.0,
        1.0 - screen_coord.y * 2.0  // Flip Y coordinate
    );

    // Compute ray in world space
    let ray_origin = camera.camera_position;
    //let ray_direction = normalize((camera.projection_matrix * camera.view_matrix * vec4<f32>(ndc_coord, 0.0, 1.0)).xyz);
  // Inverse view-projection matrix approach
    let inverse_view_proj = inverse(camera.projection_matrix * camera.view_matrix);
    let world_pos = inverse_view_proj * vec4<f32>(ndc_coord, 0.0, 1.0);
    let ray_direction = normalize(world_pos.xyz / world_pos.w - camera.camera_position);

    // Compute ray-box intersection
    let intersection = ray_box_intersection(ray_origin, ray_direction);
    
    // If no intersection, output black
    if intersection.y <= intersection.x {
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.0, 0.0, 0.0, 1.0));
        return;
    }
    
    // Ray marching parameters
    let step_size = 0.01;
    let max_distance = intersection.y - intersection.x;
    
    // Initialize accumulated color and opacity
    var accumulated_color = vec3<f32>(0.0, 0.0, 0.0);
    var accumulated_opacity = 0.0;

    var current_distance = intersection.x;
    while current_distance < intersection.y && accumulated_opacity < 0.99 {
        let current_pos = ray_origin + ray_direction * current_distance;
        
        // Sample volume texture
        let sample_value = textureSampleLevel(
            volume_texture,
            volume_sampler,
            current_pos,
            0.0
        );

        // Transfer function to get color and opacity
        let transfer_color = transfer_function(sample_value.r);
        
        // Apply Blinn-Phong shading if non-transparent
        if transfer_color.a > 0.0 {
            let shaded_color = blinn_phong_shade(
                current_pos,
                transfer_color.rgb,
                volume_texture,
                volume_sampler
            );
            
            // Front-to-back compositing
            let sample_opacity = 1.0 - exp(-transfer_color.a * step_size);

            accumulated_color += (1.0 - accumulated_opacity) * shaded_color * sample_opacity;
            accumulated_opacity += (1.0 - accumulated_opacity) * sample_opacity;
        }

        current_distance += step_size;
    }
    
    // Final color output
    textureStore(output_texture, vec2<u32>(global_id.x, global_id.y),
        vec4<f32>(accumulated_color, 1.0));

    // debug texture
    textureStore(debug_texture, vec2<u32>(global_id.x, global_id.y),
        vec4<f32>(ray_direction, 1.0));
}
