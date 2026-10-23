//! Form controls: [`Toggle`] (switch), [`Checkbox`], [`Radio`], [`ProgressBar`], [`Slider`].

use crate::interact::{track, use_interact};
use crate::paint::{round_shape, scaled_shape, shaped, shaped_id, ShapeStyle};
use crate::{hsla, theme_of};
use dodai_core::{ControlState, Theme, Tone};
use gpui::{
    canvas, div, px, relative, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Clamp to `0.0..=1.0` (NaN → 0).
pub fn clamp01(v: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Paint style of a toggle/checkbox/radio box (pure).
pub fn toggle_style(theme: &Theme, on: bool, state: ControlState) -> ShapeStyle {
    let k = theme.toggle(on, state);
    ShapeStyle::from_tokens(
        theme,
        Some(k.track),
        Some(k.border),
        on && state != ControlState::Disabled,
        false,
    )
}

/// Position of a value in `min..=max` as a 0..1 fraction.
pub fn slider_fraction(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        0.0
    } else {
        clamp01((value - min) / (max - min))
    }
}

/// Where a slider's value bubble sits relative to the thumb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubblePlacement {
    /// Above the thumb.
    Above,
    /// Below the thumb.
    Below,
}

/// Tick positions (`0..=1`) for `min..=max` at multiples of `step`;
/// empty when there would be more than 64 (too dense to draw). Pure.
pub fn tick_fractions(min: f32, max: f32, step: Option<f32>) -> Vec<f32> {
    let Some(step) = step.filter(|s| *s > 0.0 && max > min) else {
        return Vec::new();
    };
    let n = ((max - min) / step).round() as usize;
    if n == 0 || n > 64 {
        return Vec::new();
    }
    (0..=n)
        .map(|i| ((i as f32 * step) / (max - min)).clamp(0.0, 1.0))
        .collect()
}

/// Map a pointer x to a slider value. `left`/`width` are the control's bounds,
/// `thumb` the thumb diameter (the thumb centre travels `width - thumb`).
pub fn slider_value(
    x: f32,
    left: f32,
    width: f32,
    thumb: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> f32 {
    let span = (width - thumb).max(1.0);
    let f = clamp01((x - left - thumb / 2.0) / span);
    let mut v = min + f * (max - min);
    if let Some(s) = step.filter(|s| *s > 0.0) {
        v = min + ((v - min) / s).round() * s;
    }
    v.clamp(min.min(max), max.max(min))
}

type FloatHandler = Rc<dyn Fn(f32, &mut Window, &mut App)>;
type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

/// Size and look of a [`Checkbox`] / [`Radio`] under a Material (rounded) theme. Other design
/// systems draw their own box and ignore it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxVariant {
    /// M3 default: 18 px box inside a 40 px touch target with the state layer.
    #[default]
    Standard,
    /// 18 px box in a 24 px target (dense lists, tables); the state layer shrinks with it.
    Compact,
    /// Bare 18 px box, no state layer and no padding (inline text, tight rows).
    Dense,
    /// 24 px box in a 48 px target (touch-first UIs).
    Large,
    /// Standard size, circular box (selection circles, task lists).
    Round,
    /// Standard size; selected state uses the secondary container instead of `primary`.
    Tonal,
}

impl CheckboxVariant {
    /// Every variant, in display order.
    pub const ALL: [Self; 6] = [
        Self::Standard,
        Self::Compact,
        Self::Dense,
        Self::Large,
        Self::Round,
        Self::Tonal,
    ];
    /// `(touch target, box)` edge lengths in px; a target equal to the box has no state layer.
    pub fn dims(self) -> (f32, f32) {
        match self {
            Self::Standard | Self::Round | Self::Tonal => (40.0, 18.0),
            Self::Compact => (24.0, 18.0),
            Self::Dense => (18.0, 18.0),
            Self::Large => (48.0, 24.0),
        }
    }
    /// Human-readable name.
    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "Standard 40",
            Self::Compact => "Compact 24",
            Self::Dense => "Dense 18",
            Self::Large => "Large 48",
            Self::Round => "Round",
            Self::Tonal => "Tonal",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Switch,
    Checkbox,
    Radio,
}

#[allow(clippy::too_many_arguments)]
fn choice(
    kind: Kind,
    id: ElementId,
    on: bool,
    disabled: bool,
    handler: Option<BoolHandler>,
    radio: bool,
    mixed: bool,
    variant: CheckboxVariant,
    icons: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    let t = theme_of(cx);
    if t.look().tonal {
        return m3_choice(
            kind, id, on, disabled, handler, radio, mixed, variant, icons, window, cx,
        );
    }
    // A half-checked box is drawn "on" with a dash; clicking it selects all.
    let (on_value, on) = (on, on || mixed);
    let st = use_interact(&id, window, cx);
    let state = st.read(cx).control_state(disabled);
    let k = t.toggle(on, state);
    let p = crate::motion::transition(window, cx, id.clone(), on);
    let mut style = toggle_style(&t, on, state);
    {
        let (a, b) = (
            toggle_style(&t, false, state),
            toggle_style(&t, true, state),
        );
        style.fill = crate::motion::mix_opt(a.fill, b.fill, p);
        style.stroke = crate::motion::mix_opt(a.stroke, b.stroke, p);
    }
    // Physical feel: the thumb is a damped spring, so it overshoots slightly.
    let thumb = if kind == Kind::Switch {
        crate::motion::smooth_with(
            window,
            cx,
            (id.clone(), "thumb").into(),
            if on { 1.0 } else { 0.0 },
            crate::motion::Motion::new(
                std::time::Duration::from_millis(450),
                crate::motion::Easing::SPRING_STIFF,
            ),
        )
    } else {
        p
    };
    let (w, h) = match kind {
        Kind::Switch => (42.0, 22.0),
        _ => (t.sizes().selection_box, t.sizes().selection_box),
    };
    let shape = match kind {
        Kind::Switch | Kind::Radio => round_shape(&t.shape, h),
        Kind::Checkbox if t.look().tonal => t.role_shape(dodai_core::ShapeRole::Checkbox),
        Kind::Checkbox => scaled_shape(&t.shape, h),
    };
    let mut el = track(shaped_id(id, &shape, style), &st)
        .w(px(w))
        .h(px(h))
        .flex_none()
        .flex()
        .items_center()
        .justify_center();
    let mark = ShapeStyle::from_tokens(&t, Some(k.thumb), None, false, false);
    el = match kind {
        Kind::Switch => {
            let tshape = round_shape(&t.shape, 14.0);
            el.child(
                shaped(&tshape, mark)
                    .absolute()
                    .top(px(3.0))
                    .left(px(crate::motion::lerp(3.0, w - 14.0 - 3.0 - 1.0, thumb)))
                    .w(px(14.0))
                    .h(px(14.0)),
            )
        }
        Kind::Checkbox => {
            if p > 0.01 {
                el.text_color(hsla(k.thumb).opacity(p))
                    .text_size(px(h - 6.0))
                    .child(if mixed { "−" } else { "✓" })
            } else {
                el
            }
        }
        Kind::Radio => {
            if p > 0.01 {
                let tshape = round_shape(&t.shape, 8.0);
                el.child(shaped(&tshape, mark).w(px(8.0 * p)).h(px(8.0 * p)))
            } else {
                el
            }
        }
    };
    if !disabled {
        el = el.cursor_pointer();
        if let Some(f) = handler {
            el = el
                .on_click(move |_, w, cx| f(if radio || mixed { true } else { !on_value }, w, cx));
        }
    }
    el.into_any_element()
}

/// Material 3 switch (52×32), checkbox (18 px box) and radio (20 px ring), each with the 40 px
/// state layer; colours from [`Theme::choice_toggle`].
#[allow(clippy::too_many_arguments)]
fn m3_choice(
    kind: Kind,
    id: ElementId,
    on: bool,
    disabled: bool,
    handler: Option<BoolHandler>,
    radio: bool,
    mixed: bool,
    variant: CheckboxVariant,
    icons: bool,
    window: &mut Window,
    cx: &mut App,
) -> gpui::AnyElement {
    use crate::motion::{lerp, mix, smooth_with, transition, Easing, Motion};
    use dodai_core::ToggleKind;
    let t = theme_of(cx);
    let r = t.palette.roles.clone();
    let (on_value, on) = (on, on || mixed);
    let tk = match kind {
        Kind::Switch => ToggleKind::Switch,
        Kind::Checkbox => ToggleKind::Checkbox,
        Kind::Radio => ToggleKind::Radio,
    };
    let st = use_interact(&id, window, cx);
    let state = st.read(cx).control_state(disabled);
    let p = transition(window, cx, id.clone(), on);
    let (off_k, mut on_k) = (
        t.choice_toggle(tk, false, state),
        t.choice_toggle(tk, true, state),
    );
    let tonal = variant == CheckboxVariant::Tonal && kind == Kind::Checkbox && !disabled;
    if tonal {
        on_k.track = r.secondary_container;
        on_k.thumb = r.on_secondary_container;
    }
    let sz = t.sizes();
    let (target, bx) = if kind == Kind::Switch {
        (sz.state_layer, 18.0)
    } else {
        variant.dims()
    };
    let m = |a: dodai_core::Color, b: dodai_core::Color| mix(hsla(a), hsla(b), p);
    let (track_c, border_c, thumb_c) = (
        m(off_k.track, on_k.track),
        m(off_k.border, on_k.border),
        m(off_k.thumb, on_k.thumb),
    );
    let layer_alpha = match state {
        _ if target <= bx => 0.0,
        ControlState::Hovered => 0.08,
        ControlState::Pressed | ControlState::Focused => 0.10,
        _ => 0.0,
    };
    let layer_c = hsla(if on { t.palette.primary } else { r.on_surface }).opacity(layer_alpha);
    let layer_style = |d: f32| {
        let mut s = ShapeStyle::from_tokens(&t, None, None, false, false);
        s.fill = Some(layer_c);
        (round_shape(&t.shape, d), s)
    };
    let mut el = match kind {
        Kind::Switch => {
            let thumb = smooth_with(
                window,
                cx,
                (id.clone(), "thumb").into(),
                if on { 1.0 } else { 0.0 },
                Motion::new(std::time::Duration::from_millis(450), Easing::SPRING_STIFF),
            );
            let ((tw, th), (off_d, on_d)) = (sz.switch_track, sz.switch_handle);
            let size = if icons {
                on_d
            } else {
                lerp(off_d, on_d, p.clamp(0.0, 1.0))
            };
            let centre = lerp(th / 2.0, tw - th / 2.0, thumb);
            // Absolute children sit inside the 2 px outline.
            let (ls, lst) = layer_style(sz.state_layer);
            let mut track_style = ShapeStyle::from_tokens(&t, None, None, false, false);
            track_style.fill = Some(track_c);
            track_style.stroke = Some(border_c);
            track_style.stroke_width = 2.0;
            let mut handle = ShapeStyle::from_tokens(&t, None, None, false, false);
            handle.fill = Some(thumb_c);
            track(shaped_id(id, &round_shape(&t.shape, th), track_style), &st)
                .w(px(tw))
                .h(px(th))
                .flex_none()
                .child(
                    shaped(&ls, lst)
                        .absolute()
                        .left(px(centre - sz.state_layer / 2.0 - 2.0))
                        .top(px(-(sz.state_layer - th) / 2.0 - 2.0))
                        .w(px(sz.state_layer))
                        .h(px(sz.state_layer)),
                )
                .child({
                    let mut h = shaped(&round_shape(&t.shape, size), handle)
                        .absolute()
                        .left(px(centre - size / 2.0 - 2.0))
                        .top(px((th - size) / 2.0 - 2.0))
                        .w(px(size))
                        .h(px(size));
                    if icons {
                        // M3 switch with icons: a check when on, a cross when off.
                        let glyph_c = if on {
                            r.on_primary_container
                        } else {
                            r.surface_container_highest
                        };
                        h = h
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(14.0))
                            .text_color(hsla(glyph_c))
                            .child(if on { "✓" } else { "✕" });
                    }
                    h
                })
        }
        Kind::Checkbox | Kind::Radio => {
            let (ls, lst) = layer_style(target);
            let outer = track(shaped_id(id, &ls, lst), &st)
                .w(px(target))
                .h(px(target))
                .flex_none()
                .flex()
                .items_center()
                .justify_center();
            let mut ring = ShapeStyle::from_tokens(&t, None, None, false, false);
            ring.fill = Some(track_c);
            ring.stroke = Some(border_c);
            ring.stroke_width = 2.0;
            if kind == Kind::Checkbox {
                // Selected: no outline (the container is filled).
                if p > 0.5 {
                    ring.stroke = None;
                }
                let box_shape = if variant == CheckboxVariant::Round {
                    round_shape(&t.shape, bx)
                } else {
                    t.role_shape(dodai_core::ShapeRole::Checkbox)
                };
                let mut b = shaped(&box_shape, ring)
                    .w(px(bx))
                    .h(px(bx))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(thumb_c)
                    .text_size(px(bx * 0.78));
                if p > 0.01 {
                    b = b.child(if mixed { "−" } else { "✓" });
                }
                outer.child(b)
            } else {
                let d = bx + 2.0;
                let mut b = shaped(&round_shape(&t.shape, d), ring)
                    .w(px(d))
                    .h(px(d))
                    .flex()
                    .items_center()
                    .justify_center();
                if p > 0.01 {
                    let mut dot = ShapeStyle::from_tokens(&t, None, None, false, false);
                    dot.fill = Some(thumb_c);
                    b = b.child(
                        shaped(&round_shape(&t.shape, d / 2.0), dot)
                            .w(px(d / 2.0 * p))
                            .h(px(d / 2.0 * p)),
                    );
                }
                outer.child(b)
            }
        }
    };
    if !disabled {
        el = el.cursor_pointer();
        if let Some(f) = handler {
            el = el
                .on_click(move |_, w, cx| f(if radio || mixed { true } else { !on_value }, w, cx));
        }
    }
    el.into_any_element()
}

macro_rules! choice_component {
    ($(#[$m:meta])* $name:ident, $kind:expr, $radio:expr) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            id: ElementId,
            on: bool,
            disabled: bool,
            mixed: bool,
            variant: CheckboxVariant,
            icons: bool,
            on_change: Option<BoolHandler>,
        }

        impl $name {
            /// `on` is the current (controlled) value.
            pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
                Self { id: id.into(), on, disabled: false, mixed: false, variant: CheckboxVariant::Standard, icons: false, on_change: None }
            }
            /// Disable interaction.
            pub fn disabled(mut self, d: bool) -> Self {
                self.disabled = d;
                self
            }
            /// Half-checked ("some children selected"): drawn as a dash;
            /// clicking requests `true`. Meaningful for [`Checkbox`] only.
            pub fn indeterminate(mut self, m: bool) -> Self {
                self.mixed = m;
                self
            }
            /// Size / look under Material themes; see [`CheckboxVariant`]. Meaningful for
            /// [`Checkbox`] and [`Radio`] (sizes only); other design systems ignore it.
            pub fn variant(mut self, v: CheckboxVariant) -> Self {
                self.variant = v;
                self
            }
            /// Show a check / cross in the thumb (Material switch with icons); [`Toggle`] only.
            pub fn icons(mut self, on: bool) -> Self {
                self.icons = on;
                self
            }
            /// Called with the requested new value on click.
            pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
                self.on_change = Some(Rc::new(f));
                self
            }
        }

        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                choice($kind, self.id, self.on, self.disabled, self.on_change, $radio, self.mixed, self.variant, self.icons, window, cx)
            }
        }
    };
}

choice_component!(
    /// On/off switch.
    Toggle, Kind::Switch, false
);
choice_component!(
    /// Checkbox with a ✓ mark.
    Checkbox, Kind::Checkbox, false
);
choice_component!(
    /// Radio button; `on_change` is called with `true` when selected.
    Radio, Kind::Radio, true
);

fn bar(t: &Theme, frac: f32, tone: Option<Tone>, height: f32) -> gpui::Div {
    let k = t.progress(tone);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, height);
    let mut d = shaped(&shape, track_style).w_full().h(px(height));
    let f = clamp01(frac);
    let n = k.segments;
    if f > 0.0 {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .top_0()
                .left_0()
                .h_full()
                .w(relative(f)),
        );
    }
    // HUD gauge: thin gaps in the track colour cut the bar into segments.
    for i in 1..n {
        d = d.child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .left(relative(i as f32 / n as f32))
                .w(px(3.0))
                .bg(hsla(k.track)),
        );
    }
    d
}

/// Track whose fill spans from the middle to `frac`.
fn centered_bar(t: &Theme, frac: f32, height: f32) -> gpui::Div {
    let k = t.progress(None);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, height);
    let f = clamp01(frac);
    let (a, b) = (f.min(0.5), f.max(0.5));
    let mut d = shaped(&shape, track_style).w_full().h(px(height));
    if b > a {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .top_0()
                .h_full()
                .left(relative(a))
                .w(relative(b - a)),
        );
    }
    d
}

fn vbar(t: &Theme, frac: f32, width: f32) -> gpui::Div {
    let k = t.progress(None);
    let track_style = ShapeStyle::from_tokens(t, Some(k.track), Some(k.border), false, false);
    let shape = crate::paint::round_shape(&t.shape, width);
    let mut d = shaped(&shape, track_style).h_full().w(px(width));
    let f = clamp01(frac);
    if f > 0.0 {
        let fill = ShapeStyle::from_tokens(t, Some(k.fill), None, true, false);
        d = d.child(
            shaped(&shape, fill)
                .absolute()
                .bottom_0()
                .left_0()
                .w_full()
                .h(relative(f)),
        );
    }
    d
}

/// Horizontal progress bar (`0.0..=1.0`).
#[derive(IntoElement)]
pub struct ProgressBar {
    value: f32,
    tone: Option<Tone>,
    wavy: bool,
}

impl ProgressBar {
    /// Create a new [`ProgressBar`].
    pub fn new(value: f32) -> Self {
        Self {
            value,
            tone: None,
            wavy: false,
        }
    }
    /// Colour by status tone (default: brand colour).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
    /// Material 3 Expressive wavy indicator: the active part is a travelling sine wave.
    /// Other design systems draw their usual bar.
    pub fn wavy(mut self, on: bool) -> Self {
        self.wavy = on;
        self
    }
}

/// Wavy linear indicator: straight track from `frac` on, a travelling sine wave before it.
fn wavy_bar(t: &Theme, frac: f32, tone: Option<Tone>) -> gpui::Div {
    use gpui::{point, PathBuilder};
    let k = t.progress(tone);
    let (track, fill) = (hsla(k.track), hsla(k.fill));
    let frac = clamp01(frac);
    div().w_full().h(px(14.0)).relative().child(
        canvas(
            |_, _, _| (),
            move |b, _, window, _| {
                let w: f32 = b.size.width.into();
                let (ox, cy) = (b.origin.x, b.origin.y + b.size.height / 2.0);
                let stroke = 4.0;
                let active = (w * frac).max(0.0);
                // Straight remainder of the track, after a small gap.
                let from = (active + stroke * 2.0).min(w);
                if from < w {
                    let mut pb = PathBuilder::stroke(px(stroke));
                    pb.move_to(point(ox + px(from), cy));
                    pb.line_to(point(ox + px(w), cy));
                    if let Ok(p) = pb.build() {
                        window.paint_path(p, track);
                    }
                }
                if active > 0.5 {
                    let (amp, len) = (3.0_f32, 36.0_f32);
                    let phase = crate::display::elapsed_secs() * 4.0;
                    let mut pb = PathBuilder::stroke(px(stroke));
                    let steps = (active / 2.0).ceil().max(2.0) as usize;
                    for i in 0..=steps {
                        let x = active * i as f32 / steps as f32;
                        let y = amp * (x / len * std::f32::consts::TAU - phase).sin();
                        let p = point(ox + px(x), cy + px(y));
                        if i == 0 {
                            pb.move_to(p);
                        } else {
                            pb.line_to(p);
                        }
                    }
                    if let Ok(p) = pb.build() {
                        window.paint_path(p, fill);
                    }
                    crate::motion::next_frame(window);
                }
            },
        )
        .absolute()
        .size_full(),
    )
}

impl ProgressBar {
    fn build(self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "ProgressBar");
        if self.wavy && t.look().tonal {
            return wavy_bar(&t, self.value * g, self.tone).into_any_element();
        }
        bar(
            &t,
            self.value * g,
            self.tone,
            if t.look().tonal {
                4.0
            } else {
                t.spacing.sm + 2.0
            },
        )
        .into_any_element()
    }
}

crate::entry_render!(ProgressBar);

#[derive(Default)]
struct SliderState {
    bounds: Bounds<Pixels>,
    dragging: bool,
}

/// Axis of a [`Slider`]. Vertical sliders grow upwards (min at the bottom).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Orientation {
    /// Left to right.
    #[default]
    Horizontal,
    /// Bottom to top.
    Vertical,
}

/// Draggable slider over `min..=max` (default `0..=1`). Controlled.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    direction: Orientation,
    length: f32,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    ticks: bool,
    show_value: bool,
    centered: bool,
    icon: Option<crate::icons::IconName>,
    bubble: Option<BubblePlacement>,
    format: Option<Rc<dyn Fn(f32) -> String>>,
    on_change: Option<FloatHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Slider);

impl Slider {
    /// Create a new [`Slider`].
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            direction: Orientation::Horizontal,
            length: 160.0,
            value,
            min: 0.0,
            max: 1.0,
            step: None,
            ticks: false,
            show_value: false,
            centered: false,
            icon: None,
            bubble: None,
            format: None,
            on_change: None,
        }
    }
    /// Draw a tick mark at every `step` (needs [`Slider::step`]).
    pub fn ticks(mut self, on: bool) -> Self {
        self.ticks = on;
        self
    }
    /// Centred slider: the active track grows from the middle of the range towards the handle
    /// (horizontal only), for values that go both ways such as balance.
    pub fn centered(mut self, on: bool) -> Self {
        self.centered = on;
        self
    }
    /// Material 3 Expressive slider with an icon inset at the start of a tall track and a
    /// slim handle (horizontal, tonal themes only).
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Show the current value next to the slider.
    pub fn show_value(mut self, on: bool) -> Self {
        self.show_value = on;
        self
    }
    /// Value bubble that follows the thumb, above or below it (horizontal only).
    pub fn bubble(mut self, p: BubblePlacement) -> Self {
        self.bubble = Some(p);
        self
    }
    /// Custom value formatter for the readout (e.g. `|v| format!("{v:.0} dB")`).
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.format = Some(Rc::new(f));
        self
    }
    /// Horizontal (default, full width) or vertical (fixed [`Slider::length`]).
    pub fn orientation(mut self, d: Orientation) -> Self {
        self.direction = d;
        self
    }
    /// Height in px of a vertical slider (default 160).
    pub fn length(mut self, px: f32) -> Self {
        self.length = px;
        self
    }
    /// Value range.
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Snap to multiples of `step` from `min`.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step);
        self
    }
    crate::callback!(
        /// Called with the new value while dragging/clicking.
        on_change, f32
    );
}

impl Slider {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let tonal = t.look().tonal;
        // M3: 20 px handle on a 4 px track.
        let vertical = self.direction == Orientation::Vertical;
        let inset = tonal && !vertical && self.icon.is_some();
        let thumb_d = if inset {
            4.0
        } else if tonal {
            20.0
        } else {
            t.sizes().slider_thumb
        };
        let track_h = if inset {
            40.0
        } else if tonal {
            4.0
        } else {
            6.0
        };
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| SliderState::default());
        let raw = slider_fraction(self.value, self.min, self.max);
        let frac = crate::motion::smooth(window, cx, self.id.clone(), raw)
            * crate::motion::grow(window, cx, "Slider");
        let (min, max, step) = (self.min, self.max, self.step);
        let apply = {
            let st = st.clone();
            let f = self.on_change.clone();
            move |p: gpui::Point<Pixels>, w: &mut Window, cx: &mut App| {
                let b = st.read(cx).bounds;
                if let Some(f) = &f {
                    // Position along the axis, measured from the min end.
                    let (pos, len): (f32, f32) = if vertical {
                        (
                            (b.origin.y + b.size.height - p.y).into(),
                            b.size.height.into(),
                        )
                    } else {
                        ((p.x - b.origin.x).into(), b.size.width.into())
                    };
                    f(slider_value(pos, 0.0, len, thumb_d, min, max, step), w, cx);
                }
            }
        };
        let rec = st.clone();
        let (down, mv, up, up2) = (st.clone(), st.clone(), st.clone(), st.clone());
        let (apply_down, apply_move) = (apply.clone(), apply);
        let k = t.toggle(true, ControlState::Idle);
        let thumb_style = if tonal {
            let mut s = ShapeStyle::from_tokens(&t, Some(t.palette.primary), None, false, false);
            s = s.with_lift(&t, 1);
            s
        } else {
            ShapeStyle::from_tokens(&t, Some(k.track), Some(k.border), true, false)
        };
        let thumb_shape = round_shape(&t.shape, thumb_d);
        let base = div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .cursor_pointer();
        let base = if vertical {
            base.w(px(thumb_d + 4.0))
                .h(px(self.length))
                .justify_center()
        } else {
            base.w_full()
                .h(px(if inset { 48.0 } else { thumb_d + 4.0 }))
        };
        let track = if vertical {
            div().h_full().w(px(track_h)).child(vbar(&t, frac, track_h))
        } else {
            if inset {
                let r = &t.palette.roles;
                div()
                    .w_full()
                    .h(px(track_h))
                    .relative()
                    .rounded(px(12.0 * crate::shape_tune().roundness))
                    .bg(hsla(r.secondary_container))
                    .child(
                        div()
                            .h_full()
                            .w(relative(frac))
                            .rounded(px(12.0 * crate::shape_tune().roundness))
                            .bg(hsla(t.palette.primary)),
                    )
                    .children(self.icon.map(|i| {
                        div().absolute().left(px(12.0)).top(px(8.0)).child(
                            crate::icons::Icon::new(i)
                                .size(t.sizes().icon)
                                .color(hsla(t.palette.on_primary)),
                        )
                    }))
            } else if self.centered {
                div().w_full().child(centered_bar(&t, frac, track_h))
            } else {
                div().w_full().child(bar(&t, frac, None, track_h))
            }
        };
        let thumb = shaped(&thumb_shape, thumb_style)
            .absolute()
            .w(px(thumb_d))
            .h(px(if inset { 44.0 } else { thumb_d }));
        let rail = div().absolute().top_0().bottom_0().left_0().right_0();
        let tick_c = hsla(t.palette.border_strong);
        let ticks = if self.ticks {
            tick_fractions(min, max, step)
        } else {
            Vec::new()
        };
        let rail = if vertical {
            let mut r = rail.top(px(thumb_d / 2.0)).bottom(px(thumb_d / 2.0));
            for f in &ticks {
                r = r.child(
                    div()
                        .absolute()
                        .left(px(5.0))
                        .w(px(10.0))
                        .h(px(2.0))
                        .top(relative(1.0 - f))
                        .bg(tick_c),
                );
            }
            r.child(
                thumb
                    .left(px(2.0))
                    .top(relative(1.0 - frac))
                    .mt(px(-thumb_d / 2.0)),
            )
        } else {
            let mut r = rail.left(px(thumb_d / 2.0)).right(px(thumb_d / 2.0));
            for f in &ticks {
                r = r.child(
                    div()
                        .absolute()
                        .top(px(5.0))
                        .h(px(10.0))
                        .w(px(2.0))
                        .left(relative(*f))
                        .ml(px(-1.0))
                        .bg(tick_c),
                );
            }
            if let Some(bp) = self.bubble {
                let txt = match &self.format {
                    Some(f) => f(self.value),
                    None => dodai_charts_core::fmt_num((self.value * 100.0).round() / 100.0),
                };
                let y = match bp {
                    BubblePlacement::Above => -26.0,
                    BubblePlacement::Below => thumb_d + 8.0,
                };
                let pill = ShapeStyle::from_tokens(
                    &t,
                    Some(t.palette.surface_raised),
                    Some(t.palette.border),
                    false,
                    false,
                );
                r = r.child(
                    div()
                        .absolute()
                        .top(px(y))
                        .left(relative(frac))
                        .w_0()
                        .flex()
                        .justify_center()
                        .child(
                            shaped(&scaled_shape(&t.shape, 20.0), pill)
                                .px(px(t.spacing.sm))
                                .h(px(20.0))
                                .flex()
                                .items_center()
                                .whitespace_nowrap()
                                .text_size(px(t.typography.size_xs))
                                .text_color(hsla(t.palette.text))
                                .child(txt),
                        ),
                );
            }
            r.child(
                thumb
                    .top(px(2.0))
                    .left(relative(frac))
                    .ml(px(-thumb_d / 2.0)),
            )
        };
        let readout = self.show_value.then(|| {
            let txt = match &self.format {
                Some(f) => f(self.value),
                None => dodai_charts_core::fmt_num((self.value * 100.0).round() / 100.0),
            };
            div()
                .font_family(crate::text::first_family(t.typography.mono))
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text_muted))
                .child(txt)
        });
        let slider = base
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, _, cx| rec.update(cx, |s, _| s.bounds = b),
                )
                .absolute()
                .size_full(),
            )
            .child(track)
            .child(rail)
            .on_mouse_down(MouseButton::Left, move |e, w, cx| {
                down.update(cx, |s, _| s.dragging = true);
                apply_down(e.position, w, cx);
            })
            .on_mouse_move(move |e, w, cx| {
                if mv.read(cx).dragging {
                    apply_move(e.position, w, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |s, _| s.dragging = false)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up2.update(cx, |s, _| s.dragging = false)
            });
        let slider = match (self.bubble, vertical) {
            (Some(BubblePlacement::Above), false) => {
                div().pt(px(28.0)).child(slider).into_any_element()
            }
            (Some(BubblePlacement::Below), false) => {
                div().pb(px(28.0)).child(slider).into_any_element()
            }
            _ => slider.into_any_element(),
        };
        match (readout, vertical) {
            (None, _) => slider,
            (Some(r), true) => div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(t.spacing.xs))
                .child(r)
                .child(slider)
                .into_any_element(),
            (Some(r), false) => div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(div().flex_1().child(slider))
                .child(r)
                .into_any_element(),
        }
    }
}

crate::entry_render!(Slider);

#[cfg(test)]
mod tests {
    #[test]
    fn checkbox_variants_have_sane_geometry() {
        for v in CheckboxVariant::ALL {
            let (target, bx) = v.dims();
            assert!(target >= bx, "{v:?}: target smaller than box");
            assert!(!v.label().is_empty());
        }
        assert_eq!(CheckboxVariant::default().dims(), (40.0, 18.0));
        assert_eq!(CheckboxVariant::Large.dims(), (48.0, 24.0));
    }

    use super::*;
    use crate::testing::{light_rounded, square};
    use dodai::themes;

    #[test]
    fn clamp_and_fraction() {
        assert_eq!(clamp01(2.0), 1.0);
        assert_eq!(clamp01(f32::NAN), 0.0);
        assert_eq!(slider_fraction(5.0, 0.0, 10.0), 0.5);
        assert_eq!(slider_fraction(5.0, 3.0, 3.0), 0.0);
    }

    #[test]
    fn ticks_follow_step() {
        assert_eq!(tick_fractions(0.0, 10.0, Some(5.0)), vec![0.0, 0.5, 1.0]);
        assert!(tick_fractions(0.0, 10.0, None).is_empty());
        assert!(tick_fractions(0.0, 1000.0, Some(1.0)).is_empty());
    }

    #[test]
    fn slider_maps_pointer() {
        // bounds 100..300 (w=200), thumb 20: centre travels 110..290.
        assert_eq!(slider_value(100.0, 100.0, 200.0, 20.0, 0.0, 1.0, None), 0.0);
        assert!((slider_value(200.0, 100.0, 200.0, 20.0, 0.0, 1.0, None) - 0.5).abs() < 1e-6);
        assert_eq!(slider_value(999.0, 100.0, 200.0, 20.0, 0.0, 1.0, None), 1.0);
        let v = slider_value(180.0, 100.0, 200.0, 20.0, 0.0, 10.0, Some(2.0));
        assert_eq!(v, 4.0);
    }

    #[test]
    fn toggle_style_tokens() {
        for t in [light_rounded(), square(), themes::cyberpunk::yellow()] {
            let on = toggle_style(&t, true, ControlState::Idle);
            let off = toggle_style(&t, false, ControlState::Idle);
            assert_ne!(on.fill, off.fill);
            assert_eq!(on.fill, Some(hsla(t.palette.primary)));
            assert_eq!(on.glow.is_some(), t.effects.glow_enabled());
            assert!(off.glow.is_none());
        }
    }
}
