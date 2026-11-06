//! Gauge faces: one scale and one value in every [`Look`]: the ring dial of the dash, a
//! chronograph, a vintage panel meter, a heads-up ring (or tape), a segment display, a halo of
//! light, an engine-indication arc and a race rev bar.
//! They take the same data, so a dash can swap one for another without touching its values.
//! Every colour comes from the theme.

mod panels;
mod rich;
mod styles;

use crate::dash::{SidePod, Tone};
use crate::dial::severity_color;
use crate::look::{AtLimit, Look};
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
    look: Look,
    at_limit: AtLimit,
    icon: Option<Symbol>,
    label: SharedString,
    unit: SharedString,
    decimals: usize,
    ends: [Option<Tone>; 2],
    alternate: bool,
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
            look: Look::Classic,
            at_limit: AtLimit::Value,
            icon: None,
            label: SharedString::default(),
            unit: SharedString::default(),
            decimals: 0,
            ends: [None, None],
            alternate: false,
            size: 200.0,
        }
    }
    /// The look.
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
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
    /// Use the look's alternative layout where it has one: [`Look::HeadsUp`] becomes a vertical
    /// tape scrolling under a pointer box, [`Look::Vintage`] a panel meter with its pivot below
    /// the window. The other looks ignore it.
    pub fn alternate(mut self, on: bool) -> Self {
        self.alternate = on;
        self
    }
    /// Width of the face in px (at least 64); every look is square but the heads-up tape, which is
    /// taller.
    pub fn size(mut self, s: f32) -> Self {
        self.size = pm::finite(s, 200.0).max(64.0);
        self
    }
}

impl RenderOnce for GaugeFace {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = pm::finite(follow(w, cx, &self.follow, 0, self.value), self.scale.min);
        match self.look {
            Look::Classic => self.dial(),
            Look::Chrono => self.chrono(cx),
            Look::HeadsUp if self.alternate => self.tape_face(cx),
            Look::HeadsUp => self.hud(cx),
            Look::Vintage => self.vintage(cx),
            Look::Digital => self.digital(cx),
            Look::Futuristic => self.futuristic(cx),
            Look::Cockpit => self.cockpit(cx),
            Look::Race => self.race(cx),
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
    /// whole display is to take it (see [`Ink::lit`]). The screen looks (heads-up, digital,
    /// futuristic, glass cockpit) draw with it; the printed ones keep their colours.
    fn lit_ink(&self, t: &dodai_core::Theme) -> Ink {
        let ink = Ink::of(t);
        self.display_limit(t).map_or(ink, |c| ink.lit(c))
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
