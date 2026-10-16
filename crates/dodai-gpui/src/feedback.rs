//! Feedback: [`Alert`] (also banner style), [`Toast`], [`Spinner`],
//! [`IndeterminateBar`], [`Skeleton`].

use crate::paint::{scaled_shape, shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use gpui::{
    div, px, relative, Animation, AnimationExt, App, ElementId, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};
use dodai_core::{Theme, Tone};
use std::time::Duration;

/// Glyph for a tone.
pub fn tone_glyph(tone: Tone) -> &'static str {
    match tone {
        Tone::Info => "i",
        Tone::Success => "✓",
        Tone::Warning => "!",
        Tone::Danger => "✕",
    }
}

/// Alert paint: tinted outlined, or solid for banners. Returns (style, text colour). Pure.
pub fn alert_style(theme: &Theme, tone: Tone, banner: bool) -> (ShapeStyle, gpui::Hsla) {
    let c = theme.palette.tone(tone);
    if banner {
        (
            ShapeStyle::from_tokens(theme, Some(c), None, false, false),
            hsla(theme.on(c)),
        )
    } else {
        (
            ShapeStyle::from_tokens(theme, Some(c.with_alpha(0.12)), Some(c), false, false),
            hsla(theme.palette.text),
        )
    }
}

/// Inline status message with title + optional body. `banner()` = full solid strip.
#[derive(IntoElement)]
pub struct Alert {
    tone: Tone,
    title: SharedString,
    message: Option<SharedString>,
    banner: bool,
}

impl Alert {
    pub fn new(tone: Tone, title: impl Into<SharedString>) -> Self {
        Self {
            tone,
            title: title.into(),
            message: None,
            banner: false,
        }
    }
    pub fn message(mut self, m: impl Into<SharedString>) -> Self {
        self.message = Some(m.into());
        self
    }
    /// Solid, borderless banner style.
    pub fn banner(mut self, b: bool) -> Self {
        self.banner = b;
        self
    }
}

impl RenderOnce for Alert {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (style, fg) = alert_style(&t, self.tone, self.banner);
        let accent = if self.banner {
            fg
        } else {
            hsla(t.palette.tone(self.tone))
        };
        let mut body = div().flex().flex_col().flex_1().child(
            div()
                .font_family(first_family(t.typography.display))
                .child(format_label(&t.typography, &self.title)),
        );
        if let Some(m) = self.message {
            body = body.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .opacity(0.85)
                    .child(m),
            );
        }
        shaped(&t.shape, style)
            .w_full()
            .px(px(t.spacing.md))
            .py(px(t.spacing.sm + 2.0))
            .flex()
            .items_start()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg)
            .child(
                div()
                    .flex_none()
                    .w(px(20.0))
                    .text_color(accent)
                    .child(tone_glyph(self.tone)),
            )
            .child(body)
    }
}

/// Compact floating notification: raised surface, tone bar at the left, glow from tokens.
#[derive(IntoElement)]
pub struct Toast {
    tone: Tone,
    message: SharedString,
}

impl Toast {
    pub fn new(tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            tone,
            message: message.into(),
        }
    }
}

impl RenderOnce for Toast {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = t.palette.tone(self.tone);
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.surface_raised),
            Some(t.palette.border_strong),
            true,
            true,
        );
        shaped(&t.shape, style)
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .pr(px(t.spacing.md))
            .pl(px(t.spacing.md + 4.0))
            .h(px(t.shape.control_height))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .bottom_0()
                    .w(px(4.0))
                    .bg(hsla(c)),
            )
            .child(div().text_color(hsla(c)).child(tone_glyph(self.tone)))
            .child(self.message)
    }
}

/// Number of bars in a [`Spinner`].
pub const SPINNER_BARS: usize = 8;

/// Opacity of bar `i` for animation phase `delta` (0..1): a bright head chasing a fading tail.
pub fn spinner_alpha(i: usize, delta: f32) -> f32 {
    let head = (delta * SPINNER_BARS as f32) as usize % SPINNER_BARS;
    let behind = (head + SPINNER_BARS - i) % SPINNER_BARS;
    1.0 - behind as f32 / SPINNER_BARS as f32 * 0.85
}

/// Indeterminate spinner: a row of bars with a travelling highlight.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
}

impl Spinner {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for Spinner {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = hsla(t.palette.primary);
        let shape = t.shape.clone();
        let glow = t.effects.glow_enabled();
        div()
            .flex()
            .items_end()
            .gap(px(3.0))
            .h(px(20.0))
            .with_animation(
                self.id,
                Animation::new(Duration::from_millis(900)).repeat(),
                move |el, delta| {
                    let mut el = el;
                    for i in 0..SPINNER_BARS {
                        let a = spinner_alpha(i, delta);
                        let style = ShapeStyle {
                            fill: Some(gpui::Hsla { a, ..c }),
                            stroke: None,
                            stroke_width: 0.0,
                            glow: None,
                            shadow: None,
                            scanlines: false,
                        };
                        let _ = glow;
                        el = el.child(
                            shaped(&scaled_shape(&shape, 4.0), style)
                                .w(px(4.0))
                                .h(px(8.0 + 12.0 * a)),
                        );
                    }
                    el
                },
            )
    }
}

/// Visible `(left, width)` fractions of the sliding segment at phase `delta`.
pub fn indeterminate_segment(delta: f32) -> (f32, f32) {
    let w = 0.3;
    let x = -w + (1.0 + w) * delta.clamp(0.0, 1.0);
    let l = x.max(0.0);
    let r = (x + w).min(1.0);
    (l, (r - l).max(0.0))
}

/// Indeterminate progress bar: a segment sliding across the track.
#[derive(IntoElement)]
pub struct IndeterminateBar {
    id: ElementId,
}

impl IndeterminateBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for IndeterminateBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let k = t.progress(None);
        let h = t.spacing.sm + 2.0;
        let shape = scaled_shape(&t.shape, h);
        let track = ShapeStyle::from_tokens(&t, Some(k.track), Some(k.border), false, false);
        let fill = ShapeStyle::from_tokens(&t, Some(k.fill), None, true, false);
        shaped(&shape, track).w_full().h(px(h)).with_animation(
            self.id,
            Animation::new(Duration::from_millis(1400)).repeat(),
            move |el, delta| {
                let (l, w) = indeterminate_segment(delta);
                el.child(
                    shaped(&shape, fill)
                        .absolute()
                        .top_0()
                        .h_full()
                        .left(relative(l))
                        .w(relative(w)),
                )
            },
        )
    }
}

/// Pulsing placeholder block for loading content.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    w: Option<f32>,
    h: f32,
    circle: bool,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            w: None,
            h: 14.0,
            circle: false,
        }
    }
    /// Fixed width in px (default: fill the parent).
    pub fn width(mut self, w: f32) -> Self {
        self.w = Some(w);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.h = h;
        self
    }
    /// Avatar-like block (round on rounded themes).
    pub fn circle(mut self, c: bool) -> Self {
        self.circle = c;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let shape = if self.circle {
            crate::paint::round_shape(&t.shape, self.h)
        } else {
            scaled_shape(&t.shape, self.h)
        };
        let style = ShapeStyle::from_tokens(&t, Some(t.palette.surface_raised), None, false, false);
        let mut d = shaped(&shape, style).h(px(self.h));
        d = match self.w {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.w_full(),
        };
        d.with_animation(
            self.id,
            Animation::new(Duration::from_millis(1600)).repeat(),
            |el, delta| el.opacity(0.45 + 0.55 * (delta * std::f32::consts::TAU).sin().abs()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn alert_tokens() {
        let t = light_rounded();
        let (s, fg) = alert_style(&t, Tone::Danger, false);
        assert_eq!(s.stroke, Some(hsla(t.palette.danger)));
        assert_eq!(fg, hsla(t.palette.text));
        let q = square();
        let (s, fg) = alert_style(&q, Tone::Success, true);
        assert_eq!(s.fill, Some(hsla(q.palette.success)));
        assert!(s.stroke.is_none());
        assert_eq!(fg, hsla(q.on(q.palette.success)));
    }

    #[test]
    fn spinner_head_brightest() {
        for step in 0..SPINNER_BARS {
            let d = step as f32 / SPINNER_BARS as f32 + 0.01;
            let a: Vec<f32> = (0..SPINNER_BARS).map(|i| spinner_alpha(i, d)).collect();
            let max = a.iter().cloned().fold(0.0, f32::max);
            assert_eq!(a[step], max);
            assert!(a.iter().all(|x| *x > 0.0 && *x <= 1.0));
        }
    }

    #[test]
    fn segment_stays_in_track() {
        for i in 0..=100 {
            let (l, w) = indeterminate_segment(i as f32 / 100.0);
            assert!(l >= 0.0 && l + w <= 1.0 + 1e-6, "{l} {w}");
        }
        assert_eq!(indeterminate_segment(0.0).1, 0.0);
    }
}
