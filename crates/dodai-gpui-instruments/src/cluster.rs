//! Instrument-cluster sets: a dash of dials in any [`Look`], a cut-corner speedometer, a capsule,
//! a segment panel or numeric tiles.
//!
//! [`DigitalGauge`] is a numeric tile with a segment bar, and [`InstrumentCluster`] arranges a
//! complete set. Scales and severities come from `dodai_instruments_core::instrument`.

use crate::dial::severity_color;
use crate::faces::GaugeFace;
use crate::hero::{CapsuleCluster, CutSpeedometer};
use crate::look::Look;
use dodai_core::ClusterKind;
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::icons::Symbol;
use dodai_gpui::paint::{fill, shaped, ShapeStyle};
use dodai_gpui::text::first_family;
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::{readout, Scale, Severity};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

// ───────────────────────────────── DigitalGauge ─────────────────────────────────

/// A numeric tile: caption, a large value with its unit and a segment bar below.
#[derive(IntoElement)]
pub struct DigitalGauge {
    follow: Follow,
    scale: Scale,
    value: f32,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    width: f32,
    big: bool,
}

impl DigitalGauge {
    follow_api!();
    /// A tile for `value` on `scale`.
    #[track_caller]
    pub fn new(label: impl Into<SharedString>, scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
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
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let sev = self.scale.severity_at(self.value);
        let frac = self.scale.fraction(self.value);
        let width = self.width;
        let seg_n = ((width / 6.0) as usize).clamp(16, 64);
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
        let bar_h = 4.0;
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
                        (x + step * 0.75, 0.0),
                        (x + step * 0.75, bar_h),
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
                    .text_size(px(t.typography.size_xs * 0.9))
                    .text_color(hsla(t.palette.text_muted))
                    .child(self.label.to_uppercase()),
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
                            .font_weight(gpui::FontWeight::LIGHT)
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

/// A complete cluster: speed, engine speed with gear and mode, fuel, oil pressure and coolant.
#[derive(IntoElement)]
pub struct InstrumentCluster {
    kind: Option<ClusterKind>,
    look: Look,
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
            look: Look::Sport,
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
            ..Self::new(ClusterKind::Dash)
        }
    }
    /// The look of a [`ClusterKind::Dash`] cluster's dials (Sport by default); the other
    /// clusters have their own.
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
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
                return crate::segment::SegmentCluster::new()
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
            ClusterKind::Cut | ClusterKind::Capsule | ClusterKind::Segment => {
                div().into_any_element()
            }
            ClusterKind::Dash => {
                // 0.62 + 1 + 1 + 0.62 big-dial widths, three gaps.
                let big = ((inner - gap * 3.0) / 3.24).max(120.0);
                let small = big * 0.62;
                let look = self.look;
                let face = |scale: Scale, v: f32, size: f32| {
                    GaugeFace::new(scale, v).look(look).size(size)
                };
                row.child(
                    face(fuel, self.fuel, small)
                        .icon(Symbol::Fuel)
                        .label("Fuel")
                        .unit("%"),
                )
                .child(face(speed, self.speed, big).label("Speed").unit("km/h"))
                .child(
                    face(rpm, self.rpm, big)
                        .label(format!("Gear {} · {}", self.gear, self.mode))
                        .unit("×1000 rpm")
                        .decimals(1),
                )
                .child(
                    face(coolant, self.coolant, small)
                        .icon(Symbol::Coolant)
                        .label("Coolant")
                        .unit("°C"),
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
