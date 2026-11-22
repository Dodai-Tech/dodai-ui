#![no_main]
use dodai_ui_core::{Color, TonalPalette};
use libfuzzer_sys::fuzz_target;

// Colour maths and tonal palettes accept any channel and float values.
fuzz_target!(|d: ([u8; 4], [u8; 4], f32, f32, f32)| {
    let (a, b, t, chroma, deg) = d;
    let (a, b) = (Color::rgba(a[0], a[1], a[2], a[3]), Color::rgba(b[0], b[1], b[2], b[3]));
    let _ = (a.mix(b, t), a.lighten(t), a.darken(t), a.with_alpha(t));
    let _ = (a.to_hex(), a.to_css(), a.to_f32(), a.luminance(), a.contrast(b), a.to_ansi256());
    let p = TonalPalette::from_seed(a, chroma).rotated(deg);
    let _ = (p.tone(t), p.tone(0.0), p.tone(100.0));
});
