//! A radial gauge with a centre label, in any of the [`GaugeStyle`](crate::GaugeStyle) looks.

use crate::draw::*;
use dodai_gpui::paint::stroke;
use dodai_gpui::text::first_family;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

// ─────────────────────────────────── Gauge ───────────────────────────────────

/// Radial gauge with a centre label. [`crate::gauges::GaugeStyle`] picks the look: the
/// default 270° arc, segmented ring, dial, meter, reticle or slanted-cell bar.
#[derive(IntoElement)]
pub struct Gauge {
    follow: dodai_gpui::follow::Follow,
    glow: crate::draw::ChartGlow,
    value: f32,
    min: f32,
    max: f32,
    size: f32,
    label: Option<SharedString>,
    unit: &'static str,
    caption: Option<SharedString>,
    tone: Option<dodai_core::Tone>,
    style: crate::gauges::GaugeStyle,
    zones: bool,
    target: Option<f32>,
}

impl Gauge {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::draw::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`Gauge`].
    #[track_caller]
    pub fn new(value: f32) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            glow: crate::draw::ChartGlow::Theme,
            value,
            min: 0.0,
            max: 100.0,
            size: 120.0,
            label: None,
            unit: "",
            caption: None,
            tone: None,
            style: crate::gauges::GaugeStyle::Arc,
            zones: false,
            target: None,
        }
    }
    /// Value range `min..max`.
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Centre label (default: the rounded value).
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Unit appended to the default label (the rounded value), so it follows a changing value.
    pub fn unit(mut self, u: &'static str) -> Self {
        self.unit = u;
        self
    }
    /// Small caption under the label (uppercased when the theme does).
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    /// Semantic tone (colour).
    pub fn tone(mut self, t: dodai_core::Tone) -> Self {
        self.tone = Some(t);
        self
    }
    /// Visual style.
    pub fn style(mut self, s: crate::gauges::GaugeStyle) -> Self {
        self.style = s;
        self
    }
    /// Target marker at `value` (same range as the gauge).
    pub fn target(mut self, v: f32) -> Self {
        self.target = Some(v);
        self
    }
    /// Colour the value green → amber → red as it climbs (overrides `tone`).
    pub fn zones(mut self, on: bool) -> Self {
        self.zones = on;
        self
    }
}

impl Gauge {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        use crate::gauges::{GaugeStyle, Look};
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow(window, cx, "Gauge");
        let frac = norm(self.value, self.min, self.max) * g;
        let col = hsla(self.tone.map_or(t.palette.primary, |k| t.palette.tone(k)));
        let track = hsla(t.palette.border);
        let glow = self.glow.resolve(&t);
        let size = self.size;
        let style = self.style;
        let (bw, bh) = style.dims(size);
        let ring = (size * 0.1).clamp(4.0, 12.0);
        let label = self.label.unwrap_or_else(|| {
            format!("{}{}", fmt_num((self.value * g).round()), self.unit).into()
        });
        let look = Look {
            style,
            frac,
            size,
            ring,
            tone: col,
            track,
            text: hsla(t.palette.text),
            zones: self.zones.then(|| {
                [
                    hsla(t.palette.success),
                    hsla(t.palette.warning),
                    hsla(t.palette.danger),
                ]
            }),
            glow,
            alert: hsla(t.palette.danger),
            target: self.target.map(|v| norm(v, self.min, self.max)),
            // Cut-corner gauges are Cyberpunk's; every other design gets the round ones.
            soft: t.look().soft_elevation || t.shape.kind() != dodai_core::ShapeKind::Chamfered,
        };
        let soft = look.soft;
        let canvas_el = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                if style != GaugeStyle::Arc {
                    crate::gauges::paint(w, b, look);
                    return;
                }
                let c = size / 2.0;
                let r = c - ring / 2.0 - 2.0;
                let (s0, span) = crate::gauges::SWEEP;
                if soft {
                    crate::gauges::round_arc(w, b.origin, c, r, s0, s0 + span, ring, track);
                    if frac > 0.0 {
                        crate::gauges::round_arc(
                            w,
                            b.origin,
                            c,
                            r,
                            s0,
                            s0 + span * frac,
                            ring,
                            col,
                        );
                    }
                    crate::gauges::paint(w, b, look);
                    return;
                }
                stroke(
                    w,
                    b.origin,
                    &turn_arc(c, c, r, s0, s0 + span),
                    ring,
                    track,
                    false,
                );
                if frac > 0.0 {
                    let p = turn_arc(c, c, r, s0, s0 + span * frac);
                    glow_stroke(w, b.origin, &p, ring, col, glow);
                }
                crate::gauges::paint(w, b, look);
            },
        )
        .absolute()
        .size_full();
        let caption = self
            .caption
            .map(|c| dodai_gpui::text::format_label(&t.typography, &c));
        let value_size = match style {
            GaugeStyle::Bar | GaugeStyle::Chevron | GaugeStyle::Led => bh * 0.9,
            GaugeStyle::Battery | GaugeStyle::Signal | GaugeStyle::Thermometer => size * 0.16,
            GaugeStyle::Dial => size * 0.17,
            GaugeStyle::Meter => size * 0.15,
            GaugeStyle::Level => size * 0.13,
            _ => size * 0.22,
        };
        let readout = div()
            .flex()
            .flex_col()
            .items_center()
            .font_family(first_family(t.typography.display))
            .text_size(px(value_size))
            .text_color(hsla(t.palette.text))
            .child(label)
            .children(caption.map(|c| {
                div()
                    .text_size(px((size * 0.085).max(9.0)))
                    .text_color(hsla(t.palette.text_muted))
                    .font_family(first_family(t.typography.mono))
                    .child(c)
            }));
        let frame = div().relative().w(px(bw)).h(px(bh)).child(canvas_el);
        // Hovering spells out the reading against its range and target.
        let detail = {
            let (lo, hi) = (self.min.min(self.max), self.min.max(self.max));
            let mut d = format!(
                "{} of {}–{} · {:.0}%",
                fmt_num(self.value),
                fmt_num(lo),
                fmt_num(hi),
                norm(self.value, self.min, self.max) * 100.0
            );
            if let Some(tg) = self.target {
                d += &format!(" · target {}", fmt_num(tg));
            }
            d
        };
        let key = SharedString::from(format!("gauge-{:?}", self.follow.element_id(window, cx)));
        let inner = match style {
            GaugeStyle::Bar
            | GaugeStyle::Chevron
            | GaugeStyle::Level
            | GaugeStyle::Led
            | GaugeStyle::Battery
            | GaugeStyle::Signal
            | GaugeStyle::Thermometer => div()
                .flex()
                .items_center()
                .gap(px(t.spacing.md))
                .child(frame)
                .child(readout)
                .into_any_element(),
            GaugeStyle::Dial => frame
                .child(
                    div()
                        .absolute()
                        .w_full()
                        .top(px(size * 0.62))
                        .flex()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
            GaugeStyle::Meter => frame
                .child(
                    div()
                        .absolute()
                        .w_full()
                        .bottom_0()
                        .flex()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
            _ => frame
                .child(
                    div()
                        .absolute()
                        .size_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(readout),
                )
                .into_any_element(),
        };
        div()
            .id(key)
            .flex()
            .flex_none()
            .tooltip(dodai_gpui::panel::DodaiTooltip::text(detail))
            .child(inner)
    }
}

dodai_gpui::entry_render!(Gauge);
