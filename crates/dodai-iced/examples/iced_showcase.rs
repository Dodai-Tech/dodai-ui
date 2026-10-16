//! Every dodai-iced component in one scrollable window.
//!
//! `cargo run -p dodai-iced --example iced_showcase`
//! Keys: `t` / `T` = next / previous theme. Buttons: theme switcher, glow +/-.

use dodai_core::{ButtonVariant, ThemeSwitcher, Tone};
use dodai_iced::IcedTheme;
use iced::widget::{
    checkbox, column, container, pick_list, progress_bar, radio, row, scrollable, slider, stack,
    text_input, toggler, tooltip, Space,
};
use iced::{event, keyboard, Element, Length, Subscription, Theme as IcedBase};

struct App {
    sw: ThemeSwitcher,
    lt: IcedTheme,
    glow: f32,
    query: String,
    tab: usize,
    on: bool,
    checked: bool,
    choice: Choice,
    level: f32,
    pick: Option<&'static str>,
    selected: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    A,
    B,
    C,
}

#[derive(Debug, Clone)]
enum Msg {
    NextTheme,
    PrevTheme,
    SelectTheme(usize),
    Glow(f32),
    Query(String),
    Tab(usize),
    Toggle(bool),
    Check(bool),
    Choose(Choice),
    Level(f32),
    Pick(&'static str),
    Select(usize),
    Noop,
}

const OPTIONS: [&str; 3] = ["Arasaka", "Militech", "Kang Tao"];

fn boot() -> App {
    let sw = ThemeSwitcher::new();
    let glow = 1.0;
    let lt = IcedTheme::new(sw.current().clone().with_glow_intensity(glow));
    App {
        sw,
        lt,
        glow,
        query: String::new(),
        tab: 0,
        on: true,
        checked: true,
        choice: Choice::A,
        level: 60.0,
        pick: Some(OPTIONS[0]),
        selected: 1,
    }
}

fn apply(app: &mut App) {
    app.lt
        .set_theme(app.sw.current().clone().with_glow_intensity(app.glow));
}

fn update(app: &mut App, msg: Msg) {
    match msg {
        Msg::NextTheme => {
            app.sw.cycle_next();
            apply(app);
        }
        Msg::PrevTheme => {
            app.sw.cycle_prev();
            apply(app);
        }
        Msg::SelectTheme(i) => {
            app.sw.select_index(i);
            apply(app);
        }
        Msg::Glow(g) => {
            app.glow = g.clamp(0.0, 3.0);
            apply(app);
        }
        Msg::Query(q) => app.query = q,
        Msg::Tab(i) => app.tab = i,
        Msg::Toggle(b) => app.on = b,
        Msg::Check(b) => app.checked = b,
        Msg::Choose(c) => app.choice = c,
        Msg::Level(v) => app.level = v,
        Msg::Pick(p) => app.pick = Some(p),
        Msg::Select(i) => app.selected = i,
        Msg::Noop => {}
    }
}

fn subscription(_app: &App) -> Subscription<Msg> {
    event::listen_with(|event, _status, _id| match event {
        iced::Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Character(c),
            ..
        }) => match c.as_str() {
            "t" => Some(Msg::NextTheme),
            "T" => Some(Msg::PrevTheme),
            _ => None,
        },
        _ => None,
    })
}

fn section<'a>(lt: &IcedTheme, title: &str, body: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    lt.panel_view(Some(title), body, false)
}

fn view(app: &App) -> Element<'_, Msg> {
    let lt = &app.lt;
    let sp = lt.theme().spacing.clone();
    let btn = |l: &str, v: ButtonVariant, on: Option<Msg>| lt.button_view(l, v, on);

    // Header: theme switcher + glow control.
    let themes: Vec<Element<Msg>> = app
        .sw
        .themes()
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let v = if i == app.sw.index() {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Outline
            };
            btn(t.name(), v, Some(Msg::SelectTheme(i)))
        })
        .collect();
    let header = column![
        lt.heading_text("Dodai UI · Iced"),
        lt.muted_text(&format!(
            "theme: {} · shape: {:?} · keys: t / T cycle themes",
            lt.theme().name(),
            lt.theme().shape.kind()
        )),
        row(themes).spacing(sp.sm).wrap(),
        row![
            btn("Prev", ButtonVariant::Ghost, Some(Msg::PrevTheme)),
            btn("Next", ButtonVariant::Ghost, Some(Msg::NextTheme)),
            Space::new().width(sp.lg),
            lt.label_text("Glow"),
            btn(
                "−",
                ButtonVariant::OutlineSecondary,
                Some(Msg::Glow(app.glow - 0.25))
            ),
            lt.mono_text(&format!("{:.2}", app.glow)),
            btn(
                "+",
                ButtonVariant::OutlineSecondary,
                Some(Msg::Glow(app.glow + 0.25))
            ),
        ]
        .spacing(sp.sm)
        .align_y(iced::Alignment::Center),
    ]
    .spacing(sp.md);

    let buttons = section(
        lt,
        "Buttons",
        column![
            row![
                btn("Primary", ButtonVariant::Primary, Some(Msg::Noop)),
                btn("Secondary", ButtonVariant::Secondary, Some(Msg::Noop)),
                btn("Outline", ButtonVariant::Outline, Some(Msg::Noop)),
                btn(
                    "Outline 2",
                    ButtonVariant::OutlineSecondary,
                    Some(Msg::Noop)
                ),
                btn("Danger", ButtonVariant::Danger, Some(Msg::Noop)),
                btn("Ghost", ButtonVariant::Ghost, Some(Msg::Noop)),
            ]
            .spacing(sp.md)
            .wrap(),
            row![
                btn("Disabled", ButtonVariant::Primary, None),
                btn("Disabled outline", ButtonVariant::Outline, None),
            ]
            .spacing(sp.md),
        ]
        .spacing(sp.md),
    );

    let panels = section(
        lt,
        "Panels",
        row![
            lt.panel_view(Some("Idle panel"), lt.body_text("Border only."), false),
            lt.panel_view(
                Some("Focused panel"),
                lt.body_text("Glow + bright border."),
                true
            ),
        ]
        .spacing(sp.xl),
    );

    let tags = section(
        lt,
        "Tags",
        column![
            row![
                lt.tag("online", Tone::Success, true),
                lt.tag("warn", Tone::Warning, true),
                lt.tag("breach", Tone::Danger, true),
                lt.tag("info", Tone::Info, true),
            ]
            .spacing(sp.sm),
            row![
                lt.tag("online", Tone::Success, false),
                lt.tag("warn", Tone::Warning, false),
                lt.tag("breach", Tone::Danger, false),
                lt.tag("info", Tone::Info, false),
            ]
            .spacing(sp.sm),
        ]
        .spacing(sp.sm),
    );

    let tabs = section(
        lt,
        "Tabs",
        column![
            lt.tabs(
                &["Overview", "Netwatch", "Braindance", "Settings"],
                app.tab,
                Msg::Tab
            ),
            lt.divider(),
            lt.muted_text(&format!("active tab: {}", app.tab)),
        ]
        .spacing(sp.sm),
    );

    let controls = section(
        lt,
        "Toggler · checkbox · radio",
        column![
            toggler(app.on)
                .label("Neural link")
                .on_toggle(Msg::Toggle)
                .style(lt.toggler()),
            toggler(true).label("Disabled").style(lt.toggler()),
            checkbox(app.checked)
                .label("Remember me")
                .on_toggle(Msg::Check)
                .style(lt.checkbox()),
            row![
                radio("Alpha", Choice::A, Some(app.choice), Msg::Choose).style(lt.radio()),
                radio("Beta", Choice::B, Some(app.choice), Msg::Choose).style(lt.radio()),
                radio("Gamma", Choice::C, Some(app.choice), Msg::Choose).style(lt.radio()),
            ]
            .spacing(sp.lg),
        ]
        .spacing(sp.md),
    );

    let sliders = section(
        lt,
        "Progress · slider",
        column![
            progress_bar(0.0..=100.0, app.level)
                .style(lt.progress(None))
                .girth(10),
            progress_bar(0.0..=100.0, app.level)
                .style(lt.progress(Some(Tone::Success)))
                .girth(10),
            progress_bar(0.0..=100.0, app.level)
                .style(lt.progress(Some(Tone::Danger)))
                .girth(10),
            slider(0.0..=100.0, app.level, Msg::Level).style(lt.slider()),
        ]
        .spacing(sp.md),
    );

    let inputs = section(
        lt,
        "Inputs",
        column![
            text_input("search the net…", &app.query)
                .on_input(Msg::Query)
                .padding(sp.md)
                .style(lt.text_input()),
            text_input("disabled", "locked")
                .padding(sp.md)
                .style(lt.text_input()),
            pick_list(OPTIONS, app.pick, Msg::Pick)
                .padding(sp.md)
                .style(lt.pick_list())
                .menu_style(lt.menu()),
        ]
        .spacing(sp.md),
    );

    let items: Vec<Element<Msg>> = ["Afterlife", "Totentanz", "Lizzie's Bar", "Clouds"]
        .iter()
        .enumerate()
        .map(|(i, l)| lt.list_item(l, i == app.selected, Msg::Select(i)))
        .collect();
    let list = section(lt, "List items", column(items).spacing(sp.xs));

    let misc = section(
        lt,
        "Text · tooltip · scrollable",
        column![
            lt.heading_text("Heading"),
            lt.label_text("Label"),
            lt.body_text("Body text"),
            lt.muted_text("Muted text"),
            lt.mono_text("mono_text();"),
            row![
                lt.body_text("Hover me:"),
                lt.tooltip(
                    btn("?", ButtonVariant::Outline, Some(Msg::Noop)),
                    "Themed tooltip",
                    tooltip::Position::Top
                ),
            ]
            .spacing(sp.md)
            .align_y(iced::Alignment::Center),
            container(
                scrollable(
                    column(
                        (0..12)
                            .map(|i| lt.body_text(&format!("scroll row {i}")).into())
                            .collect::<Vec<Element<Msg>>>()
                    )
                    .spacing(sp.xs)
                    .width(Length::Fill)
                )
                .style(lt.scrollable())
            )
            .height(90),
        ]
        .spacing(sp.sm),
    );

    let body = column![header, buttons, panels, tags, tabs, controls, sliders, inputs, list, misc]
        .spacing(sp.lg)
        .padding(sp.xl);

    let page = container(scrollable(body).style(lt.scrollable()))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lt.background());

    stack![page, lt.scanlines()].into()
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("Dodai UI · Iced")
        .theme(|app: &App| -> IcedBase { app.lt.iced_theme() })
        .subscription(subscription)
        .run()
}
