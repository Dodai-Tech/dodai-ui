//! Themed button, tag, panel and tabs in a xilem window.
//! `cargo run -p dodai-ui-xilem --example xilem_showcase`.

use dodai_ui_core::{ButtonVariant, Theme, Tone};
use dodai_ui_xilem::{button, color, panel, tabs, tag};
use xilem::style::Style;
use xilem::view::{flex_col, flex_row, label};
use xilem::winit::error::EventLoopError;
use xilem::{EventLoop, WidgetView, WindowOptions, Xilem};

struct App {
    theme: Theme,
    tab: usize,
    clicks: u32,
}

fn view(app: &mut App) -> impl WidgetView<App> + use<> {
    let t = app.theme.clone();
    let body = flex_col((
        label("Dodai UI · xilem").color(color(t.palette.text)),
        flex_row((
            button(
                &t,
                format!("Clicked {}", app.clicks),
                ButtonVariant::Primary,
                |a: &mut App| a.clicks += 1,
            ),
            button(&t, "Outline", ButtonVariant::Outline, |_: &mut App| {}),
            button(&t, "Danger", ButtonVariant::Danger, |_: &mut App| {}),
            button(&t, "Ghost", ButtonVariant::Ghost, |_: &mut App| {}),
        )),
        flex_row((
            tag(&t, "Info", Tone::Info, false),
            tag(&t, "Success", Tone::Success, true),
            tag(&t, "Danger", Tone::Danger, true),
        )),
        tabs(
            &t,
            &["Overview", "Activity", "Settings"],
            app.tab,
            |a: &mut App, i| a.tab = i,
        ),
        panel(
            &t,
            false,
            label(format!("Tab {}", app.tab + 1)).color(color(t.palette.text)),
        ),
    ));
    dodai_ui_xilem::root(&t, panel(&t, false, body))
}

fn main() -> Result<(), EventLoopError> {
    let app = App {
        theme: dodai_ui_core::dodai::light(),
        tab: 0,
        clicks: 0,
    };
    Xilem::new_simple(app, view, WindowOptions::new("Dodai UI · xilem"))
        .run_in(EventLoop::with_user_event())?;
    Ok(())
}
