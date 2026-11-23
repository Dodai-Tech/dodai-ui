//! Tabs.

use crate::paint::{add_all, box_shapes, color32, layout};
use crate::{active_theme, display_text};
use egui::{pos2, vec2, Align2, Rect, Response, Sense, Stroke, Ui, Widget};

/// Horizontal tab strip bound to a selected index.
pub struct Tabs<'a> {
    selected: &'a mut usize,
    labels: Vec<String>,
    closed: Option<&'a mut Option<usize>>,
    added: Option<&'a mut bool>,
}

impl<'a> Tabs<'a> {
    /// Tabs with `labels`; clicking writes the index to `selected`.
    pub fn new<S: Into<String>>(
        selected: &'a mut usize,
        labels: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            selected,
            labels: labels.into_iter().map(Into::into).collect(),
            closed: None,
            added: None,
        }
    }

    /// Give every tab a close cross. A click writes the tab's index to `closed`; the app then
    /// removes the label (the strip never drops tabs itself).
    pub fn closable(mut self, closed: &'a mut Option<usize>) -> Self {
        self.closed = Some(closed);
        self
    }

    /// Show a "+" after the last tab; a click sets `added` to `true` for the app to add a tab.
    pub fn addable(mut self, added: &'a mut bool) -> Self {
        self.added = Some(added);
        self
    }
}

impl Widget for Tabs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let mut changed = None;
        let mut ids = Vec::new();
        let mut focused = None;
        let mut close_hit = None;
        let mut add_hit = false;
        let close_d = t.sizes().icon_sm;
        let closable = self.closed.is_some();
        let addable = self.added.is_some();
        let mut resp = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = t.spacing.xs;
                for (i, label) in self.labels.iter().enumerate() {
                    let g = layout(ui, display_text(&t, label), t.typography.size_md);
                    let extra = if closable {
                        close_d + t.spacing.sm
                    } else {
                        0.0
                    };
                    let size = vec2(
                        g.size().x + 2.0 * t.spacing.md + extra,
                        t.shape.control_height,
                    );
                    let (rect, r) = ui.allocate_exact_size(size, Sense::click());
                    ids.push(r.id);
                    if r.has_focus() {
                        focused = Some(i);
                    }
                    let x_rect = Rect::from_center_size(
                        pos2(rect.max.x - t.spacing.md - close_d / 2.0, rect.center().y),
                        vec2(close_d, close_d),
                    );
                    let x_hot = closable
                        && ui
                            .input(|i| i.pointer.hover_pos())
                            .is_some_and(|p| x_rect.contains(p));
                    if r.clicked() {
                        if x_hot {
                            close_hit = Some(i);
                        } else {
                            changed = Some(i);
                        }
                    }
                    if ui.is_rect_visible(rect) {
                        let active = changed.unwrap_or(*self.selected) == i;
                        let tok = t.tab(dodai_core::ItemState::of(active, r.hovered()));
                        add_all(ui, box_shapes(&t, rect, tok.background, None, false, false));
                        if let Some(c) = tok.indicator {
                            let bar =
                                Rect::from_min_max(pos2(rect.min.x, rect.max.y - 2.0), rect.max);
                            ui.painter().line_segment(
                                [bar.left_center(), bar.right_center()],
                                Stroke::new(2.0_f32, color32(c)),
                            );
                        }
                        let label_rect =
                            Rect::from_min_max(rect.min, pos2(rect.max.x - extra, rect.max.y));
                        let pos = Align2::CENTER_CENTER
                            .align_size_within_rect(g.size(), label_rect)
                            .min;
                        ui.painter().galley(pos, g, color32(tok.foreground));
                        if closable {
                            let c = color32(tok.foreground).gamma_multiply(if x_hot {
                                1.0
                            } else {
                                0.7
                            });
                            let k = close_d * 0.3;
                            let s = Stroke::new(1.5_f32, c);
                            let m = x_rect.center();
                            ui.painter()
                                .line_segment([m + vec2(-k, -k), m + vec2(k, k)], s);
                            ui.painter()
                                .line_segment([m + vec2(-k, k), m + vec2(k, -k)], s);
                        }
                    }
                }
                if addable {
                    let side = t.shape.control_height;
                    let (rect, r) = ui.allocate_exact_size(vec2(side, side), Sense::click());
                    add_hit = r.clicked();
                    if ui.is_rect_visible(rect) {
                        let c = color32(t.palette.text).gamma_multiply(if r.hovered() {
                            1.0
                        } else {
                            0.7
                        });
                        let k = close_d * 0.35;
                        let s = Stroke::new(1.5_f32, c);
                        let m = rect.center();
                        ui.painter()
                            .line_segment([m + vec2(-k, 0.0), m + vec2(k, 0.0)], s);
                        ui.painter()
                            .line_segment([m + vec2(0.0, -k), m + vec2(0.0, k)], s);
                    }
                }
            })
            .response;
        if let (Some(i), Some(out)) = (close_hit, self.closed) {
            *out = Some(i);
            resp.mark_changed();
        }
        if let (true, Some(out)) = (add_hit, self.added) {
            *out = true;
            resp.mark_changed();
        }
        // Roving focus: the arrows, Home and End move to the next tab and select it.
        if let Some(at) = focused {
            let key = ui.input(|i| {
                [
                    (egui::Key::ArrowLeft, "left"),
                    (egui::Key::ArrowRight, "right"),
                    (egui::Key::Home, "home"),
                    (egui::Key::End, "end"),
                ]
                .into_iter()
                .find(|(k, _)| i.key_pressed(*k))
                .map(|(_, n)| n)
            });
            if let Some(to) = key.and_then(|k| {
                dodai_core::roving::next(
                    k,
                    dodai_core::roving::Axis::Horizontal,
                    at,
                    ids.len(),
                    |_| true,
                )
            }) {
                ui.memory_mut(|m| m.request_focus(ids[to]));
                changed = Some(to);
            }
        }
        if let Some(i) = changed {
            if *self.selected != i {
                *self.selected = i;
                resp.mark_changed();
            }
        }
        resp
    }
}
