//! The glass-cockpit and nixie gauge faces. Glass cockpit: an engine-indication arc that opens at
//! nine o'clock, its sweep filled to the value, a thin needle and the figure in a framed box.
//! Nixie: the figure in glow tubes over a bargraph tube with its scale, and a neon bulb that
//! lights in the zone colour. Every colour comes from the theme.

use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::dash::text_box;
use crate::dial::severity_color;
use crate::draw::polar;
use crate::mfd::{arc_turn, engine_arc, page, readout_box};
use crate::pod_math as pm;
use crate::shapes::Ink;
use crate::tubes::{bargraph, bulb, chassis, glow, tubes_at, tubes_width};
use dodai_gpui::paint::{scale_alpha, stroke};
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
    /// An engine-indication arc on a display page: the sweep filled to the value, zone bands
    /// with limit lines, a thin needle, the figure in a framed box and the caption in cyan.
    pub(super) fn cockpit(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let (ax, ay, r) = (s * 0.42, s * 0.47, s * 0.33);
        let frac = self.frac();
        let col = match pm::severity_at(&self.spans(), frac) {
            Severity::Normal => ink.text,
            sev => severity_color(&t, sev),
        };
        let zones = zone_colours(&self, &t);
        let majors = self.scale.majors.max(1);
        // The box sits under the end of the arc, right of the centre.
        let (bx, by, bw, bh) = (ax, ay + r * 0.42, s * 0.38, s * 0.13);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            engine_arc(w, o, (ax, ay), r, frac, majors, &zones, col, ink);
            readout_box(w, o, (bx, by, bw, bh), false, col, ink);
            // The caption's rule under the arc.
            stroke(
                w,
                o,
                &[(s * 0.12, s * 0.8), (s * 0.88, s * 0.8)],
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
                    .w(px(bw - s * 0.02))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(bh * 0.66))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(col)
                    .child(self.figure()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.12))
                    .top(px(s * 0.82))
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

    /// Glow tubes on a chassis: the figure in tubes, the level in a bargraph tube over its
    /// scale, the caption on a strip and a neon bulb lit in the zone colour.
    pub(super) fn nixie(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let c = s / 2.0;
        let frac = self.frac();
        let sev = pm::severity_at(&self.spans(), frac);
        let neon = glow(ink);
        let (lamp, lit) = match sev {
            Severity::Normal => (neon, false),
            sev => (severity_color(&t, sev), true),
        };
        let bar_col = if lit { lamp } else { neon };
        let zones = zone_colours(&self, &t);
        let fig = self.figure();
        // Wide figures get shorter tubes so they fit the window.
        let h = (s * 0.3).min(s * 0.76 / (tubes_width(&fig, 1.0).max(0.01)));
        let (gx0, gx1, gy, gh) = (s * 0.12, s * 0.88, s * 0.62, s * 0.075);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            bargraph(w, o, (gx0, gy, gx1 - gx0, gh), frac, false, bar_col, ink);
            let y = gy + gh + s * 0.015;
            for i in 0..=20 {
                let x = gx0 + (gx1 - gx0) * i as f32 / 20.0;
                let len = if i % 5 == 0 { s * 0.03 } else { s * 0.015 };
                stroke(
                    w,
                    o,
                    &[(x, y), (x, y + len)],
                    s * 0.003,
                    scale_alpha(ink.text, 0.45),
                    false,
                );
            }
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &[
                        (gx0 + (gx1 - gx0) * a, y - s * 0.004),
                        (gx0 + (gx1 - gx0) * z, y - s * 0.004),
                    ],
                    s * 0.008,
                    *zc,
                    false,
                );
            }
            bulb(w, o, (s * 0.14, s * 0.12), s * 0.035, lamp, lit, ink);
        };
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            text_box(gx0 + (gx1 - gx0) * f, gy + gh + s * 0.05, s * 0.14)
                .text_size(px(s * 0.032))
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(paint))
            .child(
                text_box(c, s * 0.095, s * 0.6)
                    .text_size(px(s * 0.036))
                    .text_color(ink.muted)
                    .child(self.caption().to_uppercase()),
            )
            .child(tubes_at(
                &fig,
                c,
                s * 0.19 + (s * 0.3 - h) / 2.0,
                h,
                neon,
                ink,
            ))
            .children(numerals)
            .child(
                text_box(c, s * 0.83, s * 0.6)
                    .text_size(px(s * 0.034))
                    .text_color(ink.muted)
                    .child(self.unit.to_uppercase()),
            )
            .into_any_element()
    }
}
