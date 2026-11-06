//! The vintage, segment-LCD and halo gauge faces. Vintage: a round bezel on a screwed plate, an
//! engraved scale on a warm face, a spade hand and a drum counter; its alternative is a panel
//! meter with the pivot below the face. Segment LCD: an LCD module with a segment arc and a
//! seven-segment figure. Halo: a halo with a gradient arc of light, a fading wake and a
//! glowing blade. Every colour comes from the theme.

use super::rich::{major_labels, zone_colours, FINE};
use super::GaugeFace;
use crate::dash::text_box;
use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::frames::{
    aged, engraved_scale, ghost, glow_arc, halo, lcd, light_blade, lit_segments, name_plate, plate,
    plate_inset, screw, seg_arc, seg_text, spade, wake,
};
use crate::gauges::SWEEP;
use crate::pod_math as pm;
use crate::shapes::{band, chrono_glass, circle, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};

/// Width of the vintage dial's caption plate, as a share of the face.
const CAPTION_W: f32 = 0.32;
/// Height of the vintage dial's caption plate, as a share of the face.
const CAPTION_H: f32 = 0.055;
use dodai_gpui::theme_of;
use dodai_instruments_core::instrument::Severity;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    Styled,
};

/// A canvas over the whole face.
fn layer(paint: impl Fn(&mut gpui::Window, gpui::Point<Pixels>) + 'static) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

/// The characters of a drum counter for `figure`: digits get a drum each, the rest sit between.
/// Pure.
pub(crate) fn drums(figure: &str) -> Vec<(char, bool)> {
    figure.chars().map(|c| (c, c.is_ascii_digit())).collect()
}

impl GaugeFace {
    /// The colour of a hand: the ink, or the zone colour inside a zone.
    fn hand_colour(&self, t: &dodai_core::Theme, ink: &Ink) -> Hsla {
        match pm::severity_at(&self.spans(), self.frac()) {
            Severity::Normal => ink.text,
            sev => severity_color(t, sev),
        }
    }

    /// A vintage dial: plate, bezel, warm face, engraved scale, zone band, numerals, a spade
    /// hand, a caption plate and a drum counter. With [`GaugeFace::alternate`] it is a panel meter.
    pub(super) fn vintage(self, cx: &mut gpui::App) -> AnyElement {
        if self.alternate {
            return self.panel_meter(cx);
        }
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let hand = self.hand_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let majors = self.scale.majors.max(1);
        let r_face = c - s * 0.1;
        let r_scale = r_face - s * 0.035;
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(
                        c,
                        c,
                        r_scale - s * 0.07,
                        r_scale - s * 0.055,
                        start + span * a,
                        start + span * z,
                    ),
                    *zc,
                );
            }
            engraved_scale(w, o, (c, c), r_scale, SWEEP, majors, s, ink);
            name_plate(
                w,
                o,
                (
                    c - CAPTION_W / 2.0 * s,
                    c - s * 0.135,
                    CAPTION_W * s,
                    CAPTION_H * s,
                ),
                ink,
            );
            spade(c, c, start + span * frac, r_scale - s * 0.01).paint(w, o, s * 0.008, hand, ink);
            fill(w, o, &circle(c, c, s * 0.032), ink.text);
            fill(w, o, &circle(c, c, s * 0.012), aged(ink));
            chrono_glass(w, o, s, r_face, ink);
        };
        let num = s * 0.05;
        let (dw, dh) = (s * 0.062, s * 0.09);
        let drum_row = drums(&self.figure()).into_iter().map(move |(ch, drum)| {
            let cell = div()
                .flex()
                .justify_center()
                .items_center()
                .h(px(dh))
                .text_size(px(s * 0.068))
                .font_weight(FontWeight::MEDIUM)
                .child(ch.to_string());
            if drum {
                cell.w(px(dw))
                    .bg(ink.text)
                    .text_color(aged(ink))
                    .border_1()
                    .border_color(ink.line)
                    .into_any_element()
            } else {
                cell.w(px(dw * 0.4)).text_color(ink.text).into_any_element()
            }
        });
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.numerals((c, c), r_scale - s * 0.11, SWEEP, num, ink.text))
            .child(
                text_box(
                    c,
                    c - s * 0.13,
                    (CAPTION_W * s - 2.0 * plate_inset(CAPTION_H * s)).max(0.0),
                )
                .text_size(px(s * 0.032))
                .text_color(ink.text)
                .child(self.caption().to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(c + s * 0.15))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.008))
                    .child(
                        div()
                            .flex()
                            .gap(px(s * 0.004))
                            .p(px(s * 0.006))
                            .bg(scale_alpha(ink.text, 0.15))
                            .children(drum_row),
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

    /// A panel meter: a wide warm window with a flat scale arc, its zones as a thick band, a long
    /// needle from a pivot below the window, screws in the corners and a name plate.
    fn panel_meter(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let plate_h = s * 0.24;
        let pivot = (c, s * 0.96);
        let r = s * 0.64;
        let (start, span) = (0.9_f32, 0.2_f32);
        let frac = self.frac();
        let col = self.hand_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let window = (s * 0.07, s * 0.07, s * 0.86, s - plate_h - s * 0.1);
        let inset = plate_inset(plate_h * 0.62);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let (px_, py_) = pivot;
            let panel = crate::shapes::chamfered(0.0, 0.0, s, s, s * 0.03);
            fill(w, o, &panel, ink.face);
            fill(w, o, &panel, scale_alpha(ink.text, 0.08));
            stroke(w, o, &panel, s * 0.006, ink.line, true);
            let (wx, wy, ww, wh) = window;
            let win = crate::shapes::chamfered(wx, wy, ww, wh, s * 0.02);
            fill(w, o, &win, aged(ink));
            stroke(w, o, &win, s * 0.008, scale_alpha(ink.text, 0.3), true);
            stroke(
                w,
                o,
                &turn_arc(px_, py_, r, start, start + span),
                s * 0.006,
                ink.text,
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
                let a = start + span * i as f32 / marks as f32;
                let major = i % FINE == 0;
                let len = if major { s * 0.06 } else { s * 0.03 };
                stroke(
                    w,
                    o,
                    &[polar(px_, py_, r, a), polar(px_, py_, r + len, a)],
                    if major { s * 0.008 } else { s * 0.004 },
                    if major { ink.text } else { ink.muted },
                    false,
                );
            }
            // Mirror strip under the scale, as on a precision meter.
            stroke(
                w,
                o,
                &turn_arc(px_, py_, r - s * 0.06, start, start + span),
                s * 0.012,
                scale_alpha(ink.text, 0.12),
                false,
            );
            let a = start + span * frac;
            stroke(
                w,
                o,
                &[
                    polar(px_, py_, r * 0.42, a),
                    polar(px_, py_, r + s * 0.07, a),
                ],
                s * 0.008,
                col,
                false,
            );
            // The window's own shadow over the needle's root.
            fill(
                w,
                o,
                &[
                    (wx, wy + wh - s * 0.05),
                    (wx + ww, wy + wh - s * 0.05),
                    (wx + ww, wy + wh),
                    (wx, wy + wh),
                ],
                scale_alpha(ink.text, 0.08),
            );
            name_plate(w, o, (s * 0.1, s - plate_h, s * 0.8, plate_h * 0.62), ink);
            for (i, at) in [
                (s * 0.035, s * 0.035),
                (s - s * 0.035, s * 0.035),
                (s * 0.035, s - s * 0.035),
                (s - s * 0.035, s - s * 0.035),
            ]
            .into_iter()
            .enumerate()
            {
                screw(w, o, at, s * 0.016, 0.15 + i as f32 * 0.2, ink);
            }
        };
        let labels: Vec<AnyElement> = major_labels(&self)
            .into_iter()
            .map(|(f, txt)| {
                let (x, y) = polar(pivot.0, pivot.1, r + s * 0.095, start + span * f);
                text_box(x, y - s * 0.03, s * 0.2)
                    .text_size(px(s * 0.045))
                    .text_color(ink.text)
                    .child(txt)
                    .into_any_element()
            })
            .collect();
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(labels)
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.1 + inset))
                    .top(px(s - plate_h))
                    .w(px((s * 0.8 - 2.0 * inset).max(0.0)))
                    .h(px(plate_h * 0.62))
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
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(self.figure()),
                            )
                            .child(
                                div()
                                    .text_size(px(s * 0.032))
                                    .text_color(ink.muted)
                                    .child(self.unit.clone()),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// A segment display: an LCD module, a segment arc lit to the value in the zone colours, the
    /// scale's numbers inside it and a seven-segment figure.
    pub(super) fn digital(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let c = s / 2.0;
        let cy = c + s * 0.03;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let spans = self.spans();
        let tt = t.clone();
        let lit = lit_segments(40, frac, move |f| match pm::severity_at(&spans, f) {
            Severity::Normal => ink.primary,
            sev => severity_color(&tt, sev),
        });
        let off = ghost(ink.primary, t.is_dark());
        let zones = zone_colours(&self, &t);
        let majors = self.scale.majors.max(1);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            seg_arc(w, o, (c, cy), (s * 0.33, s * 0.4), (start, span), &lit, off);
            // Zones as a thin printed line outside the arc.
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, cy, s * 0.415, start + span * a, start + span * z),
                    s * 0.006,
                    *zc,
                    false,
                );
            }
            for i in 0..=majors {
                let a = start + span * i as f32 / majors as f32;
                stroke(
                    w,
                    o,
                    &[polar(c, cy, s * 0.305, a), polar(c, cy, s * 0.32, a)],
                    s * 0.006,
                    scale_alpha(ink.text, 0.5),
                    false,
                );
            }
        };
        let fig = self.figure();
        let h = s * 0.16;
        let unit = if self.unit.is_empty() {
            self.label.to_uppercase()
        } else {
            self.unit.to_uppercase()
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.numerals((c, cy), s * 0.26, SWEEP, s * 0.036, ink.muted))
            .child(
                text_box(c, s * 0.055, s * 0.6)
                    .text_size(px(s * 0.034))
                    .text_color(ink.primary)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.caption().to_uppercase()),
            )
            .child(seg_text(&fig, c, cy - h * 0.62, h, col))
            .child(
                text_box(c, cy + h * 0.5, s * 0.4)
                    .text_size(px(s * 0.032))
                    .text_color(ink.muted)
                    .child(unit),
            )
            .child(
                text_box(c, cy + s * 0.29, s * 0.5)
                    .gap(px(s * 0.01))
                    .text_size(px(s * 0.03))
                    .text_color(ink.muted)
                    .child(format!("{:.0} %", frac * 100.0)),
            )
            .into_any_element()
    }

    /// A halo of light: a gradient arc to the value, a fading wake, a glowing blade and the figure
    /// in thin type over a soft orb; with [`AtLimit::Display`](crate::look::AtLimit::Display)
    /// the whole halo and the figure take a limit's colour.
    pub(super) fn futuristic(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let r_in = c - s * 0.095;
        let r_t = r_in - s * 0.06;
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            stroke(
                w,
                o,
                &turn_arc(c, c, r_t, start, start + span),
                s * 0.024,
                scale_alpha(ink.text, 0.07),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_t + s * 0.026, start + span * a, start + span * z),
                    s * 0.006,
                    scale_alpha(*zc, 0.85),
                    false,
                );
            }
            let a = start + span * frac;
            wake(
                w,
                o,
                (c, c),
                (r_t * 0.45, r_t - s * 0.012),
                a,
                0.1_f32.min(span * frac),
                col,
            );
            glow_arc(w, o, (c, c), r_t, (start, a), s * 0.012, (ink.info, col));
            light_blade(
                w,
                o,
                (c, c),
                (r_t * 0.6, r_t + s * 0.035),
                a,
                s * 0.006,
                col,
                ink.text,
            );
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.numerals((c, c), r_t - s * 0.075, SWEEP, s * 0.036, ink.muted))
            .child(
                text_box(c, c - s * 0.15, s * 0.5)
                    .text_size(px(s * 0.03))
                    .text_color(ink.primary)
                    .child(self.caption().to_uppercase()),
            )
            .child(
                text_box(c, c - s * 0.11, s * 0.5)
                    .text_size(px(s * 0.17))
                    .text_color(figure_col)
                    .font_weight(FontWeight::LIGHT)
                    .child(self.figure()),
            )
            .child(
                text_box(c, c + s * 0.1, s * 0.4)
                    .text_size(px(s * 0.034))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            )
            .child(
                text_box(c, c + s * 0.29, s * 0.3)
                    .text_size(px(s * 0.03))
                    .text_color(col)
                    .child(format!("{:.0} %", frac * 100.0)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_digits_turn_on_drums() {
        assert_eq!(
            drums("12.5"),
            vec![('1', true), ('2', true), ('.', false), ('5', true)]
        );
        assert!(drums("").is_empty());
    }
}
