struct Uniforms {
    volume_grid_resolution: vec3<f32>,
    volume_voxel_size: vec3<f32>,
    volume_grid_size: vec3<f32>,
    camera_eye: vec3<f32>,
    camera_look_at: mat4x4<f32>,
    projection_matrix: mat4x4<f32>,
    tan_camera_fov_y: f32,
    camera_aspect_ratio: f32,
    step_size: f32,
    volume_scales: vec3<f32>,
    apply_gradient_phong_shading: i32,
    blinn_phong_ka: f32,
    blinn_phong_kd: f32,
    blinn_phong_ks: f32,
    blinn_phong_shininess: f32,
    blinn_phong_ispecular: vec3<f32>,
    world_eye_pos: vec3<f32>,
    light_source_position: vec3<f32>,
    apply_occlusion: i32,
    apply_shadow: i32,
}

struct Ray {
    origin: vec3<f32>,
    direction: vec3<f32>,
}

fn ray_aabb_intersection(
    vert_eye: vec3<f32>,
    vert_dir: vec3<f32>,
    vol_scaled_dim: vec3<f32>
) -> RayIntersection {
    let inv_dir = 1.0 / vert_dir;
    let t1 = (vec3<f32>(0.0) - vert_eye) * inv_dir;
    let t2 = vol_scaled_dim - vert_eye * inv_dir;

    let tmin = max(
        max(min(t1.x, t2.x), min(t1.y, t2.y)),
        min(t1.z, t2.z)
    );

    let tmax = min(
        min(max(t1.x, t2.x), max(t1.y, t2.y)),
        max(t1.z, t2.z)
    );

    return RayIntersection(
        tmin > 0.0 && tmin <= tmax,
        tmin,
        tmax
    );
}

struct RayIntersection {
    hit: bool,
    near: f32,
    far: f32,
}

fn shade_blinn_phong(
    tpos: vec3<f32>,
    clr: vec3<f32>,
    uniforms: Uniforms,
    volume_gradient: texture_3d<f32>
) -> vec3<f32> {
    let gradient_normal = textureSampleLevel(
        volume_gradient,
        linear_sampler,
        tpos / uniforms.volume_grid_size,
        0.0
    ).xyz;

    if length(gradient_normal) > 0.0 {
        let normalized_gradient = normalize(gradient_normal);
        let wpos = tpos - (uniforms.volume_grid_size * 0.5);

        let light_direction = normalize(uniforms.light_source_position - wpos);
        let eye_direction = normalize(uniforms.camera_eye - wpos);
        let halfway_vector = normalize(eye_direction + light_direction);

        let dot_diff = max(0.0, dot(normalized_gradient, light_direction));
        let dot_spec = max(0.0, dot(halfway_vector, normalized_gradient));

        return (clr * (uniforms.blinn_phong_ka + uniforms.blinn_phong_kd * dot_diff) + uniforms.blinn_phong_ispecular * uniforms.blinn_phong_ks * pow(dot_spec, uniforms.blinn_phong_shininess));
    }

    return clr;
}

@group(0) @binding(0)
var volume_texture: texture_3d<f32>;

@group(0) @binding(1)
var transfer_function_texture: texture_1d<f32>;

@group(0) @binding(2)
var volume_gradient_texture: texture_3d<f32>;

@group(0) @binding(3)
var linear_sampler: sampler;

@group(1) @binding(0)
var output_texture: texture_storage_2d<rgba16float, write>;

@group(2) @binding(0)
var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let output_size = textureDimensions(output_texture);
    let store_pos = vec2<i32>(global_id.xy);
    
    // Check if within output texture bounds
    if store_pos.x >= i32(output_size.x) || store_pos.y >= i32(output_size.y) {
        return;
    }
    
    // Screen position
    let fpos = vec2<f32>(f32(store_pos.x) + 0.5, f32(store_pos.y) + 0.5);
    let ver_pos = (vec3<f32>(
        fpos.x / f32(output_size.x),
        fpos.y / f32(output_size.y),
        0.0
    ) * 2.0) - 1.0;
    
    // Camera direction
    let camera_dir = normalize(vec3<f32>(
        ver_pos.x * uniforms.tan_camera_fov_y * uniforms.camera_aspect_ratio,
        ver_pos.y * uniforms.tan_camera_fov_y,
        -1.0
    ) * mat3x3<f32>(uniforms.camera_look_at));
    
    // Ray intersection
    let intersection = ray_aabb_intersection(
        uniforms.camera_eye,
        camera_dir,
        uniforms.volume_grid_size
    );
    
    // If inside volume grid
    if intersection.hit {
        let distance = abs(intersection.far - intersection.near);
        var dst = vec4<f32>(vec3<f32>(0.0), 1.0);
        
        // World position at near point
        let wld_pos = uniforms.camera_eye + camera_dir * intersection.near;
        let tex_pos = wld_pos + (uniforms.volume_grid_size * 0.5);

        var s = 0.0;
        while s < distance {
            let h = min(uniforms.step_size, distance - s);
            let s_tex_pos = tex_pos + camera_dir * (s + h * 0.5);
            
            // Sample volume density
            let density = textureSampleLevel(
                volume_texture,
                linear_sampler,
                s_tex_pos / uniforms.volume_grid_size,
                0.0
            ).r;
            
            // Get color from transfer function
            var src = textureSampleLevel(
                transfer_function_texture,
                linear_sampler,
                density,
                0.0
            );
            
            // Apply gradient shading if enabled
            if uniforms.apply_gradient_phong_shading == 1 {
                src.rgb = shade_blinn_phong(
                    s_tex_pos,
                    src.rgb,
                    uniforms,
                    volume_gradient_texture
                );
            }
            
            // Front-to-back composition
            if src.a > 0.0 {
                src.a = 1.0 - exp(-src.a * h);
                src.rgb *= src.a;
                dst = dst + (1.0 - dst.a) * src;
                
                // Opacity threshold
                if dst.a > 0.99 {
                    break;
                }
            }

            s += h;
        }
        
        // Store final color
        textureStore(output_texture, vec2<u32>(global_id.x, global_id.y), dst);
    }
}

