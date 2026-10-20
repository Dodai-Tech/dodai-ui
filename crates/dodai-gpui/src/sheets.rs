//! Modal surfaces: [`BottomSheet`], [`SideSheet`], [`FullScreenDialog`] and
//! the action [`Snackbar`]. Render modals last in a relatively positioned root.

use crate::button::button_style;
use crate::overlay::scrim_color;
use crate::paint::{shaped, ShapeStyle};
use crate::surfaces::IconButton;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, Theme};
use gpui::StatefulInteractiveElement as _;
use gpui::{
    deferred, div, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, Styled, Window,
};
use std::rc::Rc;

type DismissFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Edge a sheet is attached to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SheetEdge {
    Left,
    #[default]
    Right,
}

/// Paint style of a sheet surface (pure).
pub fn sheet_style(theme: &Theme) -> ShapeStyle {
    ShapeStyle::from_surface(theme, theme.sheet()).0
}

fn scrim(
    id: ElementId,
    t: &Theme,
    dismiss: Option<DismissFn>,
    dim: bool,
) -> gpui::Stateful<gpui::Div> {
    let mut s = div()
        .id(id)
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .occlude()
        .flex();
    if dim {
        s = s.bg(scrim_color(t));
    }
    s.on_mouse_down(MouseButton::Left, move |_, w, cx| {
        if let Some(f) = &dismiss {
            f(w, cx);
        }
    })
}

fn swallow(el: gpui::Div) -> gpui::Stateful<gpui::Div> {
    el.id("sheet-body")
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
}

fn title_el(t: &Theme, s: &SharedString) -> gpui::Div {
    div()
        .font_family(first_family(t.typography.display))
        .text_size(px(t.typography.size_lg))
        .child(format_label(&t.typography, s))
}

// ───────────────────────────── BottomSheet ─────────────────────────────

/// Sheet anchored to the bottom edge with a drag-handle affordance.
#[derive(IntoElement)]
pub struct BottomSheet {
    id: ElementId,
    title: Option<SharedString>,
    height: f32,
    body: Vec<AnyElement>,
    on_dismiss: Option<DismissFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(BottomSheet);

impl BottomSheet {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            title: None,
            height: 280.0,
            body: Vec::new(),
            on_dismiss: None,
        }
    }
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    crate::callback!(on_dismiss);
}

impl ParentElement for BottomSheet {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(e);
    }
}

impl RenderOnce for BottomSheet {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let handle = div()
            .w(px(40.0))
            .h(px(4.0))
            .bg(hsla(t.palette.border_strong));
        let mut sheet = shaped(&t.shape, sheet_style(&t))
            .w_full()
            .h(px(self.height))
            .p(px(t.spacing.lg))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text))
            .child(handle);
        if let Some(title) = &self.title {
            sheet = sheet.child(title_el(&t, title));
        }
        let sheet = sheet.child(
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .children(self.body),
        );
        deferred(
            scrim(self.id, &t, self.on_dismiss, true)
                .items_end()
                .child(swallow(div().w_full().child(sheet))),
        )
        .with_priority(100)
    }
}

// ────────────────────────────── SideSheet ──────────────────────────────

/// Sheet attached to the left or right edge with a header and close button.
#[derive(IntoElement)]
pub struct SideSheet {
    id: ElementId,
    title: SharedString,
    edge: SheetEdge,
    width: f32,
    modal: bool,
    body: Vec<AnyElement>,
    on_dismiss: Option<DismissFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(SideSheet);

impl SideSheet {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            title: title.into(),
            edge: SheetEdge::Right,
            width: 320.0,
            modal: true,
            body: Vec::new(),
            on_dismiss: None,
        }
    }
    pub fn edge(mut self, e: SheetEdge) -> Self {
        self.edge = e;
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
    /// Non-modal sheets draw no scrim.
    pub fn modal(mut self, m: bool) -> Self {
        self.modal = m;
        self
    }
    crate::callback!(on_dismiss);
}

impl ParentElement for SideSheet {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(e);
    }
}

impl RenderOnce for SideSheet {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let close = self.on_dismiss.clone();
        let mut x = IconButton::new((self.id.clone(), "close"), "✕");
        if let Some(f) = close {
            x = x.on_click(move |_, w, cx| f(w, cx));
        }
        let sheet = shaped(&t.shape, sheet_style(&t))
            .w(px(self.width))
            .h_full()
            .p(px(t.spacing.lg))
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(title_el(&t, &self.title))
                    .child(x),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.sm))
                    .children(self.body),
            );
        let mut layer = scrim(self.id, &t, self.on_dismiss, self.modal);
        layer = match self.edge {
            SheetEdge::Left => layer.justify_start(),
            SheetEdge::Right => layer.justify_end(),
        };
        deferred(layer.child(swallow(div().h_full().child(sheet)))).with_priority(100)
    }
}

// ─────────────────────────── FullScreenDialog ───────────────────────────

/// Dialog covering the whole parent: close button, title, primary action.
#[derive(IntoElement)]
pub struct FullScreenDialog {
    id: ElementId,
    title: SharedString,
    action: Option<SharedString>,
    body: Vec<AnyElement>,
    on_dismiss: Option<DismissFn>,
    on_action: Option<DismissFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(FullScreenDialog);

impl FullScreenDialog {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            title: title.into(),
            action: None,
            body: Vec::new(),
            on_dismiss: None,
            on_action: None,
        }
    }
    /// Label of the header action button (e.g. "Save").
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(label.into());
        self.on_action = Some(Rc::new(f));
        self
    }
    crate::callback!(on_dismiss);
}

impl ParentElement for FullScreenDialog {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(e);
    }
}

impl RenderOnce for FullScreenDialog {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let mut close = IconButton::new((self.id.clone(), "close"), "✕");
        if let Some(f) = self.on_dismiss.clone() {
            close = close.on_click(move |_, w, cx| f(w, cx));
        }
        let mut header = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.md))
            .child(close)
            .child(div().flex_1().child(title_el(&t, &self.title)));
        if let (Some(label), Some(f)) = (self.action, self.on_action) {
            header = header.child(
                crate::Button::new((self.id.clone(), "action"), label)
                    .variant(ButtonVariant::Primary)
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let body = shaped(&t.shape, sheet_style(&t))
            .size_full()
            .p(px(t.spacing.lg))
            .flex()
            .flex_col()
            .gap(px(t.spacing.lg))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text))
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.md))
                    .children(self.body),
            );
        deferred(
            div()
                .id(self.id)
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .bg(hsla(t.palette.background))
                .child(body),
        )
        .with_priority(100)
    }
}

// ────────────────────────────── Snackbar ──────────────────────────────

/// Message bar with an optional text action.
#[derive(IntoElement)]
pub struct Snackbar {
    id: ElementId,
    message: SharedString,
    action: Option<SharedString>,
    on_action: Option<DismissFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Snackbar);

impl Snackbar {
    pub fn new(id: impl Into<ElementId>, message: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            message: message.into(),
            action: None,
            on_action: None,
        }
    }
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(label.into());
        self.on_action = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Snackbar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::current(cx);
        crate::motion::entry(self.build(cx), "Snackbar", motion)
    }
}

impl Snackbar {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let (tokens, action_colour) = t.snackbar();
        let (style, _) = button_style(&t, ButtonVariant::Secondary, dodai_core::ControlState::Idle);
        let style = ShapeStyle {
            fill: tokens.background.map(hsla),
            stroke: tokens
                .border
                .map(hsla)
                .or(style.stroke)
                .filter(|_| tokens.border.is_some()),
            ..style
        };
        let mut bar = shaped(&t.shape, style)
            .px(px(t.spacing.lg))
            .py(px(t.spacing.sm))
            .flex()
            .items_center()
            .justify_between()
            .gap(px(t.spacing.lg))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(tokens.foreground))
            .child(self.message);
        if let (Some(l), Some(f)) = (self.action, self.on_action) {
            bar = bar.child(
                div()
                    .id(self.id)
                    .cursor_pointer()
                    .px(px(t.spacing.sm))
                    .text_color(hsla(action_colour))
                    .child(l)
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        bar
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn sheet_style_is_themed() {
        for t in [light_rounded(), square()] {
            let s = sheet_style(&t);
            assert_eq!(s.fill, Some(hsla(t.palette.surface)));
            assert!(s.stroke.is_some());
        }
    }
}
