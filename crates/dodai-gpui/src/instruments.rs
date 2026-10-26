//! Instruments for vehicles, process plants and patient monitors: [`DialGauge`], [`VitalsTile`]
//! and [`Annunciator`].
//!
//! Scales, zones, alarm limits and flashing come from `dodai_charts_core::instrument`; this
//! module only paints. Severity colours come from the theme (`success`, `info`, `warning`,
//! `danger`), outlines and shapes from the design, so every design reads the same way at a glance
//! while still looking like itself. Flashing stops when the alarm is acknowledged and only
//! requests frames while something actually flashes.

use crate::charts::{polar, turn_arc};
use crate::gauges::{disc, SWEEP};
use crate::instruments_motion::{follow, follow_api, Follow};
use crate::paint::{fill, scale_alpha, shaped, stroke, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_charts_core::instrument::{readout, Alarm, Limits, Scale, Severity};
use dodai_core::{Color, DialLook, Theme};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

fn clock() -> f32 {
    crate::instruments_motion::clock()
}

/// The theme colour of a severity: success, info, warning, danger.
pub fn severity_color(t: &Theme, s: Severity) -> Hsla {
    hsla(severity_token(t, s))
}

fn severity_token(t: &Theme, s: Severity) -> Color {
    match s {
        Severity::Normal => t.palette.success,
        Severity::Advisory => t.palette.info,
        Severity::Caution => t.palette.warning,
        Severity::Alarm => t.palette.danger,
    }
}

/// A flashing alarm keeps the window redrawing; a steady one does not.
fn flashing(window: &Window, live: bool, a: &Alarm) {
    if live && a.severity.flash_hz() > 0.0 && !a.acknowledged {
        crate::instruments_motion::next_frame(window);
    }
}

// ───────────────────────────────── DialGauge ─────────────────────────────────

/// An analogue dial: a scale with coloured zones, tick marks with labels, a needle and a digital
/// readout. A speedometer, a tachometer or a process temperature dial.
#[derive(IntoElement)]
pub struct DialGauge {
    follow: Follow,
    scale: Scale,
    value: f32,
    label: Option<SharedString>,
    unit: SharedString,
    decimals: usize,
    size: f32,
    sweep: (f32, f32),
    tick_labels: bool,
    look: Option<DialLook>,
}

impl DialGauge {
    follow_api!();
    /// A dial showing `value` on `scale` (build the scale with [`Scale::new`] and zones).
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            label: None,
            unit: SharedString::default(),
            decimals: 0,
            size: 200.0,
            sweep: SWEEP,
            tick_labels: true,
            look: None,
        }
    }
    /// Caption under the readout.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Unit shown after the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals of the readout.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(80.0);
        self
    }
    /// Start and sweep in turns (0 = 12 o'clock, clockwise); default 270° with the gap below.
    pub fn sweep(mut self, start: f32, sweep: f32) -> Self {
        self.sweep = (start, sweep.clamp(0.1, 1.0));
        self
    }
    /// Force a look; by default the design's own (`Theme::chart_style().dial`).
    pub fn look(mut self, l: DialLook) -> Self {
        self.look = Some(l);
        self
    }
    /// Show the numbers at the major ticks (default on).
    pub fn tick_labels(mut self, on: bool) -> Self {
        self.tick_labels = on;
        self
    }
}

impl RenderOnce for DialGauge {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let Self {
            follow: _,
            scale,
            value,
            label,
            unit,
            decimals,
            size,
            sweep: (start, span),
            tick_labels,
            look,
        } = self;
        let look = look.unwrap_or(t.chart_style().dial);
        let analog = look == DialLook::Analog;
        let sev = scale.severity_at(value);
        let frac = scale.fraction(value);
        // Tracks follow the design's progress/slider tokens, so a dial reads like its bars.
        let sz = t.sizes();
        let (track, ink, muted) = (
            hsla(t.progress(None).track),
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
        );
        let accent = if sev == Severity::Normal {
            hsla(t.palette.primary)
        } else {
            severity_color(&t, sev)
        };
        let needle_col = accent;
        let c = size / 2.0;
        let bezel_w = t.shape.border_width.max(1.5);
        let face = hsla(t.palette.surface);
        let rim = hsla(t.palette.border);
        let at = move |f: f32| start + span * f;
        // Ticks coloured by the zone they sit in: a redline reads as a red run of ticks.
        let ticks: Vec<(f32, bool, Hsla)> = scale
            .ticks()
            .into_iter()
            .map(|k| {
                let col = match scale.severity_at(k.value) {
                    Severity::Normal => {
                        if k.major {
                            ink
                        } else {
                            muted
                        }
                    }
                    s => severity_color(&t, s),
                };
                (k.frac, k.major, col)
            })
            .collect();
        let zone_cols: Vec<(f32, f32, Hsla)> = scale
            .zone_spans()
            .iter()
            .map(|(a, b, s)| (*a, *b, severity_color(&t, *s)))
            .collect();
        const SEGMENTS: usize = 30;
        let seg_cols: Vec<Hsla> = (0..SEGMENTS)
            .map(|i| {
                let f = (i as f32 + 0.5) / SEGMENTS as f32;
                match scale.severity_at(scale.value_at(f)) {
                    Severity::Normal => hsla(t.palette.primary),
                    s => severity_color(&t, s),
                }
            })
            .collect();
        let band = (sz.slider_track * 1.2).max(size * 0.03);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                // Bezel and face.
                fill(w, o, &disc(c, c, c - bezel_w * 0.5, 72), face);
                let rim_pts: Vec<(f32, f32)> = (0..=96)
                    .map(|i| polar(c, c, c - bezel_w * 0.5, i as f32 / 96.0))
                    .collect();
                stroke(w, o, &rim_pts, bezel_w, rim, true);
                let tick_ring = |w: &mut Window, r_out: f32, minor: f32, major: f32| {
                    for (f, is_major, col) in &ticks {
                        let len = if *is_major { major } else { minor };
                        let seg = [polar(c, c, r_out, at(*f)), polar(c, c, r_out - len, at(*f))];
                        stroke(w, o, &seg, if *is_major { 2.2 } else { 1.0 }, *col, false);
                    }
                };
                match look {
                    DialLook::Analog => {
                        let r_z = c * 0.86;
                        let thin = (band * 0.55).max(2.0);
                        stroke(
                            w,
                            o,
                            &turn_arc(c, c, r_z, at(0.0), at(1.0)),
                            thin,
                            track,
                            false,
                        );
                        for (a, z, col) in &zone_cols {
                            stroke(
                                w,
                                o,
                                &turn_arc(c, c, r_z, at(*a), at(*z)),
                                thin,
                                *col,
                                false,
                            );
                        }
                        // Lit sweep behind the needle.
                        if frac > 0.004 {
                            let mut sector = vec![(c, c)];
                            let n = ((frac * 48.0) as usize).max(2);
                            sector
                                .extend((0..=n).map(|i| {
                                    polar(c, c, c * 0.78, at(frac * i as f32 / n as f32))
                                }));
                            fill(w, o, &sector, scale_alpha(accent, 0.11));
                        }
                        tick_ring(w, c * 0.78, size * 0.028, size * 0.06);
                        // Needle: tapered, with a short tail behind the hub.
                        let tip = polar(c, c, c * 0.84, at(frac));
                        let tail = polar(c, c, size * 0.09, at(frac) + 0.5);
                        let wing = size * 0.018;
                        let l = polar(c, c, wing, at(frac) - 0.25);
                        let r = polar(c, c, wing, at(frac) + 0.25);
                        fill(w, o, &[l, tip, r, tail], needle_col);
                        fill(w, o, &disc(c, c, size * 0.05, 24), face);
                        let hub: Vec<(f32, f32)> = (0..=24)
                            .map(|i| polar(c, c, size * 0.05, i as f32 / 24.0))
                            .collect();
                        stroke(w, o, &hub, 2.0, needle_col, true);
                        fill(w, o, &disc(c, c, size * 0.018, 16), needle_col);
                    }
                    DialLook::Arc => {
                        let r_a = c * 0.84;
                        let thick = band * 1.4;
                        stroke(
                            w,
                            o,
                            &turn_arc(c, c, r_a, at(0.0), at(1.0)),
                            thick,
                            track,
                            false,
                        );
                        if frac > 0.002 {
                            stroke(
                                w,
                                o,
                                &turn_arc(c, c, r_a, at(0.0), at(frac)),
                                thick,
                                accent,
                                false,
                            );
                            let (ex, ey) = polar(c, c, r_a, at(frac));
                            fill(w, o, &disc(ex, ey, thick * 0.9, 20), face);
                            let knob: Vec<(f32, f32)> = (0..=20)
                                .map(|i| polar(ex, ey, thick * 0.9, i as f32 / 20.0))
                                .collect();
                            stroke(w, o, &knob, 2.0, accent, true);
                        }
                        tick_ring(w, r_a - thick * 1.3, size * 0.02, size * 0.04);
                    }
                    DialLook::Segmented => {
                        let r_a = c * 0.84;
                        let thick = band * 1.7;
                        let step = 1.0 / SEGMENTS as f32;
                        for (i, col) in seg_cols.iter().enumerate() {
                            let (a0, a1) = (
                                i as f32 * step + step * 0.12,
                                (i + 1) as f32 * step - step * 0.12,
                            );
                            let lit = (i as f32 + 0.5) * step <= frac;
                            let col = if lit { *col } else { track };
                            stroke(
                                w,
                                o,
                                &turn_arc(c, c, r_a, at(a0), at(a1)),
                                thick,
                                col,
                                false,
                            );
                        }
                        tick_ring(w, r_a - thick * 1.1, size * 0.02, size * 0.04);
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let mut dial = div().relative().w(px(size)).h(px(size)).child(cv);
        if tick_labels && analog {
            // Dense scales label every other major tick.
            let skip = if scale.majors > 8 { 2 } else { 1 };
            let majors = scale.ticks().into_iter().filter(|k| k.major);
            for k in majors
                .enumerate()
                .filter(|(i, _)| i % skip == 0)
                .map(|(_, k)| k)
            {
                let (x, y) = polar(c, c, c * 0.6, at(k.frac));
                dial = dial.child(
                    div()
                        .absolute()
                        .left(px(x - 20.0))
                        .top(px(y - 8.0))
                        .w(px(40.0))
                        .flex()
                        .justify_center()
                        .text_size(px(t.typography.size_xs))
                        .text_color(ink)
                        .child(readout(k.value, 0, "")),
                );
            }
        }
        let mut read = div()
            .absolute()
            .left_0()
            .w(px(size))
            .top(px(c + size * 0.16))
            .flex()
            .flex_col()
            .items_center()
            .child(
                div()
                    .text_size(px(t.typography.size_xl))
                    .font_family(first_family(t.typography.mono))
                    .text_color(if sev == Severity::Normal {
                        ink
                    } else {
                        needle_col
                    })
                    .child(readout(value, decimals, "")),
            );
        if !unit.is_empty() {
            read = read.child(
                div()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(unit),
            );
        }
        if let Some(l) = label {
            read = read.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(muted)
                    .child(format_label(&t.typography, &l)),
            );
        }
        dial.child(read)
    }
}

// ───────────────────────────────── VitalsTile ─────────────────────────────────

/// A patient-monitor numeric: label, large value, unit and alarm limits. The border and value
/// take the severity colour; an unacknowledged alarm flashes its background.
#[derive(IntoElement)]
pub struct VitalsTile {
    label: SharedString,
    value: f32,
    unit: SharedString,
    decimals: usize,
    limits: Limits,
    acknowledged: bool,
    live: bool,
    width: f32,
}

impl VitalsTile {
    /// A tile for `value`, e.g. `VitalsTile::new("HR", 72.0).unit("bpm")`.
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            unit: SharedString::default(),
            decimals: 0,
            limits: Limits::default(),
            acknowledged: false,
            live: true,
            width: 180.0,
        }
    }
    /// Unit shown beside the label.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals of the value.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Alarm limits; the tile shows them and rates the value against them.
    pub fn limits(mut self, l: Limits) -> Self {
        self.limits = l;
        self
    }
    /// The operator has acknowledged the alarm: it stops flashing and stays lit.
    pub fn acknowledged(mut self, a: bool) -> Self {
        self.acknowledged = a;
        self
    }
    /// Let an unacknowledged alarm flash (default on). Off keeps it steady.
    pub fn flash(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(100.0);
        self
    }
}

impl RenderOnce for VitalsTile {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let sev = self.limits.severity(self.value);
        let mut alarm = Alarm::default();
        alarm.update(sev);
        alarm.acknowledged = self.acknowledged;
        flashing(window, self.live, &alarm);
        let col = severity_color(&t, sev);
        let lit = !self.live || alarm.lit(clock());
        let alarmed = sev != Severity::Normal;
        let fill_col = if alarmed && lit {
            severity_token(&t, sev).with_alpha(0.22)
        } else {
            t.palette.surface
        };
        // A pale severity colour on a pale tile would not read: fall back to the text colour.
        let readable = severity_token(&t, sev).contrast(t.palette.surface) >= 3.0;
        let ink = if alarmed && readable {
            col
        } else {
            hsla(t.palette.text)
        };
        let edge = if alarmed {
            severity_token(&t, sev)
        } else {
            t.palette.border
        };
        let style = ShapeStyle::from_tokens(&t, Some(fill_col), Some(edge), false, false);
        let range = {
            let (lo, hi) = (self.limits.low, self.limits.high);
            match (lo, hi) {
                (Some(a), Some(b)) => format!("{}–{}", readout(a, 0, ""), readout(b, 0, "")),
                (None, Some(b)) => format!("< {}", readout(b, 0, "")),
                (Some(a), None) => format!("> {}", readout(a, 0, "")),
                _ => String::new(),
            }
        };
        let muted = hsla(t.palette.text_muted);
        shaped(&t.shape, style)
            .w(px(self.width))
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(t.typography.size_sm))
                    .text_color(muted)
                    .child(div().child(format_label(&t.typography, &self.label)))
                    .child(div().child(self.unit)),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_xl * 1.8))
                    .font_family(first_family(t.typography.mono))
                    .text_color(ink)
                    .child(readout(self.value, self.decimals, "")),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(range),
            )
    }
}

// ───────────────────────────────── Annunciator ─────────────────────────────────

/// A panel of alarm windows. A window is dark when normal, lit in its severity colour when
/// active, and flashes until acknowledged.
#[derive(IntoElement)]
pub struct Annunciator {
    tiles: Vec<(SharedString, Severity, bool)>,
    columns: usize,
    tile_w: f32,
    live: bool,
}

impl Annunciator {
    /// Windows as `(legend, severity, acknowledged)`.
    pub fn new<S: Into<SharedString>>(
        tiles: impl IntoIterator<Item = (S, Severity, bool)>,
    ) -> Self {
        Self {
            tiles: tiles
                .into_iter()
                .map(|(l, s, a)| (l.into(), s, a))
                .collect(),
            columns: 4,
            tile_w: 120.0,
            live: true,
        }
    }
    /// Windows per row.
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
    /// Width of one window in px.
    pub fn tile_width(mut self, w: f32) -> Self {
        self.tile_w = w.max(60.0);
        self
    }
    /// Let unacknowledged alarms flash (default on).
    pub fn flash(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
}

impl RenderOnce for Annunciator {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let now = clock();
        let alarms: Vec<Alarm> = self
            .tiles
            .iter()
            .map(|(_, s, a)| Alarm {
                severity: *s,
                acknowledged: *a,
            })
            .collect();
        if alarms
            .iter()
            .any(|a| a.severity.flash_hz() > 0.0 && !a.acknowledged)
        {
            flashing(
                window,
                self.live,
                &Alarm {
                    severity: Severity::Alarm,
                    acknowledged: false,
                },
            );
        }
        let mut grid = div().flex().flex_wrap().gap(px(t.spacing.xs));
        for ((label, sev, _), alarm) in self.tiles.iter().zip(&alarms) {
            let on = *sev != Severity::Normal && (!self.live || alarm.lit(now));
            let tok = severity_token(&t, *sev);
            let style = ShapeStyle::from_tokens(
                &t,
                Some(if on {
                    tok
                } else {
                    t.palette.border.with_alpha(0.35)
                }),
                Some(if *sev == Severity::Normal {
                    t.palette.border
                } else {
                    tok
                }),
                false,
                false,
            );
            let text = if on {
                hsla(t.palette.background)
            } else {
                hsla(t.palette.text_muted)
            };
            grid = grid.child(
                shaped(&t.shape, style)
                    .w(px(self.tile_w))
                    .h(px(t.shape.control_height * 1.2))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(t.typography.size_sm))
                    .text_color(text)
                    .child(format_label(&t.typography, label)),
            );
        }
        div()
            .w(px(
                self.tile_w * self.columns as f32 + t.spacing.xs * (self.columns as f32 - 1.0)
            ))
            .child(grid)
    }
}
