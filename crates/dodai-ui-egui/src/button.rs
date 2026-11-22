//! Button.

use crate::paint::{add_all, box_shapes_for, color32, control_state, layout, sense};
use crate::{active_theme, display_text};
use dodai_ui_core::{ButtonVariant, ControlSize, ControlState};
use egui::{vec2, Align2, Response, Ui, Widget};

/// Themed button: every [`ButtonVariant`], optionally disabled.
pub struct Button {
    text: String,
    variant: ButtonVariant,
    enabled: bool,
    min_width: f32,
    size: ControlSize,
    #[cfg(feature = "icons")]
    icon: Option<dodai_ui_icons::IconId>,
}

impl Button {
    /// Primary button.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: ButtonVariant::Primary,
            enabled: true,
            min_width: 0.0,
            size: ControlSize::Medium,
            #[cfg(feature = "icons")]
            icon: None,
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
    /// Small, medium or large: the design's height, padding and text size.
    pub fn size(mut self, size: ControlSize) -> Self {
        self.size = size;
        self
    }
    /// An icon before the label; alone (empty label) the button is square.
    #[cfg(feature = "icons")]
    pub fn icon(mut self, icon: dodai_ui_icons::IconId) -> Self {
        self.icon = Some(icon);
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
        let m = t.control_metrics(self.size);
        let g = layout(ui, display_text(&t, &self.text), m.text_size);
        #[cfg(feature = "icons")]
        let lead = self.icon.map(|_| {
            m.icon_size
                + if self.text.is_empty() {
                    0.0
                } else {
                    t.spacing.sm
                }
        });
        #[cfg(not(feature = "icons"))]
        let lead: Option<f32> = None;
        let label_w = if self.text.is_empty() {
            0.0
        } else {
            g.size().x
        };
        let width = match (lead, self.text.is_empty()) {
            (Some(_), true) => m.height,
            (Some(l), false) => label_w + l + 2.0 * m.padding,
            (None, _) => label_w + 2.0 * m.padding,
        };
        let size = vec2(width.max(self.min_width), m.height);
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
                box_shapes_for(
                    &t,
                    rect,
                    tok.background,
                    tok.border,
                    glow,
                    false,
                    Some(dodai_ui_core::SurfaceClass::Button),
                ),
            );
            let content = vec2(label_w + lead.unwrap_or(0.0), g.size().y.max(m.icon_size));
            let left = Align2::CENTER_CENTER
                .align_size_within_rect(content, rect)
                .min;
            #[cfg(feature = "icons")]
            if let Some(icon) = self.icon {
                let r = egui::Rect::from_min_size(
                    egui::pos2(left.x, rect.center().y - m.icon_size / 2.0),
                    vec2(m.icon_size, m.icon_size),
                );
                crate::icon::paint(ui, r, icon, false, color32(tok.foreground));
            }
            if label_w > 0.0 {
                let x = left.x + lead.unwrap_or(0.0);
                ui.painter().galley(
                    egui::pos2(x, rect.center().y - g.size().y / 2.0),
                    g,
                    color32(tok.foreground),
                );
            }
        }
        resp
    }
}
