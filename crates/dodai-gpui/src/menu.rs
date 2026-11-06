//! [`Menu`]: popup list of actions (use as a dropdown or context menu body).

use crate::interact::{track, use_interact, IndexHandler};
use crate::list::{list_style, Divider};
use crate::paint::{shaped, shaped_id};
use crate::select::popup_style;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, SharedString, StatefulInteractiveElement,
    Styled, Window,
};
use std::rc::Rc;

type SubHandler = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// One row of a [`Menu`].
#[derive(Clone)]
pub struct MenuItem {
    label: SharedString,
    shortcut: Option<SharedString>,
    disabled: bool,
    danger: bool,
    checked: bool,
    separator: bool,
    icon: Option<crate::icons::IconName>,
    children: Vec<MenuItem>,
}

impl MenuItem {
    /// Create a new [`MenuItem`].
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            disabled: false,
            danger: false,
            checked: false,
            separator: false,
            icon: None,
            children: Vec::new(),
        }
    }
    /// Leading icon (replaces the check column).
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Nested items: shown in a sub-menu to the right while this row is hovered.
    pub fn submenu(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.children = items.into_iter().collect();
        self
    }
    /// Horizontal rule between groups.
    pub fn separator() -> Self {
        Self {
            separator: true,
            ..Self::new("")
        }
    }
    /// Right-aligned key hint, e.g. `⌘C`.
    pub fn shortcut(mut self, s: impl Into<SharedString>) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Disable the control.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Destructive action (danger colour).
    pub fn danger(mut self, d: bool) -> Self {
        self.danger = d;
        self
    }
    /// Show a ✓ before the label.
    pub fn checked(mut self, c: bool) -> Self {
        self.checked = c;
        self
    }
}

/// Floating action list. `on_select` receives the index into the item list.
#[derive(IntoElement)]
pub struct Menu {
    id: ElementId,
    items: Vec<MenuItem>,
    width: f32,
    active: Option<usize>,
    on_select: Option<IndexHandler>,
    on_select_sub: Option<SubHandler>,
    corners: Option<dodai_core::Corners>,
    surface: dodai_core::Surface,
}

crate::corners_setter!(Menu);
crate::surface_setter!(Menu);

impl Menu {
    /// Create a new [`Menu`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            surface: dodai_core::Surface::INHERIT,
            id: id.into(),
            items: Vec::new(),
            width: 220.0,
            active: None,
            on_select: None,
            on_select_sub: None,
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
    /// Highlight the item at `i` (index into the item list) as the current
    /// one, e.g. the section a [`crate::ScrollSpy`] reports.
    pub fn active(mut self, i: Option<usize>) -> Self {
        self.active = i;
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select, usize
    );
    /// Called with `(parent index, child index)` when a sub-menu item is chosen.
    pub fn on_select_sub(
        mut self,
        f: impl Fn(usize, usize, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select_sub = Some(Rc::new(f));
        self
    }
}

impl Menu {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut col = shaped(
            &t.role_shape(dodai_core::ShapeRole::Menu),
            popup_style(&t).with_own_surface(&t, dodai_core::SurfaceClass::Panel, &self.surface),
        )
        .w(px(self.width))
        .p(px(t.spacing.xs + 2.0))
        .flex()
        .flex_col()
        .gap(px(t.spacing.half()))
        .font_family(first_family(t.typography.body))
        .text_size(px(t.typography.size_md));
        if t.look().tonal {
            col = crate::text::typeset(col, &t, dodai_core::TypeRole::LabelLarge);
        }
        let open = window.use_keyed_state((self.id.clone(), "open"), cx, |_, _| None::<usize>);
        // Keyboard: up/down walk the enabled rows, Enter or Space chooses, Right opens a
        // sub-menu and Left closes it. The row the keys are on is `cursor`.
        let cursor = window.use_keyed_state((self.id.clone(), "cursor"), cx, |_, _| None::<usize>);
        let focus = window
            .use_keyed_state((self.id.clone(), "focus"), cx, |_, cx| {
                cx.focus_handle().tab_stop(true)
            })
            .read(cx)
            .clone();
        let keys = {
            let rows: Vec<(bool, bool)> = self
                .items
                .iter()
                .map(|m| (!m.separator && !m.disabled, !m.children.is_empty()))
                .collect();
            let (cursor, open, f) = (cursor.clone(), open.clone(), self.on_select.clone());
            move |e: &gpui::KeyDownEvent, w: &mut Window, cx: &mut App| {
                let key = e.keystroke.key.as_str();
                let at = *cursor.read(cx);
                let enabled = |i: usize| rows[i].0;
                match key {
                    "down" | "up" | "home" | "end" => {
                        let to = match (at, key) {
                            (Some(a), _) => crate::roving::next(
                                key,
                                crate::roving::Axis::Vertical,
                                a,
                                rows.len(),
                                enabled,
                            ),
                            (None, "up" | "end") => (0..rows.len()).rev().find(|i| enabled(*i)),
                            (None, _) => (0..rows.len()).find(|i| enabled(*i)),
                        };
                        if let Some(to) = to {
                            cursor.update(cx, |c, cx| {
                                *c = Some(to);
                                cx.notify();
                            });
                            open.update(cx, |o, cx| {
                                *o = None;
                                cx.notify();
                            });
                        }
                    }
                    "right" => {
                        if let Some(a) = at.filter(|a| rows[*a].1) {
                            open.update(cx, |o, cx| {
                                *o = Some(a);
                                cx.notify();
                            });
                        }
                    }
                    "left" => open.update(cx, |o, cx| {
                        *o = None;
                        cx.notify();
                    }),
                    "enter" | "space" => {
                        if let (Some(a), Some(f)) = (at.filter(|a| rows[*a].0), &f) {
                            if rows[a].1 {
                                open.update(cx, |o, cx| {
                                    *o = Some(a);
                                    cx.notify();
                                });
                            } else {
                                f(a, w, cx);
                            }
                        }
                    }
                    _ => {}
                }
            }
        };
        col = gpui::InteractiveElement::on_key_down(
            gpui::InteractiveElement::track_focus(col, &focus),
            keys,
        );
        col = gpui::InteractiveElement::on_mouse_down(col, gpui::MouseButton::Left, {
            let focus = focus.clone();
            move |_, w, _| w.focus(&focus)
        });
        for (i, item) in self.items.iter().enumerate() {
            if item.separator {
                col = col.child(div().py(px(t.spacing.xs)).child(Divider::new()));
                continue;
            }
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered && !item.disabled;
            if !item.disabled && st.read(cx).hovered {
                if *cursor.read(cx) != Some(i) {
                    cursor.update(cx, |c, cx| {
                        *c = Some(i);
                        cx.notify();
                    });
                }
                let want = (!item.children.is_empty()).then_some(i);
                if *open.read(cx) != want {
                    open.update(cx, |o, cx| {
                        *o = want;
                        cx.notify();
                    });
                }
            }
            let current = (self.active == Some(i) || *cursor.read(cx) == Some(i)) && !item.disabled;
            let lit = hovered || current;
            let (style, _, marker) = list_style(&t, lit, false);
            let mut fg = if hovered || !item.disabled {
                hsla(t.palette.text)
            } else {
                hsla(t.palette.text_disabled)
            };
            if item.danger && !item.disabled {
                fg = hsla(t.palette.danger);
            }
            let slot = t.sizes().icon_sm + t.spacing.half();
            let mut row = track(shaped_id(id, &t.shape, style), &st, cx)
                .min_h(px(t.menu_row_height()))
                .py(px(t.spacing.half()))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .text_color(fg)
                .child(match item.icon {
                    Some(ic) => div()
                        .w(px(slot))
                        .min_w(px(slot))
                        .max_w(px(slot))
                        .flex_none()
                        .flex()
                        .justify_center()
                        .overflow_hidden()
                        .child(
                            crate::icons::Icon::new(ic)
                                .size(t.sizes().icon_sm)
                                .color(fg),
                        ),
                    None => div().w(px(slot * 0.7)).flex_none().child(if item.checked {
                        "✓"
                    } else {
                        ""
                    }),
                })
                .child(
                    div()
                        .flex_1()
                        .whitespace_nowrap()
                        .child(format_label(&t.typography, &item.label)),
                );
            if let Some(s) = &item.shortcut {
                row = row.child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(s.clone()),
                );
            }
            if !item.children.is_empty() {
                row = row.child(crate::surfaces::glyph_el("▸".into(), t.sizes().icon_sm, fg));
                if *open.read(cx) == Some(i) {
                    let parent = i;
                    let mut sub = Menu::new((self.id.clone(), format!("sub{i}")))
                        .items(item.children.clone())
                        .width((self.width * 0.8).max(160.0));
                    if let Some(f) = self.on_select_sub.clone() {
                        sub = sub.on_select(move |c, w, cx| f(parent, c, w, cx));
                    }
                    row = row.child(
                        div()
                            .absolute()
                            .left(px(self.width - t.spacing.xs * 2.0))
                            .top(px(-(t.spacing.xs + 2.0)))
                            .child(sub),
                    );
                }
            }
            if let (Some(m), true) = (marker, lit) {
                row = row.child(
                    div()
                        .absolute()
                        .left_0()
                        .top_0()
                        .bottom_0()
                        .w(px(t.shape.border_width.max(2.0)))
                        .bg(m),
                );
            }
            if !item.disabled {
                row = row.cursor_pointer();
                if let Some(f) = self.on_select.clone() {
                    row = row.on_click(move |_, w, cx| f(i, w, cx));
                }
            }
            col = col.child(row);
        }
        col
    }
}

crate::entry_render!(Menu);
