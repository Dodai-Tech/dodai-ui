//! [`Tabs`]: a row of selectable tabs driven by `theme.tab` tokens.

use crate::interact::{track, use_interact};
use crate::paint::{bottom_band_canvas, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, Hsla, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// Resolved tab paint: shape style, text colour, optional indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.tab(active, hovered);
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
}

crate::corners_setter!(Tabs);

impl Tabs {
    /// Create a new [`Tabs`].
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
        }
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
        on_select, usize
    );
}

impl Tabs {
    /// Tab `i`: styled for its active / hover state, with its icon, label and indicator.
    fn tab(
        &self,
        t: &dodai_core::Theme,
        i: usize,
        pan: Option<Rc<Cell<Pan>>>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        let label = self.labels.get(i);
        let id: ElementId = (self.id.clone(), i.to_string()).into();
        let st = use_interact(&id, window, cx);
        let hovered = st.read(cx).hovered;
        let active = i == self.active;
        let p = crate::motion::transition(window, cx, id.clone(), active);
        let (mut style, fg, indicator) = tab_style(t, active, hovered);
        let (on_s, on_fg, _) = tab_style(t, true, hovered);
        let (off_s, off_fg, _) = tab_style(t, false, hovered);
        style.fill = crate::motion::mix_opt(off_s.fill, on_s.fill, p);
        let fg = if p > 0.0 && p < 1.0 {
            crate::motion::mix(off_fg, on_fg, p)
        } else {
            fg
        };
        let indicator = indicator
            .or_else(|| tab_style(t, true, hovered).2.filter(|_| p > 0.01))
            .map(|c| c.opacity(p));
        let fg = if self.secondary && active {
            hsla(t.palette.roles.on_surface)
        } else {
            fg
        };
        let stacked = label.is_some() && self.icons.get(i).is_some();
        let height = if stacked && !self.secondary {
            t.scale(dodai_core::Scale::Wide)
        } else {
            t.shape.control_height
        };
        let mut tab = track(shaped_id(id, &t.shape, style), &st)
            .min_h(px(height))
            .px(px(t.spacing.md))
            .flex()
            .items_center()
            .cursor_pointer()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg);
        if t.look().tonal {
            tab = crate::text::typeset(tab, t, dodai_core::TypeRole::TitleSmall);
        }
        if stacked && !self.secondary {
            tab = tab.flex_col().justify_center().gap(px(2.0));
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
        if let Some(f) = self.on_select.clone() {
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
            window.request_animation_frame();
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
            if max > px(0.0) && d.y.abs() > d.x.abs() {
                let mut o = wheel.offset();
                o.x = (o.x - d.y).clamp(-max, px(0.0));
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

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut row = div().flex().gap(px(t.spacing.xs));
        // Scrollable strips pan with the pointer too: a swipe or drag moves them sideways.
        let scroll = self.scrollable.then(|| {
            window.use_keyed_state((self.id.clone(), "scroll"), cx, |_, _| ScrollState {
                handle: gpui::ScrollHandle::new(),
                seen: usize::MAX,
                tries: 0,
                pan: Rc::new(Cell::new(Pan::default())),
            })
        });
        let count = self.labels.len().max(self.icons.len());
        for i in 0..count {
            let pan = scroll.as_ref().map(|s| s.read(cx).pan.clone());
            row = row.child(self.tab(&t, i, pan, window, cx));
        }
        if let Some(state) = scroll {
            return self.scroll_strip(row, &state, window, cx);
        }
        row.into_any_element()
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
