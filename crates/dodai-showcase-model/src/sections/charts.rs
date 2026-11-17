//! Charts: the common chart types drawn as polygons, strokes and dots, live. (The gpui showcase
//! has the full set of 60+; these are the ones every toolkit can draw from plain shapes.)

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::charts as ch;
use crate::node::Node;

const CAT: &str = "Charts";

pub(crate) const SECTIONS: [Section; 6] = [
    Section::new(CAT, "Line · Area", lines).animated(),
    Section::new(CAT, "Bar · Stacked bar", bars).animated(),
    Section::new(CAT, "Pie · Donut · Gauge", radial).animated(),
    Section::new(CAT, "Radar · Treemap", shapes).animated(),
    Section::new(CAT, "Scatter · Heatmap", fields).animated(),
    Section::new(CAT, "Candlestick · Sparkline", market).animated(),
];

const W: f32 = 340.0;
const H: f32 = 190.0;

fn now() -> f32 {
    crate::widgets::elapsed()
}

fn card(title: &str, plot: crate::plot::Plot) -> Node {
    tile(W, vec![Node::Plot(plot), label(title)])
}

fn lines(c: &Ctx) -> Node {
    line(vec![
        card("Line", ch::line(c.theme, W, H, now(), false)),
        card("Area", ch::line(c.theme, W, H, now(), true)),
    ])
}

fn bars(c: &Ctx) -> Node {
    line(vec![
        card("Bar", ch::bars(c.theme, W, H, now(), false)),
        card("Stacked bar", ch::bars(c.theme, W, H, now(), true)),
    ])
}

fn radial(c: &Ctx) -> Node {
    line(vec![
        card("Pie", ch::pie(c.theme, 190.0, now(), false)),
        card("Donut", ch::pie(c.theme, 190.0, now(), true)),
        card("Gauge", ch::gauge(c.theme, W * 0.7, now())),
    ])
}

fn shapes(c: &Ctx) -> Node {
    line(vec![
        card("Radar", ch::radar(c.theme, 200.0, now())),
        card("Treemap", ch::treemap(c.theme, W, H, now())),
    ])
}

fn fields(c: &Ctx) -> Node {
    line(vec![
        card("Scatter and bubble", ch::scatter(c.theme, W, H, now())),
        card("Heatmap", ch::heat(c.theme, W, H, now())),
    ])
}

fn market(c: &Ctx) -> Node {
    line(vec![
        card("Candlestick", ch::candles(c.theme, W, H, now())),
        card("Sparkline", ch::spark(c.theme, W * 0.7, 70.0, now())),
    ])
}
