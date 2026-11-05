//! The other gauge looks: a half-circle with a needle, a ring of fine ticks, a ring of dots, a
//! ladder of rising bars and a big digital readout with a ruler.

use super::{GaugeFace, Ink};
use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::pod_math as pm;
use dodai_gpui::paint::{scale_alpha, stroke};
use dodai_gpui::theme_of;
use dodai_instruments_core::instrument::{readout, Severity};
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    Styled,
};

/// Fine ticks around the ring look.
const TICKS: usize = 60;
/// Dots around the dot look.
const DOTS: usize = 24;
/// Bars of the ladder.
const RUNGS: usize = 14;
/// Marks on the digital ruler.
const RULER: usize = 40;

impl GaugeFace {
    /// The colour of position `f` on the scale: its zone, else the accent.
    fn colour_at(&self, t: &dodai_core::Theme, ink: &Ink, f: f32) -> Hsla {
        match pm::severity_at(&self.spans(), f) {
            Severity::Normal => ink.primary,
            sev => severity_color(t, sev),
        }
    }

    /// A lit or unlit colour for the `i`th of `n` marks: lit up to the value, dim zone tints after.
    fn mark_colours(&self, t: &dodai_core::Theme, ink: &Ink, n: usize) -> Vec<Hsla> {
        let lit = pm::segments_lit(self.frac(), n);
        (0..n)
            .map(|i| {
                let f = (i as f32 + 0.5) / n as f32;
                if i < lit {
                    self.colour_at(t, ink, f)
                } else {
                    match pm::severity_at(&self.spans(), f) {
                        Severity::Normal => scale_alpha(ink.text, 0.13),
                        sev => scale_alpha(severity_color(t, sev), 0.3),
                    }
                }
            })
            .collect()
    }

    /// A half circle: a thick rim split into zones, a needle and the figure underneath.
    pub(super) fn half(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let (start, span) = (0.75_f32, 0.5_f32);
        let c = size / 2.0;
        let r = c - size * 0.08;
        let weight = size * 0.09;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let mut bands: Vec<(f32, f32, Hsla)> = vec![(0.0, 1.0, scale_alpha(ink.text, 0.16))];
        for (a, z, sev) in self.spans() {
            bands.push((a, z, scale_alpha(severity_color(&t, sev), 0.85)));
        }
        let (muted, text) = (scale_alpha(ink.text, 0.5), ink.text);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                for (a, z, colr) in &bands {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, start + span * a, start + span * z),
                        weight,
                        *colr,
                        false,
                    );
                }
                for i in 0..=20 {
                    let a = start + span * i as f32 / 20.0;
                    let r0 = r - weight * if i % 5 == 0 { 0.95 } else { 0.7 };
                    stroke(
                        w,
                        o,
                        &[polar(c, c, r0, a), polar(c, c, r - weight * 0.55, a)],
                        size * 0.007,
                        muted,
                        false,
                    );
                }
                let a = start + span * frac;
                stroke(
                    w,
                    o,
                    &[polar(c, c, r * 0.12, a), polar(c, c, r + weight * 0.45, a)],
                    size * 0.02,
                    col,
                    false,
                );
                stroke(
                    w,
                    o,
                    &[(c - 1.0, c), (c + 1.0, c)],
                    size * 0.07,
                    text,
                    false,
                );
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(size))
            .h(px(size * 0.62))
            .child(cv)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(c + size * 0.07))
                    .w(px(size))
                    .flex()
                    .justify_center()
                    .items_end()
                    .gap(px(size * 0.02))
                    .child(
                        div()
                            .text_size(px(size * 0.17))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .pb(px(size * 0.025))
                            .text_size(px(size * 0.06))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A full ring of fine radial ticks that light up, the figure in the middle.
    pub(super) fn ticks(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let c = size / 2.0;
        let cols = self.mark_colours(&t, &ink, TICKS);
        let lit = pm::segments_lit(self.frac(), TICKS);
        let (r_out, r_in, r_long) = (c - size * 0.04, c - size * 0.13, c - size * 0.17);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                for (i, col) in cols.iter().enumerate() {
                    let a = (i as f32 + 0.5) / TICKS as f32;
                    // The lit tip is longer, so the eye finds the value.
                    let inner = if i + 1 == lit { r_long } else { r_in };
                    stroke(
                        w,
                        b.origin,
                        &[polar(c, c, inner, a), polar(c, c, r_out, a)],
                        size * 0.011,
                        *col,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(size * 0.3))
                    .w(px(size))
                    .h(px(size * 0.4))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.readout_column(&ink, size, 0.26, ink.text)),
            )
            .into_any_element()
    }

    /// A ring of round dots.
    pub(super) fn dots(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let c = size / 2.0;
        let r = c - size * 0.07;
        let d = size * 0.075;
        let cols = self.mark_colours(&t, &ink, DOTS);
        let mut face = div().relative().w(px(size)).h(px(size));
        for (i, col) in cols.into_iter().enumerate() {
            let (x, y) = polar(c, c, r, (i as f32 + 0.5) / DOTS as f32);
            face = face.child(
                div()
                    .absolute()
                    .left(px(x - d / 2.0))
                    .top(px(y - d / 2.0))
                    .w(px(d))
                    .h(px(d))
                    .rounded_full()
                    .bg(col),
            );
        }
        face.child(
            div()
                .absolute()
                .left_0()
                .top(px(size * 0.25))
                .w(px(size))
                .h(px(size * 0.5))
                .flex()
                .items_center()
                .justify_center()
                .child(self.readout_column(&ink, size, 0.2, ink.text)),
        )
        .into_any_element()
    }

    /// Bars of growing height, lit up to the value, the figure beside them.
    pub(super) fn ladder(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let cols = self.mark_colours(&t, &ink, RUNGS);
        let h = size * 0.42;
        let mut bars = div()
            .flex()
            .items_end()
            .gap(px(size * 0.012))
            .w(px(size))
            .h(px(h));
        for (i, col) in cols.into_iter().enumerate() {
            let k = 0.25 + 0.75 * (i as f32 + 1.0) / RUNGS as f32;
            bars = bars.child(
                div()
                    .flex_1()
                    .h(px(h * k))
                    .rounded(px(size * 0.012))
                    .bg(col),
            );
        }
        div()
            .flex()
            .flex_col()
            .gap(px(size * 0.05))
            .w(px(size))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .justify_between()
                    .child(self.heading(&ink, size))
                    .child(
                        div()
                            .text_size(px(size * 0.06))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .child(bars)
            .child(
                div()
                    .text_size(px(size * 0.2))
                    .text_color(ink.text)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.figure()),
            )
            .into_any_element()
    }

    /// A big figure over a ruler of fine marks with a pointer; no frame at all.
    pub(super) fn digital(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let (w_all, h_ruler) = (size, size * 0.12);
        let marks: Vec<Hsla> = (0..=RULER)
            .map(|i| {
                let f = i as f32 / RULER as f32;
                match pm::severity_at(&self.spans(), f) {
                    Severity::Normal => scale_alpha(ink.text, if i % 5 == 0 { 0.6 } else { 0.3 }),
                    sev => severity_color(&t, sev),
                }
            })
            .collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                for (i, m) in marks.iter().enumerate() {
                    let x = w_all * i as f32 / RULER as f32;
                    let long = i % 5 == 0;
                    let y0 = if long { 0.0 } else { h_ruler * 0.35 };
                    stroke(
                        w,
                        b.origin,
                        &[(x, y0 + h_ruler * 0.3), (x, h_ruler + h_ruler * 0.3)],
                        size * 0.007,
                        *m,
                        false,
                    );
                }
                let x = w_all * frac;
                stroke(
                    w,
                    b.origin,
                    &[(x, 0.0), (x, h_ruler * 1.6)],
                    size * 0.02,
                    col,
                    false,
                );
            },
        )
        .w(px(w_all))
        .h(px(h_ruler * 1.6));
        div()
            .flex()
            .flex_col()
            .gap(px(size * 0.03))
            .w(px(size))
            .child(self.heading(&ink, size))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(size * 0.025))
                    .child(
                        div()
                            .text_size(px(size * 0.34))
                            .text_color(col)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .pb(px(size * 0.06))
                            .text_size(px(size * 0.07))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .child(cv)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(size * 0.05))
                    .text_color(ink.muted)
                    .child(readout(self.scale.min, 0, ""))
                    .child(readout(self.scale.max, 0, "")),
            )
            .into_any_element()
    }
}
