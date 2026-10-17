//! [`DialogCard`] (the dialog box) and [`Dialog`] (card + full-window scrim).

use crate::paint::{shaped, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    deferred, div, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, MouseButton,
    ParentElement, RenderOnce, SharedString, Styled, Window,
};
use std::rc::Rc;

/// Paint style of the dialog box (pure): raised surface, strong border, glow, hard shadow.
pub fn dialog_style(theme: &Theme) -> ShapeStyle {
    ShapeStyle::from_tokens(
        theme,
        Some(theme.palette.surface),
        Some(theme.palette.primary),
        true,
        true,
    )
}

/// Scrim colour: the theme overlay colour, never fully opaque.
pub fn scrim_color(theme: &Theme) -> gpui::Hsla {
    hsla(theme.palette.background.with_alpha(0.7))
}

/// Dialog box: title, body children, action row (right aligned).
#[derive(IntoElement)]
pub struct DialogCard {
    title: SharedString,
    body: Vec<AnyElement>,
    actions: Vec<AnyElement>,
    width: f32,
}

impl DialogCard {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: Vec::new(),
            actions: Vec::new(),
            width: 420.0,
        }
    }
    /// Footer action (usually a `Button`); order is left to right.
    pub fn action(mut self, a: impl IntoElement) -> Self {
        self.actions.push(a.into_any_element());
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl ParentElement for DialogCard {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.body.extend(elements);
    }
}

impl RenderOnce for DialogCard {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(cx), "DialogCard", motion)
    }
}

impl DialogCard {
    fn build(self, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let mut d = shaped(&t.shape, dialog_style(&t))
            .w(px(self.width))
            .p(px(t.spacing.lg))
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text))
            .child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_lg))
                    .child(format_label(&t.typography, &self.title)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.sm))
                    .text_color(hsla(t.palette.text_muted))
                    .children(self.body),
            );
        if !self.actions.is_empty() {
            d = d.child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(t.spacing.sm))
                    .children(self.actions),
            );
        }
        d
    }
}

type DismissFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Modal: scrim over the whole (relatively positioned) parent with a centred
/// [`DialogCard`]. Clicking the scrim calls `on_dismiss`. Render it last in the
/// root element; it sits above siblings via `deferred`.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    card: DialogCard,
    on_dismiss: Option<DismissFn>,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, card: DialogCard) -> Self {
        Self {
            id: id.into(),
            card,
            on_dismiss: None,
        }
    }
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let dismiss = self.on_dismiss;
        deferred(
            div()
                .id(self.id)
                .absolute()
                .top_0()
                .left_0()
                .size_full()
                .occlude()
                .bg(scrim_color(&t))
                .flex()
                .items_center()
                .justify_center()
                .on_mouse_down(MouseButton::Left, move |_, w, cx| {
                    if let Some(f) = &dismiss {
                        f(w, cx);
                    }
                })
                .child(
                    div()
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .child(self.card),
                ),
        )
        .with_priority(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn scrim_is_translucent_and_themed() {
        for t in [light_rounded(), square()] {
            let s = scrim_color(&t);
            assert!(s.a < 1.0 && s.a > 0.3);
            assert_eq!(s.l, hsla(t.palette.background).l);
        }
    }

    #[test]
    fn dialog_follows_tokens() {
        let l = dialog_style(&light_rounded());
        assert!(l.glow.is_none() && l.shadow.is_none());
        assert!(dialog_style(&square()).shadow.is_some());
    }
}
