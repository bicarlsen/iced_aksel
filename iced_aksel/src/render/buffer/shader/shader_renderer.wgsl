struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) primitive_type: u32,
    @location(4) param0: vec4<f32>,
    @location(5) param1: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) @interpolate(flat) primitive_type: u32,
    @location(3) param0: vec4<f32>,
    @location(4) param1: vec4<f32>,
    @location(5) pixel_position: vec2<f32>,  // Screen-space pixel position for SDF
}

// Primitive type constants
const PRIM_TYPE_MSDF_TEXT: u32 = 0u;
const PRIM_TYPE_SDF_LINE: u32 = 1u;
const PRIM_TYPE_SDF_CIRCLE: u32 = 2u;
const PRIM_TYPE_SDF_ROUNDED_RECT: u32 = 3u;
const PRIM_TYPE_SDF_ELLIPSE: u32 = 4u;

// The Uniforms (Screen Size)
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    _padding1: f32,
    _padding2: f32,
}
@group(0) @binding(0) var<uniform> u_screen: Uniforms;

// Atlas
@group(0) @binding(1) var t_diffuse: texture_2d<f32>;
@group(0) @binding(2) var s_diffuse: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Convert pixels (0 to width) to NDC (-1.0, 1.0)
    var x = (input.position.x / u_screen.screen_width) * 2.0 - 1.0;
    var y = (input.position.y / u_screen.screen_height) * 2.0 - 1.0;

    // Flip Y (screen Y goes DOWN, GPU Y goes UP)
    out.position = vec4<f32>(x, y * -1.0, 0.0, 1.0);

    // Pass data to fragment shader
    out.color = input.color;
    out.uv = input.uv;
    out.primitive_type = input.primitive_type;
    out.param0 = input.param0;
    out.param1 = input.param1;
    out.pixel_position = input.position;

    return out;
}

// ============================================================================
// SDF Helper Functions
// ============================================================================

// Median function for MSDF
fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

// Rotate a point around a center using precomputed cos/sin
fn rotate_point(p: vec2<f32>, center: vec2<f32>, cos_angle: f32, sin_angle: f32) -> vec2<f32> {
    let pc = p - center;
    return vec2<f32>(
        pc.x * cos_angle - pc.y * sin_angle,
        pc.x * sin_angle + pc.y * cos_angle
    ) + center;
}

// Distance from point to line segment
fn distance_to_line_segment(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let pa = p - a;
    let ba = b - a;
    let h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h);
}

// SDF for circle
fn sdf_circle(p: vec2<f32>, center: vec2<f32>, radius: f32) -> f32 {
    return length(p - center) - radius;
}

// SDF for ellipse
fn sdf_ellipse(p: vec2<f32>, center: vec2<f32>, radii: vec2<f32>) -> f32 {
    let pc = p - center;
    let k0 = length(pc / radii);
    let k1 = length(pc / (radii * radii));
    return k0 * (k0 - 1.0) / k1;
}

// SDF for rounded rectangle
fn sdf_rounded_rect(p: vec2<f32>, center: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let pc = abs(p - center) - half_size + vec2<f32>(radius, radius);
    return length(max(pc, vec2<f32>(0.0, 0.0))) + min(max(pc.x, pc.y), 0.0) - radius;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var final_color: vec4<f32>;

    // Branch based on primitive type
    if (input.primitive_type == PRIM_TYPE_MSDF_TEXT) {
        // Text/texture rendering (supports MSDF, SDF, and raster glyphs)
        let sampled = textureSample(t_diffuse, s_diffuse, input.uv);

        // Check if this is a white pixel (for shapes) - all RGB at 1.0
        if (sampled.r > 0.99 && sampled.g > 0.99 && sampled.b > 0.99) {
            // Legacy white pixel shape rendering
            final_color = input.color * sampled.a;
        } else if (sampled.r == sampled.g && sampled.g == sampled.b) {
            // SDF glyph (all RGB channels equal and non-white)
            let dist = sampled.r;

            // Standard MSDF formula using derivatives
            // unitRange = spread (in pixels) / atlas size = spread in UV coordinates
            let unit_range = vec2<f32>(4.0 / 1024.0);

            // Calculate how many screen pixels per UV unit
            // fwidth gives UV change per screen pixel, so 1/fwidth gives screen pixels per UV
            let screen_tex_size = 1.0 / fwidth(input.uv);

            // Calculate screen-space range: spread (in UV) * screen pixels per UV
            let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);

            // Convert distance to alpha with antialiasing
            // Values > 0.5 are inside the glyph, < 0.5 are outside
            let screen_dist = screen_px_range * (dist - 0.5);
            let alpha = clamp(screen_dist + 0.5, 0.0, 1.0);

            // Ensure fully transparent outside the glyph
            final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
        } else {
            // MSDF glyph (RGB channels differ - contains multi-channel distance field)
            // Use median of RGB channels for better quality
            let dist = median(sampled.r, sampled.g, sampled.b);

            // Standard MSDF formula using derivatives
            // unitRange = spread (in pixels) / atlas size = spread in UV coordinates
            let unit_range = vec2<f32>(4.0 / 1024.0);

            // Calculate how many screen pixels per UV unit
            // fwidth gives UV change per screen pixel, so 1/fwidth gives screen pixels per UV
            let screen_tex_size = 1.0 / fwidth(input.uv);

            // Calculate screen-space range: spread (in UV) * screen pixels per UV
            let screen_px_range = max(0.5 * dot(unit_range, screen_tex_size), 1.0);

            // Convert distance to alpha with antialiasing
            // MSDF uses the same 0.5 threshold as SDF
            let screen_dist = screen_px_range * (dist - 0.5);
            let alpha = clamp(screen_dist + 0.5, 0.0, 1.0);

            // Apply color with MSDF-based alpha
            final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
        }
    } else if (input.primitive_type == PRIM_TYPE_SDF_LINE) {
        // SDF line rendering with rotation
        let line_start = input.param0.xy;
        let line_end = input.param0.zw;
        let line_width = input.param1.x;
        let cos_angle = input.param1.y;
        let sin_angle = input.param1.z;

        // Calculate line center for rotation pivot
        let line_center = (line_start + line_end) * 0.5;

        // Apply inverse rotation to pixel position to get local coordinates
        let local_pos = rotate_point(input.pixel_position, line_center, cos_angle, -sin_angle);

        let dist = distance_to_line_segment(local_pos, line_start, line_end) - line_width * 0.5;
        let alpha = clamp(0.5 - dist, 0.0, 1.0);

        final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
    } else if (input.primitive_type == PRIM_TYPE_SDF_CIRCLE) {
        // SDF circle rendering with rotation
        // Note: Circle is rotation-invariant, but we support it for consistency
        let center = input.param0.xy;
        let radius = input.param0.z;
        let cos_angle = input.param1.x;
        let sin_angle = input.param1.y;

        // Apply inverse rotation to pixel position
        let local_pos = rotate_point(input.pixel_position, center, cos_angle, -sin_angle);

        let dist = sdf_circle(local_pos, center, radius);
        let alpha = clamp(0.5 - dist, 0.0, 1.0);

        final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
    } else if (input.primitive_type == PRIM_TYPE_SDF_ELLIPSE) {
        // SDF ellipse rendering with rotation
        let center = input.param0.xy;
        let radii = input.param0.zw;
        let cos_angle = input.param1.x;
        let sin_angle = input.param1.y;

        // Apply inverse rotation to pixel position
        let local_pos = rotate_point(input.pixel_position, center, cos_angle, -sin_angle);

        let dist = sdf_ellipse(local_pos, center, radii);
        let alpha = clamp(0.5 - dist, 0.0, 1.0);

        final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
    } else if (input.primitive_type == PRIM_TYPE_SDF_ROUNDED_RECT) {
        // SDF rounded rectangle rendering with rotation
        let center = input.param0.xy;
        let half_size = input.param0.zw;
        let corner_radius = input.param1.x;
        let cos_angle = input.param1.y;
        let sin_angle = input.param1.z;

        // Apply inverse rotation to pixel position
        let local_pos = rotate_point(input.pixel_position, center, cos_angle, -sin_angle);

        let dist = sdf_rounded_rect(local_pos, center, half_size, corner_radius);
        let alpha = clamp(0.5 - dist, 0.0, 1.0);

        final_color = vec4<f32>(input.color.rgb, input.color.a * alpha);
    } else {
        // Fallback: legacy texture rendering
        let texture_color = textureSample(t_diffuse, s_diffuse, input.uv);
        final_color = texture_color * input.color;
    }

    return final_color;
}
