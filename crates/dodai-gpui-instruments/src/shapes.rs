//! Geometry and frames the detailed looks share: circles, bands, tapered needles, chamfered
//! boxes, the knurled chronograph bezel and the heads-up ring of dashes and brackets. Pure
//! geometry functions return points; the frame painters take their colours from an [`Ink`].

use crate::draw::{polar, turn_arc};
use dodai_gpui::hsla;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
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
    pub ok: Hsla,
    pub accent: Hsla,
    /// The page background: the darkest panel in a dark theme.
    pub ground: Hsla,
    /// Text on the primary colour.
    pub on_primary: Hsla,
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
            ok: hsla(p.success),
            accent: hsla(p.accent),
            ground: hsla(p.background),
            on_primary: hsla(p.on_primary),
        }
    }

    /// The ink a lit display draws with once its value is past a limit: its lighting colours
    /// (primary, info and accent) all `col`, so every glow, accent and legend takes the limit's
    /// colour while the text, the glass and the zones stay. Pure.
    pub(crate) fn lit(self, col: Hsla) -> Self {
        Self {
            primary: col,
            info: col,
            accent: col,
            ..self
        }
    }
}

/// Unit vector of a turn (0 = up, clockwise), as `polar` measures it.
pub(crate) fn dir(turn: f32) -> (f32, f32) {
    let th = turn * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (th.cos(), th.sin())
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

/// Which side of a hand at `turn` faces the light from the upper left: `1.0` for the side the
/// normal `(-dy, dx)` of its direction points to, `-1.0` for the other. Pure.
pub(crate) fn lit_side(turn: f32) -> f32 {
    let (dx, dy) = dir(turn);
    // The normal (-dy, dx) faces up-left when -x - y > 0.
    if dy - dx >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

/// A polished hand: the [`needle`] outline with a soft shadow under it and a facet along its
/// ridge, the half towards the light lifted, the other left in the hand's colour.
#[derive(Clone, Copy)]
pub(crate) struct Hand {
    /// The pivot.
    pub centre: (f32, f32),
    /// The direction, in turns.
    pub turn: f32,
    /// How far the tail reaches behind the pivot, and the tip in front of it.
    pub reach: (f32, f32),
    /// Half the width at the pivot.
    pub half: f32,
    /// The hand's colour, and the colour of the light on its lit facet.
    pub colours: (Hsla, Hsla),
    /// The shadow's colour; `None` for a hand without one.
    pub shadow: Option<Hsla>,
}

impl Hand {
    /// Paint the shadow, the hand and its facet.
    pub(crate) fn paint(self, w: &mut Window, o: Point<Pixels>) {
        let (cx, cy) = self.centre;
        let (tail, tip) = self.reach;
        let (col, light) = self.colours;
        let h = self.half;
        if let Some(sh) = self.shadow {
            let (sx, sy) = (h * 0.6, h * 0.8);
            fill(w, o, &needle(cx + sx, cy + sy, self.turn, tip, tail, h), sh);
        }
        fill(w, o, &needle(cx, cy, self.turn, tip, tail, h), col);
        let (dx, dy) = dir(self.turn);
        let k = lit_side(self.turn);
        let (nx, ny) = (-dy * k, dx * k);
        let facet = [
            (cx + dx * tip, cy + dy * tip),
            (cx + nx * h, cy + ny * h),
            (cx - dx * tail + nx * h * 0.7, cy - dy * tail + ny * h * 0.7),
            (cx - dx * tail, cy - dy * tail),
        ];
        fill(w, o, &facet, scale_alpha(light, 0.2));
        stroke(
            w,
            o,
            &[
                (cx + dx * tip, cy + dy * tip),
                (cx - dx * tail, cy - dy * tail),
            ],
            (h * 0.12).max(0.5),
            scale_alpha(light, 0.3),
            false,
        );
    }
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

/// Paint the chronograph case for a disc of `s`: a polished bezel with a fine coin edge
/// between two bright hairlines, a stepped flange and a sunburst face. Returns the face radius.
pub(crate) fn chrono_case(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> f32 {
    let c = s / 2.0;
    let (r_bezel, r_face) = (c - s * 0.01, c - s * 0.06);
    let hair = (s * 0.003).max(0.6);
    fill_circle(w, o, (c, c), r_bezel, scale_alpha(ink.text, 0.07));
    let coin: Vec<Vec<(f32, f32)>> = (0..240)
        .map(|i| {
            let a = i as f32 / 240.0;
            vec![
                polar(c, c, r_face + s * 0.018, a),
                polar(c, c, r_bezel - s * 0.006, a),
            ]
        })
        .collect();
    dodai_gpui::paint::stroke_batch(w, o, &coin, hair, scale_alpha(ink.text, 0.13), false);
    stroke_circle(
        w,
        o,
        (c, c),
        r_bezel,
        hair * 1.6,
        scale_alpha(ink.text, 0.7),
    );
    stroke_circle(
        w,
        o,
        (c, c),
        r_face + s * 0.016,
        hair,
        scale_alpha(ink.text, 0.45),
    );
    fill_circle(w, o, (c, c), r_face, ink.face);
    crate::draw::sunburst(w, o, (c, c), r_face, ink.text);
    stroke_circle(w, o, (c, c), r_face, hair, ink.line);
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
    let hair = (s * 0.003).max(0.6);
    let mut dashes = crate::draw::Strokes::default();
    for i in 0..180 {
        let a = i as f32 / 180.0;
        let alpha = if i % 15 == 0 { 0.6 } else { 0.22 };
        dashes.add(
            turn_arc(c, c, r_out, a, a + 0.0025),
            hair,
            scale_alpha(ink.text, alpha),
        );
    }
    dashes.paint(w, o);
    stroke_circle(
        w,
        o,
        (c, c),
        r_out - s * 0.012,
        hair * 0.6,
        scale_alpha(ink.text, 0.12),
    );
    for q in [0.125_f32, 0.375, 0.625, 0.875] {
        stroke(
            w,
            o,
            &turn_arc(c, c, r_out - s * 0.025, q - 0.035, q + 0.035),
            s * 0.006,
            accent,
            false,
        );
        for e in [q - 0.035, q + 0.035] {
            stroke(
                w,
                o,
                &[
                    polar(c, c, r_out - s * 0.04, e),
                    polar(c, c, r_out - s * 0.022, e),
                ],
                s * 0.006,
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

/// A rectangle with corners rounded by `r`, as a polygon. Pure.
pub(crate) fn rounded(x: f32, y: f32, wd: f32, h: f32, r: f32) -> Vec<(f32, f32)> {
    let r = r.min(wd / 2.0).min(h / 2.0).max(0.0);
    let mut p = turn_arc(x + wd - r, y + r, r, 0.0, 0.25);
    p.extend(turn_arc(x + wd - r, y + h - r, r, 0.25, 0.5));
    p.extend(turn_arc(x + r, y + h - r, r, 0.5, 0.75));
    p.extend(turn_arc(x + r, y + r, r, 0.75, 1.0));
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lit_ink_changes_only_its_lighting() {
        let c = |h: f32| gpui::hsla(h, 0.5, 0.5, 1.0);
        let ink = Ink {
            text: c(0.0),
            muted: c(0.05),
            primary: c(0.1),
            face: c(0.15),
            line: c(0.2),
            info: c(0.25),
            warn: c(0.3),
            danger: c(0.35),
            ok: c(0.4),
            accent: c(0.45),
            ground: c(0.5),
            on_primary: c(0.55),
        };
        let lit = ink.lit(ink.danger);
        assert_eq!(
            (lit.primary, lit.info, lit.accent),
            (ink.danger, ink.danger, ink.danger)
        );
        assert_eq!(
            (lit.text, lit.muted, lit.face, lit.ground),
            (ink.text, ink.muted, ink.face, ink.ground)
        );
        assert_eq!(
            (lit.warn, lit.danger, lit.ok),
            (ink.warn, ink.danger, ink.ok)
        );
    }

    #[test]
    fn the_lit_facet_faces_the_upper_left() {
        // Pointing right the normal (-dy, dx) points down, so the other (upper) side is lit;
        // pointing left it points up; pointing up it points right, so the left side is lit.
        assert_eq!(lit_side(0.25), -1.0);
        assert_eq!(lit_side(0.75), 1.0);
        assert_eq!(lit_side(0.0), -1.0);
    }

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
