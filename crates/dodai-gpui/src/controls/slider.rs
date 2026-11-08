//! [`Slider`].

use super::bars::{bar, centered_bar, vbar};
use super::*;
use crate::paint::{round_shape, scaled_shape, shaped, ShapeStyle};
use crate::{hsla, theme_of};
use dodai_core::{ControlState, Theme};
use gpui::{
    canvas, div, px, relative, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Styled, Window,
};

#[derive(Default)]
pub(super) struct SliderState {
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
    disabled: bool,
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
            disabled: false,
        }
    }
    /// Disable the slider: dimmed, no drag and no keys.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
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
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.value);
        self.value = own.get();
        let (state, inner) = (own.setter(), self.on_change.take());
        let live = state.clone();
        self.on_change = Some(Rc::new(move |v, w, cx| {
            crate::own::choose(&state, cx, v);
            if let Some(f) = &inner {
                f(v, w, cx);
            }
        }));
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
            .id(self.id.clone())
            .relative()
            .flex()
            .items_center()
            .cursor_pointer();
        let bubble_h = t.scale(dodai_core::Scale::Half);
        let base = if vertical {
            base.w(px(thumb_d + 4.0))
                .h(px(self.length))
                .justify_center()
        } else {
            base.w_full().h(px(if inset {
                t.sizes().touch_target
            } else {
                thumb_d + 4.0
            }))
        };
        let track = self.track_el(&t, frac, track_h, inset, vertical);
        let thumb = shaped(&thumb_shape, thumb_style)
            .absolute()
            .w(px(thumb_d))
            .h(px(if inset {
                t.sizes().touch_target - 4.0
            } else {
                thumb_d
            }));
        let rail = self.rail_el(&t, frac, thumb, thumb_d, vertical, bubble_h);
        let readout = self.show_value.then(|| {
            let txt = self.value_text();
            div()
                .font_family(crate::text::first_family(t.typography.mono))
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.text_muted))
                .child(txt)
        });
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let disabled = self.disabled;
        let keys = {
            let f = self.on_change.clone();
            move |e: &gpui::KeyDownEvent, w: &mut Window, cx: &mut App| {
                // The live value: several keys can land before the next render.
                let v = live.read(cx).1;
                if let (Some(to), Some(f), false) = (
                    key_value(e.keystroke.key.as_str(), v, min, max, step),
                    &f,
                    disabled,
                ) {
                    f(to, w, cx);
                }
            }
        };
        let base = if disabled {
            base.opacity(crate::interact::DISABLED_OPACITY)
        } else {
            gpui::InteractiveElement::on_key_down(
                gpui::InteractiveElement::track_focus(base, &focus),
                keys,
            )
        };
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
                if disabled {
                    return;
                }
                w.focus(&focus);
                down.update(cx, |s, _| s.dragging = true);
                apply_down(e.position, w, cx);
            })
            .on_mouse_move(move |e, w, cx| {
                if !disabled && mv.read(cx).dragging {
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
            (Some(BubblePlacement::Above), false) => div()
                .pt(px(bubble_h + t.spacing.sm))
                .child(slider)
                .into_any_element(),
            (Some(BubblePlacement::Below), false) => div()
                .pb(px(bubble_h + t.spacing.sm))
                .child(slider)
                .into_any_element(),
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

impl Slider {
    /// The readout: the custom format, or the value to two decimals.
    fn value_text(&self) -> String {
        match &self.format {
            Some(f) => f(self.value),
            None => dodai_core::math::fmt_num((self.value * 100.0).round() / 100.0),
        }
    }

    /// The filled track: vertical bar, M3 inset track with an icon, centred or plain bar.
    fn track_el(
        &self,
        t: &Theme,
        frac: f32,
        track_h: f32,
        inset: bool,
        vertical: bool,
    ) -> gpui::Div {
        if vertical {
            div().h_full().w(px(track_h)).child(vbar(t, frac, track_h))
        } else {
            if inset {
                let r = &t.palette.roles;
                div()
                    .w_full()
                    .h(px(track_h))
                    .relative()
                    .rounded(px(t.shape.radius))
                    .bg(hsla(r.secondary_container))
                    .child(
                        div()
                            .h_full()
                            .w(relative(frac))
                            .rounded(px(t.shape.radius))
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
                div().w_full().child(centered_bar(t, frac, track_h))
            } else {
                div().w_full().child(bar(t, frac, None, track_h))
            }
        }
    }

    /// Thumb, tick marks and the value bubble, positioned over the track.
    fn rail_el(
        &self,
        t: &Theme,
        frac: f32,
        thumb: gpui::Div,
        thumb_d: f32,
        vertical: bool,
        bubble_h: f32,
    ) -> gpui::Div {
        let rail = div().absolute().top_0().bottom_0().left_0().right_0();
        let tick_c = hsla(t.palette.border_strong);
        let ticks = if self.ticks {
            tick_fractions(self.min, self.max, self.step)
        } else {
            Vec::new()
        };
        if vertical {
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
                let txt = self.value_text();
                let y = match bp {
                    BubblePlacement::Above => -26.0,
                    BubblePlacement::Below => thumb_d + 8.0,
                };
                let pill = ShapeStyle::from_tokens(
                    t,
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
                            shaped(&scaled_shape(&t.shape, bubble_h), pill)
                                .px(px(t.spacing.sm))
                                .h(px(bubble_h))
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
        }
    }
}

crate::entry_render!(Slider);
