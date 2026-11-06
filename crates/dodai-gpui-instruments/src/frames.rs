//! The frames and parts of the [`Look::Vintage`], [`Look::Segment`] and [`Look::Halo`]
//! instruments: a screwed plate with a round bezel and spade hands, an LCD module with segment
//! bars, and a halo of light with gradient arcs and glowing tips. Pure geometry returns points;
//! the painters take every colour from an [`Ink`].
//!
//! [`Look::Vintage`]: crate::look::Look::Vintage
//! [`Look::Segment`]: crate::look::Look::Segment
//! [`Look::Halo`]: crate::look::Look::Halo

use crate::draw::{polar, turn_arc};
use crate::shapes::{band, chamfered, circle, dir, Ink};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{Hsla, ParentElement, Pixels, Point, Styled, Window};

// ───────────────────────────────── Vintage ─────────────────────────────────

/// The warm face of a vintage dial: the theme's raised surface, aged slightly towards the
/// warning colour.
pub(crate) fn aged(ink: Ink) -> Hsla {
    mix(ink.face, ink.warn, 0.06)
}

/// Paint the vintage plate for a square of `s`: a bevelled panel, four slotted screws, a round
/// polished bezel and the aged face. Returns the face radius around the centre.
pub(crate) fn plate(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> f32 {
    let c = s / 2.0;
    let panel = chamfered(0.0, 0.0, s, s, s * 0.03);
    fill(w, o, &panel, ink.face);
    fill(w, o, &panel, scale_alpha(ink.text, 0.06));
    // Bevel: light on the top and left edge, shade on the bottom and right.
    let e = s * 0.006;
    stroke(
        w,
        o,
        &[
            (e, s - s * 0.03),
            (e, s * 0.03),
            (s * 0.03, e),
            (s - s * 0.03, e),
        ],
        s * 0.008,
        scale_alpha(ink.text, 0.22),
        false,
    );
    stroke(
        w,
        o,
        &[
            (s - e, s * 0.03),
            (s - e, s - s * 0.03),
            (s - s * 0.03, s - e),
            (s * 0.03, s - e),
        ],
        s * 0.008,
        scale_alpha(ink.line, 0.9),
        false,
    );
    for (i, (x, y)) in [
        (s * 0.075, s * 0.075),
        (s - s * 0.075, s * 0.075),
        (s * 0.075, s - s * 0.075),
        (s - s * 0.075, s - s * 0.075),
    ]
    .into_iter()
    .enumerate()
    {
        screw(w, o, (x, y), s * 0.022, 0.1 + i as f32 * 0.17, ink);
    }
    let r_bezel = c - s * 0.06;
    let r_face = r_bezel - s * 0.04;
    // Polished bezel: a band with a highlight on the upper left and a shade on the lower right.
    fill(
        w,
        o,
        &band(c, c, r_face, r_bezel, 0.0, 1.0),
        scale_alpha(ink.text, 0.14),
    );
    stroke(
        w,
        o,
        &turn_arc(c, c, r_bezel - s * 0.012, 0.72, 1.02),
        s * 0.014,
        scale_alpha(ink.text, 0.3),
        false,
    );
    stroke(
        w,
        o,
        &turn_arc(c, c, r_bezel - s * 0.012, 0.22, 0.52),
        s * 0.014,
        scale_alpha(ink.line, 0.8),
        false,
    );
    stroke(w, o, &circle(c, c, r_bezel), s * 0.004, ink.line, true);
    fill(w, o, &circle(c, c, r_face), aged(ink));
    stroke(
        w,
        o,
        &circle(c, c, r_face),
        s * 0.006,
        scale_alpha(ink.text, 0.35),
        true,
    );
    r_face
}

/// A slotted screw head at `at`, radius `r`, its slot turned by `turn`.
pub(crate) fn screw(w: &mut Window, o: Point<Pixels>, at: (f32, f32), r: f32, turn: f32, ink: Ink) {
    let (x, y) = at;
    fill(w, o, &circle(x, y, r), scale_alpha(ink.text, 0.18));
    stroke(w, o, &circle(x, y, r), r * 0.18, ink.line, true);
    let (dx, dy) = dir(turn);
    stroke(
        w,
        o,
        &[
            (x - dx * r * 0.75, y - dy * r * 0.75),
            (x + dx * r * 0.75, y + dy * r * 0.75),
        ],
        r * 0.28,
        scale_alpha(ink.text, 0.55),
        false,
    );
}

/// The parts of a spade (Breguet-like) hand from `(cx, cy)` at `turn`, `len` long: the shaft as a
/// polyline, the open ring near the tip as its centre and radius, and the spear tip polygon.
pub(crate) fn spade(cx: f32, cy: f32, turn: f32, len: f32) -> SpadeHand {
    let (dx, dy) = dir(turn);
    let (nx, ny) = (-dy, dx);
    let at = |d: f32| (cx + dx * d, cy + dy * d);
    let ring_r = len * 0.07;
    let ring = at(len * 0.7);
    let base = at(len * 0.7 + ring_r);
    let w = len * 0.045;
    SpadeHand {
        shaft: vec![at(-len * 0.16), at(len * 0.7 - ring_r)],
        ring: (ring.0, ring.1, ring_r),
        tip: vec![
            (base.0 + nx * w, base.1 + ny * w),
            at(len),
            (base.0 - nx * w, base.1 - ny * w),
        ],
    }
}

/// A spade hand ready to paint, see [`spade`].
pub(crate) struct SpadeHand {
    pub shaft: Vec<(f32, f32)>,
    pub ring: (f32, f32, f32),
    pub tip: Vec<(f32, f32)>,
}

impl SpadeHand {
    /// Paint the hand `width` thick in `col`, with a faint shadow.
    pub(crate) fn paint(&self, w: &mut Window, o: Point<Pixels>, width: f32, col: Hsla, ink: Ink) {
        let off = |p: &(f32, f32)| (p.0 + width * 0.8, p.1 + width * 1.1);
        let shade = scale_alpha(ink.text, 0.14);
        let shaft_s: Vec<_> = self.shaft.iter().map(off).collect();
        stroke(w, o, &shaft_s, width, shade, false);
        let (rx, ry, rr) = self.ring;
        stroke(w, o, &self.shaft, width, col, false);
        stroke(w, o, &circle(rx, ry, rr), width, col, true);
        fill(w, o, &self.tip, col);
    }
}

/// Paint a small name plate at `(x, y)`, `wd` by `h`, with two rivets.
/// How far text on a [`name_plate`] of height `h` keeps from either end, clear of its rivets.
/// Pure.
pub(crate) fn plate_inset(h: f32) -> f32 {
    h * 0.6
}

pub(crate) fn name_plate(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    ink: Ink,
) {
    let p = chamfered(x, y, wd, h, h * 0.2);
    fill(w, o, &p, scale_alpha(ink.text, 0.1));
    stroke(w, o, &p, h * 0.04, scale_alpha(ink.text, 0.3), true);
    // The rivets end before `plate_inset`, so the text never covers them.
    for rx in [x + h * 0.35, x + wd - h * 0.35] {
        fill(
            w,
            o,
            &circle(rx, y + h / 2.0, h * 0.09),
            scale_alpha(ink.text, 0.4),
        );
    }
}

/// Paint a vintage scale on a circle: fine and major marks between `r0` and `r1` across
/// `start..start + span`, `majors` intervals of `fine` marks, the major ones `major_col`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn engraved_scale(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    (start, span): (f32, f32),
    majors: usize,
    s: f32,
    ink: Ink,
) {
    let fine = 10;
    let n = majors.max(1) * fine;
    for i in 0..=n {
        let a = start + span * i as f32 / n as f32;
        let (len, wd) = if i % fine == 0 {
            (s * 0.05, s * 0.009)
        } else if i % 5 == 0 {
            (s * 0.032, s * 0.005)
        } else {
            (s * 0.018, s * 0.003)
        };
        stroke(
            w,
            o,
            &[polar(cx, cy, r - len, a), polar(cx, cy, r, a)],
            wd,
            ink.text,
            false,
        );
    }
    stroke(
        w,
        o,
        &turn_arc(cx, cy, r + s * 0.004, start, start + span),
        s * 0.004,
        ink.text,
        false,
    );
}

// ───────────────────────────────── Segment LCD ─────────────────────────────────

/// How faint an unlit segment is: brighter on light faces, where it would vanish otherwise.
pub(crate) fn ghost(on: Hsla, dark: bool) -> Hsla {
    scale_alpha(on, if dark { 0.1 } else { 0.14 })
}

/// Paint an LCD module filling `(x, y, wd, h)` of a pod of `s`: a dark bezel with corner cuts, a
/// tinted glass with scan lines and a thin inner glow. Returns the glass rectangle.
pub(crate) fn lcd(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    s: f32,
    ink: Ink,
) -> (f32, f32, f32, f32) {
    let outer = chamfered(x, y, wd, h, s * 0.05);
    fill(w, o, &outer, ink.face);
    fill(w, o, &outer, scale_alpha(ink.text, 0.1));
    stroke(w, o, &outer, s * 0.005, ink.line, true);
    let k = s * 0.035;
    let (gx, gy, gw, gh) = (x + k, y + k, wd - 2.0 * k, h - 2.0 * k);
    let glass = chamfered(gx, gy, gw, gh, s * 0.03);
    fill(w, o, &glass, ink.face);
    fill(w, o, &glass, scale_alpha(ink.primary, 0.05));
    let mut yy = gy + s * 0.012;
    while yy < gy + gh - s * 0.01 {
        stroke(
            w,
            o,
            &[(gx + s * 0.01, yy), (gx + gw - s * 0.01, yy)],
            s * 0.002,
            scale_alpha(ink.text, 0.035),
            false,
        );
        yy += s * 0.014;
    }
    stroke(w, o, &glass, s * 0.004, scale_alpha(ink.primary, 0.3), true);
    (gx, gy, gw, gh)
}

/// Seven-segment `text` in `col`, `h` tall, centred on `cx` with its top at `y`. Unlit segments
/// stay faintly visible.
pub(crate) fn seg_text(text: &str, cx: f32, y: f32, h: f32, col: Hsla) -> gpui::Div {
    let wd = seg_width(text, h);
    gpui::div()
        .absolute()
        .left(gpui::px(cx - wd / 2.0))
        .top(gpui::px(y))
        .child(
            crate::segment::SegmentDisplay::new(text.to_string())
                .height(h)
                .color(col),
        )
}

/// Width of seven-segment `text` at digit height `h`, as [`seg_text`] lays it out. Pure.
pub(crate) fn seg_width(text: &str, h: f32) -> f32 {
    text.chars()
        .map(|c| dodai_instruments_core::segments::advance(c, h))
        .sum::<f32>()
        + h * 0.1
}

/// Paint `n` segments of an arc between `r0` and `r1` over `start..start + span`, segment `i` in
/// `lit[i]` if it is some and in `off` otherwise.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_arc(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    (r0, r1): (f32, f32),
    (start, span): (f32, f32),
    lit: &[Option<Hsla>],
    off: Hsla,
) {
    let n = lit.len().max(1) as f32;
    for (i, l) in lit.iter().enumerate() {
        let a = start + span * i as f32 / n;
        let z = start + span * (i as f32 + 0.72) / n;
        let p = band(cx, cy, r0, r1, a, z);
        match l {
            Some(col) => {
                stroke(w, o, &p, (r1 - r0) * 0.22, scale_alpha(*col, 0.15), true);
                fill(w, o, &p, *col);
            }
            None => fill(w, o, &p, off),
        }
    }
}

/// The colour of every one of `n` segments of a bar lit up to `frac`: lit ones from `colour`
/// (called with the segment's middle fraction), unlit ones `None`. Pure.
pub(crate) fn lit_segments(n: usize, frac: f32, colour: impl Fn(f32) -> Hsla) -> Vec<Option<Hsla>> {
    let lit = crate::pod_math::segments_lit(frac, n);
    (0..n)
        .map(|i| (i < lit).then(|| colour((i as f32 + 0.5) / n as f32)))
        .collect()
}

/// Paint a horizontal bar of rectangular segments in `(x, y, wd, h)`, see [`seg_arc`].
pub(crate) fn seg_bar(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    lit: &[Option<Hsla>],
    off: Hsla,
) {
    let n = lit.len().max(1) as f32;
    let step = wd / n;
    for (i, l) in lit.iter().enumerate() {
        let x0 = x + step * i as f32;
        let p = [
            (x0, y),
            (x0 + step * 0.72, y),
            (x0 + step * 0.72, y + h),
            (x0, y + h),
        ];
        match l {
            Some(col) => {
                stroke(w, o, &p, h * 0.5, scale_alpha(*col, 0.18), true);
                fill(w, o, &p, *col);
            }
            None => fill(w, o, &p, off),
        }
    }
}

// ───────────────────────────────── Halo ─────────────────────────────────

/// A pill (a rectangle with round ends) at `(x, y)`, `wd` by `h`, as a polygon. Pure.
pub(crate) fn pill(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    let r = (h / 2.0).min(wd / 2.0);
    let mut p = turn_arc(x + wd - r, y + r, r, 0.0, 0.5);
    p.extend(turn_arc(x + r, y + r, r, 0.5, 1.0));
    p
}

/// Paint the halo for a square of `s`: a soft glow, a fine ring of ticks, two orbit
/// arcs of light and a dark glass disc. Returns the radius inside the tick ring.
pub(crate) fn halo(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> f32 {
    let c = s / 2.0;
    let r = c - s * 0.02;
    // Soft glow: layered discs, faint at the rim.
    for k in 0..8 {
        let rr = r - s * 0.012 * k as f32;
        fill(w, o, &circle(c, c, rr), scale_alpha(ink.primary, 0.012));
    }
    fill(w, o, &circle(c, c, r - s * 0.07), ink.face);
    fill(
        w,
        o,
        &circle(c, c, r - s * 0.07),
        scale_alpha(ink.primary, 0.04),
    );
    // The orb: brighter towards the middle.
    for k in 0..6 {
        let rr = (r - s * 0.07) * (1.0 - k as f32 * 0.15);
        fill(w, o, &circle(c, c, rr), scale_alpha(ink.info, 0.012));
    }
    stroke(
        w,
        o,
        &circle(c, c, r - s * 0.07),
        s * 0.003,
        scale_alpha(ink.primary, 0.35),
        true,
    );
    for i in 0..120 {
        let a = i as f32 / 120.0;
        let len = if i % 10 == 0 { s * 0.02 } else { s * 0.01 };
        stroke(
            w,
            o,
            &[
                polar(c, c, r - s * 0.035 - len, a),
                polar(c, c, r - s * 0.035, a),
            ],
            s * 0.003,
            scale_alpha(ink.text, if i % 10 == 0 { 0.4 } else { 0.18 }),
            false,
        );
    }
    // Orbit arcs: two sweeps of light on the rim.
    glow_arc(
        w,
        o,
        (c, c),
        r,
        (0.86, 1.04),
        s * 0.006,
        (ink.primary, ink.info),
    );
    glow_arc(
        w,
        o,
        (c, c),
        r,
        (0.36, 0.48),
        s * 0.006,
        (ink.info, ink.primary),
    );
    r - s * 0.075
}

/// Paint an arc of light from `a0` to `a1` turns, its colour running `from` → `to`, with a wide
/// soft glow under it.
#[allow(clippy::too_many_arguments)]
pub(crate) fn glow_arc(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    (a0, a1): (f32, f32),
    width: f32,
    (from, to): (Hsla, Hsla),
) {
    if a1 <= a0 {
        return;
    }
    let n = ((a1 - a0) * 120.0).ceil().max(2.0) as usize;
    for i in 0..n {
        let u0 = a0 + (a1 - a0) * i as f32 / n as f32;
        let u1 = a0 + (a1 - a0) * (i + 1) as f32 / n as f32;
        let col = mix(from, to, (i as f32 + 0.5) / n as f32);
        let arc = turn_arc(cx, cy, r, u0, u1 + 0.001);
        stroke(w, o, &arc, width * 4.0, scale_alpha(col, 0.08), false);
        stroke(w, o, &arc, width * 2.2, scale_alpha(col, 0.18), false);
        stroke(w, o, &arc, width, col, false);
    }
}

/// Paint a glowing dot: layered discs fading out to `r * 3`.
pub(crate) fn glow_dot(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y): (f32, f32),
    r: f32,
    col: Hsla,
    core: Hsla,
) {
    for k in (1..=5).rev() {
        fill(
            w,
            o,
            &circle(x, y, r * (1.0 + k as f32 * 0.4)),
            scale_alpha(col, 0.07),
        );
    }
    fill(w, o, &circle(x, y, r), col);
    fill(w, o, &circle(x, y, r * 0.45), core);
}

/// Paint a blade of light from radius `r0` to `r1` at `turn`, glowing, with a dot at the tip.
#[allow(clippy::too_many_arguments)]
pub(crate) fn light_blade(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    (r0, r1): (f32, f32),
    turn: f32,
    width: f32,
    col: Hsla,
    core: Hsla,
) {
    let line = [polar(cx, cy, r0, turn), polar(cx, cy, r1, turn)];
    stroke(w, o, &line, width * 5.0, scale_alpha(col, 0.08), false);
    stroke(w, o, &line, width * 2.5, scale_alpha(col, 0.2), false);
    stroke(w, o, &line, width, col, false);
    glow_dot(w, o, line[1], width * 1.4, col, core);
}

/// Paint a fading wake behind `turn`: `len` turns of wedges between `r0` and `r1`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wake(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    (r0, r1): (f32, f32),
    turn: f32,
    len: f32,
    col: Hsla,
) {
    let steps = 14;
    for k in 0..steps {
        let t1 = turn - len * k as f32 / steps as f32;
        let t0 = turn - len * (k + 1) as f32 / steps as f32;
        fill(
            w,
            o,
            &band(cx, cy, r0, r1, t0, t1),
            scale_alpha(col, 0.22 * (1.0 - k as f32 / steps as f32)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_text_keeps_clear_of_the_rivets() {
        let h = 20.0;
        // A rivet sits at 0.35 h with a radius of 0.09 h.
        assert!(plate_inset(h) > h * (0.35 + 0.09));
        assert!(plate_inset(h) < h);
    }

    #[test]
    fn a_spade_hand_reaches_its_length_and_has_a_tail() {
        let h = spade(50.0, 50.0, 0.25, 40.0);
        let tip = h.tip[1];
        assert!((tip.0 - 90.0).abs() < 1e-3 && (tip.1 - 50.0).abs() < 1e-3);
        assert!(h.shaft[0].0 < 50.0, "the tail sits behind the pivot");
        assert!(h.ring.0 > 50.0 && h.ring.0 < 90.0);
    }

    #[test]
    fn segment_text_widens_with_digits_more_than_dots() {
        assert!(seg_width("888", 10.0) > seg_width("88", 10.0));
        assert!(seg_width("8.8", 10.0) < seg_width("888", 10.0));
        assert!(seg_width("8:8", 10.0) < seg_width("888", 10.0));
    }

    #[test]
    fn a_pill_stays_in_its_box_and_reaches_its_ends() {
        let p = pill(10.0, 20.0, 100.0, 30.0);
        for (x, y) in &p {
            assert!(*x >= 10.0 - 1e-3 && *x <= 110.0 + 1e-3);
            assert!(*y >= 20.0 - 1e-3 && *y <= 50.0 + 1e-3);
        }
        let max_x = p.iter().map(|q| q.0).fold(f32::MIN, f32::max);
        let min_x = p.iter().map(|q| q.0).fold(f32::MAX, f32::min);
        assert!((max_x - 110.0).abs() < 0.05 && (min_x - 10.0).abs() < 0.05);
    }

    #[test]
    fn segments_light_from_the_start() {
        let red = gpui::red();
        let segs = lit_segments(10, 0.35, |_| red);
        assert_eq!(segs.iter().filter(|s| s.is_some()).count(), 4);
        assert!(segs[0].is_some() && segs[9].is_none());
        assert!(lit_segments(5, 0.0, |_| red).iter().all(Option::is_none));
        assert!(lit_segments(5, 1.0, |_| red).iter().all(Option::is_some));
    }
}
