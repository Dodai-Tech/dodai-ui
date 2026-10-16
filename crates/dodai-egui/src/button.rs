//! Button.

use crate::paint::{add_all, box_shapes, color32, control_state, layout, sense};
use crate::{active_theme, display_text};
use dodai_core::{ButtonVariant, ControlState};
use egui::{vec2, Align2, Response, Ui, Widget};

/// Themed button: every [`ButtonVariant`], optionally disabled.
pub struct Button {
    text: String,
    variant: ButtonVariant,
    enabled: bool,
    min_width: f32,
}

impl Button {
    /// Primary button.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: ButtonVariant::Primary,
            enabled: true,
            min_width: 0.0,
        }
    }
    /// Choose the emphasis.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Enable/disable.
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
    /// Minimum width.
    pub fn min_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let g = layout(ui, display_text(&t, &self.text), t.typography.size_md);
        let size = vec2(
            (g.size().x + 2.0 * t.spacing.lg).max(self.min_width),
            t.shape.control_height,
        );
        let (rect, resp) = ui.allocate_exact_size(size, sense(self.enabled));
        let state = control_state(&resp, self.enabled);
        if self.enabled {
            resp.clone().on_hover_cursor(egui::CursorIcon::PointingHand);
        }
        if ui.is_rect_visible(rect) {
            let tok = t.button(self.variant, state);
            let glow = self.enabled
                && (state != ControlState::Idle || self.variant == ButtonVariant::Primary);
            add_all(
                ui,
                box_shapes(&t, rect, tok.background, tok.border, glow, false),
            );
            let pos = Align2::CENTER_CENTER
                .align_size_within_rect(g.size(), rect)
                .min;
            ui.painter().galley(pos, g, color32(tok.foreground));
        }
        resp
    }
}
