//! The Dodai showcase in egui: the same pages, sections and controls as the gpui showcase,
//! built from `dodai-showcase-model` with dodai-egui widgets.
//!
//! `cargo run -p dodai-egui --example egui_showcase`. Env: `DODAI_THEME` (theme id), `DODAI_CAT`
//! (0 = all, 1.. = category), `DODAI_FILTER`.

mod render;
mod shader;

use dodai_core::{ButtonVariant, Kit};
use dodai_egui::{kit_box, Button, DodaiUi, Icon, Panel};
use dodai_showcase_model::{catalog, Action, Ctx, Showcase, CATEGORIES, SPEEDS};
use egui::{vec2, Align, Layout, Sense};

const LOGO_PNG: &[u8] = include_bytes!("../../../../assets/icons/dodai-128.png");

struct App {
    model: Showcase,
    /// A section the sidebar asked to scroll to.
    jump: Option<&'static str>,
    start: std::time::Instant,
}

/// A row of options of which one is picked.
fn segmented(ui: &mut egui::Ui, labels: &[String], at: usize) -> Option<usize> {
    let mut hit = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (i, l) in labels.iter().enumerate() {
            let v = if i == at {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            if ui
                .add(
                    Button::new(l.clone())
                        .variant(v)
                        .size(dodai_core::ControlSize::Small),
                )
                .clicked()
            {
                hit = Some(i);
            }
        }
    });
    hit
}

fn dropdown(ui: &mut egui::Ui, id: &str, items: &[String], at: usize) -> Option<usize> {
    let mut hit = None;
    egui::ComboBox::from_id_salt(id)
        .selected_text(items.get(at).cloned().unwrap_or_default())
        .show_ui(ui, |ui| {
            for (i, l) in items.iter().enumerate() {
                if ui.selectable_label(i == at, l).clicked() {
                    hit = Some(i);
                }
            }
        });
    hit
}

impl App {
    fn top_bar(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Action>) {
        let m = &self.model;
        let designs: Vec<String> = m.design_names().iter().map(|s| s.to_string()).collect();
        let variants: Vec<String> = m.variants().iter().map(|s| s.to_string()).collect();
        let colours = m.colours();
        let colour_at = colours.iter().position(|c| *c == m.colour()).unwrap_or(0);
        let colour_labels: Vec<String> = colours.iter().map(|c| c.label().to_string()).collect();
        let speeds: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
        let on_off = ["Off".to_string(), "On".to_string()];
        let dir = ["LTR".to_string(), "RTL".to_string()];
        ui.horizontal_wrapped(|ui| {
            ui.dodai_muted("Design");
            if let Some(i) = dropdown(ui, "design", &designs, m.design_index()) {
                acts.push(Action::Design(i));
            }
            ui.dodai_muted("Theme");
            if let Some(i) = dropdown(ui, "variant", &variants, m.variant_index()) {
                acts.push(Action::Variant(i));
            }
            ui.dodai_muted("Colour");
            if let Some(i) = segmented(ui, &colour_labels, colour_at) {
                acts.push(Action::Colour(colours[i]));
            }
            ui.dodai_muted("Animate");
            if let Some(i) = segmented(ui, &on_off, usize::from(m.animate)) {
                acts.push(Action::Animate(i == 1));
            }
            ui.dodai_muted("Speed");
            if let Some(i) = segmented(ui, &speeds, m.speed_index()) {
                acts.push(Action::Speed(i));
            }
            ui.dodai_muted("Direction");
            if let Some(i) = segmented(ui, &dir, usize::from(m.rtl)) {
                acts.push(Action::Rtl(i == 1));
            }
            ui.dodai_muted("Patterns");
            if let Some(i) = segmented(ui, &on_off, usize::from(m.patterns)) {
                acts.push(Action::Patterns(i == 1));
            }
        });
    }

    fn sidebar(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Action>) {
        let m = &self.model;
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::from_bytes("bytes://logo.png", LOGO_PNG)
                    .fit_to_exact_size(vec2(36.0, 36.0)),
            );
            ui.dodai_heading("Dodai UI");
        });
        let mut filter = m.filter().to_string();
        if ui
            .add(egui::TextEdit::singleline(&mut filter).hint_text("Filter components…"))
            .changed()
        {
            acts.push(Action::Filter(filter));
        }
        let cat = m.current_category();
        let nav = |ui: &mut egui::Ui, label: &str, count: usize, on: bool| {
            let r = kit_box(ui, Kit::Row { selected: on }, true, |ui, _| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.add(Icon::new(catalog::category_icon(label)).size(16.0));
                    ui.label(label);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.dodai_muted(&count.to_string());
                    });
                });
            });
            r.response.interact(Sense::click()).clicked()
        };
        egui::ScrollArea::vertical().show(ui, |ui| {
            if nav(ui, "All", m.count(None), cat.is_none()) {
                acts.push(Action::Category(None));
            }
            for (i, c) in CATEGORIES.iter().enumerate() {
                if nav(ui, c, m.count(Some(i)), cat == Some(i)) {
                    acts.push(Action::Category(Some(i)));
                }
                if m.lists(i) {
                    for s in m.listed(i) {
                        let on = m.active_section() == Some(s.title);
                        ui.horizontal(|ui| {
                            ui.add_space(16.0);
                            let r = kit_box(ui, Kit::Row { selected: on }, true, |ui, _| {
                                ui.set_min_width(ui.available_width());
                                ui.label(egui::RichText::new(s.title).size(11.0));
                            });
                            if r.response.interact(Sense::click()).clicked() {
                                acts.push(Action::Section(s.title));
                                self.jump = Some(s.title);
                            }
                        });
                    }
                }
            }
        });
        ui.dodai_muted(catalog::KEYS_HINT);
    }

    fn page(&mut self, ui: &mut egui::Ui, acts: &mut Vec<Action>) {
        let theme = self.model.theme();
        let title = self.model.category().map_or("All", |c| CATEGORIES[c]);
        let mut cx = render::Cx {
            theme: theme.clone(),
            vals: self.model.values.clone(),
            acts: Vec::new(),
            time: self.start.elapsed().as_secs_f64(),
        };
        let sections: Vec<_> = self
            .model
            .visible()
            .iter()
            .map(|s| {
                (
                    s.title,
                    s.heading,
                    (s.build)(&Ctx {
                        theme: &theme,
                        values: &self.model.values,
                    }),
                )
            })
            .collect();
        let overview = self.model.overview("egui");
        let jump = self.jump.take();
        let out = egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.dodai_heading(title);
                if let Some(o) = &overview {
                    render::view(ui, o, &mut cx);
                }
                if sections.is_empty() {
                    ui.dodai_muted("No components match");
                }
                let mut tops = Vec::new();
                let top = ui.cursor().min.y;
                for (t, heading, node) in &sections {
                    let r = Panel::new().title(*heading).show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        render::view(ui, node, &mut cx);
                    });
                    if jump == Some(*t) {
                        ui.scroll_to_rect(r.response.rect, Some(Align::Min));
                    }
                    tops.push((*t, r.response.rect.min.y - top));
                }
                tops
            });
        let offset = out.state.clamped_offset().y;
        let end = offset + out.inner_rect.height() >= out.content_size.y - 1.0;
        let tops: Vec<(&'static str, f32)> =
            out.inner.iter().map(|(t, y)| (*t, *y - offset)).collect();
        let held = self.model.held_section();
        if let Some(t) = dodai_core::spy::pick(&tops, held, 0.0, end, 60.0) {
            if self.model.active_section() != Some(t) {
                acts.push(Action::Visible(t));
            }
        }
        acts.append(&mut cx.acts);
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = root.ctx().clone();
        let theme = self.model.theme();
        dodai_egui::install(&ctx, &theme);
        let mut acts: Vec<Action> = Vec::new();
        egui::containers::Panel::left("side")
            .default_size(260.0)
            .show(root, |ui| self.sidebar(ui, &mut acts));
        dodai_egui::root(root, &theme, |ui| {
            egui::containers::Panel::top("bar").show(ui, |ui| self.top_bar(ui, &mut acts));
            egui::CentralPanel::default().show(ui, |ui| self.page(ui, &mut acts));
        });
        for a in acts {
            self.model.update(a);
        }
        if let Some(d) = self.model.redraw_every() {
            ctx.request_repaint_after(d);
        }
        dodai_egui::paint_scanlines(&ctx);
    }
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(LOGO_PNG).expect("the logo is a PNG");
    eframe::run_native(
        "Dodai UI · egui",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1500.0, 900.0])
                .with_icon(std::sync::Arc::new(icon)),
            ..Default::default()
        },
        Box::new(|cc| {
            egui_extras_loaders(&cc.egui_ctx);
            Ok(Box::new(App {
                model: Showcase::from_env(),
                jump: None,
                start: std::time::Instant::now(),
            }))
        }),
    )
}

/// Image loaders for the logo (bytes and `include_image!`).
fn egui_extras_loaders(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}
