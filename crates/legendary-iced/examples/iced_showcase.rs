//! `cargo run -p legendary-iced --example iced_showcase`

use iced::widget::{button, column, container, row, stack, text, text_input, Space};
use iced::{Element, Length, Theme as IcedBase};
use legendary_core::{ButtonVariant, ThemeSwitcher, Tone};
use legendary_iced::{chamfer_frame, IcedTheme};

struct App {
    sw: ThemeSwitcher,
    lt: IcedTheme,
    query: String,
}

#[derive(Debug, Clone)]
enum Msg {
    NextTheme,
    PrevTheme,
    Query(String),
    Noop,
}

fn boot() -> App {
    let sw = ThemeSwitcher::new();
    let lt = IcedTheme::new(sw.current().clone());
    App {
        sw,
        lt,
        query: String::new(),
    }
}

fn update(app: &mut App, msg: Msg) {
    match msg {
        Msg::NextTheme => app.lt.set_theme(app.sw.cycle_next().clone()),
        Msg::PrevTheme => app.lt.set_theme(app.sw.cycle_prev().clone()),
        Msg::Query(q) => app.query = q,
        Msg::Noop => {}
    }
}

fn view(app: &App) -> Element<'_, Msg> {
    let lt = &app.lt;
    let btn = |label: &str, v: ButtonVariant, m: Msg| -> Element<'_, Msg> {
        button(text(lt.label(label)).size(15))
            .padding([12, 28])
            .style(lt.button(v))
            .on_press(m)
            .into()
    };

    let card = stack![
        chamfer_frame(lt.chamfer(true)),
        container(
            column![
                text(lt.label("Night City Dashboard"))
                    .size(28)
                    .style(lt.heading()),
                text("Wake up, samurai. We have a city to burn.").style(lt.muted()),
                row![
                    text("ONLINE").style(lt.tone(Tone::Success)),
                    text("WARN").style(lt.tone(Tone::Warning)),
                    text("BREACH").style(lt.tone(Tone::Danger)),
                    text("INFO").style(lt.tone(Tone::Info)),
                ]
                .spacing(16),
                text_input("search the net…", &app.query)
                    .on_input(Msg::Query)
                    .padding(12)
                    .style(lt.text_input()),
                row![
                    btn("Jack in", ButtonVariant::Primary, Msg::Noop),
                    btn("Hack", ButtonVariant::OutlineSecondary, Msg::Noop),
                    btn("Abort", ButtonVariant::Danger, Msg::Noop),
                    btn("Ghost", ButtonVariant::Ghost, Msg::Noop),
                ]
                .spacing(12),
            ]
            .spacing(18),
        )
        .padding(28),
    ];

    let bar = row![
        text(lt.theme().name()).style(lt.heading()),
        Space::new().width(Length::Fill),
        btn("◀ Prev", ButtonVariant::Outline, Msg::PrevTheme),
        btn("Next theme ▶", ButtonVariant::Primary, Msg::NextTheme),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);

    container(column![bar, card].spacing(24))
        .padding(32)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(lt.background())
        .into()
}

fn main() -> iced::Result {
    iced::application(boot, update, view)
        .title("Legendary Themes · Iced")
        .theme(|app: &App| -> IcedBase { app.lt.iced_theme() })
        .run()
}
