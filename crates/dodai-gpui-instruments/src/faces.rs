//! Gauge faces: one scale and one value in every [`Look`]: the ring dial of the dash, a
//! chronograph, a vintage panel meter, a heads-up ring (or tape), a segment display, a halo of
//! light, an engine-indication arc and a race rev bar.
//! They take the same data, so a dash can swap one for another without touching its values.
//! Every colour comes from the theme.

mod arcs;
mod bars;
mod dials;
mod glass;
mod hero;
mod layouts;
mod lcd;
mod line;
mod panels;
mod rich;
mod round;
mod styles;
mod watch;

use crate::dash::{SidePod, Tone};
use crate::dial::severity_color;
use crate::look::{AtLimit, Layout, Look};
use crate::pod_math as pm;
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::icons::{Icon, Symbol};
use dodai_instruments_core::instrument::{readout, Scale, Severity};
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, AnyElement, App, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window,
};

/// One reading on a scale, drawn in the chosen [`Look`].
#[derive(IntoElement)]
pub struct GaugeFace {
    follow: Follow,
    scale: Scale,
    value: f32,
    /// The look drawn: the given one, else the design's, settled when rendered.
    look: Look,
    /// The look given with `.look(..)`; without one, the design's own.
    given_look: Option<Look>,
    at_limit: AtLimit,
    icon: Option<Symbol>,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    ends: [Option<Tone>; 2],
    layout: Layout,
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
            look: Look::Sport,
            given_look: None,
            at_limit: AtLimit::Value,
            icon: None,
            label: SharedString::default(),
            unit: SharedString::default(),
            decimals: 0,
            ends: [None, None],
            layout: Layout::Dial,
            size: 200.0,
        }
    }
    /// The look (see [`Look`]); by default the design's own.
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
    /// The layout within the look: every look has four (see [`Look::layouts`]), e.g. a tape
    /// for [`Look::HeadsUp`], a meter or a ribbon for [`Look::Vintage`], an arc or a bar for
    /// [`Look::Halo`]. A look without the layout draws its dial.
    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }
    /// Width of the face in px (at least 64). Dials are square; the tape is taller, the ribbon,
    /// arc and bar flatter.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pm::finite(s, 200.0).max(64.0);
        self
    }
}

impl RenderOnce for GaugeFace {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.look = crate::look::resolve(self.given_look, cx);
        self.value = pm::finite(follow(w, cx, &self.follow, 0, self.value), self.scale.min);
        match (self.look, self.look.fit(self.layout)) {
            (Look::Sport, Layout::Sweep) => self.sport_sweep(cx),
            (Look::Sport, Layout::Track) => self.sport_track(cx),
            (Look::Sport, Layout::Ribbed) => self.sport_ribbed(cx),
            (Look::Sport, _) => self.dial(cx),
            (Look::Chronograph, Layout::Bezel) => self.chrono_bezel(cx),
            (Look::Chronograph, Layout::Register) => self.chrono_register(cx),
            (Look::Chronograph, Layout::Sector) => self.chrono_sector(cx),
            (Look::Chronograph, _) => self.chrono(cx),
            (Look::HeadsUp, Layout::Tape) => self.tape_face(cx),
            (Look::HeadsUp, Layout::Cut) => self.hud_cut(cx),
            (Look::HeadsUp, Layout::Arc) => self.hud_arc(cx),
            (Look::HeadsUp, _) => self.hud(cx),
            (Look::Vintage, Layout::Meter) => self.panel_meter(cx),
            (Look::Vintage, Layout::Ribbon) => self.ribbon(cx),
            (Look::Vintage, Layout::Bezel) => self.vintage_bezel(cx),
            (Look::Vintage, _) => self.vintage(cx),
            (Look::Halo, Layout::Arc) => self.halo_arc(cx),
            (Look::Halo, Layout::Capsule) => self.halo_capsule(cx),
            (Look::Halo, Layout::Bar) => self.halo_bar(cx),
            (Look::Segment, Layout::Bar) => self.segment_bar(cx),
            (Look::Segment, Layout::Column) => self.segment_column(cx),
            (Look::Segment, Layout::Arc) => self.segment_arc(cx),
            (Look::Segment, _) => self.digital(cx),
            (Look::Halo, _) => self.futuristic(cx),
            (Look::GlassCockpit, Layout::Tape) => self.cockpit_tape(cx),
            (Look::GlassCockpit, Layout::Column) => self.cockpit_column(cx),
            (Look::GlassCockpit, Layout::Bar) => self.cockpit_bar(cx),
            (Look::GlassCockpit, _) => self.cockpit(cx),
            (Look::Race, Layout::Arc) => self.race_arc(cx),
            (Look::Race, Layout::Bar) => self.race_bar(cx),
            (Look::Race, Layout::Column) => self.race_column(cx),
            (Look::Race, _) => self.race(cx),
            (Look::Line, Layout::Bar) => self.line_bar(cx),
            (Look::Line, Layout::Arc) => self.line_arc(cx),
            (Look::Line, Layout::Column) => self.line_column(cx),
            (Look::Line, _) => self.line_dial(cx),
        }
    }
}

use crate::shapes::Ink;

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

    /// The colour of the zone the value is in once it is past a limit; `None` in the normal
    /// range.
    fn limit(&self, t: &dodai_core::Theme) -> Option<Hsla> {
        match pm::severity_at(&self.spans(), self.frac()) {
            Severity::Normal => None,
            sev => Some(severity_color(t, sev)),
        }
    }

    /// The colour of the value: the zone it is in, else the accent.
    fn value_colour(&self, t: &dodai_core::Theme, ink: &Ink) -> Hsla {
        self.limit(t).unwrap_or(ink.primary)
    }

    /// The limit's colour when the whole display is to take it ([`AtLimit::Display`]); `None`
    /// in the normal range or when only the value is coloured.
    fn display_limit(&self, t: &dodai_core::Theme) -> Option<Hsla> {
        match self.at_limit {
            AtLimit::Value => None,
            AtLimit::Display => self.limit(t),
        }
    }

    /// The ink of a screen look: the theme's, lit in the limit's colour past a limit when the
    /// whole display is to take it (see [`Ink::lit`]). The screen looks (heads-up, segment LCD,
    /// halo, glass cockpit, line) draw with it; the printed ones keep their colours.
    fn lit_ink(&self, t: &dodai_core::Theme) -> Ink {
        let ink = Ink::of(t);
        self.display_limit(t).map_or(ink, |c| ink.lit(c))
    }

    fn figure(&self) -> String {
        readout(self.value, self.decimals, "")
    }

    /// The sport dial: the dash's ring with the figure, its unit and the label in the middle.
    fn dial(self, cx: &App) -> AnyElement {
        let t = dodai_gpui::theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let figure_col = self.limit(&t).unwrap_or(ink.text);
        let middle = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(s * 0.008))
            .when_some(self.icon, |d, i| {
                d.child(Icon::symbol(i).size(s * 0.075).color(ink.muted))
            })
            .child(
                div()
                    .text_size(px(s * 0.17))
                    .line_height(px(s * 0.18))
                    .font_weight(gpui::FontWeight::LIGHT)
                    .text_color(figure_col)
                    .child(self.figure()),
            )
            .child(
                div()
                    .text_size(px(s * 0.045))
                    .text_color(ink.muted)
                    .child(self.unit.clone()),
            )
            .when(!self.label.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(s * 0.036))
                        .text_color(ink.muted)
                        .child(self.label.to_uppercase()),
                )
            });
        SidePod::new(self.scale.clone(), self.value)
            .animate(false)
            .ends(self.ends[0], self.ends[1])
            .centre(middle)
            .size(s)
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
}
