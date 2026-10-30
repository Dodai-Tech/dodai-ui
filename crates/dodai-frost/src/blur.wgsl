// One box-blur pass, one invocation per row (horizontal) or column. Integer maths identical to
// the CPU `box_pass`, so both backends give the same bytes.
struct P { w: u32, h: u32, r: u32, horizontal: u32 }
@group(0) @binding(0) var<storage, read> src: array<u32>;
@group(0) @binding(1) var<storage, read_write> dst: array<u32>;
@group(0) @binding(2) var<uniform> p: P;

fn unpack(v: u32) -> vec4<u32> {
    return vec4<u32>(v & 255u, (v >> 8u) & 255u, (v >> 16u) & 255u, v >> 24u);
}
fn pack(v: vec4<u32>) -> u32 {
    return v.x | (v.y << 8u) | (v.z << 16u) | (v.w << 24u);
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let horizontal = p.horizontal == 1u;
    let line = id.x;
    var lines = p.w;
    var len = p.h;
    var step = p.w;
    var base = line;
    if horizontal {
        lines = p.h;
        len = p.w;
        step = 1u;
        base = line * p.w;
    }
    if line >= lines { return; }
    let r = i32(p.r);
    let last = i32(len) - 1;
    let win = 2u * p.r + 1u;
    var sum = vec4<u32>(0u);
    for (var i = -r; i <= r; i++) {
        sum += unpack(src[base + u32(clamp(i, 0, last)) * step]);
    }
    for (var i = 0; i < i32(len); i++) {
        dst[base + u32(i) * step] = pack((sum + vec4<u32>(win / 2u)) / vec4<u32>(win));
        sum += unpack(src[base + u32(clamp(i + r + 1, 0, last)) * step]);
        sum -= unpack(src[base + u32(clamp(i - r, 0, last)) * step]);
    }
}
