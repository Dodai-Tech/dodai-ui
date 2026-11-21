//! Foundation tokens as xilem values: every palette colour, the type scale with font stacks,
//! and spacing and shape metrics. Components and apps read these instead of raw numbers.

use dodai_core::{Spacing, Theme, Typography};
use xilem::style::Style;
use xilem::view::label;
use xilem::{Color, WidgetView};

use crate::color;

/// Every [`dodai_core::Palette`] colour as a vello colour.
#[derive(Debug, Clone, Copy)]
pub struct Colors {
    pub background: Color,
    pub surface: Color,
    pub surface_raised: Color,
    pub overlay: Color,
    pub border: Color,
    pub border_strong: Color,
    pub text: Color,
    pub text_muted: Color,
    pub text_disabled: Color,
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,
    pub on_primary: Color,
    pub secondary: Color,
    pub secondary_hover: Color,
    pub on_secondary: Color,
    pub accent: Color,
    pub info: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
    pub selection: Color,
    pub focus_ring: Color,
    pub link: Color,
    /// Chart series colours.
    pub series: [Color; 8],
}

impl Colors {
    /// The colours of `t`.
    pub fn of(t: &Theme) -> Self {
        let p = &t.palette;
        Self {
            background: color(p.background),
            surface: color(p.surface),
            surface_raised: color(p.surface_raised),
            overlay: color(p.overlay),
            border: color(p.border),
            border_strong: color(p.border_strong),
            text: color(p.text),
            text_muted: color(p.text_muted),
            text_disabled: color(p.text_disabled),
            primary: color(p.primary),
            primary_hover: color(p.primary_hover),
            primary_active: color(p.primary_active),
            on_primary: color(p.on_primary),
            secondary: color(p.secondary),
            secondary_hover: color(p.secondary_hover),
            on_secondary: color(p.on_secondary),
            accent: color(p.accent),
            info: color(p.info),
            success: color(p.success),
            warning: color(p.warning),
            danger: color(p.danger),
            selection: color(p.selection),
            focus_ring: color(p.focus_ring),
            link: color(p.link),
            series: p.series.map(color),
        }
    }
}

/// A step of the type scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Size {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
}

/// Which font family of the design a text uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Face {
    Display,
    Body,
    Mono,
}

/// The type scale and families of a theme.
#[derive(Debug, Clone)]
pub struct Type {
    typography: Typography,
}

impl Type {
    /// The typography of `t`.
    pub fn of(t: &Theme) -> Self {
        Self {
            typography: t.typography.clone(),
        }
    }

    /// Size in px of a scale step.
    pub fn size(&self, s: Size) -> f32 {
        let t = &self.typography;
        match s {
            Size::Xs => t.size_xs,
            Size::Sm => t.size_sm,
            Size::Md => t.size_md,
            Size::Lg => t.size_lg,
            Size::Xl => t.size_xl,
            Size::Xxl => t.size_xxl,
        }
    }

    /// The CSS-style family list of a face.
    pub fn family(&self, f: Face) -> &'static str {
        match f {
            Face::Display => self.typography.display,
            Face::Body => self.typography.body,
            Face::Mono => self.typography.mono,
        }
    }

    /// A label in `face` at `size`, in `colour`; capitals when the design sets labels that way
    /// and `face` is [`Face::Display`].
    pub fn text<State: 'static>(
        &self,
        text: impl Into<String>,
        face: Face,
        size: Size,
        colour: Color,
    ) -> impl WidgetView<State> {
        let text = text.into();
        let text = if face == Face::Display && self.typography.uppercase_labels {
            text.to_uppercase()
        } else {
            text
        };
        label(text)
            .font(self.family(face))
            .text_size(self.size(size))
            .color(colour)
    }
}

/// Spacing scale and shape metrics in px.
#[derive(Debug, Clone, PartialEq)]
pub struct Metrics {
    pub spacing: Spacing,
    pub radius: f64,
    pub border_width: f64,
    pub control_height: f64,
}

impl Metrics {
    /// The metrics of `t`.
    pub fn of(t: &Theme) -> Self {
        Self {
            spacing: t.spacing.clone(),
            radius: f64::from(t.shape.radius),
            border_width: f64::from(t.shape.border_width),
            control_height: f64::from(t.shape.control_height),
        }
    }
}

/// Colours, type and metrics of one theme, built once per frame.
#[derive(Debug, Clone)]
pub struct Foundation {
    pub colors: Colors,
    pub typography: Type,
    pub metrics: Metrics,
}

impl Foundation {
    /// The foundation of `t`.
    pub fn of(t: &Theme) -> Self {
        Self {
            colors: Colors::of(t),
            typography: Type::of(t),
            metrics: Metrics::of(t),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scale_grows_and_follows_the_theme() {
        let t = Theme::default();
        let ty = Type::of(&t);
        let steps = [Size::Xs, Size::Sm, Size::Md, Size::Lg, Size::Xl, Size::Xxl];
        assert!(steps.windows(2).all(|w| ty.size(w[0]) <= ty.size(w[1])));
        assert_eq!(ty.size(Size::Md), t.typography.size_md);
        assert_eq!(ty.family(Face::Mono), t.typography.mono);
    }

    #[test]
    fn colours_and_metrics_come_from_tokens() {
        let t = Theme::default();
        let f = Foundation::of(&t);
        assert_eq!(f.colors.primary, color(t.palette.primary));
        assert_eq!(f.colors.series[3], color(t.palette.series[3]));
        assert_eq!(f.metrics.radius, f64::from(t.shape.radius));
        let _ = f
            .typography
            .text::<()>("Hi", Face::Display, Size::Lg, f.colors.text);
    }
}
