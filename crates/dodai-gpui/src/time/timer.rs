//! [`Timer`]: countdown.

use super::{helpers::*, id2, VoidCb};
use crate::button::Button;
use crate::display::CircularProgress;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, Tone};
use gpui::{div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window};
use std::time::{Duration, Instant};

// ───────────────────────────── timer ─────────────────────────────

struct TimerState {
    total_ms: u64,
    elapsed: Duration,
    started: Option<Instant>,
    fired: bool,
}

impl TimerState {
    fn elapsed_ms(&self) -> u64 {
        let live = self.started.map(|s| s.elapsed()).unwrap_or_default();
        ((self.elapsed + live).as_millis() as u64).min(self.total_ms)
    }
}

/// Countdown timer with a progress ring, start/pause/reset and ±1 min.
///
/// ```ignore
/// Timer::new("t").minutes(5).on_finish(|_, _| println!("ding"))
/// ```
#[derive(IntoElement)]
pub struct Timer {
    id: ElementId,
    total_ms: u64,
    size: f32,
    on_finish: Option<VoidCb>,
}

impl Timer {
    /// Create a new [`Timer`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            total_ms: 60_000,
            size: 160.0,
            on_finish: None,
        }
    }
    /// Duration in seconds.
    pub fn seconds(mut self, s: u64) -> Self {
        self.total_ms = s * 1000;
        self
    }
    /// Duration in minutes.
    pub fn minutes(self, m: u64) -> Self {
        self.seconds(m * 60)
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    crate::callback!(
        /// Called once when the countdown reaches zero.
        on_finish
    );
}

impl RenderOnce for Timer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let total = self.total_ms;
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| TimerState {
            total_ms: total,
            elapsed: Duration::ZERO,
            started: None,
            fired: false,
        });
        let (ms, total, running, done) = {
            let s = st.read(cx);
            (s.elapsed_ms(), s.total_ms, s.started.is_some(), s.fired)
        };
        if running && ms >= total && !done {
            st.update(cx, |s, _| {
                s.elapsed = Duration::from_millis(s.total_ms);
                s.started = None;
                s.fired = true;
            });
            if let Some(f) = &self.on_finish {
                f(window, cx);
            }
        }
        let (running, done) = (st.read(cx).started.is_some(), st.read(cx).fired);
        if running {
            crate::motion::next_frame(window);
        }
        let left = total.saturating_sub(ms);
        let frac = if total == 0 {
            0.0
        } else {
            left as f32 / total as f32
        };
        let tone = if done || frac < 0.1 {
            Some(Tone::Danger)
        } else if running {
            None
        } else {
            Some(Tone::Info)
        };
        let ring = div()
            .relative()
            .flex_none()
            .w(px(self.size))
            .h(px(self.size))
            .flex()
            .items_center()
            .justify_center()
            .child({
                let mut p = CircularProgress::new(frac)
                    .size(self.size)
                    .thickness(7.0)
                    .label(false);
                if let Some(k) = tone {
                    p = p.tone(k);
                }
                p
            })
            .child(
                div()
                    .absolute()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(self.size * 0.2))
                    .text_color(hsla(if done {
                        t.palette.danger
                    } else {
                        t.palette.text
                    }))
                    .child(countdown_text(left)),
            );
        let id = self.id.clone();
        let (s1, s2, s3, s4) = (st.clone(), st.clone(), st.clone(), st.clone());
        let go = Button::new(
            id2(&id, "go"),
            if running {
                "Pause"
            } else if done {
                "Done"
            } else {
                "Start"
            },
        )
        .variant(if running {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Primary
        })
        .disabled(done)
        .on_click(move |_, _, cx| {
            s1.update(cx, |s, cx| {
                if let Some(at) = s.started.take() {
                    s.elapsed += at.elapsed();
                } else if s.elapsed_ms() < s.total_ms {
                    s.started = Some(Instant::now());
                }
                cx.notify();
            })
        });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    s.elapsed = Duration::ZERO;
                    s.started = None;
                    s.fired = false;
                    cx.notify();
                })
            });
        let adj = |label: &'static str, key: &'static str, d: i64, s: gpui::Entity<TimerState>| {
            Button::new(id2(&id, key), label)
                .variant(ButtonVariant::Outline)
                .disabled(running)
                .on_click(move |_, _, cx| {
                    s.update(cx, |s, cx| {
                        s.total_ms =
                            (s.total_ms as i64 + d * 60_000).clamp(60_000, 99 * 60_000) as u64;
                        s.elapsed = Duration::ZERO;
                        s.fired = false;
                        cx.notify();
                    })
                })
        };
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(ring)
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(adj("−1 min", "minus", -1, s3))
                    .child(go)
                    .child(reset)
                    .child(adj("+1 min", "plus", 1, s4)),
            )
    }
}
