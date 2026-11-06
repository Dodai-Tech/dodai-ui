//! The glass-cockpit and race gauge faces. Glass cockpit: an engine-indication arc that opens at
//! nine o'clock, its sweep filled to the value, a thin needle and the figure in a framed box.
//! Race: the caption on a slanted tag, a rising segment bar lit in the zone colours and the
//! figure big and leaning. Every colour comes from the theme.

use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::dash::text_box;
use crate::dial::severity_color;
use crate::draw::polar;
use crate::mfd::{arc_turn, drum_box, drum_digits, engine_arc, framed, page};
use crate::pod_math as pm;
use crate::race::{figure, plate, seg_bar, tag, tag_text};
use crate::shapes::Ink;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use dodai_instruments_core::instrument::Severity;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, IntoElement, ParentElement, Pixels,
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

impl GaugeFace {
    /// An engine-indication arc on a display page: the sweep filled to the value (tinted past a
    /// limit), zone bands with limit lines, a thin needle, the figure in a framed box whose last
    /// digit rolls on a drum (the box filled in the alarm colour past the alarm limit) and the
    /// caption in cyan.
    pub(super) fn cockpit(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let (ax, ay, r) = (s * 0.44, s * 0.5, s * 0.35);
        let frac = self.frac();
        let sev = pm::severity_at(&self.spans(), frac);
        let col = match sev {
            Severity::Normal => ink.text,
            sev => severity_color(&t, sev),
        };
        // An exceedance shows in inverse video: the box filled in the alarm colour.
        let alarm = sev == Severity::Alarm;
        let zones = zone_colours(&self, &t);
        let majors = self.scale.majors.max(1);
        // The box sits under the end of the arc, right of the centre; its last digit rolls on a
        // drum in a window that stands taller than the box.
        let bh = s * 0.12;
        let (cw, e) = (bh * 0.6, bh * 0.34);
        let (bx, by, bw) = (ax, ay + r * 0.42 + e, s * 0.38);
        let figure = self.figure();
        let drum = figure.chars().last().and_then(drum_digits);
        let head = match drum {
            Some(_) => figure[..figure.len() - 1].to_string(),
            None => figure.clone(),
        };
        let fg = if alarm { ink.ground } else { col };
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            engine_arc(w, o, (ax, ay), r, frac, majors, &zones, col, ink);
            let p = drum_box((bx, by, bw, bh), cw, e);
            framed(w, o, &p, (bh * 0.05).max(1.0), col, ink);
            if alarm {
                fill(w, o, &p, col);
            }
            // The caption's rule under the arc.
            stroke(
                w,
                o,
                &[(s * 0.12, s * 0.855), (s * 0.88, s * 0.855)],
                s * 0.003,
                scale_alpha(ink.text, 0.25),
                false,
            );
        };
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            let (x, y) = polar(ax, ay, r * 0.72, arc_turn(f));
            text_box(x, y - s * 0.022, s * 0.16)
                .text_size(px(s * 0.036))
                .text_color(scale_alpha(ink.text, 0.8))
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left(px(bx))
                    .top(px(by))
                    .w(px(bw - if drum.is_some() { cw } else { s * 0.02 }))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(bh * 0.66))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(head),
            )
            .children(drum.map(|digits| {
                let line = bh * 0.6;
                let window = bh + 2.0 * e;
                div()
                    .absolute()
                    .left(px(bx + bw - cw))
                    .top(px(by - e))
                    .w(px(cw))
                    .h(px(window))
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .left_0()
                            .top(px((window - 3.0 * line) / 2.0))
                            .w(px(cw))
                            .flex()
                            .flex_col()
                            .text_size(px(bh * 0.66))
                            .font_weight(FontWeight::MEDIUM)
                            .children(digits.into_iter().enumerate().map(move |(i, d)| {
                                // The digits either side of the shown one are on their way.
                                let c = if i == 1 { fg } else { scale_alpha(fg, 0.55) };
                                div()
                                    .h(px(line))
                                    .line_height(px(line))
                                    .flex()
                                    .justify_center()
                                    .text_color(c)
                                    .child(d.to_string())
                            })),
                    )
            }))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.12))
                    .top(px(s * 0.87))
                    .w(px(s * 0.76))
                    .flex()
                    .justify_between()
                    .text_size(px(s * 0.045))
                    .child(
                        div()
                            .text_color(ink.info)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(div().text_color(ink.muted).child(self.unit.to_uppercase())),
            )
            .into_any_element()
    }

    /// A race display: the caption on a slanted tag, a rising segment bar lit to the value in
    /// the zone colours with the scale under it, and the figure big and leaning.
    pub(super) fn race(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let frac = self.frac();
        let col = match pm::severity_at(&self.spans(), frac) {
            Severity::Normal => ink.text,
            sev => severity_color(&t, sev),
        };
        let zones = zone_colours(&self, &t);
        let bar = (s * 0.08, s * 0.25, s * 0.84, s * 0.18);
        let head = (s * 0.08, s * 0.12, s * 0.44, s * 0.08);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            tag(w, o, head, ink.primary);
            let colour = |at: f32| {
                zones
                    .iter()
                    .find(|(a, z, _)| at >= *a && at <= *z)
                    .map_or(ink.ok, |z| z.2)
            };
            seg_bar(w, o, bar, frac, 18, true, colour, ink);
        };
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            text_box(bar.0 + bar.2 * f, bar.1 + bar.3 + s * 0.02, s * 0.14)
                .text_size(px(s * 0.034))
                .italic()
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .child(tag_text(head, ink.on_primary, self.caption()))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.55))
                    .top(px(head.1))
                    .w(px(s * 0.37))
                    .h(px(head.3))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(s * 0.04))
                    .italic()
                    .text_color(ink.muted)
                    .child(self.unit.to_uppercase()),
            )
            .children(numerals)
            .child(
                text_box(c, s * 0.53, s * 0.9)
                    .flex()
                    .justify_center()
                    .child(figure(self.figure(), s * 0.3, col)),
            )
            .into_any_element()
    }
}
