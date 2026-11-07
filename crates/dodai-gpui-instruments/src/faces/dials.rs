//! Two more round layouts. The chronograph's sector dial: a watch face printed in concentric
//! bands, the scale in the outer band, a baton hand and the figure in a small window. The
//! vintage bezel: a cream gauge in a polished chrome ring, without the screwed panel, as set
//! into an old dashboard. Every colour comes from the theme.

use super::layouts::layer;
use super::line::hair;
use super::rich::{major_labels, zone_colours, FINE};
use super::GaugeFace;
use crate::dash::text_box;
use crate::draw::{hub, polar, sunburst, turn_arc};
use crate::frames::{aged, spade};
use crate::gauges::SWEEP;
use crate::shapes::{band, chrono_glass, circle, needle, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, Pixels, Styled};

/// The radii of a sector dial `s` wide: the outer scale band, the band of indices and the centre
/// disc, inside the case. Pure.
fn sector_radii(s: f32) -> [f32; 3] {
    [0.96, 0.74, 0.5].map(|k| (s / 2.0 - s * 0.04) * k)
}

impl GaugeFace {
    /// A sector dial: a polished case, the face printed in three concentric bands parted by
    /// hairline rings, the scale's fine marks and numerals in the outer band, baton indices in
    /// the middle one, a sunburst centre, a long baton hand with a lume tip and the figure in a
    /// small window at six o'clock.
    pub(super) fn chrono_sector(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let hand = self.hand_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let r_case = c - s * 0.01;
        let [ro, rm, ri] = sector_radii(s);
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            // The case: a polished ring with a bright lip.
            fill(w, o, &circle(c, c, r_case), ink.ground);
            fill(w, o, &circle(c, c, r_case), scale_alpha(ink.text, 0.14));
            stroke(
                w,
                o,
                &circle(c, c, r_case),
                hl * 2.0,
                scale_alpha(ink.text, 0.45),
                true,
            );
            // The face: the outer band a shade darker than the middle, the centre brushed.
            fill(w, o, &circle(c, c, ro), ink.face);
            fill(w, o, &circle(c, c, ro), scale_alpha(ink.ground, 0.25));
            fill(w, o, &circle(c, c, rm), ink.face);
            fill(w, o, &circle(c, c, ri), scale_alpha(ink.ground, 0.18));
            sunburst(w, o, (c, c), ri, ink.text);
            for r in [ro, rm, ri] {
                stroke(w, o, &circle(c, c, r), hl, scale_alpha(ink.text, 0.5), true);
            }
            // The zones as a fine band at the inside edge of the scale band.
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(c, c, rm + s * 0.006, rm + s * 0.016, at(*a), at(*z)),
                    *zc,
                );
            }
            // The scale's marks across the outer band: long at the majors.
            for i in 0..=marks {
                let a = at(i as f32 / marks as f32);
                let major = i % FINE == 0;
                let r0 = if major { ro - s * 0.04 } else { ro - s * 0.024 };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r0, a), polar(c, c, ro - s * 0.01, a)],
                    if major { hl * 2.2 } else { hl },
                    ink.text,
                    false,
                );
            }
            // Applied batons in the middle band at the majors.
            for f in &majors {
                let a = at(*f);
                let p = band(c, c, ri + s * 0.03, rm - s * 0.03, a - 0.004, a + 0.004);
                fill(w, o, &p, ink.text);
                stroke(w, o, &p, hl, scale_alpha(ink.ground, 0.5), true);
            }
            // The hand: a shadow, a long baton with a lume tip, a short tail and the hub.
            let a = at(frac);
            let tip = ro - s * 0.02;
            fill(
                w,
                o,
                &needle(c + s * 0.006, c + s * 0.008, a, tip, s * 0.1, s * 0.008),
                scale_alpha(ink.ground, 0.45),
            );
            fill(w, o, &needle(c, c, a, tip, s * 0.1, s * 0.008), hand);
            // The lume: a pale slip near the tip.
            stroke(
                w,
                o,
                &[polar(c, c, rm, a), polar(c, c, ro - s * 0.06, a)],
                s * 0.007,
                col,
                false,
            );
            hub(w, o, (c, c), s * 0.026, (ink.text, ink.face, hand));
            chrono_glass(w, o, s, r_case - s * 0.01, ink);
        };
        let numerals = self.numerals((c, c), rm + s * 0.035, SWEEP, s * 0.04, ink.text);
        let (ww, wh) = (s * 0.22, s * 0.075);
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                text_box(c, c - s * 0.2, s * 0.4)
                    .text_size(px(s * 0.03))
                    .text_color(ink.muted)
                    .child(self.caption().to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - ww / 2.0))
                    .top(px(c + s * 0.15))
                    .w(px(ww))
                    .h(px(wh))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(s * 0.01))
                    .rounded(px(s * 0.008))
                    .border_1()
                    .border_color(scale_alpha(ink.text, 0.4))
                    .bg(ink.ground)
                    .child(
                        div()
                            .text_size(px(s * 0.046))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(ink.text)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.026))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A vintage bezel: a cream gauge in a polished chrome ring, set straight into the dash
    /// without the screwed panel; a fine scale with numerals inside it, the zones as a band, a
    /// spade hand, a black hub and the caption and unit printed below the centre.
    pub(super) fn vintage_bezel(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let frac = self.frac();
        let hand = self.hand_colour(&t, &ink);
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE * 2;
        let r_out = c - s * 0.01;
        let r_face = c - s * 0.08;
        let r_scale = r_face - s * 0.03;
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            // Chrome: bands from bright to dark and back, like a turned ring catching light.
            let steps = 8;
            for k in 0..steps {
                let r = r_out - (r_out - r_face) * k as f32 / steps as f32;
                let shine = 1.0 - (k as f32 / (steps - 1) as f32 - 0.35).abs() * 1.6;
                fill(w, o, &circle(c, c, r), ink.ground);
                fill(
                    w,
                    o,
                    &circle(c, c, r),
                    scale_alpha(ink.text, 0.12 + 0.3 * shine.max(0.0)),
                );
            }
            stroke(
                w,
                o,
                &circle(c, c, r_out),
                hl * 1.5,
                scale_alpha(ink.text, 0.6),
                true,
            );
            // The cream face.
            fill(w, o, &circle(c, c, r_face), aged(ink));
            stroke(
                w,
                o,
                &circle(c, c, r_face),
                hl * 2.0,
                scale_alpha(ink.ground, 0.5),
                true,
            );
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(
                        c,
                        c,
                        r_scale - s * 0.05,
                        r_scale - s * 0.035,
                        at(*a),
                        at(*z),
                    ),
                    *zc,
                );
            }
            for i in 0..=marks {
                let a = at(i as f32 / marks as f32);
                let (len, wd) = if i % (FINE * 2) == 0 {
                    (s * 0.045, hl * 3.0)
                } else if i % FINE == 0 {
                    (s * 0.03, hl * 1.6)
                } else {
                    (s * 0.016, hl)
                };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r_scale - len, a), polar(c, c, r_scale, a)],
                    wd,
                    ink.text,
                    false,
                );
            }
            stroke(
                w,
                o,
                &turn_arc(c, c, r_scale, at(0.0), at(1.0)),
                hl,
                ink.text,
                false,
            );
            spade(c, c, at(frac), r_scale - s * 0.012).paint(w, o, s * 0.008, hand, ink);
            fill(w, o, &circle(c, c, s * 0.036), ink.text);
            fill(w, o, &circle(c, c, s * 0.03), scale_alpha(ink.ground, 0.9));
            fill(
                w,
                o,
                &circle(c - s * 0.008, c - s * 0.008, s * 0.008),
                scale_alpha(ink.face, 0.5),
            );
            chrono_glass(w, o, s, r_face, ink);
        };
        let numerals = self.numerals((c, c), r_scale - s * 0.1, SWEEP, s * 0.05, ink.text);
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(c + s * 0.12))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.004))
                    .child(
                        div()
                            .text_size(px(s * 0.034))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(ink.text)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.028))
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
    fn the_sector_bands_step_inwards_inside_the_case() {
        let s = 200.0;
        let r = sector_radii(s);
        assert!(r.windows(2).all(|p| p[0] > p[1]));
        assert!(r[2] > 0.0 && r[0] < s / 2.0 - s * 0.01);
    }
}
