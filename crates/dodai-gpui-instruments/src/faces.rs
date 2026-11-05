//! Gauge faces: one scale and one value shown in really different looks. The ring dial of the
//! dash, a thin arc with a big figure, a lit-segment arc, a horizontal bar tile and a vertical
//! column. They take the same data, so a dash can swap one for another without touching its
//! values. Every colour comes from the theme.

use crate::dash::{text_box, SidePod, Tone};
use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::gauges::SWEEP;
use crate::pod_math as pm;
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::icons::{Icon, Symbol};
use dodai_gpui::paint::{scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::{readout, Scale, Severity};
use gpui::prelude::FluentBuilder;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
};

/// How a [`GaugeFace`] looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaugeLook {
    /// The dash's ring dial: a bright rim, a scale with numbers and a needle.
    #[default]
    Dial,
    /// A thin arc that fills up to the value, with a big figure inside.
    Arc,
    /// An arc of separate segments that light up one by one.
    Segments,
    /// A tile with the figure on top and a horizontal bar with its zones underneath.
    Bar,
    /// A tall tile with a vertical column that fills from the bottom, like a thermometer.
    Column,
}

impl GaugeLook {
    /// Every look, in the order a gallery shows them.
    pub const ALL: [Self; 5] = [
        Self::Dial,
        Self::Arc,
        Self::Segments,
        Self::Bar,
        Self::Column,
    ];

    /// A short name for a gallery caption.
    pub fn name(self) -> &'static str {
        match self {
            Self::Dial => "Dial",
            Self::Arc => "Arc",
            Self::Segments => "Segments",
            Self::Bar => "Bar",
            Self::Column => "Column",
        }
    }
}

/// Lit segments in the [`GaugeLook::Segments`] arc.
const SEGMENTS: usize = 28;

/// One reading on a scale, drawn in the chosen [`GaugeLook`].
#[derive(IntoElement)]
pub struct GaugeFace {
    follow: Follow,
    scale: Scale,
    value: f32,
    look: GaugeLook,
    icon: Option<Symbol>,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    ends: [Option<Tone>; 2],
    size: f32,
}

impl GaugeFace {
    follow_api!();

    /// A face for `value` on `scale`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            look: GaugeLook::Dial,
            icon: None,
            label: SharedString::default(),
            unit: SharedString::default(),
            decimals: 0,
            ends: [None, None],
            size: 200.0,
        }
    }
    /// The look.
    pub fn look(mut self, look: GaugeLook) -> Self {
        self.look = look;
        self
    }
    /// A symbol beside the label (or above the figure).
    pub fn icon(mut self, i: Symbol) -> Self {
        self.icon = Some(i);
        self
    }
    /// What is measured, e.g. `Coolant`.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// The unit, e.g. `°C`.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// Decimals in the figure.
    pub fn decimals(mut self, d: usize) -> Self {
        self.decimals = d;
        self
    }
    /// Colour the two ends of a dial's rim (a cold end, a hot end).
    pub fn ends(mut self, low: Option<Tone>, high: Option<Tone>) -> Self {
        self.ends = [low, high];
        self
    }
    /// Width of the face in px; the tile looks are shorter or taller than that.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pm::finite(s, 200.0).max(100.0);
        self
    }
}

impl RenderOnce for GaugeFace {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = pm::finite(follow(w, cx, &self.follow, 0, self.value), self.scale.min);
        match self.look {
            GaugeLook::Dial => self.dial(),
            GaugeLook::Arc => self.arc(cx),
            GaugeLook::Segments => self.segments(cx),
            GaugeLook::Bar => self.bar(cx),
            GaugeLook::Column => self.column(cx),
        }
    }
}

/// The colours every look shares.
struct Ink {
    text: Hsla,
    muted: Hsla,
    primary: Hsla,
    face: Hsla,
    line: Hsla,
}

impl Ink {
    fn of(t: &dodai_core::Theme) -> Self {
        Self {
            text: hsla(t.palette.text),
            muted: hsla(t.palette.text_muted),
            primary: hsla(t.palette.primary),
            face: hsla(t.palette.surface_raised),
            line: hsla(t.palette.border),
        }
    }
}

impl GaugeFace {
    fn frac(&self) -> f32 {
        self.scale.fraction(self.value)
    }

    fn spans(&self) -> Vec<(f32, f32, Severity)> {
        self.scale
            .zone_spans()
            .into_iter()
            .filter(|z| z.2 != Severity::Normal)
            .collect()
    }

    /// The colour of the value: the zone it is in, else the accent.
    fn value_colour(&self, t: &dodai_core::Theme, ink: &Ink) -> Hsla {
        match pm::severity_at(&self.spans(), self.frac()) {
            Severity::Normal => ink.primary,
            sev => severity_color(t, sev),
        }
    }

    fn figure(&self) -> String {
        readout(self.value, self.decimals, "")
    }

    fn dial(self) -> AnyElement {
        let mut pod = SidePod::new(self.scale.clone(), self.value)
            .animate(false)
            .unit(self.unit.clone())
            .ends(self.ends[0], self.ends[1])
            .size(self.size);
        if let Some(i) = self.icon {
            pod = pod.icon(i);
        }
        pod.into_any_element()
    }

    /// The icon, big figure and unit stacked.
    fn readout_column(&self, ink: &Ink, size: f32, figure_k: f32, col: Hsla) -> gpui::Div {
        let mut c = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(size * 0.01))
            .when_some(self.icon, |d, i| {
                d.child(Icon::symbol(i).size(size * 0.09).color(ink.muted))
            });
        c = c.child(
            div()
                .text_size(px(size * figure_k))
                .text_color(col)
                .font_weight(FontWeight::MEDIUM)
                .child(self.figure()),
        );
        c.child(
            div()
                .text_size(px(size * 0.06))
                .text_color(ink.muted)
                .child(self.unit.clone()),
        )
    }

    fn arc(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let c = size / 2.0;
        let (start, span) = SWEEP;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let r = c - size * 0.07;
        let weight = size * 0.07;
        let track = scale_alpha(ink.text, 0.14);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    weight,
                    track,
                    false,
                );
                if frac > 0.0 {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, start, start + span * frac),
                        weight,
                        col,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let at = |f: f32| polar(c, c, r + weight * 0.2, start + span * f);
        let (lo, hi) = (at(0.0), at(1.0));
        let end = |p: (f32, f32), v: f32| {
            text_box(p.0, p.1 + size * 0.045, size * 0.2)
                .text_size(px(size * 0.05))
                .text_color(ink.muted)
                .child(readout(v, 0, ""))
        };
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .child(end(lo, self.scale.min))
            .child(end(hi, self.scale.max))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(size * 0.2))
                    .w(px(size))
                    .h(px(size * 0.5))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.readout_column(&ink, size, 0.22, ink.text)),
            )
            .into_any_element()
    }

    fn segments(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let c = size / 2.0;
        let (start, span) = SWEEP;
        let lit = pm::segments_lit(self.frac(), SEGMENTS);
        let spans = self.spans();
        let on_col = |f: f32| match pm::severity_at(&spans, f) {
            Severity::Normal => ink.primary,
            sev => severity_color(&t, sev),
        };
        let cols: Vec<Hsla> = (0..SEGMENTS)
            .map(|i| {
                if i < lit {
                    on_col((i as f32 + 0.5) / SEGMENTS as f32)
                } else {
                    scale_alpha(ink.text, 0.14)
                }
            })
            .collect();
        let (r_out, r_in) = (c - size * 0.05, c - size * 0.15);
        let width = size * 0.035;
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                for (i, col) in cols.iter().enumerate() {
                    let f = (i as f32 + 0.5) / SEGMENTS as f32;
                    let a = start + span * f;
                    stroke(
                        w,
                        b.origin,
                        &[polar(c, c, r_in, a), polar(c, c, r_out, a)],
                        width,
                        *col,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(size * 0.22))
                    .w(px(size))
                    .h(px(size * 0.46))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.readout_column(&ink, size, 0.21, ink.text)),
            )
            .into_any_element()
    }

    /// The label row of a tile: icon and label.
    fn heading(&self, ink: &Ink, size: f32) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .gap(px(size * 0.03))
            .text_size(px(size * 0.06))
            .text_color(ink.muted)
            .when_some(self.icon, |d, i| {
                d.child(Icon::symbol(i).size(size * 0.075).color(ink.muted))
            })
            .child(self.label.clone())
    }

    fn tile(&self, ink: &Ink, size: f32) -> gpui::Div {
        div()
            .flex()
            .flex_col()
            .rounded(px(size * 0.07))
            .border_1()
            .border_color(ink.line)
            .bg(ink.face)
            .p(px(size * 0.06))
    }

    /// The zone colours painted on a track as absolutely placed pieces; `vertical` runs them
    /// bottom to top.
    fn zone_marks(&self, t: &dodai_core::Theme, vertical: bool) -> Vec<AnyElement> {
        self.spans()
            .into_iter()
            .map(|(a, z, sev)| {
                let d = div()
                    .absolute()
                    .bg(scale_alpha(severity_color(t, sev), 0.45));
                if vertical {
                    d.left_0()
                        .w_full()
                        .bottom(gpui::relative(a))
                        .h(gpui::relative(z - a))
                } else {
                    d.top_0()
                        .h_full()
                        .left(gpui::relative(a))
                        .w(gpui::relative(z - a))
                }
                .into_any_element()
            })
            .collect()
    }

    fn bar(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let track = div()
            .relative()
            .w_full()
            .h(px(size * 0.075))
            .rounded(px(size * 0.02))
            .overflow_hidden()
            .bg(scale_alpha(ink.text, 0.12))
            .children(self.zone_marks(&t, false))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .h_full()
                    .w(gpui::relative(frac))
                    .rounded(px(size * 0.02))
                    .bg(col),
            );
        self.tile(&ink, size)
            .w(px(size))
            .gap(px(size * 0.04))
            .child(self.heading(&ink, size))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(size * 0.02))
                    .child(
                        div()
                            .text_size(px(size * 0.22))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .pb(px(size * 0.03))
                            .text_size(px(size * 0.07))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    ),
            )
            .child(track)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .text_size(px(size * 0.05))
                    .text_color(ink.muted)
                    .child(readout(self.scale.min, 0, ""))
                    .child(readout(self.scale.max, 0, "")),
            )
            .into_any_element()
    }

    fn column(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let size = self.size;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let track_h = size * 0.54;
        let track = div()
            .relative()
            .w(px(size * 0.14))
            .h(px(track_h))
            .rounded(px(size * 0.04))
            .overflow_hidden()
            .bg(scale_alpha(ink.text, 0.12))
            .children(self.zone_marks(&t, true))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .bottom_0()
                    .w_full()
                    .h(gpui::relative(frac))
                    .bg(col),
            );
        let mut ticks = div().relative().w(px(size * 0.2)).h(px(track_h));
        let majors: Vec<_> = self.scale.ticks().into_iter().filter(|k| k.major).collect();
        // Label every other (or third...) tick when there are too many to read.
        let step = majors.len().div_ceil(6).max(1);
        for k in majors.into_iter().step_by(step) {
            let y = track_h * (1.0 - k.frac);
            ticks = ticks.child(
                div()
                    .absolute()
                    .left_0()
                    .top(px((y - size * 0.03).clamp(0.0, track_h - size * 0.06)))
                    .text_size(px(size * 0.05))
                    .text_color(ink.muted)
                    .child(readout(k.value, 0, "")),
            );
        }
        self.tile(&ink, size)
            .w(px(size * 0.62))
            .h(px(size))
            .gap(px(size * 0.04))
            .items_center()
            .child(
                div()
                    .text_size(px(size * 0.15))
                    .text_color(ink.text)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.figure()),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(size * 0.03))
                    .child(track)
                    .child(ticks),
            )
            .child(
                div()
                    .text_size(px(size * 0.055))
                    .text_color(ink.muted)
                    .child(if self.label.is_empty() {
                        self.unit.clone()
                    } else {
                        self.label.clone()
                    }),
            )
            .into_any_element()
    }
}
