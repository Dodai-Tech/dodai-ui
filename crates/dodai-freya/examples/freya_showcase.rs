//! Every Dodai component in Freya. Keys: `t`/`T` next/prev theme, `+`/`-` glow.
//! Run: `cargo run -p dodai-freya --example freya_showcase`

use freya::prelude::*;
use dodai_core::{themes, ButtonVariant, Tone};
use dodai_freya::*;

fn main() {
    launch_with_props(app, "Dodai UI · Freya", (1100.0, 900.0));
}

fn app() -> Element {
    let lg = use_init_dodai(themes::cyberpunk::night_city());
    let t = lg.theme();
    let mut tab = use_signal(|| 0usize);
    let mut on = use_signal(|| true);
    let mut checked = use_signal(|| true);
    let mut radio = use_signal(|| 0usize);
    let mut slider = use_signal(|| 40.0f64);
    let mut text = use_signal(String::new);
    let mut sel = use_signal(|| 1usize);
    let bg = css(t.palette.background);
    let gap = t.spacing.lg;
    let glow = lg.glow_intensity();

    rsx! {
        rect {
            width: "100%", height: "100%", background: "{bg}",
            onglobalkeydown: move |e: KeyboardEvent| match &e.key {
                Key::Character(c) if c == "t" => lg.next(),
                Key::Character(c) if c == "T" => lg.prev(),
                Key::Character(c) if c == "+" || c == "=" => lg.set_glow_intensity(lg.glow_intensity() + 0.25),
                Key::Character(c) if c == "-" => lg.set_glow_intensity(lg.glow_intensity() - 0.25),
                _ => {}
            },
            ScrollView {
                rect { padding: "{gap}", spacing: "{gap}",
                    LgHeading { text: "Dodai UI", size: t.typography.size_xxl }
                    LgMuted { text: "{t.name()} · t/T switch theme · +/- glow ({glow:.2})" }
                    rect { direction: "horizontal", spacing: "8", cross_align: "center",
                        LgThemeSwitcher {}
                        LgButton { label: "Glow −", variant: ButtonVariant::Secondary, onclick: move |_| lg.set_glow_intensity(lg.glow_intensity() - 0.25) }
                        LgButton { label: "Glow +", variant: ButtonVariant::Secondary, onclick: move |_| lg.set_glow_intensity(lg.glow_intensity() + 0.25) }
                    }
                    LgDivider {}
                    LgHeading { text: "Buttons", size: t.typography.size_lg }
                    rect { direction: "horizontal", spacing: "12",
                        LgButton { label: "Primary", variant: ButtonVariant::Primary }
                        LgButton { label: "Secondary", variant: ButtonVariant::Secondary }
                        LgButton { label: "Outline", variant: ButtonVariant::Outline }
                        LgButton { label: "Outline 2", variant: ButtonVariant::OutlineSecondary }
                        LgButton { label: "Danger", variant: ButtonVariant::Danger }
                        LgButton { label: "Ghost", variant: ButtonVariant::Ghost }
                        LgButton { label: "Disabled", disabled: true }
                    }
                    LgHeading { text: "Tags", size: t.typography.size_lg }
                    rect { direction: "horizontal", spacing: "8",
                        LgTag { label: "info", tone: Tone::Info }
                        LgTag { label: "ok", tone: Tone::Success }
                        LgTag { label: "warn", tone: Tone::Warning }
                        LgTag { label: "danger", tone: Tone::Danger }
                        LgTag { label: "outline", tone: Tone::Info, solid: false }
                    }
                    LgTabs { labels: vec!["Alpha".to_string(), "Beta".into(), "Gamma".into()], selected: *tab.read(), onselect: move |i| tab.set(i) }
                    rect { direction: "horizontal", spacing: "{gap}",
                        LgPanel { title: "Panel", width: 300.0, height: 200.0,
                            LgLabel { text: "Label text" }
                            LgMuted { text: "Muted text" }
                            LgMono { text: "mono 0xDEADBEEF" }
                        }
                        LgPanel { title: "Focused panel", width: 300.0, height: 200.0, focused: true,
                            LgLabel { text: "Glows with theme" }
                        }
                        rect { spacing: "10",
                            LgHeading { text: "Controls", size: t.typography.size_lg }
                            rect { direction: "horizontal", spacing: "12", cross_align: "center",
                                LgToggle { on: *on.read(), onchange: move |v| on.set(v) }
                                LgToggle { on: false, disabled: true }
                                LgCheckbox { checked: *checked.read(), label: "Checkbox", onchange: move |v| checked.set(v) }
                            }
                            rect { direction: "horizontal", spacing: "12",
                                for i in 0..3usize {
                                    LgRadio { key: "{i}", selected: *radio.read() == i, label: format!("Radio {i}"), onselect: move |_| radio.set(i) }
                                }
                            }
                            LgProgress { value: *slider.read() as f32 }
                            LgProgress { value: 70.0, tone: Tone::Success }
                            LgSlider { value: *slider.read(), onmoved: move |v| slider.set(v) }
                            LgTextInput { value: text.read().clone(), placeholder: "type…", onchange: move |v| text.set(v) }
                            LgTooltipHost { tip: "Themed tooltip", LgButton { label: "Hover me", variant: ButtonVariant::Outline } }
                        }
                    }
                    rect { width: "260",
                        for i in 0..3usize {
                            LgListItem { key: "{i}", label: format!("List item {i}"), selected: *sel.read() == i, onclick: move |_| sel.set(i) }
                        }
                    }
                }
            }
        }
    }
}
