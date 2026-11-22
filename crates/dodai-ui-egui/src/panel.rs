//! Panel / card, text input and tooltip.

use crate::paint::{box_shapes_for, color32};
use crate::{active_theme, display_text};
use egui::{vec2, Frame, InnerResponse, Margin, Response, RichText, Shape, TextEdit, Ui, Widget};

/// Bordered card with optional title; glows when focused.
#[derive(Default)]
pub struct Panel {
    title: Option<String>,
    focused: bool,
}

impl Panel {
    /// Untitled panel.
    pub fn new() -> Self {
        Self::default()
    }
    /// Title line (uppercase per theme).
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// Highlight border + glow.
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    /// Draw the panel around `add`.
    pub fn show<R>(self, ui: &mut Ui, add: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let t = active_theme(ui.ctx());
        let tok = t.panel(self.focused);
        let idx = ui.painter().add(Shape::Noop);
        let inner = Frame::NONE
            .inner_margin(Margin::same(t.spacing.md.round() as i8))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                if let Some(title) = &self.title {
                    ui.label(
                        RichText::new(display_text(&t, title))
                            .size(t.typography.size_sm)
                            .strong()
                            .color(color32(tok.title)),
                    );
                    ui.add_space(t.spacing.xs);
                }
                add(ui)
            });
        ui.painter().set(
            idx,
            Shape::Vec(box_shapes_for(
                &t,
                inner.response.rect,
                Some(tok.background),
                Some(tok.border),
                self.focused,
                true,
                Some(dodai_ui_core::SurfaceClass::Panel),
            )),
        );
        if crate::surface::take_animated() {
            ui.ctx().request_repaint();
        }
        inner
    }
}

/// Single-line text input in a token-shaped frame.
pub struct TextInput<'a> {
    text: &'a mut String,
    hint: String,
    width: f32,
}

impl<'a> TextInput<'a> {
    /// Input editing `text`.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            hint: String::new(),
            width: 240.0,
        }
    }
    /// Placeholder text.
    pub fn hint(mut self, h: impl Into<String>) -> Self {
        self.hint = h.into();
        self
    }
    /// Total width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl Widget for TextInput<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let t = active_theme(ui.ctx());
        let idx = ui.painter().add(Shape::Noop);
        let (rect, _) = ui.allocate_exact_size(
            vec2(self.width, t.shape.control_height),
            egui::Sense::hover(),
        );
        let inner = rect.shrink2(vec2(t.spacing.md, 0.0));
        let tok0 = t.input(false);
        let resp = ui
            .scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                ui.centered_and_justified(|ui| {
                    ui.add(
                        TextEdit::singleline(self.text)
                            .frame(egui::Frame::NONE)
                            .text_color(color32(tok0.text))
                            .hint_text(RichText::new(&self.hint).color(color32(tok0.placeholder))),
                    )
                })
                .inner
            })
            .inner;
        let tok = t.input(resp.has_focus());
        ui.painter().set(
            idx,
            Shape::Vec(box_shapes_for(
                &t,
                rect,
                Some(tok.background),
                Some(tok.border),
                resp.has_focus(),
                false,
                Some(dodai_ui_core::SurfaceClass::Input),
            )),
        );
        if crate::surface::take_animated() {
            ui.ctx().request_repaint();
        }
        resp
    }
}

/// Themed hover tooltip (monospace, token colours).
pub fn tooltip(resp: Response, text: &str) -> Response {
    let t = active_theme(&resp.ctx);
    let s = RichText::new(text)
        .monospace()
        .size(t.typography.size_sm)
        .color(color32(t.palette.text));
    resp.on_hover_ui(|ui| {
        ui.label(s);
    })
}
