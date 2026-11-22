#![no_main]
use dodai_ui_core::{BorderTint, ControlColour, CornerStyle, GradientMode, ShadowDir, ShapeTune};
use libfuzzer_sys::fuzz_target;

// Any tuning (NaN, negative, huge) applied to every built-in theme must not panic.
fuzz_target!(|d: (f32, [f32; 3], Option<f32>, Option<f32>, u8, bool, u8, u8, [f32; 2], f32)| {
    let (roundness, groups, width, shadow, tint, outline_all, dir, style, size, density) = d;
    let tune = ShapeTune {
        roundness,
        groups,
        border_width: width,
        tint: [BorderTint::Design, BorderTint::Strong, BorderTint::Accent, BorderTint::Text][tint as usize % 4],
        outline_all,
        hard_shadow: shadow,
        shadow_dir: [ShadowDir::BottomRight, ShadowDir::BottomLeft, ShadowDir::TopRight, ShadowDir::TopLeft][dir as usize % 4],
        corner_style: [None, Some(CornerStyle::Round), Some(CornerStyle::Cut), Some(CornerStyle::Scoop)][style as usize % 4],
        gradient: GradientMode::ALL[(style as usize / 4) % 3],
        control_colour: ControlColour::ALL[(dir as usize / 4) % 4],
        density,
        cut_gap: width.map(|w| w * 0.5),
    };
    for t in dodai_ui_core::themes::all() {
        let tuned = t.tuned(&tune);
        let _ = tuned.scale(dodai_ui_core::Scale::ListMax);
        // The tune read at paint time (tick gap, outlines, shadows, panel frame) never panics.
        let _ = (tuned.tick_gap(), tuned.hard_offset(), tuned.panel_frame(outline_all));
        let _ = tuned.outline(Some(tuned.palette.border), true);
        // Scooped and cut outlines must stay finite for any box size.
        let pts = tuned.shape.outline(size[0], size[1]);
        assert!(pts.iter().all(|p| p.0.is_finite() && p.1.is_finite()) || !(size[0].is_finite() && size[1].is_finite()));
    }
});
