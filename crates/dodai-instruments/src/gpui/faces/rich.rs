//! The chronograph and heads-up gauge faces: a chronograph dial with a knurled bezel, hatched
//! zones, a tapered needle and a sub-dial; a heads-up ring of dashes, brackets and a framed
//! readout; and the heads-up tape scrolling under a pointer box. Every colour comes from the
//! theme.

use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::dial::severity_color;
use crate::gpui::draw::{polar, turn_arc};
use crate::gpui::gauges::SWEEP;
use crate::gpui::pod_math as pm;
use crate::gpui::shapes::{band, chamfered, chrono_case, chrono_glass, dir, hud_ring, Hand, Ink};
use crate::instrument::readout;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
use dodai_gpui::theme_of;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    Styled,
};

/// Zone spans with their theme colours.
pub(super) fn zone_colours(face: &GaugeFace, t: &dodai_core::Theme) -> Vec<(f32, f32, Hsla)> {
    face.spans()
        .into_iter()
        .map(|(a, z, sev)| (a, z, severity_color(t, sev)))
        .collect()
}

/// `(fraction, label)` of every major tick.
pub(super) fn major_labels(face: &GaugeFace) -> Vec<(f32, String)> {
    face.scale
        .ticks()
        .into_iter()
        .filter(|t| t.major)
        .map(|t| (t.frac, readout(t.value, 0, "")))
        .collect()
}

/// Fine marks per major interval for the detailed looks.
pub(super) const FINE: usize = 5;

impl GaugeFace {
    /// What the readout is captioned with: the label, else the unit.
    pub(super) fn caption(&self) -> gpui::SharedString {
        if self.label.is_empty() {
            self.unit.clone()
        } else {
            self.label.clone()
        }
    }

    /// Numerals placed on a circle of radius `r` for every major tick of a sweep.
    pub(super) fn numerals(
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
        let r_face = c - s * 0.06;
        let r_scale = r_face - s * 0.03;
        let (text, muted, line) = (ink.text, ink.muted, ink.line);
        let sub = (c, c - s * 0.17, s * 0.075);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                chrono_case(w, o, s, ink);
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
                fill_circle(w, o, (sx, sy), sr, scale_alpha(text, 0.05));
                stroke_circle(w, o, (sx, sy), sr, s * 0.005, line);
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
                Hand {
                    centre: (sx, sy),
                    turn: frac,
                    reach: (sr * 0.2, sr * 0.85),
                    half: s * 0.008,
                    colours: (col, text),
                    shadow: None,
                }
                .paint(w, o);
                fill_circle(w, o, (sx, sy), s * 0.012, text);
                // Needle: a shadow, the tapered hand, a counterweight and the hub.
                let a = start + span * frac;
                Hand {
                    centre: (c, c),
                    turn: a,
                    reach: (s * 0.12, r_scale - s * 0.01),
                    half: s * 0.02,
                    colours: (col, text),
                    shadow: Some(scale_alpha(text, 0.12)),
                }
                .paint(w, o);
                let (dx, dy) = dir(a);
                fill_circle(w, o, (c - dx * s * 0.1, c - dy * s * 0.1), s * 0.022, col);
                crate::gpui::draw::hub(w, o, (c, c), s * 0.038, (ink.text, ink.face, col));
                chrono_glass(w, o, s, r_face, ink);
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
                            .text_size(px(s * 0.07))
                            .text_color(col)
                            .font_weight(FontWeight::LIGHT)
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
        let ink = self.lit_ink(&t);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let accent = ink.primary;
        let zones = zone_colours(&self, &t);
        let r = c - s * 0.11;
        let r_in = r - s * 0.1;
        let (text, muted) = (ink.text, ink.muted);
        let (fw, fh) = (s * 0.46, s * 0.2);
        let majors = self.scale.majors.max(1);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                hud_ring(w, o, s, ink, accent);
                // Track and the dashed progress.
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    s * 0.003,
                    scale_alpha(text, 0.3),
                    false,
                );
                let n = 90;
                for i in 0..pm::segments_lit(frac, n) {
                    let a0 = i as f32 / n as f32;
                    let a1 = a0 + 0.55 / n as f32;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, start + span * a0, start + span * a1),
                        s * 0.028,
                        col,
                        false,
                    );
                }
                for (a, z, zc) in &zones {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r + s * 0.026, start + span * a, start + span * z),
                        s * 0.005,
                        *zc,
                        false,
                    );
                }
                // End caps of the scale.
                for e in [start, start + span] {
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r - s * 0.03, e), polar(c, c, r + s * 0.03, e)],
                        s * 0.004,
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
                stroke_circle(w, o, (c, c), r_in, s * 0.004, scale_alpha(text, 0.2));
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
                fill(w, o, &frame, scale_alpha(accent, 0.06));
                stroke(w, o, &frame, s * 0.003, scale_alpha(accent, 0.7), true);
                let inner = chamfered(
                    c - fw / 2.0 + s * 0.012,
                    c - fh / 2.0 + s * 0.012,
                    fw - s * 0.024,
                    fh - s * 0.024,
                    s * 0.022,
                );
                stroke(w, o, &inner, s * 0.0015, scale_alpha(accent, 0.3), true);
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
                    .text_size(px(s * 0.03))
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
                            .text_color(figure_col)
                            .font_weight(FontWeight::LIGHT)
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
    pub(super) fn tape_face(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
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
}
