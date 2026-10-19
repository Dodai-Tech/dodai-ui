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
    Above,
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
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let t = theme_of(cx);
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
        _ => (20.0, 20.0),
    };
    let shape = match kind {
        Kind::Switch | Kind::Radio => round_shape(&t.shape, h),
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
    el
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
            on_change: Option<BoolHandler>,
        }

        impl $name {
            /// `on` is the current (controlled) value.
            pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
                Self { id: id.into(), on, disabled: false, mixed: false, on_change: None }
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
            /// Called with the requested new value on click.
            pub fn on_change(mut self, f: impl Fn(bool, &mut Window, &mut App) + 'static) -> Self {
                self.on_change = Some(Rc::new(f));
                self
            }
        }

        impl RenderOnce for $name {
            fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
                choice($kind, self.id, self.on, self.disabled, self.on_change, $radio, self.mixed, window, cx)
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
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self { value, tone: None }
    }
    /// Colour by status tone (default: brand colour).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
}

impl ProgressBar {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "ProgressBar");
        bar(&t, self.value * g, self.tone, t.spacing.sm + 2.0)
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
pub enum Direction {
    #[default]
    Horizontal,
    Vertical,
}

/// Draggable slider over `min..=max` (default `0..=1`). Controlled.
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    direction: Direction,
    length: f32,
    value: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    ticks: bool,
    show_value: bool,
    bubble: Option<BubblePlacement>,
    format: Option<Rc<dyn Fn(f32) -> String>>,
    on_change: Option<FloatHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Slider);

const THUMB: f32 = 16.0;

impl Slider {
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            direction: Direction::Horizontal,
            length: 160.0,
            value,
            min: 0.0,
            max: 1.0,
            step: None,
            ticks: false,
            show_value: false,
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
    pub fn direction(mut self, d: Direction) -> Self {
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
        let vertical = self.direction == Direction::Vertical;
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
                    f(slider_value(pos, 0.0, len, THUMB, min, max, step), w, cx);
                }
            }
        };
        let rec = st.clone();
        let (down, mv, up, up2) = (st.clone(), st.clone(), st.clone(), st.clone());
        let (apply_down, apply_move) = (apply.clone(), apply);
        let k = t.toggle(true, ControlState::Idle);
        let thumb_style = ShapeStyle::from_tokens(&t, Some(k.track), Some(k.border), true, false);
        let thumb_shape = round_shape(&t.shape, THUMB);
        let base = div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .cursor_pointer();
        let base = if vertical {
            base.w(px(THUMB + 4.0)).h(px(self.length)).justify_center()
        } else {
            base.w_full().h(px(THUMB + 4.0))
        };
        let track = if vertical {
            div().h_full().w(px(6.0)).child(vbar(&t, frac, 6.0))
        } else {
            div().w_full().child(bar(&t, frac, None, 6.0))
        };
        let thumb = shaped(&thumb_shape, thumb_style)
            .absolute()
            .w(px(THUMB))
            .h(px(THUMB));
        let rail = div().absolute().top_0().bottom_0().left_0().right_0();
        let tick_c = hsla(t.palette.border_strong);
        let ticks = if self.ticks {
            tick_fractions(min, max, step)
        } else {
            Vec::new()
        };
        let rail = if vertical {
            let mut r = rail.top(px(THUMB / 2.0)).bottom(px(THUMB / 2.0));
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
                    .mt(px(-THUMB / 2.0)),
            )
        } else {
            let mut r = rail.left(px(THUMB / 2.0)).right(px(THUMB / 2.0));
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
                    BubblePlacement::Below => THUMB + 8.0,
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
            r.child(thumb.top(px(2.0)).left(relative(frac)).ml(px(-THUMB / 2.0)))
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
        for t in [light_rounded(), square(), themes::cyberpunk::night_city()] {
            let on = toggle_style(&t, true, ControlState::Idle);
            let off = toggle_style(&t, false, ControlState::Idle);
            assert_ne!(on.fill, off.fill);
            assert_eq!(on.fill, Some(hsla(t.palette.primary)));
            assert_eq!(on.glow.is_some(), t.effects.glow_enabled());
            assert!(off.glow.is_none());
        }
    }
}
