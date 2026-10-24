// CRT scanline overlay: dark horizontal lines every `params.x` px.
#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var<uniform> color: vec4<f32>;
// x = line period px, y = fraction of the period covered by the line
@group(1) @binding(1) var<uniform> params: vec4<f32>;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let y = in.uv.y * in.size.y;
    let line = step(fract(y / max(params.x, 1.0)), params.y);
    return vec4<f32>(color.rgb, color.a * line);
}
