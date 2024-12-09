// Render the input texture to the screen

@group(0) @binding(0)
var input_sampler: sampler;
@group(0) @binding(1)
var input_texture: texture_2d<f32>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {

  // Draw a full-screen quad
  //
  //  (-1, 1)                  (1, 1)
  //     +------------------------+
  //     |       Triangle 2       /|
  //     |                      /  |
  //     |                   /     |
  //     |                /        |
  //     |             /           |
  //     |          /              |
  //     |       /                 |
  //     |    /                    |
  //     | /      Triangle 1       |
  //     +-------------------------+
  //  (-1, -1)                  (1, -1)

    var positions = array<vec2<f32>, 6>(
        vec2<f32>(1.0, 1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, 1.0)
    );

    return vec4<f32>(positions[vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = frag_coord.xy / vec2<f32>(textureDimensions(input_texture));
    return textureSample(input_texture, input_sampler, uv);
}
