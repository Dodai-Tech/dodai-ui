//! The small token-styled boxes every toolkit draws the same way (cards, alerts, chips, code,
//! tooltips, rows, popups, ...): [`kit_box`] draws a [`dodai_ui_core::Kit`] around any content.

use crate::active_theme;
use crate::paint::{box_shapes_for, color32};
use dodai_ui_core::Kit;
use egui::{InnerResponse, Margin, Rect, Shape, Stroke, Ui};

/// Draw `content` inside a box of `kit` (fill, border, glow, padding from the theme tokens).
/// `clickable` boxes read their hover state from the previous frame's rectangle.
pub fn kit_box<R>(
    ui: &mut Ui,
    kit: Kit,
    clickable: bool,
    content: impl FnOnce(&mut Ui, egui::Color32) -> R,
) -> InnerResponse<R> {
    let t = active_theme(ui.ctx());
    let id = ui.next_auto_id();
    let slot = id.with("dodai-ui-egui/kit");
    let last: Option<Rect> = ui.ctx().data(|d| d.get_temp(slot));
    let hovered = clickable
        && last
            .zip(ui.input(|i| i.pointer.hover_pos()))
            .is_some_and(|(r, p)| r.contains(p));
    let style = t.kit(kit, hovered);
    let (px, py) = style.padding;
    let bg = ui.painter().add(Shape::Noop);
    let fg = color32(style.fg);
    let inner = egui::Frame::new()
        .inner_margin(Margin::symmetric(px.round() as i8, py.round() as i8))
        .show(ui, |ui| {
            ui.visuals_mut().override_text_color = Some(fg);
            ui.spacing_mut().interact_size.y = 0.0;
            content(ui, fg)
        });
    let rect = inner.response.rect;
    ui.ctx().data_mut(|d| d.insert_temp(slot, rect));
    let mut shapes = box_shapes_for(
        &t,
        rect,
        style.fill,
        style.border,
        style.glow,
        style.lift,
        kit.surface_class(),
    );
    if style.pill && t.shape.kind() != dodai_ui_core::ShapeKind::Chamfered {
        let r = rect.height() / 2.0;
        shapes = vec![Shape::rect_filled(
            rect,
            r,
            style.fill.map_or(egui::Color32::TRANSPARENT, color32),
        )];
        if let Some(b) = style.border {
            shapes.push(Shape::rect_stroke(
                rect,
                r,
                Stroke::new(t.shape.border_width, color32(b)),
                egui::StrokeKind::Inside,
            ));
        }
    }
    ui.painter().set(bg, Shape::Vec(shapes));
    inner
}
