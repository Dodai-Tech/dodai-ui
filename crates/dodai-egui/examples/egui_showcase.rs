//! Every Dodai UI component in egui, with theme switcher (buttons, `t`/`Shift+T`)
//! and glow `+`/`-` controls.
//!
//! `cargo run -p dodai-egui --example egui_showcase`

use dodai_core::{ButtonVariant, Tone};
use dodai_egui::{
    paint_scanlines, tooltip, Button, Checkbox, Divider, DodaiUi, ListItem, Panel, ProgressBar,
    Radio, Slider, Tabs, Tag, TextInput, ThemeController, Toggle,
};
use eframe::egui;

struct App {
    theme: ThemeController,
    tab: usize,
    toggle: bool,
    check: bool,
    radio: u8,
    slider: f32,
    text: String,
    selected: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            theme: ThemeController::new(),
            tab: 0,
            toggle: true,
            check: true,
            radio: 0,
            slider: 40.0,
            text: String::new(),
            selected: 1,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.theme.handle_keys(ctx);
        self.theme.install(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let name = self.theme.switcher.current().name();
                ui.dodai_heading(&format!("Dodai UI · egui · {name}"));
                ui.dodai_muted("t / Shift+T: theme · + / -: glow");
                self.theme.show(ui);
                ui.add(Divider);
                ui.add_space(8.0);

                ui.dodai_label("Buttons");
                ui.horizontal_wrapped(|ui| {
                    for (n, v) in [
                        ("Primary", ButtonVariant::Primary),
                        ("Secondary", ButtonVariant::Secondary),
                        ("Outline", ButtonVariant::Outline),
                        ("Outline 2", ButtonVariant::OutlineSecondary),
                        ("Danger", ButtonVariant::Danger),
                        ("Ghost", ButtonVariant::Ghost),
                    ] {
                        let r = ui.add(Button::new(n).variant(v));
                        tooltip(r, "Themed tooltip");
                    }
                    ui.add(Button::new("Disabled").enabled(false));
                });

                ui.add_space(8.0);
                ui.dodai_label("Tags");
                ui.horizontal_wrapped(|ui| {
                    for (n, tone) in [
                        ("Info", Tone::Info),
                        ("Success", Tone::Success),
                        ("Warning", Tone::Warning),
                        ("Danger", Tone::Danger),
                    ] {
                        ui.add(Tag::new(n).tone(tone));
                        ui.add(Tag::new(n).tone(tone).solid(true));
                    }
                });

                ui.add_space(8.0);
                ui.add(Tabs::new(&mut self.tab, ["Overview", "Network", "Logs"]));

                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.add(Toggle::new(&mut self.toggle));
                    ui.add(Toggle::new(&mut false).enabled(false));
                    ui.add(Checkbox::new(&mut self.check, "Checkbox"));
                    for (i, l) in ["Radio A", "Radio B", "Radio C"].into_iter().enumerate() {
                        ui.add(Radio::new(&mut self.radio, i as u8, l));
                    }
                });

                ui.add_space(8.0);
                ui.add(ProgressBar::new(self.slider / 100.0));
                ui.add(ProgressBar::new(0.7).tone(Tone::Success).width(240.0));
                ui.add(Slider::new(&mut self.slider, 0.0..=100.0));
                ui.add(TextInput::new(&mut self.text).hint("Type here…"));

                ui.add_space(8.0);
                ui.columns(2, |cols| {
                    Panel::new().title("Panel").show(&mut cols[0], |ui| {
                        ui.dodai_label("Card body");
                        ui.dodai_muted("Muted text");
                        ui.dodai_mono("mono_text()");
                    });
                    Panel::new()
                        .title("Focused")
                        .focused(true)
                        .show(&mut cols[1], |ui| {
                            for (i, n) in ["Alpha", "Bravo", "Charlie"].into_iter().enumerate() {
                                if ui
                                    .add(ListItem::new(n).selected(self.selected == i))
                                    .clicked()
                                {
                                    self.selected = i;
                                }
                            }
                        });
                });
            });
        });
        paint_scanlines(ctx);
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Dodai UI · egui",
        eframe::NativeOptions::default(),
        Box::new(|_| Ok(Box::<App>::default())),
    )
}
