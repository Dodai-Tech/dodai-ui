//! [`Scanlines`]: CRT overlay, drawn only when `effects.scanlines` is set.

use crate::theme_of;
use gpui::{
    canvas, div, fill, hsla, point, px, size, App, Bounds, IntoElement, RenderOnce, Styled, Window,
};

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
        canvas(
            |_, _, _| (),
            |b: Bounds<gpui::Pixels>, _, window, _| {
                let h: f32 = b.size.height.into();
                for y in scanline_ys(h, 3.0) {
                    window.paint_quad(fill(
                        Bounds::new(
                            point(b.origin.x, b.origin.y + px(y)),
                            size(b.size.width, px(1.0)),
                        ),
                        hsla(0.0, 0.0, 0.0, 0.18),
                    ));
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
