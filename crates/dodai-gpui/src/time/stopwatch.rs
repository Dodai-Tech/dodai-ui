//! [`Stopwatch`]: chronometer with laps.

use super::{helpers::*, id2};
use crate::button::Button;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::ButtonVariant;
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
};
use std::time::{Duration, Instant};

// ───────────────────────────── stopwatch ─────────────────────────────

struct SwState {
    acc: Duration,
    started: Option<Instant>,
    laps: Vec<u64>,
}

impl SwState {
    fn ms(&self) -> u64 {
        let live = self.started.map(|s| s.elapsed()).unwrap_or_default();
        (self.acc + live).as_millis() as u64
    }
}

/// Chronometer: start/stop, lap, reset; shows the newest laps with splits.
#[derive(IntoElement)]
pub struct Stopwatch {
    id: ElementId,
    max_laps: usize,
}

impl Stopwatch {
    /// Create a new [`Stopwatch`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            max_laps: 5,
        }
    }
    /// How many of the newest laps to list (default 5).
    pub fn max_laps(mut self, n: usize) -> Self {
        self.max_laps = n;
        self
    }
}

impl RenderOnce for Stopwatch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| SwState {
            acc: Duration::ZERO,
            started: None,
            laps: vec![],
        });
        let (ms, running, laps) = {
            let s = st.read(cx);
            (s.ms(), s.started.is_some(), s.laps.clone())
        };
        if running {
            crate::motion::next_frame(window);
        }
        let id = self.id.clone();
        let (s1, s2, s3) = (st.clone(), st.clone(), st.clone());
        let go = Button::new(id2(&id, "go"), if running { "Stop" } else { "Start" })
            .variant(if running {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            })
            .on_click(move |_, _, cx| {
                s1.update(cx, |s, cx| {
                    if let Some(at) = s.started.take() {
                        s.acc += at.elapsed();
                    } else {
                        s.started = Some(Instant::now());
                    }
                    cx.notify();
                })
            });
        let lap = Button::new(id2(&id, "lap"), "Lap")
            .variant(ButtonVariant::Outline)
            .disabled(!running)
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    let ms = s.ms();
                    s.laps.push(ms);
                    cx.notify();
                })
            });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .disabled(running || (ms == 0 && laps.is_empty()))
            .on_click(move |_, _, cx| {
                s3.update(cx, |s, cx| {
                    s.acc = Duration::ZERO;
                    s.started = None;
                    s.laps.clear();
                    cx.notify();
                })
            });
        let splits = lap_splits(&laps);
        let mut list = div().flex().flex_col().gap(px(t.spacing.half())).w_full();
        for i in (0..laps.len()).rev().take(self.max_laps) {
            list = list.child(
                div()
                    .flex()
                    .justify_between()
                    .gap(px(t.spacing.lg))
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(SharedString::from(format!("LAP {}", i + 1)))
                    .child(SharedString::from(format!(
                        "+{}",
                        stopwatch_text(splits[i])
                    )))
                    .child(SharedString::from(stopwatch_text(laps[i]))),
            );
        }
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_xxl * 1.4))
                    .text_color(hsla(t.palette.text))
                    .child(stopwatch_text(ms)),
            )
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(go)
                    .child(lap)
                    .child(reset),
            )
            .child(list)
    }
}
