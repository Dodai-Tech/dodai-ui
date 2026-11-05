//! A three-pod digital dash: a ribbed ring speedometer in the middle, two open-ring pods beside it
//! that carry a gauge or a menu, a row of telltales and the small print around them.
//!
//! The pieces stand alone ([`RingTach`], [`SidePod`], [`MenuRing`], [`LimitSign`], [`Telltales`])
//! and [`DashTriptych`] arranges them. Everything is drawn from theme tokens, so the dash follows
//! the design and the light or dark theme.

use crate::dial::severity_color;
use crate::draw::{polar, turn_arc};
use crate::gauges::{disc, SWEEP};
use crate::look::Look;
use dodai_core::Theme;
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::icons::{Icon, Symbol};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use dodai_instruments_core::instrument::{readout, Scale, Severity};
use gpui::prelude::FluentBuilder;
use gpui::{
    canvas, div, px, AnyElement, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels,
    RenderOnce, SharedString, Styled, Window,
};

mod menu_looks;
mod menu_more;

/// What a telltale reports; the colour comes from the theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    /// Dim: present, not lit.
    #[default]
    Off,
    /// Something is wrong now.
    Danger,
    /// Look at this soon.
    Warning,
    /// A system is working.
    Success,
    /// Information: lights, assists.
    Info,
}

impl Tone {
    /// The theme colour of this tone.
    pub fn color(self, t: &Theme) -> Hsla {
        let p = &t.palette;
        hsla(match self {
            Tone::Off => p.text_muted,
            Tone::Danger => p.danger,
            Tone::Warning => p.warning,
            Tone::Success => p.success,
            Tone::Info => p.info,
        })
    }
}

/// A box of `w` px centred on `x`, for a line of text at `y`.
pub(crate) fn text_box(x: f32, y: f32, w: f32) -> gpui::Div {
    div()
        .absolute()
        .left(px(x - w / 2.0))
        .top(px(y))
        .w(px(w))
        .flex()
        .justify_center()
        .whitespace_nowrap()
}

/// The open ring both kinds of pod share: a bright rim over [`SWEEP`] with optional coloured ends.
pub(crate) fn rim(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    c: f32,
    r: f32,
    width: f32,
    ink: Hsla,
    ends: [Option<Hsla>; 2],
) {
    let (start, span) = SWEEP;
    stroke(
        w,
        o,
        &turn_arc(c, c, r, start, start + span),
        width,
        scale_alpha(ink, 0.92),
        false,
    );
    let cap = span * 0.07;
    if let Some(col) = ends[0] {
        stroke(
            w,
            o,
            &turn_arc(c, c, r, start, start + cap),
            width + 1.0,
            col,
            false,
        );
    }
    if let Some(col) = ends[1] {
        stroke(
            w,
            o,
            &turn_arc(c, c, r, start + span - cap, start + span),
            width + 1.0,
            col,
            false,
        );
    }
}

/// How a pod's frame is drawn. Every pod takes one with `.style(..)`; the contents are the same.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PodStyle {
    /// An open ring: a bright rim over the sweep, open at the bottom.
    #[default]
    Ring,
    /// A closed face: a panel-coloured disc inside a fine full circle, the sweep in bright ink.
    Disc,
    /// The open ring in the accent colour with a soft glow round it.
    Glow,
    /// A heavy bezel with a fine inner line, like a real gauge.
    Bezel,
}

impl PodStyle {
    /// Every style, in the order a gallery shows them.
    pub const ALL: [Self; 4] = [Self::Ring, Self::Disc, Self::Glow, Self::Bezel];

    /// A short name for a gallery caption.
    pub fn name(self) -> &'static str {
        match self {
            Self::Ring => "Ring",
            Self::Disc => "Disc",
            Self::Glow => "Glow",
            Self::Bezel => "Bezel",
        }
    }
}

/// A pod's frame with its colours resolved from the theme, ready to paint.
#[derive(Clone, Copy)]
pub(crate) struct Frame {
    style: PodStyle,
    ink: Hsla,
    accent: Hsla,
    face: Hsla,
}

impl Frame {
    pub(crate) fn new(t: &Theme, style: PodStyle) -> Self {
        Self {
            style,
            ink: hsla(t.palette.text),
            accent: hsla(t.palette.primary),
            face: hsla(t.palette.surface_raised),
        }
    }

    /// Paints the frame of a pod of radius `c`: `r` is the rim radius, `width` its stroke.
    pub(crate) fn paint(
        self,
        w: &mut Window,
        o: gpui::Point<Pixels>,
        c: f32,
        r: f32,
        width: f32,
        ends: [Option<Hsla>; 2],
    ) {
        let (start, span) = SWEEP;
        match self.style {
            PodStyle::Ring => rim(w, o, c, r, width, self.ink, ends),
            PodStyle::Disc => {
                fill(w, o, &disc(c, c, r, 120), scale_alpha(self.face, 0.9));
                stroke(
                    w,
                    o,
                    &disc(c, c, r, 120),
                    width * 0.6,
                    scale_alpha(self.ink, 0.35),
                    true,
                );
                rim(w, o, c, r, width, self.ink, ends);
            }
            PodStyle::Glow => {
                for i in (1..=4).rev() {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, start, start + span),
                        width * (1.0 + i as f32 * 2.2),
                        scale_alpha(self.accent, 0.07),
                        false,
                    );
                }
                rim(w, o, c, r, width, self.accent, ends);
            }
            PodStyle::Bezel => {
                rim(w, o, c, r - width * 0.5, width * 2.0, self.ink, ends);
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r - width * 2.0, start, start + span),
                    width * 0.4,
                    scale_alpha(self.ink, 0.4),
                    false,
                );
            }
        }
    }
}

// ───────────────────────────────── RingTach ─────────────────────────────────

/// The centre of the dash: a ribbed ring scale for engine speed with a red band, a sweet-spot
/// marker and a rim needle, a big speed readout inside, and (when given) a range tab under it
/// and a fuel arc across the open bottom.
#[derive(IntoElement)]
pub struct RingTach {
    follow: Follow,
    scale: Scale,
    rpm: f32,
    speed: f32,
    unit: SharedString,
    marker: Option<f32>,
    tab: SharedString,
    fuel: Option<f32>,
    reach: SharedString,
    size: f32,
}

impl RingTach {
    follow_api!();
    /// A ring tachometer for engine speed `rpm` on `scale`, reading `speed` in the middle.
    #[track_caller]
    pub fn new(scale: Scale, rpm: f32, speed: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            rpm,
            speed,
            unit: SharedString::from("km/h"),
            marker: None,
            tab: SharedString::default(),
            fuel: None,
            reach: SharedString::default(),
            size: 380.0,
        }
    }
    /// Unit under the speed.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// A green marker at this engine speed: where the engine is happiest.
    pub fn marker(mut self, rpm: f32) -> Self {
        self.marker = Some(rpm);
        self
    }
    /// The text on the tab under the speed (a range, a mode).
    pub fn tab(mut self, t: impl Into<SharedString>) -> Self {
        self.tab = t.into();
        self
    }
    /// Fuel as `0..=1` and the reach written beside the pump. Without it the bottom stays empty.
    pub fn fuel(mut self, level: f32, reach: impl Into<SharedString>) -> Self {
        self.fuel = Some(level.clamp(0.0, 1.0));
        self.reach = reach.into();
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(200.0);
        self
    }
}

impl RenderOnce for RingTach {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.rpm = follow(w, cx, &self.follow, 0, self.rpm);
        self.speed = follow(w, cx, &self.follow, 1, self.speed);
        self.fuel = self.fuel.map(|f| follow(w, cx, &self.follow, 2, f));
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (r_out, r_in) = (c * 0.98, c * 0.72);
        let frac = self.scale.fraction(self.rpm);
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let face = hsla(t.palette.background);
        let body = hsla(t.palette.background.mix(t.palette.text, 0.07));
        let groove = scale_alpha(ink, 0.10);
        let danger = hsla(t.palette.danger);
        let go = hsla(t.palette.success);
        let reds: Vec<(f32, f32)> = self
            .scale
            .zone_spans()
            .into_iter()
            .filter(|z| z.2 == Severity::Alarm)
            .map(|z| (z.0, z.1))
            .collect();
        let marker = self.marker.map(|m| self.scale.fraction(m));
        let fuel = self.fuel;
        let ticks: Vec<(f32, bool)> = self
            .scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let weight = (size * 0.008).max(1.5);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                // The ribbed body: a disc, fine concentric grooves, then the dark face.
                fill(w, o, &disc(c, c, r_out, 120), body);
                let grooves = 9;
                for i in 0..grooves {
                    let r = r_in + (r_out - r_in) * (i as f32 + 0.5) / grooves as f32;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r, 0.0, 1.0),
                        weight * 0.7,
                        groove,
                        false,
                    );
                }
                // The open bottom: nothing is printed there but the fuel arc.
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_out, 0.0, 1.0),
                    weight * 1.6,
                    scale_alpha(ink, 0.85),
                    true,
                );
                fill(w, o, &disc(c, c, r_in, 96), face);
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_in, 0.0, 1.0),
                    weight * 1.2,
                    scale_alpha(ink, 0.7),
                    true,
                );
                // Red band along the rim.
                for (a, z) in &reds {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r_out - (r_out - r_in) * 0.14, at(*a), at(*z)),
                        (r_out - r_in) * 0.28,
                        danger,
                        false,
                    );
                }
                // Fine ticks on the outer rim.
                for (f, major) in &ticks {
                    let len = if *major {
                        (r_out - r_in) * 0.2
                    } else {
                        (r_out - r_in) * 0.11
                    };
                    let seg = [
                        polar(c, c, r_out - weight * 1.4, at(*f)),
                        polar(c, c, r_out - weight * 1.4 - len, at(*f)),
                    ];
                    stroke(
                        w,
                        o,
                        &seg,
                        if *major { weight } else { weight * 0.6 },
                        scale_alpha(ink, 0.8),
                        false,
                    );
                }
                // Track and value inside the ring.
                let rt = r_in - weight * 4.0;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, rt, at(0.0), at(1.0)),
                    weight * 2.0,
                    scale_alpha(ink, 0.22),
                    false,
                );
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, rt, at(0.0), at(frac)),
                    weight * 2.0,
                    ink,
                    false,
                );
                if let Some(m) = marker {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, r_in + weight, at(m) - 0.006, at(m) + 0.006),
                        weight * 3.5,
                        go,
                        false,
                    );
                }
                // The needle: a short bar across the ring.
                let (p0, p1) = (
                    polar(c, c, r_in - weight * 2.0, at(frac)),
                    polar(c, c, r_out, at(frac)),
                );
                stroke(w, o, &[p0, p1], weight * 2.4, ink, false);
                // Fuel arc across the open bottom, with its own end stops.
                if let Some(fuel) = fuel {
                    let (f0, f1) = (0.405, 0.595);
                    let rf = r_out - weight * 1.5;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, rf, f0, f1),
                        weight * 3.0,
                        scale_alpha(ink, 0.25),
                        false,
                    );
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, rf, f0, f0 + (f1 - f0) * fuel),
                        weight * 3.0,
                        ink,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut ring = div().relative().w(px(size)).h(px(size)).child(cv);
        for k in self.scale.ticks().into_iter().filter(|k| k.major) {
            let (x, y) = polar(c, c, (r_out + r_in) / 2.0 - size * 0.004, at(k.frac));
            ring = ring.child(
                text_box(x, y - size * 0.035, size * 0.12)
                    .text_size(px(size * 0.062))
                    .text_color(ink)
                    .font_weight(FontWeight::MEDIUM)
                    .child(readout(k.value, 0, "")),
            );
        }
        ring.child(
            text_box(c, c - size * 0.2, size * 0.5)
                .text_size(px(size * 0.2))
                .text_color(ink)
                .font_weight(FontWeight::SEMIBOLD)
                .child(readout(self.speed, 0, "")),
        )
        .child(
            text_box(c, c + size * 0.045, size * 0.4)
                .text_size(px(size * 0.05))
                .text_color(ink)
                .child(self.unit.clone()),
        )
        .when(!self.tab.is_empty(), |d| {
            d.child(
                div()
                    .absolute()
                    .left(px(c - size * 0.15))
                    .top(px(c + size * 0.12))
                    .w(px(size * 0.3))
                    .h(px(size * 0.085))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(size * 0.012))
                    .bg(hsla(t.palette.background.mix(t.palette.text, 0.14)))
                    .text_size(px(size * 0.05))
                    .text_color(go)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.tab.clone()),
            )
        })
        .when(self.fuel.is_some(), |d| {
            d.child(
                div()
                    .absolute()
                    .left(px(c - size * 0.2))
                    .top(px(c + size * 0.255))
                    .w(px(size * 0.4))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(size * 0.015))
                    .text_size(px(size * 0.052))
                    .text_color(ink)
                    .child(Icon::symbol(Symbol::Fuel).size(size * 0.065).color(muted))
                    .child(self.reach.clone()),
            )
        })
    }
}

// ───────────────────────────────── SidePod ─────────────────────────────────

/// An open-ring pod: a bright rim with a scale and numbers inside it, a needle that crosses the
/// rim, and room in the middle for an icon, a unit and a reading (or anything else).
#[derive(IntoElement)]
pub struct SidePod {
    follow: Follow,
    scale: Scale,
    value: f32,
    icon: Option<Symbol>,
    unit: SharedString,
    reading: SharedString,
    ends: [Option<Tone>; 2],
    centre: Option<AnyElement>,
    needle: bool,
    style: PodStyle,
    size: f32,
}

impl SidePod {
    follow_api!();
    /// A pod for `value` on `scale`.
    #[track_caller]
    pub fn new(scale: Scale, value: f32) -> Self {
        Self {
            follow: Follow::here(),
            scale,
            value,
            icon: None,
            unit: SharedString::default(),
            reading: SharedString::default(),
            ends: [None, None],
            centre: None,
            needle: true,
            size: 300.0,
            style: PodStyle::Ring,
        }
    }
    /// Show or hide the needle (a speed scale with no needle only carries numbers).
    pub fn needle(mut self, on: bool) -> Self {
        self.needle = on;
        self
    }
    /// The symbol in the middle.
    pub fn icon(mut self, i: Symbol) -> Self {
        self.icon = Some(i);
        self
    }
    /// The unit under the symbol.
    pub fn unit(mut self, u: impl Into<SharedString>) -> Self {
        self.unit = u.into();
        self
    }
    /// A written reading in the middle.
    pub fn reading(mut self, r: impl Into<SharedString>) -> Self {
        self.reading = r.into();
        self
    }
    /// Colour the two ends of the rim (a cold end, a hot end).
    pub fn ends(mut self, low: Option<Tone>, high: Option<Tone>) -> Self {
        self.ends = [low, high];
        self
    }
    /// Replace the icon, unit and reading with your own content.
    pub fn centre(mut self, e: impl IntoElement) -> Self {
        self.centre = Some(e.into_any_element());
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(120.0);
        self
    }
}

impl RenderOnce for SidePod {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.value = follow(w, cx, &self.follow, 0, self.value);
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (start, span) = SWEEP;
        let at = move |f: f32| start + span * f;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let frac = self.scale.fraction(self.value);
        let r_rim = c - size * 0.012;
        let width = (size * 0.014).max(2.0);
        let ends = self.ends.map(|e| e.map(|tone| tone.color(&t)));
        let frame = Frame::new(&t, self.style);
        let zones: Vec<(f32, f32, Hsla)> = self
            .scale
            .zone_spans()
            .into_iter()
            .filter(|z| z.2 != Severity::Normal)
            .map(|z| (z.0, z.1, severity_color(&t, z.2)))
            .collect();
        let needle = self.needle;
        let ticks: Vec<(f32, bool)> = self
            .scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                frame.paint(w, o, c, r_rim, width, ends);
                let rt = r_rim - width * 3.0;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, rt, at(0.0), at(1.0)),
                    width * 1.4,
                    scale_alpha(ink, 0.18),
                    false,
                );
                for (a, z, col) in &zones {
                    stroke(
                        w,
                        o,
                        &turn_arc(c, c, rt, at(*a), at(*z)),
                        width * 1.4,
                        *col,
                        false,
                    );
                }
                for (f, major) in &ticks {
                    let len = if *major { size * 0.035 } else { size * 0.018 };
                    let seg = [
                        polar(c, c, rt - width, at(*f)),
                        polar(c, c, rt - width - len, at(*f)),
                    ];
                    stroke(
                        w,
                        o,
                        &seg,
                        if *major { width * 0.8 } else { width * 0.5 },
                        scale_alpha(ink, 0.85),
                        false,
                    );
                }
                if needle {
                    let (p0, p1) = (
                        polar(c, c, rt - size * 0.1, at(frac)),
                        polar(c, c, r_rim + width * 0.5, at(frac)),
                    );
                    stroke(w, o, &[p0, p1], width * 1.4, ink, false);
                }
            },
        )
        .absolute()
        .size_full();
        let mut pod = div().relative().w(px(size)).h(px(size)).child(cv);
        for k in self.scale.ticks().into_iter().filter(|k| k.major) {
            let (x, y) = polar(c, c, r_rim - size * 0.15, at(k.frac));
            pod = pod.child(
                text_box(x, y - size * 0.03, size * 0.14)
                    .text_size(px(size * 0.052))
                    .text_color(ink)
                    .child(readout(k.value, 0, "")),
            );
        }
        let middle = match self.centre.take() {
            Some(e) => e,
            None => {
                let mut col = div().flex().flex_col().items_center().gap(px(size * 0.012));
                if let Some(i) = self.icon {
                    col = col.child(Icon::symbol(i).size(size * 0.1).color(ink));
                }
                if !self.reading.is_empty() {
                    col = col.child(
                        div()
                            .text_size(px(size * 0.075))
                            .text_color(ink)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.reading.clone()),
                    );
                }
                col.child(
                    div()
                        .text_size(px(size * 0.05))
                        .text_color(muted)
                        .child(self.unit.clone()),
                )
                .into_any_element()
            }
        };
        pod.child(
            div()
                .absolute()
                .left(px(size * 0.2))
                .top(px(size * 0.28))
                .w(px(size * 0.6))
                .h(px(size * 0.4))
                .flex()
                .items_center()
                .justify_center()
                .child(middle),
        )
    }
}

// ───────────────────────────────── MenuRing ─────────────────────────────────

/// An open-ring pod holding a short menu: a heading, rows, and one row selected.
#[derive(IntoElement)]
pub struct MenuRing {
    heading: SharedString,
    rows: Vec<SharedString>,
    selected: usize,
    chosen: Option<usize>,
    pressed: bool,
    look: Look,
    style: PodStyle,
    size: f32,
}

impl MenuRing {
    /// A menu with `rows`; the first row (index 0) is selected.
    pub fn new(rows: impl IntoIterator<Item = impl Into<SharedString>>) -> Self {
        Self {
            heading: SharedString::default(),
            rows: rows.into_iter().map(Into::into).collect(),
            selected: 0,
            chosen: None,
            pressed: false,
            size: 300.0,
            look: Look::Classic,
            style: PodStyle::Ring,
        }
    }
    /// A heading above the rows, in the info colour.
    pub fn heading(mut self, h: impl Into<SharedString>) -> Self {
        self.heading = h.into();
        self
    }
    /// The row under the cursor, drawn as a bar.
    pub fn selected(mut self, i: usize) -> Self {
        self.selected = i;
        self
    }
    /// The row that is switched on: its text takes the info colour.
    pub fn chosen(mut self, i: usize) -> Self {
        self.chosen = Some(i);
        self
    }
    /// The cursor bar while the button is held: it fills with the primary colour.
    pub fn pressed(mut self, on: bool) -> Self {
        self.pressed = on;
        self
    }
    /// The look: the round pod (default), a watch case or a heads-up display.
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }
    /// How the frame is drawn (open ring, closed disc, glow or bezel).
    pub fn style(mut self, style: PodStyle) -> Self {
        self.style = style;
        self
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(120.0);
        self
    }
}

impl RenderOnce for MenuRing {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        if self.look != Look::Classic {
            return self.detailed(&t);
        }
        let size = self.size;
        let c = size / 2.0;
        let ink = hsla(t.palette.text);
        let info = hsla(t.palette.info);
        let bar = hsla(t.palette.surface_raised.mix(t.palette.text, 0.14));
        let width = (size * 0.014).max(2.0);
        let r_rim = c - size * 0.012;
        let frame = Frame::new(&t, self.style);
        let row_h = size * 0.13;
        let top = c - row_h * 1.6;
        // The selected row's band, cut to the inside of the rim so it never pokes out.
        let first = usize::from(!self.heading.is_empty());
        let band = (self.selected < self.rows.len()).then(|| {
            let y0 = top + row_h * (first + self.selected) as f32;
            crate::shapes::chord_band(c, r_rim - width, y0, y0 + row_h)
        });
        let (pressed, primary) = (self.pressed, hsla(t.palette.primary));
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let o = b.origin;
                if let Some((poly, left, right)) = &band {
                    if pressed {
                        fill(w, o, poly, scale_alpha(primary, 0.35));
                        stroke(w, o, left, width, info, false);
                        stroke(w, o, right, width, info, false);
                    } else {
                        fill(w, o, poly, bar);
                    }
                }
                frame.paint(w, o, c, r_rim, width, [None, None]);
            },
        )
        .absolute()
        .size_full();
        let mut list = div()
            .absolute()
            .left(px(size * 0.04))
            .top(px(top))
            .w(px(size * 0.92))
            .flex()
            .flex_col();
        if !self.heading.is_empty() {
            list = list.child(
                div()
                    .h(px(row_h))
                    .flex()
                    .items_center()
                    .px(px(size * 0.07))
                    .text_size(px(size * 0.062))
                    .text_color(info)
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.heading.clone()),
            );
        }
        for (i, row) in self.rows.iter().enumerate() {
            let on = i == self.selected;
            list = list.child(
                div()
                    .h(px(row_h))
                    .flex()
                    .items_center()
                    .px(px(size * 0.07))
                    .text_size(px(size * 0.062))
                    .text_color(if self.chosen == Some(i) { info } else { ink })
                    .when(on, |d| d.font_weight(FontWeight::MEDIUM))
                    .child(row.clone()),
            );
        }
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .child(list)
            .into_any_element()
    }
}

// ───────────────────────────────── LimitSign ─────────────────────────────────

/// A round speed-limit sign: a heavy red ring around the limit.
#[derive(IntoElement)]
pub struct LimitSign {
    limit: u32,
    size: f32,
}

impl LimitSign {
    /// A sign for `limit`.
    pub fn new(limit: u32) -> Self {
        Self { limit, size: 40.0 }
    }
    /// Diameter in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(16.0);
        self
    }
}

impl RenderOnce for LimitSign {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let size = self.size;
        let c = size / 2.0;
        let (danger, face, ink) = (
            hsla(t.palette.danger),
            hsla(t.palette.text),
            hsla(t.palette.background),
        );
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                fill(w, b.origin, &disc(c, c, c - 0.5, 64), danger);
                fill(w, b.origin, &disc(c, c, c * 0.72, 64), face);
            },
        )
        .absolute()
        .size_full();
        div()
            .relative()
            .w(px(size))
            .h(px(size))
            .flex()
            .items_center()
            .justify_center()
            .child(cv)
            .child(
                div()
                    .relative()
                    .text_size(px(size * if self.limit >= 100 { 0.3 } else { 0.38 }))
                    .text_color(ink)
                    .font_weight(FontWeight::BOLD)
                    .child(self.limit.to_string()),
            )
    }
}

// ───────────────────────────────── Telltales ─────────────────────────────────

/// A row of dashboard symbols, each lit in the colour of what it reports.
#[derive(IntoElement)]
pub struct Telltales {
    items: Vec<(Symbol, Tone)>,
    size: f32,
    columns: usize,
}

impl Telltales {
    /// An empty row.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            size: 22.0,
            columns: usize::MAX,
        }
    }
    /// Add a symbol in a tone.
    pub fn push(mut self, icon: Symbol, tone: Tone) -> Self {
        self.items.push((icon, tone));
        self
    }
    /// Symbol size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(10.0);
        self
    }
    /// Wrap into rows of this many symbols.
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
}

impl Default for Telltales {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Telltales {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let gap = self.size * 0.3;
        let mut rows = div().flex().flex_col().gap(px(gap)).items_center();
        let mut row = div().flex().gap(px(gap));
        let mut n = 0;
        for (icon, tone) in &self.items {
            row = row.child(
                Icon::symbol(*icon)
                    .size(self.size)
                    .weight(2.0)
                    .color(tone.color(&t)),
            );
            n += 1;
            if n == self.columns {
                rows = rows.child(std::mem::replace(&mut row, div().flex().gap(px(gap))));
                n = 0;
            }
        }
        if n > 0 {
            rows = rows.child(row);
        }
        rows
    }
}

/// A pod a [`DashTriptych`] can hold.
pub enum Pod {
    /// A gauge pod.
    Side(SidePod),
    /// A menu pod.
    Menu(MenuRing),
    /// Any other pod (see [`crate::pods`]): built at the size the dash gives it.
    Custom(Box<dyn FnOnce(f32) -> AnyElement>),
}

impl From<SidePod> for Pod {
    fn from(p: SidePod) -> Self {
        Pod::Side(p)
    }
}

impl From<MenuRing> for Pod {
    fn from(p: MenuRing) -> Self {
        Pod::Menu(p)
    }
}

impl Pod {
    pub(crate) fn sized(self, size: f32) -> AnyElement {
        match self {
            Pod::Side(p) => p.size(size).into_any_element(),
            Pod::Menu(p) => p.size(size).into_any_element(),
            Pod::Custom(f) => f(size),
        }
    }
}

// ───────────────────────────────── DashTriptych ─────────────────────────────────

/// A whole dash: [`RingTach`] in the middle, a pod on each side, telltales between, and the
/// outside temperature, clock, odometer, drive mode and gear around the edges.
#[derive(IntoElement)]
pub struct DashTriptych {
    centre: RingTach,
    left: Pod,
    right: Pod,
    telltales: Telltales,
    limit: Option<u32>,
    mode: SharedString,
    gear: char,
    odometer: SharedString,
    temp: SharedString,
    clock: SharedString,
    width: f32,
}

impl DashTriptych {
    /// A dash around `centre`, with a [`SidePod`] or [`MenuRing`] on each side (the dash sizes them).
    pub fn new(centre: RingTach, left: impl Into<Pod>, right: impl Into<Pod>) -> Self {
        Self {
            centre,
            left: left.into(),
            right: right.into(),
            telltales: Telltales::new(),
            limit: None,
            mode: SharedString::default(),
            gear: 'D',
            odometer: SharedString::default(),
            temp: SharedString::default(),
            clock: SharedString::default(),
            width: 1000.0,
        }
    }
    /// The telltales shown between the centre and the pods.
    pub fn telltales(mut self, t: Telltales) -> Self {
        self.telltales = t;
        self
    }
    /// The speed limit sign beside the centre.
    pub fn limit(mut self, l: u32) -> Self {
        self.limit = Some(l);
        self
    }
    /// The drive mode under the right pod.
    pub fn mode(mut self, m: impl Into<SharedString>) -> Self {
        self.mode = m.into();
        self
    }
    /// The engaged gear: `P`, `R`, `N` or `D`.
    pub fn gear(mut self, g: char) -> Self {
        self.gear = g;
        self
    }
    /// The small print under the left pod: odometer, outside temperature and clock.
    pub fn trip(
        mut self,
        odometer: impl Into<SharedString>,
        temp: impl Into<SharedString>,
        clock: impl Into<SharedString>,
    ) -> Self {
        self.odometer = odometer.into();
        self.temp = temp.into();
        self.clock = clock.into();
        self
    }
    /// Width in px; the height follows (0.46 of it).
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(600.0);
        self
    }
}

impl RenderOnce for DashTriptych {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let width = self.width;
        let h = width * 0.46;
        let ring = h * 0.94;
        let pod = h * 0.68;
        let (ink, muted) = (hsla(t.palette.text), hsla(t.palette.text_muted));
        let small = (h * 0.045).max(11.0);
        let gears = ['R', 'N', 'D'];
        let mut gear_col = div()
            .flex()
            .flex_col()
            .items_center()
            .text_size(px(small))
            .text_color(muted);
        for g in gears {
            let on = g == self.gear;
            gear_col = gear_col.child(
                div()
                    .text_color(if on { hsla(t.palette.info) } else { muted })
                    .font_weight(if on {
                        FontWeight::BOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .child(g.to_string()),
            );
        }
        let park = div()
            .flex()
            .items_center()
            .justify_center()
            .w(px(small * 1.5))
            .h(px(small * 1.5))
            .rounded(px(small * 0.25))
            .bg(if self.gear == 'P' {
                hsla(t.palette.info)
            } else {
                scale_alpha(muted, 0.35)
            })
            .text_color(hsla(t.palette.background))
            .text_size(px(small))
            .font_weight(FontWeight::BOLD)
            .child("P");
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
            // Pods first, so the ring overlaps them.
            .child(
                div()
                    .absolute()
                    .left(px(width * 0.008))
                    .top(px(h * 0.12))
                    .w(px(pod))
                    .h(px(pod))
                    .child(self.left.sized(pod)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(width - width * 0.008 - pod))
                    .top(px(h * 0.12))
                    .w(px(pod))
                    .h(px(pod))
                    .child(self.right.sized(pod)),
            )
            .child(
                div()
                    .absolute()
                    .left(px((width - ring) / 2.0))
                    .top(px(h * 0.015))
                    .child(self.centre.size(ring)),
            );
        // Telltales under the ring's left shoulder.
        root = root.child(
            div()
                .absolute()
                .left(px(width * 0.19))
                .top(px(h * 0.7))
                .child(self.telltales.size(h * 0.062).columns(3)),
        );
        if let Some(l) = self.limit {
            root = root.child(
                div()
                    .absolute()
                    .left(px(width * 0.245))
                    .top(px(h * 0.04))
                    .child(LimitSign::new(l).size(h * 0.1)),
            );
        }
        root.child(
            div()
                .absolute()
                .left(px(width * 0.03))
                .top(px(h * 0.82))
                .w(px(pod))
                .flex()
                .flex_col()
                .items_center()
                .text_size(px(small))
                .text_color(ink)
                .child(
                    div()
                        .text_size(px(small * 0.8))
                        .text_color(muted)
                        .child(format!("{} {}", self.temp, self.clock)),
                )
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child(self.odometer.clone()),
                ),
        )
        .child(
            div()
                .absolute()
                .left(px(width - width * 0.03 - pod))
                .top(px(h * 0.85))
                .w(px(pod))
                .flex()
                .justify_center()
                .text_size(px(small * 0.9))
                .text_color(ink)
                .font_weight(FontWeight::MEDIUM)
                .child(self.mode.clone()),
        )
        .child(
            div()
                .absolute()
                .right(px(width * 0.03))
                .top(px(h * 0.78))
                .child(gear_col),
        )
        .child(
            div()
                .absolute()
                .right(px(width * 0.065))
                .top(px(h * 0.8))
                .child(park),
        )
    }
}
