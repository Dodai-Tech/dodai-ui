//! The detailed gauge looks: a chronograph dial with a knurled bezel, hatched zones, a tapered
//! needle and a sub-dial; a heads-up ring of dashes, brackets and a framed readout; a scrolling
//! tape with a pointer box; a sweep dial with a fading trail; and a moving-coil meter with its
//! pivot below the face. Every colour comes from the theme.

use super::{GaugeFace, Ink};
use crate::dash::text_box;
use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::gauges::SWEEP;
use crate::pod_math as pm;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use dodai_instruments_core::instrument::{readout, Severity};
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    Styled,
};

/// Unit vector of a turn (0 = up, clockwise), as `polar` measures it.
fn dir(turn: f32) -> (f32, f32) {
    let th = turn * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (th.cos(), th.sin())
}

/// A closed circle polygon.
fn circle(cx: f32, cy: f32, r: f32) -> Vec<(f32, f32)> {
    turn_arc(cx, cy, r, 0.0, 1.0)
}

/// A band between two radii over `a..b` turns, as a fillable polygon.
fn band(cx: f32, cy: f32, r_in: f32, r_out: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let mut p = turn_arc(cx, cy, r_out, a, b);
    let mut inner = turn_arc(cx, cy, r_in, a, b);
    inner.reverse();
    p.extend(inner);
    p
}

/// A tapered needle from `tail` behind the pivot to `tip`, `half` wide at the pivot.
fn needle(cx: f32, cy: f32, turn: f32, tip: f32, tail: f32, half: f32) -> Vec<(f32, f32)> {
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
fn chamfered(x: f32, y: f32, w: f32, h: f32, k: f32) -> Vec<(f32, f32)> {
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

/// Zone spans with their theme colours.
fn zone_colours(face: &GaugeFace, t: &dodai_core::Theme) -> Vec<(f32, f32, Hsla)> {
    face.spans()
        .into_iter()
        .map(|(a, z, sev)| (a, z, severity_color(t, sev)))
        .collect()
}

/// `(fraction, label)` of every major tick.
fn major_labels(face: &GaugeFace) -> Vec<(f32, String)> {
    face.scale
        .ticks()
        .into_iter()
        .filter(|t| t.major)
        .map(|t| (t.frac, readout(t.value, 0, "")))
        .collect()
}

/// Fine marks per major interval for the detailed looks.
const FINE: usize = 5;

impl GaugeFace {
    /// What the readout is captioned with: the label, else the unit.
    fn caption(&self) -> gpui::SharedString {
        if self.label.is_empty() {
            self.unit.clone()
        } else {
            self.label.clone()
        }
    }

    /// Numerals placed on a circle of radius `r` for every major tick of a sweep.
    fn numerals(
        &self,
        c: (f32, f32),
        r: f32,
        sweep: (f32, f32),
        text: f32,
        col: Hsla,
    ) -> Vec<AnyElement> {
        major_labels(self)
            .into_iter()
            .map(|(f, s)| {
                let (x, y) = polar(c.0, c.1, r, sweep.0 + sweep.1 * f);
                text_box(x, y - text * 0.65, text * 4.0)
                    .text_size(px(text))
                    .text_color(col)
                    .child(s)
                    .into_any_element()
            })
            .collect()
    }

    /// A chronograph: knurled bezel, fine scale with hatched zones, numerals, a sub-dial for the
    /// share of the scale, a readout window and a tapered needle with a counterweight.
    pub(super) fn chrono(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors * FINE;
        let (r_bezel, r_face) = (c - s * 0.01, c - s * 0.06);
        let r_scale = r_face - s * 0.03;
        let (text, muted, face, line) = (ink.text, ink.muted, ink.face, ink.line);
        let bg = scale_alpha(ink.text, 0.04);
        let sub = (c, c - s * 0.17, s * 0.075);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                // Bezel: a ring with fine knurling.
                fill(w, o, &circle(c, c, r_bezel), scale_alpha(text, 0.08));
                for i in 0..120 {
                    let a = i as f32 / 120.0;
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r_face + s * 0.008, a), polar(c, c, r_bezel, a)],
                        s * 0.004,
                        scale_alpha(text, 0.22),
                        false,
                    );
                }
                fill(w, o, &circle(c, c, r_face), face);
                fill(w, o, &circle(c, c, r_face), bg);
                stroke(w, o, &circle(c, c, r_face), s * 0.006, line, true);
                // Zones: an outlined band with diagonal hatching.
                for (a, z, zc) in &zones {
                    let (r0, r1) = (r_scale - s * 0.05, r_scale - s * 0.015);
                    let (ta, tz) = (start + span * a, start + span * z);
                    fill(w, o, &band(c, c, r0, r1, ta, tz), scale_alpha(*zc, 0.18));
                    let n = ((tz - ta) * 260.0).ceil().max(2.0) as usize;
                    for k in 0..n {
                        let u = ta + (tz - ta) * k as f32 / n as f32;
                        let v = (u + 0.008).min(tz);
                        stroke(
                            w,
                            o,
                            &[polar(c, c, r0, u), polar(c, c, r1, v)],
                            s * 0.006,
                            *zc,
                            false,
                        );
                    }
                    stroke(w, o, &turn_arc(c, c, r1, ta, tz), s * 0.006, *zc, false);
                }
                // Scale: majors, half marks and fine marks.
                for i in 0..=marks {
                    let f = i as f32 / marks as f32;
                    let a = start + span * f;
                    let (len, wd, cl) = if i % FINE == 0 {
                        (s * 0.06, s * 0.012, text)
                    } else if FINE % 2 == 0 && i % (FINE / 2) == 0 {
                        (s * 0.04, s * 0.007, text)
                    } else {
                        (s * 0.025, s * 0.005, muted)
                    };
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r_scale - len, a), polar(c, c, r_scale, a)],
                        wd,
                        cl,
                        false,
                    );
                }
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_scale + s * 0.004, start, start + span),
                    s * 0.004,
                    muted,
                    false,
                );
                // Sub-dial: the share of the scale, with its own small hand.
                let (sx, sy, sr) = sub;
                fill(w, o, &circle(sx, sy, sr), scale_alpha(text, 0.05));
                stroke(w, o, &circle(sx, sy, sr), s * 0.005, line, true);
                for i in 0..20 {
                    let a = i as f32 / 20.0;
                    let r0 = if i % 5 == 0 { sr * 0.7 } else { sr * 0.84 };
                    stroke(
                        w,
                        o,
                        &[polar(sx, sy, r0, a), polar(sx, sy, sr * 0.95, a)],
                        s * 0.004,
                        muted,
                        false,
                    );
                }
                fill(
                    w,
                    o,
                    &needle(sx, sy, frac, sr * 0.85, sr * 0.2, s * 0.008),
                    col,
                );
                fill(w, o, &circle(sx, sy, s * 0.012), text);
                // Needle: a shadow, the tapered hand, a counterweight and the hub.
                let a = start + span * frac;
                fill(
                    w,
                    o,
                    &needle(
                        c + s * 0.006,
                        c + s * 0.008,
                        a,
                        r_scale - s * 0.01,
                        s * 0.12,
                        s * 0.02,
                    ),
                    scale_alpha(text, 0.12),
                );
                fill(
                    w,
                    o,
                    &needle(c, c, a, r_scale - s * 0.01, s * 0.12, s * 0.02),
                    col,
                );
                let (dx, dy) = dir(a);
                fill(
                    w,
                    o,
                    &circle(c - dx * s * 0.1, c - dy * s * 0.1, s * 0.022),
                    col,
                );
                fill(w, o, &circle(c, c, s * 0.04), text);
                fill(w, o, &circle(c, c, s * 0.016), face);
                // Glass: a soft crescent of light over the top half.
                let mut glass = turn_arc(c, c, r_face * 0.97, 0.8, 1.2);
                glass.extend(turn_arc(c, c + r_face * 0.45, r_face * 0.9, 1.15, 0.85));
                fill(w, o, &glass, scale_alpha(text, 0.035));
            },
        )
        .absolute()
        .size_full();
        let num = s * 0.055;
        let window_w = s * 0.34;
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(cv)
            .children(self.numerals((c, c), r_scale - s * 0.11, SWEEP, num, ink.text))
            .child(
                text_box(c, c + s * 0.12, s * 0.5)
                    .text_size(px(s * 0.04))
                    .text_color(ink.muted)
                    .child(self.caption()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - window_w / 2.0))
                    .top(px(c + s * 0.19))
                    .w(px(window_w))
                    .py(px(s * 0.008))
                    .flex()
                    .justify_center()
                    .items_baseline()
                    .gap(px(s * 0.012))
                    .rounded(px(s * 0.015))
                    .border_1()
                    .border_color(ink.line)
                    .bg(scale_alpha(ink.text, 0.06))
                    .child(
                        div()
                            .text_size(px(s * 0.075))
                            .text_color(col)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.035))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A heads-up ring: dashed outer ring with brackets, a dashed progress arc, an outside
    /// pointer, an inner scale ring and the figure in a chamfered frame.
    pub(super) fn hud(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let accent = ink.primary;
        let zones = zone_colours(&self, &t);
        let (r_out, r) = (c - s * 0.015, c - s * 0.11);
        let r_in = r - s * 0.1;
        let (text, muted) = (ink.text, ink.muted);
        let (fw, fh) = (s * 0.46, s * 0.2);
        let majors = self.scale.majors.max(1);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                // Outer ring: fine dashes and four brackets on the diagonals.
                for i in 0..96 {
                    let a = i as f32 / 96.0;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r_out, a, a + 0.005),
                        s * 0.006,
                        scale_alpha(text, 0.35),
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
                // Track and the dashed progress.
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    s * 0.006,
                    scale_alpha(text, 0.25),
                    false,
                );
                let n = 90;
                for i in 0..n {
                    let a0 = i as f32 / n as f32;
                    if a0 >= frac {
                        break;
                    }
                    let a1 = (a0 + 0.65 / n as f32).min(frac);
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, start + span * a0, start + span * a1),
                        s * 0.04,
                        col,
                        false,
                    );
                }
                for (a, z, zc) in &zones {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r + s * 0.032, start + span * a, start + span * z),
                        s * 0.008,
                        *zc,
                        false,
                    );
                }
                // End caps of the scale.
                for e in [start, start + span] {
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r - s * 0.035, e), polar(c, c, r + s * 0.035, e)],
                        s * 0.008,
                        text,
                        false,
                    );
                }
                // Pointer outside the arc and a hairline to the inner ring.
                let a = start + span * frac;
                let tip = polar(c, c, r + s * 0.03, a);
                let (b1, b2) = (
                    polar(c, c, r + s * 0.07, a - 0.012),
                    polar(c, c, r + s * 0.07, a + 0.012),
                );
                fill(w, o, &[tip, b1, b2], text);
                stroke(
                    w,
                    o,
                    &[polar(c, c, r_in, a), polar(c, c, r - s * 0.025, a)],
                    s * 0.004,
                    text,
                    false,
                );
                // Inner ring with major marks as small diamonds.
                stroke(
                    w,
                    o,
                    &circle(c, c, r_in),
                    s * 0.004,
                    scale_alpha(text, 0.2),
                    true,
                );
                for i in 0..=majors {
                    let f = i as f32 / majors as f32;
                    let ang = start + span * f;
                    let (x, y) = polar(c, c, r_in, ang);
                    let k = s * 0.009;
                    fill(
                        w,
                        o,
                        &[(x, y - k), (x + k, y), (x, y + k), (x - k, y)],
                        if f <= frac { accent } else { muted },
                    );
                }
                // The readout frame and crosshair stubs.
                let frame = chamfered(c - fw / 2.0, c - fh / 2.0, fw, fh, s * 0.03);
                fill(w, o, &frame, scale_alpha(accent, 0.1));
                stroke(w, o, &frame, s * 0.006, scale_alpha(accent, 0.8), true);
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(cv)
            .children(self.numerals((c, c), r - s * 0.048, SWEEP, s * 0.036, ink.muted))
            .child(
                text_box(c, c - fh / 2.0 - s * 0.075, s * 0.5)
                    .text_size(px(s * 0.035))
                    .text_color(ink.primary)
                    .child(self.caption().to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - fw / 2.0))
                    .top(px(c - fh / 2.0))
                    .w(px(fw))
                    .h(px(fh))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(s * 0.13))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    ),
            )
            .child(
                text_box(c, c + fh / 2.0 + s * 0.03, s * 0.5)
                    .text_size(px(s * 0.035))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            )
            .into_any_element()
    }

    /// A vertical tape that scrolls under a fixed pointer box, with a zone strip, and a small
    /// overview column of the whole scale beside it.
    pub(super) fn tape(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let (min, max) = (self.scale.min, self.scale.max);
        let frac = self.frac();
        let value = self.scale.value_at(frac);
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let (tw, th) = (s * 0.5, s * 0.9);
        let mid = th / 2.0;
        // The tape shows 40 % of the scale.
        let per = th / ((max - min) * 0.4);
        let y_of = move |v: f32| mid - (v - value) * per;
        let marks: Vec<(f32, bool, String)> = {
            let total = self.scale.majors * FINE;
            (0..=total)
                .map(|i| {
                    let v = min + (max - min) * i as f32 / total as f32;
                    (v, i % FINE == 0, readout(v, 0, ""))
                })
                .collect()
        };
        let (text, muted, face) = (ink.text, ink.muted, ink.face);
        let zone_px: Vec<(f32, f32, Hsla)> = zones
            .iter()
            .map(|(a, z, zc)| {
                (
                    y_of(min + (max - min) * z),
                    y_of(min + (max - min) * a),
                    *zc,
                )
            })
            .collect();
        let tick_marks: Vec<(f32, bool)> = marks.iter().map(|(v, m, _)| (y_of(*v), *m)).collect();
        let tape_cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                for (y0, y1, zc) in &zone_px {
                    fill(
                        w,
                        o,
                        &[
                            (tw - s * 0.03, *y0),
                            (tw, *y0),
                            (tw, *y1),
                            (tw - s * 0.03, *y1),
                        ],
                        *zc,
                    );
                }
                for (y, major) in &tick_marks {
                    let len = if *major { s * 0.08 } else { s * 0.04 };
                    stroke(
                        w,
                        o,
                        &[(tw - s * 0.035 - len, *y), (tw - s * 0.035, *y)],
                        if *major { s * 0.008 } else { s * 0.005 },
                        if *major { text } else { muted },
                        false,
                    );
                }
                // Fade both ends of the tape into the face.
                let steps = 10;
                let h = th * 0.022;
                for k in 0..steps {
                    let a = 0.9 * (1.0 - k as f32 / steps as f32);
                    let (y0, y1) = (k as f32 * h, th - (k as f32 + 1.0) * h);
                    for y in [y0, y1] {
                        fill(
                            w,
                            o,
                            &[(0.0, y), (tw, y), (tw, y + h), (0.0, y + h)],
                            scale_alpha(face, a),
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let labels: Vec<AnyElement> = marks
            .into_iter()
            .filter(|(_, m, _)| *m)
            .map(|(v, _, txt)| {
                div()
                    .absolute()
                    .left(px(s * 0.02))
                    .top(px(y_of(v) - s * 0.035))
                    .w(px(tw * 0.5))
                    .text_size(px(s * 0.05))
                    .text_color(ink.muted)
                    .child(txt)
                    .into_any_element()
            })
            .collect();
        let (bw, bh) = (tw + s * 0.06, s * 0.15);
        let pointer_cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let k = bh * 0.5;
                let p = vec![(0.0, 0.0), (bw - k, 0.0), (bw, k), (bw - k, bh), (0.0, bh)];
                fill(w, o, &p, ink.face);
                stroke(w, o, &p, s * 0.008, col, true);
            },
        )
        .absolute()
        .size_full();
        let tape = div()
            .relative()
            .w(px(tw))
            .h(px(th))
            .overflow_hidden()
            .rounded(px(s * 0.02))
            .border_1()
            .border_color(ink.line)
            .bg(scale_alpha(ink.text, 0.04))
            .child(tape_cv)
            .children(labels);
        let pointer = div()
            .absolute()
            .left(px(-s * 0.005))
            .top(px(mid - bh / 2.0))
            .w(px(bw))
            .h(px(bh))
            .child(pointer_cv)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .w(px(bw - bh * 0.4))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(s * 0.09))
                    .text_color(col)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.figure()),
            );
        // Overview column: the whole scale, its zones and a marker.
        let oh = th * 0.62;
        let ov_zones: Vec<AnyElement> = zones
            .iter()
            .map(|(a, z, zc)| {
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .bottom(px(oh * a))
                    .h(px(oh * (z - a)))
                    .bg(*zc)
                    .into_any_element()
            })
            .collect();
        let overview = div()
            .relative()
            .w(px(s * 0.035))
            .h(px(oh))
            .rounded(px(s * 0.01))
            .bg(scale_alpha(ink.text, 0.12))
            .children(ov_zones)
            .child(
                div()
                    .absolute()
                    .left(px(-s * 0.02))
                    .bottom(px(oh * frac - s * 0.006))
                    .w(px(s * 0.075))
                    .h(px(s * 0.012))
                    .rounded(px(s * 0.006))
                    .bg(ink.text),
            );
        let side = div()
            .flex()
            .flex_col()
            .justify_between()
            .h(px(th))
            .py(px(s * 0.02))
            .child(self.heading(&ink, s))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(s * 0.04))
                    .child(overview)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .justify_between()
                            .h(px(oh))
                            .text_size(px(s * 0.045))
                            .text_color(ink.muted)
                            .child(readout(max, 0, ""))
                            .child(format!("{}%", (frac * 100.0).round() as i32))
                            .child(readout(min, 0, "")),
                    ),
            )
            .child(
                div()
                    .text_size(px(s * 0.055))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            );
        div()
            .flex()
            .gap(px(s * 0.06))
            .w(px(s))
            .child(div().relative().child(tape).child(pointer))
            .child(side)
            .into_any_element()
    }

    /// A sweep dial: a grid of rings and spokes, a graded scale, a fading trail behind a thin
    /// needle, numerals outside and the figure in the gap.
    pub(super) fn sweep(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let spans = self.spans();
        let r = c - s * 0.12;
        let n = 72;
        let grade: Vec<Hsla> = (0..n)
            .map(|i| {
                let f = (i as f32 + 0.5) / n as f32;
                match pm::severity_at(&spans, f) {
                    Severity::Normal => scale_alpha(ink.primary, 0.2 + 0.8 * f),
                    sev => severity_color(&t, sev),
                }
            })
            .collect();
        let (text, accent) = (ink.text, ink.primary);
        let majors = self.scale.majors;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                for k in 1..=3 {
                    stroke(
                        w,
                        o,
                        &circle(c, c, r * k as f32 / 3.5),
                        s * 0.003,
                        scale_alpha(text, 0.1),
                        true,
                    );
                }
                for i in 0..=majors {
                    let a = start + span * i as f32 / majors as f32;
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r * 0.15, a), polar(c, c, r - s * 0.03, a)],
                        s * 0.003,
                        scale_alpha(text, 0.12),
                        false,
                    );
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r + s * 0.015, a), polar(c, c, r + s * 0.045, a)],
                        s * 0.008,
                        text,
                        false,
                    );
                }
                for (i, g) in grade.iter().enumerate() {
                    let a0 = start + span * i as f32 / n as f32;
                    let a1 = start + span * (i as f32 + 0.8) / n as f32;
                    stroke(w, o, &turn_arc(c, c, r, a0, a1), s * 0.03, *g, false);
                }
                // The trail: wedges fading out behind the needle.
                let steps = 16;
                let len = 0.18_f32.min(frac);
                for k in 0..steps {
                    let f1 = frac - len * k as f32 / steps as f32;
                    let f0 = frac - len * (k + 1) as f32 / steps as f32;
                    let mut wedge = vec![(c, c)];
                    wedge.extend(turn_arc(
                        c,
                        c,
                        r - s * 0.02,
                        start + span * f0,
                        start + span * f1,
                    ));
                    fill(
                        w,
                        o,
                        &wedge,
                        scale_alpha(accent, 0.28 * (1.0 - k as f32 / steps as f32)),
                    );
                }
                let a = start + span * frac;
                stroke(
                    w,
                    o,
                    &[(c, c), polar(c, c, r + s * 0.03, a)],
                    s * 0.03,
                    scale_alpha(col, 0.25),
                    false,
                );
                stroke(
                    w,
                    o,
                    &[(c, c), polar(c, c, r + s * 0.03, a)],
                    s * 0.01,
                    text,
                    false,
                );
                fill(w, o, &circle(c, c, s * 0.03), text);
                fill(w, o, &circle(c, c, s * 0.014), col);
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(cv)
            .children(self.numerals((c, c), r + s * 0.085, SWEEP, s * 0.045, ink.text))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(c + s * 0.2))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(s * 0.12))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.04))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A moving-coil meter: a wide face, a flat scale arc with its zones as a thick band, a long
    /// needle from a pivot below the face, screws in the corners and a name plate.
    pub(super) fn meter(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let (w_all, h_all) = (s, s * 0.74);
        let plate = h_all * 0.22;
        let pivot = (w_all / 2.0, h_all * 1.02);
        let r = h_all * 0.84;
        let (start, span) = (0.9_f32, 0.2_f32);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors * FINE;
        let (text, muted, line, face) = (ink.text, ink.muted, ink.line, ink.face);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let (px_, py_) = pivot;
                stroke(
                    w,
                    o,
                    &turn_arc(px_, py_, r, start, start + span),
                    s * 0.006,
                    text,
                    false,
                );
                for (a, z, zc) in &zones {
                    fill(
                        w,
                        o,
                        &band(
                            px_,
                            py_,
                            r - s * 0.035,
                            r - s * 0.008,
                            start + span * a,
                            start + span * z,
                        ),
                        *zc,
                    );
                }
                for i in 0..=marks {
                    let f = i as f32 / marks as f32;
                    let a = start + span * f;
                    let major = i % FINE == 0;
                    let len = if major { s * 0.06 } else { s * 0.03 };
                    stroke(
                        w,
                        o,
                        &[polar(px_, py_, r, a), polar(px_, py_, r + len, a)],
                        if major { s * 0.008 } else { s * 0.004 },
                        if major { text } else { muted },
                        false,
                    );
                }
                // Mirror strip under the scale, as on a precision meter.
                stroke(
                    w,
                    o,
                    &turn_arc(px_, py_, r - s * 0.06, start, start + span),
                    s * 0.012,
                    scale_alpha(text, 0.1),
                    false,
                );
                let a = start + span * frac;
                stroke(
                    w,
                    o,
                    &[
                        polar(px_, py_, h_all * 0.25, a),
                        polar(px_, py_, r + s * 0.07, a),
                    ],
                    s * 0.008,
                    col,
                    false,
                );
                // Name plate over the pivot.
                let y0 = h_all - plate;
                let plate_box = [(0.0, y0), (w_all, y0), (w_all, h_all), (0.0, h_all)];
                fill(w, o, &plate_box, face);
                fill(w, o, &plate_box, scale_alpha(text, 0.07));
                stroke(w, o, &[(0.0, y0), (w_all, y0)], s * 0.004, line, false);
                // Screws: a ring and a slot.
                for (x, y) in [
                    (s * 0.05, s * 0.05),
                    (w_all - s * 0.05, s * 0.05),
                    (s * 0.05, h_all - s * 0.05),
                    (w_all - s * 0.05, h_all - s * 0.05),
                ] {
                    let k = s * 0.018;
                    fill(w, o, &circle(x, y, k), scale_alpha(text, 0.15));
                    stroke(w, o, &circle(x, y, k), s * 0.003, line, true);
                    stroke(
                        w,
                        o,
                        &[(x - k * 0.7, y - k * 0.3), (x + k * 0.7, y + k * 0.3)],
                        s * 0.004,
                        muted,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let labels: Vec<AnyElement> = major_labels(&self)
            .into_iter()
            .map(|(f, txt)| {
                let (x, y) = polar(pivot.0, pivot.1, r + s * 0.1, start + span * f);
                text_box(x, y - s * 0.03, s * 0.2)
                    .text_size(px(s * 0.045))
                    .text_color(ink.text)
                    .child(txt)
                    .into_any_element()
            })
            .collect();
        div()
            .relative()
            .w(px(w_all))
            .h(px(h_all))
            .overflow_hidden()
            .rounded(px(s * 0.04))
            .border_1()
            .border_color(ink.line)
            .bg(ink.face)
            .child(cv)
            .children(labels)
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.1))
                    .top(px(h_all - plate))
                    .w(px(w_all - s * 0.2))
                    .h(px(plate))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(s * 0.04))
                            .text_color(ink.muted)
                            .child(self.caption()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(s * 0.012))
                            .child(
                                div()
                                    .text_size(px(s * 0.07))
                                    .text_color(col)
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(self.figure()),
                            )
                            .child(
                                div()
                                    .text_size(px(s * 0.035))
                                    .text_color(ink.muted)
                                    .child(self.unit.clone()),
                            ),
                    ),
            )
            .into_any_element()
    }
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
    fn chamfered_corners_are_cut() {
        let p = chamfered(0.0, 0.0, 10.0, 6.0, 2.0);
        assert_eq!(p.len(), 8);
        assert!(!p.contains(&(0.0, 0.0)), "no square corner left");
    }
}
