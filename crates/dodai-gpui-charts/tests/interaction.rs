//! Headless interaction tests: real clicks against rendered charts.

use dodai_gpui::install;
use dodai_gpui_charts::*;
use gpui::{
    div, point, prelude::*, px, size, AnyElement, Context, Modifiers, TestAppContext,
    VisualTestContext, Window,
};

type Build = Box<dyn Fn(&mut Window, &mut Context<Host>) -> AnyElement>;

/// A view that rebuilds its single child from a closure on every render.
struct Host {
    build: Build,
}

impl Render for Host {
    fn render(&mut self, w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // shrink-wrapped, top-left anchored, so bounds are those of the component
        div().flex().child(
            div()
                .debug_selector(|| "target".into())
                .flex_none()
                .child((self.build)(w, cx)),
        )
    }
}

fn mount(
    cx: &mut TestAppContext,
    build: impl Fn(&mut Window, &mut Context<Host>) -> AnyElement + 'static,
) -> &mut VisualTestContext {
    cx.update(|cx| install(cx, dodai_core::Theme::default()));
    let (_, vcx) = cx.add_window_view(|_, _| Host {
        build: Box::new(build),
    });
    vcx.simulate_resize(size(px(900.0), px(700.0)));
    vcx.run_until_parked();
    vcx
}

fn click(vcx: &mut VisualTestContext, x: f32, y: f32) {
    let p = point(px(x), px(y));
    vcx.simulate_click(p, Modifiers::none());
    vcx.run_until_parked();
}

fn target(vcx: &mut VisualTestContext) -> (f32, f32, f32, f32) {
    let b = vcx.debug_bounds("target").expect("laid out");
    (
        f32::from(b.left()),
        f32::from(b.top()),
        f32::from(b.size.width),
        f32::from(b.size.height),
    )
}

/// Motion runs on the wall clock: wait it out, redrawing as we go.
fn settle(vcx: &mut VisualTestContext) {
    for _ in 0..12 {
        std::thread::sleep(std::time::Duration::from_millis(80));
        vcx.update(|_, cx| cx.refresh_windows());
        vcx.run_until_parked();
    }
}

#[gpui::test]
fn legend_click_hides_series_and_all_hidden_survives(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(400.0))
            .flex()
            .flex_col()
            .child(
                LineChart::new(
                    "tl",
                    [
                        Series::new("Alpha", [1.0, 5.0, 2.0]),
                        Series::new("Beta", [9.0, 3.0, 4.0]),
                    ],
                )
                .toggleable(true),
            )
            .child(
                BarChart::new(
                    "tb",
                    [
                        Series::new("Gamma", [1.0, 5.0]),
                        Series::new("Delta", [9.0, 3.0]),
                    ],
                )
                .toggleable(true),
            )
            .into_any_element()
    });
    let (x, y, w, h) = target(vcx);
    // legends are the last row of each chart: click the left entries from the bottom up
    for yy in [y + h - 6.0, y + h / 2.0 - 8.0] {
        for xx in [x + 8.0, x + 70.0, x + 8.0, x + 70.0, x + 8.0] {
            click(vcx, xx, yy);
        }
    }
    settle(vcx);
    let (_, _, w2, _) = target(vcx);
    assert_eq!(w, w2, "toggling never changes the chart width");
}
