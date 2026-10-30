//! Flight and sensor instruments: an [`AttitudeIndicator`], a [`HeadingTape`], a
//! [`VerticalTape`] for speed and altitude, a [`SonarWaterfall`] and a [`Countermeasures`]
//! counter. Geometry comes from `dodai_charts_core::flight`; this file only paints it.

use crate::dial::severity_color;
use crate::draw::polar;
use crate::scope::{clock, frame};
use crate::segment::SegmentDisplay;
use dodai_charts_core::flight;
use dodai_charts_core::instrument::Severity;
use dodai_core::Color;
use dodai_gpui::follow::{follow, follow_angle, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::paint::{fill, fill_batch, scale_alpha, stroke, stroke_batch};
use dodai_gpui::text::{first_family, format_label};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, prelude::FluentBuilder, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
};

/// An artificial horizon: sky and ground, pitch ladder, roll scale and a fixed aircraft symbol.
#[derive(IntoElement)]
pub struct AttitudeIndicator {
    follow: Follow,
    pitch: f32,
    roll: f32,
    size: f32,
    caption: Option<SharedString>,
}

impl AttitudeIndicator {
    follow_api!();
    /// A horizon at `pitch` degrees (positive = nose up) and `roll` degrees (positive = right
    /// wing down).
    #[track_caller]
    pub fn new(pitch: f32, roll: f32) -> Self {
        Self {
            follow: Follow::here(),
            pitch,
            roll,
            size: 240.0,
            caption: None,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

impl RenderOnce for AttitudeIndicator {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.pitch = follow(w, cx, &self.follow, 0, self.pitch);
        self.roll = follow(w, cx, &self.follow, 1, self.roll);
        let t = theme_of(cx);
        let (size, pitch, roll) = (self.size, self.pitch, self.roll);
        let sky = scale_alpha(hsla(t.palette.info), 0.5);
        let ground = scale_alpha(hsla(t.palette.warning), 0.4);
        let ink = hsla(t.palette.text);
        let own = hsla(t.palette.primary);
        let rim = hsla(t.palette.border);
        let (c, r) = (size / 2.0, size / 2.0 - 10.0);
        let ppd = r / 28.0;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let half = flight::horizon_halves(c, c, r, pitch, roll, ppd);
                fill(w, o, &half.sky, sky);
                fill(w, o, &half.ground, ground);
                let h = flight::horizon_center(c, c, pitch, roll, ppd);
                let a = roll.to_radians();
                let d = (a.cos() * r, -a.sin() * r);
                stroke(
                    w,
                    o,
                    &[(h.0 - d.0, h.1 - d.1), (h.0 + d.0, h.1 + d.1)],
                    2.0,
                    ink,
                    false,
                );
                let rungs: Vec<Vec<(f32, f32)>> = flight::ladder(c, c, r, pitch, roll, ppd)
                    .iter()
                    .map(|k| vec![k.a, k.b])
                    .collect();
                stroke_batch(w, o, &rungs, 1.5, scale_alpha(ink, 0.9), false);
                stroke(w, o, &crate::gauges::disc(c, c, r, 72), 2.0, rim, true);
                // Roll scale on the rim, fixed to the airframe, with a pointer for the roll.
                let ticks: Vec<Vec<(f32, f32)>> = flight::roll_ticks()
                    .iter()
                    .map(|(turn, major)| {
                        let len = if *major { 0.1 } else { 0.06 };
                        vec![polar(c, c, r, *turn), polar(c, c, r * (1.0 - len), *turn)]
                    })
                    .collect();
                stroke_batch(w, o, &ticks, 1.5, ink, false);
                let tip = polar(c, c, r * 0.9, roll / 360.0);
                let l = polar(c, c, r * 0.8, roll / 360.0 - 0.012);
                let rr = polar(c, c, r * 0.8, roll / 360.0 + 0.012);
                fill(w, o, &[tip, l, rr], own);
                // The aircraft.
                let wing = r * 0.32;
                stroke_batch(
                    w,
                    o,
                    &[
                        vec![
                            (c - wing, c),
                            (c - wing * 0.35, c),
                            (c - wing * 0.35, c + 5.0),
                        ],
                        vec![
                            (c + wing, c),
                            (c + wing * 0.35, c),
                            (c + wing * 0.35, c + 5.0),
                        ],
                    ],
                    3.5,
                    own,
                    false,
                );
                fill(
                    w,
                    o,
                    &[
                        (c - 3.0, c - 3.0),
                        (c + 3.0, c - 3.0),
                        (c + 3.0, c + 3.0),
                        (c - 3.0, c + 3.0),
                    ],
                    own,
                );
            },
        )
        .absolute()
        .size_full();
        let muted = hsla(t.palette.text_muted);
        let mut body = div().relative().size_full().child(cv);
        for k in flight::ladder(c, c, r, pitch, roll, ppd)
            .iter()
            .filter(|k| k.major)
        {
            body = body.child(
                div()
                    .absolute()
                    .left(px(k.b.0 + 3.0))
                    .top(px(k.b.1 - 7.0))
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(k.pitch.abs().to_string()),
            );
        }
        frame(&t, size, size, self.caption, body)
    }
}

/// A heading tape: a scrolling compass strip with a fixed pointer and an optional target bug.
#[derive(IntoElement)]
pub struct HeadingTape {
    follow: Follow,
    heading: f32,
    bug: Option<f32>,
    width: f32,
    span: f32,
}

impl HeadingTape {
    follow_api!();
    /// A tape centred on `heading` degrees.
    #[track_caller]
    pub fn new(heading: f32) -> Self {
        Self {
            follow: Follow::here(),
            heading,
            bug: None,
            width: 420.0,
            span: 60.0,
        }
    }
    /// A target heading shown as a bug on the tape.
    pub fn bug(mut self, deg: f32) -> Self {
        self.bug = Some(deg);
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl RenderOnce for HeadingTape {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.heading = follow_angle(w, cx, &self.follow, 0, self.heading);
        let t = theme_of(cx);
        let (width, span, heading) = (self.width, self.span, flight::wrap_heading(self.heading));
        let h = 46.0;
        let marks = flight::heading_marks(heading, span, 5.0, 2);
        let ink = hsla(t.palette.text);
        let own = hsla(t.palette.primary);
        let warn = hsla(t.palette.warning);
        let bug = self.bug.map(|b| {
            let mut diff = (flight::wrap_heading(b) - heading + 180.0).rem_euclid(360.0) - 180.0;
            diff = diff.clamp(-span / 2.0, span / 2.0);
            diff / span
        });
        let m2 = marks.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let ticks: Vec<Vec<(f32, f32)>> = m2
                    .iter()
                    .map(|m| {
                        let x = width / 2.0 + m.offset * width;
                        vec![(x, if m.major { 16.0 } else { 22.0 }), (x, 30.0)]
                    })
                    .collect();
                stroke_batch(w, o, &ticks, 1.5, ink, false);
                stroke(
                    w,
                    o,
                    &[(0.0, 30.5), (width, 30.5)],
                    1.0,
                    scale_alpha(ink, 0.5),
                    false,
                );
                let x = width / 2.0;
                fill(w, o, &[(x - 6.0, 46.0), (x + 6.0, 46.0), (x, 34.0)], own);
                if let Some(off) = bug {
                    let x = width / 2.0 + off * width;
                    fill(w, o, &[(x - 6.0, 0.0), (x + 6.0, 0.0), (x, 10.0)], warn);
                }
            },
        )
        .absolute()
        .size_full();
        let mut body = div().relative().size_full().child(cv);
        for m in marks.iter().filter(|m| m.major) {
            let x = width / 2.0 + m.offset * width;
            body = body.child(
                div()
                    .absolute()
                    .left(px(x - 14.0))
                    .top(px(0.0))
                    .w(px(28.0))
                    .flex()
                    .justify_center()
                    .text_size(px(t.typography.size_xs))
                    .text_color(ink)
                    .child(flight::heading_label(m.value)),
            );
        }
        let readout = div().flex().justify_center().mt(px(t.spacing.xs)).child(
            div()
                .px(px(t.spacing.sm))
                .border_1()
                .border_color(own)
                .text_color(ink)
                .text_size(px(t.typography.size_sm))
                .font_family(first_family(t.typography.mono))
                .child(format!("{:03.0}°", heading)),
        );
        frame(&t, width, h, None, body).child(readout)
    }
}

/// A vertical scrolling tape with a boxed readout, for speed and altitude.
#[derive(IntoElement)]
pub struct VerticalTape {
    follow: Follow,
    value: f32,
    span: f32,
    step: f32,
    major_every: u32,
    label: SharedString,
    height: f32,
    right: bool,
}

impl VerticalTape {
    follow_api!();
    /// A tape centred on `value`.
    #[track_caller]
    pub fn new(value: f32) -> Self {
        Self {
            follow: Follow::here(),
            value,
            span: 120.0,
            step: 10.0,
            major_every: 2,
            label: "".into(),
            height: 240.0,
            right: false,
        }
    }
    /// Units visible on the tape and the distance between marks.
    pub fn scale(mut self, span: f32, step: f32, major_every: u32) -> Self {
        self.span = span;
        self.step = step;
        self.major_every = major_every;
        self
    }
    /// Caption (for example `KT`).
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// Puts the ticks on the right, as for an altitude tape.
    pub fn ticks_right(mut self, on: bool) -> Self {
        self.right = on;
        self
    }
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl RenderOnce for VerticalTape {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let (width, height, right) = (84.0, self.height, self.right);
        let marks = flight::tape_marks(self.value, self.span, self.step, self.major_every);
        let ink = hsla(t.palette.text);
        let own = hsla(t.palette.primary);
        let m2 = marks.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let edge = if right { 0.0 } else { width };
                let ticks: Vec<Vec<(f32, f32)>> = m2
                    .iter()
                    .map(|m| {
                        let y = height / 2.0 - m.offset * height;
                        let len = if m.major { 14.0 } else { 8.0 };
                        let dir = if right { 1.0 } else { -1.0 };
                        vec![(edge, y), (edge + dir * len, y)]
                    })
                    .collect();
                stroke_batch(w, o, &ticks, 1.5, ink, false);
                stroke(
                    w,
                    o,
                    &[(edge, 0.0), (edge, height)],
                    1.0,
                    scale_alpha(ink, 0.5),
                    false,
                );
                let y = height / 2.0;
                let (x0, x1) = if right {
                    (edge, edge + 10.0)
                } else {
                    (edge - 10.0, edge)
                };
                let tip = if right { x1 } else { x0 };
                fill(w, o, &[(edge, y), (tip, y - 6.0), (tip, y + 6.0)], own);
            },
        )
        .absolute()
        .size_full();
        let mut body = div().relative().size_full().child(cv);
        for m in marks.iter().filter(|m| m.major) {
            let y = height / 2.0 - m.offset * height;
            let (l, w_) = if right { (22.0, 60.0) } else { (0.0, 60.0) };
            body = body.child(
                div()
                    .absolute()
                    .left(px(l))
                    .top(px(y - 7.0))
                    .w(px(w_))
                    .flex()
                    .when(!right, |d| d.justify_end())
                    .text_size(px(t.typography.size_xs))
                    .text_color(ink)
                    .child(format!("{:.0}", m.value)),
            );
        }
        let readout = div().mt(px(t.spacing.xs)).flex().justify_center().child(
            div()
                .px(px(t.spacing.sm))
                .border_1()
                .border_color(own)
                .text_color(ink)
                .text_size(px(t.typography.size_md))
                .font_family(first_family(t.typography.mono))
                .child(format!("{:.0} {}", self.value, self.label)),
        );
        frame(&t, width, height, None, body).child(readout)
    }
}

/// A scrolling sonar waterfall: frequency across, time down, brighter means louder.
#[derive(IntoElement)]
pub struct SonarWaterfall {
    bins: usize,
    rows: usize,
    size: (f32, f32),
    caption: Option<SharedString>,
}

impl SonarWaterfall {
    /// A live waterfall of synthetic contacts (no data source yet): `bins` columns, `rows` lines.
    pub fn new(bins: usize, rows: usize) -> Self {
        Self {
            bins: bins.max(8),
            rows: rows.max(8),
            size: (360.0, 220.0),
            caption: None,
        }
    }
    /// Size in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.size = (w, h);
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
}

/// Colour ramp of the waterfall: background, primary, warning and danger.
fn ramp(stops: &[Color; 4], v: f32) -> Hsla {
    let v = v.clamp(0.0, 1.0) * 3.0;
    let i = (v.floor() as usize).min(2);
    hsla(stops[i].mix(stops[i + 1], v - i as f32))
}

impl RenderOnce for SonarWaterfall {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let ((w_, h_), bins, rows) = (self.size, self.bins, self.rows);
        let stops = [
            t.palette.background,
            t.palette.primary.with_alpha(0.6),
            t.palette.warning,
            t.palette.danger,
        ];
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let (cw, ch) = (bw / bins as f32, bh / rows as f32);
                let now = clock();
                // Quantise to the row rate so the picture scrolls in whole rows.
                let tick = (now * 8.0).floor() / 8.0;
                for r in 0..rows {
                    let row = flight::waterfall_row(tick - r as f32 / 8.0, bins);
                    // Cells of near-equal colour share one fill per row.
                    let mut buckets: [Vec<Vec<(f32, f32)>>; 6] = Default::default();
                    for (i, v) in row.iter().enumerate() {
                        let k = (v * 5.0).round() as usize;
                        let (x, y) = (i as f32 * cw, r as f32 * ch);
                        buckets[k].push(vec![
                            (x, y),
                            (x + cw + 0.5, y),
                            (x + cw + 0.5, y + ch + 0.5),
                            (x, y + ch + 0.5),
                        ]);
                    }
                    for (k, polys) in buckets.iter().enumerate() {
                        fill_batch(w, b.origin, polys, ramp(&stops, k as f32 / 5.0));
                    }
                }
                dodai_gpui::follow::next_frame(w);
            },
        )
        .absolute()
        .size_full();
        frame(&t, w_, h_, self.caption, cv)
    }
}

/// Chaff and flare counters in seven-segment digits; low counts turn amber, empty turns red.
#[derive(IntoElement)]
pub struct Countermeasures {
    chaff: u32,
    flare: u32,
    low: u32,
}

impl Countermeasures {
    /// Remaining chaff and flare, warning below `low` (default 10).
    pub fn new(chaff: u32, flare: u32) -> Self {
        Self {
            chaff,
            flare,
            low: 10,
        }
    }
    /// Count below which a counter turns amber.
    pub fn low(mut self, n: u32) -> Self {
        self.low = n;
        self
    }
}

impl RenderOnce for Countermeasures {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let tint = |n: u32| match n {
            0 => severity_color(&t, Severity::Alarm),
            n if n < self.low => severity_color(&t, Severity::Caution),
            _ => hsla(t.palette.primary),
        };
        let one = |name: &str, n: u32| {
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.xs))
                .child(
                    div()
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format_label(&t.typography, name)),
                )
                .child(
                    SegmentDisplay::new(format!("{n:>3}"))
                        .height(40.0)
                        .color(tint(n)),
                )
        };
        div()
            .flex()
            .gap(px(t.spacing.lg))
            .child(one("Chaff", self.chaff))
            .child(one("Flare", self.flare))
    }
}
