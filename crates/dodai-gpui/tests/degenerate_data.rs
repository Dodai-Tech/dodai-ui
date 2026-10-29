//! Charts must survive hostile data (empty, single point, NaN, ±inf, all
//! equal, negative, ragged) without panicking, under a chamfered and a
//! rounded theme.

use dodai_gpui::*;
use gpui::{div, prelude::*, px, size, AnyElement, Context, TestAppContext, Window};

fn datasets() -> Vec<(&'static str, Vec<f32>)> {
    vec![
        ("empty", vec![]),
        ("single", vec![5.0]),
        ("zeros", vec![0.0; 6]),
        ("equal", vec![3.0; 6]),
        ("nan", vec![1.0, f32::NAN, 3.0, 2.0]),
        ("inf", vec![1.0, f32::INFINITY, f32::NEG_INFINITY, 2.0]),
        ("negative", vec![-4.0, -1.0, -9.0, -2.0]),
        ("huge", vec![1e30, 2e30, -1e30]),
    ]
}

fn charts(name: &'static str, v: Vec<f32>) -> Vec<AnyElement> {
    let s = || Series::new(name, v.clone());
    let mut out: Vec<AnyElement> = vec![
        LineChart::new(format!("l-{name}"), [s()]).into_any_element(),
        BarChart::new(format!("b-{name}"), [s(), s()]).into_any_element(),
        Heatmap::new(
            format!("h-{name}"),
            vec![v.clone(), v.iter().take(2).copied().collect()],
        )
        .into_any_element(),
        Histogram::new(format!("hi-{name}"), v.clone()).into_any_element(),
        RadarChart::new(["a", "b", "c"], [s()]).into_any_element(),
        Gauge::new(v.first().copied().unwrap_or(0.0))
            .range(0.0, v.get(1).copied().unwrap_or(0.0))
            .into_any_element(),
        PieChart::new(v.iter().enumerate().map(|(i, x)| (format!("s{i}"), *x))).into_any_element(),
        Funnel::new(v.iter().enumerate().map(|(i, x)| (format!("s{i}"), *x))).into_any_element(),
        Waterfall::new(
            format!("w-{name}"),
            v.iter().enumerate().map(|(i, x)| (format!("s{i}"), *x)),
        )
        .into_any_element(),
        Treemap::new(
            format!("t-{name}"),
            v.iter().enumerate().map(|(i, x)| (format!("s{i}"), *x)),
        )
        .into_any_element(),
        StackedAreaChart::new([s(), s()]).into_any_element(),
        LineChart::new(format!("tx-{name}"), [s()])
            .time_axis(v.iter().map(|x| f64::from(*x) * 1e9))
            .crosshair(true)
            .into_any_element(),
        LineChart::new(format!("ty-{name}"), [s()])
            .time_axis([f64::NAN, 1.0])
            .utc_offset(-12_345)
            .into_any_element(),
        RadialBar::new(
            v.iter()
                .enumerate()
                .map(|(i, x)| (format!("r{i}"), *x, 10.0)),
        )
        .into_any_element(),
        Candlestick::new(
            format!("c-{name}"),
            v.windows(2)
                .map(|w| dodai_charts_core::candles::Candle::new(w[0], w[1], w[0], w[1])),
        )
        .into_any_element(),
    ];
    for st in GaugeStyle::ALL {
        out.push(
            Gauge::new(v.first().copied().unwrap_or(0.0))
                .style(st)
                .zones(true)
                .into_any_element(),
        );
    }
    for st in [
        PieStyle::Solid,
        PieStyle::Segmented,
        PieStyle::Hud,
        PieStyle::Rose,
    ] {
        out.push(
            PieChart::new(v.iter().enumerate().map(|(i, x)| (format!("s{i}"), *x)))
                .style(st)
                .into_any_element(),
        );
    }
    for o in [
        StackOffset::Zero,
        StackOffset::Expand,
        StackOffset::Silhouette,
        StackOffset::Wiggle,
    ] {
        out.push(
            StackedAreaChart::new([s(), s()])
                .offset(o)
                .into_any_element(),
        );
    }
    let rev: Vec<f32> = v.iter().rev().copied().collect();
    out.push(
        LineChart::new(format!("band-{name}"), [s().band(v.clone(), rev.clone())])
            .x_title("x")
            .y_title("y")
            .into_any_element(),
    );
    out.push(
        BumpChart::new([s(), Series::new("r", rev.clone())])
            .labels(["only"])
            .into_any_element(),
    );
    out.push(RidgelineChart::new([(name, v.clone()), ("r", rev)]).into_any_element());
    out.push(HorizonChart::new([s(), s()]).bands(4).into_any_element());
    out.push(
        Bullet::new("", v.first().copied().unwrap_or(0.0))
            .target(v.last().copied().unwrap_or(0.0))
            .nice_scale(true)
            .into_any_element(),
    );
    for g in [SparkGuide::Mean, SparkGuide::Peak, SparkGuide::At(f32::NAN)] {
        for st in [SparkStyle::Line, SparkStyle::Bars] {
            out.push(
                Sparkline::new(v.clone())
                    .guide(g)
                    .style(st)
                    .into_any_element(),
            );
        }
    }
    out.push(Sparkline::new(v).into_any_element());
    out
}

struct Page;

impl Render for Page {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut root = div().flex().flex_wrap().gap(px(8.0)).size_full();
        for (name, v) in datasets() {
            for c in charts(name, v) {
                root = root.child(div().w(px(260.0)).h(px(180.0)).child(c));
            }
        }
        root
    }
}

#[gpui::test]
fn charts_survive_degenerate_data(cx: &mut TestAppContext) {
    for id in ["cyberpunk", "material"] {
        let theme = dodai_core::switcher()
            .themes()
            .iter()
            .find(|t| t.id() == id)
            .cloned()
            .expect("built-in theme");
        cx.update(|cx| install(cx, theme.clone()));
        let (_, vcx) = cx.add_window_view(|_, _| Page);
        vcx.simulate_resize(size(px(1600.0), px(1200.0)));
        vcx.run_until_parked();
    }
}

struct Controls;

impl Render for Controls {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let bad = [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, -5.0, 1e30, 0.0];
        let mut root = div().flex().flex_wrap().gap(px(8.0)).size_full();
        for (i, v) in bad.into_iter().enumerate() {
            root = root
                .child(ProgressBar::new(v))
                .child(Slider::new(("sl", i), v))
                .child(Knob::new(("kn", i), v))
                .child(CircularProgress::new(v))
                .child(RangeSlider::new(("rs", i), v, v));
        }
        root
            // empty / out-of-range structural inputs
            .child(Slider::new("sl-eq", 5.0).range(5.0, 5.0))
            .child(Slider::new("sl-inv", 5.0).range(9.0, 1.0))
            .child(Knob::new("kn-eq", 2.0).range(2.0, 2.0))
            .child(Pagination::new("pg0", 0, 0))
            .child(Pagination::new("pg1", 99, 3))
            .child(Stepper::new(["a", "b"], 9))
            .child(Stepper::new(Vec::<&str>::new(), 0))
            .child(Segmented::new("sg", Vec::<&str>::new()))
            .child(Carousel::new("car0"))
            .child(Carousel::new("car1").item("x").selected(7).visible(0))
            .child(Tabs::new("tabs"))
            .child(Accordion::new("acc").item("t", "b").expanded([5]))
            .child(Breadcrumb::new("bc", Vec::<&str>::new()))
            .child(AvatarGroup::new(Vec::<&str>::new()))
            .child(DataTable::new("dt0"))
            .child(
                DataTable::new("dt1")
                    .columns([Column::new("A")])
                    .rows([vec!["1", "2", "3"], vec![]]),
            )
            .child(Menu::new("m"))
            .child(TreeView::new("tv", vec![]))
    }
}

#[gpui::test]
fn controls_survive_degenerate_inputs(cx: &mut TestAppContext) {
    for theme in dodai_core::switcher().themes().to_vec() {
        cx.update(|cx| install(cx, theme));
        let (_, vcx) = cx.add_window_view(|_, _| Controls);
        vcx.simulate_resize(size(px(1200.0), px(900.0)));
        vcx.run_until_parked();
    }
}
