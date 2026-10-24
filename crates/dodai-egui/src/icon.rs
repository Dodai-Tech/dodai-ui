//! Icons: any shared [`IconId`], drawn in the pack of the active design (Cyberpunk strokes,
//! Material Symbols, bold Phosphor).

use crate::{active_theme, color32};
use dodai_icons::{glyph_for, IconId};
use egui::{vec2, Color32, ColorImage, Response, Sense, TextureHandle, TextureOptions, Ui, Widget};

/// A vector icon, rasterised at the display scale and tinted with the text colour (or
/// [`Icon::color`]).
pub struct Icon {
    id: IconId,
    size: f32,
    color: Option<Color32>,
    filled: bool,
}

impl Icon {
    /// An icon of `id`, 16 px.
    pub fn new(id: IconId) -> Self {
        Self {
            id,
            size: 16.0,
            color: None,
            filled: false,
        }
    }
    /// Side in points.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Tint (default: the theme's text colour).
    pub fn color(mut self, c: Color32) -> Self {
        self.color = Some(c);
        self
    }
    /// The filled form where the pack has one.
    pub fn filled(mut self, f: bool) -> Self {
        self.filled = f;
        self
    }
}

impl Widget for Icon {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let (rect, resp) = ui.allocate_exact_size(vec2(self.size, self.size), Sense::hover());
        let px = (self.size * ui.ctx().pixels_per_point()).round().max(8.0) as u32;
        let set = t.look().icons;
        let key = egui::Id::unique(("dodai-egui/icon", self.id, set as u8, self.filled, px));
        let tex: Option<TextureHandle> = ui.ctx().data_mut(|d| d.get_temp(key));
        let tex = match tex {
            Some(t) => Some(t),
            None => glyph_for(self.id, set, self.filled).map(|g| {
                let cov = g.coverage(px);
                let rgba: Vec<u8> = cov.iter().flat_map(|&a| [255, 255, 255, a]).collect();
                let img = ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rgba);
                let h = ui
                    .ctx()
                    .load_texture(format!("icon-{key:?}"), img, TextureOptions::LINEAR);
                ui.ctx().data_mut(|d| d.insert_temp(key, h.clone()));
                h
            }),
        };
        if let (Some(tex), true) = (tex, ui.is_rect_visible(rect)) {
            let tint = self.color.unwrap_or_else(|| color32(t.palette.text));
            ui.painter().image(
                tex.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_design_draws_a_common_icon() {
        for t in dodai::themes::all() {
            assert!(
                glyph_for(IconId::Home, t.look().icons, false).is_some(),
                "{}",
                t.name()
            );
        }
    }
}
