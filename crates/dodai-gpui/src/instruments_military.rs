//! Military instruments: a [`TacticalPlot`] of tracked contacts, a [`ThreatRosette`], a
//! [`StoresPanel`] and a [`MissionTimer`]. Geometry and severity come from
//! `dodai_charts_core::tactical`; this file only paints it with the theme's tokens.

use crate::charts::polar;
use crate::instrument_charts::frame;
use crate::instruments::severity_color;
use crate::instruments_segment::SegmentDisplay;
use crate::paint::{fill, scale_alpha, shaped, stroke, stroke_batch, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_charts_core::math::turn_arc;
use dodai_charts_core::tactical::{self, Affiliation, Contact};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

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
    contacts: Vec<Contact>,
    size: f32,
    caption: Option<SharedString>,
}

impl TacticalPlot {
    /// A plot of `contacts`.
    pub fn new(contacts: impl IntoIterator<Item = Contact>) -> Self {
        Self {
            contacts: contacts.into_iter().collect(),
            size: 280.0,
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

impl RenderOnce for TacticalPlot {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let size = self.size;
        let contacts = self.contacts;
        let grid = scale_alpha(hsla(t.palette.border), 0.8);
        let own = hsla(t.palette.primary);
        let t2 = t.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let c = size / 2.0;
                let r = c - 4.0;
                let rings: Vec<_> = (1..=4)
                    .map(|i| turn_arc(c, c, r * i as f32 / 4.0, 0.0, 1.0))
                    .collect();
                stroke_batch(w, o, &rings, 1.0, grid, true);
                let spokes: Vec<_> = (0..12)
                    .map(|i| {
                        let a = i as f32 / 12.0;
                        vec![polar(c, c, r * 0.9, a), polar(c, c, r, a)]
                    })
                    .collect();
                stroke_batch(w, o, &spokes, 1.0, grid, false);
                let axes = [
                    vec![(c, 4.0), (c, size - 4.0)],
                    vec![(4.0, c), (size - 4.0, c)],
                ];
                stroke_batch(w, o, &axes, 1.0, scale_alpha(grid, 0.5), false);
                for k in &contacts {
                    let col = affiliation_color(&t2, k.affiliation);
                    let (x, y) = k.position(c, c, r);
                    if let Some(h) = k.heading {
                        stroke(
                            w,
                            o,
                            &[(x, y), polar(x, y, size * 0.07, h)],
                            2.0,
                            col,
                            false,
                        );
                    }
                    let sym = tactical::symbol(k.affiliation, x, y, size * 0.025);
                    fill(w, o, &sym, scale_alpha(col, 0.35));
                    stroke(w, o, &sym, 2.0, col, true);
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
        frame(&t, size, size, self.caption, cv)
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
        let threats = self.threats;
        let grid = scale_alpha(hsla(t.palette.border), 0.8);
        let own = hsla(t.palette.primary);
        let t2 = t.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                let c = size / 2.0;
                let r = c - 4.0;
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
                    fill(w, o, &p, scale_alpha(col, 0.85));
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
        frame(&t, size, size, self.caption, cv)
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
}

impl StoresPanel {
    /// A panel of `stations`.
    pub fn new(stations: impl IntoIterator<Item = Station>) -> Self {
        Self {
            stations: stations.into_iter().collect(),
        }
    }
}

impl RenderOnce for StoresPanel {
    fn render(self, _w: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let row = div().flex().gap(px(t.spacing.sm));
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
            shaped(&t.shape, style)
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
            dodai_charts_core::instrument::Severity::Normal => hsla(t.palette.primary),
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
