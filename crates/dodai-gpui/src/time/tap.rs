//! [`TapTempo`]: measure BPM by tapping, with a visual metronome.

use super::{helpers::*, id2, Cb};
use crate::button::Button;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::ButtonVariant;
use gpui::{div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window};
use std::time::Instant;

// ───────────────────────────── tap tempo ─────────────────────────────

struct TapState {
    t0: Instant,
    taps: Vec<f64>,
    bpm: Option<f32>,
    playing: Option<Instant>,
}

/// Measure the tempo of music: tap along and read the BPM, then press Play to
/// hear/see it back as a visual metronome (a bar of beat lights, beat 1 accented).
#[derive(IntoElement)]
pub struct TapTempo {
    id: ElementId,
    beats: usize,
    on_change: Option<Cb<f32>>,
}

impl TapTempo {
    /// Create a new [`TapTempo`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            beats: 4,
            on_change: None,
        }
    }
    /// Beats per bar in the metronome lights (default 4).
    pub fn beats(mut self, n: usize) -> Self {
        self.beats = n.clamp(1, 12);
        self
    }
    crate::callback!(
        /// Called with the new BPM after a tap or a ± adjustment.
        on_change, f32
    );
}

impl RenderOnce for TapTempo {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| TapState {
            t0: Instant::now(),
            taps: vec![],
            bpm: None,
            playing: None,
        });
        let (bpm, taps, playing) = {
            let s = st.read(cx);
            (s.bpm, s.taps.clone(), s.playing)
        };
        let id = self.id.clone();
        let on_change = self.on_change.clone();
        let (s1, s2, s3, s4, s5) = (st.clone(), st.clone(), st.clone(), st.clone(), st.clone());

        let nudge = |label: &'static str, key: &'static str, d: f32, s: gpui::Entity<TapState>| {
            let f = on_change.clone();
            Button::new(id2(&id, key), label)
                .variant(ButtonVariant::Outline)
                .disabled(bpm.is_none())
                .on_click(move |_, w, cx| {
                    let nb = s.update(cx, |s, cx| {
                        let b = (s.bpm.unwrap_or(120.0) + d).clamp(20.0, 300.0);
                        s.bpm = Some(b);
                        cx.notify();
                        b
                    });
                    if let Some(f) = &f {
                        f(nb, w, cx);
                    }
                })
        };
        let f = on_change.clone();
        let tap = Button::new(id2(&id, "tap"), "Tap")
            .variant(ButtonVariant::Primary)
            .refine(|d| d.w(px(120.0)).h(px(56.0)))
            .on_click(move |_, w, cx| {
                let nb = s1.update(cx, |s, cx| {
                    let now = s.t0.elapsed().as_secs_f64();
                    if s.taps.last().is_some_and(|l| now - l > 2.5) {
                        s.taps.clear();
                    }
                    s.taps.push(now);
                    if s.taps.len() > 12 {
                        s.taps.remove(0);
                    }
                    if let Some(b) = tap_bpm(&s.taps) {
                        s.bpm = Some(b.clamp(20.0, 300.0));
                    }
                    cx.notify();
                    s.bpm
                });
                if let (Some(b), Some(f)) = (nb, &f) {
                    f(b, w, cx);
                }
            });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .disabled(bpm.is_none())
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    s.taps.clear();
                    s.bpm = None;
                    s.playing = None;
                    cx.notify();
                })
            });
        let play = Button::new(
            id2(&id, "play"),
            if playing.is_some() { "Stop" } else { "Play" },
        )
        .variant(if playing.is_some() {
            ButtonVariant::Danger
        } else {
            ButtonVariant::Secondary
        })
        .disabled(bpm.is_none())
        .on_click(move |_, _, cx| {
            s3.update(cx, |s, cx| {
                s.playing = if s.playing.is_some() {
                    None
                } else {
                    Some(Instant::now())
                };
                cx.notify();
            })
        });

        // Beat lights.
        let (cur, frac) = match (playing, bpm) {
            (Some(at), Some(b)) => {
                crate::motion::next_frame(window);
                let (i, f) = beat_phase(at.elapsed().as_secs_f64(), b);
                (Some((i % self.beats as u64) as usize), f)
            }
            _ => (None, 1.0),
        };
        let mut lights = div().flex().gap(px(t.spacing.sm)).items_center();
        for i in 0..self.beats {
            let base = if i == 0 {
                t.palette.primary
            } else {
                t.palette.secondary
            };
            let lit = if cur == Some(i) {
                (1.0 - frac).powi(2)
            } else {
                0.0
            };
            let c = crate::motion::mix(hsla(t.palette.border), hsla(base), 0.15 + 0.85 * lit);
            lights = lights.child(div().w(px(22.0)).h(px(22.0)).rounded_full().bg(c));
        }

        let readout = match bpm {
            Some(b) => format!("{}", b.round() as u32),
            None => "—".into(),
        };
        let note = match (bpm, tap_jitter(&taps)) {
            (Some(b), Some(j)) => format!(
                "{} · {} taps · {}",
                tempo_name(b),
                taps.len(),
                if j < 0.04 {
                    "steady"
                } else if j < 0.1 {
                    "fairly steady"
                } else {
                    "uneven"
                }
            ),
            (Some(b), None) => format!("{} · {} taps", tempo_name(b), taps.len()),
            _ => "Tap along with the beat".into(),
        };
        let nb = nudge("−1", "m1", -1.0, s4.clone());
        let pb = nudge("+1", "p1", 1.0, s5.clone());
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(t.spacing.sm))
                    .child(
                        div()
                            .font_family(first_family(t.typography.display))
                            .text_size(px(t.typography.size_xxl * 1.8))
                            .text_color(hsla(t.palette.text))
                            .child(readout),
                    )
                    .child(
                        div()
                            .pb(px(8.0))
                            .text_size(px(t.typography.size_md))
                            .text_color(hsla(t.palette.text_muted))
                            .child("BPM"),
                    ),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(note),
            )
            .child(lights)
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .items_center()
                    .child(nb)
                    .child(tap)
                    .child(pb),
            )
            .child(div().flex().gap(px(t.spacing.sm)).child(play).child(reset))
    }
}
