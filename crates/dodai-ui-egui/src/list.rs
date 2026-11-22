//! List item and divider.

use crate::paint::{color32, layout};
use crate::{active_theme, display_text};
use egui::{pos2, vec2, Rect, Response, Sense, Ui, Widget};

/// Selectable row. Click to select; `selected` is drawn with the marker bar.
pub struct ListItem {
    text: String,
    selected: bool,
}

impl ListItem {
    /// Unselected row.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            selected: false,
        }
    }
    /// Mark selected.
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
}

impl Widget for ListItem {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let size = vec2(ui.available_width(), t.shape.control_height);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        if ui.is_rect_visible(rect) {
            let tok = t.list_item(dodai_ui_core::ItemState::of(self.selected, resp.hovered()));
            if let Some(bg) = tok.background {
                // Rows are flat strips: plain rect, not the chamfered control shape.
                ui.painter().rect_filled(rect, 0.0, color32(bg));
            }
            if let Some(m) = tok.marker {
                let bar = Rect::from_min_size(rect.min, vec2(3.0, rect.height()));
                ui.painter().rect_filled(bar, 0.0, color32(m));
            }
            let g = layout(ui, display_text(&t, &self.text), t.typography.size_md);
            let pos = pos2(
                rect.min.x + t.spacing.md,
                rect.center().y - g.size().y / 2.0,
            );
            ui.painter().galley(pos, g, color32(tok.foreground));
        }
        resp.on_hover_cursor(egui::CursorIcon::PointingHand)
    }
}

/// Horizontal divider line.
#[derive(Default)]
pub struct Divider;

impl Widget for Divider {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let (rect, resp) = ui.allocate_exact_size(
            vec2(ui.available_width(), t.shape.border_width.max(1.0)),
            Sense::hover(),
        );
        ui.painter().rect_filled(rect, 0.0, color32(t.divider()));
        resp
    }
}
