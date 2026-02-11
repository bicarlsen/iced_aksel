struct VertexInput{
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput{
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

// The Uniforms (Screen Size)
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    _padding1: f32,  // Padding to align to 16 bytes
    _padding2: f32,  // Padding to align to 16 bytes
}
@group(0) @binding(0) var<uniform> u_screen: Uniforms;

// Atlas
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput{
    // Initialize output
    var out: VertexOutput;

    // Convert pixels (0 to width) to NDC (-1.0, 1.0)
    // Formula: (pos / size) * 2.0 - 1.0
    var x = (input.position.x / u_screen.screen_width) * 2.0 - 1.0;
    var y = (input.position.y / u_screen.screen_height) * 2.0 - 1.0;

    // Note: GPU Y-coordinates usually go UP, but screen Y goes DOWN.
    // We flip Y here: 'y * -1.0'
    out.position = vec4<f32>(x, y * -1.0, input.position.z, 1.0);

    // 2. Pass data through to the fragment shader
    out.color = input.color;
    out.uv = input.uv;

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // 1. Look up the pixel from the texture
    let texture_color = textureSample(t_diffuse, s_diffuse, input.uv);

    // 2. Multiply by the vertex color (The Tint)
    // If texture_color is White (1,1,1,1) -> Result is in.color
    return texture_color * input.color;
}
