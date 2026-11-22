//! [`PageGrid`]: the page grid of the active design (columns, gap and outer margins at the
//! viewport width). Compiled with the `dodai` feature, which is the design that defines one.

use crate::theme_of;
use gpui::{div, px, AnyElement, App, IntoElement, ParentElement, Styled, Window};

/// Width in px of a cell spanning
/// `span` of `columns` columns inside `inner` px with `gap` between columns.
fn grid_span_width(inner: f32, columns: u32, gap: f32, span: u32) -> f32 {
    let columns = columns.max(1);
    let span = span.clamp(1, columns);
    let col = (inner - gap * (columns - 1) as f32) / columns as f32;
    col * span as f32 + gap * (span - 1) as f32
}

/// The design's page grid (columns, gap and outer margins at the viewport width) laid out as
/// wrapping rows of cells that each span a number of columns.
///
/// The width is the window's unless set with [`PageGrid::width`] (use it when the page sits
/// beside a sidebar).
#[derive(IntoElement)]
pub struct PageGrid {
    width: Option<f32>,
    cells: Vec<(u32, AnyElement)>,
}

impl Default for PageGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl PageGrid {
    /// Create a new [`PageGrid`].
    pub fn new() -> Self {
        Self {
            width: None,
            cells: Vec::new(),
        }
    }
    /// The width the grid lays out in, instead of the window's.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    /// A cell spanning `span` columns (clamped to the grid's column count).
    pub fn cell(mut self, span: u32, el: impl IntoElement) -> Self {
        self.cells.push((span, el.into_any_element()));
        self
    }
}

impl gpui::RenderOnce for PageGrid {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let viewport = self
            .width
            .unwrap_or_else(|| window.viewport_size().width.into());
        let g = t.page_grid(viewport);
        let page = g.max_width.map_or(viewport, |m| viewport.min(m));
        let inner = (page - 2.0 * g.margin).max(0.0);
        let mut row = div()
            .flex()
            .flex_wrap()
            .gap(px(g.gap))
            .w(px(inner))
            .mx_auto();
        for (span, el) in self.cells {
            row = row.child(
                div()
                    .w(px(grid_span_width(inner, g.columns, g.gap, span)))
                    .child(el),
            );
        }
        row
    }
}

#[cfg(test)]
mod grid_tests {
    use super::grid_span_width;

    #[test]
    fn spans_add_up_to_the_row() {
        let (inner, cols, gap) = (1000.0, 12, 20.0);
        let half = grid_span_width(inner, cols, gap, 6);
        assert!((half * 2.0 + gap - inner).abs() < 0.01);
        assert!((grid_span_width(inner, cols, gap, 12) - inner).abs() < 0.01);
        assert_eq!(
            grid_span_width(inner, cols, gap, 99),
            grid_span_width(inner, cols, gap, 12)
        );
    }
}
