//! [`CountUp`]: a number that counts from zero to its value using the
//! app-wide / section [`Motion`](crate::Motion) (same duration, easing and
//! stagger as the charts' value-in animation).

use crate::motion::grow;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_ui_core::math::fmt_num;
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};

/// Animated numeric readout (`0 -> value`).
#[derive(IntoElement)]
pub struct CountUp {
    id: ElementId,
    value: f32,
    prefix: SharedString,
    suffix: SharedString,
    decimals: usize,
    size: Option<f32>,
}

impl CountUp {
    /// `id` must be unique among siblings (it keys the animation state).
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            prefix: SharedString::default(),
            suffix: SharedString::default(),
            decimals: 0,
            size: None,
        }
    }
    /// Text before the number.
    pub fn prefix(mut self, p: impl Into<SharedString>) -> Self {
        self.prefix = p.into();
        self
    }
    /// Text after the number.
    pub fn suffix(mut self, s: impl Into<SharedString>) -> Self {
        self.suffix = s.into();
        self
    }
    /// Digits after the decimal point.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Font size in px (default: the theme's large size).
    pub fn size(mut self, s: f32) -> Self {
        self.size = Some(s);
        self
    }
}

impl RenderOnce for CountUp {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = window.with_id(self.id.clone(), |w| grow(w, cx, "CountUp"));
        let v = self.value * g;
        let text = if self.decimals == 0 {
            fmt_num(v.round())
        } else {
            format!("{v:.*}", self.decimals)
        };
        div()
            .font_family(first_family(t.typography.mono))
            .text_size(px(self.size.unwrap_or(t.typography.size_lg)))
            .text_color(hsla(t.palette.text))
            .child(format!("{}{text}{}", self.prefix, self.suffix))
    }
}
