//! [`Toggle`], [`Checkbox`] and [`Radio`].

use super::*;
use crate::interact::{track, use_interact};
use crate::paint::{round_shape, scaled_shape, shaped, shaped_id, ShapeStyle};
use crate::{hsla, theme_of};
use dodai_core::ControlState;
use gpui::{
    px, App, ElementId, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement, Styled,
    Window,
};

pub(super) type BoolHandler = Rc<dyn Fn(bool, &mut Window, &mut App)>;

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
pub(super) enum Kind {
    Switch,
    Checkbox,
    Radio,
}

/// What a switch, checkbox or radio is asked to draw.
pub(super) struct Choice {
    pub kind: Kind,
    pub id: ElementId,
    pub on: bool,
    pub disabled: bool,
    pub handler: Option<BoolHandler>,
    /// A radio: clicking always selects.
    pub radio: bool,
    /// A half-checked box.
    pub mixed: bool,
    pub variant: CheckboxVariant,
    /// Icons in the switch handle.
    pub icons: bool,
}

pub(super) fn choice(mut c: Choice, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    // Works controlled or not: the choice remembers its last click, and follows `on` again as
    // soon as the caller passes a different value. Without this a control given a fixed value
    // (or no `on_change`) would ignore every click.
    let own = window.use_keyed_state((c.id.clone(), "own"), cx, |_, _| (c.on, c.on));
    let (given, shown) = *own.read(cx);
    if c.on != given {
        own.update(cx, |s, _| *s = (c.on, c.on));
    } else {
        c.on = shown;
    }
    let inner = c.handler.take();
    c.handler = Some(Rc::new(move |v, w, cx| {
        own.update(cx, |s, _| s.1 = v);
        if let Some(f) = &inner {
            f(v, w, cx);
        }
    }));
    let t = theme_of(cx);
    if t.look().tonal {
        return m3_choice(c, window, cx);
    }
    let Choice {
        kind,
        id,
        on,
        disabled,
        handler,
        radio,
        mixed,
        ..
    } = c;
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
        Kind::Switch => {
            let h = t.sizes().selection_box + 2.0;
            (h * 1.9, h)
        }
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
            let thumb_d = (h * 0.64).round();
            let inset = (h - thumb_d) / 2.0 - 1.0;
            let tshape = round_shape(&t.shape, thumb_d);
            el.child(
                shaped(&tshape, mark)
                    .absolute()
                    .top(px(inset))
                    .left(px(crate::motion::lerp(
                        inset,
                        w - thumb_d - inset - 1.0,
                        thumb,
                    )))
                    .w(px(thumb_d))
                    .h(px(thumb_d)),
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
                let dot = (h * 0.4).round();
                let tshape = round_shape(&t.shape, dot);
                el.child(shaped(&tshape, mark).w(px(dot * p)).h(px(dot * p)))
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

/// The colours and sizes shared by the Material switch / checkbox / radio parts.
struct M3Look<'a> {
    t: &'a dodai_core::Theme,
    p: f32,
    on: bool,
    mixed: bool,
    icons: bool,
    variant: CheckboxVariant,
    track_c: gpui::Hsla,
    border_c: gpui::Hsla,
    thumb_c: gpui::Hsla,
    layer_c: gpui::Hsla,
    /// Touch target (the state layer) and box size.
    target: f32,
    bx: f32,
}

impl M3Look<'_> {
    /// The state layer: a disc of diameter `d` in the layer colour.
    fn layer_style(&self, d: f32) -> (dodai_core::Shape, ShapeStyle) {
        let mut s = ShapeStyle::from_tokens(self.t, None, None, false, false);
        s.fill = Some(self.layer_c);
        (round_shape(&self.t.shape, d), s)
    }

    fn switch(
        &self,
        id: ElementId,
        st: &gpui::Entity<crate::interact::Interact>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::Stateful<gpui::Div> {
        use crate::motion::{lerp, smooth_with, Easing, Motion};
        let Self {
            t,
            p,
            on,
            icons,
            track_c,
            border_c,
            thumb_c,
            ..
        } = *self;
        let sz = t.sizes();
        let r = &t.palette.roles;
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
        let (ls, lst) = self.layer_style(sz.state_layer);
        let mut track_style = ShapeStyle::from_tokens(t, None, None, false, false);
        track_style.fill = Some(track_c);
        track_style.stroke = Some(border_c);
        track_style.stroke_width = 2.0;
        let mut handle = ShapeStyle::from_tokens(t, None, None, false, false);
        handle.fill = Some(thumb_c);
        track(shaped_id(id, &round_shape(&t.shape, th), track_style), st)
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
                        .text_size(px(t.typography.size_md))
                        .text_color(hsla(glyph_c))
                        .child(if on { "✓" } else { "✕" });
                }
                h
            })
    }

    fn checkbox_or_radio(
        &self,
        kind: Kind,
        id: ElementId,
        st: &gpui::Entity<crate::interact::Interact>,
    ) -> gpui::Stateful<gpui::Div> {
        let Self {
            t,
            p,
            mixed,
            variant,
            track_c,
            border_c,
            thumb_c,
            target,
            bx,
            ..
        } = *self;
        let (ls, lst) = self.layer_style(target);
        let outer = track(shaped_id(id, &ls, lst), st)
            .w(px(target))
            .h(px(target))
            .flex_none()
            .flex()
            .items_center()
            .justify_center();
        let mut ring = ShapeStyle::from_tokens(t, None, None, false, false);
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
                let mut dot = ShapeStyle::from_tokens(t, None, None, false, false);
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
}

/// Material 3 switch (52×32), checkbox (18 px box) and radio (20 px ring), each with the 40 px
/// state layer; colours from [`Theme::choice_toggle`].
pub(super) fn m3_choice(c: Choice, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    let Choice {
        kind,
        id,
        on,
        disabled,
        handler,
        radio,
        mixed,
        variant,
        icons,
    } = c;
    use crate::motion::{mix, transition};
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
        ControlState::Hovered => dodai_core::StateLayer::HOVER,
        ControlState::Pressed => dodai_core::StateLayer::PRESSED,
        ControlState::Focused => dodai_core::StateLayer::FOCUS,
        _ => 0.0,
    };
    let layer_c = hsla(if on { t.palette.primary } else { r.on_surface }).opacity(layer_alpha);
    let look = M3Look {
        t: &t,
        p,
        on,
        mixed,
        icons,
        variant,
        track_c,
        border_c,
        thumb_c,
        layer_c,
        target,
        bx,
    };
    let mut el = match kind {
        Kind::Switch => look.switch(id, &st, window, cx),
        Kind::Checkbox | Kind::Radio => look.checkbox_or_radio(kind, id, &st),
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
                choice(Choice { kind: $kind, id: self.id, on: self.on, disabled: self.disabled, handler: self.on_change, radio: $radio, mixed: self.mixed, variant: self.variant, icons: self.icons }, window, cx)
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
