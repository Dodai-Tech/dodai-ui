//! Feedback: [`Alert`] (also banner style), [`Toast`], [`Spinner`],
//! [`IndeterminateBar`], [`Skeleton`].

use crate::motion::{Easing, Motion, MotionExt};
use crate::paint::{scaled_shape, shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{Theme, Tone};
use gpui::{
    div, px, relative, Animation, AnimationExt, App, ElementId, IntoElement, ParentElement,
    RenderOnce, SharedString, Styled, Window,
};
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
    ShapeStyle::from_surface(theme, theme.alert(tone, banner))
}

/// Inline status message with title + optional body. `banner()` = full solid strip.
#[derive(IntoElement)]
pub struct Alert {
    tone: Tone,
    title: SharedString,
    message: Option<SharedString>,
    banner: bool,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Alert);

impl Alert {
    /// Create a new [`Alert`].
    pub fn new(tone: Tone, title: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            tone,
            title: title.into(),
            message: None,
            banner: false,
        }
    }
    /// Message text.
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

impl Alert {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
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
                    .w(px(t.sizes().icon_sm + t.spacing.half()))
                    .text_color(accent)
                    .child(crate::surfaces::glyph_el(
                        tone_glyph(self.tone).into(),
                        t.sizes().icon_sm,
                        accent,
                    )),
            )
            .child(body)
    }
}

crate::entry_render!(Alert);

/// Compact floating notification: raised surface, tone bar at the left, glow from tokens.
#[derive(IntoElement)]
pub struct Toast {
    tone: Tone,
    message: SharedString,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Toast);

impl Toast {
    /// Create a new [`Toast`].
    pub fn new(tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            tone,
            message: message.into(),
        }
    }
}

impl RenderOnce for Toast {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::appear(self.build(cx), "Toast", motion)
    }
}

impl Toast {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
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
            .child(div().text_color(hsla(c)).child(crate::surfaces::glyph_el(
                tone_glyph(self.tone).into(),
                t.sizes().icon_sm,
                hsla(c),
            )))
            .child(self.message)
    }
}

pub use crate::paint::Playback;

/// Animation for a loading indicator: the override, or a linear cycle of
/// `default_ms`, played per [`Playback`].
fn cycle(m: Option<Motion>, default_ms: u64, pb: Playback) -> Animation {
    let m = m.unwrap_or_else(|| Motion::new(Duration::from_millis(default_ms), Easing::Linear));
    match pb {
        Playback::Loop => m.looping(),
        Playback::Once => m.animation(),
        Playback::Stopped => Animation::new(Duration::from_millis(1)).with_easing(|_| 0.35),
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
    motion: Option<Motion>,
    playback: Playback,
    corners: Option<dodai_core::Corners>,
    style: Option<dodai_core::SpinnerStyle>,
    size: f32,
}

crate::corners_setter!(Spinner);

impl Spinner {
    /// Create a new [`Spinner`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            motion: None,
            playback: Playback::Loop,
            style: None,
            size: 24.0,
        }
    }
    /// Draw this spinner instead of the design's own (to show them side by side).
    pub fn style(mut self, s: dodai_core::SpinnerStyle) -> Self {
        self.style = Some(s);
        self
    }
    /// Side of the spinner in px (default 24).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
    /// Override cycle duration and easing (default 900 ms, linear).
    pub fn motion(mut self, m: Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// Play once, loop (default) or hold still.
    pub fn playback(mut self, p: Playback) -> Self {
        self.playback = p;
        self
    }
}

impl RenderOnce for Spinner {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let c = hsla(t.palette.primary);
        let spin = self.style.unwrap_or(t.look().spinner);
        let cycle = cycle(self.motion, 900, self.playback);
        if spin == dodai_core::SpinnerStyle::Ring {
            return self
                .render_bars(c, t.shape.clone(), cycle)
                .into_any_element();
        }
        let size = self.size;
        if dodai_core::busy::arc_span(spin, 0.0).is_some() {
            return div()
                .relative()
                .w(px(size))
                .h(px(size))
                .with_animation(self.id, cycle, move |el, delta| {
                    let secs = delta / dodai_core::busy::SPIN_RATE;
                    let (start, len, track) =
                        dodai_core::busy::arc_span(spin, secs).unwrap_or_default();
                    el.child(
                        gpui::canvas(
                            |_, _, _| (),
                            move |b, _, window, _| paint_arc(window, b, c, start, len, track),
                        )
                        .absolute()
                        .size_full(),
                    )
                })
                .into_any_element();
        }
        let cells = dodai_core::busy::cells(spin, size);
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .with_animation(self.id, cycle, move |el, delta| {
                let secs = delta / dodai_core::busy::SPIN_RATE;
                let mut el = el;
                for cell in &cells {
                    let (x, y, w, h, a) = dodai_core::busy::pose(spin, cell, size, secs);
                    let mut d = div()
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .w(px(w))
                        .h(px(h))
                        .bg(gpui::Hsla { a, ..c });
                    if cell.round {
                        d = d.rounded_full();
                    } else if cell.radius > 0.0 {
                        d = d.rounded(px(cell.radius));
                    }
                    el = el.child(d);
                }
                el
            })
            .into_any_element()
    }
}

/// Stroke a ring arc `start..start+len` (turns clockwise from the top) with round ends, over
/// an optional faint full track.
fn paint_arc(
    window: &mut Window,
    b: gpui::Bounds<gpui::Pixels>,
    c: gpui::Hsla,
    start: f32,
    len: f32,
    track: f32,
) {
    use std::f32::consts::TAU;
    let size: f32 = b.size.width.into();
    let w = size * 0.1;
    let r = (size - w) / 2.0;
    let mid = gpui::point(
        b.origin.x + b.size.width / 2.0,
        b.origin.y + b.size.height / 2.0,
    );
    let at = |turn: f32| {
        let a = turn * TAU;
        gpui::point(mid.x + px(r * a.sin()), mid.y - px(r * a.cos()))
    };
    let mut stroke = |from: f32, span: f32, col: gpui::Hsla, caps: bool| {
        let steps = ((span * 160.0).ceil() as usize).max(2);
        let mut pb = gpui::PathBuilder::stroke(px(w));
        for i in 0..=steps {
            let p = at(from + span * i as f32 / steps as f32);
            if i == 0 {
                pb.move_to(p);
            } else {
                pb.line_to(p);
            }
        }
        if let Ok(p) = pb.build() {
            window.paint_path(p, col);
        }
        if caps {
            for turn in [from, from + span] {
                let p = at(turn);
                let q = gpui::Bounds::new(
                    gpui::point(p.x - px(w / 2.0), p.y - px(w / 2.0)),
                    gpui::size(px(w), px(w)),
                );
                window.paint_quad(gpui::fill(q, col).corner_radii(px(w / 2.0)));
            }
        }
    };
    if track > 0.0 {
        stroke(0.0, 1.0, gpui::Hsla { a: track, ..c }, false);
    }
    stroke(start, len, c, true);
}

impl Spinner {
    fn render_bars(
        self,
        c: gpui::Hsla,
        shape: dodai_core::Shape,
        cycle: Animation,
    ) -> impl IntoElement {
        div()
            .flex()
            .items_end()
            .gap(px(3.0))
            .h(px(20.0))
            .with_animation(self.id, cycle, move |el, delta| {
                let mut el = el;
                for i in 0..SPINNER_BARS {
                    let a = spinner_alpha(i, delta);
                    let style = ShapeStyle {
                        fill: Some(gpui::Hsla { a, ..c }),
                        stroke: None,
                        stroke_width: 0.0,
                        glow: None,
                        shadow: None,
                        shadow_dir: (1.0, 1.0),
                        scanlines: false,
                        scanline: c,
                        scanline_pitch: dodai_core::Effects::SCANLINE_PITCH,
                        surface: None,
                        shimmer: None,
                        elevation: None,
                        lift_layers: None,
                        edge_gap: None,
                        frost: None,
                    };
                    el = el.child(
                        shaped(&scaled_shape(&shape, 4.0), style)
                            .w(px(4.0))
                            .h(px(8.0 + 12.0 * a)),
                    );
                }
                el
            })
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
    motion: Option<Motion>,
    playback: Playback,
    corners: Option<dodai_core::Corners>,
    style: Option<dodai_core::ProgressStyle>,
    wavy: bool,
}

crate::corners_setter!(IndeterminateBar);

impl IndeterminateBar {
    /// Create a new [`IndeterminateBar`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            motion: None,
            playback: Playback::Loop,
            style: None,
            wavy: false,
        }
    }
    /// Draw the bar the way a design does (see [`dodai_core::ProgressStyle`]).
    pub fn style(mut self, s: dodai_core::ProgressStyle) -> Self {
        self.style = Some(s);
        self
    }
    /// Material 3 Expressive wavy indeterminate bar. Ignored by other design systems.
    pub fn wavy(mut self, on: bool) -> Self {
        self.wavy = on;
        self
    }
    /// Override cycle duration and easing (default 1400 ms, linear).
    pub fn motion(mut self, m: Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// Play once, loop (default) or hold still.
    pub fn playback(mut self, p: Playback) -> Self {
        self.playback = p;
        self
    }
}

impl RenderOnce for IndeterminateBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        if let Some(st) = self.style {
            return crate::controls::styled_bar(&t, st, None, None).into_any_element();
        }
        if self.wavy && t.look().tonal {
            return crate::controls::wavy_indeterminate(&t, None).into_any_element();
        }
        let k = t.progress(None);
        let h = t.spacing.sm + 2.0;
        let shape = crate::paint::round_shape(&t.shape, h);
        let track = ShapeStyle::from_tokens(&t, Some(k.track), Some(k.border), false, false);
        let fill = ShapeStyle::from_tokens(&t, Some(k.fill), None, true, false);
        shaped(&shape, track)
            .w_full()
            .h(px(h))
            .with_animation(
                self.id,
                cycle(self.motion, 1400, self.playback),
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
            .into_any_element()
    }
}

/// Pulsing placeholder block for loading content.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    w: Option<f32>,
    h: f32,
    circle: bool,
    motion: Option<Motion>,
    playback: Playback,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Skeleton);

impl Skeleton {
    /// Create a new [`Skeleton`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            w: None,
            h: 14.0,
            circle: false,
            motion: None,
            playback: Playback::Loop,
        }
    }
    /// Override cycle duration and easing (default 1600 ms, linear).
    pub fn motion(mut self, m: Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// Play once, loop (default) or hold still.
    pub fn playback(mut self, p: Playback) -> Self {
        self.playback = p;
        self
    }
    /// Fixed width in px (default: fill the parent).
    pub fn width(mut self, w: f32) -> Self {
        self.w = Some(w);
        self
    }
    /// Height in px.
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
        let ms = self.motion.map_or(1600, |m| m.duration.as_millis() as u32);
        let style = style.with_shimmer(crate::paint::Shimmer {
            color: hsla(t.palette.text).opacity(0.14),
            ms,
            playback: self.playback,
        });
        let d = crate::paint::shaped_id(self.id, &shape, style).h(px(self.h));
        match self.w {
            Some(w) => d.w(px(w)).flex_none(),
            None => d.w_full(),
        }
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
