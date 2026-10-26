//! Vehicle and machine instruments beside [`crate::DialGauge`]: [`LevelBar`], [`TirePressure`],
//! [`GearIndicator`], [`Odometer`], [`PowerMeter`] and [`Compass`].
//!
//! Scales and severities come from `dodai_charts_core::instrument`; colours from the theme's
//! severity tokens; panels from the design's shape tokens.

use crate::charts::polar;
use crate::instruments::severity_color;
use crate::instruments_motion::{follow, follow_all, follow_angle, follow_api, Follow};
use crate::paint::{fill, shaped, stroke, ShapeStyle};
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_charts_core::gauge::odometer_positions;
use dodai_charts_core::instrument::{readout, Limits, Scale, Severity};
use dodai_core::{Color, Theme};
use gpui::{
    canvas, div, px, App, Bounds, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window,
};

fn token(t: &Theme, s: Severity) -> Color {
    match s {
        Severity::Normal => t.palette.primary,
        Severity::Advisory => t.palette.info,
        Severity::Caution => t.palette.warning,
        Severity::Alarm => t.palette.danger,
    }
}

// ───────────────────────────────── LevelBar ─────────────────────────────────

/// A vertical level bar (fuel, tank, battery) with a zone strip and numbered ticks.
#[derive(IntoElement)]
pub struct LevelBar {
    follow: Follow,
    scale: Scale,
    value: f32,
    label: Option<SharedString>,
    unit: SharedString,
    height: f32,
}

impl LevelBar {
    follow_api!();
    /// A bar showing `value` on `scale`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            label: None,
            unit: SharedString::default(),
            height: 160.0,
        }
    }
    /// Caption under the bar.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Unit after the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Bar height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h.max(60.0);
        self
    }
}

impl RenderOnce for LevelBar {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let sev = self.scale.severity_at(self.value);
        let frac = self.scale.fraction(self.value);
        let h = self.height;
        let bar_w = 24.0;
        let track = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.surface),
            Some(t.palette.border),
            false,
            false,
        );
        let fill_col = token(&t, sev);
        let bar = div()
            .relative()
            .w(px(bar_w))
            .h(px(h))
            .child(shaped(&t.shape, track).size_full())
            .child(
                div()
                    .absolute()
                    .left(px(3.0))
                    .bottom(px(3.0))
                    .w(px(bar_w - 6.0))
                    .h(px(((h - 6.0) * frac).max(0.0)))
                    .rounded_sm()
                    .bg(hsla(fill_col)),
            );
        let mut strip = div().relative().w(px(5.0)).h(px(h));
        for (a, b, s) in self.scale.zone_spans() {
            strip = strip.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .bottom(px(h * a))
                    .h(px(h * (b - a)))
                    .bg(severity_color(&t, s)),
            );
        }
        let mut ticks = div().relative().w(px(34.0)).h(px(h));
        for k in self.scale.ticks().into_iter().filter(|k| k.major) {
            ticks = ticks.child(
                div()
                    .absolute()
                    .left(px(6.0))
                    .bottom(px(h * k.frac - 7.0))
                    .text_size(px(t.typography.size_xs))
                    .text_color(hsla(t.palette.text_muted))
                    .child(readout(k.value, 0, "")),
            );
        }
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.xs))
            .child(
                div()
                    .flex()
                    .gap(px(3.0))
                    .child(bar)
                    .child(strip)
                    .child(ticks),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_lg))
                    .font_family(first_family(t.typography.mono))
                    .text_color(if sev == Severity::Normal {
                        hsla(t.palette.text)
                    } else {
                        severity_color(&t, sev)
                    })
                    .child(readout(self.value, 0, &self.unit)),
            );
        if let Some(l) = self.label {
            col = col.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(l),
            );
        }
        col
    }
}

// ───────────────────────────────── TirePressure ─────────────────────────────────

/// Four tyre pressures around a vehicle outline. Each is rated against the same limits.
#[derive(IntoElement)]
pub struct TirePressure {
    /// Front-left, front-right, rear-left, rear-right.
    values: [f32; 4],
    limits: Limits,
    unit: SharedString,
}

impl TirePressure {
    /// Pressures in front-left, front-right, rear-left, rear-right order.
    pub fn new(values: [f32; 4], limits: Limits) -> Self {
        Self {
            values,
            limits,
            unit: "bar".into(),
        }
    }
    /// Unit shown under the values.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
}

impl RenderOnce for TirePressure {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let chip = |name: &'static str, v: f32| {
            let sev = self.limits.severity(v);
            let tok = token(&t, sev);
            let style = ShapeStyle::from_tokens(
                &t,
                Some(if sev == Severity::Normal {
                    t.palette.surface
                } else {
                    tok.with_alpha(0.2)
                }),
                Some(if sev == Severity::Normal {
                    t.palette.border
                } else {
                    tok
                }),
                false,
                false,
            );
            shaped(&t.shape, style)
                .w(px(84.0))
                .p(px(t.spacing.xs))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(name),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_lg))
                        .font_family(first_family(t.typography.mono))
                        .text_color(if matches!(sev, Severity::Normal | Severity::Caution) {
                            hsla(t.palette.text)
                        } else {
                            severity_color(&t, sev)
                        })
                        .child(readout(v, 1, "")),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(self.unit.clone()),
                )
        };
        let body = ShapeStyle::from_tokens(&t, None, Some(t.palette.border), false, false);
        let [fl, fr, rl, rr] = self.values;
        div()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.lg))
                    .child(chip("FL", fl))
                    .child(chip("RL", rl)),
            )
            .child(shaped(&t.shape, body).w(px(54.0)).h(px(150.0)))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.lg))
                    .child(chip("FR", fr))
                    .child(chip("RR", rr)),
            )
    }
}

// ───────────────────────────────── GearIndicator ─────────────────────────────────

/// A gear selector strip: the engaged position is lit.
#[derive(IntoElement)]
pub struct GearIndicator {
    gears: Vec<SharedString>,
    selected: usize,
}

impl GearIndicator {
    /// A selector over `gears` with `selected` engaged.
    pub fn new<S: Into<SharedString>>(gears: impl IntoIterator<Item = S>, selected: usize) -> Self {
        Self {
            gears: gears.into_iter().map(Into::into).collect(),
            selected,
        }
    }
    /// The usual automatic: P R N D.
    pub fn automatic(selected: usize) -> Self {
        Self::new(["P", "R", "N", "D"], selected)
    }
}

impl RenderOnce for GearIndicator {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let mut row = div().flex().gap(px(t.spacing.xs));
        for (i, g) in self.gears.into_iter().enumerate() {
            let on = i == self.selected;
            let style = ShapeStyle::from_tokens(
                &t,
                Some(if on {
                    t.palette.primary
                } else {
                    t.palette.surface
                }),
                Some(if on {
                    t.palette.primary
                } else {
                    t.palette.border
                }),
                false,
                false,
            );
            row = row.child(
                shaped(&t.shape, style)
                    .w(px(t.shape.control_height))
                    .h(px(t.shape.control_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_lg))
                    .text_color(if on {
                        hsla(t.palette.on_primary)
                    } else {
                        hsla(t.palette.text_muted)
                    })
                    .child(g),
            );
        }
        row
    }
}

// ───────────────────────────────── Odometer ─────────────────────────────────

const CELL_H: f32 = 34.0;

/// A column of digits 0–9 and a repeated 0, shifted so digit position `p` shows in the cell.
fn digit_strip(p: f32) -> gpui::Div {
    let mut col = div().absolute().left(px(0.0)).top(px(-p * CELL_H)).w_full();
    for d in 0..=10 {
        col = col.child(
            div()
                .h(px(CELL_H))
                .flex()
                .items_center()
                .justify_center()
                .child(format!("{}", d % 10)),
        );
    }
    col
}

/// A row of digit cells: odometer, trip counter or any fixed-width number.
#[derive(IntoElement)]
pub struct Odometer {
    follow: Follow,
    value: f32,
    digits: usize,
    decimals: usize,
    unit: SharedString,
}

impl Odometer {
    /// A counter showing `value` in `digits` cells. A changed value rolls the digits like a
    /// mechanical counter: a digit moves one step when the one to its right wraps.
    #[track_caller]
    pub fn new(value: f32, digits: usize) -> Self {
        Self {
            follow: Follow::here(),
            value,
            digits: digits.max(1),
            decimals: 0,
            unit: SharedString::default(),
        }
    }
    follow_api!();
    /// Decimal cells after the point (included in `digits`).
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Unit after the cells.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
}

impl RenderOnce for Odometer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let scaled = self.value.max(0.0) * 10f32.powi(self.decimals as i32);
        let eased = follow_all(window, cx, &self.follow, &[scaled]);
        let digits = self.digits;
        // Most significant digit first.
        let mut pos = odometer_positions(eased.first().copied().unwrap_or(0.0), digits);
        pos.reverse();
        let text = pos;
        let split = text.len().saturating_sub(self.decimals);
        let mut row = div().flex().items_end().gap(px(2.0));
        for (i, p) in text.iter().enumerate() {
            if i == split && self.decimals > 0 {
                row = row.child(
                    div()
                        .text_size(px(t.typography.size_lg))
                        .text_color(hsla(t.palette.text))
                        .child("."),
                );
            }
            let style = ShapeStyle::from_tokens(
                &t,
                Some(if i >= split && self.decimals > 0 {
                    t.palette.primary.with_alpha(0.25)
                } else {
                    t.palette.surface
                }),
                Some(t.palette.border),
                false,
                false,
            );
            row = row.child(
                shaped(&t.shape, style)
                    .relative()
                    .overflow_hidden()
                    .w(px(22.0))
                    .h(px(CELL_H))
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_lg))
                    .text_color(hsla(t.palette.text))
                    .child(digit_strip(*p)),
            );
        }
        if !self.unit.is_empty() {
            row = row.child(
                div()
                    .pl(px(t.spacing.xs))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(self.unit),
            );
        }
        row
    }
}

// ───────────────────────────────── PowerMeter ─────────────────────────────────

/// A bipolar bar around zero: drive power to the right, regeneration to the left.
#[derive(IntoElement)]
pub struct PowerMeter {
    follow: Follow,
    scale: Scale,
    value: f32,
    unit: SharedString,
    width: f32,
}

impl PowerMeter {
    follow_api!();
    /// `scale` should span negative to positive, e.g. `Scale::new(-60.0, 120.0)`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            unit: "kW".into(),
            width: 320.0,
        }
    }
    /// Unit after the value.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(120.0);
        self
    }
}

impl RenderOnce for PowerMeter {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let w = self.width;
        let h = 22.0;
        let zero = self.scale.fraction(0.0);
        let f = self.scale.fraction(self.value);
        let (lo, hi) = (zero.min(f), zero.max(f));
        let sev = self.scale.severity_at(self.value);
        let col = if self.value < 0.0 {
            hsla(t.palette.success)
        } else if sev == Severity::Normal {
            hsla(t.palette.primary)
        } else {
            severity_color(&t, sev)
        };
        let track = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.surface),
            Some(t.palette.border),
            false,
            false,
        );
        let muted = hsla(t.palette.text_muted);
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w(px(w))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child("REGEN")
                    .child(
                        div()
                            .font_family(first_family(t.typography.mono))
                            .text_size(px(t.typography.size_md))
                            .text_color(hsla(t.palette.text))
                            .child(readout(self.value, 0, &self.unit)),
                    )
                    .child("POWER"),
            )
            .child(
                div()
                    .relative()
                    .w(px(w))
                    .h(px(h))
                    .child(shaped(&t.shape, track).size_full())
                    .child(
                        div()
                            .absolute()
                            .top(px(3.0))
                            .left(px(3.0 + (w - 6.0) * lo))
                            .w(px(((w - 6.0) * (hi - lo)).max(0.0)))
                            .h(px(h - 6.0))
                            .bg(col),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(1.0))
                            .left(px(3.0 + (w - 6.0) * zero))
                            .w(px(2.0))
                            .h(px(h - 2.0))
                            .bg(hsla(t.palette.text)),
                    ),
            )
    }
}

// ───────────────────────────────── Compass ─────────────────────────────────

/// A heading ring that turns under a fixed pointer, with the bearing and cardinal letter.
#[derive(IntoElement)]
pub struct Compass {
    follow: Follow,
    heading: f32,
    size: f32,
}

impl Compass {
    follow_api!();
    /// A compass at `heading` degrees (0 = north, clockwise).
    #[track_caller]
    pub fn new(heading: f32) -> Self {
        Self {
            follow: Follow::here(),
            heading: heading.rem_euclid(360.0),
            size: 170.0,
        }
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(90.0);
        self
    }
}

impl RenderOnce for Compass {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.heading = follow_angle(w, cx, &self.follow, 0, self.heading);
        let t = theme_of(cx);
        let (size, heading) = (self.size, self.heading);
        let c = size / 2.0;
        let r = c * 0.92;
        let (muted, ink, accent) = (
            hsla(t.palette.text_muted),
            hsla(t.palette.text),
            hsla(t.palette.primary),
        );
        let ring = hsla(t.palette.border);
        let turn = heading / 360.0;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let circle: Vec<(f32, f32)> =
                    (0..=96).map(|i| polar(c, c, r, i as f32 / 96.0)).collect();
                stroke(w, b.origin, &circle, 2.0, ring, true);
                for i in 0..36 {
                    let f = i as f32 / 36.0 - turn;
                    let len = if i % 3 == 0 { 0.1 } else { 0.05 };
                    stroke(
                        w,
                        b.origin,
                        &[polar(c, c, r, f), polar(c, c, r - size * len, f)],
                        if i % 9 == 0 { 2.0 } else { 1.0 },
                        muted,
                        false,
                    );
                }
                let tri = [
                    (c, c - r + size * 0.17),
                    (c - size * 0.04, c - r - size * 0.02),
                    (c + size * 0.04, c - r - size * 0.02),
                ];
                fill(w, b.origin, &tri, accent);
            },
        )
        .absolute()
        .size_full();
        let mut dial = div().relative().w(px(size)).h(px(size)).child(cv);
        for (name, deg, hot) in [
            ("N", 0.0, true),
            ("E", 90.0, false),
            ("S", 180.0, false),
            ("W", 270.0, false),
        ] {
            let (x, y) = polar(c, c, r - size * 0.17, deg / 360.0 - turn);
            dial = dial.child(
                div()
                    .absolute()
                    .left(px(x - 10.0))
                    .top(px(y - 9.0))
                    .w(px(20.0))
                    .flex()
                    .justify_center()
                    .text_size(px(t.typography.size_md))
                    .text_color(if hot { hsla(t.palette.danger) } else { ink })
                    .child(name),
            );
        }
        let names = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
        let card = names[((heading + 22.5) / 45.0) as usize % 8];
        dial.child(
            div()
                .absolute()
                .left_0()
                .top(px(c - size * 0.1))
                .w(px(size))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .font_family(first_family(t.typography.mono))
                        .text_size(px(t.typography.size_lg))
                        .text_color(ink)
                        .child(format!("{:03.0}°", heading)),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(muted)
                        .child(card),
                ),
        )
    }
}
