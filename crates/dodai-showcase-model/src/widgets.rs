//! The live data-display widgets (circular progress, clock, timer, stopwatch, tap tempo) as
//! nodes: geometry is computed here, so a toolkit only strokes polylines and lays out text.
//! Sizes and colours follow the gpui components.

use std::f32::consts::{FRAC_PI_2, TAU};

use dodai_core::clock_math::{
    countdown_text, day_secs, hand_angles, hms, stopwatch_text, unix_now,
};
use dodai_core::{ButtonVariant as V, Color, Theme, Tone};

use crate::b::*;
use crate::node::{Font, Gap, Ink, Node};
use crate::plot::{Line, Plot};

/// Seconds since the showcase started (drives spinning and travelling waves).
#[cfg(target_arch = "wasm32")]
pub fn elapsed() -> f32 {
    // No `Instant` in a browser build: the host's clock, from the first call.
    static START: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    let now = dodai_core::clock_math::unix_now();
    (now - START.get_or_init(|| now)) as f32
}

/// Seconds since the showcase started (drives spinning and travelling waves).
#[cfg(not(target_arch = "wasm32"))]
pub fn elapsed() -> f32 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f32()
}

/// Points of an arc from `start` rad sweeping `sweep` rad, radius `r` around `c`.
pub fn arc_points(c: (f32, f32), r: f32, start: f32, sweep: f32) -> Vec<(f32, f32)> {
    let n = ((sweep.abs() / TAU) * 96.0).ceil().max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = start + sweep * i as f32 / n as f32;
            (c.0 + r * a.cos(), c.1 + r * a.sin())
        })
        .collect()
}

/// [`arc_points`] whose radius swings by 1.2 px with seven crests a turn (the wavy ring);
/// `phase` travels the wave.
pub fn wavy_arc_points(
    c: (f32, f32),
    r: f32,
    start: f32,
    sweep: f32,
    phase: f32,
) -> Vec<(f32, f32)> {
    let (amp, waves) = (1.2, 7.0);
    let n = ((sweep.abs() / TAU) * 192.0).ceil().max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = start + sweep * i as f32 / n as f32;
            let rr = r + amp * (a * waves - phase).sin();
            (c.0 + rr * a.cos(), c.1 + rr * a.sin())
        })
        .collect()
}

/// A circular progress ring as in the gpui `CircularProgress`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ring {
    /// Fraction `0..=1`, or spinning when `None`.
    pub value: Option<f32>,
    /// Status tone (`None` = primary).
    pub tone: Option<Tone>,
    /// Diameter px (`None` = the design's: 48 on tonal designs, else 56).
    pub size: Option<f32>,
    /// Stroke px (`None` = 4 on tonal designs, else 5).
    pub thickness: Option<f32>,
    /// Wavy track (tonal designs only).
    pub wavy: bool,
    /// The percentage in the middle.
    pub label: bool,
}

impl Ring {
    /// A determinate ring at `v`.
    pub fn new(v: f32) -> Self {
        Self {
            value: Some(v.clamp(0.0, 1.0)),
            tone: None,
            size: None,
            thickness: None,
            wavy: false,
            label: true,
        }
    }

    /// A spinning ring.
    pub fn spinning() -> Self {
        Self {
            value: None,
            label: false,
            ..Self::new(0.0)
        }
    }

    /// The ring as a node (`now` = seconds, for the spin and the travelling wave).
    pub fn node(self, t: &Theme, now: f32) -> Node {
        let k = t.progress(self.tone);
        let m3 = t.look().tonal;
        let (size, th) = if m3 { (48.0, 4.0) } else { (56.0, 5.0) };
        let size = self.size.unwrap_or(size);
        let th = self.thickness.unwrap_or(th);
        let wavy = self.wavy && m3;
        let c = (size / 2.0, size / 2.0);
        let r = size / 2.0 - th / 2.0;
        let line = |color: Color, points| Line {
            color,
            width: th,
            points,
        };
        let mut lines = Vec::new();
        let start = -FRAC_PI_2;
        match self.value {
            Some(v) if m3 && v > 0.0 && v < 1.0 => {
                // The track resumes one stroke-width past the active arc's end.
                let gap = (th * 1.5 / r).min(0.5);
                let a = TAU * v;
                lines.push(line(
                    k.track,
                    arc_points(c, r, start + a + gap, (TAU - a - 2.0 * gap).max(0.0)),
                ));
            }
            _ => lines.push(line(k.track, arc_points(c, r, 0.0, TAU))),
        }
        match self.value {
            Some(v) if v > 0.0 => {
                let sweep = TAU * v.min(0.9999);
                let pts = if wavy {
                    wavy_arc_points(c, r - 1.5, start, sweep, now * 3.0)
                } else {
                    arc_points(c, r, start, sweep)
                };
                lines.push(line(k.fill, pts));
            }
            Some(_) => {}
            None => {
                let a = now * 4.0;
                let pts = if wavy {
                    wavy_arc_points(c, r - 1.5, a, TAU * 0.4, a * 2.0)
                } else {
                    arc_points(c, r, a, TAU * 0.27)
                };
                lines.push(line(k.fill, pts));
            }
        }
        let plot = Node::Plot(Plot {
            w: size,
            h: size,
            lines,
        });
        match (self.label, self.value) {
            (true, Some(v)) => Node::Overlay(vec![
                plot,
                styled(
                    format!("{}%", (v * 100.0).round() as u32),
                    size * 0.26,
                    Font::Display,
                    Ink::Text,
                ),
            ]),
            _ => plot,
        }
    }
}

/// The analog clock (local time) with the digital time under it; `size` px dial.
pub fn clock(t: &Theme, size: f32, offset_min: i32) -> Node {
    let p = &t.palette;
    let now = day_secs(unix_now(), offset_min);
    let (hour, min, sec) = hand_angles(now);
    let r = size / 2.0;
    let c = (r, r);
    let pt = |a: f32, len: f32| {
        // 0 rad = 12 o'clock, clockwise.
        let (s, co) = a.sin_cos();
        (c.0 + len * s, c.1 - len * co)
    };
    let seg = |a: f32, from: f32, to: f32, width: f32, color: Color| Line {
        color,
        width,
        points: vec![pt(a, from), pt(a, to)],
    };
    let mut lines: Vec<Line> = (0..60)
        .map(|i| {
            let a = i as f32 * TAU / 60.0;
            if i % 5 == 0 {
                seg(a, r * 0.84, r * 0.96, 2.0, p.text)
            } else {
                seg(a, r * 0.92, r * 0.96, 1.0, p.border_strong)
            }
        })
        .collect();
    lines.push(seg(hour, 0.0, r * 0.5, 4.0, p.text));
    lines.push(seg(min, 0.0, r * 0.74, 3.0, p.text));
    lines.push(seg(sec, -r * 0.14, r * 0.86, 1.5, p.primary));
    // Hub dot.
    lines.push(Line {
        color: p.primary,
        width: 3.5,
        points: arc_points(c, 1.75, 0.0, TAU),
    });
    dial(vec![
        Node::Plot(Plot {
            w: size,
            h: size,
            lines,
        }),
        styled(
            hms(now as u64),
            t.typography.size_xl,
            Font::Display,
            Ink::Text,
        ),
    ])
    .gap(Gap::Sm)
}

/// The countdown: a 160 px ring with the time left, then − / start / reset / + minute.
pub fn timer(t: &Theme, v: &crate::Values, now: f32) -> Node {
    use crate::live_state as L;
    let size = 160.0;
    let s = L::timer(v, now);
    let frac = if s.total_ms == 0 {
        0.0
    } else {
        s.left_ms as f32 / s.total_ms as f32
    };
    let tone = if s.done || frac < 0.1 {
        Some(Tone::Danger)
    } else if s.running {
        None
    } else {
        Some(Tone::Info)
    };
    let ring = Ring {
        tone,
        size: Some(size),
        thickness: Some(7.0),
        label: false,
        ..Ring::new(frac)
    };
    let adj = |label: &str, key: &str| {
        let b = button(label, V::Outline).counter(key);
        if s.running { b.disabled() } else { b }.node()
    };
    let go = button(
        if s.running {
            "Pause"
        } else if s.done {
            "Done"
        } else {
            "Start"
        },
        if s.running { V::Outline } else { V::Primary },
    )
    .counter(L::TIMER_GO);
    dial(vec![
        Node::Overlay(vec![
            ring.node(t, now),
            styled(
                countdown_text(s.left_ms),
                size * 0.2,
                Font::Display,
                if s.done {
                    Ink::Tone(Tone::Danger)
                } else {
                    Ink::Text
                },
            ),
        ]),
        line(vec![
            adj("−1 min", L::TIMER_MINUS),
            if s.done { go.disabled() } else { go }.node(),
            button("Reset", V::Outline).counter(L::TIMER_RESET).node(),
            adj("+1 min", L::TIMER_PLUS),
        ]),
    ])
    .gap(Gap::Md)
}

/// The stopwatch: the time, start / lap / reset and the last laps (split and total).
pub fn stopwatch(t: &Theme, v: &crate::Values, now: f32) -> Node {
    use crate::live_state as L;
    let s = L::stopwatch(v, now);
    let off = |b: crate::node::Button, off: bool| if off { b.disabled() } else { b }.node();
    let splits = dodai_core::clock_math::lap_splits(&s.laps);
    let mut items = vec![
        styled(
            stopwatch_text(s.ms),
            t.typography.size_xxl * 1.4,
            Font::Mono,
            Ink::Text,
        ),
        line(vec![
            button(
                if s.running { "Stop" } else { "Start" },
                if s.running { V::Danger } else { V::Primary },
            )
            .counter(L::SW_GO)
            .node(),
            off(button("Lap", V::Outline).counter(L::SW_LAP), !s.running),
            off(
                button("Reset", V::Outline).counter(L::SW_RESET),
                s.running || (s.ms == 0 && s.laps.is_empty()),
            ),
        ]),
    ];
    let small = |s: String| styled(s, t.typography.size_sm, Font::Mono, Ink::Muted);
    for i in (0..s.laps.len()).rev().take(L::MAX_LAPS) {
        items.push(line(vec![
            small(format!("LAP {}", i + 1)),
            small(format!("+{}", stopwatch_text(splits[i]))),
            small(stopwatch_text(s.laps[i])),
        ]));
    }
    dial(items).gap(Gap::Md)
}

/// The tap-tempo pad: readout, note, beat lights, − / tap / + and play / reset.
pub fn tap_tempo(t: &Theme, v: &crate::Values, now: f32) -> Node {
    use crate::live_state as L;
    use dodai_core::clock_math::{tap_jitter, tempo_name};
    let p = &t.palette;
    let ty = &t.typography;
    let s = L::tap(v, now);
    let light = |i: usize| {
        let base = if i == 0 { p.primary } else { p.secondary };
        let lit = match s.beat {
            Some((b, f)) if b == i => (1.0 - f).powi(2),
            _ => 0.0,
        };
        Node::Block {
            w: 22.0,
            h: 22.0,
            fill: p.border.mix(base, 0.15 + 0.85 * lit),
            border: None,
            radius: 11.0,
            label: None,
        }
    };
    let readout = s
        .bpm
        .map_or("—".to_string(), |b| (b.round() as u32).to_string());
    let note = match (s.bpm, tap_jitter(&s.taps)) {
        (Some(b), Some(j)) => format!(
            "{} · {} taps · {}",
            tempo_name(b),
            s.taps.len(),
            if j < 0.04 {
                "steady"
            } else if j < 0.1 {
                "fairly steady"
            } else {
                "uneven"
            }
        ),
        (Some(b), None) => format!("{} · {} taps", tempo_name(b), s.taps.len()),
        _ => "Tap along with the beat".into(),
    };
    let none = s.bpm.is_none();
    let off = |b: crate::node::Button| if none { b.disabled() } else { b }.node();
    let playing = s.beat.is_some();
    dial(vec![
        line(vec![
            styled(readout, ty.size_xxl * 1.8, Font::Display, Ink::Text),
            styled("BPM", ty.size_md, Font::Body, Ink::Muted),
        ]),
        styled(note, ty.size_sm, Font::Body, Ink::Muted),
        line((0..L::BEATS).map(light).collect()),
        line(vec![
            off(button("−1", V::Outline).counter(L::TAP_MINUS)),
            button("Tap", V::Primary)
                .size(dodai_core::ControlSize::Large)
                .counter(L::TAP)
                .node(),
            off(button("+1", V::Outline).counter(L::TAP_PLUS)),
        ]),
        line(vec![
            off(button(
                if playing { "Stop" } else { "Play" },
                if playing { V::Danger } else { V::Secondary },
            )
            .counter(L::TAP_PLAY)),
            off(button("Reset", V::Outline).counter(L::TAP_RESET)),
        ]),
    ])
    .gap(Gap::Md)
}

/// The machine's UTC offset in minutes (the clock shows local time).
#[cfg(not(target_arch = "wasm32"))]
pub fn local_offset_minutes() -> i32 {
    jiff::Zoned::now().offset().seconds() / 60
}

/// The UTC offset in minutes the host of a browser build gave.
#[cfg(target_arch = "wasm32")]
pub fn local_offset_minutes() -> i32 {
    dodai_core::clock_math::host_utc_offset_minutes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arcs_start_and_end_where_asked() {
        let pts = arc_points((10.0, 10.0), 5.0, 0.0, FRAC_PI_2);
        assert!((pts[0].0 - 15.0).abs() < 1e-4 && (pts[0].1 - 10.0).abs() < 1e-4);
        let last = pts[pts.len() - 1];
        assert!((last.0 - 10.0).abs() < 1e-3 && (last.1 - 15.0).abs() < 1e-3);
        assert!(arc_points((0.0, 0.0), 1.0, 0.0, 0.0).len() >= 2);
    }

    #[test]
    fn a_ring_has_a_track_and_a_value_arc_and_a_label_only_when_determinate() {
        let t = dodai_core::themes::neutral();
        let n = Ring::new(0.25).node(&t, 0.0);
        let Node::Overlay(items) = &n else { panic!() };
        let Node::Plot(p) = &items[0] else { panic!() };
        assert_eq!(p.lines.len(), 2);
        assert!(matches!(&items[1], Node::Text(s, _) if s == "25%"));
        assert!(matches!(Ring::spinning().node(&t, 1.0), Node::Plot(_)));
        let Node::Plot(empty) = Ring {
            label: false,
            ..Ring::new(0.0)
        }
        .node(&t, 0.0) else {
            panic!()
        };
        assert_eq!(empty.lines.len(), 1);
    }

    #[test]
    fn the_clock_draws_sixty_ticks_three_hands_and_a_hub() {
        let t = dodai_core::themes::neutral();
        let Node::Stack(s) = clock(&t, 180.0, 0) else {
            panic!()
        };
        let Node::Plot(p) = &s.items[0] else { panic!() };
        assert_eq!(p.lines.len(), 64);
        assert!(p
            .lines
            .iter()
            .flat_map(|l| &l.points)
            .all(|(x, y)| x.is_finite() && y.is_finite()));
    }
}
