//! The drawing helpers instruments share with charts: geometry from `dodai-charts-core` and the
//! glow from `dodai-gpui`.

pub(crate) use dodai_charts_core::{fmt_num, norm, polar, turn_arc};
pub(crate) use dodai_gpui::plot::{glow_stroke, rect, ChartGlow};

use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{Hsla, Pixels, Point, Window};

/// Paint a sunburst finish on a disc of radius `r` round `(cx, cy)`: faint rays round
/// the centre, as on a brushed dial.
pub(crate) fn sunburst(w: &mut Window, o: Point<Pixels>, (cx, cy): (f32, f32), r: f32, ink: Hsla) {
    let hair = (r * 0.004).max(0.5);
    for i in 0..180 {
        let a = i as f32 / 180.0;
        stroke(
            w,
            o,
            &[polar(cx, cy, r * 0.3, a), polar(cx, cy, r * 0.98, a)],
            hair,
            scale_alpha(ink, 0.025),
            false,
        );
    }
}

/// Paint a polished hub of radius `r` at `(cx, cy)`: a bright cap, a ring of the face colour
/// and a centre in the hand's colour.
pub(crate) fn hub(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    (cap, face, hand): (Hsla, Hsla, Hsla),
) {
    let disc = |r: f32| -> Vec<(f32, f32)> {
        (0..=32)
            .map(|i| polar(cx, cy, r, i as f32 / 32.0))
            .collect()
    };
    fill(w, o, &disc(r), cap);
    fill(w, o, &disc(r * 0.72), face);
    fill(w, o, &disc(r * 0.32), hand);
}
