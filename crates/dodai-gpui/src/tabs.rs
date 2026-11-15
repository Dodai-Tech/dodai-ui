//! [`Tabs`]: a row of selectable tabs driven by `theme.tab` tokens.

use crate::interact::{track, use_interact};
use crate::motion::{Motion, MotionExt, SelectionMotion};
use crate::paint::{bottom_band_canvas, shaped_id, ShapeStyle};
use crate::selection::{SelectionMarker, SelectionTrack};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, prelude::FluentBuilder, px, App, ElementId, Hsla, IntoElement, ParentElement,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// Resolved tab paint: shape style, text colour, optional indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.tab(dodai_core::ItemState::of(active, hovered));
    (
        ShapeStyle::from_tokens(
            theme,
            k.background,
            k.outline.then_some(theme.palette.border_strong),
            active,
            k.outline,
        ),
        hsla(k.foreground),
        k.indicator.map(hsla),
    )
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Tab strip. Controlled: pass `selected` and update it in `on_select`.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    labels: Vec<SharedString>,
    active: usize,
    on_select: Option<SelectHandler>,
    corners: Option<dodai_core::Corners>,
    icons: Vec<crate::icons::IconName>,
    secondary: bool,
    scrollable: bool,
    animation: Option<SelectionMotion>,
    motion: Option<Motion>,
    disabled: Vec<usize>,
}

crate::corners_setter!(Tabs);

impl Tabs {
    /// Create a new [`Tabs`]. The active indicator slides from tab to tab.
    #[track_caller]
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            labels: Vec::new(),
            active: 0,
            on_select: None,
            icons: Vec::new(),
            secondary: false,
            scrollable: false,
            animation: None,
            motion: None,
            disabled: Vec::new(),
        }
    }
    /// Tabs (by index) that can't be picked: dimmed, skipped by the arrow keys.
    pub fn disabled(mut self, tabs: impl IntoIterator<Item = usize>) -> Self {
        self.disabled = tabs.into_iter().collect();
        self
    }
    /// How the selection moves between tabs. `None` (the default) is the design's own
    /// ([`Look::selection`](dodai_core::Look::selection)).
    pub fn animation(mut self, a: impl Into<Option<SelectionMotion>>) -> Self {
        self.animation = a.into();
        self
    }
    /// Pace, easing and travel of the tab animations. Defaults to the design's motion.
    pub fn motion(mut self, m: Motion) -> Self {
        self.motion = Some(m);
        self
    }
    /// Scrollable tabs: keep every tab at its natural width and scroll the strip sideways when
    /// it overflows (instead of squeezing the labels).
    pub fn scrollable(mut self, on: bool) -> Self {
        self.scrollable = on;
        self
    }
    /// One icon per tab, stacked above the label (icon-only when there are no labels).
    pub fn icons(mut self, icons: impl IntoIterator<Item = crate::icons::IconName>) -> Self {
        self.icons = icons.into_iter().collect();
        self
    }
    /// Secondary tabs: an on-surface label and a full-width underline, for sub-navigation.
    pub fn secondary(mut self, on: bool) -> Self {
        self.secondary = on;
        self
    }
    /// Tab labels in order.
    pub fn labels<S: Into<SharedString>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }
    /// Index of the active tab.
    pub fn selected(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    crate::callback!(
        /// Called with the clicked tab index.
        on_select,
        usize
    );
}

impl Tabs {
    /// Tab `i`: styled for its active / hover state, with its icon, label and indicator.
    fn tab(
        &self,
        t: &dodai_core::Theme,
        i: usize,
        pan: Option<Rc<Cell<Pan>>>,
        measure: Option<&SelectionTrack>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let label = self.labels.get(i);
        let id: ElementId = (self.id.clone(), i.to_string()).into();
        let st = use_interact(&id, window, cx);
        let hovered = st.read(cx).hovered;
        let active = i == self.active;
        let p = crate::motion::smooth_with(
            window,
            cx,
            id.clone(),
            if active { 1.0 } else { 0.0 },
            self.motion_of(cx),
        );
        let off = self.disabled.contains(&i);
        let (mut style, fg, indicator) = tab_style(t, active, hovered && !off);
        let (on_s, on_fg, _) = tab_style(t, true, hovered);
        let (off_s, off_fg, _) = tab_style(t, false, hovered);
        style.fill =
            if measure.is_some() && t.tab(dodai_core::ItemState::SELECTED).background.is_some() {
                // The active pill is one shared element that slides between tabs.
                if active {
                    None
                } else {
                    off_s.fill
                }
            } else {
                crate::motion::mix_opt(off_s.fill, on_s.fill, p)
            };
        let fg = if p > 0.0 && p < 1.0 {
            crate::motion::mix(off_fg, on_fg, p)
        } else {
            fg
        };
        let indicator = if measure.is_some() {
            None
        } else {
            indicator
                .or_else(|| tab_style(t, true, hovered).2.filter(|_| p > 0.01))
                .map(|c| c.opacity(p))
        };
        let fg = if off {
            hsla(t.palette.text_disabled)
        } else {
            fg
        };
        let fg = if self.secondary
            && active
            && !off
            && t.tab(dodai_core::ItemState::SELECTED).background.is_none()
        {
            hsla(t.palette.roles.on_surface)
        } else {
            fg
        };
        let stacked = label.is_some() && self.icons.get(i).is_some();
        let height = if stacked && !self.secondary {
            t.scale(dodai_core::Scale::Wide)
        } else if t.look().segmented_tabs {
            t.shape.control_height - 6.0
        } else {
            t.shape.control_height
        };
        let shape = if t.look().segmented_tabs {
            crate::paint::scaled_shape(&t.shape, t.shape.control_height)
        } else {
            t.shape.clone()
        };
        let mut tab = track(shaped_id(id, &shape, style), &st, cx)
            .min_h(px(height))
            .px(px(t.spacing.md))
            .flex()
            .items_center()
            .when(!off, |d| d.cursor_pointer())
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg);
        if t.look().tonal {
            tab = crate::text::typeset(tab, t, dodai_core::TypeRole::TitleSmall);
        }
        if stacked && !self.secondary {
            tab = tab.flex_col().justify_center().gap(px(t.spacing.half()));
        } else {
            tab = tab.gap(px(t.spacing.sm));
        }
        if let Some(icon) = self.icons.get(i) {
            tab = tab.child(
                crate::icons::Icon::new(*icon)
                    .size(t.sizes().icon_sm)
                    .color(fg),
            );
        }
        if let Some(label) = label {
            tab = tab.child(format_label(&t.typography, label));
        }
        if let Some(c) = indicator {
            tab = tab.child(bottom_band_canvas(&t.shape, t.shape.border_width, c));
        }
        if let Some(m) = measure {
            // Report where this tab sits so the shared indicator can travel to it.
            tab = tab.relative().child(m.item(i));
        }
        if let (false, Some(f)) = (off, self.on_select.clone()) {
            tab = tab.on_click(move |_, w, cx| {
                // A drag that ends over a tab is a pan, not a click.
                if pan.as_ref().is_some_and(|p| p.get().moved) {
                    return;
                }
                f(i, w, cx)
            });
        }
        if self.scrollable {
            tab = tab.flex_none().whitespace_nowrap();
        }
        if self.animation_of(t) == SelectionMotion::Lift {
            // Nothing slides: the chosen tab rises off the page and the old one drops back.
            tab = tab
                .relative()
                .top(px(-(t.shape.border_width * 2.0).max(2.0) * p));
        }
        tab
    }

    /// The scrolling strip around `row`: keeps the selected tab in view and pans with the wheel
    /// and the pointer.
    fn scroll_strip(
        &self,
        row: gpui::Div,
        state: &gpui::Entity<ScrollState>,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui::AnyElement {
        // Keep the selected tab in view, but only when the selection changes so the user
        // can still scroll the strip freely. gpui only knows the strip's size after its
        // first layout, so the request is repeated for a few frames after a change.
        let (handle, pan) = {
            let s = state.read(cx);
            (s.handle.clone(), s.pan.clone())
        };
        let (seen, tries) = (state.read(cx).seen, state.read(cx).tries);
        if seen != self.active || tries < 3 {
            handle.scroll_to_item(self.active);
            let fresh = seen != self.active;
            state.update(cx, |s, _| {
                s.seen = self.active;
                s.tries = if fresh { 1 } else { s.tries + 1 };
            });
            crate::motion::next_frame(window);
        }
        // A mouse wheel only scrolls vertically; over an overflowing strip, turn it into a
        // sideways scroll (trackpads already send horizontal deltas, which gpui handles).
        let (wheel, down, drag, up) = (
            handle.clone(),
            (handle.clone(), pan.clone()),
            (handle.clone(), pan.clone()),
            pan,
        );
        let el = gpui::StatefulInteractiveElement::track_scroll(
            gpui::InteractiveElement::id(row.w_full(), (self.id.clone(), "scroll")),
            &handle,
        )
        .overflow_x_scroll();
        let el = gpui::InteractiveElement::on_scroll_wheel(el, move |ev, window, cx| {
            let d = ev.delta.pixel_delta(window.line_height());
            let max = wheel.max_offset().width;
            if max > gpui::Pixels::ZERO && d.y.abs() > d.x.abs() {
                let mut o = wheel.offset();
                o.x = (o.x - d.y).clamp(-max, gpui::Pixels::ZERO);
                wheel.set_offset(o);
                window.refresh();
                cx.stop_propagation();
            }
        });
        let el = gpui::InteractiveElement::on_mouse_down(
            el,
            gpui::MouseButton::Left,
            move |ev, _, _| {
                down.1.set(Pan {
                    from: Some((f32::from(ev.position.x), f32::from(down.0.offset().x))),
                    moved: false,
                });
            },
        );
        let el = gpui::InteractiveElement::on_mouse_move(el, move |ev, window, _| {
            let mut p = drag.1.get();
            let Some((x0, o0)) = p.from else {
                return;
            };
            if ev.pressed_button != Some(gpui::MouseButton::Left) {
                drag.1.set(Pan::default());
                return;
            }
            let dx = f32::from(ev.position.x) - x0;
            if dx.abs() > 4.0 {
                p.moved = true;
            }
            if p.moved {
                let max = f32::from(drag.0.max_offset().width);
                let mut o = drag.0.offset();
                o.x = px((o0 + dx).clamp(-max, 0.0));
                drag.0.set_offset(o);
                window.refresh();
            }
            drag.1.set(p);
        });
        gpui::InteractiveElement::on_mouse_up(el, gpui::MouseButton::Left, move |_, _, _| {
            let p = up.get();
            // Keep `moved` through the click that follows the release, clear it after.
            up.set(Pan {
                from: None,
                moved: p.moved,
            });
        })
        .into_any_element()
    }

    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.active);
        self.active = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        let current = state.clone();
        self.on_select = Some(Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, i);
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let strip = t.look().segmented_tabs;
        // Arrow keys, Home and End move the selection once the strip has focus.
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let keys = {
            let (n, f) = (self.labels.len(), self.on_select.clone());
            let disabled = self.disabled.clone();
            move |e: &gpui::KeyDownEvent, w: &mut Window, cx: &mut App| {
                // The live selection: several keys can land before the next render.
                let at = current.read(cx).1;
                let to = crate::roving::next(
                    e.keystroke.key.as_str(),
                    crate::roving::Axis::Horizontal,
                    at,
                    n,
                    |i| !disabled.contains(&i),
                );
                if let (Some(to), Some(f)) = (to, &f) {
                    f(to, w, cx);
                }
            }
        };
        let row = div().relative().flex().gap(px(t.spacing.xs));
        let mut row = gpui::InteractiveElement::on_key_down(
            gpui::InteractiveElement::track_focus(row, &focus),
            keys,
        );
        row = gpui::InteractiveElement::on_mouse_down(row, gpui::MouseButton::Left, {
            let focus = focus.clone();
            move |_, w, _| w.focus(&focus)
        });
        // One indicator slides between tabs when the design underlines the active tab.
        let tokens = t.tab(dodai_core::ItemState::SELECTED);
        let pill = tokens.background.is_some() && !tokens.outline;
        let kind = self.animation_of(&t);
        let travels = kind != SelectionMotion::Lift;
        let track = (travels && (pill || (!tokens.outline && tokens.indicator.is_some())))
            .then(|| SelectionTrack::new(self.id.clone(), window, cx));
        // Scrollable strips pan with the pointer too: a swipe or drag moves them sideways.
        let scroll = self.scrollable.then(|| {
            window.use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollState {
                handle: gpui::ScrollHandle::new(),
                seen: usize::MAX,
                tries: 0,
                pan: Rc::new(Cell::new(Pan::default())),
            })
        });
        let motion = self.motion_of(cx);
        if let Some(track) = &track {
            row = row.child(track.container());
        }
        if let (true, Some(track)) = (pill, &track) {
            // The filled tab travels to the active one, the design's way.
            let (style, _, _) = tab_style(&t, true, false);
            let shape = if strip {
                crate::paint::scaled_shape(&t.shape, t.shape.control_height)
            } else {
                t.shape.clone()
            };
            let marker = SelectionMarker::fill(shape, style);
            row = row.child(track.indicator(self.active, kind, motion, marker, window, cx));
        }
        let count = self.labels.len().max(self.icons.len());
        for i in 0..count {
            let pan = scroll.as_ref().map(|s| s.read(cx).pan.clone());
            row = row.child(self.tab(&t, i, pan, track.as_ref(), window, cx));
        }
        if let (false, Some(track)) = (pill, &track) {
            let colour = tab_style(&t, true, false).2.unwrap_or_default();
            let marker = SelectionMarker::underline(colour);
            row = row.child(track.indicator(self.active, kind, motion, marker, window, cx));
        }
        let inner = match scroll {
            Some(state) => self.scroll_strip(row, &state, window, cx),
            None => row.into_any_element(),
        };
        if !strip {
            return inner;
        }
        // The same bordered capsule as a segmented control; a row parent keeps it as wide as its
        // tabs, and a scrolling strip pans inside it.
        let p = &t.palette;
        let style =
            ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border_strong), false, false)
                .with_surface(&t, dodai_core::SurfaceClass::Bar);
        let capsule = crate::paint::shaped(&t.shape, style)
            .flex()
            .min_w_0()
            .max_w_full()
            .p(px(t.spacing.half()))
            .child(inner);
        div().flex().child(capsule).into_any_element()
    }
}

/// Pointer pan state of a scrollable strip.
#[derive(Clone, Copy, Default)]
struct Pan {
    /// Pointer x and scroll offset when the button went down.
    from: Option<(f32, f32)>,
    /// The pointer travelled far enough that this press is a drag, not a click.
    moved: bool,
}

struct ScrollState {
    handle: gpui::ScrollHandle,
    seen: usize,
    tries: u8,
    pan: Rc<Cell<Pan>>,
}

crate::entry_render!(Tabs);

impl Tabs {
    fn animation_of(&self, t: &Theme) -> SelectionMotion {
        self.animation.unwrap_or(t.look().selection)
    }
    fn motion_of(&self, cx: &App) -> Motion {
        self.motion.unwrap_or_else(|| Motion::of(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::light_rounded;

    #[test]
    fn active_tab_uses_indicator_tokens() {
        let t = light_rounded();
        let (s, fg, ind) = tab_style(&t, true, false);
        // M3 primary tab: no fill, `primary` label and indicator.
        assert!(s.fill.is_none());
        assert_eq!(fg, hsla(t.palette.primary));
        assert_eq!(ind, Some(hsla(t.palette.primary)));
        let (s, fg, ind) = tab_style(&t, false, false);
        assert!(s.fill.is_none() && ind.is_none());
        assert_eq!(fg, hsla(t.palette.roles.on_surface_variant));
        let (s, _, _) = tab_style(&t, false, true);
        assert_eq!(
            s.fill,
            Some(hsla(
                t.palette
                    .roles
                    .on_surface
                    .with_alpha(dodai_core::StateLayer::HOVER)
            ))
        );
    }
}
