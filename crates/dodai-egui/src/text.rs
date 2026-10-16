//! Text helpers honouring `uppercase_labels`, sizes and colours from tokens.

use crate::{active_theme, paint::color32};
use dodai_core::Theme;
use egui::{RichText, Ui};

/// Label text as the theme wants it (UPPERCASE when `uppercase_labels`).
pub fn display_text(t: &Theme, s: &str) -> String {
    if t.typography.uppercase_labels {
        s.to_uppercase()
    } else {
        s.to_owned()
    }
}

/// Heading rich text.
pub fn heading_text(t: &Theme, s: &str) -> RichText {
    RichText::new(display_text(t, s))
        .size(t.typography.size_xl)
        .strong()
        .color(color32(t.palette.text))
}

/// Body label rich text.
pub fn label_text(t: &Theme, s: &str) -> RichText {
    RichText::new(display_text(t, s))
        .size(t.typography.size_md)
        .color(color32(t.palette.text))
}

/// Muted secondary text (case untouched).
pub fn muted_text(t: &Theme, s: &str) -> RichText {
    RichText::new(s)
        .size(t.typography.size_sm)
        .color(color32(t.palette.text_muted))
}

/// Monospace text (case untouched).
pub fn mono_text(t: &Theme, s: &str) -> RichText {
    RichText::new(s)
        .monospace()
        .size(t.typography.size_sm)
        .color(color32(t.palette.text))
}

/// Themed text on [`Ui`].
pub trait DodaiUi {
    /// Heading.
    fn dodai_heading(&mut self, s: &str) -> egui::Response;
    /// Body label.
    fn dodai_label(&mut self, s: &str) -> egui::Response;
    /// Muted text.
    fn dodai_muted(&mut self, s: &str) -> egui::Response;
    /// Monospace text.
    fn dodai_mono(&mut self, s: &str) -> egui::Response;
}

impl DodaiUi for Ui {
    fn dodai_heading(&mut self, s: &str) -> egui::Response {
        let t = active_theme(self.ctx());
        self.label(heading_text(&t, s))
    }
    fn dodai_label(&mut self, s: &str) -> egui::Response {
        let t = active_theme(self.ctx());
        self.label(label_text(&t, s))
    }
    fn dodai_muted(&mut self, s: &str) -> egui::Response {
        let t = active_theme(self.ctx());
        self.label(muted_text(&t, s))
    }
    fn dodai_mono(&mut self, s: &str) -> egui::Response {
        let t = active_theme(self.ctx());
        self.label(mono_text(&t, s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{rounded, square};

    #[test]
    fn uppercase_from_token() {
        assert_eq!(display_text(&rounded(), "Jack in"), "Jack in");
        assert_eq!(display_text(&square(), "Jack in"), "JACK IN");
    }

    #[test]
    fn sizes_from_tokens() {
        let t = rounded();
        assert_eq!(heading_text(&t, "x").text(), "x");
        assert_eq!(mono_text(&t, "Ab").text(), "Ab");
    }
}
