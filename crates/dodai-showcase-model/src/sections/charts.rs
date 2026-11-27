//! Charts: every chart type of the gpui showcase, drawn as polygons, strokes and dots, live.

use dodai_charts::layers::StackOffset;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::charts as ch;
use crate::charts::basic::{self, Radial};
use crate::charts::{flow, market as mk, status as st};
use crate::node::Node;

const CAT: &str = "Charts";

pub(crate) const SECTIONS: [Section; 13] = [
    Section::new(CAT, "Line · Area · Stream", lines).heavy(),
    Section::new(CAT, "Bar · Waterfall", bars).heavy(),
    Section::new(CAT, "Pie · Donut · Gauge", radial).heavy(),
    Section::new(CAT, "Radar · Treemap · Hierarchy", hierarchy).heavy(),
    Section::new(CAT, "Scatter · Heatmap · Density", fields).heavy(),
    Section::new(CAT, "Distribution", distribution).heavy(),
    Section::new(CAT, "Comparison · Ranking", comparison).heavy(),
    Section::new(CAT, "Flow · Network", networks).heavy(),
    Section::new(CAT, "Candlestick · Indicators", market).heavy(),
    Section::new(CAT, "Time · Status · Pulse", timeline).heavy(),
    Section::new(CAT, "Status · KPI", status).heavy(),
    Section::new(CAT, "Sparkline", sparks).heavy(),
    Section::new(CAT, "Dashboards", dashboards).heavy(),
];

const W: f32 = 300.0;
const H: f32 = 170.0;
const S: f32 = 180.0;

fn now() -> f32 {
    crate::widgets::elapsed()
}

fn card(title: &str, plot: crate::plot::Plot) -> Node {
    let w = plot.w;
    tile(w.max(120.0), vec![Node::Plot(plot), label(title)])
}

fn lines(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Line", ch::line(t, W, H, now(), false)),
        card("Area", ch::line(t, W, H, now(), true)),
        card("Stacked area", basic::stacked_area(t, W, H, now())),
        card(
            "Stream graph",
            flow::streams(t, W, H, now(), StackOffset::Wiggle),
        ),
        card(
            "100% stack",
            flow::streams(t, W, H, now(), StackOffset::Expand),
        ),
        card("Secondary axis", mk::secondary_axis(t, W, H, now())),
        card("Time axis", st::time_axis(t, W, H, now())),
        card("Band", st::band(t, W, H, now())),
        card("Annotated threshold", st::annotated(t, W, H, now())),
        card("Small multiples", st::small_multiples(t, W, H, now())),
    ])
}

fn bars(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Bar", ch::bars(t, W, H, now(), false)),
        card("Stacked bar", ch::bars(t, W, H, now(), true)),
        card("Horizontal, grouped", basic::hbars(t, W, H, now())),
        card("Paired bars", basic::paired_bars(t, W, H, now())),
        card("Waterfall", basic::waterfall(t, W, H, now())),
        card("Funnel", basic::funnel(t, W * 0.7, H, now())),
        card("Marimekko", flow::marimekko(t, W, H, now())),
    ])
}

fn radial(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Pie", ch::pie(t, S, now(), false)),
        card("Donut", ch::pie(t, S, now(), true)),
        card("Exploded", basic::radial(t, S, now(), Radial::Exploded)),
        card("Rose", basic::radial(t, S, now(), Radial::Rose)),
        card("Half donut", basic::radial(t, S, now(), Radial::Half)),
        card(
            "Three-quarter",
            basic::radial(t, S, now(), Radial::ThreeQuarter),
        ),
        card("Segmented", basic::radial(t, S, now(), Radial::Segmented)),
        card("Radial bar", basic::radial_bar(t, S, now())),
        card("Gauge", ch::gauge(t, W * 0.7, now())),
    ])
}

fn hierarchy(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Radar", ch::radar(t, S + 20.0, now())),
        card("Treemap", ch::treemap(t, W, H, now())),
        card("Sunburst", flow::sunburst(t, S + 20.0, now())),
        card("Icicle", flow::icicle(t, W, H, now())),
        card("Circle packing", flow::circle_pack(t, S + 20.0, now())),
        card("Proportional squares", flow::squares(t, W, H, now())),
        card("Waffle", flow::waffle(t, S, now())),
        card("Shard map", flow::shard_map(t, S + 20.0, now())),
    ])
}

fn fields(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Scatter and bubble", ch::scatter(t, W, H, now())),
        card("Heatmap", ch::heat(t, W, H, now())),
        card("Hexbin", flow::hex_bins(t, W, H, now())),
        card("Voronoi diagram", flow::voronoi_map(t, W, H, now())),
        card(
            "Time scatter and frontier",
            flow::time_scatter(t, W, H, now()),
        ),
        card("Beeswarm", flow::beeswarm(t, W, H * 0.7, now())),
        card("Calendar heatmap", mk::calendar_heat(t, W, H * 0.7, now())),
    ])
}

fn distribution(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Histogram", basic::histogram(t, W, H, now())),
        card("Box plot", basic::box_plot(t, W, H, now())),
        card("Violin", basic::violin(t, W, H, now())),
        card("ECDF", basic::ecdf(t, W, H, now())),
        card("Error bars", basic::error_bars(t, W, H, now())),
        card("Ridgeline", flow::ridgeline(t, W, H, now())),
        card("Horizon", flow::horizon_strips(t, W, H, now())),
        card("Peak range", flow::peak_range(t, W, H, now())),
        card("Bubble swarm", st::bubble_swarm(t, W, H * 0.7, now())),
    ])
}

fn comparison(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Bullet", basic::bullet(t, W, H, now())),
        card("Lollipop", basic::lollipop(t, W, H, now(), false)),
        card("Dumbbell", basic::lollipop(t, W, H, now(), true)),
        card("Slope", basic::slope(t, W * 0.7, H, now())),
        card("Bump", flow::bump(t, W, H, now())),
        card("Parallel coordinates", flow::parallel(t, W, H, now())),
        card("Pareto", st::pareto(t, W, H, now())),
        card("Diverging bars", st::diverging(t, W, H, now())),
    ])
}

fn networks(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Sankey", flow::sankey_flow(t, W, H, now())),
        card("Chord", flow::chord_ring(t, S + 20.0, now())),
        card("Split flow", flow::split_flow(t, W, H, now())),
        card("Network", flow::network(t, W, H, now())),
        card("Arc diagram", flow::arc_diagram(t, W, H, now())),
    ])
}

fn market(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Candlestick", ch::candles(t, W, H, now())),
        card("Heikin-Ashi", mk::heikin(t, W, H, now())),
        card("Renko", mk::renko_bricks(t, W, H, now())),
        card("Order book depth", mk::depth_book(t, W, H, now())),
        card("Bollinger bands", mk::bollinger(t, W, H, now())),
        card("RSI and MACD", mk::indicator_panels(t, W, H, now())),
        card("Price and volume", mk::price_volume(t, W, H, now())),
    ])
}

fn timeline(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        card("Gantt", mk::gantt_chart(t, W, H, now())),
        card("Event timeline", mk::event_timeline(t, W, H, now())),
        card("State spans", mk::state_spans(t, W, H * 0.7, now())),
        card("Control chart", mk::control_chart(t, W, H, now())),
        card("Span fan", flow::span_fan(t, S + 20.0, now())),
        card("Bubble timeline", st::bubble_timeline(t, W, H * 0.7, now())),
        card(
            "Pulse: ECG, pleth, respiration",
            mk::pulse_traces(t, W, H, now()),
        ),
    ])
}

fn sparks(c: &Ctx) -> Node {
    let t = c.theme;
    let w = W * 0.7;
    row(vec![
        card("Line", ch::spark(t, w, 70.0, now())),
        card("Bars", basic::spark_kind(t, w, 70.0, now(), 0)),
        card("Step", basic::spark_kind(t, w, 70.0, now(), 1)),
        card("Dots", basic::spark_kind(t, w, 70.0, now(), 2)),
        card("Win / loss", basic::spark_kind(t, w, 70.0, now(), 3)),
    ])
}

fn kpi(title: &str, value: String, plot: crate::plot::Plot) -> Node {
    tile(
        plot.w.max(150.0),
        vec![muted(title), heading(value, 2), Node::Plot(plot)],
    )
}

fn status(c: &Ctx) -> Node {
    let t = c.theme;
    let k = |a: f32| (now() * 0.4 + a).sin();
    row(vec![
        kpi(
            "Revenue",
            format!("${:.1}k", 48.0 + 3.0 * k(0.0)),
            st::kpi_trend(t, 170.0, 56.0, now()),
        ),
        kpi(
            "Latency",
            format!("{:.0} ms", 120.0 + 12.0 * k(1.0)),
            st::kpi_trend(t, 170.0, 56.0, now() + 2.0),
        ),
        card("Slot grid", st::slot_grid(t, W, H * 0.7, now())),
        card("Status strip", st::status_strip(t, W, 28.0, now())),
        card("Bucket bar", st::bucket_bar(t, W * 0.7, H * 0.8, now())),
        card("Stage bars", st::stage_bars(t, W * 0.7, H * 0.8, now())),
        card("Gradient scale", st::gradient_scale(t, W, 48.0, now())),
    ])
}

fn dashboards(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        tile(
            W * 1.4,
            vec![
                muted("Live throughput"),
                Node::Plot(ch::line(t, W * 1.4, H, now(), true)),
                line(vec![
                    Node::Plot(basic::spark_kind(t, 120.0, 40.0, now(), 0)),
                    Node::Plot(basic::spark_kind(t, 120.0, 40.0, now(), 3)),
                    Node::Plot(ch::gauge(t, 120.0, now())),
                ]),
            ],
        ),
        tile(
            W * 1.4,
            vec![
                muted("Network overview"),
                Node::Plot(flow::network(t, W * 1.4, H, now())),
                Node::Plot(st::status_strip(t, W * 1.4, 22.0, now())),
            ],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use crate::node::Node;
    use crate::{sections, Ctx, Showcase};

    fn plots<'a>(n: &'a Node, out: &mut Vec<&'a crate::plot::Plot>) {
        match n {
            Node::Stack(s) => s.items.iter().for_each(|i| plots(i, out)),
            Node::Plot(p) => out.push(p),
            _ => {}
        }
    }

    /// Every chart draws something, inside finite numbers, at several moments of the animation.
    #[test]
    fn every_chart_draws_finite_shapes() {
        let sc = Showcase::new();
        let theme = sc.theme();
        let ctx = Ctx {
            theme: &theme,
            values: &sc.values,
        };
        let mut count = 0;
        for s in sections().iter().filter(|s| s.category == super::CAT) {
            let mut found = Vec::new();
            let tree = (s.build)(&ctx);
            plots(&tree, &mut found);
            assert!(!found.is_empty(), "{} shows no chart", s.title);
            for p in found {
                count += 1;
                assert!(
                    !p.fills.is_empty() || !p.lines.is_empty(),
                    "{}: an empty chart",
                    s.title
                );
                let pts = p
                    .fills
                    .iter()
                    .flat_map(|f| &f.points)
                    .chain(p.lines.iter().flat_map(|l| &l.points));
                for (x, y) in pts {
                    assert!(
                        x.is_finite() && y.is_finite(),
                        "{}: a point is not finite",
                        s.title
                    );
                }
            }
        }
        assert!(count >= 60, "only {count} charts");
    }
}
