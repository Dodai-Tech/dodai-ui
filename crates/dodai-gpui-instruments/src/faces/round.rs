//! The sport look's round variations beside its pod: the sweep, a thick segmented band filled
//! to the value; the needle, a long centre pointer over an outside scale with the figure in a
//! tab; and the ribbed ring, concentric grooves crossed by a bright cursor. Each is a full
//! round: a closed bright rim, the scale over three quarters of the turn and the bottom quarter
//! put to use, for the needle's figure or for a chip showing where the first limit lies (filled
//! once the value reaches it). Every colour comes from the theme.

use super::layouts::layer;
use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::dash::text_box;
use crate::draw::{polar, turn_arc};
use crate::mfd::glow;
use crate::shapes::{band, circle, needle};
use dodai_gpui::icons::Icon;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::readout;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// The colours a sport face draws with, from the theme.
#[derive(Clone, Copy)]
struct Sport {
    ink: Hsla,
    muted: Hsla,
    accent: Hsla,
    ground: Hsla,
}

impl Sport {
    fn of(t: &dodai_core::Theme) -> Self {
        Self {
            ink: hsla(t.palette.text),
            muted: hsla(t.palette.text_muted),
            accent: hsla(t.palette.primary),
            ground: hsla(t.palette.background),
        }
    }
}

/// The parts every round sport variation shares: rim radius and width, the end colours, the
/// zones and the ticks `(fraction, major)`.
struct Parts {
    r_rim: f32,
    width: f32,
    ends: [Option<Hsla>; 2],
    zones: Vec<(f32, f32, Hsla)>,
    ticks: Vec<(f32, bool)>,
}

/// The scale of the round variations: three quarters of the turn over the top, leaving the
/// quarter at the bottom for a readout set into the closed rim.
pub(crate) const ROUND: (f32, f32) = (0.625, 0.75);

/// The share of the scale at each end that takes an end's colour.
const CAP: f32 = 0.06;

/// Paint a polished bezel of outer radius `r` round a face centred on `(c, c)`: a bright outer
/// ring, a hairline inside it, the bottom quarter between them lit as a reflection, and the
/// ends of the scale in their colours.
fn full_rim(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    (c, r, width): (f32, f32, f32),
    ink: Hsla,
    ends: [Option<Hsla>; 2],
) {
    let (start, span) = ROUND;
    stroke(
        w,
        o,
        &circle(c, c, r),
        width * 0.7,
        scale_alpha(ink, 0.95),
        true,
    );
    stroke(
        w,
        o,
        &circle(c, c, r - width * 1.5),
        width * 0.25,
        scale_alpha(ink, 0.4),
        true,
    );
    fill(
        w,
        o,
        &band(c, c, r - width * 1.5, r, start + span, start + 1.0),
        scale_alpha(ink, 0.18),
    );
    for (i, end) in ends.iter().enumerate() {
        if let Some(col) = end {
            let a = start + span * if i == 0 { 0.0 } else { 1.0 - CAP };
            fill(
                w,
                o,
                &band(c, c, r - width * 1.4, r - width * 0.3, a, a + span * CAP),
                *col,
            );
        }
    }
}

/// Paint fine concentric hairlines filling a disc of radius `r`, as a guilloché dial.
fn guilloche(w: &mut gpui::Window, o: gpui::Point<Pixels>, c: f32, r: f32, ink: Hsla) {
    let n = 18;
    for k in 1..=n {
        let rk = r * k as f32 / n as f32;
        stroke(
            w,
            o,
            &circle(c, c, rk),
            (r * 0.004).max(0.5),
            scale_alpha(ink, 0.05),
            true,
        );
    }
}

/// Paint an applied index at turn `a`: a polished baton from `r` inwards by `len`, its bright
/// face split by a darker facet line.
#[allow(clippy::too_many_arguments)]
fn index(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    c: f32,
    (r, len, width): (f32, f32, f32),
    a: f32,
    ink: Hsla,
    ground: Hsla,
) {
    let pts = [polar(c, c, r, a), polar(c, c, r - len, a)];
    stroke(w, o, &pts, width, ink, false);
    stroke(w, o, &pts, width * 0.22, scale_alpha(ground, 0.3), false);
}

/// Paint a railway track over the scale: two hairlines from `r` to `r - gap` with the minor
/// ticks between them.
fn railway(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    c: f32,
    (r, gap): (f32, f32),
    ticks: &[(f32, bool)],
    ink: Hsla,
) {
    let (start, span) = ROUND;
    let hair = (gap * 0.06).max(0.5);
    for rr in [r, r - gap] {
        stroke(
            w,
            o,
            &turn_arc(c, c, rr, start, start + span),
            hair,
            scale_alpha(ink, 0.5),
            false,
        );
    }
    for (f, _) in ticks {
        let a = start + span * f;
        stroke(
            w,
            o,
            &[polar(c, c, r, a), polar(c, c, r - gap, a)],
            hair * 1.4,
            scale_alpha(ink, 0.6),
            false,
        );
    }
}

/// The first limit on the scale: the lowest start of the zones `(from, to)`, if any. Pure.
pub(crate) fn first_limit(zones: &[(f32, f32)]) -> Option<f32> {
    zones.iter().map(|z| z.0).reduce(f32::min)
}

/// Where a zone `(from, to)` overlaps the filled part of a sweep up to `frac`, if it does. Pure.
pub(crate) fn filled_part(from: f32, to: f32, frac: f32) -> Option<(f32, f32)> {
    let end = to.min(frac);
    (end > from).then_some((from, end))
}

impl GaugeFace {
    fn sport_parts(&self, t: &dodai_core::Theme) -> Parts {
        let s = self.size;
        Parts {
            r_rim: s / 2.0 - s * 0.012,
            width: (s * 0.014).max(2.0),
            ends: self.ends.map(|e| e.map(|tone| tone.color(t))),
            zones: zone_colours(self, t),
            ticks: self
                .scale
                .ticks()
                .into_iter()
                .map(|k| (k.frac, k.major))
                .collect(),
        }
    }

    /// The icon, label, figure and unit stacked in the middle of a round sport face.
    fn sport_middle(&self, p: Sport, top: f32, figure: f32) -> AnyElement {
        let s = self.size;
        div()
            .absolute()
            .left(px(s * 0.2))
            .top(px(top))
            .w(px(s * 0.6))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(s * 0.008))
            .when_some(self.icon, |d, i| {
                d.child(Icon::symbol(i).size(s * 0.08).color(p.ink))
            })
            .when(!self.label.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(s * 0.036))
                        .text_color(p.muted)
                        .child(self.label.to_uppercase()),
                )
            })
            .child(
                div()
                    .text_size(px(figure))
                    .line_height(px(figure * 1.05))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(p.ink)
                    .child(self.figure()),
            )
            .child(
                div()
                    .text_size(px(s * 0.05))
                    .text_color(p.muted)
                    .child(self.unit.clone()),
            )
            .into_any_element()
    }

    /// The chip in the bottom quarter of a round sport face: where the first limit lies, in its
    /// colour, and filled with that colour once the value has reached it.
    fn limit_chip(&self, t: &dodai_core::Theme, p: Sport) -> Option<AnyElement> {
        let s = self.size;
        let zones = zone_colours(self, t);
        let spans: Vec<(f32, f32)> = zones.iter().map(|z| (z.0, z.1)).collect();
        let at = first_limit(&spans)?;
        let col = zones.iter().find(|z| z.0 == at).map(|z| z.2)?;
        let reached = self.frac() >= at;
        let v = self.scale.value_at(at);
        let w = s * 0.24;
        Some(
            div()
                .absolute()
                .left(px(s / 2.0 - w / 2.0))
                .top(px(s / 2.0 + s * 0.3))
                .w(px(w))
                .h(px(s * 0.06))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(s * 0.015))
                .rounded(px(s * 0.03))
                .border_1()
                .border_color(scale_alpha(col, 0.8))
                .when(reached, |d| d.bg(col))
                .text_size(px(s * 0.034))
                .text_color(if reached { p.ground } else { col })
                .child("LIMIT")
                .child(readout(v, self.decimals, ""))
                .into_any_element(),
        )
    }

    /// Numerals for every major on a circle of radius `r`.
    fn sport_numerals(&self, r: f32, size: f32, col: Hsla) -> Vec<AnyElement> {
        let c = self.size / 2.0;
        let (start, span) = ROUND;
        major_labels(self)
            .into_iter()
            .map(|(f, label)| {
                let (x, y) = polar(c, c, r, start + span * f);
                text_box(x, y - size * 0.62, size * 3.0)
                    .text_size(px(size))
                    .text_color(col)
                    .child(label)
                    .into_any_element()
            })
            .collect()
    }

    /// A sport sweep: a slim band filled to the value, brightening towards a lit leading edge
    /// and taking a zone's colour inside it, the rest of each zone traced along the band; a
    /// railway track and applied indices outside it and a guilloché centre round the figure.
    pub(super) fn sport_sweep(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let p = Sport::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f;
        let frac = self.frac();
        let Parts {
            r_rim,
            width,
            ends,
            zones,
            ticks,
        } = self.sport_parts(&t);
        let rt = r_rim - width * 2.6;
        let (r1, r0) = (rt - s * 0.075, rt - s * 0.1);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            guilloche(w, o, c, r0 - s * 0.09, p.ink);
            full_rim(w, o, (c, r_rim, width), p.ink, ends);
            railway(w, o, c, (rt, s * 0.022), &ticks, p.ink);
            for (f, major) in &ticks {
                if *major {
                    index(
                        w,
                        o,
                        c,
                        (rt - s * 0.028, s * 0.04, width * 1.1),
                        at(*f),
                        p.ink,
                        p.ground,
                    );
                }
            }
            fill(
                w,
                o,
                &band(c, c, r0, r1, at(0.0), at(1.0)),
                scale_alpha(p.ink, 0.07),
            );
            let n = 48;
            for i in 0..n {
                let (u, v) = (frac * i as f32 / n as f32, frac * (i + 1) as f32 / n as f32);
                let k = 0.25 + 0.7 * (i + 1) as f32 / n as f32;
                fill(
                    w,
                    o,
                    &band(c, c, r0, r1, at(u), at(v) + 0.001),
                    scale_alpha(p.ink, k),
                );
            }
            for (a, z, col) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r1 + s * 0.006, at(*a), at(*z)),
                    s * 0.004,
                    *col,
                    false,
                );
                if let Some((u, v)) = filled_part(*a, *z, frac) {
                    fill(w, o, &band(c, c, r0, r1, at(u), at(v)), *col);
                }
            }
            let edge = [
                polar(c, c, r0 - s * 0.012, at(frac)),
                polar(c, c, r1 + s * 0.012, at(frac)),
            ];
            glow(w, o, &edge, width * 0.9, p.ink, false);
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.sport_numerals(r0 - s * 0.055, s * 0.042, p.ink))
            .child(self.sport_middle(p, c - s * 0.17, s * 0.15))
            .children(self.limit_chip(&t, p))
            .into_any_element()
    }

    /// A sport needle: a railway track with applied indices under the bezel, the zones as a
    /// fine band inside it and a hairline in the accent tracing the travel; a slim lance with a
    /// counterweight from a polished hub over a guilloché centre, and the figure in an aperture
    /// in the bottom quarter.
    pub(super) fn sport_needle(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let p = Sport::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f;
        let frac = self.frac();
        let Parts {
            r_rim,
            width,
            ends,
            zones,
            ticks,
        } = self.sport_parts(&t);
        let rt = r_rim - width * 2.6;
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            guilloche(w, o, c, rt - s * 0.12, p.ink);
            full_rim(w, o, (c, r_rim, width), p.ink, ends);
            railway(w, o, c, (rt, s * 0.022), &ticks, p.ink);
            for (f, major) in &ticks {
                if *major {
                    index(
                        w,
                        o,
                        c,
                        (rt - s * 0.028, s * 0.055, width * 1.2),
                        at(*f),
                        p.ink,
                        p.ground,
                    );
                }
            }
            for (a, z, col) in &zones {
                fill(
                    w,
                    o,
                    &band(c, c, rt - s * 0.034, rt - s * 0.024, at(*a), at(*z)),
                    *col,
                );
            }
            if frac > 0.0 {
                let trace = turn_arc(c, c, rt - s * 0.1, at(0.0), at(frac));
                stroke(w, o, &trace, s * 0.004, scale_alpha(p.accent, 0.7), false);
            }
            let a = at(frac);
            let tip = rt - s * 0.012;
            let shadow = needle(c + s * 0.006, c + s * 0.009, a, tip, s * 0.1, s * 0.01);
            fill(w, o, &shadow, scale_alpha(p.ground, 0.5));
            fill(w, o, &needle(c, c, a, tip, s * 0.1, s * 0.01), p.accent);
            let (bx, by) = polar(c, c, s * 0.075, a + 0.5);
            fill(w, o, &circle(bx, by, s * 0.018), p.accent);
            fill(w, o, &circle(c, c, s * 0.038), p.ink);
            fill(w, o, &circle(c, c, s * 0.028), p.ground);
            fill(w, o, &circle(c, c, s * 0.011), p.accent);
        };
        let (aw, ah) = (s * 0.3, s * 0.09);
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.sport_numerals(rt - s * 0.13, s * 0.046, p.ink))
            .child(
                text_box(c, c - s * 0.2, s * 0.5)
                    .text_size(px(s * 0.036))
                    .text_color(p.muted)
                    .child(self.caption().to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - aw / 2.0))
                    .top(px(c + s * 0.26))
                    .w(px(aw))
                    .h(px(ah))
                    .flex()
                    .justify_center()
                    .items_center()
                    .gap(px(s * 0.012))
                    .rounded(px(s * 0.012))
                    .border_1()
                    .border_color(scale_alpha(p.ink, 0.45))
                    .bg(scale_alpha(p.ground, 0.6))
                    .child(
                        div()
                            .text_size(px(s * 0.058))
                            .line_height(px(s * 0.06))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(p.ink)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.032))
                            .text_color(p.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A sport ribbed ring: fine concentric grooves lit up to the value, the outer grooves
    /// traced in the zones' colours, crossed by applied indices at the majors and a slim
    /// polished cursor at the value; the figure in the middle.
    pub(super) fn sport_ribbed(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let p = Sport::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f;
        let frac = self.frac();
        let Parts {
            r_rim,
            width,
            ends,
            zones,
            ticks,
        } = self.sport_parts(&t);
        let (r0, r1) = (r_rim - s * 0.15, r_rim - width * 2.6);
        let ribs = 14;
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            full_rim(w, o, (c, r_rim, width), p.ink, ends);
            let hair = (s * 0.0025).max(0.6);
            for k in 0..ribs {
                let r = r0 + (r1 - r0) * (k as f32 + 0.5) / ribs as f32;
                let outer = k >= ribs - 3;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, at(0.0), at(1.0)),
                    hair,
                    scale_alpha(p.ink, 0.14),
                    false,
                );
                if frac > 0.0 {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, at(0.0), at(frac)),
                        hair,
                        scale_alpha(p.ink, 0.5),
                        false,
                    );
                }
                if outer {
                    for (a, z, col) in &zones {
                        stroke(
                            w,
                            o,
                            &turn_arc(c, c, r, at(*a), at(*z)),
                            hair * 1.6,
                            *col,
                            false,
                        );
                    }
                }
            }
            for (f, major) in &ticks {
                if *major {
                    index(
                        w,
                        o,
                        c,
                        (r1, (r1 - r0) * 0.45, width * 1.0),
                        at(*f),
                        p.ink,
                        p.ground,
                    );
                }
            }
            let a = at(frac);
            let cursor = [
                polar(c, c, r0 - s * 0.012, a),
                polar(c, c, r1 + width * 0.6, a),
            ];
            glow(w, o, &cursor, width * 0.9, p.accent, false);
            index(
                w,
                o,
                c,
                (r1 + width * 0.6, r1 - r0 + s * 0.018, width * 1.3),
                a,
                p.ink,
                p.ground,
            );
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(self.sport_numerals(r0 - s * 0.05, s * 0.042, p.ink))
            .child(self.sport_middle(p, c - s * 0.17, s * 0.15))
            .children(self.limit_chip(&t, p))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_limit_is_the_lowest_zone_start() {
        assert_eq!(first_limit(&[(0.9, 1.0), (0.75, 0.9)]), Some(0.75));
        assert_eq!(first_limit(&[]), None, "no zones, no limit");
    }

    #[test]
    fn a_zone_fills_only_up_to_the_value() {
        assert_eq!(filled_part(0.8, 1.0, 0.9), Some((0.8, 0.9)));
        assert_eq!(
            filled_part(0.8, 1.0, 0.5),
            None,
            "the value has not reached it"
        );
        assert_eq!(
            filled_part(0.2, 0.4, 0.9),
            Some((0.2, 0.4)),
            "passed: filled whole"
        );
    }
}
