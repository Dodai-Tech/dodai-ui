//! Every Dodai component in Freya. Keys: `t`/`T` next/prev theme, `+`/`-` glow.
//! Run: `cargo run -p dodai-freya --example freya_showcase`

use dodai::icons::IconId;
use dodai::themes;

use dodai_core::{ButtonVariant, Tone};
use dodai_freya::*;
use freya::prelude::*;

fn main() {
    launch_with_props(app, "Dodai UI · Freya", (1100.0, 900.0));
}

fn app() -> Element {
    let dodai = use_init_dodai(themes::cyberpunk::yellow());
    let t = dodai.theme();
    let mut tab = use_signal(|| 0usize);
    let mut on = use_signal(|| true);
    let mut checked = use_signal(|| true);
    let mut radio = use_signal(|| 0usize);
    let mut slider = use_signal(|| 40.0f64);
    let mut text = use_signal(String::new);
    let mut sel = use_signal(|| 1usize);
    let bg = css(t.palette.background);
    let gap = t.spacing.lg;
    let glow = dodai.glow_intensity();

    rsx! {
        rect {
            width: "100%", height: "100%", background: "{bg}",
            onglobalkeydown: move |e: KeyboardEvent| match &e.key {
                Key::Character(c) if c == "t" => dodai.next(),
                Key::Character(c) if c == "T" => dodai.prev(),
                Key::Character(c) if c == "+" || c == "=" => dodai.set_glow_intensity(dodai.glow_intensity() + 0.25),
                Key::Character(c) if c == "-" => dodai.set_glow_intensity(dodai.glow_intensity() - 0.25),
                _ => {}
            },
            ScrollView {
                rect { padding: "{gap}", spacing: "{gap}",
                    DodaiHeading { text: "Dodai UI", size: t.typography.size_xxl }
                    DodaiMuted { text: "{t.name()} · t/T switch theme · +/- glow ({glow:.2})" }
                    rect { direction: "horizontal", spacing: "8", cross_align: "center",
                        DodaiThemeSwitcher {}
                        DodaiButton { label: "Glow −", variant: ButtonVariant::Secondary, onclick: move |_| dodai.set_glow_intensity(dodai.glow_intensity() - 0.25) }
                        DodaiButton { label: "Glow +", variant: ButtonVariant::Secondary, onclick: move |_| dodai.set_glow_intensity(dodai.glow_intensity() + 0.25) }
                    }
                    DodaiDivider {}
                    DodaiHeading { text: "Buttons", size: t.typography.size_lg }
                    rect { direction: "horizontal", spacing: "12",
                        DodaiButton { label: "Primary", variant: ButtonVariant::Primary }
                        DodaiButton { label: "Secondary", variant: ButtonVariant::Secondary }
                        DodaiButton { label: "Outline", variant: ButtonVariant::Outline }
                        DodaiButton { label: "Outline 2", variant: ButtonVariant::OutlineSecondary }
                        DodaiButton { label: "Danger", variant: ButtonVariant::Danger }
                        DodaiButton { label: "Ghost", variant: ButtonVariant::Ghost }
                        DodaiButton { label: "Disabled", disabled: true }
                    }
                    DodaiHeading { text: "Tags", size: t.typography.size_lg }
                    rect { direction: "horizontal", spacing: "8",
                        DodaiTag { label: "info", tone: Tone::Info }
                        DodaiTag { label: "ok", tone: Tone::Success }
                        DodaiTag { label: "warn", tone: Tone::Warning }
                        DodaiTag { label: "danger", tone: Tone::Danger }
                        DodaiTag { label: "outline", tone: Tone::Info, solid: false }
                    }
                    DodaiHeading { text: "Icons", size: t.typography.size_lg }
                    rect { direction: "horizontal", spacing: "12",
                        DodaiIcon { icon: IconId::Home }
                        DodaiIcon { icon: IconId::Search }
                        DodaiIcon { icon: IconId::Settings }
                        DodaiIcon { icon: IconId::Heart, filled: true }
                        DodaiIcon { icon: IconId::Airplane }
                    }
                    DodaiTabs { labels: vec!["Alpha".to_string(), "Beta".into(), "Gamma".into()], selected: *tab.read(), onselect: move |i| tab.set(i) }
                    rect { direction: "horizontal", spacing: "{gap}",
                        DodaiPanel { title: "Panel", width: 300.0, height: 200.0,
                            DodaiLabel { text: "Label text" }
                            DodaiMuted { text: "Muted text" }
                            DodaiMono { text: "mono 0xDEADBEEF" }
                        }
                        DodaiPanel { title: "Focused panel", width: 300.0, height: 200.0, focused: true,
                            DodaiLabel { text: "Glows with theme" }
                        }
                        rect { spacing: "10",
                            DodaiHeading { text: "Controls", size: t.typography.size_lg }
                            rect { direction: "horizontal", spacing: "12", cross_align: "center",
                                DodaiToggle { on: *on.read(), onchange: move |v| on.set(v) }
                                DodaiToggle { on: false, disabled: true }
                                DodaiCheckbox { checked: *checked.read(), label: "Checkbox", onchange: move |v| checked.set(v) }
                            }
                            rect { direction: "horizontal", spacing: "12",
                                for i in 0..3usize {
                                    DodaiRadio { key: "{i}", selected: *radio.read() == i, label: format!("Radio {i}"), onselect: move |_| radio.set(i) }
                                }
                            }
                            DodaiProgress { value: *slider.read() as f32 }
                            DodaiProgress { value: 70.0, tone: Tone::Success }
                            DodaiSlider { value: *slider.read(), onmoved: move |v| slider.set(v) }
                            DodaiTextInput { value: text.read().clone(), placeholder: "type…", onchange: move |v| text.set(v) }
                            DodaiTooltipHost { tip: "Themed tooltip", DodaiButton { label: "Hover me", variant: ButtonVariant::Outline } }
                        }
                    }
                    rect { width: "260",
                        for i in 0..3usize {
                            DodaiListItem { key: "{i}", label: format!("List item {i}"), selected: *sel.read() == i, onclick: move |_| sel.set(i) }
                        }
                    }
                }
            }
        }
    }
}
