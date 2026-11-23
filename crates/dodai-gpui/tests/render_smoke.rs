//! Headless render smoke test: every component draws under every built-in
//! theme without panicking (key collisions, divide-by-zero, bad geometry).

use dodai_gpui::*;
use gpui::{div, prelude::*, px, size, Context, TestAppContext, Window};
use std::sync::atomic::{AtomicUsize, Ordering};

static DRAWS: AtomicUsize = AtomicUsize::new(0);

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        DRAWS.fetch_add(1, Ordering::SeqCst);
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(Button::new("b", "Go").variant(dodai_core::ButtonVariant::Primary))
            .child(Panel::new().child("panel"))
            .child(Clock::new())
            .child(Clock::new().face(ClockFace::Analog))
            .child(Timer::new("timer"))
            .child(Stopwatch::new("sw"))
            .child(TapTempo::new("tap"))
            .child(CountUp::new("cu", 42.0).suffix("%"))
            .child(Breadcrumb::new("bc", ["a", "b", "c"]))
            .child(Pagination::new("pg", 3, 12))
            .child(
                Popover::new("pop")
                    .trigger(Button::new("pb", "Open"))
                    .content("body"),
            )
            .child(FrostedScene::mesh("fs", 200.0, 120.0).panel((10.0, 10.0), (80.0, 40.0), "hi"))
            .child(ThemeSwitcherBar::new("tsb"))
            .child(List::new(
                "ls",
                [ListItem::new("a", "One"), ListItem::new("b", "Two")],
            ))
            .child(CircularProgress::new(0.4))
            .child(Banner::new("bn", dodai_core::Tone::Info, "hello"))
            .child(OtpInput::new("otp", 6))
            .child(CodeBlock::new("code", "fn main() {}\n"))
            .child(Markdown::new(
                "md",
                "# Title\n\n- a\n- b\n\n`code` **bold**",
            ))
            .child(
                DataTable::new("dt")
                    .columns([Column::new("A"), Column::new("B")])
                    .rows([["1", "2"], ["3", "4"]])
                    .height(120.0),
            )
    }
}

#[gpui::test]
fn renders_under_every_theme(cx: &mut TestAppContext) {
    for theme in dodai_core::switcher().themes().to_vec() {
        cx.update(|cx| install(cx, theme.clone()));
        let (_, vcx) = cx.add_window_view(|_, _| Page);
        vcx.simulate_resize(size(px(900.0), px(1400.0)));
        vcx.run_until_parked();
    }
    // Not vacuous: the page was really rendered for each theme.
    assert!(DRAWS.load(Ordering::SeqCst) >= dodai_core::switcher().themes().len());
}
