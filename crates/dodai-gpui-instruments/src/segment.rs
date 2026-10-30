//! Seven-segment instruments: a [`SegmentDisplay`] for numbers and clocks and a
//! [`SegmentCluster`] that combines speed, engine speed, gear, fuel, oil, coolant and the time in
//! one clock-style panel. Unlit segments stay faintly visible like a real display.

use crate::dial::severity_color;
use dodai_gpui::paint::{fill, scale_alpha, shaped, ShapeStyle};
use dodai_gpui::text::format_label;
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::{Scale, Severity};
use dodai_instruments_core::segments;
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

/// A row of seven-segment digits. `:` and `.` are drawn as dots, other characters as blanks.
#[derive(IntoElement)]
pub struct SegmentDisplay {
    follow: dodai_gpui::follow::Follow,
    fade: bool,
    text: SharedString,
    height: f32,
    color: Option<Hsla>,
    ghost: bool,
}

impl SegmentDisplay {
    dodai_gpui::follow_api!();
    /// Segments cross-fade when the text changes instead of switching. Displays built at one
    /// call site are told apart by their order in the frame.
    pub fn fade(mut self, on: bool) -> Self {
        self.fade = on;
        self
    }
    /// Shows `text` (digits, `-`, `:`, `.` and spaces).
    #[track_caller]
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            fade: false,
            text: text.into(),
            height: 48.0,
            color: None,
            ghost: true,
        }
    }
    /// Digit height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h.max(10.0);
        self
    }
    /// Lit colour (default: the theme's primary).
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
    /// Whether unlit segments stay faintly visible (default on).
    pub fn ghost(mut self, on: bool) -> Self {
        self.ghost = on;
        self
    }
}

impl RenderOnce for SegmentDisplay {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let h = self.height;
        let on = self.color.unwrap_or_else(|| hsla(t.palette.primary));
        let ghost = if t.is_dark() { 0.09 } else { 0.22 };
        let off = scale_alpha(on, if self.ghost { ghost } else { 0.0 });
        let chars: Vec<char> = self.text.chars().collect();
        let width: f32 = chars.iter().map(|c| segments::advance(*c, h)).sum();
        // Lit amount of every segment, eased so a changed digit cross-fades.
        let flags: Vec<f32> = chars
            .iter()
            .flat_map(|c| (0..7).map(move |i| f32::from(segments::mask(*c) >> i & 1 == 1)))
            .collect();
        let flags = if self.fade {
            dodai_gpui::follow::follow_all(window, cx, &self.follow, &flags)
        } else {
            flags
        };
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let th = segments::thickness(h);
                let mut x0 = 0.0;
                for (ci, c) in chars.iter().enumerate() {
                    match c {
                        ':' | '.' => {
                            let s = th * 0.9;
                            let xs = x0 + h * 0.08;
                            let ys: &[f32] = if *c == ':' {
                                &[h * 0.3, h * 0.7]
                            } else {
                                &[h - s]
                            };
                            for y in ys {
                                let sq = [(xs, *y), (xs + s, *y), (xs + s, *y + s), (xs, *y + s)]
                                    .map(|p| segments::slant(p, 0.0, h));
                                fill(w, o, &sq, on);
                            }
                        }
                        _ => {
                            for (i, seg) in segments::digit_segments(h).iter().enumerate() {
                                let pts: Vec<(f32, f32)> =
                                    seg.iter().map(|p| segments::slant(*p, x0, h)).collect();
                                let k = flags
                                    .get(ci * 7 + i)
                                    .copied()
                                    .unwrap_or(0.0)
                                    .clamp(0.0, 1.0);
                                fill(w, o, &pts, dodai_gpui::motion::mix(off, on, k));
                            }
                        }
                    }
                    x0 += segments::advance(*c, h);
                }
            },
        )
        .absolute()
        .size_full();
        div().relative().w(px(width + h * 0.1)).h(px(h)).child(cv)
    }
}

/// A segmented LED bar that lights up to `value`, coloured per severity zone.
fn led_bar(
    t: &dodai_core::Theme,
    scale: &Scale,
    value: f32,
    width: f32,
    height: f32,
) -> impl IntoElement {
    let n = ((width / 8.0) as usize).clamp(10, 80);
    let frac = scale.fraction(value);
    let cols: Vec<Hsla> = (0..n)
        .map(
            |i| match scale.severity_at(scale.value_at((i as f32 + 0.5) / n as f32)) {
                Severity::Normal => hsla(t.palette.primary),
                s => severity_color(t, s),
            },
        )
        .collect();
    let ghost = if t.is_dark() { 0.12 } else { 0.24 };
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| {
            let step = width / n as f32;
            for (i, c) in cols.iter().enumerate() {
                let x = i as f32 * step;
                let lit = (i as f32 + 0.5) / n as f32 <= frac;
                let r = [
                    (x, 0.0),
                    (x + step - 2.0, 0.0),
                    (x + step - 2.0, height),
                    (x, height),
                ];
                fill(
                    w,
                    b.origin,
                    &r,
                    if lit { *c } else { scale_alpha(*c, ghost) },
                );
            }
        },
    )
    .w(px(width))
    .h(px(height))
}

/// Readings for a [`SegmentCluster`].
#[derive(IntoElement)]
pub struct SegmentCluster {
    speed: f32,
    rpm: f32,
    gear: SharedString,
    mode: SharedString,
    fuel: f32,
    oil: f32,
    coolant: f32,
    clock: SharedString,
    width: f32,
}

impl SegmentCluster {
    /// A cluster with plausible cruising values.
    pub fn new() -> Self {
        Self {
            speed: 251.0,
            rpm: 6.9,
            gear: "6".into(),
            mode: "NORMAL".into(),
            fuel: 62.0,
            oil: 4.8,
            coolant: 96.0,
            clock: "14:48".into(),
            width: 760.0,
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
    /// Engaged gear (digits show as a segment digit, letters as text).
    pub fn gear(mut self, g: impl Into<SharedString>) -> Self {
        self.gear = g.into();
        self
    }
    /// Drive mode label.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = m.into();
        self
    }
    /// Fuel in %.
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
    /// The time, e.g. `"14:48"`.
    pub fn clock(mut self, c: impl Into<SharedString>) -> Self {
        self.clock = c.into();
        self
    }
    /// Total width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(480.0);
        self
    }
}

impl Default for SegmentCluster {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for SegmentCluster {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (gap, pad) = (t.spacing.md, t.spacing.md);
        let inner = self.width - pad * 2.0;
        let (muted, ink) = (hsla(t.palette.text_muted), hsla(t.palette.text));
        let rpm = Scale::new(0.0, 8.0)
            .ticks_per(8, 1)
            .zone(7.0, 8.0, Severity::Alarm);
        let fuel_s = Scale::new(0.0, 100.0)
            .zone(0.0, 12.0, Severity::Alarm)
            .zone(12.0, 25.0, Severity::Caution);
        let cool_s = Scale::new(40.0, 140.0).zone(115.0, 140.0, Severity::Alarm);
        let tint = |s: &Scale, v: f32| match s.severity_at(v) {
            Severity::Normal => hsla(t.palette.primary),
            sev => severity_color(&t, sev),
        };
        let label = |s: &str| {
            div()
                .text_size(px(t.typography.size_xs))
                .text_color(muted)
                .child(format_label(&t.typography, s))
        };
        let unit = |s: &str| {
            div()
                .pb(px(2.0))
                .text_size(px(t.typography.size_sm))
                .text_color(muted)
                .child(s.to_string())
        };
        let mini = |name: &str, text: String, u: &str, col: Hsla| {
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(label(name))
                .child(
                    div()
                        .flex()
                        .items_end()
                        .gap(px(t.spacing.xs))
                        .child(SegmentDisplay::new(text).height(34.0).color(col))
                        .child(unit(u)),
                )
        };
        // Top: engine speed bar with its scale and the gear.
        let bar_w = inner - 96.0;
        let rpm_row = div()
            .flex()
            .items_center()
            .gap(px(gap))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .w(px(bar_w))
                            .children((0..=8).map(|i| {
                                div()
                                    .text_size(px(t.typography.size_xs))
                                    .text_color(muted)
                                    .child(i.to_string())
                            })),
                    )
                    .child(led_bar(&t, &rpm, self.rpm, bar_w, 18.0))
                    .child(label("RPM ×1000")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .w(px(80.0))
                    .child(label("GEAR"))
                    .child(
                        SegmentDisplay::new(self.gear.clone())
                            .height(44.0)
                            .color(ink),
                    ),
            );
        let speed = div()
            .flex()
            .flex_col()
            .items_center()
            .child(label(&self.mode))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(t.spacing.sm))
                    .child(
                        SegmentDisplay::new(format!("{:>3.0}", self.speed))
                            .height(inner * 0.2)
                            .color(ink),
                    )
                    .child(unit("km/h")),
            );
        let left = div()
            .flex()
            .flex_col()
            .gap(px(gap))
            .child(mini(
                "FUEL",
                format!("{:>3.0}", self.fuel),
                "%",
                tint(&fuel_s, self.fuel),
            ))
            .child(mini(
                "COOLANT",
                format!("{:>3.0}", self.coolant),
                "°C",
                tint(&cool_s, self.coolant),
            ));
        let right = div()
            .flex()
            .flex_col()
            .gap(px(gap))
            .child(mini(
                "OIL",
                format!("{:.1}", self.oil),
                "bar",
                hsla(t.palette.primary),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .child(label("TIME"))
                    .child(
                        SegmentDisplay::new(self.clock.clone())
                            .height(34.0)
                            .color(muted),
                    ),
            );
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.background),
            Some(t.palette.border),
            false,
            false,
        );
        shaped(&t.shape, style)
            .w(px(self.width))
            .p(px(pad))
            .flex()
            .flex_col()
            .gap(px(gap))
            .child(rpm_row)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(left)
                    .child(speed)
                    .child(right),
            )
    }
}
