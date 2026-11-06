//! More pods for the digital dash: tyre pressure, a list of readings, a notice, G-force, boost,
//! media and a compass. Each is an open ring ([`crate::dash::SidePod`] and
//! [`crate::dash::MenuRing`] are the first two) that a [`crate::dash::DashTriptych`] or a
//! [`DashQuad`] can hold, and each stands alone. All colours come from the theme.

use crate::dash::{text_box, Frame, LimitSign, Pod, PodStyle, RingTach, Telltales, Tone};
use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::gauges::disc;
use crate::look::{AtLimit, Layout, Look, Shape};
use crate::pod_math as pm;
use dodai_gpui::follow::{follow, follow_angle, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::icons::IconId;
use dodai_gpui::icons::{Icon, Symbol};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::{readout, Scale, Severity};
use gpui::prelude::FluentBuilder;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
};

/// An icon in a pod row: a dashboard symbol or a named icon.
#[derive(Clone, Copy, Debug)]
pub enum PodIcon {
    /// A dashboard symbol.
    Symbol(Symbol),
    /// A named icon.
    Named(IconId),
}

impl From<Symbol> for PodIcon {
    fn from(s: Symbol) -> Self {
        Self::Symbol(s)
    }
}

impl From<IconId> for PodIcon {
    fn from(i: IconId) -> Self {
        Self::Named(i)
    }
}

impl PodIcon {
    fn element(self, size: f32, color: Hsla) -> AnyElement {
        match self {
            Self::Symbol(s) => Icon::symbol(s).size(size).color(color).into_any_element(),
            Self::Named(i) => Icon::new(i).size(size).color(color).into_any_element(),
        }
    }
}

/// The ring every pod shares, with `content` laid over it.
fn ring(
    size: f32,
    ends: [Option<Tone>; 2],
    style: PodStyle,
    t: &dodai_core::Theme,
    paint: impl Fn(&mut Window, gpui::Point<Pixels>) + 'static,
    content: impl IntoElement,
) -> gpui::Div {
    let c = size / 2.0;
    let frame = Frame::new(t, style);
    let width = (size * 0.014).max(2.0);
    let r = c - size * 0.012;
    let ends = ends.map(|e| e.map(|tone| tone.color(t)));
    let cv = canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| {
            frame.paint(w, b.origin, c, r, width, ends);
            paint(w, b.origin);
        },
    )
    .absolute()
    .size_full();
    div()
        .relative()
        .w(px(size))
        .h(px(size))
        .child(cv)
        .child(content)
}

/// An absolutely placed box over the ring: `(left, top, width)` as fractions of `size`.
fn slot(size: f32, left: f32, top: f32, width: f32) -> gpui::Div {
    div()
        .absolute()
        .left(px(size * left))
        .top(px(size * top))
        .w(px(size * width))
        .flex()
        .flex_col()
        .items_center()
}

macro_rules! into_pod {
    ($($t:ty),*) => {$(
        impl From<$t> for Pod {
            fn from(p: $t) -> Self {
                Pod::Custom(Box::new(move |s, look| {
                    match look {
                        Some(l) => p.look(l),
                        None => p,
                    }
                    .size(s)
                    .into_any_element()
                }))
            }
        }
    )*};
}

fn pod_size(s: f32) -> f32 {
    pm::pod_diameter(s)
}

mod cockpit;
mod detailed;
mod digital;
mod dual;
mod energy;
mod future;
mod race;
mod vintage;

pub use energy::{charge_tone, power_frac, power_text, EnergyPod};

// ───────────────────────────────── TyrePod ─────────────────────────────────

/// Tyre pressures, two rows of two: front left and right, rear left and right.
#[derive(IntoElement)]
pub struct TyrePod {
    heading: SharedString,
    note: SharedString,
    unit: SharedString,
    bars: [Option<f32>; 4],
    low: f32,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl TyrePod {
    /// A tyre pod with no readings yet (`---`).
    pub fn new() -> Self {
        Self {
            heading: "Tyre pressure".into(),
            note: SharedString::default(),
            unit: "bar".into(),
            bars: [None; 4],
            low: 0.0,
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// The heading above the pressures.
    pub fn heading(mut self, h: impl Into<SharedString>) -> Self {
        self.heading = h.into();
        self
    }
    /// A line under the pressures, such as when readings start.
    pub fn note(mut self, n: impl Into<SharedString>) -> Self {
        self.note = n.into();
        self
    }
    /// The unit of the pressures, such as `bar`.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// The four pressures: front left, front right, rear left, rear right.
    pub fn bars(mut self, b: [Option<f32>; 4]) -> Self {
        self.bars = b;
        self
    }
    /// A pressure under this shows as a warning.
    pub fn low_below(mut self, v: f32) -> Self {
        self.low = v;
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl Default for TyrePod {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for TyrePod {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let size = self.size;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let c = size / 2.0;
        let bh = size * 0.4;
        let top = c - bh / 2.0 + size * 0.04;
        let mut content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(
                text_box(c, size * 0.17, size * 0.6)
                    .text_size(px(size * 0.058))
                    .text_color(ink)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.heading.clone()),
            )
            .child(
                text_box(c, top + bh * 0.46, size * 0.8)
                    .text_size(px(size * 0.05))
                    .text_color(ink)
                    .child(self.note.clone()),
            )
            .child(
                text_box(c, top + bh + size * 0.03, size * 0.4)
                    .text_size(px(size * 0.05))
                    .text_color(ink)
                    .child(self.unit.clone()),
            );
        for (i, bar) in self.bars.iter().enumerate() {
            let (side, row) = (i % 2, i / 2);
            let left = 0.5 + if side == 0 { -0.2 } else { 0.2 } - 0.11;
            let y = top + bh * if row == 0 { -0.12 } else { 0.62 };
            let col = if pm::is_low(*bar, self.low) {
                Tone::Warning.color(&t)
            } else if bar.is_some_and(f32::is_finite) {
                ink
            } else {
                muted
            };
            content = content.child(
                slot(size, left, y / size, 0.22)
                    .gap(px(size * 0.008))
                    .child(Icon::symbol(Symbol::Tyre).size(size * 0.09).color(col))
                    .child(
                        div()
                            .text_size(px(size * 0.055))
                            .text_color(col)
                            .child(pm::pressure_text(*bar)),
                    ),
            );
        }
        ring(size, [None, None], self.style, &t, |_, _| (), content).into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

/// One row of a [`RowsPod`]: an icon, an optional level bar and a value.
#[derive(Clone)]
pub struct PodRow {
    icon: PodIcon,
    value: SharedString,
    bar: Option<(f32, Tone)>,
    span: bool,
}

impl PodRow {
    /// A row with `icon` and `value` text.
    pub fn new(icon: impl Into<PodIcon>, value: impl Into<SharedString>) -> Self {
        Self {
            icon: icon.into(),
            value: value.into(),
            bar: None,
            span: false,
        }
    }
    /// Mark the working range of the bar: a cold tick at the start, a hot one at the end.
    pub fn span(mut self) -> Self {
        self.span = true;
        self
    }
    /// A level bar (`0..=1`) in a tone between the icon and the value.
    pub fn bar(mut self, frac: f32, tone: Tone) -> Self {
        self.bar = Some((pm::unit(frac), tone));
        self
    }
}

/// A pod holding a short list of readings on dark tiles: vehicle levels, a trip, today's items.
#[derive(IntoElement)]
pub struct RowsPod {
    heading: SharedString,
    rows: Vec<PodRow>,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl RowsPod {
    /// A pod with these `rows`.
    pub fn new(rows: impl IntoIterator<Item = PodRow>) -> Self {
        Self {
            heading: SharedString::default(),
            rows: rows.into_iter().collect(),
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// A heading above the rows.
    pub fn heading(mut self, h: impl Into<SharedString>) -> Self {
        self.heading = h.into();
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for RowsPod {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let size = self.size;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let tile = hsla(t.palette.surface_raised);
        let row_h = size * 0.105;
        let n = self.rows.len() as f32 + if self.heading.is_empty() { 0.0 } else { 1.0 };
        let mut list =
            slot(size, 0.13, 0.5 - (row_h * n) / size / 2.0 + 0.02, 0.74).gap(px(size * 0.01));
        if !self.heading.is_empty() {
            list = list.child(
                div()
                    .h(px(row_h))
                    .flex()
                    .items_center()
                    .text_size(px(size * 0.06))
                    .text_color(ink)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.heading.clone()),
            );
        }
        for row in &self.rows {
            let mut line = div()
                .w_full()
                .h(px(row_h))
                .flex()
                .items_center()
                .gap(px(size * 0.03))
                .px(px(size * 0.03))
                .bg(tile)
                .child(row.icon.element(size * 0.065, muted));
            if let Some((f, tone)) = row.bar {
                line = line.child(
                    div()
                        .flex_1()
                        .h(px(size * 0.022))
                        .bg(scale_alpha(ink, 0.2))
                        .relative()
                        .child(div().h_full().w(gpui::relative(f)).bg(tone.color(&t)))
                        .when(row.span, |d| {
                            d.child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .top(px(-size * 0.01))
                                    .w(px(size * 0.008))
                                    .h(px(size * 0.042))
                                    .bg(Tone::Info.color(&t)),
                            )
                            .child(
                                div()
                                    .absolute()
                                    .right_0()
                                    .top(px(-size * 0.01))
                                    .w(px(size * 0.008))
                                    .h(px(size * 0.042))
                                    .bg(Tone::Danger.color(&t)),
                            )
                        }),
                );
            } else {
                line = line.child(div().flex_1());
            }
            list = list.child(
                line.child(
                    div()
                        .text_size(px(size * 0.058))
                        .text_color(ink)
                        .whitespace_nowrap()
                        .child(row.value.clone()),
                ),
            );
        }
        ring(size, [None, None], self.style, &t, |_, _| (), list).into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

/// A message in a pod: a back arrow, a symbol, a title and a line of advice.
#[derive(IntoElement)]
pub struct NoticePod {
    icon: PodIcon,
    title: SharedString,
    text: SharedString,
    tone: Tone,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl NoticePod {
    /// A notice with `icon` and `title`.
    pub fn new(icon: impl Into<PodIcon>, title: impl Into<SharedString>) -> Self {
        Self {
            icon: icon.into(),
            title: title.into(),
            text: SharedString::default(),
            tone: Tone::Off,
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// The advice under the title.
    pub fn text(mut self, t: impl Into<SharedString>) -> Self {
        self.text = t.into();
        self
    }
    /// The colour of the symbol (default: the text colour).
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = t;
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for NoticePod {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let size = self.size;
        let ink = hsla(t.palette.text);
        let icon_col = if self.tone == Tone::Off {
            ink
        } else {
            self.tone.color(&t)
        };
        let content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(
                slot(size, 0.35, 0.025, 0.3).child(
                    div()
                        .flex()
                        .justify_center()
                        .w_full()
                        .py(px(size * 0.012))
                        .rounded_b(px(size * 0.03))
                        .bg(hsla(t.palette.surface_raised.mix(t.palette.text, 0.14)))
                        .child(Icon::new(IconId::ArrowLeft).size(size * 0.06).color(ink)),
                ),
            )
            .child(
                slot(size, 0.12, 0.3, 0.76)
                    .gap(px(size * 0.02))
                    .child(self.icon.element(size * 0.14, icon_col))
                    .child(
                        div()
                            .text_size(px(size * 0.06))
                            .text_color(ink)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(size * 0.05))
                            .text_color(ink)
                            .child(self.text.clone()),
                    ),
            );
        ring(
            size,
            [Some(Tone::Info), Some(Tone::Info)],
            self.style,
            &t,
            |_, _| (),
            content,
        )
        .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

/// G-force: a dotted cross with a dot that moves with the acceleration, and the four readings.
#[derive(IntoElement)]
pub struct GForcePod {
    follow: Follow,
    lateral: f32,
    longitudinal: f32,
    limit: f32,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl GForcePod {
    follow_api!();
    /// A pod at `lateral` (right is positive) and `longitudinal` (forward is positive) g.
    #[track_caller]
    pub fn new(lateral: f32, longitudinal: f32) -> Self {
        Self {
            follow: Follow::here(),
            lateral,
            longitudinal,
            limit: 1.5,
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// The g at the edge of the cross (default 1.5).
    pub fn limit(mut self, g: f32) -> Self {
        self.limit = pm::finite(g, 1.5).max(0.1);
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for GForcePod {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.lateral = pm::finite(follow(w, cx, &self.follow, 0, self.lateral), 0.0);
        self.longitudinal = pm::finite(follow(w, cx, &self.follow, 1, self.longitudinal), 0.0);
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let size = self.size;
        let c = size / 2.0;
        let (ink, muted, info) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.info),
        );
        let reach = size * 0.27;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r = size * 0.011;
            for i in -4i32..=4 {
                if i == 0 {
                    continue;
                }
                let d = reach * i as f32 / 4.0;
                fill(w, o, &disc(c + d, c, r, 10), scale_alpha(muted, 0.8));
                fill(w, o, &disc(c, c + d, r, 10), scale_alpha(muted, 0.8));
            }
            fill(
                w,
                o,
                &disc(c + gx * reach, c - gy * reach, size * 0.03, 24),
                info,
            );
        };
        let g = pm::g_text;
        let label = |x: f32, y: f32, v: f32| {
            text_box(x, y, size * 0.2)
                .text_size(px(size * 0.058))
                .text_color(ink)
                .child(g(v))
        };
        let content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(label(
                c,
                c - reach - size * 0.12,
                self.longitudinal.max(0.0),
            ))
            .child(label(
                c,
                c + reach + size * 0.04,
                (-self.longitudinal).max(0.0),
            ))
            .child(label(
                c - reach - size * 0.11,
                c - size * 0.03,
                (-self.lateral).max(0.0),
            ))
            .child(label(
                c + reach + size * 0.11,
                c - size * 0.03,
                self.lateral.max(0.0),
            ));
        ring(
            size,
            [Some(Tone::Info), Some(Tone::Info)],
            self.style,
            &t,
            paint,
            content,
        )
        .into_any_element()
    }
}

// ───────────────────────────────── BoostPod ─────────────────────────────────

/// Boost or charge: a percentage over a stack of cells, a label and a temperature.
#[derive(IntoElement)]
pub struct BoostPod {
    follow: Follow,
    percent: f32,
    label: SharedString,
    temp: SharedString,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    at_limit: AtLimit,
    style: PodStyle,
    size: f32,
}

impl BoostPod {
    follow_api!();
    /// A pod at `percent` (0 to 100).
    #[track_caller]
    pub fn new(percent: f32) -> Self {
        Self {
            follow: Follow::here(),
            percent: pm::percent(percent),
            label: "BOOST".into(),
            temp: SharedString::default(),
            look: Look::Sport,
            given_look: None,
            at_limit: AtLimit::Value,
            size: 300.0,
            style: PodStyle::Ring,
        }
    }
    /// The label on the left.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// The look (see [`Look`]); every look but [`Look::Sport`] draws the level as a
    /// [`GaugeFace`](crate::faces::GaugeFace) from 0 to 100 %, amber over 80.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// What a reading past a limit colours in the screen looks (see [`AtLimit`]): only the
    /// value, the default, or the whole display.
    pub fn at_limit(mut self, at: AtLimit) -> Self {
        self.at_limit = at;
        self
    }
    /// The temperature on the right, e.g. `9 °C`.
    pub fn temp(mut self, t: impl Into<SharedString>) -> Self {
        self.temp = t.into();
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for BoostPod {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.percent = pm::percent(follow(w, cx, &self.follow, 0, self.percent));
        if self.look != Look::Sport {
            return crate::faces::GaugeFace::new(
                Scale::new(0.0, 100.0)
                    .ticks_per(5, 2)
                    .zone(80.0, 100.0, Severity::Caution),
                self.percent,
            )
            .animate(false)
            .look(self.look)
            .at_limit(self.at_limit)
            .label(self.label)
            .icon(Symbol::Boost)
            .unit("%")
            .size(self.size)
            .into_any_element();
        }
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (ink, muted, ok) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.success),
        );
        let (bw, bh) = (size * 0.2, size * 0.3);
        let (bx, by) = (c - bw / 2.0, c - bh / 2.0 + size * 0.04);
        let percent_now = self.percent;
        let danger = hsla(t.palette.danger);
        let band_w = (size * 0.03).max(3.0);
        let band_r = c - size * 0.012 - band_w * 1.3;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            // The segmented band: arcs split at fixed gaps, a red mark at the limit gap.
            let (start, span) = crate::gauges::SWEEP;
            let cuts = [0.0, 0.2, 0.5, 0.72, 0.87, 1.0];
            let gap = 0.012;
            for (k, pair) in cuts.windows(2).enumerate() {
                let (u, v) = (
                    start + span * (pair[0] + gap),
                    start + span * (pair[1] - gap),
                );
                if k == 0 || k == cuts.len() - 2 {
                    continue; // the end caps already carry the cold and hot colours
                }
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, band_r, u, v),
                    band_w,
                    scale_alpha(ink, 0.5),
                    false,
                );
            }
            let m = start + span * 0.5;
            stroke(
                w,
                o,
                &[
                    polar(c, c, band_r - band_w * 0.8, m),
                    polar(c, c, band_r + band_w * 0.8, m),
                ],
                band_w * 0.6,
                danger,
                false,
            );
            let frame = [(bx, by), (bx + bw, by), (bx + bw, by + bh), (bx, by + bh)];
            stroke(w, o, &frame, 2.0, ink, true);
            let cells = 10;
            let lit = pm::lit_cells(percent_now, cells);
            for i in 0..cells {
                let y = by + bh - (i as f32 + 1.0) * bh / cells as f32;
                let on = i < lit;
                stroke(
                    w,
                    o,
                    &[
                        (bx + bw * 0.08, y + bh / cells as f32 * 0.5),
                        (bx + bw * 0.92, y + bh / cells as f32 * 0.5),
                    ],
                    if on { 3.0 } else { 1.0 },
                    if on { ok } else { scale_alpha(ink, 0.45) },
                    false,
                );
            }
            stroke(w, o, &[(bx, by + bh), (bx + bw, by + bh)], 3.0, ok, false);
        };
        let content = div()
            .absolute()
            .left_0()
            .top_0()
            .size_full()
            .child(
                text_box(c, by - size * 0.09, size * 0.4)
                    .text_size(px(size * 0.06))
                    .text_color(ink)
                    .child(format!("{:.0} %", self.percent)),
            )
            .child(
                text_box(c - size * 0.3, c - size * 0.02, size * 0.2)
                    .text_size(px(size * 0.05))
                    .text_color(ink)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.label.clone()),
            )
            .child(
                slot(size, 0.68, 0.43, 0.2)
                    .gap(px(size * 0.01))
                    .child(
                        div()
                            .text_size(px(size * 0.05))
                            .text_color(ink)
                            .child(self.temp.clone()),
                    )
                    .child(Icon::symbol(Symbol::Battery).size(size * 0.07).color(muted)),
            );
        ring(
            size,
            [Some(Tone::Info), Some(Tone::Danger)],
            self.style,
            &t,
            paint,
            content,
        )
        .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

/// What is playing: a cover, the title and artist, and the time.
#[derive(IntoElement)]
pub struct MediaPod {
    title: SharedString,
    artist: SharedString,
    time: SharedString,
    cover: Option<AnyElement>,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl MediaPod {
    /// A pod for `title`.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            artist: SharedString::default(),
            time: "0:00 | 0:00".into(),
            cover: None,
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// The artist or show under the title.
    pub fn artist(mut self, a: impl Into<SharedString>) -> Self {
        self.artist = a.into();
        self
    }
    /// The elapsed and total time line.
    pub fn time(mut self, t: impl Into<SharedString>) -> Self {
        self.time = t.into();
        self
    }
    /// Your own cover art (default: a framed note).
    pub fn cover(mut self, e: impl IntoElement) -> Self {
        self.cover = Some(e.into_any_element());
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for MediaPod {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let size = self.size;
        let (ink, muted, info) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.info),
        );
        let side = size * 0.3;
        let cover = self.cover.take().unwrap_or_else(|| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(Icon::new(IconId::Music).size(side * 0.5).color(muted))
                .into_any_element()
        });
        let content = div().absolute().left_0().top_0().size_full().child(
            slot(size, 0.1, 0.2, 0.8)
                .gap(px(size * 0.025))
                .child(
                    div()
                        .w(px(side))
                        .h(px(side))
                        .overflow_hidden()
                        .bg(hsla(t.palette.surface_raised))
                        .border_1()
                        .border_color(scale_alpha(ink, 0.5))
                        .child(cover),
                )
                .child(
                    div()
                        .text_size(px(size * 0.065))
                        .text_color(info)
                        .whitespace_nowrap()
                        .child(self.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(size * 0.05))
                        .text_color(ink)
                        .child(self.artist.clone()),
                )
                .child(
                    div()
                        .text_size(px(size * 0.05))
                        .text_color(info)
                        .child(self.time.clone()),
                ),
        );
        ring(
            size,
            [None, Some(Tone::Info)],
            self.style,
            &t,
            |_, _| (),
            content,
        )
        .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

/// A compass in a pod: a turning card under a fixed pointer, the bearing, the cardinal point and
/// the height.
#[derive(IntoElement)]
pub struct CompassPod {
    follow: Follow,
    heading: f32,
    height: SharedString,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    style: PodStyle,
    size: f32,
}

impl CompassPod {
    follow_api!();
    /// A compass at `heading` degrees (0 = north, clockwise).
    #[track_caller]
    pub fn new(heading: f32) -> Self {
        Self {
            follow: Follow::here(),
            heading: pm::bearing(heading),
            height: SharedString::default(),
            size: 300.0,
            look: Look::Sport,
            given_look: None,
            style: PodStyle::Ring,
        }
    }
    /// The height line under the bearing, e.g. `460 m`.
    pub fn height(mut self, h: impl Into<SharedString>) -> Self {
        self.height = h.into();
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for CompassPod {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.heading = pm::bearing(follow_angle(w, cx, &self.follow, 0, self.heading));
        let t = theme_of(cx);
        if self.look == Look::Line {
            // The line look keeps the ring pod, drawn as a single hairline circle.
            self.style = PodStyle::Line;
        }
        if !matches!(self.look, Look::Sport | Look::Line) {
            return self.detailed(&t);
        }
        let (size, heading) = (self.size, self.heading);
        let c = size / 2.0;
        let r = size * 0.35;
        let (ink, muted, info, danger) = (
            hsla(t.palette.text),
            hsla(t.palette.text_muted),
            hsla(t.palette.info),
            hsla(t.palette.danger),
        );
        let turn = heading / 360.0;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let circle: Vec<(f32, f32)> =
                (0..=96).map(|i| polar(c, c, r, i as f32 / 96.0)).collect();
            stroke(w, o, &circle, 1.5, scale_alpha(ink, 0.8), true);
            for i in 0..72 {
                let f = i as f32 / 72.0 - turn;
                let len = if i % 6 == 0 { 0.05 } else { 0.02 };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r, f), polar(c, c, r - size * len, f)],
                    1.0,
                    scale_alpha(muted, 0.9),
                    false,
                );
            }
            let tip = [
                (c, c - r - size * 0.01),
                (c - size * 0.03, c - r + size * 0.06),
                (c + size * 0.03, c - r + size * 0.06),
            ];
            fill(w, o, &tip, info);
        };
        let mut content = div().absolute().left_0().top_0().size_full();
        for (name, deg, north) in [
            ("N", 0.0, true),
            ("E", 90.0, false),
            ("S", 180.0, false),
            ("W", 270.0, false),
        ] {
            let (x, y) = polar(c, c, r + size * 0.065, deg / 360.0 - turn);
            content = content.child(
                text_box(x, y - size * 0.03, size * 0.1)
                    .text_size(px(size * 0.05))
                    .text_color(if north { danger } else { ink })
                    .child(name),
            );
        }
        let card = pm::cardinal(heading);
        content = content.child(
            slot(size, 0.25, 0.34, 0.5)
                .gap(px(size * 0.01))
                .child(
                    div()
                        .text_size(px(size * 0.06))
                        .text_color(ink)
                        .child(format!("{heading:.0}°")),
                )
                .child(
                    div()
                        .text_size(px(size * 0.085))
                        .text_color(info)
                        .font_weight(FontWeight::BOLD)
                        .child(card),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(size * 0.015))
                        .text_size(px(size * 0.055))
                        .text_color(ink)
                        .child(
                            Icon::symbol(Symbol::Altitude)
                                .size(size * 0.06)
                                .color(muted),
                        )
                        .child(self.height.clone()),
                ),
        );
        ring(
            size,
            [Some(Tone::Info), Some(Tone::Info)],
            self.style,
            &t,
            paint,
            content,
        )
        .into_any_element()
    }
}

// ───────────────────────────────── DualGauge ─────────────────────────────────

/// One scale of a [`DualGauge`]: its range, the value, the unit written in the middle and an
/// optional caption.
#[derive(Clone)]
pub struct DualScale {
    scale: Scale,
    value: f32,
    unit: SharedString,
    label: Option<SharedString>,
    decimals: usize,
}

impl DualScale {
    /// `value` on `scale`, with `unit` written in the middle.
    pub fn new(scale: Scale, value: f32, unit: impl Into<SharedString>) -> Self {
        Self {
            scale,
            value: pm::finite(value, 0.0),
            unit: unit.into(),
            label: None,
            decimals: 0,
        }
    }
    /// Decimals of the written value (default 0), in every look but [`Look::Sport`], which
    /// writes no value.
    pub fn decimals(mut self, n: usize) -> Self {
        self.decimals = n.min(3);
        self
    }
    /// A caption ("Pressure"), written on the reading's own face in every look but
    /// [`Look::Sport`], where the unit alone tells the two scales apart.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// A pod with two scales on one ring: the upper half for one reading, the lower arc inside for
/// another, each with its own needle (oil pressure over oil temperature). Sport's other layouts
/// and the other looks draw the first reading on one face with the second on a strip under it
/// in the look's style; the heads-up tapes stand side by side.
#[derive(IntoElement)]
pub struct DualGauge {
    follow: Follow,
    upper: DualScale,
    lower: DualScale,
    icon: Symbol,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    at_limit: AtLimit,
    layout: Layout,
    style: PodStyle,
    size: f32,
}

impl DualGauge {
    follow_api!();
    /// A ring with an `upper` and a `lower` scale and a symbol between them.
    #[track_caller]
    pub fn new(upper: DualScale, lower: DualScale, icon: Symbol) -> Self {
        Self {
            follow: Follow::here(),
            upper,
            lower,
            icon,
            look: Look::Sport,
            given_look: None,
            at_limit: AtLimit::Value,
            layout: Layout::Dial,
            size: 300.0,
            style: PodStyle::Ring,
        }
    }
    /// The look (see [`Look`]); every look but [`Look::Sport`] draws the upper scale on a
    /// [`GaugeFace`](crate::faces::GaugeFace) with the lower one on a strip under it.
    pub fn look(mut self, look: Look) -> Self {
        self.given_look = Some(look);
        self
    }
    /// What a reading past a limit colours in the screen looks (see [`AtLimit`]): only the
    /// value, the default, or the whole display.
    pub fn at_limit(mut self, at: AtLimit) -> Self {
        self.at_limit = at;
        self
    }
    /// The face's layout within the look (see
    /// [`GaugeFace::layout`](crate::faces::GaugeFace::layout)); heads-up tapes draw both scales
    /// as tapes side by side. In [`Look::Sport`] only [`Layout::Dial`] keeps both scales on one
    /// ring.
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pod_size(s);
        self
    }
}

impl RenderOnce for DualGauge {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.upper.value = pm::finite(follow(w, cx, &self.follow, 0, self.upper.value), 0.0);
        self.lower.value = pm::finite(follow(w, cx, &self.follow, 1, self.lower.value), 0.0);
        // Sport's own dial carries both scales on one ring; its other layouts and every other
        // look draw a face for the upper reading with the lower one on a strip under it.
        if self.look != Look::Sport || self.look.fit(self.layout) != Layout::Dial {
            // One face for the upper reading with the lower one on a strip under it, in the look's
            // style. Tall tapes have the room for two, so they stand side by side.
            let size = self.size;
            let layout = self.look.fit(self.layout);
            // Tapes and columns stand side by side; the flat layouts leave room for the strip
            // at full width.
            let (tapes, small) = match (layout, crate::look::shape(self.look, layout)) {
                (_, Shape::Tall) => (true, size),
                (Layout::Tape, _) => (true, size * 0.5),
                (_, Shape::Wide) => (false, size),
                (_, Shape::Square) => (false, size * (1.0 - dual::STRIP)),
            };
            let (look, at_limit) = (self.look, self.at_limit);
            let face = |d: DualScale, icon: Option<Symbol>| {
                let f = crate::faces::GaugeFace::new(d.scale, d.value)
                    .animate(false)
                    .look(look)
                    .at_limit(at_limit)
                    .layout(layout)
                    .unit(d.unit)
                    .decimals(d.decimals)
                    .size(small);
                let f = match d.label {
                    Some(l) => f.label(l),
                    None => f,
                };
                match icon {
                    Some(i) => f.icon(i),
                    None => f,
                }
            };
            if tapes {
                let (upper, lower) = (face(self.upper, Some(self.icon)), face(self.lower, None));
                return div()
                    .w(px(size))
                    .h(px(size))
                    .flex()
                    .items_center()
                    .justify_around()
                    .child(upper)
                    .child(lower)
                    .into_any_element();
            }
            let t = theme_of(cx);
            let strip = dual::strip(
                self.look,
                &self.lower,
                small,
                size * (dual::STRIP - 0.03),
                &t,
            );
            return div()
                .w(px(size))
                .h(px(size))
                .flex()
                .flex_col()
                .items_center()
                .justify_between()
                .child(face(self.upper, Some(self.icon)))
                .child(strip)
                .into_any_element();
        }
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let (r_rim, width) = (c - size * 0.012, (size * 0.014).max(2.0));
        let r_up = r_rim - width * 3.0;
        let r_lo = c * 0.64;
        // The upper scale runs left over the top to the right; the lower one along the bottom,
        // left to right as well (turn fractions: 0 is twelve o'clock, growing clockwise).
        let up_at = move |f: f32| 0.75 + 0.5 * f;
        let lo_at = move |f: f32| 0.63 - 0.26 * f;
        let theme = t.clone();
        let zone_col = move |s: Severity| severity_color(&theme, s);
        let paint_scale = move |w: &mut Window,
                                o: gpui::Point<Pixels>,
                                d: &DualScale,
                                r: f32,
                                at: &dyn Fn(f32) -> f32| {
            stroke(
                w,
                o,
                &turn_arc(c, c, r, at(0.0), at(1.0)),
                width * 1.2,
                scale_alpha(ink, 0.18),
                false,
            );
            for (a, z, sev) in d.scale.zone_spans() {
                if sev != Severity::Normal {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, at(a), at(z)),
                        width * 1.2,
                        zone_col(sev),
                        false,
                    );
                }
            }
            for k in d.scale.ticks() {
                let len = if k.major { size * 0.035 } else { size * 0.018 };
                let seg = [
                    polar(c, c, r - width, at(k.frac)),
                    polar(c, c, r - width - len, at(k.frac)),
                ];
                stroke(w, o, &seg, width * 0.7, scale_alpha(ink, 0.85), false);
            }
            let f = d.scale.fraction(d.value);
            stroke(
                w,
                o,
                &[
                    polar(c, c, r - size * 0.07, at(f)),
                    polar(c, c, r + width, at(f)),
                ],
                width * 1.4,
                ink,
                false,
            );
        };
        let (up, lo) = (self.upper.clone(), self.lower.clone());
        let frame = Frame::new(&t, self.style);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                frame.paint(w, o, c, r_rim, width, [None, None]);
                paint_scale(w, o, &up, r_up, &up_at);
                paint_scale(w, o, &lo, r_lo, &lo_at);
            },
        )
        .absolute()
        .size_full();
        let mut content = div().absolute().left_0().top_0().size_full().child(cv);
        for (d, r, at, dy) in [
            (
                &self.upper,
                r_up - size * 0.13,
                &up_at as &dyn Fn(f32) -> f32,
                -0.03,
            ),
            (
                &self.lower,
                r_lo - size * 0.1,
                &lo_at as &dyn Fn(f32) -> f32,
                -0.03,
            ),
        ] {
            for k in d.scale.ticks().into_iter().filter(|k| k.major) {
                let (x, y) = polar(c, c, r, at(k.frac));
                content = content.child(
                    text_box(x, y + size * dy, size * 0.14)
                        .text_size(px(size * 0.05))
                        .text_color(ink)
                        .child(readout(k.value, 0, "")),
                );
            }
        }
        content = content
            .child(
                text_box(c, size * 0.38, size * 0.3)
                    .text_size(px(size * 0.045))
                    .text_color(muted)
                    .child(self.upper.unit.clone()),
            )
            .child(
                slot(size, 0.45, 0.45, 0.1)
                    .child(Icon::symbol(self.icon).size(size * 0.1).color(ink)),
            )
            .child(
                text_box(c, size * 0.6, size * 0.3)
                    .text_size(px(size * 0.045))
                    .text_color(muted)
                    .child(self.lower.unit.clone()),
            );
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(content)
            .into_any_element()
    }
}

into_pod!(
    DualGauge, TyrePod, RowsPod, NoticePod, GForcePod, BoostPod, MediaPod, CompassPod, EnergyPod
);

// ───────────────────────────────── DashQuad ─────────────────────────────────

/// A dash of four pods around the ring: two on each side, the outer ones smaller and tucked
/// behind the inner ones.
#[derive(IntoElement)]
pub struct DashQuad {
    centre: RingTach,
    left: [Pod; 2],
    right: [Pod; 2],
    telltales: Telltales,
    limit: Option<u32>,
    mode: SharedString,
    gear: char,
    look: Option<Look>,
    width: f32,
}

impl DashQuad {
    /// A dash around `centre`. `left` and `right` list the outer pod first, then the inner one.
    pub fn new(centre: RingTach, left: [Pod; 2], right: [Pod; 2]) -> Self {
        Self {
            centre,
            left,
            right,
            telltales: Telltales::new(),
            limit: None,
            mode: SharedString::default(),
            gear: 'D',
            width: 1100.0,
            look: None,
        }
    }
    /// One look for the whole dash (see [`Look`]): the ring, the four pods, the lamps and the
    /// limit sign. Without it every part keeps its own.
    pub fn look(mut self, look: Look) -> Self {
        self.look = Some(look);
        self
    }
    /// The telltales under the ring.
    pub fn telltales(mut self, t: Telltales) -> Self {
        self.telltales = t;
        self
    }
    /// The speed limit sign above the left of the ring.
    pub fn limit(mut self, l: u32) -> Self {
        self.limit = Some(l);
        self
    }
    /// The drive mode at the bottom right.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = m.into();
        self
    }
    /// The engaged gear: `P`, `R`, `N` or `D`.
    pub fn gear(mut self, g: char) -> Self {
        self.gear = g;
        self
    }
    /// Width in px; the height follows (0.37 of it).
    pub fn width(mut self, w: f32) -> Self {
        self.width = pm::finite(w, 1100.0).max(700.0);
        self
    }
}

impl RenderOnce for DashQuad {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let look = self.look;
        let centre = match look {
            Some(l) => self.centre.look(l),
            None => self.centre,
        };
        let telltales = match look {
            Some(l) => self.telltales.look(l),
            None => self.telltales,
        };
        let width = self.width;
        let h = width * 0.37;
        let ring_d = h * 0.86;
        let (outer, inner) = (h * 0.38, h * 0.6);
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let small = (h * 0.045).max(11.0);
        let place = |e: AnyElement, left: f32, top: f32, d: f32| {
            div()
                .absolute()
                .left(px(left))
                .top(px(top))
                .w(px(d))
                .h(px(d))
                .child(e)
        };
        let [lo, li] = self.left;
        let [ri, ro] = self.right;
        // Right side mirrors the left: the inner pod is the first of the pair there.
        let mut root = div()
            .relative()
            .flex_none()
            .w(px(width))
            .h(px(h))
            .overflow_hidden()
            .rounded(px(t.shape.radius))
            .bg(hsla(t.palette.background.mix(t.palette.text, 0.02)))
            .border_1()
            .border_color(hsla(t.palette.border))
            .child(place(lo.sized(outer, look), 0.0, h * 0.31, outer))
            .child(place(ro.sized(outer, look), width - outer, h * 0.31, outer))
            .child(place(li.sized(inner, look), outer, h * 0.2, inner))
            .child(place(
                ri.sized(inner, look),
                width - outer - inner,
                h * 0.2,
                inner,
            ))
            .child(
                div()
                    .absolute()
                    .left(px((width - ring_d) / 2.0))
                    .top(px((h - ring_d) / 2.0))
                    .child(centre.size(ring_d)),
            );
        root = root.child(
            div()
                .absolute()
                .left(px(width * 0.17))
                .top(px(h * 0.8))
                .child(telltales.size(h * 0.055).columns(6)),
        );
        if let Some(l) = self.limit {
            root = root.child(
                div()
                    .absolute()
                    .left(px(width * 0.34))
                    .top(px(h * 0.04))
                    .child(
                        LimitSign::new(l)
                            .look(look.unwrap_or_default())
                            .size(h * 0.1),
                    ),
            );
        }
        let gears: Vec<AnyElement> = ['R', 'N', 'D']
            .into_iter()
            .map(|g| {
                div()
                    .text_color(if g == self.gear {
                        hsla(t.palette.info)
                    } else {
                        muted
                    })
                    .font_weight(if g == self.gear {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .child(g.to_string())
                    .into_any_element()
            })
            .collect();
        root.child(
            div()
                .absolute()
                .right(px(width * 0.04))
                .top(px(h * 0.8))
                .flex()
                .gap(px(small * 0.6))
                .text_size(px(small))
                .child(
                    div()
                        .text_color(ink)
                        .font_weight(FontWeight::MEDIUM)
                        .child(self.mode.clone()),
                )
                .child(div().flex().flex_col().children(gears)),
        )
    }
}
