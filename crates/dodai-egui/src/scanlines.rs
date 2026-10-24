//! Optional CRT scanline overlay.

use crate::active_theme;
use egui::{pos2, Color32, Context, LayerId, Order, Rect, Stroke};

/// Y positions of scanlines within `rect`, one every `pitch` px.
pub fn scanline_ys(rect: Rect, pitch: f32) -> Vec<f32> {
    if pitch <= 0.0 {
        return Vec::new();
    }
    let n = (rect.height() / pitch).floor() as usize;
    (0..=n).map(|i| rect.min.y + i as f32 * pitch).collect()
}

/// Paint scanlines over the whole screen when `Effects::scanlines` is on.
/// Call last in the frame.
pub fn paint_scanlines(ctx: &Context) {
    let t = active_theme(ctx);
    if !t.effects.scanlines {
        return;
    }
    let screen = ctx.content_rect();
    let p = ctx.layer_painter(LayerId::new(
        Order::Foreground,
        egui::Id::unique("dodai-scanlines"),
    ));
    let s = Stroke::new(1.0_f32, Color32::from_black_alpha(36));
    for y in scanline_ys(screen, 3.0) {
        p.line_segment([pos2(screen.min.x, y), pos2(screen.max.x, y)], s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanline_spacing() {
        let r = Rect::from_min_max(pos2(0.0, 0.0), pos2(10.0, 9.0));
        assert_eq!(scanline_ys(r, 3.0), vec![0.0, 3.0, 6.0, 9.0]);
        assert!(scanline_ys(r, 0.0).is_empty());
    }
}
