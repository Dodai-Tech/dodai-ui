//! Military instruments: a [`TacticalPlot`] of tracked contacts, a [`ThreatRosette`], a
//! [`StoresPanel`] and a [`MissionTimer`]. Geometry and severity come from
//! `crate::tactical`; this file only paints it with the theme's tokens.

use crate::gpui::dial::severity_color;
use crate::gpui::draw::{fmt_num, polar};
use crate::gpui::scope::frame;
use crate::gpui::scope::{blip_alpha, clock};
use crate::gpui::segment::SegmentDisplay;
use crate::instrument::Severity;
use crate::tactical::{self, Affiliation, Contact};
use dodai_charts::math::turn_arc;
use dodai_gpui::paint::{fill, scale_alpha, shaped, stroke, stroke_batch, ShapeStyle};
use dodai_gpui::text::{first_family, format_label};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, prelude::FluentBuilder, px, App, Bounds, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Called with the index of the contact, row or station the user picked.
type OnSelect = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn affiliation_color(t: &dodai_core::Theme, a: Affiliation) -> Hsla {
    hsla(match a {
        Affiliation::Friendly => t.palette.info,
        Affiliation::Hostile => t.palette.danger,
        Affiliation::Neutral => t.palette.success,
        Affiliation::Unknown => t.palette.warning,
    })
}

/// A plan-position plot: range rings, bearing ticks, own position in the centre and contacts
/// drawn with an affiliation symbol and a heading line.
#[derive(IntoElement)]
pub struct TacticalPlot {
    on_select: Option<OnSelect>,
    contacts: Vec<Contact>,
    selected: Option<usize>,
    range: (f32, &'static str),
    live: bool,
    size: f32,
    caption: Option<SharedString>,
}

impl TacticalPlot {
    /// A plot of `contacts`.
    pub fn new(contacts: impl IntoIterator<Item = Contact>) -> Self {
        Self {
            on_select: None,
            contacts: contacts.into_iter().collect(),
            selected: None,
            range: (100.0, "nm"),
            live: true,
            size: 300.0,
            caption: None,
        }
    }
    /// Makes contacts clickable; called with the contact's index.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    /// Highlights the contact at this index with a selection bracket.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    /// Distance of the outer ring and its unit, e.g. `(100.0, "nm")`.
    pub fn range(mut self, max: f32, unit: &'static str) -> Self {
        self.range = (max, unit);
        self
    }
    /// Sweep a beam round the plot and let contacts brighten as it passes (default on).
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
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

impl RenderOnce for TacticalPlot {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (size, live, selected) = (self.size, self.live, self.selected);
        let (max, unit) = self.range;
        let contacts = self.contacts;
        let grid = scale_alpha(hsla(t.palette.border), 0.8);
        let own = hsla(t.palette.primary);
        let (c, r) = (size / 2.0, size / 2.0 - 14.0);
        let t2 = t.clone();
        let cs = contacts.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let rings: Vec<_> = (1..=4)
                    .map(|i| turn_arc(c, c, r * i as f32 / 4.0, 0.0, 1.0))
                    .collect();
                stroke_batch(w, o, &rings, 1.0, grid, true);
                let spokes: Vec<_> = (0..36)
                    .map(|i| {
                        let a = i as f32 / 36.0;
                        let len = if i % 3 == 0 { 0.08 } else { 0.04 };
                        vec![polar(c, c, r * (1.0 - len), a), polar(c, c, r, a)]
                    })
                    .collect();
                stroke_batch(w, o, &spokes, 1.0, grid, false);
                let axes = [vec![(c, c - r), (c, c + r)], vec![(c - r, c), (c + r, c)]];
                stroke_batch(w, o, &axes, 1.0, scale_alpha(grid, 0.5), false);
                let sweep = if live { (clock() / 5.0).fract() } else { 0.0 };
                if live {
                    for i in 0..12 {
                        let a1 = sweep - i as f32 * 0.012;
                        let a0 = a1 - 0.012;
                        let mut p = turn_arc(c, c, r, a0, a1);
                        p.push((c, c));
                        fill(w, o, &p, scale_alpha(own, 0.16 * (1.0 - i as f32 / 12.0)));
                    }
                    stroke(
                        w,
                        o,
                        &[(c, c), polar(c, c, r, sweep)],
                        1.5,
                        scale_alpha(own, 0.8),
                        false,
                    );
                    dodai_gpui::follow::next_frame(w);
                }
                for (i, k) in cs.iter().enumerate() {
                    let col = affiliation_color(&t2, k.affiliation);
                    let (x, y) = k.position(c, c, r);
                    let glow = if live {
                        0.45 + 0.55 * blip_alpha(sweep, k.bearing.rem_euclid(1.0))
                    } else {
                        1.0
                    };
                    if let Some(h) = k.heading {
                        stroke(
                            w,
                            o,
                            &[(x, y), polar(x, y, size * 0.07, h)],
                            2.0,
                            scale_alpha(col, glow),
                            false,
                        );
                    }
                    let sym = tactical::symbol(k.affiliation, x, y, size * 0.022);
                    fill(w, o, &sym, scale_alpha(col, 0.35 * glow));
                    stroke(w, o, &sym, 2.0, scale_alpha(col, glow), true);
                    if selected == Some(i) {
                        let e = size * 0.045;
                        let br = [
                            vec![(x - e, y - e + 5.0), (x - e, y - e), (x - e + 5.0, y - e)],
                            vec![(x + e - 5.0, y - e), (x + e, y - e), (x + e, y - e + 5.0)],
                            vec![(x + e, y + e - 5.0), (x + e, y + e), (x + e - 5.0, y + e)],
                            vec![(x - e + 5.0, y + e), (x - e, y + e), (x - e, y + e - 5.0)],
                        ];
                        stroke_batch(w, o, &br, 1.5, hsla(t2.palette.text), false);
                    }
                }
                let ship = [
                    (c, c - 7.0),
                    (c + 5.0, c + 6.0),
                    (c, c + 3.0),
                    (c - 5.0, c + 6.0),
                ];
                fill(w, o, &ship, own);
            },
        )
        .absolute()
        .size_full();
        let muted = hsla(t.palette.text_muted);
        let small = px(t.typography.size_xs);
        let tag = |x: f32, y: f32, text: String, col: Hsla| {
            div()
                .absolute()
                .left(px(x))
                .top(px(y))
                .text_size(small)
                .text_color(col)
                .child(text)
        };
        let mut body = div().relative().size_full().child(cv);
        for (txt, a) in [("N", 0.0), ("E", 0.25), ("S", 0.5), ("W", 0.75)] {
            let (x, y) = polar(c, c, r + 8.0, a);
            body = body.child(tag(x - 4.0, y - 7.0, txt.into(), muted));
        }
        // Range labels sit on the diagonal that keeps farthest from every contact and its tag.
        let spot = |i: usize, a: f32| {
            let (lx, ly) = polar(c, c, r * i as f32 / 4.0, a);
            (lx + 2.0, ly - 12.0)
        };
        let crowd = |a: f32| {
            let mut worst = f32::MAX;
            for i in 1..=4 {
                let (lx, ly) = spot(i, a);
                for k in &contacts {
                    let (x, y) = k.position(c, c, r);
                    for (tx, ty) in [(x, y), (x + size * 0.03 + 12.0, y - size * 0.045 + 6.0)] {
                        worst =
                            worst.min(((lx + 12.0 - tx).powi(2) + (ly + 6.0 - ty).powi(2)).sqrt());
                    }
                }
            }
            worst
        };
        let range_at = [0.125, 0.375, 0.625, 0.875]
            .into_iter()
            .max_by(|a, b| crowd(*a).total_cmp(&crowd(*b)))
            .unwrap_or(0.125);
        for i in 1..=4 {
            let v = max * i as f32 / 4.0;
            let (lx, ly) = polar(c, c, r * i as f32 / 4.0, range_at);
            body = body.child(tag(
                lx + 2.0,
                ly - 12.0,
                if i == 4 {
                    format!("{} {unit}", fmt_num(v))
                } else {
                    fmt_num(v)
                },
                muted,
            ));
        }
        for k in &contacts {
            let (x, y) = k.position(c, c, r);
            body = body.child(tag(
                x + size * 0.03,
                y - size * 0.045,
                format!("{:03}", k.id),
                affiliation_color(&t, k.affiliation),
            ));
        }
        if let Some(cb) = &self.on_select {
            for (i, k) in contacts.iter().enumerate() {
                let (x, y) = k.position(c, c, r);
                let cb = cb.clone();
                body = body.child(
                    div()
                        .id(("track-hit", i))
                        .absolute()
                        .left(px(x - 14.0))
                        .top(px(y - 14.0))
                        .size(px(28.0))
                        .cursor_pointer()
                        .on_click(move |_, w, cx| cb(i, w, cx)),
                );
            }
        }
        frame(&t, size, size, self.caption, body)
    }
}

/// A table of the tracks a [`TacticalPlot`] shows: number, side, bearing and distance.
#[derive(IntoElement)]
pub struct TrackList {
    on_select: Option<OnSelect>,
    contacts: Vec<Contact>,
    selected: Option<usize>,
    range: (f32, &'static str),
}

impl TrackList {
    /// A list of `contacts`.
    pub fn new(contacts: impl IntoIterator<Item = Contact>) -> Self {
        Self {
            on_select: None,
            contacts: contacts.into_iter().collect(),
            selected: None,
            range: (100.0, "nm"),
        }
    }
    /// Makes rows clickable; called with the row's index.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    /// Highlights a row.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    /// Distance of the plot's outer ring and its unit.
    pub fn range(mut self, max: f32, unit: &'static str) -> Self {
        self.range = (max, unit);
        self
    }
}

impl RenderOnce for TrackList {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let (max, unit) = self.range;
        let mono = first_family(t.typography.mono);
        let cell = |text: String, w: f32, col: Hsla| {
            div()
                .w(px(w))
                .text_size(px(t.typography.size_sm))
                .font_family(mono.clone())
                .text_color(col)
                .child(text)
        };
        let muted = hsla(t.palette.text_muted);
        let head = div()
            .flex()
            .pb(px(t.spacing.xs))
            .border_b_1()
            .border_color(hsla(t.palette.border))
            .child(cell("TRK".into(), 48.0, muted))
            .child(cell("SIDE".into(), 84.0, muted))
            .child(cell("BRG".into(), 52.0, muted))
            .child(cell(format!("RNG {unit}"), 76.0, muted));
        let rows = self.contacts.iter().enumerate().map(|(i, k)| {
            let col = affiliation_color(&t, k.affiliation);
            let sel = self.selected == Some(i);
            let cb = self.on_select.clone();
            div()
                .id(("track-row", i))
                .flex()
                .py(px(2.0))
                .when(sel, |d| d.bg(scale_alpha(col, 0.18)))
                .when_some(cb, |d, cb| {
                    d.cursor_pointer().on_click(move |_, w, cx| cb(i, w, cx))
                })
                .child(cell(format!("{:03}", k.id), 48.0, hsla(t.palette.text)))
                .child(cell(k.affiliation.name().into(), 84.0, col))
                .child(cell(
                    format!("{:03}°", k.bearing_deg()),
                    52.0,
                    hsla(t.palette.text),
                ))
                .child(cell(
                    fmt_num(k.distance(max).round()),
                    76.0,
                    hsla(t.palette.text),
                ))
        });
        shaped(&t.shape, dodai_gpui::panel::panel_style(&t, false))
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .child(head)
            .children(rows)
    }
}

/// A threat on a [`ThreatRosette`]: `bearing` in turns and `level` `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Threat {
    /// Direction of the emitter, in turns (0 = ahead).
    pub bearing: f32,
    /// Strength, `0..=1`; longer wedges and hotter colours mean closer, more dangerous threats.
    pub level: f32,
}

impl Threat {
    /// A threat at `bearing` turns with `level` strength.
    pub fn new(bearing: f32, level: f32) -> Self {
        Self {
            bearing,
            level: level.clamp(0.0, 1.0),
        }
    }
}

/// A threat-warning rosette: a ring with a wedge per emitter pointing at its bearing, coloured
/// and sized by threat level.
#[derive(IntoElement)]
pub struct ThreatRosette {
    threats: Vec<Threat>,
    size: f32,
    caption: Option<SharedString>,
}

impl ThreatRosette {
    /// A rosette showing `threats`.
    pub fn new(threats: impl IntoIterator<Item = Threat>) -> Self {
        Self {
            threats: threats.into_iter().collect(),
            size: 220.0,
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

impl RenderOnce for ThreatRosette {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let size = self.size;
        let threats = self.threats.clone();
        let labels = self.threats.clone();
        let grid = scale_alpha(hsla(t.palette.border), 0.8);
        let own = hsla(t.palette.primary);
        let t2 = t.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let c = size / 2.0;
                let r = c - 14.0;
                let rings: Vec<_> = [0.25, 0.6, 1.0]
                    .iter()
                    .map(|k| turn_arc(c, c, r * k, 0.0, 1.0))
                    .collect();
                stroke_batch(w, o, &rings, 1.0, grid, true);
                let spokes: Vec<_> = (0..8)
                    .map(|i| {
                        vec![
                            polar(c, c, r * 0.25, i as f32 / 8.0),
                            polar(c, c, r, i as f32 / 8.0),
                        ]
                    })
                    .collect();
                stroke_batch(w, o, &spokes, 1.0, scale_alpha(grid, 0.5), false);
                for th in &threats {
                    let col = severity_color(&t2, tactical::threat_severity(th.level));
                    let p = tactical::wedge(
                        c,
                        c,
                        r * 0.28,
                        r * (0.28 + 0.72 * th.level),
                        th.bearing,
                        0.07,
                    );
                    let alarm = tactical::threat_severity(th.level) == Severity::Alarm;
                    let pulse = if alarm {
                        0.6 + 0.4 * (clock() * 6.0).sin().abs()
                    } else {
                        0.85
                    };
                    if alarm {
                        dodai_gpui::follow::next_frame(w);
                    }
                    fill(w, o, &p, scale_alpha(col, pulse));
                    stroke(w, o, &p, 1.0, col, true);
                }
                let ship = [
                    (c, c - 9.0),
                    (c + 7.0, c + 8.0),
                    (c, c + 4.0),
                    (c - 7.0, c + 8.0),
                ];
                fill(w, o, &ship, own);
            },
        )
        .absolute()
        .size_full();
        let mut body = div().relative().size_full().child(cv);
        for th in &labels {
            let (x, y) = polar(size / 2.0, size / 2.0, size / 2.0 - 14.0 + 9.0, th.bearing);
            body = body.child(
                div()
                    .absolute()
                    .left(px(x - 12.0))
                    .top(px(y - 7.0))
                    .w(px(24.0))
                    .flex()
                    .justify_center()
                    .text_size(px(t.typography.size_xs))
                    .text_color(severity_color(&t, tactical::threat_severity(th.level)))
                    .child(format!(
                        "{:03}",
                        (th.bearing.rem_euclid(1.0) * 360.0).round() as u32 % 360
                    )),
            );
        }
        frame(&t, size, size, self.caption, body)
    }
}

/// What a weapon station is doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StationState {
    /// Loaded and safed.
    Safe,
    /// Loaded and ready to arm.
    Ready,
    /// Armed.
    Armed,
    /// Nothing loaded.
    Empty,
}

impl StationState {
    fn label(self) -> &'static str {
        match self {
            Self::Safe => "SAFE",
            Self::Ready => "READY",
            Self::Armed => "ARMED",
            Self::Empty => "EMPTY",
        }
    }
}

/// One weapon station of a [`StoresPanel`].
#[derive(Debug, Clone, PartialEq)]
pub struct Station {
    name: SharedString,
    state: StationState,
}

impl Station {
    /// A station carrying `name`, in `state`.
    pub fn new(name: impl Into<SharedString>, state: StationState) -> Self {
        Self {
            name: name.into(),
            state,
        }
    }
}

/// A row of weapon stations with a state lamp each.
#[derive(IntoElement)]
pub struct StoresPanel {
    stations: Vec<Station>,
    selected: Option<usize>,
    on_select: Option<OnSelect>,
}

impl StoresPanel {
    /// A panel of `stations`.
    pub fn new(stations: impl IntoIterator<Item = Station>) -> Self {
        Self {
            stations: stations.into_iter().collect(),
            selected: None,
            on_select: None,
        }
    }
    /// Highlights a station.
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    /// Makes stations clickable; called with the station's index.
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for StoresPanel {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let row = div().flex().gap(px(t.spacing.sm));
        let (selected, on_select) = (self.selected, self.on_select);
        row.children(self.stations.into_iter().enumerate().map(|(i, s)| {
            let (fg, bg) = match s.state {
                StationState::Armed => (t.palette.danger, Some(t.palette.danger)),
                StationState::Ready => (t.palette.success, None),
                StationState::Safe => (t.palette.warning, None),
                StationState::Empty => (t.palette.text_muted, None),
            };
            let style = ShapeStyle::from_tokens(
                &t,
                Some(bg.map_or(t.palette.surface, |c| c.with_alpha(0.25))),
                Some(fg),
                false,
                false,
            );
            let text = hsla(t.palette.text);
            let picked = selected == Some(i);
            let cb = on_select.clone();
            shaped(&t.shape, style)
                .id(("station", i))
                .when(picked, |d| d.border_2().border_color(hsla(t.palette.text)))
                .when_some(cb, |d, cb| {
                    d.cursor_pointer().on_click(move |_, w, cx| cb(i, w, cx))
                })
                .w(px(86.0))
                .p(px(t.spacing.sm))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(t.spacing.xs))
                .child(
                    div()
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.text_muted))
                        .child(format!("STA {}", i + 1)),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_md))
                        .text_color(text)
                        .font_family(first_family(t.typography.mono))
                        .child(format_label(&t.typography, &s.name)),
                )
                .child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(fg))
                        .child(s.state.label()),
                )
        }))
    }
}

/// A countdown in seven-segment digits, hotter as the last minutes run out.
#[derive(IntoElement)]
pub struct MissionTimer {
    secs: i64,
    label: SharedString,
    height: f32,
}

impl MissionTimer {
    /// A timer showing `secs` to go (negative: elapsed).
    pub fn new(secs: i64) -> Self {
        Self {
            secs,
            label: "MISSION TIME".into(),
            height: 44.0,
        }
    }
    /// The caption above the digits.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = l.into();
        self
    }
    /// Digit height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
}

impl RenderOnce for MissionTimer {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let col = match tactical::timer_severity(self.secs) {
            Severity::Normal => hsla(t.palette.primary),
            s => severity_color(&t, s),
        };
        // `T-` is a label, the digits show the time itself.
        let text = tactical::countdown(self.secs);
        let (sign, digits) = text.split_at(2);
        div()
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
                    .gap(px(t.spacing.sm))
                    .child(
                        div()
                            .pb(px(2.0))
                            .text_size(px(t.typography.size_md))
                            .text_color(col)
                            .child(sign.to_string()),
                    )
                    .child(
                        SegmentDisplay::new(digits.to_string())
                            .height(self.height)
                            .color(col),
                    ),
            )
    }
}
