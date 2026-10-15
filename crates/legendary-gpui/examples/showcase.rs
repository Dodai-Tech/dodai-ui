//! `cargo run -p legendary-gpui --example showcase`
//! Click "NEXT THEME" to cycle the Cyberpunk variants.

use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, ClickEvent, Context, SharedString, Window,
    WindowBounds, WindowOptions,
};
use legendary_core::{ButtonVariant, ControlState, Tone};
use legendary_gpui::{cycle_next, cycle_prev, hsla, install, ActiveLegendary, LegendaryStyled};

struct Showcase;

impl Render for Showcase {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let lt = cx.legendary();
        let t = lt.theme().clone();
        let label = |s: &str| -> SharedString {
            if t.typography.uppercase_labels {
                s.to_uppercase().into()
            } else {
                s.to_string().into()
            }
        };

        let button = |id: &'static str, text: &str, v: ButtonVariant| {
            div()
                .id(id)
                .flex()
                .items_center()
                .justify_center()
                .lg_button(&t, v, ControlState::Idle)
                .hover(|s| s.opacity(0.85))
                .child(label(text))
        };

        div()
            .size_full()
            .lg_background(&t)
            .p(px(t.spacing.xl))
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .text_size(px(t.typography.size_xl))
                    .text_color(hsla(t.palette.primary))
                    .child(label(t.name())),
            )
            .child(
                div()
                    .lg_panel(&t, true)
                    .p(px(t.spacing.lg))
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.sm))
                    .child(
                        div()
                            .text_color(hsla(t.palette.text_muted))
                            .child("Wake up, samurai. We have a city to burn."),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(t.spacing.md))
                            .child(div().text_color(lt.tone(Tone::Success)).child("ONLINE"))
                            .child(div().text_color(lt.tone(Tone::Warning)).child("WARN"))
                            .child(div().text_color(lt.tone(Tone::Danger)).child("BREACH"))
                            .child(div().text_color(lt.tone(Tone::Info)).child("INFO")),
                    )
                    .child(
                        div()
                            .lg_input(&t, true)
                            .h(px(t.shape.control_height))
                            .flex()
                            .items_center()
                            .child("search the net…"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.md))
                    .child(
                        button("prev", "◀ Prev", ButtonVariant::Outline)
                            .on_click(cx.listener(|_, _: &ClickEvent, _w, cx| cycle_prev(cx))),
                    )
                    .child(
                        button("next", "Next theme ▶", ButtonVariant::Primary)
                            .on_click(cx.listener(|_, _: &ClickEvent, _w, cx| cycle_next(cx))),
                    )
                    .child(button("hack", "Hack", ButtonVariant::OutlineSecondary))
                    .child(button("abort", "Abort", ButtonVariant::Danger)),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        install(cx, legendary_core::Theme::default());
        let bounds = Bounds::centered(None, size(px(900.0), px(520.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| Showcase),
        )
        .expect("open window");
        cx.activate(true);
    });
}
