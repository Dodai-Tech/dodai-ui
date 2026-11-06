//! The vintage and halo looks' extra layouts (see [`Look::layouts`](crate::look::Look::layouts)): the vintage
//! ribbon speedometer, and the halo's arc and bar. Each takes the same scale and value as the
//! dial and is drawn as wide as the face's size, flatter than a dial. Every colour comes from the
//! theme.

use super::rich::{major_labels, zone_colours, FINE};
use super::GaugeFace;
use crate::dash::text_box;
use crate::draw::{polar, turn_arc};
use crate::frames::{aged, glow_arc, glow_dot, light_blade, name_plate, plate_inset, screw, wake};
use crate::shapes::{band, chamfered, rounded, Ink};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, IntoElement, ParentElement, Pixels,
    Styled,
};

/// Height of the ribbon speedometer, as a share of its width.
pub(crate) const RIBBON_H: f32 = 0.56;
/// Height of the halo arc, as a share of its width.
pub(crate) const ARC_H: f32 = 0.62;
/// Height of the halo bar, as a share of its width.
pub(crate) const BAR_H: f32 = 0.5;

/// The x of `frac` on a straight scale from `x0` to `x1`, clamped to it. Pure.
pub(crate) fn along(x0: f32, x1: f32, frac: f32) -> f32 {
    x0 + (x1 - x0) * frac.clamp(0.0, 1.0)
}

pub(super) fn layer(
    paint: impl Fn(&mut gpui::Window, gpui::Point<Pixels>) + 'static,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

/// Paint a glowing straight bar from `x0` to `x1` at `y`, its colour running `from` → `to`.
#[allow(clippy::too_many_arguments)]
fn glow_bar(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    (x0, x1): (f32, f32),
    y: f32,
    width: f32,
    (from, to): (gpui::Hsla, gpui::Hsla),
) {
    if x1 <= x0 {
        return;
    }
    let n = (((x1 - x0) / width).ceil() as usize).clamp(2, 60);
    for i in 0..n {
        let u0 = x0 + (x1 - x0) * i as f32 / n as f32;
        let u1 = x0 + (x1 - x0) * (i + 1) as f32 / n as f32;
        let col = mix(from, to, (i as f32 + 0.5) / n as f32);
        let seg = [(u0, y), (u1 + 0.5, y)];
        stroke(w, o, &seg, width * 4.0, scale_alpha(col, 0.08), false);
        stroke(w, o, &seg, width * 2.2, scale_alpha(col, 0.18), false);
        stroke(w, o, &seg, width, col, false);
    }
}

impl GaugeFace {
    /// A ribbon speedometer: a screwed panel with a long warm window, a straight engraved scale,
    /// a coloured ribbon that runs out to the value and a name plate with the figure.
    pub(super) fn ribbon(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let h = s * RIBBON_H;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let (wx, wy, ww, wh) = (s * 0.07, s * 0.07, s * 0.86, s * 0.26);
        let (x0, x1) = (wx + s * 0.06, wx + ww - s * 0.06);
        let tick_y = wy + s * 0.085;
        let ribbon_y = wy + s * 0.15;
        let plate = (s * 0.1, wy + wh + s * 0.045, s * 0.8, s * 0.11);
        let inset = plate_inset(plate.3);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let panel = chamfered(0.0, 0.0, s, h, s * 0.03);
            fill(w, o, &panel, ink.face);
            fill(w, o, &panel, scale_alpha(ink.text, 0.08));
            stroke(w, o, &panel, s * 0.006, ink.line, true);
            let win = chamfered(wx, wy, ww, wh, s * 0.02);
            fill(w, o, &win, aged(ink));
            stroke(w, o, &win, s * 0.008, scale_alpha(ink.text, 0.3), true);
            for i in 0..=marks {
                let x = along(x0, x1, i as f32 / marks as f32);
                let major = i % FINE == 0;
                let len = if major { s * 0.035 } else { s * 0.018 };
                stroke(
                    w,
                    o,
                    &[(x, tick_y), (x, tick_y + len)],
                    if major { s * 0.006 } else { s * 0.003 },
                    if major { ink.text } else { ink.muted },
                    false,
                );
            }
            for (a, z, zc) in &zones {
                let r = [
                    (along(x0, x1, *a), tick_y + s * 0.042),
                    (along(x0, x1, *z), tick_y + s * 0.042),
                ];
                stroke(w, o, &r, s * 0.012, *zc, false);
            }
            // The groove, then the ribbon run out to the value with a bright edge.
            let groove = rounded(x0, ribbon_y, x1 - x0, s * 0.06, s * 0.008);
            fill(w, o, &groove, scale_alpha(ink.text, 0.1));
            let end = along(x0, x1, frac);
            if end > x0 {
                fill(
                    w,
                    o,
                    &rounded(x0, ribbon_y, end - x0, s * 0.06, s * 0.008),
                    col,
                );
                stroke(
                    w,
                    o,
                    &[(end, ribbon_y - s * 0.01), (end, ribbon_y + s * 0.07)],
                    s * 0.006,
                    ink.text,
                    false,
                );
            }
            // A sheen over the top of the glass.
            fill(
                w,
                o,
                &chamfered(wx, wy, ww, wh * 0.3, s * 0.02),
                scale_alpha(ink.text, 0.05),
            );
            name_plate(w, o, plate, ink);
            for (i, at) in [
                (s * 0.035, s * 0.035),
                (s - s * 0.035, s * 0.035),
                (s * 0.035, h - s * 0.035),
                (s - s * 0.035, h - s * 0.035),
            ]
            .into_iter()
            .enumerate()
            {
                screw(w, o, at, s * 0.016, 0.15 + i as f32 * 0.2, ink);
            }
        };
        let labels = major_labels(&self).into_iter().map(move |(f, txt)| {
            text_box(along(x0, x1, f), wy + s * 0.025, s * 0.12)
                .text_size(px(s * 0.036))
                .text_color(ink.text)
                .child(txt)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(labels)
            .child(
                div()
                    .absolute()
                    .left(px(plate.0 + inset))
                    .top(px(plate.1))
                    .w(px((plate.2 - 2.0 * inset).max(0.0)))
                    .h(px(plate.3))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(s * 0.038))
                            .text_color(ink.text)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(s * 0.012))
                            .child(
                                div()
                                    .text_size(px(s * 0.065))
                                    .text_color(ink.text)
                                    .child(self.figure()),
                            )
                            .child(
                                div()
                                    .text_size(px(s * 0.03))
                                    .text_color(ink.muted)
                                    .child(self.unit.clone()),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// A halo arc: a half ring of light over a large thin figure, with a fading wake and a
    /// glowing blade at the value.
    pub(super) fn halo_arc(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let s = self.size;
        let h = s * ARC_H;
        let c = s / 2.0;
        let cy = s * 0.5;
        let r = s * 0.4;
        let (start, span) = (0.75_f32, 0.5_f32);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let dots: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            // A soft glow under the ring, fading outwards.
            for k in 0..6 {
                let d = s * 0.012 * k as f32;
                fill(
                    w,
                    o,
                    &band(
                        c,
                        cy,
                        r - s * 0.06 - d,
                        r + s * 0.03 + d,
                        start,
                        start + span,
                    ),
                    scale_alpha(ink.info, 0.025),
                );
            }
            stroke(
                w,
                o,
                &turn_arc(c, cy, r, start, start + span),
                s * 0.024,
                scale_alpha(ink.text, 0.07),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, cy, r + s * 0.028, start + span * a, start + span * z),
                    s * 0.006,
                    scale_alpha(*zc, 0.85),
                    false,
                );
            }
            // A fine dotted ring outside, one dot per major tick.
            for f in &dots {
                let p = polar(c, cy, r + s * 0.05, start + span * f);
                fill(
                    w,
                    o,
                    &crate::shapes::circle(p.0, p.1, s * 0.004),
                    scale_alpha(ink.text, 0.4),
                );
            }
            let a = start + span * frac;
            wake(
                w,
                o,
                (c, cy),
                (r * 0.82, r - s * 0.012),
                a,
                0.1_f32.min(span * frac),
                col,
            );
            glow_arc(w, o, (c, cy), r, (start, a), s * 0.012, (ink.info, col));
            light_blade(
                w,
                o,
                (c, cy),
                (r * 0.7, r + s * 0.035),
                a,
                s * 0.006,
                col,
                ink.text,
            );
        };
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(self.numerals((c, cy), r - s * 0.075, (start, span), s * 0.034, ink.muted))
            .child(
                text_box(c, cy - s * 0.25, s * 0.5)
                    .text_size(px(s * 0.03))
                    .text_color(ink.primary)
                    .child(self.caption().to_uppercase()),
            )
            .child(
                text_box(c, cy - s * 0.215, s * 0.5)
                    .text_size(px(s * 0.17))
                    .text_color(figure_col)
                    .font_weight(FontWeight::LIGHT)
                    .child(self.figure()),
            )
            .child(
                text_box(c, cy - s * 0.01, s * 0.4)
                    .text_size(px(s * 0.034))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            )
            .child(
                text_box(c, cy + s * 0.05, s * 0.3)
                    .text_size(px(s * 0.03))
                    .text_color(col)
                    .child(format!("{:.0} %", frac * 100.0)),
            )
            .into_any_element()
    }

    /// A halo bar: a dark glass tile with a large thin figure over a straight bar of light, the
    /// scale's numerals under it and a glowing dot at the value.
    pub(super) fn halo_bar(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let s = self.size;
        let h = s * BAR_H;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let (x0, x1) = (s * 0.08, s * 0.92);
        let y = s * 0.34;
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let tile = rounded(0.0, 0.0, s, h, s * 0.06);
            fill(w, o, &tile, ink.ground);
            // Light from below the bar, fading upwards.
            for k in 0..8 {
                let top = y - s * 0.03 * k as f32;
                fill(
                    w,
                    o,
                    &rounded(0.0, top.max(0.0), s, h - top.max(0.0), s * 0.06),
                    scale_alpha(ink.info, 0.012),
                );
            }
            stroke(w, o, &tile, s * 0.004, scale_alpha(ink.info, 0.3), true);
            stroke(
                w,
                o,
                &[(x0, y), (x1, y)],
                s * 0.024,
                scale_alpha(ink.text, 0.07),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &[
                        (along(x0, x1, *a), y + s * 0.03),
                        (along(x0, x1, *z), y + s * 0.03),
                    ],
                    s * 0.006,
                    scale_alpha(*zc, 0.85),
                    false,
                );
            }
            let end = along(x0, x1, frac);
            glow_bar(w, o, (x0, end), y, s * 0.012, (ink.info, col));
            glow_dot(w, o, (end, y), s * 0.012, col, ink.text);
        };
        let labels = major_labels(&self).into_iter().map(move |(f, txt)| {
            text_box(along(x0, x1, f), y + s * 0.05, s * 0.12)
                .text_size(px(s * 0.03))
                .text_color(ink.muted)
                .child(txt)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(labels)
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(s * 0.04))
                    .w(px(x1 - x0))
                    .flex()
                    .justify_between()
                    .text_size(px(s * 0.03))
                    .child(
                        div()
                            .text_color(ink.primary)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(
                        div()
                            .text_color(col)
                            .child(format!("{:.0} %", frac * 100.0)),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(s * 0.06))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(s * 0.15))
                            .line_height(px(s * 0.17))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.034))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_straight_scale_runs_from_end_to_end_and_clamps() {
        assert_eq!(along(10.0, 110.0, 0.0), 10.0);
        assert_eq!(along(10.0, 110.0, 0.5), 60.0);
        assert_eq!(along(10.0, 110.0, 1.0), 110.0);
        assert_eq!(along(10.0, 110.0, 2.0), 110.0);
        assert_eq!(along(10.0, 110.0, -1.0), 10.0);
    }
}
