//! [`ProgressBar`] and the bar painters the slider shares.

use super::*;
use crate::paint::{shaped, ShapeStyle};
use crate::{hsla, theme_of};
use dodai_core::{Theme, Tone};
use gpui::{canvas, div, px, relative, App, IntoElement, ParentElement, Styled, Window};

pub(super) fn bar(t: &Theme, frac: f32, tone: Option<Tone>, height: f32) -> gpui::Div {
    let k = t.progress(tone);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, height);
    let mut d = shaped(&shape, track_style).w_full().h(px(height));
    let f = clamp01(frac);
    let n = k.segments;
    if f > 0.0 {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .top_0()
                .left_0()
                .h_full()
                .w(relative(f)),
        );
    }
    // HUD gauge: thin gaps in the track colour cut the bar into segments.
    for i in 1..n {
        d = d.child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .left(relative(i as f32 / n as f32))
                .w(px(3.0))
                .bg(hsla(k.track)),
        );
    }
    d
}

/// Track whose fill spans from the middle to `frac`.
pub(super) fn centered_bar(t: &Theme, frac: f32, height: f32) -> gpui::Div {
    let k = t.progress(None);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, height);
    let f = clamp01(frac);
    let (a, b) = (f.min(0.5), f.max(0.5));
    let mut d = shaped(&shape, track_style).w_full().h(px(height));
    if b > a {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .top_0()
                .h_full()
                .left(relative(a))
                .w(relative(b - a)),
        );
    }
    d
}

pub(super) fn vbar(t: &Theme, frac: f32, width: f32) -> gpui::Div {
    let k = t.progress(None);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, width);
    let mut d = shaped(&shape, track_style).h_full().w(px(width));
    let f = clamp01(frac);
    if f > 0.0 {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .bottom_0()
                .left_0()
                .w_full()
                .h(relative(f)),
        );
    }
    d
}

/// Horizontal progress bar (`0.0..=1.0`).
#[derive(IntoElement)]
pub struct ProgressBar {
    follow: crate::follow::Follow,
    value: f32,
    tone: Option<Tone>,
    wavy: bool,
    style: Option<dodai_core::ProgressStyle>,
}

impl ProgressBar {
    /// Create a new [`ProgressBar`]; a changed value glides.
    #[track_caller]
    pub fn new(value: f32) -> Self {
        Self {
            follow: crate::follow::Follow::here(),
            value,
            tone: None,
            wavy: false,
            style: None,
        }
    }
    crate::follow_api!();
    /// Colour by status tone (default: brand colour).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
    /// Draw the bar the way a design does (see [`dodai_core::ProgressStyle`]).
    pub fn style(mut self, s: dodai_core::ProgressStyle) -> Self {
        self.style = Some(s);
        self
    }
    /// Material 3 Expressive wavy indicator: the active part is a travelling sine wave.
    /// Other design systems draw their usual bar.
    pub fn wavy(mut self, on: bool) -> Self {
        self.wavy = on;
        self
    }
}

/// Wavy linear indicator: straight track from `frac` on, a travelling sine wave before it.
pub(super) fn wavy_bar(t: &Theme, frac: f32, tone: Option<Tone>) -> gpui::Div {
    use gpui::{point, PathBuilder};
    let k = t.progress(tone);
    let (track, fill) = (hsla(k.track), hsla(k.fill));
    let frac = clamp01(frac);
    div().w_full().h(px(t.sizes().chart_cell)).relative().child(
        canvas(
            |_, _, _| (),
            move |b, _, window, _| {
                let w: f32 = b.size.width.into();
                let (ox, cy) = (b.origin.x, b.origin.y + b.size.height / 2.0);
                let stroke = 4.0;
                let active = (w * frac).max(0.0);
                // Straight remainder of the track, after a small gap.
                let from = (active + stroke * 2.0).min(w);
                if from < w {
                    let mut pb = PathBuilder::stroke(px(stroke));
                    pb.move_to(point(ox + px(from), cy));
                    pb.line_to(point(ox + px(w), cy));
                    if let Ok(p) = pb.build() {
                        window.paint_path(p, track);
                    }
                }
                if active > 0.5 {
                    let (amp, len) = (3.0_f32, 36.0_f32);
                    let phase = crate::display::elapsed_secs() * 4.0;
                    let mut pb = PathBuilder::stroke(px(stroke));
                    let steps = (active / 2.0).ceil().max(2.0) as usize;
                    for i in 0..=steps {
                        let x = active * i as f32 / steps as f32;
                        let y = amp * (x / len * std::f32::consts::TAU - phase).sin();
                        let p = point(ox + px(x), cy + px(y));
                        if i == 0 {
                            pb.move_to(p);
                        } else {
                            pb.line_to(p);
                        }
                    }
                    if let Ok(p) = pb.build() {
                        window.paint_path(p, fill);
                    }
                    crate::motion::next_frame(window);
                }
            },
        )
        .absolute()
        .size_full(),
    )
}

impl ProgressBar {
    fn build(self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "ProgressBar");
        let value = crate::follow::follow(window, cx, &self.follow, 0, self.value);
        if let Some(st) = self.style {
            return styled_bar(&t, st, Some(value * g), self.tone).into_any_element();
        }
        if self.wavy && t.look().tonal {
            return wavy_bar(&t, value * g, self.tone).into_any_element();
        }
        bar(&t, value * g, self.tone, t.sizes().bar_thickness).into_any_element()
    }
}

crate::entry_render!(ProgressBar);
