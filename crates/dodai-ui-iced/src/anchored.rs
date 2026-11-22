//! A panel anchored under its trigger and drawn above everything else (a popover, a hover
//! card, a dropdown or context menu): the trigger keeps its place in the layout, the panel
//! floats in an overlay, so opening it never moves the page.

use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{mouse, overlay, renderer, Clipboard, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// `trigger` with `panel` floating `gap` px under it while `panel` is `Some`.
pub struct Anchored<'a, Message> {
    trigger: Element<'a, Message>,
    panel: Option<Element<'a, Message>>,
    gap: f32,
}

/// `trigger` with `panel` (when `Some`) floating `gap` px under its start edge.
pub fn anchored<'a, Message: 'a>(
    trigger: impl Into<Element<'a, Message>>,
    panel: Option<Element<'a, Message>>,
    gap: f32,
) -> Element<'a, Message> {
    Element::new(Anchored {
        trigger: trigger.into(),
        panel,
        gap,
    })
}

impl<Message> Widget<Message, iced::Theme, iced::Renderer> for Anchored<'_, Message> {
    fn children(&self) -> Vec<Tree> {
        let mut c = vec![Tree::new(&self.trigger)];
        if let Some(p) = &self.panel {
            c.push(Tree::new(p));
        }
        c
    }

    fn diff(&self, tree: &mut Tree) {
        match &self.panel {
            Some(p) => tree.diff_children(&[self.trigger.as_widget(), p.as_widget()]),
            None => tree.diff_children(&[self.trigger.as_widget()]),
        }
    }

    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.trigger
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.trigger.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.trigger.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, iced::Theme, iced::Renderer>> {
        let (first, rest) = tree.children.split_at_mut(1);
        let inner = self.trigger.as_widget_mut().overlay(
            &mut first[0],
            layout,
            renderer,
            viewport,
            translation,
        );
        let bounds = layout.bounds();
        let panel = match (&mut self.panel, rest.first_mut()) {
            (Some(p), Some(t)) => Some(overlay::Element::new(Box::new(Floating {
                panel: p,
                tree: t,
                at: Point::new(
                    bounds.x + translation.x,
                    bounds.y + bounds.height + self.gap + translation.y,
                ),
            }))),
            _ => None,
        };
        let all: Vec<_> = inner.into_iter().chain(panel).collect();
        (!all.is_empty()).then(|| overlay::Group::with_children(all).overlay())
    }
}

/// The open panel, at `at` (kept inside the window).
struct Floating<'a, 'b, Message> {
    panel: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    at: Point,
}

impl<Message> overlay::Overlay<Message, iced::Theme, iced::Renderer> for Floating<'_, '_, Message> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        let node = self.panel.as_widget_mut().layout(
            self.tree,
            renderer,
            &layout::Limits::new(Size::ZERO, bounds),
        );
        let size = node.size();
        let x = self.at.x.min(bounds.width - size.width).max(0.0);
        let y = self.at.y.min(bounds.height - size.height).max(0.0);
        node.move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &iced::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.panel.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();
        self.panel.as_widget_mut().update(
            self.tree, event, layout, cursor, renderer, clipboard, shell, &bounds,
        );
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.panel.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_open_and_closed() {
        let _: Element<'_, ()> = anchored(iced::widget::text("open"), None, 4.0);
        let panel: Element<'_, ()> = iced::widget::text("panel").into();
        let _: Element<'_, ()> = anchored(iced::widget::text("open"), Some(panel), 4.0);
    }
}
