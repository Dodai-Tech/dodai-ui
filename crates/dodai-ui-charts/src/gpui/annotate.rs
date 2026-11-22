//! [`Annotation`]: reference lines and highlighted bands drawn over a chart.
//!
//! Charts that take `.annotations(...)` place them in data coordinates:
//! the Y value is on the value axis; the X value is a category index (so
//! `2.0` is the third category's centre) or, on scatter plots, an X value.

use crate::gpui::common::{label_style, norm};
use dodai_ui_core::{Color, Theme};
use dodai_ui_gpui::hsla;
use dodai_ui_gpui::paint::scale_alpha;
use gpui::{div, px, relative, Div, ParentElement, SharedString, Styled};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    HLine(f32),
    VLine(f32),
    YBand(f32, f32),
    XBand(f32, f32),
}

/// A threshold line or highlighted range on a chart.
#[derive(Clone, Debug)]
pub struct Annotation {
    kind: Kind,
    label: Option<SharedString>,
    color: Option<Color>,
}

impl Annotation {
    fn of(kind: Kind) -> Self {
        Self {
            kind,
            label: None,
            color: None,
        }
    }
    /// Horizontal dashed line at Y value `y` (a target or threshold).
    pub fn hline(y: f32) -> Self {
        Self::of(Kind::HLine(y))
    }
    /// Vertical dashed line at X (category index, or X value on scatter plots).
    pub fn vline(x: f32) -> Self {
        Self::of(Kind::VLine(x))
    }
    /// Shaded horizontal band between two Y values (an acceptable range).
    pub fn y_band(lo: f32, hi: f32) -> Self {
        Self::of(Kind::YBand(lo.min(hi), lo.max(hi)))
    }
    /// Shaded vertical band between two X positions (a highlighted period).
    pub fn x_band(from: f32, to: f32) -> Self {
        Self::of(Kind::XBand(from.min(to), from.max(to)))
    }
    /// Caption drawn at the line or band.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Colour (default: the muted text colour for lines, the primary for bands).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

/// Fraction of the plot (0 = left/bottom edge, 1 = right/top edge) or `None`
/// when `v` falls outside it.
fn inside(f: f32) -> Option<f32> {
    (f.is_finite() && (-1e-3..=1.0 + 1e-3).contains(&f)).then(|| f.clamp(0.0, 1.0))
}

/// The overlay for `anns`: `y` is the value range of the plot, `x` maps an
/// annotation X to a `0..=1` fraction across it.
pub(crate) fn layer(t: &Theme, anns: &[Annotation], y: (f32, f32), x: &dyn Fn(f32) -> f32) -> Div {
    let mut out = div().absolute().size_full();
    let tag = |text: &SharedString, c: gpui::Hsla| {
        label_style(t, div())
            .absolute()
            .px(px(t.spacing.xs))
            .bg(hsla(t.palette.surface).opacity(0.85))
            .text_color(c)
            .child(text.clone())
    };
    for a in anns {
        match a.kind {
            Kind::HLine(v) => {
                let Some(f) = inside(norm(v, y.0, y.1)) else {
                    continue;
                };
                let c = hsla(a.color.unwrap_or(t.palette.text_muted));
                out = out.child(
                    div()
                        .absolute()
                        .left_0()
                        .w_full()
                        .top(relative(1.0 - f))
                        .border_t(px(t.sizes().hairline.max(1.0)))
                        .border_dashed()
                        .border_color(c),
                );
                if let Some(l) = &a.label {
                    out = out.child(tag(l, c).right_0().top(relative(1.0 - f)).mt(px(-14.0)));
                }
            }
            Kind::VLine(v) => {
                let Some(f) = inside(x(v)) else { continue };
                let c = hsla(a.color.unwrap_or(t.palette.text_muted));
                out = out.child(
                    div()
                        .absolute()
                        .top_0()
                        .h_full()
                        .left(relative(f))
                        .border_l(px(t.sizes().hairline.max(1.0)))
                        .border_dashed()
                        .border_color(c),
                );
                if let Some(l) = &a.label {
                    out = out.child(tag(l, c).top_0().left(relative(f)));
                }
            }
            Kind::YBand(lo, hi) => {
                let (Some(a0), Some(a1)) = (
                    inside(norm(lo, y.0, y.1).clamp(0.0, 1.0)),
                    inside(norm(hi, y.0, y.1).clamp(0.0, 1.0)),
                ) else {
                    continue;
                };
                if a1 <= a0 {
                    continue;
                }
                let c = hsla(a.color.unwrap_or(t.palette.primary));
                out = out.child(
                    div()
                        .absolute()
                        .left_0()
                        .w_full()
                        .top(relative(1.0 - a1))
                        .h(relative(a1 - a0))
                        .bg(scale_alpha(c, 0.14)),
                );
                if let Some(l) = &a.label {
                    out = out.child(tag(l, c).right_0().top(relative(1.0 - a1)));
                }
            }
            Kind::XBand(from, to) => {
                let (f0, f1) = (x(from).clamp(0.0, 1.0), x(to).clamp(0.0, 1.0));
                if !(f0.is_finite() && f1.is_finite()) || f1 <= f0 {
                    continue;
                }
                let c = hsla(a.color.unwrap_or(t.palette.primary));
                out = out.child(
                    div()
                        .absolute()
                        .top_0()
                        .h_full()
                        .left(relative(f0))
                        .w(relative(f1 - f0))
                        .bg(scale_alpha(c, 0.14)),
                );
                if let Some(l) = &a.label {
                    out = out.child(tag(l, c).top_0().left(relative(f0)));
                }
            }
        }
    }
    out
}

/// Category-centre mapping for `n` categories: index `i` → centre of slot `i`.
pub(crate) fn category_x(n: usize) -> impl Fn(f32) -> f32 {
    let n = n.max(1) as f32;
    move |i| (i + 0.5) / n
}
