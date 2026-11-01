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
                LineChart::new([
                    Series::new("Alpha", [1.0, 5.0, 2.0]),
                    Series::new("Beta", [9.0, 3.0, 4.0]),
                ])
                .toggleable(true),
            )
            .child(
                BarChart::new([
                    Series::new("Gamma", [1.0, 5.0]),
                    Series::new("Delta", [9.0, 3.0]),
                ])
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

// ───────────────────────────── layout of the market, status and single-value charts ─────────────────────────────

#[gpui::test]
fn chart_stack_splits_its_height_by_weight(cx: &mut TestAppContext) {
    use std::cell::RefCell;
    use std::rc::Rc;
    let heights: Rc<RefCell<Vec<f32>>> = Rc::default();
    let h2 = heights.clone();
    let vcx = mount(cx, move |_, _| {
        let pane = |h: Rc<RefCell<Vec<f32>>>| {
            move |px_h: f32| {
                h.borrow_mut().push(px_h);
                div().h(px(px_h))
            }
        };
        h2.borrow_mut().clear();
        div()
            .w(px(400.0))
            .child(
                ChartStack::new()
                    .height(400.0)
                    .gap(10.0)
                    .pane(3.0, pane(h2.clone()))
                    .pane(1.0, pane(h2.clone())),
            )
            .into_any_element()
    });
    let hs = heights.borrow().clone();
    assert_eq!(hs.len(), 2, "both panes build");
    assert!(
        (hs[0] + hs[1] - 390.0).abs() < 0.5,
        "gap comes off the total: {hs:?}"
    );
    assert!((hs[0] / hs[1] - 3.0).abs() < 0.05, "weights 3:1 -> {hs:?}");
    let (_, _, _, h) = target(vcx);
    assert!(
        (h - 400.0).abs() < 1.0,
        "the stack is as tall as asked: {h}"
    );
}

#[gpui::test]
fn market_and_planning_charts_keep_their_height(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(480.0))
            .flex()
            .flex_col()
            .child(RenkoChart::new([10.0, 12.0, 15.0, 13.0, 9.0, 14.0, 18.0], 2.0).height(200.0))
            .child(
                DepthChart::new(
                    [(99.0, 5.0), (98.0, 9.0), (97.0, 4.0)],
                    [(101.0, 6.0), (102.0, 8.0), (103.0, 3.0)],
                )
                .height(200.0),
            )
            .into_any_element()
    });
    let (_, _, _, h) = target(vcx);
    assert!(h >= 400.0, "two 200 px charts stack: {h}");
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(480.0))
            .child(Gantt::new([
                GanttTask::new("Design", 0.0, 4.0).progress(1.0),
                GanttTask::new("Build", 3.0, 9.0).progress(0.4).after(0),
                GanttTask::new("Ship", 9.0, 10.0).after(1),
            ]))
            .into_any_element()
    });
    let (_, _, _, h) = target(vcx);
    assert!(h >= 3.0 * 26.0, "a row per task: {h}");
}

#[gpui::test]
fn hovering_status_charts_never_moves_the_layout(cx: &mut TestAppContext) {
    let vcx = mount(cx, |_, _| {
        div()
            .w(px(480.0))
            .flex()
            .flex_col()
            .gap_4()
            .child(BucketBar::new(["0%", "5%", "10%", "20%"], [3, 0, 7]))
            .child(StageBars::new([
                ("Connect", 99.0),
                ("Sign in", 91.0),
                ("DNS", 70.0),
            ]))
            .child(GradientScale::new(-90.0, -30.0, -55.0))
            .child(StatusStrip::new(
                0.0,
                10.0,
                [
                    (0.0, 6.0, dodai_core::Tone::Success),
                    (6.0, 8.0, dodai_core::Tone::Danger),
                ],
            ))
            .into_any_element()
    });
    let (x, y, w, h0) = target(vcx);
    for fy in [0.05, 0.3, 0.55, 0.8, 0.95] {
        for fx in [0.1, 0.5, 0.9] {
            vcx.simulate_mouse_move(
                point(px(x + w * fx), px(y + h0 * fy)),
                None,
                Modifiers::none(),
            );
            vcx.run_until_parked();
            let (_, _, _, h) = target(vcx);
            assert!(
                (h - h0).abs() < 0.5,
                "hover at ({fx}, {fy}) moved the layout: {h0} -> {h}"
            );
        }
    }
}
