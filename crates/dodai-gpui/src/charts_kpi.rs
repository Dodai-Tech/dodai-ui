//! [`KpiTile`]: one headline number, its change since last time and its recent trend.

use crate::charts::{down_color, fmt_with, up_color, NumFmt, Sparkline};
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{div, px, App, IntoElement, ParentElement, SharedString, Styled, Window};
use std::rc::Rc;

/// A key figure: the label, the value (counting to it as it changes), the change since the
/// previous value (coloured by whether it is good news) and an optional trend sparkline.
#[derive(IntoElement)]
pub struct KpiTile {
    follow: crate::instruments_motion::Follow,
    label: SharedString,
    value: f32,
    previous: Option<f32>,
    trend: Vec<f32>,
    lower_is_better: bool,
    fmt: Option<NumFmt>,
}

impl KpiTile {
    crate::instruments_motion::follow_api!();
    /// Create a new [`KpiTile`] showing `value` under `label`.
    #[track_caller]
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            label: label.into(),
            value,
            previous: None,
            trend: Vec::new(),
            lower_is_better: false,
            fmt: None,
        }
    }
    /// The value last period; the tile shows the change from it.
    pub fn previous(mut self, v: f32) -> Self {
        self.previous = v.is_finite().then_some(v);
        self
    }
    /// Recent values, drawn as a sparkline under the number.
    pub fn trend(mut self, values: impl IntoIterator<Item = f32>) -> Self {
        self.trend = values.into_iter().collect();
        self
    }
    /// A fall is good news (costs, errors, latency): falls show in the rising colour.
    pub fn lower_is_better(mut self, l: bool) -> Self {
        self.lower_is_better = l;
        self
    }
    /// Custom number formatter for the value.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }
}

/// The relative change from `prev` to `v` in percent (`None` without a usable base).
fn change(prev: f32, v: f32) -> Option<f32> {
    (prev.is_finite() && v.is_finite() && prev != 0.0).then(|| (v - prev) / prev.abs() * 100.0)
}

impl KpiTile {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let shown =
            crate::instruments_motion::follow_all(window, cx, &self.follow.sub("v"), &[self.value])
                .first()
                .copied()
                .unwrap_or(self.value);
        let delta = self.previous.and_then(|p| change(p, self.value));
        let chip = delta.map(|d| {
            let good = (d >= 0.0) != self.lower_is_better;
            let col = if d.abs() < 0.05 {
                hsla(t.palette.text_muted)
            } else if good {
                up_color(&t)
            } else {
                down_color(&t)
            };
            let arrow = if d.abs() < 0.05 {
                "■"
            } else if d > 0.0 {
                "▲"
            } else {
                "▼"
            };
            div()
                .font_family(first_family(t.typography.mono))
                .text_size(px(t.typography.size_sm))
                .text_color(col)
                .child(format!("{arrow} {:.1}%", d.abs()))
        });
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .min_w(px(160.0))
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(self.label.clone()),
            )
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(t.spacing.sm))
                    .child(
                        div()
                            .font_family(first_family(t.typography.display))
                            .text_size(px(t.typography.size_xxl))
                            .text_color(hsla(t.palette.text))
                            .child(fmt_with(&self.fmt, shown)),
                    )
                    .children(chip),
            );
        if self.trend.len() > 1 {
            root = root.child(Sparkline::new(self.trend).area(true).size(160.0, 32.0));
        }
        root
    }
}

crate::entry_render!(KpiTile);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_change_is_relative_to_the_previous_value() {
        assert_eq!(change(100.0, 125.0), Some(25.0));
        assert_eq!(change(-50.0, -25.0), Some(50.0));
        assert_eq!(change(0.0, 5.0), None);
        assert_eq!(change(f32::NAN, 5.0), None);
        assert_eq!(
            KpiTile::new("x", 1.0).previous(f32::INFINITY).previous,
            None
        );
    }
}
