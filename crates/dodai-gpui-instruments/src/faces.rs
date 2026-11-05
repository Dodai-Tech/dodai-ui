//! Gauge faces: one scale and one value shown in really different looks. The ring dial of the
//! dash, a chronograph, a heads-up ring, a sweep dial, a moving-coil meter and a scrolling tape.
//! They take the same data, so a dash can swap one for another without touching its values.
//! Every colour comes from the theme.

mod rich;

use crate::dash::{SidePod, Tone};
use crate::dial::severity_color;
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

/// How a [`GaugeFace`] looks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GaugeLook {
    /// The dash's ring dial: a bright rim, a scale with numbers and a needle.
    #[default]
    Dial,
    /// A chronograph: knurled bezel, fine scale, hatched zones, sub-dial and a tapered needle.
    Chrono,
    /// A heads-up ring: dashes, brackets, a dashed arc, an outside pointer and a framed figure.
    Hud,
    /// A vertical tape scrolling under a pointer box, with an overview column.
    Tape,
    /// A graded scale with a grid and a needle trailing a fading wake.
    Sweep,
    /// A moving-coil meter: wide face, pivot below it, mirror strip and a name plate.
    Meter,
}

impl GaugeLook {
    /// Every look, in the order a gallery shows them.
    pub const ALL: [Self; 6] = [
        Self::Dial,
        Self::Chrono,
        Self::Hud,
        Self::Sweep,
        Self::Meter,
        Self::Tape,
    ];

    /// A short name for a gallery caption.
    pub fn name(self) -> &'static str {
        match self {
            Self::Dial => "Dial",
            Self::Chrono => "Chrono",
            Self::Hud => "Heads-up",
            Self::Tape => "Tape",
            Self::Sweep => "Sweep",
            Self::Meter => "Meter",
        }
    }
}

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
            GaugeLook::Chrono => self.chrono(cx),
            GaugeLook::Hud => self.hud(cx),
            GaugeLook::Tape => self.tape(cx),
            GaugeLook::Sweep => self.sweep(cx),
            GaugeLook::Meter => self.meter(cx),
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
