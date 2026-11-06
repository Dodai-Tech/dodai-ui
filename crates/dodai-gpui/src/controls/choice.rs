//! [`Toggle`], [`Checkbox`] and [`Radio`].

use super::*;
use crate::interact::{track, use_interact};
use crate::motion::{play_on_change, Clip, Glitch, Motion, MotionExt, Pose, SelectionMotion};
use crate::paint::{round_shape, scaled_shape, shaped, shaped_id, ShapeStyle};
use crate::{hsla, theme_of};
use dodai_core::motion::{nudge, pop_in};
use dodai_core::ControlState;
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window,
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
    pub animation: Option<SelectionMotion>,
    pub motion: Option<Motion>,
}

/// How the switch thumb, tick or dot moves, resolved against the theme.
#[derive(Clone, Copy)]
struct Moves {
    kind: SelectionMotion,
    motion: Motion,
}

impl Moves {
    fn of(c: &Choice, t: &dodai_core::Theme, cx: &App) -> Self {
        Self {
            kind: c.animation.unwrap_or(t.look().selection),
            motion: c.motion.unwrap_or_else(|| Motion::of(cx)),
        }
    }

    /// The `0..=1` on-ness that colours follow.
    fn progress(self, window: &mut Window, cx: &mut App, id: &ElementId, on: bool) -> f32 {
        let to = if on { 1.0 } else { 0.0 };
        match self.kind {
            // A hard switch: colours change with the mark, at once.
            SelectionMotion::Lift | SelectionMotion::Glitch => to,
            _ => crate::motion::smooth_with(window, cx, id.clone(), to, self.motion),
        }
    }

    /// `(left, width)` of a switch thumb `d` wide that rests at `x0` off and `x1` on.
    fn thumb(
        self,
        window: &mut Window,
        cx: &mut App,
        id: &ElementId,
        on: bool,
        (x0, x1, d): (f32, f32, f32),
    ) -> (f32, f32) {
        let x = if on { x1 } else { x0 };
        let key = ElementId::from((id.clone(), "thumb"));
        let m = self.motion;
        match self.kind {
            SelectionMotion::Slide => {
                let k = crate::motion::smooth_with(window, cx, key, if on { 1.0 } else { 0.0 }, m);
                (crate::motion::lerp(x0, x1, k), d)
            }
            SelectionMotion::Stretch => crate::motion::stretch(window, cx, key, (x, d), m),
            SelectionMotion::Glitch => {
                let g = play_on_change(window, cx, key, on, || Glitch::clip(&m.speed(0.5)));
                (x + g.map_or(0.0, |g| g.slice_dx(0, d * 0.6)), d)
            }
            SelectionMotion::Lift => {
                // Jumps across, then a little push that springs back.
                let push = play_on_change(window, cx, key, on, || nudge(m.personality));
                let dx = push.map_or(0.0, |p| p.dx);
                (x + if on { dx } else { -dx }, d)
            }
        }
    }

    /// How the tick or dot shows: faded and grown with `p`, popped in on a spring (Stretch),
    /// glitched in (Glitch) or there at once (Lift).
    fn mark(self, window: &mut Window, cx: &mut App, id: &ElementId, on: bool, p: f32) -> Pose {
        let key = ElementId::from((id.clone(), "mark"));
        let m = self.motion;
        let shown = Pose {
            scale: p,
            ..Pose::opacity(p)
        };
        match self.kind {
            SelectionMotion::Slide => shown,
            SelectionMotion::Stretch => {
                let pop = play_on_change(window, cx, key, on, || {
                    if on {
                        pop_in(m.personality)
                    } else {
                        Clip::constant(Pose::IDENTITY)
                    }
                });
                pop.filter(|_| on).unwrap_or(shown)
            }
            SelectionMotion::Glitch => {
                let g = play_on_change(window, cx, key, on, || Glitch::clip(&m.speed(0.5)));
                match (on, g) {
                    (true, Some(g)) => Pose {
                        dx: g.slice_dx(0, 6.0),
                        ..Pose::opacity(if g.slice_hidden(0) { 0.0 } else { 1.0 })
                    },
                    _ => Pose::opacity(if on { 1.0 } else { 0.0 }),
                }
            }
            SelectionMotion::Lift => Pose::opacity(if on { 1.0 } else { 0.0 }),
        }
    }
}

/// Outline of the Material switch track, checkbox box and radio ring, px.
const OUTLINE: f32 = 2.0;
/// Size of the check / dash glyph in a classic checkbox, as a share of its box.
const CHECK_MARK: f32 = 0.72;
/// The same in a Material checkbox.
const M3_CHECK_MARK: f32 = 0.78;

pub(super) fn choice(mut c: Choice, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
    // Works controlled or not (see `own`).
    let own = crate::own::Own::new(window, cx, c.id.clone(), c.on);
    c.on = own.get();
    let (state, inner) = (own.setter(), c.handler.take());
    c.handler = Some(Rc::new(move |v, w, cx| {
        crate::own::choose(&state, cx, v);
        if let Some(f) = &inner {
            f(v, w, cx);
        }
    }));
    let t = theme_of(cx);
    let moves = Moves::of(&c, &t, cx);
    if t.look().tonal {
        return m3_choice(c, moves, window, cx);
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
    let p = moves.progress(window, cx, &id, on);
    let mut style = toggle_style(&t, on, state);
    {
        let (a, b) = (
            toggle_style(&t, false, state),
            toggle_style(&t, true, state),
        );
        style.fill = crate::motion::mix_opt(a.fill, b.fill, p);
        style.stroke = crate::motion::mix_opt(a.stroke, b.stroke, p);
    }
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
    let id_mark = id.clone();
    let mut el = track(shaped_id(id, &shape, style), &st, cx)
        .w(px(w))
        .h(px(h))
        .flex_none()
        .flex()
        .items_center()
        .justify_center();
    let mark = ShapeStyle::from_tokens(&t, Some(k.thumb), None, false, false);
    let shown = (kind != Kind::Switch).then(|| moves.mark(window, cx, &id_mark, on, p));
    el = match kind {
        Kind::Switch => {
            let thumb_d = (h * 0.64).round();
            let inset = (h - thumb_d) / 2.0 - 1.0;
            let ends = (inset, w - thumb_d - inset - 1.0, thumb_d);
            let (left, width) = moves.thumb(window, cx, &id_mark, on, ends);
            let tshape = round_shape(&t.shape, thumb_d);
            el.child(
                shaped(&tshape, mark)
                    .absolute()
                    .top(px(inset))
                    .left(px(left))
                    .w(px(width))
                    .h(px(thumb_d)),
            )
        }
        Kind::Checkbox => match shown {
            Some(m) if m.opacity > 0.01 => el.child(
                div()
                    .relative()
                    .left(px(m.dx))
                    .text_color(hsla(k.thumb).opacity(m.opacity.min(1.0)))
                    .text_size(px(h * CHECK_MARK * m.scale.max(0.0)))
                    .child(if mixed { "−" } else { "✓" }),
            ),
            _ => el,
        },
        Kind::Radio => match shown {
            Some(m) if m.opacity > 0.01 => {
                let dot = (h * 0.4).round();
                let tshape = round_shape(&t.shape, dot);
                let d = dot * m.scale.max(0.0);
                el.child(
                    shaped(&tshape, mark)
                        .relative()
                        .left(px(m.dx))
                        .opacity(m.opacity.min(1.0))
                        .w(px(d))
                        .h(px(d)),
                )
            }
            _ => el,
        },
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
    moves: Moves,
    /// How the tick or dot shows.
    mark: Pose,
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
        use crate::motion::lerp;
        let Self {
            t,
            p,
            moves,
            on,
            icons,
            track_c,
            border_c,
            thumb_c,
            ..
        } = *self;
        let sz = t.sizes();
        let r = &t.palette.roles;
        let ((tw, th), (off_d, on_d)) = (sz.switch_track, sz.switch_handle);
        let size = if icons {
            on_d
        } else {
            lerp(off_d, on_d, p.clamp(0.0, 1.0))
        };
        let ends = (th / 2.0 - size / 2.0, tw - th / 2.0 - size / 2.0, size);
        let (left, width) = moves.thumb(window, cx, &id, on, ends);
        let centre = left + width / 2.0;
        // Absolute children sit inside the outline.
        let (ls, lst) = self.layer_style(sz.state_layer);
        let mut track_style = ShapeStyle::from_tokens(t, None, None, false, false);
        track_style.fill = Some(track_c);
        track_style.stroke = Some(border_c);
        track_style.stroke_width = OUTLINE;
        let mut handle = ShapeStyle::from_tokens(t, None, None, false, false);
        handle.fill = Some(thumb_c);
        track(
            shaped_id(id, &round_shape(&t.shape, th), track_style),
            st,
            cx,
        )
        .w(px(tw))
        .h(px(th))
        .flex_none()
        .child(
            shaped(&ls, lst)
                .absolute()
                .left(px(centre - sz.state_layer / 2.0 - OUTLINE))
                .top(px(-(sz.state_layer - th) / 2.0 - OUTLINE))
                .w(px(sz.state_layer))
                .h(px(sz.state_layer)),
        )
        .child({
            let mut h = shaped(&round_shape(&t.shape, size), handle)
                .absolute()
                .left(px(left - OUTLINE))
                .top(px((th - size) / 2.0 - OUTLINE))
                .w(px(width))
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
                    .child(crate::surfaces::glyph_el(
                        if on { "✓" } else { "✕" }.into(),
                        t.typography.size_md,
                        hsla(glyph_c),
                    ));
            }
            h
        })
    }

    fn checkbox_or_radio(
        &self,
        kind: Kind,
        id: ElementId,
        st: &gpui::Entity<crate::interact::Interact>,
        cx: &App,
    ) -> gpui::Stateful<gpui::Div> {
        let Self {
            t,
            p,
            mark,
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
        let outer = track(shaped_id(id, &ls, lst), st, cx)
            .w(px(target))
            .h(px(target))
            .flex_none()
            .flex()
            .items_center()
            .justify_center();
        let mut ring = ShapeStyle::from_tokens(t, None, None, false, false);
        ring.fill = Some(track_c);
        ring.stroke = Some(border_c);
        ring.stroke_width = OUTLINE;
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
                .text_color(thumb_c.opacity(mark.opacity.min(1.0)))
                .text_size(px(bx * M3_CHECK_MARK * mark.scale.max(0.0)));
            if mark.opacity > 0.01 {
                b = b.child(div().relative().left(px(mark.dx)).child(if mixed {
                    "−"
                } else {
                    "✓"
                }));
            }
            outer.child(b)
        } else {
            let d = bx + OUTLINE;
            let mut b = shaped(&round_shape(&t.shape, d), ring)
                .w(px(d))
                .h(px(d))
                .flex()
                .items_center()
                .justify_center();
            if mark.opacity > 0.01 {
                let mut dot = ShapeStyle::from_tokens(t, None, None, false, false);
                dot.fill = Some(thumb_c);
                let size = d / 2.0 * mark.scale.max(0.0);
                b = b.child(
                    shaped(&round_shape(&t.shape, d / 2.0), dot)
                        .relative()
                        .left(px(mark.dx))
                        .opacity(mark.opacity.min(1.0))
                        .w(px(size))
                        .h(px(size)),
                );
            }
            outer.child(b)
        }
    }
}

/// Material 3 switch (52×32), checkbox (18 px box) and radio (20 px ring), each with the 40 px
/// state layer; colours from [`Theme::choice_toggle`].
fn m3_choice(c: Choice, moves: Moves, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
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
        ..
    } = c;
    use crate::motion::mix;
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
    let p = moves.progress(window, cx, &id, on);
    let mark = moves.mark(window, cx, &id, on, p);
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
        moves,
        mark,
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
        Kind::Checkbox | Kind::Radio => look.checkbox_or_radio(kind, id, &st, cx),
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
            animation: Option<SelectionMotion>,
            motion: Option<Motion>,
        }

        impl $name {
            /// `on` is the current (controlled) value.
            pub fn new(id: impl Into<ElementId>, on: bool) -> Self {
                Self { id: id.into(), on, disabled: false, mixed: false, variant: CheckboxVariant::Standard, icons: false, on_change: None, animation: None, motion: None }
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
            /// How the switch thumb travels and the tick or dot appears; `None` (the default) is the
            /// design's own ([`Look::selection`](dodai_core::Look::selection)).
            pub fn animation(mut self, a: impl Into<Option<SelectionMotion>>) -> Self {
                self.animation = a.into();
                self
            }
            /// Pace, easing and travel (eased or spring); defaults to the design's motion.
            pub fn motion(mut self, m: Motion) -> Self {
                self.motion = Some(m);
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
                choice(Choice { kind: $kind, id: self.id, on: self.on, disabled: self.disabled, handler: self.on_change, radio: $radio, mixed: self.mixed, variant: self.variant, icons: self.icons, animation: self.animation, motion: self.motion }, window, cx)
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
