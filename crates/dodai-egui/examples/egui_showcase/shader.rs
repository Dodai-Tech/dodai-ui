//! Shader nodes: one playing [`Live`] per distinct spec, drawn as an image.

use std::cell::RefCell;
use std::collections::HashMap;

use dodai_shaders::player::{Live, Poll};
use dodai_showcase_model::ShaderSpec;
use egui::{vec2, Ui};

struct Entry {
    live: Live,
    tex: Option<egui::TextureHandle>,
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Entry>> = RefCell::new(HashMap::new());
}

/// Draw the picture of `spec` (blank until its first frame is ready); keeps repainting while
/// the shader animates.
pub fn view(ui: &mut Ui, spec: &ShaderSpec, theme: &dodai_core::Theme) {
    let key = format!("{spec:?}{}", theme.name());
    let w = spec.width;
    let h = w / spec.aspect;
    let (rect, _) = ui.allocate_exact_size(vec2(w, h), egui::Sense::hover());
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        let e = c.entry(key.clone()).or_insert_with(|| Entry {
            live: Live::new(spec.to_spec(theme)),
            tex: None,
        });
        if let Poll::New(f) = e.live.tick(w, ui.ctx().pixels_per_point()) {
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [f.width as usize, f.height as usize],
                &f.rgba(),
            );
            e.tex = Some(
                ui.ctx()
                    .load_texture(key, img, egui::TextureOptions::LINEAR),
            );
        }
        if let Some(t) = &e.tex {
            ui.painter().image(
                t.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
        if let Some(d) = e.live.next_in() {
            ui.ctx().request_repaint_after(d);
        }
    });
}
