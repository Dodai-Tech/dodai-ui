//! Tabs.

use crate::paint::{add_all, box_shapes, color32, layout};
use crate::{active_theme, display_text};
use egui::{pos2, vec2, Align2, Rect, Response, Sense, Stroke, Ui, Widget};

/// Horizontal tab strip bound to a selected index.
pub struct Tabs<'a> {
    selected: &'a mut usize,
    labels: Vec<String>,
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
        }
    }
}

impl Widget for Tabs<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let mut changed = None;
        let mut resp = ui
            .horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = t.spacing.xs;
                for (i, label) in self.labels.iter().enumerate() {
                    let g = layout(ui, display_text(&t, label), t.typography.size_md);
                    let size = vec2(g.size().x + 2.0 * t.spacing.md, t.shape.control_height);
                    let (rect, r) = ui.allocate_exact_size(size, Sense::click());
                    if r.clicked() {
                        changed = Some(i);
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
                        let pos = Align2::CENTER_CENTER
                            .align_size_within_rect(g.size(), rect)
                            .min;
                        ui.painter().galley(pos, g, color32(tok.foreground));
                    }
                }
            })
            .response;
        if let Some(i) = changed {
            if *self.selected != i {
                *self.selected = i;
                resp.mark_changed();
            }
        }
        resp
    }
}
