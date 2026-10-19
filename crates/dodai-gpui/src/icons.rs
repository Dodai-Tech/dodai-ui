//! Vector [`Icon`]s drawn with gpui paths (no asset files, any colour/size).

use crate::paint::stroke;
pub use dodai_cyberpunk_icons::IconName;
use gpui::{canvas, div, px, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled};

/// Stroked vector icon. Defaults: 16 px, white-ish — set `.color(..)`.
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: f32,
    color: Hsla,
    weight: f32,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: 16.0,
            color: gpui::white(),
            weight: 1.75,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = c;
        self
    }
    /// Stroke width in px at 24 px size (scales with `size`).
    pub fn weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        let (size, color, weight) = (self.size, self.color, self.weight);
        let shapes = self.name.shapes();
        let k = size / 24.0;
        div().size(px(size)).flex_none().child(
            canvas(
                |_, _, _| (),
                move |b: Bounds<Pixels>, _, w, _| {
                    for (p, closed) in &shapes {
                        let p: Vec<(f32, f32)> = p.iter().map(|&(x, y)| (x * k, y * k)).collect();
                        stroke(w, b.origin, &p, (weight * k).max(1.0), color, *closed);
                    }
                },
            )
            .size_full(),
        )
    }
}
