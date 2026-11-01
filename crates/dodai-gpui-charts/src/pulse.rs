//! [`Pulse`]: a configurable patient-monitor style trace (ECG and friends).
//!
//! The trace is a pure function of time (see [`dodai_charts_core::pulse`]),
//! drawn as a classic sweep with an eraser gap, a scrolling strip or a static
//! snapshot, over a paper-style or dotted grid, with a rate readout, alarm
//! limits, beat LED, R-peak markers and calibration pulse.

use crate::common::{glow_stroke, ChartGlow};
use dodai_charts_core::pulse::{self, Signal};
pub use dodai_charts_core::pulse::{Kind as PulseKind, Rhythm as PulseRhythm};
use dodai_core::{Color, Theme, Tone};
use dodai_gpui::follow::clock;
use dodai_gpui::paint::{fill, fill_batch, scale_alpha, shaped, stroke, stroke_batch};
use dodai_gpui::text::first_family;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window,
};

/// How the trace moves across the screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulseMode {
    /// Classic monitor: a write head sweeps left to right, erasing ahead of it.
    #[default]
    Sweep,
    /// A strip chart scrolling right to left.
    Scroll,
    /// A still snapshot of the first screen.
    Static,
}

/// Background grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PulseGrid {
    /// No grid.
    None,
    /// Dots at the large-cell corners.
    Dots,
    /// Large-cell lines only.
    Lines,
    /// ECG paper: small and large cells, tinted.
    #[default]
    Paper,
}

/// A professional pulse / vital-sign trace. Build with [`Pulse::ecg`],
/// [`Pulse::pleth`], [`Pulse::art`], [`Pulse::resp`], [`Pulse::eeg`] or
/// [`Pulse::data`], then tune it.
#[derive(IntoElement)]
pub struct Pulse {
    signal: Signal,
    data: Option<(Vec<f32>, f32)>,
    mode: PulseMode,
    live: bool,
    seconds: f32,
    speed: f32,
    gain: f32,
    grid: PulseGrid,
    cells: usize,
    color: Option<Color>,
    line_w: f32,
    glow: ChartGlow,
    trail: f32,
    gap: f32,
    fill: bool,
    head: bool,
    markers: bool,
    calibration: bool,
    beat_led: bool,
    label: Option<SharedString>,
    readout: Option<(SharedString, SharedString)>,
    show_rate: bool,
    alarm: Option<(f32, f32)>,
    footer: Option<SharedString>,
    w: f32,
    h: f32,
}

impl Pulse {
    fn base(kind: PulseKind, rate: f32) -> Self {
        Self {
            signal: Signal {
                kind,
                rate,
                ..Signal::default()
            },
            data: None,
            mode: PulseMode::Sweep,
            live: true,
            seconds: 6.0,
            speed: 1.0,
            gain: 1.0,
            grid: PulseGrid::Paper,
            cells: 12,
            color: None,
            line_w: 2.0,
            glow: ChartGlow::Theme,
            trail: 0.7,
            gap: 0.06,
            fill: false,
            head: true,
            markers: false,
            calibration: false,
            beat_led: false,
            label: None,
            readout: None,
            show_rate: false,
            alarm: None,
            footer: None,
            w: 480.0,
            h: 160.0,
        }
    }
    /// Electrocardiogram at `bpm`.
    pub fn ecg(bpm: f32) -> Self {
        Self::base(PulseKind::Ecg, bpm)
    }
    /// SpO₂ plethysmograph at `bpm`.
    pub fn pleth(bpm: f32) -> Self {
        Self::base(PulseKind::Pleth, bpm)
    }
    /// Arterial pressure wave at `bpm`.
    pub fn art(bpm: f32) -> Self {
        Self::base(PulseKind::Art, bpm)
    }
    /// Respiration at `rpm` breaths per minute.
    pub fn resp(rpm: f32) -> Self {
        Self::base(PulseKind::Resp, rpm)
    }
    /// EEG-like activity.
    pub fn eeg() -> Self {
        Self::base(PulseKind::Eeg, 60.0)
    }
    /// Any other generated kind (sine, square, triangle) at `rate` cycles/min.
    pub fn kind(kind: PulseKind, rate: f32) -> Self {
        Self::base(kind, rate)
    }
    /// Play back recorded `samples` (nominally `-1..=1`) at `rate` Hz, looping.
    pub fn data(samples: impl IntoIterator<Item = f32>, rate: f32) -> Self {
        let mut p = Self::base(PulseKind::Ecg, 60.0);
        p.data = Some((
            samples
                .into_iter()
                .map(crate::common::finite_or_zero)
                .collect(),
            rate.max(1.0),
        ));
        p
    }
    /// ECG rhythm (normal, atrial fibrillation, ST elevation, ...).
    pub fn rhythm(mut self, r: PulseRhythm) -> Self {
        self.signal.rhythm = r;
        self
    }
    /// Beat-to-beat timing variation `0..=1` (heart-rate variability).
    pub fn variability(mut self, v: f32) -> Self {
        self.signal.variability = v.clamp(0.0, 1.0);
        self
    }
    /// Random noise amplitude `0..=0.5` (electrode / motion artefact).
    pub fn noise(mut self, n: f32) -> Self {
        self.signal.noise = n.clamp(0.0, 0.5);
        self
    }
    /// Baseline wander amplitude `0..=0.5`.
    pub fn wander(mut self, n: f32) -> Self {
        self.signal.wander = n.clamp(0.0, 0.5);
        self
    }
    /// Distinguish two otherwise identical traces.
    pub fn seed(mut self, s: f32) -> Self {
        self.signal.seed = s;
        self
    }
    /// Sweep, scroll or static.
    pub fn mode(mut self, m: PulseMode) -> Self {
        self.mode = m;
        self
    }
    /// Animate (default on). Off draws one still frame.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Seconds of signal across the screen (default 6).
    pub fn seconds(mut self, s: f32) -> Self {
        self.seconds = s.clamp(0.5, 60.0);
        self
    }
    /// Playback speed (`1` real time).
    pub fn speed(mut self, s: f32) -> Self {
        self.speed = s.clamp(0.1, 8.0);
        self
    }
    /// Amplitude gain (default 1).
    pub fn gain(mut self, g: f32) -> Self {
        self.gain = g.clamp(0.1, 4.0);
        self
    }
    /// Background grid.
    pub fn grid(mut self, g: PulseGrid) -> Self {
        self.grid = g;
        self
    }
    /// Large grid cells across the width (default 12).
    pub fn cells(mut self, n: usize) -> Self {
        self.cells = n.clamp(2, 60);
        self
    }
    /// Trace colour (default by signal kind).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// Line width in px.
    pub fn line_width(mut self, w: f32) -> Self {
        self.line_w = w.clamp(0.5, 6.0);
        self
    }
    /// Glow under the trace.
    pub fn glow(mut self, g: ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// How strongly older parts of the trace fade: `0` none, `1` strong.
    pub fn trail(mut self, t: f32) -> Self {
        self.trail = t.clamp(0.0, 1.0);
        self
    }
    /// Width of the erased gap ahead of the sweep head, as a fraction of the
    /// screen (default 0.06).
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g.clamp(0.0, 0.4);
        self
    }
    /// Shade the area between the trace and its baseline.
    pub fn fill(mut self, on: bool) -> Self {
        self.fill = on;
        self
    }
    /// A bright dot at the write head (default on).
    pub fn head(mut self, on: bool) -> Self {
        self.head = on;
        self
    }
    /// Mark detected peaks (R waves) with small ticks.
    pub fn markers(mut self, on: bool) -> Self {
        self.markers = on;
        self
    }
    /// Draw a 1 mV calibration pulse at the left edge.
    pub fn calibration(mut self, on: bool) -> Self {
        self.calibration = on;
        self
    }
    /// A LED beside the readout that flashes with each beat.
    pub fn beat_led(mut self, on: bool) -> Self {
        self.beat_led = on;
        self
    }
    /// Lead / channel label, e.g. `"II"`.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// A fixed readout: value and unit, e.g. `("98", "%")`.
    pub fn readout(
        mut self,
        value: impl Into<SharedString>,
        unit: impl Into<SharedString>,
    ) -> Self {
        self.readout = Some((value.into(), unit.into()));
        self
    }
    /// Show the signal's rate as the readout (bpm for ECG, rpm for resp).
    pub fn show_rate(mut self, on: bool) -> Self {
        self.show_rate = on;
        self
    }
    /// Alarm limits for the rate readout: outside `lo..=hi` it turns red and
    /// blinks.
    pub fn alarm(mut self, lo: f32, hi: f32) -> Self {
        self.alarm = Some((lo.min(hi), lo.max(hi)));
        self
    }
    /// Footer text, e.g. `"25 mm/s · 10 mm/mV"`.
    pub fn footer(mut self, f: impl Into<SharedString>) -> Self {
        self.footer = Some(f.into());
        self
    }
    /// Screen size in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w.max(120.0);
        self.h = h.max(60.0);
        self
    }
}

/// Value of the trace at `tau` seconds.
fn value_at(signal: &Signal, data: &Option<(Vec<f32>, f32)>, tau: f32) -> f32 {
    match data {
        Some((v, rate)) if !v.is_empty() => {
            let pos = (tau * rate).rem_euclid(v.len() as f32);
            let (i, f) = (pos.floor() as usize % v.len(), pos.fract());
            let (a, b) = (v[i], v[(i + 1) % v.len()]);
            a + (b - a) * f
        }
        _ => signal.at(tau),
    }
}

/// One drawn sample: position fraction, age `0..1` (0 = newest), value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Pt {
    pub x: f32,
    pub age: f32,
    pub v: f32,
}

/// Screen layout of `n` samples at time `t` for `mode`: which fractions are
/// drawn, how old each is, and where the sweep break falls. Pure.
pub(crate) fn layout(
    mode: PulseMode,
    n: usize,
    t: f32,
    seconds: f32,
    gap: f32,
) -> Vec<Vec<(f32, f32, f32)>> {
    // (x fraction, age, tau) in runs; a new run starts at the sweep head.
    let mut runs: Vec<Vec<(f32, f32, f32)>> = Vec::new();
    let head = (t / seconds).rem_euclid(1.0);
    let mut prev_age = f32::NEG_INFINITY;
    for i in 0..=n {
        let x = i as f32 / n as f32;
        let (age, tau) = match mode {
            PulseMode::Sweep => {
                let age = (head - x).rem_euclid(1.0);
                (age, t - age * seconds)
            }
            PulseMode::Scroll => (1.0 - x, t - (1.0 - x) * seconds),
            PulseMode::Static => (1.0 - x, x * seconds),
        };
        if mode == PulseMode::Sweep && age > 1.0 - gap {
            prev_age = f32::NEG_INFINITY;
            continue;
        }
        if runs.is_empty() || age > prev_age && mode == PulseMode::Sweep {
            runs.push(Vec::new());
        }
        prev_age = age;
        runs.last_mut().unwrap().push((x, age, tau));
    }
    runs.retain(|r| r.len() > 1);
    runs
}

impl Pulse {
    fn trace_color(&self, t: &Theme) -> Color {
        self.color.unwrap_or_else(|| match self.signal.kind {
            PulseKind::Ecg => t.palette.tone(Tone::Success),
            PulseKind::Pleth => t.palette.tone(Tone::Info),
            PulseKind::Art => t.palette.tone(Tone::Danger),
            PulseKind::Resp => t.palette.tone(Tone::Warning),
            _ => t.palette.primary,
        })
    }

    /// The rate to display: the signal's, or estimated from recorded data.
    fn rate(&self) -> Option<f32> {
        match &self.data {
            Some((v, hz)) => {
                let top = dodai_charts_core::peak(v.iter().copied());
                let p = pulse::peaks(v, top * 0.6, (*hz * 0.25) as usize);
                pulse::rate_from_peaks(&p, *hz)
            }
            None => (self.signal.rhythm != PulseRhythm::Asystole).then_some(self.signal.rate),
        }
    }
}

impl RenderOnce for Pulse {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let col = hsla(self.trace_color(&t));
        let glow = self.glow.resolve(&t);
        let now = if self.live { clock() * self.speed } else { 0.0 };
        let mode = if self.live {
            self.mode
        } else {
            PulseMode::Static
        };
        let (w_px, h_px) = (self.w, self.h);
        let (grid_kind, cells) = (self.grid, self.cells);
        let (seconds, gain, gap, trail) = (self.seconds, self.gain, self.gap, self.trail);
        let (line_w, fill_on, head_on, markers, cal, live) = (
            self.line_w,
            self.fill,
            self.head,
            self.markers,
            self.calibration,
            self.live,
        );
        let (signal, data) = (self.signal, self.data.clone());
        let paper = hsla(t.palette.tone(Tone::Danger));
        let (grid_c, text_c) = (hsla(t.palette.border), hsla(t.palette.text));
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let o = b.origin;
                let mid = bh / 2.0;
                let big = bw / cells as f32;
                // grid
                match grid_kind {
                    PulseGrid::None => {}
                    PulseGrid::Dots => {
                        let mut dots = Vec::new();
                        let mut y = 0.0;
                        while y <= bh {
                            for i in 0..=cells {
                                let x = i as f32 * big;
                                dots.push(crate::common::rect(x - 1.0, y - 1.0, 2.0, 2.0).to_vec());
                            }
                            y += big;
                        }
                        fill_batch(w, o, &dots, scale_alpha(grid_c, 0.9));
                    }
                    PulseGrid::Lines | PulseGrid::Paper => {
                        let paper_on = grid_kind == PulseGrid::Paper;
                        let (c_big, c_small) = if paper_on {
                            (scale_alpha(paper, 0.34), scale_alpha(paper, 0.13))
                        } else {
                            (scale_alpha(grid_c, 0.8), grid_c)
                        };
                        let step = big / 5.0;
                        if paper_on {
                            let (mut v, mut h) = (Vec::new(), Vec::new());
                            let mut x = 0.0;
                            while x <= bw + 0.5 {
                                v.push(vec![(x, 0.0), (x, bh)]);
                                x += step;
                            }
                            let mut y = mid.rem_euclid(step);
                            while y <= bh + 0.5 {
                                h.push(vec![(0.0, y), (bw, y)]);
                                y += step;
                            }
                            stroke_batch(w, o, &v, 1.0, c_small, false);
                            stroke_batch(w, o, &h, 1.0, c_small, false);
                        }
                        let (mut v, mut h) = (Vec::new(), Vec::new());
                        for i in 0..=cells {
                            v.push(vec![(i as f32 * big, 0.0), (i as f32 * big, bh)]);
                        }
                        let mut y = mid.rem_euclid(big);
                        while y <= bh + 0.5 {
                            h.push(vec![(0.0, y), (bw, y)]);
                            y += big;
                        }
                        stroke_batch(w, o, &v, 1.0, c_big, false);
                        stroke_batch(w, o, &h, 1.0, c_big, false);
                    }
                }
                let amp = bh * 0.4 * gain;
                let n = ((bw / 1.5) as usize).clamp(16, 1200);
                let runs = layout(mode, n, now, seconds, gap);
                let mut head_pt = None;
                for run in &runs {
                    let pts: Vec<Pt> = run
                        .iter()
                        .map(|&(x, age, tau)| Pt {
                            x,
                            age,
                            v: value_at(&signal, &data, tau),
                        })
                        .collect();
                    let xy: Vec<(f32, f32)> =
                        pts.iter().map(|p| (p.x * bw, mid - p.v * amp)).collect();
                    // Newest sample of the run is the head candidate.
                    if let Some((i, _)) = pts
                        .iter()
                        .enumerate()
                        .min_by(|a, b| a.1.age.total_cmp(&b.1.age))
                    {
                        if mode == PulseMode::Sweep || i + 1 == pts.len() {
                            head_pt = Some((xy[i], pts[i].age));
                        }
                    }
                    if fill_on {
                        let quads: Vec<Vec<(f32, f32)>> = xy
                            .windows(2)
                            .map(|p| vec![p[0], p[1], (p[1].0, mid), (p[0].0, mid)])
                            .collect();
                        fill_batch(w, o, &quads, scale_alpha(col, 0.14));
                    }
                    // Fade older chunks.
                    let chunk = 8;
                    let mut s = 0;
                    while s + 1 < xy.len() {
                        let e = (s + chunk).min(xy.len() - 1);
                        let age =
                            pts[s..=e].iter().map(|p| p.age).sum::<f32>() / (e - s + 1) as f32;
                        let a = (1.0 - trail * 0.85 * age.clamp(0.0, 1.0)).clamp(0.08, 1.0);
                        glow_stroke(
                            w,
                            o,
                            &xy[s..=e],
                            line_w,
                            scale_alpha(col, a),
                            glow.filter(|_| age < 0.25),
                        );
                        s = e;
                    }
                    if markers {
                        let vals: Vec<f32> = pts.iter().map(|p| p.v).collect();
                        let top = dodai_charts_core::peak(vals.iter().copied());
                        if top > 0.25 {
                            let gap_n = (n as f32 * 0.08) as usize;
                            for i in pulse::peaks(&vals, top * 0.7, gap_n.max(3)) {
                                let (x, y) = xy[i];
                                fill(
                                    w,
                                    o,
                                    &[(x - 3.0, y - 12.0), (x + 3.0, y - 12.0), (x, y - 6.0)],
                                    scale_alpha(text_c, 0.8),
                                );
                            }
                        }
                    }
                }
                if cal {
                    let hgt = amp * 0.5;
                    stroke(
                        w,
                        o,
                        &[
                            (6.0, mid),
                            (6.0, mid - hgt),
                            (6.0 + big * 0.6, mid - hgt),
                            (6.0 + big * 0.6, mid),
                        ],
                        1.5,
                        scale_alpha(text_c, 0.7),
                        false,
                    );
                }
                if head_on && live {
                    if let Some(((x, y), age)) = head_pt {
                        if age < 0.02 {
                            for (r, a) in [(7.0, 0.18), (4.5, 0.4), (2.6, 1.0)] {
                                fill(
                                    w,
                                    o,
                                    &crate::common::rect(x - r, y - r, r * 2.0, r * 2.0),
                                    scale_alpha(col, a),
                                );
                            }
                        }
                    }
                }
                if live {
                    dodai_gpui::follow::next_frame(w);
                }
            },
        )
        .absolute()
        .size_full();
        let shown = match (&self.readout, self.show_rate) {
            (Some((v, u)), _) => Some((v.to_string(), u.clone())),
            (None, true) => self.rate().map(|r| {
                (
                    format!("{r:.0}"),
                    SharedString::from(
                        if self.signal.kind == PulseKind::Resp && self.data.is_none() {
                            "rpm"
                        } else {
                            "bpm"
                        },
                    ),
                )
            }),
            _ => None,
        };
        let alarming = match (self.alarm, self.rate()) {
            (Some((lo, hi)), Some(r)) => r < lo || r > hi,
            _ => false,
        };
        let blink = !alarming || (clock() * 2.0).fract() < 0.6;
        let readout_c = if alarming {
            hsla(t.palette.tone(Tone::Danger))
        } else {
            col
        };
        let mono = first_family(t.typography.mono);
        let mut head_row = div().flex().items_center().justify_between().w(px(w_px));
        head_row = head_row.child(
            div()
                .font_family(mono.clone())
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(self.label.clone().unwrap_or_default()),
        );
        if let Some((v, u)) = shown {
            let beat = if self.beat_led && self.live {
                let x = pulse::beat_phase(clock() * self.speed, self.signal.rate, 0.0, 0.0);
                (-x * 7.0).exp()
            } else {
                0.0
            };
            head_row = head_row.child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(t.spacing.xs))
                    .children(self.beat_led.then(|| {
                        div()
                            .w(px(8.0))
                            .h(px(8.0))
                            .rounded_full()
                            .bg(scale_alpha(readout_c, 0.18 + 0.82 * beat))
                    }))
                    .child(
                        div()
                            .font_family(mono.clone())
                            .text_size(px(t.typography.size_xxl))
                            .text_color(scale_alpha(readout_c, if blink { 1.0 } else { 0.25 }))
                            .child(v),
                    )
                    .child(
                        div()
                            .font_family(mono.clone())
                            .text_size(px(t.typography.size_xs))
                            .text_color(hsla(t.palette.text_muted))
                            .child(u),
                    ),
            );
        }
        let mut panel = shaped(&t.shape, dodai_gpui::panel::panel_style(&t, false))
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .child(head_row)
            .child(div().relative().w(px(w_px)).h(px(h_px)).child(cv));
        if let Some(f) = self.footer {
            panel = panel.child(
                div()
                    .font_family(mono)
                    .text_size(px(t.typography.size_xs))
                    .text_color(hsla(t.palette.text_muted))
                    .child(f),
            );
        }
        panel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sweep_has_a_gap_and_breaks_at_the_head() {
        let runs = layout(PulseMode::Sweep, 100, 2.0, 6.0, 0.1);
        assert_eq!(runs.len(), 2, "one run before the head, one after the gap");
        let drawn: usize = runs.iter().map(Vec::len).sum();
        assert!(drawn < 101 && drawn > 80, "drawn {drawn}");
        // ages are 0..1 and the newest sample is at the head
        let min = runs
            .iter()
            .flatten()
            .map(|p| p.1)
            .fold(f32::INFINITY, f32::min);
        assert!(min < 0.02);
    }

    #[test]
    fn scroll_and_static_are_single_runs() {
        assert_eq!(layout(PulseMode::Scroll, 50, 9.3, 6.0, 0.1).len(), 1);
        let s = layout(PulseMode::Static, 50, 9.3, 6.0, 0.1);
        assert_eq!(s.len(), 1);
        // static ignores the clock
        assert_eq!(s, layout(PulseMode::Static, 50, 0.0, 6.0, 0.1));
    }

    #[test]
    fn recorded_data_loops_and_interpolates() {
        let d = Some((vec![0.0, 1.0], 2.0));
        let s = Signal::default();
        assert!((value_at(&s, &d, 0.25) - 0.5).abs() < 1e-5);
        assert!((value_at(&s, &d, 1.25) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn rate_is_estimated_from_recorded_data() {
        let sig = Signal::default();
        let hz = 200.0;
        let v: Vec<f32> = (0..(hz * 10.0) as usize)
            .map(|i| sig.at(i as f32 / hz))
            .collect();
        let r = Pulse::data(v, hz).rate().unwrap();
        assert!((r - 72.0).abs() < 3.0, "rate {r}");
    }
}
