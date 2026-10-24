// Chamfered (cut-corner) surface: fill + border, antialiased, all from uniforms.
// Mirrors `dodai_bevy::surface::chamfer_distance` (Rust, unit-tested).
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(1) @binding(0) var<uniform> fill: vec4<f32>;
@group(1) @binding(1) var<uniform> border: vec4<f32>;
// x = border width px, y = chamfer px, z/w unused
@group(1) @binding(2) var<uniform> params: vec4<f32>;
// 1.0 where the corner is cut: top-left, top-right, bottom-right, bottom-left
@group(1) @binding(3) var<uniform> corners: vec4<f32>;

const K: f32 = 0.70710678;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size;
    let p = in.uv * size;
    let bw = params.x;
    let c = min(params.y, min(size.x, size.y) / 3.0);
    var d = min(min(p.x, size.x - p.x), min(p.y, size.y - p.y));
    if corners.x > 0.5 { d = min(d, (p.x + p.y - c) * K); }
    if corners.y > 0.5 { d = min(d, ((size.x - p.x) + p.y - c) * K); }
    if corners.z > 0.5 { d = min(d, ((size.x - p.x) + (size.y - p.y) - c) * K); }
    if corners.w > 0.5 { d = min(d, (p.x + (size.y - p.y) - c) * K); }
    let cover = clamp(d + 0.5, 0.0, 1.0);
    let inner = clamp(d - bw + 0.5, 0.0, 1.0);
    let rgb = mix(border.rgb, fill.rgb, inner);
    let a = mix(border.a, fill.a, inner) * cover;
    return vec4<f32>(rgb, a);
}
