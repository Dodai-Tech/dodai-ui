//! Every chart family lays out to a non-empty box under every built-in theme.

use dodai_gpui_charts::*;
use gpui::{div, prelude::*, px, size, AnyElement, Context, TestAppContext, Window};

fn charts() -> Vec<(&'static str, AnyElement)> {
    let s = |n: &'static str| Series::new(n, vec![3.0, 5.0, 4.0, 8.0, 6.0]);
    let parts = || {
        ["a", "b", "c", "d"]
            .into_iter()
            .zip([4.0, 3.0, 2.0, 1.0])
            .map(|(n, v)| (n.to_string(), v))
    };
    vec![
        ("line", LineChart::new([s("x"), s("y")]).into_any_element()),
        ("bar", BarChart::new([s("x"), s("y")]).into_any_element()),
        (
            "area",
            StackedAreaChart::new([s("x"), s("y")]).into_any_element(),
        ),
        (
            "heatmap",
            Heatmap::new(vec![vec![1.0, 2.0, 3.0], vec![3.0, 2.0, 1.0]]).into_any_element(),
        ),
        (
            "histogram",
            Histogram::new(vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0]).into_any_element(),
        ),
        (
            "radar",
            RadarChart::new(["a", "b", "c"], [s("x")]).into_any_element(),
        ),
        ("pie", PieChart::new(parts()).into_any_element()),
        ("funnel", Funnel::new(parts()).into_any_element()),
        ("waterfall", Waterfall::new(parts()).into_any_element()),
        ("treemap", Treemap::new(parts()).into_any_element()),
    ]
}

struct Host;

impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut col = div().flex().flex_col().gap(px(4.0));
        for (name, el) in charts() {
            col = col.child(
                div().flex().child(
                    div()
                        .debug_selector(move || name.into())
                        .w(px(320.0))
                        .child(el),
                ),
            );
        }
        col
    }
}

#[gpui::test]
fn every_chart_lays_out_in_every_theme(cx: &mut TestAppContext) {
    let themes = dodai_core::switcher().themes().to_vec();
    assert!(themes.len() > 4, "the built-in themes");
    for theme in themes {
        cx.update(|cx| dodai_gpui::install(cx, theme.clone()));
        let (_, vcx) = cx.add_window_view(|_, _| Host);
        vcx.simulate_resize(size(px(800.0), px(6000.0)));
        vcx.run_until_parked();
        for (name, _) in charts() {
            let b = vcx
                .debug_bounds(name)
                .unwrap_or_else(|| panic!("{name} under {}: not laid out", theme.name()));
            let (w, h) = (f32::from(b.size.width), f32::from(b.size.height));
            assert!(
                (w - 320.0).abs() < 1.0 && h > 20.0 && h.is_finite(),
                "{name} under {}: {w} x {h}",
                theme.name()
            );
        }
    }
}
