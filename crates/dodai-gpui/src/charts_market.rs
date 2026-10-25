//! Market and planning charts: Renko, order-book depth, Gantt and a stack of
//! linked panes. The maths lives in `dodai-charts-core`; this only paints.

use crate::charts::{
    chart_cfg, fill, fmt_with, framed, glow_stroke, label_style, nice_ticks, norm, rect, stroke,
    titled, NumFmt,
};
use crate::paint::scale_alpha;
use crate::panel::DodaiTooltip;
use crate::{hsla, theme_of};
use dodai_charts_core::depth::{depth_curves, mid_price, Level};
use dodai_charts_core::gantt::{layout, Task};
use dodai_charts_core::renko::renko;
use gpui::{
    canvas, div, px, relative, AnyElement, App, Bounds, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── Renko ───────────────────────────────

/// Renko chart: fixed-size price bricks, time ignored.
#[derive(IntoElement)]
pub struct RenkoChart {
    id: SharedString,
    closes: Vec<f32>,
    brick: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(RenkoChart);

impl RenkoChart {
    /// Create a new [`RenkoChart`].
    pub fn new(
        id: impl Into<SharedString>,
        closes: impl IntoIterator<Item = f32>,
        brick: f32,
    ) -> Self {
        Self {
            id: id.into(),
            closes: closes.into_iter().collect(),
            brick,
            height: 200.0,
            fmt: None,
            title: None,
        }
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "RenkoChart");
        let bricks = renko(&self.closes, self.brick);
        let n = bricks.len();
        let lo = bricks.iter().map(|b| b.lo).fold(f32::INFINITY, f32::min);
        let hi = bricks
            .iter()
            .map(|b| b.hi)
            .fold(f32::NEG_INFINITY, f32::max);
        let (lo, hi) = if n == 0 { (0.0, 1.0) } else { (lo, hi) };
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let (up, down, grid) = (
            hsla(t.palette.tone(dodai_core::Tone::Success)),
            hsla(t.palette.tone(dodai_core::Tone::Danger)),
            hsla(t.palette.border),
        );
        let tf: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();
        let bs = bricks.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &tf {
                    let y = bh - f * bh;
                    stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                }
                let slot = bw / bs.len().max(1) as f32;
                for (i, k) in bs.iter().enumerate() {
                    let gi = crate::motion::reveal(g, i, bs.len());
                    let (yt, yb) = (bh - norm(k.hi, lo, hi) * bh, bh - norm(k.lo, lo, hi) * bh);
                    let h = (yb - yt) * gi;
                    let x = i as f32 * slot + slot * 0.08;
                    let c = if k.up { up } else { down };
                    fill(
                        w,
                        b.origin,
                        &rect(x, yb - h, slot * 0.84, h),
                        scale_alpha(c, 0.85),
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        let mut hit = div().absolute().size_full().flex();
        for (i, k) in bricks.iter().enumerate() {
            hit = hit.child(div().id((self.id.clone(), i)).flex_1().h_full().tooltip(
                DodaiTooltip::text(format!(
                    "{} {} → {}",
                    if k.up { "▲" } else { "▼" },
                    fmt_with(&self.fmt, k.lo),
                    fmt_with(&self.fmt, k.hi)
                )),
            ));
        }
        plot = plot.child(hit);
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            Vec::new(),
            plot,
        ));
        titled(&t, self.title, root)
    }
}

crate::entry_render!(RenkoChart);

// ─────────────────────────────── Depth ───────────────────────────────

/// Order-book depth: cumulative bid (green) and ask (red) step areas.
#[derive(IntoElement)]
pub struct DepthChart {
    glow: crate::charts::ChartGlow,
    id: SharedString,
    bids: Vec<Level>,
    asks: Vec<Level>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(DepthChart);

impl DepthChart {
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::charts::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// `(price, size)` levels in any order.
    pub fn new(
        id: impl Into<SharedString>,
        bids: impl IntoIterator<Item = Level>,
        asks: impl IntoIterator<Item = Level>,
    ) -> Self {
        Self {
            glow: crate::charts::ChartGlow::Theme,
            id: id.into(),
            bids: bids.into_iter().collect(),
            asks: asks.into_iter().collect(),
            height: 200.0,
            fmt: None,
            title: None,
        }
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "DepthChart");
        let (bc, ac) = depth_curves(&self.bids, &self.asks);
        let top = bc
            .last()
            .map_or(0.0, |p| p.1)
            .max(ac.last().map_or(0.0, |p| p.1));
        let ticks = nice_ticks(0.0, top, 4);
        let top = *ticks.last().unwrap_or(&top);
        let pmin = bc.last().or(ac.first()).map_or(0.0, |p| p.0);
        let pmax = ac.last().or(bc.first()).map_or(1.0, |p| p.0);
        let (up, down, grid) = (
            hsla(t.palette.tone(dodai_core::Tone::Success)),
            hsla(t.palette.tone(dodai_core::Tone::Danger)),
            hsla(t.palette.border),
        );
        let glow = self.glow.resolve(&t);
        let tf: Vec<f32> = ticks.iter().map(|v| norm(*v, 0.0, top)).collect();
        let curves = [(bc.clone(), up), (ac.clone(), down)];
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &tf {
                    let y = bh - f * bh;
                    stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                }
                for (c, col) in &curves {
                    // Step outline from the spread outward, then area to the floor.
                    let mut p: Vec<(f32, f32)> = Vec::new();
                    let mut prev = 0.0f32;
                    for (price, cum) in c {
                        let x = norm(*price, pmin, pmax) * bw;
                        p.push((x, bh - norm(prev, 0.0, top) * bh * g));
                        p.push((x, bh - norm(*cum, 0.0, top) * bh * g));
                        prev = *cum;
                    }
                    if p.len() < 2 {
                        continue;
                    }
                    for pair in p.windows(2) {
                        let (a, z) = (pair[0], pair[1]);
                        let quad = [a, z, (z.0, bh), (a.0, bh)];
                        if (a.0 - z.0).abs() > 0.01 {
                            fill(w, b.origin, &quad, scale_alpha(*col, 0.25));
                        }
                    }
                    glow_stroke(w, b.origin, &p, 2.0, *col, glow);
                }
            },
        )
        .absolute()
        .size_full();
        let mid = mid_price(&self.bids, &self.asks);
        let tip = format!(
            "bid {} · ask {} · mid {}",
            fmt_with(&self.fmt, bc.first().map_or(0.0, |p| p.0)),
            fmt_with(&self.fmt, ac.first().map_or(0.0, |p| p.0)),
            fmt_with(&self.fmt, mid.unwrap_or(0.0)),
        );
        let plot = div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv)
            .child(
                div()
                    .id(self.id.clone())
                    .absolute()
                    .size_full()
                    .tooltip(DodaiTooltip::text(tip)),
            );
        let xl: Vec<SharedString> = [pmin, (pmin + pmax) / 2.0, pmax]
            .iter()
            .map(|v| SharedString::from(fmt_with(&self.fmt, *v)))
            .collect();
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| crate::charts::fmt_num(*v)).collect(),
            xl,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

crate::entry_render!(DepthChart);

// ─────────────────────────────── Gantt ───────────────────────────────

const ROW_H: f32 = 26.0;

/// Gantt timeline: one row per task, progress fill and dependency links.
#[derive(IntoElement)]
pub struct Gantt {
    id: SharedString,
    tasks: Vec<Task>,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    label_w: f32,
}

impl Gantt {
    /// Create a new [`Gantt`].
    pub fn new(id: impl Into<SharedString>, tasks: impl IntoIterator<Item = Task>) -> Self {
        Self {
            id: id.into(),
            tasks: tasks.into_iter().collect(),
            fmt: None,
            title: None,
            label_w: 110.0,
        }
    }
    /// Format task times in tooltips.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }
    /// Title text.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// Width of the task-name column.
    pub fn label_width(mut self, w: f32) -> Self {
        self.label_w = w;
        self
    }

    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "Gantt");
        let l = layout(&self.tasks);
        let n = l.bars.len();
        let (grid, text) = (hsla(t.palette.border), hsla(t.palette.text));
        let links = l.links.clone();
        let spans: Vec<(f32, f32)> = l.bars.iter().map(|b| (b.x, b.x + b.w)).collect();
        let starts: Vec<f32> = l.bars.iter().map(|b| b.x).collect();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let bw: f32 = b.size.width.into();
                for (from, to) in &links {
                    let (x0, y0) = (spans[*from].1 * bw, *from as f32 * ROW_H + ROW_H / 2.0);
                    let (x1, y1) = (starts[*to] * bw, *to as f32 * ROW_H + ROW_H / 2.0);
                    let mx = x0 + 6.0;
                    stroke(
                        w,
                        b.origin,
                        &[(x0, y0), (mx, y0), (mx, y1), (x1, y1)],
                        1.0,
                        scale_alpha(text, 0.6),
                        false,
                    );
                }
                for i in 0..=4 {
                    let x = bw * i as f32 / 4.0;
                    stroke(
                        w,
                        b.origin,
                        &[(x, 0.0), (x, n as f32 * ROW_H)],
                        1.0,
                        grid,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut tracks = div()
            .relative()
            .flex_1()
            .min_w(px(120.0))
            .h(px(n as f32 * ROW_H))
            .child(cv);
        let mut names = label_style(&t, div()).w(px(self.label_w)).flex().flex_col();
        for (i, (task, bar)) in self.tasks.iter().zip(&l.bars).enumerate() {
            let c = t.palette.series[i % t.palette.series.len()];
            let c = hsla(c);
            let reveal = crate::motion::reveal(g, i, n);
            names = names.child(
                div()
                    .h(px(ROW_H))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .child(task.name.clone()),
            );
            let tip = format!(
                "{}: {} → {} ({}%)",
                task.name,
                fmt_with(&self.fmt, task.start),
                fmt_with(&self.fmt, task.end),
                (task.progress * 100.0).round()
            );
            let done = if bar.w > 0.0 { bar.done / bar.w } else { 0.0 };
            tracks = tracks.child(
                div()
                    .id((self.id.clone(), i))
                    .absolute()
                    .top(px(i as f32 * ROW_H + 5.0))
                    .h(px(ROW_H - 10.0))
                    .left(relative(bar.x))
                    .w(relative((bar.w * reveal).max(0.004)))
                    .bg(scale_alpha(c, 0.35))
                    .border_1()
                    .border_color(c)
                    .tooltip(DodaiTooltip::text(tip))
                    .child(div().h_full().w(relative(done)).bg(c)),
            );
        }
        let body = div().flex().w_full().child(names).child(tracks);
        let root = div().flex().flex_col().w_full().child(body);
        titled(&t, self.title, root)
    }
}

crate::entry_render!(Gantt);

// ─────────────────────────────── ChartStack ───────────────────────────────

type PaneFn = Box<dyn FnOnce(f32) -> AnyElement>;

/// Vertical stack of chart panes sharing one total height, split by weight
/// (price on top, volume and indicators below). Each pane builds its chart
/// for the pixel height it is given, so y-gutters line up.
#[derive(IntoElement)]
pub struct ChartStack {
    panes: Vec<(f32, PaneFn)>,
    height: f32,
    gap: f32,
}

impl Default for ChartStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartStack {
    /// Create a new [`ChartStack`].
    pub fn new() -> Self {
        Self {
            panes: Vec::new(),
            height: 360.0,
            gap: 8.0,
        }
    }
    /// Add a pane of relative `weight`; `build` receives its height in px.
    pub fn pane<E: IntoElement>(
        mut self,
        weight: f32,
        build: impl FnOnce(f32) -> E + 'static,
    ) -> Self {
        self.panes
            .push((weight, Box::new(move |h| build(h).into_any_element())));
        self
    }
    /// Total height of the stack in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Gap in px.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }
}

impl RenderOnce for ChartStack {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let weights: Vec<f32> = self.panes.iter().map(|p| p.0).collect();
        let slots = dodai_charts_core::stack::split(self.height, &weights, self.gap);
        let mut col = div().flex().flex_col().gap(px(self.gap)).w_full();
        for ((_, build), (_, h)) in self.panes.into_iter().zip(slots) {
            col = col.child(div().w_full().h(px(h)).child(build(h)));
        }
        col
    }
}
