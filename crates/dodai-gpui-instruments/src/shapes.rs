//! Geometry and frames the detailed looks share: circles, bands, tapered needles, chamfered
//! boxes, the knurled chronograph bezel and the heads-up ring of dashes and brackets. Pure
//! geometry functions return points; the frame painters take their colours from an [`Ink`].

use crate::draw::{polar, turn_arc};
use dodai_gpui::hsla;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{Hsla, Pixels, Point, Window};

/// The theme colours the detailed looks use.
#[derive(Clone, Copy)]
pub(crate) struct Ink {
    pub text: Hsla,
    pub muted: Hsla,
    pub primary: Hsla,
    pub face: Hsla,
    pub line: Hsla,
    pub info: Hsla,
    pub warn: Hsla,
    pub danger: Hsla,
}

impl Ink {
    pub(crate) fn of(t: &dodai_core::Theme) -> Self {
        let p = &t.palette;
        Self {
            text: hsla(p.text),
            muted: hsla(p.text_muted),
            primary: hsla(p.primary),
            face: hsla(p.surface_raised),
            line: hsla(p.border),
            info: hsla(p.info),
            warn: hsla(p.warning),
            danger: hsla(p.danger),
        }
    }
}

/// Unit vector of a turn (0 = up, clockwise), as `polar` measures it.
pub(crate) fn dir(turn: f32) -> (f32, f32) {
    let th = turn * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (th.cos(), th.sin())
}

/// A closed circle polygon.
pub(crate) fn circle(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    turn_arc(cx, cy, r, 0.0, 1.0)
}

/// A band between two radii over `a..b` turns, as a fillable polygon.
pub(crate) fn band(cx: f32, cy: f32, r_in: f32, r_out: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let mut p = turn_arc(cx, cy, r_out, a, b);
    let mut inner = turn_arc(cx, cy, r_in, a, b);
    inner.reverse();
    p.extend(inner);
    p
}

/// A tapered needle from `tail` behind the pivot to `tip`, `half` wide at the pivot.
pub(crate) fn needle(
    cx: f32,
    cy: f32,
    turn: f32,
    tip: f32,
    tail: f32,
    half: f32,
) -> Vec<(f32, f32)> {
    let (dx, dy) = dir(turn);
    let (nx, ny) = (-dy, dx);
    vec![
        (cx + dx * tip, cy + dy * tip),
        (cx + nx * half, cy + ny * half),
        (
            cx - dx * tail + nx * half * 0.7,
            cy - dy * tail + ny * half * 0.7,
        ),
        (
            cx - dx * tail - nx * half * 0.7,
            cy - dy * tail - ny * half * 0.7,
        ),
        (cx - nx * half, cy - ny * half),
    ]
}

/// A rectangle with its four corners cut by `k`.
pub(crate) fn chamfered(x: f32, y: f32, w: f32, h: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x + k, y),
        (x + w - k, y),
        (x + w, y + k),
        (x + w, y + h - k),
        (x + w - k, y + h),
        (x + k, y + h),
        (x, y + h - k),
        (x, y + k),
    ]
}

/// A four-pointed star (a compass point) of `long` and `short` radius turned by `turn`.
pub(crate) fn star(
    cx: f32,
    cy: f32,
    long: f32,
    short: f32,
    turn: f32,
    points: usize,
) -> Vec<(f32, f32)> {
    let n = points.max(2) * 2;
    (0..n)
        .map(|i| {
            let r = if i % 2 == 0 { long } else { short };
            polar(cx, cy, r, turn + i as f32 / n as f32)
        })
        .collect()
}

/// Paint the chronograph case for a disc of `s`: a knurled bezel and the face. Returns the
/// face radius.
pub(crate) fn chrono_case(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> f32 {
    let c = s / 2.0;
    let (r_bezel, r_face) = (c - s * 0.01, c - s * 0.06);
    fill(w, o, &circle(c, c, r_bezel), scale_alpha(ink.text, 0.08));
    for i in 0..120 {
        let a = i as f32 / 120.0;
        stroke(
            w,
            o,
            &[polar(c, c, r_face + s * 0.008, a), polar(c, c, r_bezel, a)],
            s * 0.004,
            scale_alpha(ink.text, 0.22),
            false,
        );
    }
    fill(w, o, &circle(c, c, r_face), ink.face);
    fill(w, o, &circle(c, c, r_face), scale_alpha(ink.text, 0.04));
    stroke(w, o, &circle(c, c, r_face), s * 0.006, ink.line, true);
    r_face
}

/// A soft crescent of light over the top of a chronograph face of radius `r_face`.
pub(crate) fn chrono_glass(w: &mut Window, o: Point<Pixels>, s: f32, r_face: f32, ink: Ink) {
    let c = s / 2.0;
    let mut glass = turn_arc(c, c, r_face * 0.97, 0.8, 1.2);
    glass.extend(turn_arc(c, c + r_face * 0.45, r_face * 0.9, 1.15, 0.85));
    fill(w, o, &glass, scale_alpha(ink.text, 0.035));
}

/// Paint the heads-up ring for a square of `s`: fine dashes and four brackets in `accent`.
/// Returns the radius just inside the brackets.
pub(crate) fn hud_ring(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink, accent: Hsla) -> f32 {
    let c = s / 2.0;
    let r_out = c - s * 0.015;
    for i in 0..96 {
        let a = i as f32 / 96.0;
        stroke(
            w,
            o,
            &turn_arc(c, c, r_out, a, a + 0.005),
            s * 0.006,
            scale_alpha(ink.text, 0.35),
            false,
        );
    }
    for q in [0.125_f32, 0.375, 0.625, 0.875] {
        stroke(
            w,
            o,
            &turn_arc(c, c, r_out - s * 0.025, q - 0.035, q + 0.035),
            s * 0.012,
            accent,
            false,
        );
        for e in [q - 0.035, q + 0.035] {
            stroke(
                w,
                o,
                &[
                    polar(c, c, r_out - s * 0.045, e),
                    polar(c, c, r_out - s * 0.019, e),
                ],
                s * 0.012,
                accent,
                false,
            );
        }
    }
    r_out - s * 0.06
}

/// A row of `n` cells, `lit` of them in `on`, the rest dim; `x, y, w, h` in px.
pub(crate) fn cells(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    n: usize,
    lit: usize,
    on: Hsla,
    off: Hsla,
) {
    let n = n.max(1);
    let gap = wd * 0.25 / n as f32;
    let cw = (wd - gap * (n as f32 - 1.0)) / n as f32;
    for i in 0..n {
        let x0 = x + i as f32 * (cw + gap);
        let k = (cw * 0.35).min(h * 0.35);
        // Slanted cells, like a segment bar.
        let p = [(x0 + k, y), (x0 + cw + k, y), (x0 + cw, y + h), (x0, y + h)];
        fill(w, o, &p, if i < lit { on } else { off });
    }
}

/// A polyline or polygon.
pub(crate) type Points = Vec<(f32, f32)>;

/// The part of a horizontal band `y0..y1` inside a circle of radius `r` centred on `(c, c)`:
/// the fillable outline, then its left and right curved edges on their own. Empty when the band
/// misses the circle.
pub(crate) fn chord_band(c: f32, r: f32, y0: f32, y1: f32) -> (Points, Points, Points) {
    let (a, b) = (y0.max(c - r), y1.min(c + r));
    if a.is_nan() || b.is_nan() || b <= a {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    let steps = 12;
    let half = |y: f32| (r * r - (y - c) * (y - c)).max(0.0).sqrt();
    let ys: Vec<f32> = (0..=steps)
        .map(|i| a + (b - a) * i as f32 / steps as f32)
        .collect();
    let left: Vec<(f32, f32)> = ys.iter().map(|&y| (c - half(y), y)).collect();
    let right: Vec<(f32, f32)> = ys.iter().map(|&y| (c + half(y), y)).collect();
    let mut poly = left.clone();
    poly.extend(right.iter().rev());
    (poly, left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn needle_points_along_its_turn() {
        let p = needle(0.0, 0.0, 0.25, 10.0, 2.0, 1.0);
        assert!(
            (p[0].0 - 10.0).abs() < 1e-4 && p[0].1.abs() < 1e-4,
            "a quarter turn is right"
        );
        assert!(p[2].0 < 0.0 && p[3].0 < 0.0, "the tail is behind the pivot");
    }

    #[test]
    fn a_band_closes_back_along_the_inner_radius() {
        let p = band(0.0, 0.0, 5.0, 10.0, 0.0, 0.25);
        let (first, last) = (p[0], p[p.len() - 1]);
        assert!(
            (first.1 + 10.0).abs() < 1e-3,
            "starts on the outer radius at the top"
        );
        assert!(
            (last.1 + 5.0).abs() < 1e-3,
            "ends on the inner radius at the top"
        );
    }

    #[test]
    fn a_chord_band_stays_inside_the_circle() {
        let (poly, left, right) = chord_band(50.0, 40.0, 20.0, 35.0);
        assert!(!poly.is_empty());
        for (x, y) in &poly {
            let d = ((x - 50.0).powi(2) + (y - 50.0).powi(2)).sqrt();
            assert!(d <= 40.0 + 1e-3, "({x}, {y}) is outside");
        }
        assert!(
            left[0].0 > 10.0,
            "narrower than the diameter away from the middle"
        );
        assert!(right.iter().all(|p| p.0 > 50.0));
        assert!(
            chord_band(50.0, 40.0, 95.0, 99.0).0.is_empty(),
            "below the circle"
        );
    }

    #[test]
    fn chamfered_corners_are_cut() {
        let p = chamfered(0.0, 0.0, 10.0, 6.0, 2.0);
        assert_eq!(p.len(), 8);
        assert!(!p.contains(&(0.0, 0.0)), "no square corner left");
    }

    #[test]
    fn a_star_alternates_long_and_short_points() {
        let p = star(0.0, 0.0, 10.0, 2.0, 0.0, 4);
        assert_eq!(p.len(), 8);
        assert!(
            (p[0].1 + 10.0).abs() < 1e-3,
            "the first point is up and long"
        );
        let r1 = (p[1].0.powi(2) + p[1].1.powi(2)).sqrt();
        assert!((r1 - 2.0).abs() < 1e-3);
    }
}
