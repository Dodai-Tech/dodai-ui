//! Instrument-cluster sets: round pods around a big tachometer (analog) or a digital panel with
//! segment bars (digital).
//!
//! [`PodGauge`] is a small round instrument with a digital value in the middle and a ring of
//! ticks around it, [`Tachometer`] the big inverted hero dial with gear and drive mode,
//! [`DigitalGauge`] a numeric tile with a segment bar, and [`InstrumentCluster`] arranges them
//! into a complete set. Scales and severities come from `dodai_charts_core::instrument`.

use crate::charts::{polar, turn_arc};
use crate::gauges::{disc, SWEEP};
use crate::instruments::severity_color;
use crate::instruments::DialGauge;
use crate::instruments_hero::{CapsuleCluster, CutSpeedometer};
use crate::paint::{fill, shaped, stroke, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_charts_core::instrument::{readout, Scale, Severity};
use dodai_core::{ClusterKind, DialLook, Theme};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

fn ring(c: f32, r: f32) -> Vec<(f32, f32)> {
    (0..=96).map(|i| polar(c, c, r, i as f32 / 96.0)).collect()
}

fn accent(t: &Theme, s: Severity) -> Hsla {
    if s == Severity::Normal {
        hsla(t.palette.danger)
    } else {
        severity_color(t, s)
    }
}

// ───────────────────────────────── PodGauge ─────────────────────────────────

/// A round pod: a thin tick ring, a marker at the value and the value itself in the middle.
#[derive(IntoElement)]
pub struct PodGauge {
    scale: Scale,
    value: f32,
    label: Option<SharedString>,
    unit: SharedString,
    decimals: usize,
    size: f32,
    sweep: (f32, f32),
    text: Option<SharedString>,
}

impl PodGauge {
    /// A pod showing `value` on `scale`.
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            scale,
            value,
            label: None,
            unit: SharedString::default(),
            decimals: 0,
            size: 170.0,
            sweep: SWEEP,
            text: None,
        }
    }
    /// Small caption above the value.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Unit under the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals of the value.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(90.0);
        self
    }
    /// Start and sweep in turns (0 = 12 o'clock, clockwise).
    pub fn sweep(mut self, start: f32, sweep: f32) -> Self {
        self.sweep = (start, sweep.clamp(0.1, 1.0));
        self
    }
    /// Replace the number in the middle (e.g. a distance).
    pub fn text(mut self, t: impl Into<SharedString>) -> Self {
        self.text = Some(t.into());
        self
    }
}

impl RenderOnce for PodGauge {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let Self {
            scale,
            value,
            label,
            unit,
            decimals,
            size,
            sweep: (start, span),
            text,
        } = self;
        let sev = scale.severity_at(value);
        let frac = scale.fraction(value);
        let (ink, muted, face) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.surface),
        );
        let rim = hsla(t.palette.border);
        let mark = accent(&t, sev);
        let bezel = t.shape.border_width.max(1.5);
        let c = size / 2.0;
        let r_a = c * 0.84;
        let at = move |f: f32| start + span * f;
        let ticks: Vec<(f32, bool, Hsla)> = scale
            .ticks()
            .into_iter()
            .map(|k| {
                let col = match scale.severity_at(k.value) {
                    Severity::Normal => ink,
                    s => severity_color(&t, s),
                };
                (k.frac, k.major, col)
            })
            .collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                fill(w, o, &disc(c, c, c - bezel * 0.5, 72), face);
                stroke(w, o, &ring(c, c - bezel * 0.5), bezel, rim, true);
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_a, at(0.0), at(1.0)),
                    1.5,
                    ink,
                    false,
                );
                for (f, major, col) in &ticks {
                    let len = if *major { size * 0.06 } else { size * 0.03 };
                    let seg = [polar(c, c, r_a, at(*f)), polar(c, c, r_a - len, at(*f))];
                    stroke(w, o, &seg, if *major { 2.2 } else { 1.0 }, *col, false);
                }
                let m = [
                    polar(c, c, r_a + size * 0.035, at(frac)),
                    polar(c, c, r_a - size * 0.08, at(frac)),
                ];
                stroke(w, o, &m, 4.0, mark, false);
            },
        )
        .absolute()
        .size_full();
        let mut pod = div().relative().w(px(size)).h(px(size)).child(cv);
        for k in scale.ticks().into_iter().filter(|k| k.major) {
            let (x, y) = polar(c, c, c * 0.62, at(k.frac));
            pod = pod.child(
                div()
                    .absolute()
                    .left(px(x - 18.0))
                    .top(px(y - 8.0))
                    .w(px(36.0))
                    .flex()
                    .justify_center()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(readout(k.value, 0, "")),
            );
        }
        let mut mid = div()
            .absolute()
            .left_0()
            .top(px(c - size * 0.12))
            .w(px(size))
            .flex()
            .flex_col()
            .items_center();
        mid = mid.child(
            div()
                .font_family(first_family(t.typography.mono))
                .text_size(px((size * 0.2).max(t.typography.size_lg)))
                .text_color(ink)
                .child(text.unwrap_or_else(|| readout(value, decimals, "").into())),
        );
        if !unit.is_empty() {
            mid = mid.child(
                div()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(unit),
            );
        }
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.xs))
            .child(pod.child(mid));
        if let Some(l) = label {
            col = col.child(
                div()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(format_label(&t.typography, &l)),
            );
        }
        col
    }
}

// ───────────────────────────────── Tachometer ─────────────────────────────────

/// The big centre dial: a bold inverted ring with large numerals, a redline block, a thin needle,
/// the engaged gear in a box and the drive mode in a pill.
#[derive(IntoElement)]
pub struct Tachometer {
    scale: Scale,
    value: f32,
    gear: Option<SharedString>,
    mode: Option<SharedString>,
    size: f32,
}

impl Tachometer {
    /// A tachometer showing `value` on `scale` (rpm ×1000).
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            scale,
            value,
            gear: None,
            mode: None,
            size: 260.0,
        }
    }
    /// The engaged gear, e.g. `"6"` or `"D"`.
    pub fn gear(mut self, g: impl Into<SharedString>) -> Self {
        self.gear = Some(g.into());
        self
    }
    /// The drive mode, e.g. `"NORMAL"`.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = Some(m.into());
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(140.0);
        self
    }
}

impl RenderOnce for Tachometer {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let Self {
            scale,
            value,
            gear,
            mode,
            size,
        } = self;
        let sev = scale.severity_at(value);
        let frac = scale.fraction(value);
        let (ink, face_dark, paper) = (
            hsla(t.palette.text),
            hsla(t.palette.background),
            hsla(t.palette.background),
        );
        let needle = accent(&t, sev);
        let c = size / 2.0;
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let zones: Vec<(f32, f32, Hsla)> = scale
            .zone_spans()
            .iter()
            .map(|(a, b, s)| (*a, *b, severity_color(&t, *s)))
            .collect();
        let ticks: Vec<(f32, bool)> = scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                fill(w, o, &disc(c, c, c - 1.0, 96), ink);
                fill(w, o, &disc(c, c, c * 0.6, 72), face_dark);
                for (f, major) in &ticks {
                    let len = if *major { size * 0.05 } else { size * 0.025 };
                    let seg = [
                        polar(c, c, c * 0.97, at(*f)),
                        polar(c, c, c * 0.97 - len, at(*f)),
                    ];
                    stroke(w, o, &seg, if *major { 2.4 } else { 1.0 }, paper, false);
                }
                for (a, z, col) in &zones {
                    let r = c * 0.9;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, at(*a), at(*z)),
                        size * 0.04,
                        *col,
                        false,
                    );
                }
                let line = [
                    polar(c, c, c * 0.14, at(frac)),
                    polar(c, c, c * 0.98, at(frac)),
                ];
                stroke(w, o, &line, 3.5, needle, false);
                fill(w, o, &disc(c, c, size * 0.045, 24), face_dark);
                stroke(w, o, &ring(c, size * 0.045), 2.0, ink, true);
            },
        )
        .absolute()
        .size_full();
        let mut dial = div().relative().w(px(size)).h(px(size)).child(cv);
        for k in scale.ticks().into_iter().filter(|k| k.major) {
            let (x, y) = polar(c, c, c * 0.78, at(k.frac));
            dial = dial.child(
                div()
                    .absolute()
                    .left(px(x - 20.0))
                    .top(px(y - size * 0.045))
                    .w(px(40.0))
                    .flex()
                    .justify_center()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(size * if k.value.abs() >= 100.0 { 0.065 } else { 0.085 }))
                    .text_color(paper)
                    .child(readout(k.value, 0, "")),
            );
        }
        if let Some(g) = gear {
            let style = ShapeStyle::from_tokens(&t, Some(t.palette.text), None, false, false);
            dial = dial.child(
                shaped(&t.shape, style)
                    .absolute()
                    .left(px(c + size * 0.1))
                    .top(px(c - size * 0.08))
                    .w(px(size * 0.15))
                    .h(px(size * 0.16))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(size * 0.11))
                    .text_color(paper)
                    .child(g),
            );
        }
        if let Some(m) = mode {
            let style = ShapeStyle::from_tokens(&t, Some(t.palette.text), None, false, false);
            dial = dial.child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(c + size * 0.26))
                    .w(px(size))
                    .flex()
                    .justify_center()
                    .child(
                        shaped(&t.shape, style)
                            .px(px(size * 0.05))
                            .py(px(size * 0.012))
                            .text_size(px(t.typography.size_xs))
                            .text_color(paper)
                            .child(format_label(&t.typography, &m)),
                    ),
            );
        }
        dial
    }
}

// ───────────────────────────────── DigitalGauge ─────────────────────────────────

/// A numeric tile: caption, a large value with its unit and a segment bar below.
#[derive(IntoElement)]
pub struct DigitalGauge {
    scale: Scale,
    value: f32,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    width: f32,
    big: bool,
}

impl DigitalGauge {
    /// A tile for `value` on `scale`.
    pub fn new(label: impl Into<SharedString>, scale: Scale, value: f32) -> Self {
        Self {
            scale,
            value,
            label: label.into(),
            unit: SharedString::default(),
            decimals: 0,
            width: 160.0,
            big: false,
        }
    }
    /// Unit after the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals of the value.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(100.0);
        self
    }
    /// A hero tile: a much larger number.
    pub fn big(mut self, on: bool) -> Self {
        self.big = on;
        self
    }
}

impl RenderOnce for DigitalGauge {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let sev = self.scale.severity_at(self.value);
        let frac = self.scale.fraction(self.value);
        let width = self.width;
        let seg_n = ((width / 9.0) as usize).clamp(12, 48);
        let cols: Vec<Hsla> = (0..seg_n)
            .map(|i| {
                match self
                    .scale
                    .severity_at(self.scale.value_at((i as f32 + 0.5) / seg_n as f32))
                {
                    Severity::Normal => hsla(t.palette.primary),
                    s => severity_color(&t, s),
                }
            })
            .collect();
        let off = hsla(t.progress(None).track);
        let bar_h = 10.0;
        let inner = width - t.spacing.sm * 2.0;
        let bar = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let step = inner / seg_n as f32;
                for (i, col) in cols.iter().enumerate() {
                    let x = i as f32 * step;
                    let lit = (i as f32 + 0.5) / seg_n as f32 <= frac;
                    let r = [
                        (x, 0.0),
                        (x + step - 2.0, 0.0),
                        (x + step - 2.0, bar_h),
                        (x, bar_h),
                    ];
                    fill(w, b.origin, &r, if lit { *col } else { off });
                }
            },
        )
        .w(px(inner))
        .h(px(bar_h));
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.surface),
            Some(if sev == Severity::Normal {
                t.palette.border
            } else {
                match sev {
                    Severity::Advisory => t.palette.info,
                    Severity::Caution => t.palette.warning,
                    _ => t.palette.danger,
                }
            }),
            false,
            false,
        );
        let num = if self.big {
            t.typography.size_xl * 2.6
        } else {
            t.typography.size_xl * 1.5
        };
        shaped(&t.shape, style)
            .w(px(width))
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .child(
                div()
                    .text_size(px(t.typography.size_xs))
                    .text_color(hsla(t.palette.text_muted))
                    .child(format_label(&t.typography, &self.label)),
            )
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(t.spacing.xs))
                    .child(
                        div()
                            .font_family(first_family(t.typography.mono))
                            .text_size(px(num))
                            .line_height(px(num * 1.05))
                            .text_color(if sev == Severity::Normal {
                                hsla(t.palette.text)
                            } else {
                                severity_color(&t, sev)
                            })
                            .child(readout(self.value, self.decimals, "")),
                    )
                    .child(
                        div()
                            .pb(px(4.0))
                            .text_size(px(t.typography.size_sm))
                            .text_color(hsla(t.palette.text_muted))
                            .child(self.unit),
                    ),
            )
            .child(bar)
    }
}

// ───────────────────────────────── InstrumentCluster ─────────────────────────────────

fn dial_look(kind: ClusterKind) -> Option<DialLook> {
    match kind {
        ClusterKind::Classic => Some(DialLook::Analog),
        ClusterKind::Arc => Some(DialLook::Arc),
        ClusterKind::Segmented => Some(DialLook::Segmented),
        _ => None,
    }
}

/// A complete cluster: speed, engine speed with gear and mode, fuel, oil pressure and coolant.
#[derive(IntoElement)]
pub struct InstrumentCluster {
    kind: Option<ClusterKind>,
    speed: f32,
    rpm: f32,
    gear: SharedString,
    mode: SharedString,
    fuel: f32,
    oil: f32,
    coolant: f32,
    width: f32,
}

impl InstrumentCluster {
    /// A cluster of the given family with plausible cruising values.
    pub fn new(kind: ClusterKind) -> Self {
        Self {
            kind: Some(kind),
            speed: 251.0,
            rpm: 6.9,
            gear: "6".into(),
            mode: "NORMAL".into(),
            fuel: 62.0,
            oil: 4.8,
            coolant: 96.0,
            width: 800.0,
        }
    }
    /// A cluster in the current design's own style (`ChartStyle::cluster`).
    pub fn themed() -> Self {
        Self {
            kind: None,
            ..Self::new(ClusterKind::Pods)
        }
    }
    /// Speed in km/h.
    pub fn speed(mut self, v: f32) -> Self {
        self.speed = v;
        self
    }
    /// Engine speed in ×1000 rpm.
    pub fn rpm(mut self, v: f32) -> Self {
        self.rpm = v;
        self
    }
    /// Engaged gear.
    pub fn gear(mut self, g: impl Into<SharedString>) -> Self {
        self.gear = g.into();
        self
    }
    /// Drive mode label.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = m.into();
        self
    }
    /// Fuel level in %.
    pub fn fuel(mut self, v: f32) -> Self {
        self.fuel = v;
        self
    }
    /// Oil pressure in bar.
    pub fn oil(mut self, v: f32) -> Self {
        self.oil = v;
        self
    }
    /// Coolant temperature in °C.
    pub fn coolant(mut self, v: f32) -> Self {
        self.coolant = v;
        self
    }
    /// Total width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(480.0);
        self
    }
}

impl RenderOnce for InstrumentCluster {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let gap = t.spacing.md;
        let pad = t.spacing.md;
        let inner = self.width - pad * 2.0;
        let speed = Scale::new(0.0, 360.0)
            .ticks_per(6, 1)
            .zone(300.0, 360.0, Severity::Caution);
        let rpm = Scale::new(0.0, 8.0)
            .ticks_per(8, 1)
            .zone(7.0, 8.0, Severity::Alarm);
        let fuel = Scale::new(0.0, 100.0)
            .ticks_per(2, 1)
            .zone(0.0, 12.0, Severity::Alarm)
            .zone(12.0, 25.0, Severity::Caution);
        let oil = Scale::new(0.0, 6.0)
            .ticks_per(6, 1)
            .zone(0.0, 1.0, Severity::Alarm);
        let coolant = Scale::new(40.0, 140.0)
            .ticks_per(5, 1)
            .zone(115.0, 140.0, Severity::Alarm);
        let dash = Scale::new(0.0, 300.0)
            .ticks_per(10, 1)
            .zone(260.0, 300.0, Severity::Caution);
        let kind = self.kind.unwrap_or(t.chart_style().cluster);
        match kind {
            ClusterKind::Cut => {
                let d = Scale::new(0.0, 160.0).ticks_per(8, 1);
                return CutSpeedometer::new(d, self.speed.min(160.0) * 0.28)
                    .unit("km/h")
                    .width(self.width.min(420.0))
                    .into_any_element();
            }
            ClusterKind::Segment => {
                return crate::instruments_segment::SegmentCluster::new()
                    .speed(self.speed)
                    .rpm(self.rpm)
                    .gear(self.gear.clone())
                    .mode(self.mode.clone())
                    .fuel(self.fuel)
                    .oil(self.oil)
                    .coolant(self.coolant)
                    .width(self.width)
                    .into_any_element();
            }
            ClusterKind::Capsule => {
                return CapsuleCluster::new(dash, self.speed.min(300.0) * 0.84)
                    .mode(self.mode.clone())
                    .battery(self.fuel, 460.0)
                    .temp(self.coolant * 0.23)
                    .width(self.width)
                    .into_any_element();
            }
            _ => {}
        }
        let housing = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.background),
            Some(t.palette.border),
            false,
            false,
        );
        let row = div().flex().items_center().justify_center().gap(px(gap));
        let body = match kind {
            ClusterKind::Pods => {
                // 0.62 + 0.78 + 1 + 0.78 + 0.62 hero widths, four gaps.
                let hero = ((inner - gap * 4.0) / 3.8).max(120.0);
                let (mid, small) = (hero * 0.78, hero * 0.62);
                row.child(
                    PodGauge::new(fuel, self.fuel)
                        .label("FUEL")
                        .unit("%")
                        .size(small),
                )
                .child(
                    PodGauge::new(speed, self.speed)
                        .label("SPEED")
                        .unit("km/h")
                        .size(mid),
                )
                .child(
                    Tachometer::new(rpm, self.rpm)
                        .gear(self.gear)
                        .mode(self.mode)
                        .size(hero),
                )
                .child(
                    PodGauge::new(oil, self.oil)
                        .label("OIL")
                        .unit("bar")
                        .decimals(1)
                        .size(mid),
                )
                .child(
                    PodGauge::new(coolant, self.coolant)
                        .label("COOLANT")
                        .unit("°C")
                        .size(small),
                )
                .into_any_element()
            }
            ClusterKind::Cut | ClusterKind::Capsule | ClusterKind::Segment => {
                div().into_any_element()
            }
            ClusterKind::Round => {
                let d = ((inner - gap * 4.0) / 5.0).max(90.0);
                row.child(
                    PodGauge::new(fuel, self.fuel)
                        .label("FUEL")
                        .unit("%")
                        .size(d),
                )
                .child(
                    PodGauge::new(speed, self.speed)
                        .label("SPEED")
                        .unit("km/h")
                        .size(d),
                )
                .child(
                    PodGauge::new(rpm, self.rpm)
                        .label(format!("GEAR {} · {}", self.gear, self.mode))
                        .unit("×1000 rpm")
                        .decimals(1)
                        .size(d),
                )
                .child(
                    PodGauge::new(oil, self.oil)
                        .label("OIL")
                        .unit("bar")
                        .decimals(1)
                        .size(d),
                )
                .child(
                    PodGauge::new(coolant, self.coolant)
                        .label("COOLANT")
                        .unit("°C")
                        .size(d),
                )
                .into_any_element()
            }
            ClusterKind::Bold => {
                let big = ((inner - gap * 3.0) / 3.24).max(120.0);
                let small = big * 0.62;
                row.child(Tachometer::new(fuel, self.fuel).size(small))
                    .child(Tachometer::new(speed, self.speed).size(big))
                    .child(
                        Tachometer::new(rpm, self.rpm)
                            .gear(self.gear)
                            .mode(self.mode)
                            .size(big),
                    )
                    .child(Tachometer::new(coolant, self.coolant).size(small))
                    .into_any_element()
            }
            ClusterKind::Classic | ClusterKind::Arc | ClusterKind::Segmented => {
                let look = dial_look(kind).unwrap_or(DialLook::Analog);
                // 0.62 + 1 + 1 + 0.62 big-dial widths, three gaps.
                let big = ((inner - gap * 3.0) / 3.24).max(120.0);
                let small = big * 0.62;
                row.child(
                    DialGauge::new(fuel, self.fuel)
                        .look(look)
                        .label("Fuel")
                        .unit("%")
                        .tick_labels(false)
                        .size(small),
                )
                .child(
                    DialGauge::new(speed, self.speed)
                        .look(look)
                        .label("Speed")
                        .unit("km/h")
                        .size(big),
                )
                .child(
                    DialGauge::new(rpm, self.rpm)
                        .look(look)
                        .label(format!("Gear {} · {}", self.gear, self.mode))
                        .unit("×1000 rpm")
                        .decimals(1)
                        .size(big),
                )
                .child(
                    DialGauge::new(coolant, self.coolant)
                        .look(look)
                        .label("Coolant")
                        .unit("°C")
                        .tick_labels(false)
                        .size(small),
                )
                .into_any_element()
            }
            ClusterKind::Digital => {
                let side = (inner - gap * 2.0) * 0.26;
                let centre = inner - gap * 2.0 - side * 2.0;
                let col = || div().flex().flex_col().gap(px(gap));
                div()
                    .flex()
                    .items_start()
                    .gap(px(gap))
                    .child(
                        col()
                            .child(
                                DigitalGauge::new("FUEL", fuel, self.fuel)
                                    .unit("%")
                                    .width(side),
                            )
                            .child(
                                DigitalGauge::new("COOLANT", coolant, self.coolant)
                                    .unit("°C")
                                    .width(side),
                            ),
                    )
                    .child(
                        col()
                            .child(
                                DigitalGauge::new("SPEED", speed, self.speed)
                                    .unit("km/h")
                                    .big(true)
                                    .width(centre),
                            )
                            .child(
                                DigitalGauge::new(
                                    format!("ENGINE · GEAR {} · {}", self.gear, self.mode),
                                    rpm,
                                    self.rpm,
                                )
                                .unit("×1000 rpm")
                                .decimals(1)
                                .width(centre),
                            ),
                    )
                    .child(
                        col()
                            .child(
                                DigitalGauge::new("OIL", oil, self.oil)
                                    .unit("bar")
                                    .decimals(1)
                                    .width(side),
                            )
                            .child(
                                DigitalGauge::new(
                                    "BOOST",
                                    Scale::new(0.0, 2.0).ticks_per(4, 1).zone(
                                        1.7,
                                        2.0,
                                        Severity::Caution,
                                    ),
                                    1.2,
                                )
                                .unit("bar")
                                .decimals(1)
                                .width(side),
                            ),
                    )
                    .into_any_element()
            }
        };
        shaped(&t.shape, housing)
            .w(px(self.width))
            .p(px(pad))
            .child(body)
            .into_any_element()
    }
}
