//! Toggle, checkbox, radio, progress bar and slider.

use crate::paint::{add_all, box_shapes, color32, control_state, layout, sense};
use crate::{active_theme, display_text};
use egui::{pos2, vec2, Rect, Response, Sense, Stroke, Ui, Widget};
use dodai_core::{ControlState, ShapeKind, Theme, Tone};
use std::ops::RangeInclusive;

/// Side of checkbox/radio marks.
fn mark_size(t: &Theme) -> f32 {
    (t.typography.size_md * 1.3).round()
}

/// Label to the right of a mark; returns the combined response.
fn labelled(
    ui: &mut Ui,
    t: &Theme,
    label: &str,
    enabled: bool,
    draw_mark: impl FnOnce(&Ui, Rect, ControlState),
) -> Response {
    let s = mark_size(t);
    let g = layout(ui, display_text(t, label), t.typography.size_md);
    let size = vec2(
        s + t.spacing.sm + g.size().x,
        s.max(g.size().y).max(t.shape.control_height * 0.6),
    );
    let (rect, resp) = ui.allocate_exact_size(size, sense(enabled));
    if ui.is_rect_visible(rect) {
        let state = control_state(&resp, enabled);
        let mark = Rect::from_min_size(pos2(rect.min.x, rect.center().y - s / 2.0), vec2(s, s));
        draw_mark(ui, mark, state);
        let col = if enabled {
            t.palette.text
        } else {
            t.palette.text_disabled
        };
        let pos = pos2(
            mark.max.x + t.spacing.sm,
            rect.center().y - g.size().y / 2.0,
        );
        ui.painter().galley(pos, g, color32(col));
    }
    resp
}

/// Switch (track + thumb) bound to a `bool`.
pub struct Toggle<'a> {
    on: &'a mut bool,
    enabled: bool,
}

impl<'a> Toggle<'a> {
    /// New switch.
    pub fn new(on: &'a mut bool) -> Self {
        Self { on, enabled: true }
    }
    /// Enable/disable.
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let h = (t.typography.size_md * 1.5).round();
        let (rect, mut resp) = ui.allocate_exact_size(vec2(h * 1.9, h), sense(self.enabled));
        if resp.clicked() {
            *self.on = !*self.on;
            resp.mark_changed();
        }
        if ui.is_rect_visible(rect) {
            let state = control_state(&resp, self.enabled);
            let tok = t.toggle(*self.on, state);
            add_all(
                ui,
                box_shapes(
                    &t,
                    rect,
                    Some(tok.track),
                    Some(tok.border),
                    *self.on && self.enabled,
                    false,
                ),
            );
            let inset = (t.shape.border_width + 2.0).max(3.0);
            let d = h - 2.0 * inset;
            let x = if *self.on {
                rect.max.x - inset - d
            } else {
                rect.min.x + inset
            };
            let thumb = Rect::from_min_size(pos2(x, rect.min.y + inset), vec2(d, d));
            add_all(
                ui,
                box_shapes(&t, thumb, Some(tok.thumb), None, false, false),
            );
        }
        resp
    }
}

fn check_mark(ui: &Ui, r: Rect, col: egui::Color32) {
    let p = |x: f32, y: f32| pos2(r.min.x + r.width() * x, r.min.y + r.height() * y);
    ui.painter().add(egui::Shape::line(
        vec![p(0.24, 0.52), p(0.43, 0.72), p(0.78, 0.30)],
        Stroke::new(2.0_f32, col),
    ));
}

/// Checkbox with label.
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    label: String,
    enabled: bool,
}

impl<'a> Checkbox<'a> {
    /// New checkbox.
    pub fn new(checked: &'a mut bool, label: impl Into<String>) -> Self {
        Self {
            checked,
            label: label.into(),
            enabled: true,
        }
    }
    /// Enable/disable.
    pub fn enabled(mut self, e: bool) -> Self {
        self.enabled = e;
        self
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let on = *self.checked;
        let enabled = self.enabled;
        let mut resp = labelled(ui, &t, &self.label, enabled, |ui, r, state| {
            let tok = t.toggle(on, state);
            let fill = if on {
                tok.track
            } else {
                t.palette.surface_raised
            };
            add_all(
                ui,
                box_shapes(&t, r, Some(fill), Some(tok.border), on && enabled, false),
            );
            if on {
                check_mark(ui, r, color32(tok.thumb));
            }
        });
        if resp.clicked() {
            *self.checked = !on;
            resp.mark_changed();
        }
        resp
    }
}

/// Radio option: selected when `*value == option`.
pub struct Radio<'a, T: PartialEq + Clone> {
    value: &'a mut T,
    option: T,
    label: String,
}

impl<'a, T: PartialEq + Clone> Radio<'a, T> {
    /// Radio for `option`; clicking assigns it to `value`.
    pub fn new(value: &'a mut T, option: T, label: impl Into<String>) -> Self {
        Self {
            value,
            option,
            label: label.into(),
        }
    }
}

impl<T: PartialEq + Clone> Widget for Radio<'_, T> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let on = *self.value == self.option;
        let mut resp = labelled(ui, &t, &self.label, true, |ui, r, state| {
            let tok = t.toggle(on, state);
            let stroke_w = t.shape.border_width;
            if t.shape.kind() == ShapeKind::Rounded {
                let c = r.center();
                ui.painter().circle(
                    c,
                    r.width() / 2.0,
                    color32(t.palette.surface_raised),
                    Stroke::new(stroke_w, color32(tok.border)),
                );
                if on {
                    ui.painter()
                        .circle_filled(c, r.width() * 0.25, color32(tok.track));
                }
            } else {
                add_all(
                    ui,
                    box_shapes(
                        &t,
                        r,
                        Some(t.palette.surface_raised),
                        Some(tok.border),
                        on,
                        false,
                    ),
                );
                if on {
                    let dot = r.shrink(r.width() * 0.27);
                    add_all(ui, box_shapes(&t, dot, Some(tok.track), None, false, false));
                }
            }
        });
        if resp.clicked() && !on {
            *self.value = self.option;
            resp.mark_changed();
        }
        resp
    }
}

/// Fraction → clipped fill rect inside `rect`.
pub(crate) fn fill_rect(rect: Rect, fraction: f32) -> Rect {
    let f = fraction.clamp(0.0, 1.0);
    Rect::from_min_max(rect.min, pos2(rect.min.x + rect.width() * f, rect.max.y))
}

/// Progress bar (`0.0..=1.0`).
pub struct ProgressBar {
    fraction: f32,
    tone: Option<Tone>,
    width: Option<f32>,
}

impl ProgressBar {
    /// Bar at `fraction`.
    pub fn new(fraction: f32) -> Self {
        Self {
            fraction,
            tone: None,
            width: None,
        }
    }
    /// Status tone (default brand colour).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
    /// Fixed width (default: available width).
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
}

impl Widget for ProgressBar {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let w = self.width.unwrap_or_else(|| ui.available_width().max(60.0));
        let (rect, resp) = ui.allocate_exact_size(vec2(w, t.spacing.md.max(8.0)), Sense::hover());
        if ui.is_rect_visible(rect) {
            let tok = t.progress(self.tone);
            let f = self.fraction.clamp(0.0, 1.0);
            add_all(
                ui,
                box_shapes(&t, rect, Some(tok.track), Some(tok.border), false, false),
            );
            if f > 0.0 {
                // Re-draw the same outline in the fill colour, clipped to the fraction,
                // so chamfered/rounded ends stay exact.
                let p = ui.painter().with_clip_rect(fill_rect(rect, f));
                p.extend(box_shapes(
                    &t,
                    rect,
                    Some(tok.fill),
                    Some(tok.border),
                    true,
                    false,
                ));
            }
        }
        resp
    }
}

/// Horizontal slider.
pub struct Slider<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    width: f32,
}

impl<'a> Slider<'a> {
    /// Slider over `range`.
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        Self {
            value,
            range,
            width: 200.0,
        }
    }
    /// Track width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

/// Value ↔ x helper (clamped).
pub(crate) fn value_from_x(range: &RangeInclusive<f32>, x0: f32, x1: f32, x: f32) -> f32 {
    let f = ((x - x0) / (x1 - x0).max(f32::EPSILON)).clamp(0.0, 1.0);
    range.start() + (range.end() - range.start()) * f
}

impl Widget for Slider<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let (rect, mut resp) = ui.allocate_exact_size(
            vec2(self.width, t.shape.control_height),
            Sense::click_and_drag(),
        );
        let thumb_w = (t.typography.size_md * 0.8).round();
        let (x0, x1) = (rect.min.x + thumb_w / 2.0, rect.max.x - thumb_w / 2.0);
        if let (true, Some(p)) = (
            resp.dragged() || resp.clicked(),
            resp.interact_pointer_pos(),
        ) {
            let v = value_from_x(&self.range, x0, x1, p.x);
            if v != *self.value {
                *self.value = v;
                resp.mark_changed();
            }
        }
        if ui.is_rect_visible(rect) {
            let span = (self.range.end() - self.range.start()).max(f32::EPSILON);
            let f = ((*self.value - self.range.start()) / span).clamp(0.0, 1.0);
            let cx = x0 + (x1 - x0) * f;
            let track = Rect::from_center_size(rect.center(), vec2(rect.width(), 6.0));
            let tok = t.progress(None);
            add_all(
                ui,
                box_shapes(&t, track, Some(tok.track), Some(tok.border), false, false),
            );
            let fill = Rect::from_min_max(track.min, pos2(cx, track.max.y));
            ui.painter().with_clip_rect(fill).extend(box_shapes(
                &t,
                track,
                Some(tok.fill),
                None,
                false,
                false,
            ));
            let state = control_state(&resp, true);
            let th = t.toggle(true, state);
            let thumb = Rect::from_center_size(
                pos2(cx, rect.center().y),
                vec2(thumb_w, rect.height() * 0.6),
            );
            add_all(
                ui,
                box_shapes(
                    &t,
                    thumb,
                    Some(th.track),
                    Some(th.border),
                    state != ControlState::Idle,
                    false,
                ),
            );
        }
        resp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_rect_fraction() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 10.0));
        assert_eq!(fill_rect(r, 0.25).width(), 25.0);
        assert_eq!(fill_rect(r, 2.0).width(), 100.0);
        assert_eq!(fill_rect(r, -1.0).width(), 0.0);
    }

    #[test]
    fn slider_value_mapping() {
        let r = 0.0..=10.0;
        assert_eq!(value_from_x(&r, 0.0, 100.0, 50.0), 5.0);
        assert_eq!(value_from_x(&r, 0.0, 100.0, -5.0), 0.0);
        assert_eq!(value_from_x(&r, 0.0, 100.0, 500.0), 10.0);
    }

    #[test]
    fn mark_scales_with_type() {
        let t = crate::testing::rounded();
        assert_eq!(mark_size(&t), (15.0f32 * 1.3).round());
    }
}
