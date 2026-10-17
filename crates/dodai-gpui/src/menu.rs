//! [`Menu`]: popup list of actions (use as a dropdown or context menu body).

use crate::interact::{track, use_interact, IndexHandler};
use crate::list::{list_style, Divider};
use crate::paint::{shaped, shaped_id};
use crate::select::popup_style;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// One row of a [`Menu`].
#[derive(Clone)]
pub struct MenuItem {
    label: SharedString,
    shortcut: Option<SharedString>,
    disabled: bool,
    danger: bool,
    checked: bool,
    separator: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            disabled: false,
            danger: false,
            checked: false,
            separator: false,
        }
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
    on_select: Option<IndexHandler>,
    corners: Option<dodai_core::Corners>,
}

impl Menu {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl Menu {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            items: Vec::new(),
            width: 220.0,
            on_select: None,
        }
    }
    pub fn items(mut self, i: impl IntoIterator<Item = MenuItem>) -> Self {
        self.items = i.into_iter().collect();
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl Menu {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut col = shaped(&t.shape, popup_style(&t))
            .w(px(self.width))
            .p(px(t.spacing.xs + 2.0))
            .flex()
            .flex_col()
            .gap(px(2.0))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md));
        for (i, item) in self.items.iter().enumerate() {
            if item.separator {
                col = col.child(div().py(px(t.spacing.xs)).child(Divider::new()));
                continue;
            }
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered && !item.disabled;
            let (style, _, marker) = list_style(&t, hovered, false);
            let mut fg = if hovered || !item.disabled {
                hsla(t.palette.text)
            } else {
                hsla(t.palette.text_disabled)
            };
            if item.danger && !item.disabled {
                fg = hsla(t.palette.danger);
            }
            let mut row = track(shaped_id(id, &t.shape, style), &st)
                .h(px(t.shape.control_height * 0.8))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .text_color(fg)
                .child(
                    div()
                        .w(px(14.0))
                        .flex_none()
                        .child(if item.checked { "✓" } else { "" }),
                )
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
            if let (Some(m), true) = (marker, hovered) {
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

impl RenderOnce for Menu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "Menu", motion)
    }
}
