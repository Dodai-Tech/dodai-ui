//! The line look ([`Look::Line`](crate::gpui::look::Look::Line)): the quietest of the looks. A gauge
//! is one hairline ring or line with a dot riding on it, a thin arc up to the value, the scale's
//! end figures and a large light figure in open space. No bezel, face or fill; every colour
//! comes from the theme.

use super::layouts::{along, layer};
use super::rich::{major_labels, zone_colours};
use super::round::ROUND;
use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::draw::{polar, turn_arc};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use dodai_gpui::Icon;
use gpui::prelude::FluentBuilder;
use gpui::{div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, Pixels, Styled};

/// Height of the line bar, as a share of its width.
const LINE_BAR_H: f32 = 0.4;

/// A filled dot of radius `r` at `(x, y)`.
fn dot(w: &mut gpui::Window, o: gpui::Point<Pixels>, (x, y): (f32, f32), r: f32, col: gpui::Hsla) {
    let pts: Vec<(f32, f32)> = (0..=24).map(|i| polar(x, y, r, i as f32 / 24.0)).collect();
    fill(w, o, &pts, col);
}

/// The hairline of a face `s` wide. Pure.
pub(super) fn hair(s: f32) -> f32 {
    (s * 0.003).max(0.6)
}

impl GaugeFace {
    /// The line dial: a hairline ring over three quarters of a turn, dots at the major ticks, a
    /// thin arc up to the value with a dot riding on its end, and the figure alone in the middle.
    pub(super) fn line_dial(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let c = s / 2.0;
        let r = s * 0.43;
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f.clamp(0.0, 1.0);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            stroke(
                w,
                o,
                &turn_arc(c, c, r, at(0.0), at(1.0)),
                hl,
                scale_alpha(ink.text, 0.28),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r - s * 0.022, at(*a), at(*z)),
                    hl * 1.6,
                    scale_alpha(*zc, 0.9),
                    false,
                );
            }
            for f in &majors {
                dot(
                    w,
                    o,
                    polar(c, c, r - s * 0.022, at(*f)),
                    hl * 1.3,
                    ink.muted,
                );
            }
            if frac > 0.0 {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, at(0.0), at(frac)),
                    (s * 0.008).max(1.5),
                    col,
                    false,
                );
            }
            let tip = polar(c, c, r, at(frac));
            dot(w, o, tip, s * 0.022, ink.ground);
            dot(w, o, tip, s * 0.014, col);
        };
        let ends = self.end_figures().map(|(f, txt)| {
            let (x, y) = polar(c, c, r - s * 0.07, at(f));
            text_box(x, y, s * 0.16)
                .text_size(px(s * 0.032))
                .text_color(ink.muted)
                .child(txt)
                .into_any_element()
        });
        let centre = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .when_some(self.icon, |d, i| {
                d.child(Icon::symbol(i).size(s * 0.07).weight(1.0).color(ink.muted))
            })
            .child(
                div()
                    .text_size(px(s * 0.22))
                    .line_height(px(s * 0.24))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(figure_col)
                    .child(self.figure()),
            )
            .child(
                div()
                    .text_size(px(s * 0.034))
                    .text_color(ink.muted)
                    .child(self.unit.to_uppercase()),
            )
            .when(!self.label.is_empty(), |d| {
                d.child(
                    div()
                        .mt(px(s * 0.012))
                        .text_size(px(s * 0.03))
                        .text_color(ink.muted)
                        .child(self.label.to_uppercase()),
                )
            });
        div()
            .relative()
            .size(px(s))
            .child(layer(paint))
            .children(ends)
            .child(centre)
            .into_any_element()
    }

    /// The line bar: the caption and the figure over a hairline, a thin line up to the value
    /// with a dot on its end, and the scale's end figures under the ends.
    pub(super) fn line_bar(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * LINE_BAR_H;
        let (x0, x1) = (s * 0.04, s * 0.96);
        let y = h * 0.74;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            stroke(
                w,
                o,
                &[(x0, y), (x1, y)],
                hl,
                scale_alpha(ink.text, 0.28),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &[
                        (along(x0, x1, *a), y + s * 0.02),
                        (along(x0, x1, *z), y + s * 0.02),
                    ],
                    hl * 1.6,
                    scale_alpha(*zc, 0.9),
                    false,
                );
            }
            for f in &majors {
                dot(w, o, (along(x0, x1, *f), y + s * 0.02), hl * 1.3, ink.muted);
            }
            let end = along(x0, x1, frac);
            if frac > 0.0 {
                stroke(w, o, &[(x0, y), (end, y)], (s * 0.006).max(1.5), col, false);
            }
            dot(w, o, (end, y), s * 0.016, ink.ground);
            dot(w, o, (end, y), s * 0.01, col);
        };
        let ends = self.end_figures().map(|(f, txt)| {
            text_box(along(x0, x1, f), y + s * 0.06, s * 0.14)
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
            .children(ends)
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(h * 0.04))
                    .w(px(x1 - x0))
                    .flex()
                    .items_end()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(s * 0.015))
                            .when_some(self.icon, |d, i| {
                                d.child(Icon::symbol(i).size(s * 0.04).weight(1.0).color(ink.muted))
                            })
                            .child(
                                div()
                                    .text_size(px(s * 0.036))
                                    .text_color(ink.muted)
                                    .child(self.caption().to_uppercase()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_baseline()
                            .gap(px(s * 0.012))
                            .child(
                                div()
                                    .text_size(px(s * 0.16))
                                    .line_height(px(s * 0.17))
                                    .font_weight(FontWeight::LIGHT)
                                    .text_color(figure_col)
                                    .child(self.figure()),
                            )
                            .child(
                                div()
                                    .text_size(px(s * 0.03))
                                    .text_color(ink.muted)
                                    .child(self.unit.to_uppercase()),
                            ),
                    ),
            )
            .into_any_element()
    }

    /// The first and last major figures with their place on the scale: the only figures the
    /// line look writes.
    pub(super) fn end_figures(&self) -> impl Iterator<Item = (f32, String)> {
        let labels = major_labels(self);
        let n = labels.len();
        labels
            .into_iter()
            .enumerate()
            .filter(move |(i, _)| *i == 0 || *i + 1 == n)
            .map(|(_, l)| l)
    }
}

#[cfg(test)]
mod tests {
    use super::hair;

    #[test]
    fn the_hairline_grows_with_the_face_but_never_vanishes() {
        assert_eq!(hair(10.0), 0.6);
        assert!((hair(400.0) - 1.2).abs() < 1e-6);
    }
}
