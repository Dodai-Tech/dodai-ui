//! Two single-piece cluster designs: a hexagonal speedometer with radiating hairlines and a
//! capsule cluster with a gradient sweep, side arcs and a gear selector.

use crate::charts::{polar, turn_arc};
use crate::gauges::{disc, SWEEP};
use crate::instruments::severity_color;
use crate::paint::{fill, scale_alpha, stroke};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_charts_core::instrument::{readout, Scale, Severity};
use gpui::{
    canvas, div, px, App, Bounds, IntoElement, ParentElement, Pixels, RenderOnce, SharedString,
    Styled, Window,
};

fn num_box(x: f32, y: f32, w: f32) -> gpui::Div {
    div()
        .absolute()
        .left(px(x - w / 2.0))
        .top(px(y))
        .w(px(w))
        .flex()
        .justify_center()
}

// ───────────────────────────────── CutSpeedometer ─────────────────────────────────

/// A speedometer in a cut-corner bezel: hairline ticks radiate from the centre, the lit ones
/// follow the value, and a big thin number sits in the middle.
#[derive(IntoElement)]
pub struct CutSpeedometer {
    scale: Scale,
    value: f32,
    unit: SharedString,
    width: f32,
}

impl CutSpeedometer {
    /// A hex speedometer for `value` on `scale`.
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            scale,
            value,
            unit: SharedString::default(),
            width: 400.0,
        }
    }
    /// Unit label above the number.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Width in px; the height follows.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(240.0);
        self
    }
}

impl RenderOnce for CutSpeedometer {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let Self {
            scale,
            value,
            unit,
            width,
        } = self;
        let h = width * 0.9;
        let (cxp, cyp) = (width / 2.0, h / 2.0);
        let rh = width * 0.5 - 1.0;
        let rd = width * 0.33;
        let frac = scale.fraction(value);
        let sev = scale.severity_at(value);
        let (ink, muted, face, rim) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.background),
            hsla(t.palette.border),
        );
        let lit = if sev == Severity::Normal {
            hsla(t.palette.primary)
        } else {
            severity_color(&t, sev)
        };
        let bezel = t.shape.border_width.max(1.5);
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        // The Cyberpunk shape: a rectangle with cut corners, `r` insets it.
        let hex = move |r: f32| -> Vec<(f32, f32)> {
            let inset = 1.0 - r / rh;
            let (hw, hh) = (
                width / 2.0 - inset * width * 0.5 + 1.0,
                h / 2.0 - inset * h * 0.5 + 1.0,
            );
            let k = width * 0.14 * (1.0 - inset);
            vec![
                (cxp - hw + k, cyp - hh),
                (cxp + hw - k, cyp - hh),
                (cxp + hw, cyp - hh + k),
                (cxp + hw, cyp + hh - k),
                (cxp + hw - k, cyp + hh),
                (cxp - hw + k, cyp + hh),
                (cxp - hw, cyp + hh - k),
                (cxp - hw, cyp - hh + k),
            ]
        };
        let majors: Vec<(f32, f32)> = scale
            .ticks()
            .into_iter()
            .filter(|k| k.major)
            .map(|k| (k.frac, k.value))
            .collect();
        let fine = 100usize;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                fill(w, o, &hex(rh), face);
                stroke(w, o, &hex(rh), bezel, rim, true);
                stroke(w, o, &hex(rh * 0.95), 1.0, scale_alpha(rim, 0.5), true);
                for i in 0..=fine {
                    let f = i as f32 / fine as f32;
                    let on = f <= frac;
                    let col = if on { lit } else { ink };
                    let (a_in, a_out) = if on { (0.25, 0.9) } else { (0.08, 0.3) };
                    let mid = rd * 0.78;
                    let inner = [
                        polar(cxp, cyp, rd * 0.5, at(f)),
                        polar(cxp, cyp, mid, at(f)),
                    ];
                    let outer = [polar(cxp, cyp, mid, at(f)), polar(cxp, cyp, rd, at(f))];
                    stroke(w, o, &inner, 1.0, scale_alpha(col, a_in), false);
                    stroke(w, o, &outer, 1.0, scale_alpha(col, a_out), false);
                }
                for (f, _) in &majors {
                    let seg = [
                        polar(cxp, cyp, rd, at(*f)),
                        polar(cxp, cyp, rd * 1.1, at(*f)),
                    ];
                    stroke(w, o, &seg, 2.0, muted, false);
                }
                let needle = [
                    polar(cxp, cyp, rd * 0.62, at(frac)),
                    polar(cxp, cyp, rd * 1.14, at(frac)),
                ];
                stroke(w, o, &needle, 3.5, lit, false);
            },
        )
        .absolute()
        .size_full();
        let mut hexw = div().relative().w(px(width)).h(px(h)).child(cv);
        for (f, v) in majors_labels(&scale) {
            let (x, y) = polar(cxp, cyp, rd * 1.27, at(f));
            hexw = hexw.child(
                num_box(x, y - 7.0, 40.0)
                    .text_size(px(t.typography.size_xs))
                    .text_color(muted)
                    .child(readout(v, 0, "")),
            );
        }
        let big = width * 0.26;
        hexw.child(
            div()
                .absolute()
                .left_0()
                .top(px(cyp - big * 0.95))
                .w(px(width))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(ink)
                        .child(format_label(&t.typography, &unit)),
                )
                .child(
                    div()
                        .font_family(first_family(t.typography.display))
                        .text_size(px(big))
                        .line_height(px(big * 1.1))
                        .text_color(ink)
                        .child(readout(value, 0, "")),
                ),
        )
    }
}

fn majors_labels(scale: &Scale) -> Vec<(f32, f32)> {
    scale
        .ticks()
        .into_iter()
        .filter(|k| k.major)
        .map(|k| (k.frac, k.value))
        .collect()
}

// ───────────────────────────────── CapsuleCluster ─────────────────────────────────

/// What a [`CapsuleCluster`] shows around its speed dial.
#[derive(IntoElement)]
pub struct CapsuleCluster {
    scale: Scale,
    value: f32,
    mode: SharedString,
    gear: char,
    power: f32,
    battery: f32,
    range_km: f32,
    temp: f32,
    width: f32,
}

impl CapsuleCluster {
    /// A capsule cluster for speed `value` on `scale`.
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            scale,
            value,
            mode: SharedString::default(),
            gear: 'D',
            power: 18.0,
            battery: 70.0,
            range_km: 460.0,
            temp: 22.0,
            width: 760.0,
        }
    }
    /// Drive-mode label under the speed.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = m.into();
        self
    }
    /// Selected gear letter: `P`, `R`, `N` or `D`.
    pub fn gear(mut self, g: char) -> Self {
        self.gear = g;
        self
    }
    /// Power in kW (left arc, 0..100).
    pub fn power(mut self, kw: f32) -> Self {
        self.power = kw;
        self
    }
    /// Battery charge in % and the range in km.
    pub fn battery(mut self, percent: f32, range_km: f32) -> Self {
        self.battery = percent;
        self.range_km = range_km;
        self
    }
    /// Temperature in °C (right arc, 0..40).
    pub fn temp(mut self, c: f32) -> Self {
        self.temp = c;
        self
    }
    /// Width in px; the height follows.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(480.0);
        self
    }
}

impl RenderOnce for CapsuleCluster {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let width = self.width;
        let h = width * 0.4;
        let dia = h * 0.9;
        let (mx, my) = (width / 2.0, h / 2.0);
        let c = dia / 2.0;
        let frac = self.scale.fraction(self.value);
        let sev = self.scale.severity_at(self.value);
        let (ink, muted, track) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.progress(None).track),
        );
        let rim = hsla(t.palette.border);
        let a0 = t.palette.primary;
        let a1 = t.palette.secondary;
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let band = (dia * 0.075).max(t.sizes().slider_track);
        let r_band = c * 0.9;
        let (power, temp, battery) = (
            (self.power / 100.0).clamp(0.0, 1.0),
            (self.temp / 40.0).clamp(0.0, 1.0),
            (self.battery / 100.0).clamp(0.0, 1.0),
        );
        let ticks: Vec<(f32, bool)> = self
            .scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let alarm = if sev == Severity::Normal {
            None
        } else {
            Some(severity_color(&t, sev))
        };
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let r = h / 2.0 - 1.0;
                // Stadium outline: left cap, top, right cap, bottom.
                let mut pts = Vec::new();
                for i in 0..=32 {
                    let a = std::f32::consts::PI * (0.5 + i as f32 / 32.0);
                    pts.push((r + r * a.cos(), r - r * a.sin()));
                }
                for i in 0..=32 {
                    let a = std::f32::consts::PI * (1.5 + i as f32 / 32.0);
                    pts.push((width - r + r * a.cos(), r - r * a.sin()));
                }
                stroke(w, o, &pts, 2.0, rim, true);
                // Gradient rim glow on the lower edge.
                for i in 0..40 {
                    let f = i as f32 / 40.0;
                    let x0 = r + (width - 2.0 * r) * f;
                    let x1 = r + (width - 2.0 * r) * (f + 1.0 / 40.0);
                    let col = hsla(a0.mix(a1, f));
                    stroke(
                        w,
                        o,
                        &[(x0, h - 1.5), (x1, h - 1.5)],
                        2.0,
                        scale_alpha(col, 0.8),
                        false,
                    );
                }
                // Side arcs on the end caps.
                let ar = r - 14.0;
                let left = |f: f32| 0.55 + 0.4 * f;
                let right = |f: f32| 0.45 - 0.4 * f;
                stroke(
                    w,
                    o,
                    &turn_arc(r, r, ar, left(0.0), left(1.0)),
                    3.0,
                    track,
                    false,
                );
                stroke(
                    w,
                    o,
                    &turn_arc(r, r, ar, left(0.0), left(power)),
                    3.0,
                    hsla(a0),
                    false,
                );
                let (rx, ry) = (width - r, r);
                stroke(
                    w,
                    o,
                    &turn_arc(rx, ry, ar, right(1.0), right(0.0)),
                    3.0,
                    track,
                    false,
                );
                stroke(
                    w,
                    o,
                    &turn_arc(rx, ry, ar, right(temp), right(0.0)),
                    3.0,
                    hsla(a1),
                    false,
                );
                // Speed dial: track, gradient value, ticks.
                let o2 = gpui::point(o.x + px(mx - c), o.y + px(my - c));
                stroke(
                    w,
                    o2,
                    &turn_arc(c, c, r_band, at(0.0), at(1.0)),
                    band,
                    track,
                    false,
                );
                let n = 48usize;
                for i in 0..n {
                    let f0 = frac * i as f32 / n as f32;
                    let f1 = frac * (i + 1) as f32 / n as f32;
                    let col = alarm.unwrap_or_else(|| hsla(a0.mix(a1, i as f32 / n as f32)));
                    stroke(
                        w,
                        o2,
                        &turn_arc(c, c, r_band, at(f0), at(f1)),
                        band,
                        col,
                        false,
                    );
                }
                let knob = polar(c, c, r_band, at(frac));
                fill(w, o2, &disc(knob.0, knob.1, band * 0.7, 20), ink);
                for (f, major) in &ticks {
                    let len = if *major { dia * 0.04 } else { dia * 0.02 };
                    let ro = r_band - band * 0.5 - 3.0;
                    let seg = [polar(c, c, ro, at(*f)), polar(c, c, ro - len, at(*f))];
                    stroke(w, o2, &seg, if *major { 1.8 } else { 1.0 }, ink, false);
                }
            },
        )
        .absolute()
        .size_full();
        let mut cap = div().relative().w(px(width)).h(px(h)).child(cv);
        for k in self.scale.ticks().into_iter().filter(|k| k.major) {
            let (x, y) = polar(c, c, c * 0.68, at(k.frac));
            cap = cap.child(
                num_box(mx - c + x, my - c + y - 7.0, 36.0)
                    .text_size(px(t.typography.size_xs))
                    .text_color(ink)
                    .child(readout(k.value, 0, "")),
            );
        }
        let big = dia * 0.2;
        cap = cap.child(
            div()
                .absolute()
                .left(px(mx - c))
                .top(px(my - big * 0.9))
                .w(px(dia))
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .font_family(first_family(t.typography.display))
                        .text_size(px(big))
                        .line_height(px(big * 1.05))
                        .text_color(alarm.unwrap_or(ink))
                        .child(readout(self.value, 0, "")),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(muted)
                        .child("km/h"),
                )
                .child(
                    div()
                        .pt(px(t.spacing.xs))
                        .text_size(px(t.typography.size_md))
                        .text_color(hsla(a1))
                        .child(format_label(&t.typography, &self.mode)),
                ),
        );
        // Gear selector under the dial.
        let mut gears = div().flex().gap(px(t.spacing.md));
        for g in ['P', 'R', 'N', 'D'] {
            let on = g == self.gear;
            gears = gears.child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_lg))
                    .text_color(if on { ink } else { muted })
                    .child(g.to_string()),
            );
        }
        cap = cap.child(
            div()
                .absolute()
                .left(px(mx - 80.0))
                .top(px(h - 34.0))
                .w(px(160.0))
                .flex()
                .justify_center()
                .child(gears),
        );
        // Side readouts: power, battery and range, temperature.
        let side = |x: f32, align_right: bool| {
            let d = div()
                .absolute()
                .top(px(my - 14.0))
                .w(px(110.0))
                .flex()
                .flex_col()
                .text_size(px(t.typography.size_sm))
                .text_color(ink);
            if align_right {
                d.left(px(x - 110.0)).items_end()
            } else {
                d.left(px(x)).items_start()
            }
        };
        let (lx, rx) = (h * 0.27, width - h * 0.27);
        cap = cap
            .child(side(lx, false).child(readout(self.power, 0, " kW")).child(
                div().text_color(muted).child(format!(
                    "{:.0}% · {:.0} km",
                    battery * 100.0,
                    self.range_km
                )),
            ))
            .child(
                side(rx, true)
                    .child(readout(self.temp, 0, " °C"))
                    .child(div().text_color(muted).child("Outside")),
            );
        cap
    }
}
