//! [`Scanlines`]: CRT overlay, drawn only when `effects.scanlines` is set.

use crate::theme_of;
use gpui::{canvas, div, point, px, App, Bounds, IntoElement, RenderOnce, Styled, Window};

/// Y offsets (px) of scanlines in a box of `height`, every `pitch` px.
pub fn scanline_ys(height: f32, pitch: f32) -> Vec<f32> {
    if pitch <= 0.0 || height <= 0.0 {
        return Vec::new();
    }
    let mut v = Vec::new();
    let mut y = 0.0;
    while y < height {
        v.push(y);
        y += pitch;
    }
    v
}

/// Absolutely-positioned overlay filling its (relative) parent. Place it as the
/// last child of the root. It adds no hitbox, so it never blocks input.
#[derive(IntoElement, Default)]
pub struct Scanlines;

impl Scanlines {
    /// Create a new [`Scanlines`].
    pub fn new() -> Self {
        Self
    }
}

impl RenderOnce for Scanlines {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        if !t.effects.scanlines {
            return div().into_any_element();
        }
        let line = crate::hsla(t.effects.scanline);
        canvas(
            |_, _, _| (),
            move |b: Bounds<gpui::Pixels>, _, window, _| {
                let h: f32 = b.size.height.into();
                let w: f32 = b.size.width.into();
                // All lines in one path: one tessellation + one draw instead of
                // a quad per line.
                let mut pb = gpui::PathBuilder::fill();
                for y in scanline_ys(h, 3.0) {
                    let (x0, y0) = (b.origin.x, b.origin.y + px(y));
                    pb.add_polygon(
                        &[
                            point(x0, y0),
                            point(x0 + px(w), y0),
                            point(x0 + px(w), y0 + px(1.0)),
                            point(x0, y0 + px(1.0)),
                        ],
                        true,
                    );
                }
                if let Ok(path) = pb.build() {
                    window.paint_path(path, line);
                }
            },
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ys() {
        assert_eq!(scanline_ys(10.0, 3.0), vec![0.0, 3.0, 6.0, 9.0]);
        assert!(scanline_ys(10.0, 0.0).is_empty());
    }
}
