//! Anchored overlays: [`Popover`] (click), [`HoverCard`] (hover),
//! [`ContextMenu`] (right-click) and [`DropdownMenu`] (button + menu).
//!
//! Every overlay closes on an outside click, `Esc` (Popover) or after a
//! choice (DropdownMenu, ContextMenu), like [`crate::Select`].

use crate::interact::IndexHandler;
use crate::menu::{Menu, MenuItem};
use crate::paint::shaped;
use crate::select::popup_style;
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    anchored, canvas, deferred, div, px, AnyElement, App, Bounds, ElementId, FocusHandle,
    InteractiveElement, IntoElement, MouseButton, ParentElement, Pixels, Point, RenderOnce,
    StatefulInteractiveElement, Styled, Window,
};

/// Side of the anchor an overlay opens on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PopoverPlacement {
    /// Below the trigger.
    #[default]
    Below,
    /// Above the trigger.
    Above,
    /// To the right of the trigger.
    Right,
    /// To the left of the trigger.
    Left,
}

impl PopoverPlacement {
    /// Position `d` (already `absolute`) against its relative anchor, `gap` px away.
    fn place(self, d: gpui::Div, gap: f32) -> gpui::Div {
        match self {
            PopoverPlacement::Below => d.top_full().left_0().mt(px(gap)),
            PopoverPlacement::Above => d.bottom_full().left_0().mb(px(gap)),
            PopoverPlacement::Right => d.left_full().top_0().ml(px(gap)),
            PopoverPlacement::Left => d.right_full().top_0().mr(px(gap)),
        }
    }
    /// Transparent padding that bridges the anchor and the card (hover cards).
    fn bridge(self, d: gpui::Div, gap: f32) -> gpui::Div {
        match self {
            PopoverPlacement::Below => d.top_full().left_0().pt(px(gap)),
            PopoverPlacement::Above => d.bottom_full().left_0().pb(px(gap)),
            PopoverPlacement::Right => d.left_full().top_0().pl(px(gap)),
            PopoverPlacement::Left => d.right_full().top_0().pr(px(gap)),
        }
    }
}

// ───────────────────────────── popover ─────────────────────────────

struct PopState {
    open: bool,
    bounds: Bounds<Pixels>,
    focus: FocusHandle,
}

/// Click-to-open panel anchored to a trigger. Closes on outside click or `Esc`.
#[derive(IntoElement)]
pub struct Popover {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    placement: PopoverPlacement,
    width: Option<f32>,
    close_on_click: bool,
    bare: bool,
}

impl Popover {
    /// Create a new [`Popover`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            placement: PopoverPlacement::Below,
            width: None,
            close_on_click: false,
            bare: false,
        }
    }
    /// The element that toggles the popover.
    pub fn trigger(mut self, t: impl IntoElement) -> Self {
        self.trigger = Some(t.into_any_element());
        self
    }
    /// What the popover shows.
    pub fn content(mut self, c: impl IntoElement) -> Self {
        self.content = Some(c.into_any_element());
        self
    }
    /// Where the popup sits relative to its trigger.
    pub fn placement(mut self, p: PopoverPlacement) -> Self {
        self.placement = p;
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// Close after any click inside the content (menus).
    pub fn close_on_click(mut self, on: bool) -> Self {
        self.close_on_click = on;
        self
    }
    /// No frame or padding: the content already is a popup (e.g. a [`Menu`]).
    pub fn bare(mut self, on: bool) -> Self {
        self.bare = on;
        self
    }
}

impl RenderOnce for Popover {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, cx| PopState {
            open: false,
            bounds: Bounds::default(),
            focus: cx.focus_handle(),
        });
        let (open, focus) = {
            let s = st.read(cx);
            (s.open, s.focus.clone())
        };
        let close = {
            let st = st.clone();
            move |cx: &mut App| {
                st.update(cx, |s, cx| {
                    if s.open {
                        s.open = false;
                        cx.notify();
                    }
                })
            }
        };
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .track_focus(&focus)
            .on_key_down({
                let close = close.clone();
                move |e, _, cx| {
                    if e.keystroke.key == "escape" {
                        close(cx);
                    }
                }
            })
            .child(
                canvas(
                    {
                        let st = st.clone();
                        move |b, _, cx| st.update(cx, |s, _| s.bounds = b)
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .size_full(),
            )
            .child(
                div()
                    .id((self.id.clone(), "trigger"))
                    .cursor_pointer()
                    .on_click({
                        let st = st.clone();
                        let focus = focus.clone();
                        move |_, w, cx| {
                            w.focus(&focus);
                            st.update(cx, |s, cx| {
                                s.open = !s.open;
                                cx.notify();
                            });
                        }
                    })
                    .children(self.trigger.take()),
            );
        if open {
            if let Some(content) = self.content.take() {
                let mut popup = div().absolute();
                if !self.bare {
                    popup = shaped(&t.role_shape(dodai_core::ShapeRole::Menu), popup_style(&t))
                        .absolute()
                        .p(px(t.spacing.md))
                        .font_family(first_family(t.typography.body))
                        .text_size(px(t.typography.size_md))
                        .text_color(hsla(t.palette.text));
                }
                popup = self.placement.place(popup, 6.0).occlude();
                if let Some(w) = self.width {
                    popup = popup.w(px(w));
                }
                let out = st.clone();
                let mut popup = popup
                    .id((self.id.clone(), "popup"))
                    .on_mouse_down_out(move |e, _, cx| {
                        let inside = out.read(cx).bounds.contains(&e.position);
                        if !inside {
                            out.update(cx, |s, cx| {
                                s.open = false;
                                cx.notify();
                            });
                        }
                    })
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
                if self.close_on_click {
                    popup = popup.on_click({
                        let close = close.clone();
                        move |_, _, cx| close(cx)
                    });
                }
                root = root.child(deferred(popup.child(content)).with_priority(10));
            }
        }
        root
    }
}

// ───────────────────────────── hover card ─────────────────────────────

#[derive(Default)]
struct HoverState {
    trigger: bool,
    card: bool,
}

/// Card shown while the pointer is over the trigger or the card itself.
#[derive(IntoElement)]
pub struct HoverCard {
    id: ElementId,
    trigger: Option<AnyElement>,
    content: Option<AnyElement>,
    placement: PopoverPlacement,
    width: f32,
}

impl HoverCard {
    /// Create a new [`HoverCard`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            trigger: None,
            content: None,
            placement: PopoverPlacement::Below,
            width: 260.0,
        }
    }
    /// The element that opens the popup.
    pub fn trigger(mut self, t: impl IntoElement) -> Self {
        self.trigger = Some(t.into_any_element());
        self
    }
    /// The popup content.
    pub fn content(mut self, c: impl IntoElement) -> Self {
        self.content = Some(c.into_any_element());
        self
    }
    /// Where the popup sits relative to its trigger.
    pub fn placement(mut self, p: PopoverPlacement) -> Self {
        self.placement = p;
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl RenderOnce for HoverCard {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| HoverState::default());
        let open = {
            let s = st.read(cx);
            s.trigger || s.card
        };
        let set = |st: &gpui::Entity<HoverState>, cx: &mut App, trig: bool, v: bool| {
            st.update(cx, |s, cx| {
                let slot = if trig { &mut s.trigger } else { &mut s.card };
                if *slot != v {
                    *slot = v;
                    cx.notify();
                }
            })
        };
        let mut root = div()
            .id(self.id.clone())
            .relative()
            .flex_none()
            .on_hover({
                let st = st.clone();
                move |h, _, cx| set(&st, cx, true, *h)
            })
            .children(self.trigger.take());
        if open {
            if let Some(content) = self.content.take() {
                let card = shaped(&t.role_shape(dodai_core::ShapeRole::Menu), popup_style(&t))
                    .w(px(self.width))
                    .p(px(t.spacing.md))
                    .font_family(first_family(t.typography.body))
                    .text_size(px(t.typography.size_md))
                    .text_color(hsla(t.palette.text))
                    .child(content);
                let outer = self
                    .placement
                    .bridge(div().absolute(), 6.0)
                    .occlude()
                    .id((self.id.clone(), "card"))
                    .on_hover({
                        let st = st.clone();
                        move |h, _, cx| set(&st, cx, false, *h)
                    })
                    .child(card);
                root = root.child(deferred(outer).with_priority(10));
            }
        }
        root
    }
}

// ───────────────────────────── context menu ─────────────────────────────

#[derive(Default)]
struct CtxState {
    at: Option<Point<Pixels>>,
}

/// Right-click anywhere on the wrapped area to open a [`Menu`] at the pointer.
#[derive(IntoElement)]
pub struct ContextMenu {
    id: ElementId,
    child: Option<AnyElement>,
    items: Vec<MenuItem>,
    width: f32,
    on_select: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
    surface: dodai_core::Surface,
}

crate::corners_setter!(ContextMenu);
crate::surface_setter!(ContextMenu);

impl ContextMenu {
    /// Create a new [`ContextMenu`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            child: None,
            items: Vec::new(),
            width: 200.0,
            on_select: None,
            corners: None,
            surface: dodai_core::Surface::INHERIT,
        }
    }
    /// The area that reacts to right-clicks.
    pub fn child(mut self, c: impl IntoElement) -> Self {
        self.child = Some(c.into_any_element());
        self
    }
    /// The items.
    pub fn items(mut self, i: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = i.into_iter().collect();
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select,
        usize
    );
}

impl RenderOnce for ContextMenu {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| CtxState::default());
        let at = st.read(cx).at;
        let mut root = div()
            .id(self.id.clone())
            .on_mouse_down(MouseButton::Right, {
                let st = st.clone();
                move |e, _, cx| {
                    let p = e.position;
                    st.update(cx, |s, cx| {
                        s.at = Some(p);
                        cx.notify();
                    });
                }
            })
            .children(self.child.take());
        if let Some(p) = at {
            let close = {
                let st = st.clone();
                move |cx: &mut App| {
                    st.update(cx, |s, cx| {
                        s.at = None;
                        cx.notify();
                    })
                }
            };
            let sel = self.on_select.clone();
            let mut menu = Menu::new((self.id.clone(), "menu"))
                .items(self.items.clone())
                .width(self.width)
                .surface(self.surface)
                .on_select({
                    let close = close.clone();
                    move |i, w, cx| {
                        close(cx);
                        if let Some(f) = &sel {
                            f(i, w, cx);
                        }
                    }
                });
            if let Some(c) = self.corners {
                menu = menu.corners(c);
            }
            let host = div()
                .occlude()
                .on_mouse_down_out(move |_, _, cx| close(cx))
                .child(menu);
            root = root.child(
                deferred(
                    anchored()
                        .position(p)
                        .snap_to_window_with_margin(px(crate::theme_of(cx).spacing.sm))
                        .child(host),
                )
                .with_priority(20),
            );
        }
        root
    }
}

// ───────────────────────────── dropdown menu ─────────────────────────────

/// A button that opens a [`Menu`] below it; closes after a choice or an outside click.
#[derive(IntoElement)]
pub struct DropdownMenu {
    id: ElementId,
    label: gpui::SharedString,
    items: Vec<MenuItem>,
    width: f32,
    on_select: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
    surface: dodai_core::Surface,
}

crate::corners_setter!(DropdownMenu);
crate::surface_setter!(DropdownMenu);

impl DropdownMenu {
    /// Create a new [`DropdownMenu`].
    pub fn new(id: impl Into<ElementId>, label: impl Into<gpui::SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            items: Vec::new(),
            width: 200.0,
            on_select: None,
            corners: None,
            surface: dodai_core::Surface::INHERIT,
        }
    }
    /// The items.
    pub fn items(mut self, i: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = i.into_iter().collect();
        self
    }
    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select,
        usize
    );
}

impl RenderOnce for DropdownMenu {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let mut menu = Menu::new((self.id.clone(), "menu"))
            .items(self.items)
            .width(self.width)
            .surface(self.surface);
        if let Some(c) = self.corners {
            menu = menu.corners(c);
        }
        if let Some(f) = self.on_select {
            menu = menu.on_select(move |i, w, cx| f(i, w, cx));
        }
        Popover::new((self.id.clone(), "pop"))
            .trigger(
                crate::Button::new((self.id.clone(), "btn"), format!("{} ▾", self.label))
                    .variant(dodai_core::ButtonVariant::Outline),
            )
            .content(menu)
            .bare(true)
            .close_on_click(true)
    }
}
