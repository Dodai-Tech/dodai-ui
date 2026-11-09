// Chamfered (cut-corner) surface: fill + border, antialiased, all from uniforms, and a hard
// offset shadow of the same outline (a box shadow is a rectangle: the cuts would show through).
// Mirrors `dodai_bevy::surface::{cut_distance, shade_rect}` (Rust, unit-tested).
import bevy_render::view::View;
import bevy_ui_render::ui_vertex_output::UiVertexOutput;

@group(0) @binding(0) var<uniform> view: View;

@group(1) @binding(0) var<uniform> fill: vec4<f32>;
@group(1) @binding(1) var<uniform> border: vec4<f32>;
// x = border width px, y = chamfer px, z = 1.0 for concave bites instead of cuts,
// w = physical px per logical px (0 = 1; the node size arrives in physical px)
@group(1) @binding(2) var<uniform> params: vec4<f32>;
// 1.0 where the corner is cut: top-left, top-right, bottom-right, bottom-left
@group(1) @binding(3) var<uniform> corners: vec4<f32>;
// The hard shadow's colour (alpha 0 = none).
@group(1) @binding(4) var<uniform> shadow: vec4<f32>;
// xy = the hard shadow's offset px, z = the busy shimmer band's left edge, w = its width
// (fractions of the width; w = 0: no shimmer). Mirrors `dodai_core::motion::shimmer_stops`.
@group(1) @binding(5) var<uniform> shade: vec4<f32>;
// The busy shimmer's colour.
@group(1) @binding(6) var<uniform> sheen: vec4<f32>;

const K: f32 = 0.70710678;

fn scale() -> f32 {
    return select(params.w, 1.0, params.w <= 0.0);
}

// The quad grows towards the shadow so the offset copy is drawn outside the node too.
@vertex
fn vertex(
    @location(0) vertex_position: vec3<f32>,
    @location(1) vertex_uv: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) border_widths: vec4<f32>,
    @location(4) border_radius_x: vec4<f32>,
    @location(5) border_radius_y: vec4<f32>,
) -> UiVertexOutput {
    var out: UiVertexOutput;
    var o = vec2<f32>(0.0);
    if shadow.a > 0.0 { o = shade.xy * scale(); }
    let q = vertex_uv * size;
    let lo = min(o, vec2<f32>(0.0));
    let hi = size + max(o, vec2<f32>(0.0));
    let q2 = select(lo, hi, vertex_uv > vec2<f32>(0.5));
    out.uv = q2 / max(size, vec2<f32>(1.0));
    out.position = view.clip_from_world * vec4<f32>(vertex_position.xy + (q2 - q), vertex_position.z, 1.0);
    out.size = size;
    out.border_widths = border_widths;
    out.border_radius_x = border_radius_x;
    out.border_radius_y = border_radius_y;
    return out;
}

// Signed distance into the outline (negative outside).
fn dist(p: vec2<f32>, size: vec2<f32>) -> f32 {
    let c = min(params.y * scale(), min(size.x, size.y) / 3.0);
    var d = min(min(p.x, size.x - p.x), min(p.y, size.y - p.y));
    if params.z > 0.5 {
        if corners.x > 0.5 { d = min(d, length(p) - c); }
        if corners.y > 0.5 { d = min(d, length(p - vec2<f32>(size.x, 0.0)) - c); }
        if corners.z > 0.5 { d = min(d, length(p - size) - c); }
        if corners.w > 0.5 { d = min(d, length(p - vec2<f32>(0.0, size.y)) - c); }
        return d;
    }
    if corners.x > 0.5 { d = min(d, (p.x + p.y - c) * K); }
    if corners.y > 0.5 { d = min(d, ((size.x - p.x) + p.y - c) * K); }
    if corners.z > 0.5 { d = min(d, ((size.x - p.x) + (size.y - p.y) - c) * K); }
    if corners.w > 0.5 { d = min(d, (p.x + (size.y - p.y) - c) * K); }
    return d;
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let size = in.size;
    let p = in.uv * size;
    let d = dist(p, size);
    let cover = clamp(d + 0.5, 0.0, 1.0);
    let inner = clamp(d - params.x * scale() + 0.5, 0.0, 1.0);
    var rgb = mix(border.rgb, fill.rgb, inner);
    var a = mix(border.a, fill.a, inner) * cover;
    if shade.w > 0.0 {
        // The shimmer band over the face: transparent at its left edge, full at its right.
        let u = (p.x / size.x - shade.z) / shade.w;
        if u >= 0.0 && u <= 1.0 {
            let ba = sheen.a * u * cover;
            let na = ba + a * (1.0 - ba);
            if na > 0.0 {
                rgb = (sheen.rgb * ba + rgb * a * (1.0 - ba)) / na;
            }
            a = na;
        }
    }
    if shadow.a <= 0.0 {
        return vec4<f32>(rgb, a);
    }
    // The face over its shadow.
    let sa = shadow.a * clamp(dist(p - shade.xy * scale(), size) + 0.5, 0.0, 1.0) * (1.0 - a);
    let out_a = a + sa;
    if out_a <= 0.0 {
        return vec4<f32>(0.0);
    }
    return vec4<f32>((rgb * a + shadow.rgb * sa) / out_a, out_a);
}
