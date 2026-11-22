//! Every Dodai UI component in egui, with theme switcher (buttons, `t`/`Shift+T`)
//! and glow `+`/`-` controls.
//!
//! `cargo run -p dodai-ui-egui --example egui_showcase`

use dodai_ui_core::{BorderTint, ButtonVariant, ShadowDir, Texture, Tone};
use dodai_ui_egui::{
    paint_scanlines, tooltip, Button, Checkbox, Divider, DodaiUi, ListItem, Panel, ProgressBar,
    Radio, Slider, Spinner, Tabs, Tag, TextInput, ThemeController, Toggle,
};
use eframe::egui;

/// The Dodai UI logo (`assets/icons/dodai-ui.svg`) at 128 px.
const LOGO_PNG: &[u8] = include_bytes!("../../../assets/icons/dodai-ui-128.png");

/// The logo decoded to RGBA (it is a PNG, see the test).
fn logo() -> egui::IconData {
    eframe::icon_data::from_png_bytes(LOGO_PNG).expect("the logo is a PNG")
}

struct App {
    theme: ThemeController,
    tab: usize,
    toggle: bool,
    check: bool,
    radio: u8,
    slider: f32,
    text: String,
    selected: usize,
    /// The logo's texture, uploaded on the first frame.
    logo: Option<egui::TextureHandle>,
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
            logo: None,
        }
    }
}

/// One labelled row of toggle buttons: the current entry is the primary one.
fn choice<T: Copy + PartialEq>(ui: &mut egui::Ui, label: &str, cur: &mut T, items: &[(&str, T)]) {
    ui.horizontal_wrapped(|ui| {
        ui.dodai_muted(label);
        for (name, v) in items {
            let variant = if *cur == *v {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            if ui.add(Button::new(*name).variant(variant)).clicked() {
                *cur = *v;
            }
        }
    });
}

impl App {
    /// Shape tuning and the surface, like the gpui showcase's Shape and Surface effects.
    fn tuning(&mut self, ui: &mut egui::Ui) {
        let t = self.theme.theme();
        ui.dodai_label("Shape · roundness and borders");
        if t.shape.kind() != dodai_ui_core::ShapeKind::Chamfered {
            ui.horizontal(|ui| {
                ui.dodai_muted("Roundness");
                ui.add(Slider::new(&mut self.theme.tune.roundness, 0.0..=2.0));
            });
        }
        choice(
            ui,
            "Border width",
            &mut self.theme.tune.border_width,
            &[
                ("Default", None),
                ("None", Some(0.0)),
                ("Hairline", Some(1.0)),
                ("Medium", Some(2.0)),
                ("Thick", Some(3.0)),
            ],
        );
        let tints = BorderTint::ALL.map(|b| (b.label(), b));
        choice(ui, "Border colour", &mut self.theme.tune.tint, &tints);
        choice(
            ui,
            "Outline all",
            &mut self.theme.tune.outline_all,
            &[("Off", false), ("On", true)],
        );
        if t.effects.hard_shadow > 0.0 {
            let dirs = ShadowDir::ALL.map(|d| (d.label(), d));
            choice(
                ui,
                "Shadow direction",
                &mut self.theme.tune.shadow_dir,
                &dirs,
            );
            let mut off = self.theme.tune.hard_shadow.unwrap_or(t.effects.hard_shadow);
            ui.horizontal(|ui| {
                ui.dodai_muted("Shadow offset");
                if ui.add(Slider::new(&mut off, 0.0..=12.0)).changed() {
                    self.theme.tune.hard_shadow = Some(off);
                }
            });
        }
        if ui
            .add(Button::new("Reset shape").variant(ButtonVariant::Ghost))
            .clicked()
        {
            self.theme.tune = dodai_ui_core::ShapeTune::DEFAULT;
        }

        ui.add_space(8.0);
        ui.dodai_label("Surface effects");
        let mode = dodai_ui_core::SurfaceMode::of(&t, &self.theme.surface);
        ui.horizontal_wrapped(|ui| {
            for m in dodai_ui_core::SurfaceMode::ALL {
                let variant = if m == mode {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                };
                if ui.add(Button::new(m.label()).variant(variant)).clicked() {
                    self.theme.surface = m.apply(&t, &self.theme.surface);
                }
            }
        });
        ui.dodai_muted(&mode.describe(&t));
        let before = t.resolve_surface(&self.theme.surface);
        let mut mix = before;
        let fx = &mut mix;
        ui.horizontal_wrapped(|ui| {
            ui.dodai_muted("Texture");
            for tex in t.surface_textures() {
                let on = fx.texture == *tex;
                let v = if on {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                };
                if ui.add(Button::new(tex.label()).variant(v)).clicked() {
                    fx.texture = *tex;
                    fx.container_texture = *tex;
                    if fx.strength <= 0.0 {
                        fx.strength = 0.5;
                        fx.container_strength = 0.3;
                    }
                }
            }
        });
        if fx.texture != Texture::None {
            ui.horizontal(|ui| {
                ui.dodai_muted("Strength");
                ui.add(Slider::new(&mut fx.strength, 0.0..=1.0));
            });
            ui.horizontal(|ui| {
                ui.dodai_muted("Scale");
                ui.add(Slider::new(&mut fx.scale, 0.5..=3.0));
            });
        }
        let ef = t.surface_effects();
        for (on, name, v) in [
            (ef.sheen, "Sheen", &mut fx.sheen),
            (ef.pulse, "Pulse", &mut fx.pulse),
            (ef.chase, "Chase", &mut fx.chase),
            (ef.edge, "Edge light", &mut fx.edge),
            (ef.depth, "Depth", &mut fx.depth),
            (ef.brackets, "Brackets", &mut fx.brackets),
        ] {
            if on {
                ui.horizontal(|ui| {
                    ui.dodai_muted(name);
                    ui.add(Slider::new(v, 0.0..=1.0));
                });
            }
        }
        // Only a slider the user moved leaves the design's own surface.
        if mix != before {
            mix.targets = dodai_ui_core::SurfaceTargets::ALL;
            self.theme.surface = mix.into();
        }

        ui.add_space(8.0);
        ui.dodai_label("Icons (the design's own pack)");
        ui.horizontal_wrapped(|ui| {
            use dodai_ui_icons::IconId as I;
            for id in [
                I::Home,
                I::Search,
                I::Heart,
                I::Settings,
                I::Bell,
                I::Calendar,
                I::Camera,
                I::Car,
                I::ChargingStation,
                I::Download,
                I::Envelope,
                I::Eye,
                I::Globe,
                I::Lock,
                I::User,
                I::Wrench,
            ] {
                ui.add(dodai_ui_egui::Icon::new(id).size(24.0));
            }
        });
    }
}

impl eframe::App for App {
    fn ui(&mut self, root: &mut egui::Ui, _: &mut eframe::Frame) {
        let ctx = &root.ctx().clone();
        self.theme.handle_keys(ctx);
        self.theme.install(ctx);
        let theme = self.theme.switcher.current().clone();
        dodai_ui_egui::root(root, &theme, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let name = self.theme.switcher.current().name();
                let logo = self.logo.get_or_insert_with(|| {
                    let icon = logo();
                    ctx.load_texture(
                        "dodai-logo",
                        egui::ColorImage::from_rgba_unmultiplied(
                            [icon.width as usize, icon.height as usize],
                            &icon.rgba,
                        ),
                        egui::TextureOptions::LINEAR,
                    )
                });
                ui.horizontal(|ui| {
                    ui.add(egui::Image::new(egui::load::SizedTexture::new(
                        logo.id(),
                        egui::vec2(44.0, 44.0),
                    )));
                    ui.dodai_heading(&format!("Dodai UI · egui · {name}"));
                });
                ui.dodai_muted("t / Shift+T: theme · + / -: glow");
                // This showcase runs on egui: that toolkit is the solid, highlighted one.
                ui.horizontal_wrapped(|ui| {
                    for name in [
                        "gpui", "iced", "egui", "slint", "ratatui", "dioxus", "leptos", "yew",
                        "bevy", "web",
                    ] {
                        ui.add(Tag::new(name).tone(Tone::Info).solid(name == "egui"));
                    }
                });
                self.theme.show(ui);
                ui.add(Divider);
                egui::CollapsingHeader::new("Shape · Surface effects · Icons")
                    .default_open(false)
                    .show(ui, |ui| self.tuning(ui));
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
                ui.horizontal(|ui| {
                    use dodai_ui_core::SpinnerStyle as S;
                    ui.add(Spinner::new());
                    for st in [S::Orbit, S::Bars, S::Steps, S::Grid] {
                        ui.add(Spinner::new().size(32.0).style(st));
                    }
                });
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
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_icon(std::sync::Arc::new(logo())),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::<App>::default())),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_logo_decodes_to_a_square_icon() {
        let icon = super::logo();
        assert_eq!((icon.width, icon.height), (128, 128));
        assert_eq!(icon.rgba.len(), 128 * 128 * 4);
    }
}
