//! Tag / badge.

use crate::paint::{add_all, box_shapes_for, color32, layout};
use crate::{active_theme, display_text};
use dodai_core::Tone;
use egui::{vec2, Align2, Response, Sense, Ui, Widget};

/// Status tag: solid or outlined in a [`Tone`].
pub struct Tag {
    text: String,
    tone: Tone,
    solid: bool,
}

impl Tag {
    /// Info-toned outlined tag.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: Tone::Info,
            solid: false,
        }
    }
    /// Set tone.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    /// Solid fill instead of outline.
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
}

impl Widget for Tag {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let g = layout(ui, display_text(&t, &self.text), t.typography.size_xs);
        let size = vec2(g.size().x + 2.0 * t.spacing.sm, g.size().y + t.spacing.sm);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            let tok = t.tag(self.tone, self.solid);
            add_all(
                ui,
                box_shapes_for(
                    &t,
                    rect,
                    tok.background,
                    tok.border,
                    false,
                    false,
                    Some(dodai_core::SurfaceClass::Tag),
                ),
            );
            let pos = Align2::CENTER_CENTER
                .align_size_within_rect(g.size(), rect)
                .min;
            ui.painter().galley(pos, g, color32(tok.foreground));
        }
        resp
    }
}
