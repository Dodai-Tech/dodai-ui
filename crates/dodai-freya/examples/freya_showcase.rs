//! Every Dodai component in Freya. Keys: `t`/`T` next/prev theme, `+`/`-` glow.
//! Run: `cargo run -p dodai-freya --example freya_showcase`

use dodai::icons::IconId;
use dodai::themes;

use dodai_core::{ButtonVariant, Tone};
use dodai_freya::paint::color;
use dodai_freya::*;
use freya::prelude::*;

fn main() {
    launch(
        LaunchConfig::new().with_window(
            WindowConfig::new(app)
                .with_title("Dodai UI · Freya")
                .with_size(1100.0, 900.0),
        ),
    );
}

fn app() -> impl IntoElement {
    let dodai = use_init_dodai(themes::cyberpunk::yellow());
    let t = dodai.theme();
    let mut tab = use_state(|| 0usize);
    let mut on = use_state(|| true);
    let mut checked = use_state(|| true);
    let mut radio = use_state(|| 0usize);
    let mut slider = use_state(|| 40.0f64);
    let text = use_state(String::new);
    let mut sel = use_state(|| 1usize);
    let gap = t.spacing.lg;
    let glow = dodai.glow_intensity();

    let buttons = rect()
        .direction(Direction::Horizontal)
        .spacing(12.0)
        .child(DodaiButton::new("Primary").variant(ButtonVariant::Primary))
        .child(DodaiButton::new("Secondary").variant(ButtonVariant::Secondary))
        .child(DodaiButton::new("Outline").variant(ButtonVariant::Outline))
        .child(DodaiButton::new("Outline 2").variant(ButtonVariant::OutlineSecondary))
        .child(DodaiButton::new("Danger").variant(ButtonVariant::Danger))
        .child(DodaiButton::new("Ghost").variant(ButtonVariant::Ghost))
        .child(DodaiButton::new("Disabled").disabled(true));

    let tags = rect()
        .direction(Direction::Horizontal)
        .spacing(8.0)
        .child(DodaiTag::new("info").tone(Tone::Info))
        .child(DodaiTag::new("ok").tone(Tone::Success))
        .child(DodaiTag::new("warn").tone(Tone::Warning))
        .child(DodaiTag::new("danger").tone(Tone::Danger))
        .child(DodaiTag::new("outline").tone(Tone::Info).solid(false));

    let icons = rect()
        .direction(Direction::Horizontal)
        .spacing(12.0)
        .child(DodaiIcon::new(IconId::Home))
        .child(DodaiIcon::new(IconId::Search))
        .child(DodaiIcon::new(IconId::Settings))
        .child(DodaiIcon::new(IconId::Heart).filled(true))
        .child(DodaiIcon::new(IconId::Airplane));

    let controls = rect()
        .spacing(10.0)
        .child(DodaiText::heading("Controls").size(t.typography.size_lg))
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(12.0)
                .cross_align(Alignment::Center)
                .child(DodaiToggle::new(on()).on_change(move |v| on.set(v)))
                .child(DodaiToggle::new(false).disabled(true))
                .child(
                    DodaiCheckbox::new(checked())
                        .label("Checkbox")
                        .on_change(move |v| checked.set(v)),
                ),
        )
        .child(
            rect()
                .direction(Direction::Horizontal)
                .spacing(12.0)
                .children((0..3usize).map(|i| {
                    DodaiRadio::new(radio() == i)
                        .label(format!("Radio {i}"))
                        .on_select(move || radio.set(i))
                        .into_element()
                })),
        )
        .child(DodaiProgress::new(slider() as f32))
        .child(DodaiProgress::new(70.0).tone(Tone::Success))
        .child(DodaiSlider::new(slider()).on_moved(move |v| slider.set(v)))
        .child(DodaiTextInput::new(text).placeholder("type…"))
        .child(
            DodaiTooltipHost::new("Themed tooltip")
                .child(DodaiButton::new("Hover me").variant(ButtonVariant::Outline)),
        );

    let panels = rect()
        .direction(Direction::Horizontal)
        .spacing(gap)
        .child(
            DodaiPanel::new()
                .title("Panel")
                .size(300.0, 200.0)
                .child(DodaiText::label("Label text"))
                .child(DodaiText::muted("Muted text"))
                .child(DodaiText::mono("mono 0xDEADBEEF")),
        )
        .child(
            DodaiPanel::new()
                .title("Focused panel")
                .size(300.0, 200.0)
                .focused(true)
                .child(DodaiText::label("Glows with theme")),
        )
        .child(controls);

    let list = rect().width(Size::px(260.0)).children((0..3usize).map(|i| {
        DodaiListItem::new(format!("List item {i}"))
            .selected(sel() == i)
            .on_press(move || sel.set(i))
            .key(i)
            .into_element()
    }));

    rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(color(t.palette.background))
        .on_global_key_down(move |e: Event<KeyboardEventData>| match &e.key {
            Key::Character(c) if c == "t" => dodai.next(),
            Key::Character(c) if c == "T" => dodai.prev(),
            Key::Character(c) if c == "+" || c == "=" => {
                dodai.set_glow_intensity(dodai.glow_intensity() + 0.25)
            }
            Key::Character(c) if c == "-" => {
                dodai.set_glow_intensity(dodai.glow_intensity() - 0.25)
            }
            _ => {}
        })
        .child(
            ScrollView::new().child(
                rect()
                    .padding(gap)
                    .spacing(gap)
                    .child(DodaiText::heading("Dodai UI").size(t.typography.size_xxl))
                    .child(DodaiText::muted(format!(
                        "{} · t/T switch theme · +/- glow ({glow:.2})",
                        t.name()
                    )))
                    .child(
                        rect()
                            .direction(Direction::Horizontal)
                            .spacing(8.0)
                            .cross_align(Alignment::Center)
                            .child(DodaiThemeSwitcher)
                            .child(
                                DodaiButton::new("Glow −")
                                    .variant(ButtonVariant::Secondary)
                                    .on_press(move || {
                                        dodai.set_glow_intensity(dodai.glow_intensity() - 0.25)
                                    }),
                            )
                            .child(
                                DodaiButton::new("Glow +")
                                    .variant(ButtonVariant::Secondary)
                                    .on_press(move || {
                                        dodai.set_glow_intensity(dodai.glow_intensity() + 0.25)
                                    }),
                            ),
                    )
                    .child(DodaiDivider)
                    .child(DodaiText::heading("Buttons").size(t.typography.size_lg))
                    .child(buttons)
                    .child(DodaiText::heading("Tags").size(t.typography.size_lg))
                    .child(tags)
                    .child(DodaiText::heading("Icons").size(t.typography.size_lg))
                    .child(icons)
                    .child(
                        DodaiTabs::new(["Alpha", "Beta", "Gamma"])
                            .selected(tab())
                            .on_select(move |i| tab.set(i)),
                    )
                    .child(panels)
                    .child(list),
            ),
        )
}
