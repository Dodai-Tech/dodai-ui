//! The parts of the [`Look::Cockpit`] instruments: a display page with soft keys on its bezel,
//! engine-indication arcs with a filled sweep, boxed readouts, bugs and a heading arc. Pure
//! geometry returns points; the painters take every colour from an [`Ink`].
//!
//! [`Look::Cockpit`]: crate::look::Look::Cockpit

use crate::draw::{polar, turn_arc};
use crate::shapes::Ink;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{Hsla, Pixels, Point, Window};

/// The rectangle `(x, y, wd, h)` as a polygon. Pure.
pub(crate) fn rect(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    vec![(x, y), (x + wd, y), (x + wd, y + h), (x, y + h)]
}

/// Paint a display page filling a square of `s`: a bezel with a row of soft keys on each side and
/// the dark glass. Returns the glass rectangle.
pub(crate) fn page(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> (f32, f32, f32, f32) {
    let bezel = rect(0.0, 0.0, s, s);
    fill(w, o, &bezel, ink.face);
    fill(w, o, &bezel, scale_alpha(ink.text, 0.08));
    stroke(w, o, &bezel, s * 0.004, ink.line, true);
    let k = s * 0.045;
    let glass = (k, k, s - 2.0 * k, s - 2.0 * k);
    let g = rect(glass.0, glass.1, glass.2, glass.3);
    fill(w, o, &g, ink.face);
    fill(w, o, &g, scale_alpha(ink.line, 0.25));
    stroke(w, o, &g, s * 0.003, scale_alpha(ink.text, 0.25), true);
    // Soft keys: short bars on the left and right bezel.
    for i in 0..5 {
        let y = s * (0.2 + 0.15 * i as f32);
        for x in [k * 0.25, s - k * 0.75] {
            fill(
                w,
                o,
                &rect(x, y - s * 0.022, k * 0.5, s * 0.044),
                scale_alpha(ink.text, 0.14),
            );
        }
    }
    glass
}

/// The turn of the engine-indication arc at `f` of the scale: it opens at nine o'clock and runs
/// clockwise over the top to just past three o'clock. Pure.
pub(crate) fn arc_turn(f: f32) -> f32 {
    ARC.0 + ARC.1 * f.clamp(0.0, 1.0)
}

/// Start and span of the engine-indication arc, in turns.
pub(crate) const ARC: (f32, f32) = (0.75, 0.55);

/// The filled sweep of an engine-indication arc from the start to `f`: a pie slice. Pure.
pub(crate) fn sweep(cx: f32, cy: f32, r: f32, f: f32) -> Vec<(f32, f32)> {
    let mut p = vec![(cx, cy)];
    p.extend(turn_arc(cx, cy, r, ARC.0, arc_turn(f)));
    p
}

/// Paint an engine-indication arc around `(cx, cy)` of radius `r`: the sweep to `f` filled, the
/// white arc with its majors, the zones `(from, to, colour)` as wide bands with a radial limit
/// line, and a thin needle in `col`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn engine_arc(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    f: f32,
    majors: usize,
    zones: &[(f32, f32, Hsla)],
    col: Hsla,
    ink: Ink,
) {
    let lw = (r * 0.03).max(1.0);
    if f > 0.0 {
        fill(w, o, &sweep(cx, cy, r, f), scale_alpha(ink.text, 0.16));
    }
    stroke(
        w,
        o,
        &turn_arc(cx, cy, r, ARC.0, ARC.0 + ARC.1),
        lw,
        ink.text,
        false,
    );
    for &(a, z, zc) in zones {
        stroke(
            w,
            o,
            &turn_arc(cx, cy, r + lw * 1.6, arc_turn(a), arc_turn(z)),
            lw * 2.2,
            zc,
            false,
        );
        let t = arc_turn(a);
        stroke(
            w,
            o,
            &[polar(cx, cy, r * 0.82, t), polar(cx, cy, r + lw * 2.8, t)],
            lw * 1.2,
            zc,
            false,
        );
    }
    let majors = majors.max(1);
    for i in 0..=majors {
        let t = arc_turn(i as f32 / majors as f32);
        stroke(
            w,
            o,
            &[polar(cx, cy, r, t), polar(cx, cy, r * 0.88, t)],
            lw,
            ink.text,
            false,
        );
    }
    let t = arc_turn(f);
    stroke(
        w,
        o,
        &[polar(cx, cy, r * 0.12, t), polar(cx, cy, r * 1.04, t)],
        lw * 1.4,
        col,
        false,
    );
}

/// Paint a readout box: the glass colour inside a white frame, an optional pointer notch on its
/// left edge pointing at a scale.
pub(crate) fn readout_box(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    notch: bool,
    frame: Hsla,
    ink: Ink,
) {
    let mut p = vec![(x, y), (x + wd, y), (x + wd, y + h), (x, y + h)];
    if notch {
        let k = h * 0.3;
        p.extend([
            (x, y + h / 2.0 + k),
            (x - k, y + h / 2.0),
            (x, y + h / 2.0 - k),
        ]);
    }
    fill(w, o, &p, ink.face);
    stroke(w, o, &p, (h * 0.05).max(1.0), frame, true);
}

/// A bug (a target marker) at `(x, y)` pointing down at a scale: a notched block `k` wide. Pure.
pub(crate) fn bug(x: f32, y: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x - k, y - k * 0.8),
        (x + k, y - k * 0.8),
        (x + k, y),
        (x + k * 0.35, y),
        (x, y - k * 0.4),
        (x - k * 0.35, y),
        (x - k, y),
    ]
}

/// A dotted leader from `x0` to `x1` at `y`, as dot centres every `step`. Pure.
pub(crate) fn leader(x0: f32, x1: f32, y: f32, step: f32) -> Vec<(f32, f32)> {
    let step = step.max(1.0);
    let n = ((x1 - x0) / step).floor().max(0.0) as usize;
    (0..n).map(|i| (x0 + step * (i as f32 + 0.5), y)).collect()
}

/// Paint a horizontal scale from `x0` to `x1` at `y` with a filled triangle pointer at `f`, in
/// `col`, ticks at quarters.
pub(crate) fn pointer_scale(
    w: &mut Window,
    o: Point<Pixels>,
    (x0, x1, y): (f32, f32, f32),
    f: f32,
    k: f32,
    col: Hsla,
    ink: Ink,
) {
    stroke(w, o, &[(x0, y), (x1, y)], k * 0.25, ink.text, false);
    for i in 0..=4 {
        let x = x0 + (x1 - x0) * i as f32 / 4.0;
        stroke(w, o, &[(x, y), (x, y + k * 0.8)], k * 0.25, ink.text, false);
    }
    let x = x0 + (x1 - x0) * f.clamp(0.0, 1.0);
    fill(
        w,
        o,
        &[(x, y - k * 0.2), (x - k, y - k * 1.6), (x + k, y - k * 1.6)],
        col,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_arc_opens_at_nine_and_runs_clockwise() {
        assert_eq!(arc_turn(0.0), 0.75);
        assert!((arc_turn(1.0) - 1.3).abs() < 1e-6);
        assert_eq!(arc_turn(2.0), arc_turn(1.0), "the arc stops at its end");
        let pie = sweep(0.0, 0.0, 10.0, 0.5);
        assert_eq!(pie[0], (0.0, 0.0), "the sweep starts at the centre");
        assert!(pie.len() > 3);
    }

    #[test]
    fn a_leader_fits_between_its_ends() {
        let dots = leader(0.0, 10.0, 5.0, 2.0);
        assert_eq!(dots.len(), 5);
        assert!(dots.iter().all(|d| d.0 > 0.0 && d.0 < 10.0 && d.1 == 5.0));
        assert!(leader(5.0, 1.0, 0.0, 1.0).is_empty());
    }

    #[test]
    fn a_bug_points_down_at_its_spot() {
        let b = bug(10.0, 20.0, 4.0);
        assert!(b.iter().all(|p| p.1 <= 20.0));
        assert!(b.iter().any(|p| (p.0 - 10.0).abs() < 1e-6));
    }
}
