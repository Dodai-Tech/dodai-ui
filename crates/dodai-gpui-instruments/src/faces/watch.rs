//! The chronograph's extra layouts: the bezel, a watch dial with the scale engraved on a dark
//! bezel round it, and the register, a wide case with two registers (the scale and its share)
//! over a readout window. Every colour comes from the theme.

use super::layouts::layer;
use super::rich::{major_labels, zone_colours, FINE};
use super::GaugeFace;
use crate::dash::text_box;
use crate::draw::polar;
use crate::gauges::SWEEP;
use crate::shapes::{band, chrono_glass, circle, dir, needle, rounded, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// Height of the register case, as a share of its width.
pub(crate) const REGISTER_H: f32 = 0.7;

/// The bezel's scale: almost a full turn from just past twelve o'clock, the gap at the top.
const AROUND: (f32, f32) = (0.04, 0.92);

/// A register dial at `(x, y)` of radius `r`: a sunk disc, `marks` fine marks over `sweep` (every
/// `every`-th long), the zones as a band inside the marks and a tapered hand at `frac`.
#[allow(clippy::too_many_arguments)]
fn register(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    (x, y, r): (f32, f32, f32),
    (start, span): (f32, f32),
    (marks, every): (usize, usize),
    zones: &[(f32, f32, Hsla)],
    frac: f32,
    col: Hsla,
    ink: Ink,
) {
    fill(w, o, &circle(x, y, r), scale_alpha(ink.text, 0.05));
    stroke(w, o, &circle(x, y, r), r * 0.025, ink.line, true);
    // Fine concentric grooves, like the snailed finish of a watch register.
    for k in 1..6 {
        stroke(
            w,
            o,
            &circle(x, y, r * (0.3 + 0.1 * k as f32)),
            r * 0.006,
            scale_alpha(ink.text, 0.04),
            true,
        );
    }
    for (a, z, zc) in zones {
        fill(
            w,
            o,
            &band(x, y, r * 0.8, r * 0.86, start + span * a, start + span * z),
            scale_alpha(*zc, 0.85),
        );
    }
    let marks = marks.max(1);
    for i in 0..=marks {
        let a = start + span * i as f32 / marks as f32;
        let (r0, wd, c) = if i % every.max(1) == 0 {
            (r * 0.84, r * 0.03, ink.text)
        } else {
            (r * 0.9, r * 0.018, ink.muted)
        };
        stroke(
            w,
            o,
            &[polar(x, y, r0, a), polar(x, y, r * 0.96, a)],
            wd,
            c,
            false,
        );
    }
    let a = start + span * frac.clamp(0.0, 1.0);
    fill(w, o, &needle(x, y, a, r * 0.9, r * 0.2, r * 0.045), col);
    fill(w, o, &circle(x, y, r * 0.07), ink.text);
}

impl GaugeFace {
    /// A watch dial: the scale engraved on a dark bezel round almost the whole turn with the
    /// zones on its rim, a plain face with a minute track, a register for the share at nine
    /// o'clock, a date-style window for the figure at three and a long centre hand reaching onto
    /// the bezel.
    pub(super) fn chrono_bezel(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = AROUND;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let (r_out, r_in) = (c - s * 0.01, c - s * 0.15);
        let r_face = r_in - s * 0.012;
        let sub = (c - s * 0.19, c, s * 0.085);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            // The bezel: a dark ring with a bright outer lip.
            fill(w, o, &circle(c, c, r_out), ink.ground);
            fill(w, o, &circle(c, c, r_out), scale_alpha(ink.text, 0.07));
            stroke(
                w,
                o,
                &circle(c, c, r_out),
                s * 0.008,
                scale_alpha(ink.text, 0.3),
                true,
            );
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(
                        c,
                        c,
                        r_out - s * 0.025,
                        r_out - s * 0.01,
                        start + span * a,
                        start + span * z,
                    ),
                    *zc,
                );
            }
            for i in 0..=marks {
                let a = start + span * i as f32 / marks as f32;
                let (len, wd) = if i % FINE == 0 {
                    (s * 0.03, s * 0.008)
                } else {
                    (s * 0.014, s * 0.004)
                };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r_in, a), polar(c, c, r_in + len, a)],
                    wd,
                    ink.text,
                    false,
                );
            }
            // The triangle at twelve, where the scale has its gap.
            fill(
                w,
                o,
                &[
                    polar(c, c, r_out - s * 0.015, 0.0),
                    polar(c, c, r_in + s * 0.035, -0.012),
                    polar(c, c, r_in + s * 0.035, 0.012),
                ],
                col,
            );
            // The face: plain, with a minute track.
            fill(w, o, &circle(c, c, r_face), ink.face);
            fill(w, o, &circle(c, c, r_face), scale_alpha(ink.text, 0.03));
            stroke(w, o, &circle(c, c, r_in), s * 0.006, ink.line, true);
            for i in 0..60 {
                let a = i as f32 / 60.0;
                let r0 = if i % 5 == 0 {
                    r_face - s * 0.04
                } else {
                    r_face - s * 0.025
                };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r0, a), polar(c, c, r_face - s * 0.012, a)],
                    s * 0.003,
                    ink.muted,
                    false,
                );
            }
            register(w, o, sub, (0.0, 1.0), (20, 5), &[], frac, col, ink);
            // The hand: a shadow, the long blade with a lume tip, a counterweight and the hub.
            let a = start + span * frac;
            let tip = r_in + s * 0.05;
            fill(
                w,
                o,
                &needle(c + s * 0.006, c + s * 0.008, a, tip, s * 0.14, s * 0.01),
                scale_alpha(ink.text, 0.12),
            );
            fill(w, o, &needle(c, c, a, tip, s * 0.14, s * 0.01), col);
            let (lx, ly) = polar(c, c, r_in - s * 0.035, a);
            fill(w, o, &circle(lx, ly, s * 0.016), col);
            stroke(w, o, &circle(lx, ly, s * 0.016), s * 0.004, ink.text, true);
            let (dx, dy) = dir(a);
            fill(
                w,
                o,
                &circle(c - dx * s * 0.11, c - dy * s * 0.11, s * 0.02),
                col,
            );
            fill(w, o, &circle(c, c, s * 0.03), ink.text);
            fill(w, o, &circle(c, c, s * 0.012), ink.face);
            chrono_glass(w, o, s, r_face, ink);
        };
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            let (x, y) = polar(c, c, r_in + s * 0.075, start + span * f);
            text_box(x, y - s * 0.028, s * 0.14)
                .text_size(px(s * 0.045))
                .font_weight(FontWeight::MEDIUM)
                .text_color(ink.text)
                .child(label)
                .into_any_element()
        });
        let (wx, wy, ww, wh) = (c + s * 0.07, c - s * 0.05, s * 0.24, s * 0.1);
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                text_box(c, c - s * 0.21, s * 0.4)
                    .text_size(px(s * 0.038))
                    .text_color(ink.muted)
                    .child(self.caption()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(wx))
                    .top(px(wy))
                    .w(px(ww))
                    .h(px(wh))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(s * 0.01))
                    .border_1()
                    .border_color(ink.line)
                    .bg(scale_alpha(ink.text, 0.06))
                    .text_size(px(s * 0.07))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(col)
                    .child(self.figure()),
            )
            .child(
                text_box(c, c + s * 0.16, s * 0.4)
                    .text_size(px(s * 0.035))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            )
            .into_any_element()
    }

    /// A wide watch case with two registers: the scale with its numerals and zones on the left,
    /// its share from 0 to 100 % on the right, the caption between them and the figure in a
    /// window under them.
    pub(super) fn chrono_register(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let h = s * REGISTER_H;
        let c = s / 2.0;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let r = s * 0.2;
        let (left, right) = ((s * 0.27, s * 0.31, r), (s * 0.73, s * 0.31, r));
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let case = rounded(0.0, 0.0, s, h, h * 0.3);
            fill(w, o, &case, scale_alpha(ink.text, 0.1));
            stroke(w, o, &case, s * 0.006, ink.line, true);
            let k = s * 0.025;
            let face = rounded(k, k, s - 2.0 * k, h - 2.0 * k, h * 0.3 - k);
            fill(w, o, &face, ink.face);
            fill(w, o, &face, scale_alpha(ink.text, 0.03));
            stroke(w, o, &face, s * 0.004, ink.line, true);
            register(w, o, left, SWEEP, (marks, FINE), &zones, frac, col, ink);
            register(w, o, right, (0.0, 1.0), (20, 5), &[], frac, col, ink);
            // A glint across the top of the crystal.
            fill(
                w,
                o,
                &rounded(k * 2.0, k * 1.5, s - 4.0 * k, h * 0.18, h * 0.09),
                scale_alpha(ink.text, 0.025),
            );
        };
        let num = s * 0.038;
        let mut share = Vec::new();
        for (i, label) in ["0", "25", "50", "75"].into_iter().enumerate() {
            let (x, y) = polar(right.0, right.1, r * 0.66, i as f32 * 0.25);
            share.push(
                text_box(x, y - num * 0.65, num * 3.0)
                    .text_size(px(num))
                    .text_color(ink.muted)
                    .child(label)
                    .into_any_element(),
            );
        }
        let window_w = s * 0.4;
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(self.numerals((left.0, left.1), r * 0.64, SWEEP, num, ink.text))
            .children(share)
            .child(
                text_box(right.0, right.1 + r * 0.32, s * 0.12)
                    .text_size(px(num * 0.8))
                    .text_color(ink.muted)
                    .child("%"),
            )
            .child(
                text_box(c, s * 0.035, s * 0.3)
                    .text_size(px(s * 0.035))
                    .text_color(ink.muted)
                    .child(self.caption()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - window_w / 2.0))
                    .top(px(s * 0.54))
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
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.032))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }
}
