//! Scatter plot and heatmap.

use crate::common::*;
use dodai_core::{Color, Theme};
use dodai_gpui::paint::{scale_alpha, shaped, ShapeStyle};
use dodai_gpui::panel::DodaiTooltip;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── ScatterPlot ───────────────────────────────

/// Scatter plot of `(x, y)` points per series, with hover tooltips.
#[derive(IntoElement)]
pub struct ScatterPlot {
    follow: dodai_gpui::follow::Follow,
    annotations: Vec<crate::annotate::Annotation>,
    id: SharedString,
    series: Vec<(SharedString, Vec<(f32, f32)>)>,
    height: f32,
    legend: bool,
}

impl ScatterPlot {
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(mut self, a: impl IntoIterator<Item = crate::annotate::Annotation>) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    dodai_gpui::follow_api!();
    /// Create a new [`ScatterPlot`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        series: impl IntoIterator<Item = (S, Vec<(f32, f32)>)>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            annotations: Vec::new(),
            id: SharedString::default(),
            series: series.into_iter().map(|(n, p)| (n.into(), p)).collect(),
            height: 200.0,
            legend: true,
        }
    }
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl ScatterPlot {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.id = self.follow.element_id(window, cx).to_string().into();
        let flat: Vec<f32> = self
            .series
            .iter()
            .flat_map(|s| s.1.iter().flat_map(|p| [p.0, p.1]))
            .collect();
        let mut shown = dodai_gpui::follow::follow_all(window, cx, &self.follow, &flat).into_iter();
        for p in self.series.iter_mut().flat_map(|s| s.1.iter_mut()) {
            p.0 = shown.next().unwrap_or(p.0);
            p.1 = shown.next().unwrap_or(p.1);
        }
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "ScatterPlot");
        let t = theme_of(cx);
        let all = || self.series.iter().flat_map(|s| s.1.iter());
        // The finite extent; no points (or none finite) give a 0..1 axis.
        let fold = |f: fn(&(f32, f32)) -> f32| {
            let (l, h) = all()
                .map(f)
                .filter(|v| v.is_finite())
                .fold((f32::INFINITY, f32::NEG_INFINITY), |(l, h), v| {
                    (l.min(v), h.max(v))
                });
            if l <= h {
                (l, h)
            } else {
                (0.0, 1.0)
            }
        };
        let (xl, xh) = fold(|p| p.0);
        let (yl, yh) = fold(|p| p.1);
        let xt = nice_ticks(xl, xh, 4);
        let yt = nice_ticks(yl, yh, 4);
        // `nice_ticks` always gives at least two.
        let (x0, x1) = (xt[0], xt[xt.len() - 1]);
        let (y0, y1) = (yt[0], yt[yt.len() - 1]);
        let grid = hsla(t.palette.border);
        let yf: Vec<f32> = yt.iter().map(|v| norm(*v, y0, y1)).collect();
        let xf: Vec<f32> = xt.iter().map(|v| norm(*v, x0, x1)).collect();
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &yf {
                    stroke(
                        w,
                        b.origin,
                        &[(0.0, bh - f * bh), (bw, bh - f * bh)],
                        1.0,
                        grid,
                        false,
                    );
                }
                for f in &xf {
                    stroke(
                        w,
                        b.origin,
                        &[(f * bw, 0.0), (f * bw, bh)],
                        1.0,
                        grid,
                        false,
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let mut plot = div().relative().w_full().h(px(self.height)).child(cvs);
        plot = plot.child(crate::annotate::layer(
            &t,
            &self.annotations,
            (y0, y1),
            &|v: f32| norm(v, x0, x1),
        ));

        for (si, (name, pts)) in self.series.iter().enumerate() {
            let c = hsla(t.palette.series_at(si));
            for (i, (x, y)) in pts.iter().enumerate() {
                let gi = dodai_gpui::motion::reveal(g, i, pts.len());
                plot = plot.child(
                    div()
                        .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
                        .absolute()
                        .left(relative(norm(*x, x0, x1)))
                        .top(relative(1.0 - norm(*y, y0, y1)))
                        .ml(px(-t.sizes().scatter_dot / 2.0 * gi))
                        .mt(px(-t.sizes().scatter_dot / 2.0 * gi))
                        .w(px(t.sizes().scatter_dot * gi))
                        .h(px(t.sizes().scatter_dot * gi))
                        .rounded_full()
                        .bg(scale_alpha(c, 0.85))
                        .hover(move |d| d.bg(c))
                        .tooltip(DodaiTooltip::text(format!(
                            "{name}: ({}, {})",
                            fmt_num(*x),
                            fmt_num(*y)
                        ))),
                );
            }
        }
        let xl: Vec<SharedString> = xt.iter().map(|v| fmt_num(*v).into()).collect();
        let mut root = div().flex().flex_col().gap(px(t.spacing.sm)).w_full();
        root = root.child(framed(
            &t,
            yt.iter().map(|v| fmt_num(*v)).collect(),
            xl,
            plot,
        ));
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::new(
                self.series
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (s.0.clone(), t.palette.series_at(i))),
            ));
        }
        root
    }
}

dodai_gpui::entry_render!(ScatterPlot);

// ─────────────────────────────── Heatmap ───────────────────────────────

/// Colour of heat `v` (`0..=1`): surface → primary. Pure.
pub fn heat_color(t: &Theme, v: f32) -> Color {
    t.palette
        .surface_raised
        .mix(t.palette.primary, v.clamp(0.0, 1.0))
}

/// Like [`heat_color`] but with a selectable gradient. Pure.
pub fn heat_color_with(t: &Theme, g: dodai_charts_core::Gradient, v: f32) -> Color {
    match g.sample(v) {
        Some([r, gr, b]) => Color::rgba(r, gr, b, 255),
        None => heat_color(t, v),
    }
}

/// Grid of cells coloured by value, with hover tooltips.
#[derive(IntoElement)]
pub struct Heatmap {
    follow: dodai_gpui::follow::Follow,
    id: SharedString,
    rows: Vec<Vec<f32>>,
    row_labels: Vec<SharedString>,
    col_labels: Vec<SharedString>,
    cell: f32,
    stretch: bool,
    gradient: dodai_charts_core::Gradient,
}

impl Heatmap {
    dodai_gpui::follow_api!();
    /// Create a new [`Heatmap`].
    #[track_caller]
    pub fn new(rows: Vec<Vec<f32>>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            id: SharedString::default(),
            rows,
            row_labels: Vec::new(),
            col_labels: Vec::new(),
            cell: 28.0,
            stretch: false,
            gradient: dodai_charts_core::Gradient::Theme,
        }
    }
    /// Row labels.
    pub fn row_labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.row_labels = l.into_iter().map(Into::into).collect();
        self
    }
    /// Column labels.
    pub fn col_labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.col_labels = l.into_iter().map(Into::into).collect();
        self
    }
    /// Let cells grow to fill the full width instead of capping at 4× the cell size.
    pub fn stretch(mut self, on: bool) -> Self {
        self.stretch = on;
        self
    }
    /// Cell size in px.
    pub fn cell(mut self, s: f32) -> Self {
        self.cell = s;
        self
    }
    /// Colour scale; `Gradient::Theme` (default) mixes surface → primary.
    pub fn gradient(mut self, g: dodai_charts_core::Gradient) -> Self {
        self.gradient = g;
        self
    }
}

impl Heatmap {
    /// Dense grid (many cells): painted on one canvas with a hover readout
    /// instead of one element per cell.
    fn build_dense(mut self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        self.id = self.follow.element_id(window, cx).to_string().into();
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "Heatmap");
        let t = theme_of(cx);
        let (lo, hi) = self
            .rows
            .iter()
            .flatten()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let cell = self.cell;
        let (nr, nc) = (
            self.rows.len(),
            self.rows.iter().map(Vec::len).max().unwrap_or(0),
        );
        let hov = window.use_keyed_state(
            SharedString::from(format!("{}-hov", self.id)),
            cx,
            |_, _| None::<(usize, usize)>,
        );
        let cur = *hov.read(cx);
        let colors: Rc<Vec<gpui::Hsla>> = Rc::new(
            self.rows
                .iter()
                .flat_map(|r| {
                    r.iter()
                        .map(|v| hsla(heat_color_with(&t, self.gradient, norm(*v, lo, hi))))
                })
                .collect(),
        );
        let lens: Rc<Vec<usize>> = Rc::new(self.rows.iter().map(Vec::len).collect());
        let outline = hsla(t.palette.text);
        let hov_w = hov.clone();
        let canvas = gpui::canvas(
            |_, _, _| (),
            move |b: gpui::Bounds<gpui::Pixels>, _, w, _| {
                let cw = f32::from(b.size.width) / nc.max(1) as f32;
                let mut i = 0;
                for r in 0..nr {
                    for c in 0..lens[r] {
                        let o = b.origin + gpui::point(px(c as f32 * cw), px(r as f32 * cell));
                        w.paint_quad(gpui::fill(
                            gpui::Bounds::new(o, gpui::size(px(cw), px(cell))),
                            colors[i].opacity(dodai_gpui::motion::reveal(g, c, nc)),
                        ));
                        i += 1;
                    }
                }
                if let Some((r, c)) = cur {
                    let o = b.origin + gpui::point(px(c as f32 * cw), px(r as f32 * cell));
                    w.paint_quad(gpui::outline(
                        gpui::Bounds::new(o, gpui::size(px(cw), px(cell))),
                        outline,
                        gpui::BorderStyle::Solid,
                    ));
                }
                let st = hov_w.clone();
                w.on_mouse_event(move |e: &gpui::MouseMoveEvent, _, w, cx| {
                    let cw = f32::from(b.size.width) / nc.max(1) as f32;
                    let next = b.contains(&e.position).then(|| {
                        let d = e.position - b.origin;
                        (
                            ((f32::from(d.y) / cell) as usize).min(nr.saturating_sub(1)),
                            ((f32::from(d.x) / cw) as usize).min(nc.saturating_sub(1)),
                        )
                    });
                    if *st.read(cx) != next {
                        st.update(cx, |v, _| *v = next);
                        w.refresh();
                    }
                });
            },
        )
        .flex_1()
        .min_w_0()
        .h(px(nr as f32 * cell));
        let readout = match cur {
            Some((r, c)) => format!(
                "{} · {} = {}",
                self.row_labels
                    .get(r)
                    .filter(|l| !l.is_empty())
                    .map_or(format!("row {r}"), |l| l.to_string()),
                self.col_labels
                    .get(c)
                    .filter(|l| !l.is_empty())
                    .map_or(format!("col {c}"), |l| l.to_string()),
                fmt_num(self.rows[r][c])
            ),
            None => "hover a cell".to_string(),
        };
        let mut cols = div().flex().flex_col();
        if !self.col_labels.is_empty() {
            let mut hdr = div().flex().items_center().h(px(t.sizes().chart_cell));
            if !self.row_labels.is_empty() {
                hdr = hdr.child(div().w(px(t.sizes().axis_label)).flex_none());
            }
            // Painted, so a label wider than its column shows whole (neighbours make room).
            cols = cols.child(
                hdr.child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .child(slot_labels(&t, self.col_labels.clone())),
                ),
            );
        }
        let mut body = div().flex();
        if !self.row_labels.is_empty() {
            body = body.child(
                label_style(&t, div())
                    .w(px(t.sizes().axis_label))
                    .flex_none()
                    .children(
                        self.row_labels
                            .iter()
                            .map(|l| div().h(px(cell)).whitespace_nowrap().child(l.clone())),
                    ),
            );
        }
        cols.child(body.child(canvas))
            .child(
                label_style(&t, div())
                    .pt(px(t.spacing.sm - t.spacing.xs / 2.0))
                    .child(readout),
            )
            .into_any_element()
    }

    fn build(mut self, window: &mut Window, cx: &mut App) -> gpui::AnyElement {
        dodai_gpui::follow::follow_rows(window, cx, &self.follow, &mut self.rows);
        let cells: usize = self.rows.iter().map(Vec::len).sum();
        if cells > 200 {
            return self.build_dense(window, cx);
        }
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "HeatmapGrid");
        self.build_grid(cx, g).into_any_element()
    }

    fn build_grid(self, cx: &mut App, g: f32) -> impl IntoElement {
        let t = theme_of(cx);
        let (lo, hi) = self
            .rows
            .iter()
            .flatten()
            .fold((f32::MAX, f32::MIN), |(l, h), v| (l.min(*v), h.max(*v)));
        let cell = self.cell;
        let max_w = if self.stretch { f32::MAX } else { cell * 4.0 };
        let mut root = div().flex().flex_col().gap(px(t.spacing.xs / 2.0));
        if !self.col_labels.is_empty() {
            let mut hdr = label_style(&t, div()).flex().gap(px(t.spacing.xs / 2.0));
            if !self.row_labels.is_empty() {
                hdr = hdr.child(div().w(px(t.sizes().axis_label)));
            }
            root = root.child(hdr.children(self.col_labels.iter().map(|l| {
                div()
                    .flex_1()
                    .min_w_0()
                    .max_w(px(max_w))
                    .flex()
                    .justify_center()
                    .child(l.clone())
            })));
        }
        let waves = self.rows.len() + self.rows.first().map_or(0, Vec::len);
        for (r, row) in self.rows.iter().enumerate() {
            let mut line = div().flex().gap(px(t.spacing.xs / 2.0)).items_center();
            if let Some(l) = self.row_labels.get(r) {
                line = line.child(
                    label_style(&t, div())
                        .w(px(t.sizes().axis_label))
                        .child(l.clone()),
                );
            } else if !self.row_labels.is_empty() {
                line = line.child(div().w(px(t.sizes().axis_label)));
            }
            for (c, v) in row.iter().enumerate() {
                let color = heat_color_with(&t, self.gradient, norm(*v, lo, hi));
                let style = ShapeStyle::from_tokens(&t, Some(color), None, false, false);
                line = line.child(
                    div()
                        .id((SharedString::from(format!("{}-{r}", self.id)), c))
                        .flex_1()
                        .min_w_0()
                        .max_w(px(max_w))
                        .h(px(cell))
                        .opacity(dodai_gpui::motion::reveal(g, r + c, waves))
                        .tooltip(DodaiTooltip::text(fmt_num(*v)))
                        .child(shaped(&t.shape, style).size_full()),
                );
            }
            root = root.child(line);
        }
        root
    }
}

dodai_gpui::entry_render!(Heatmap);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_runs_from_the_raised_surface_to_the_primary_and_clamps() {
        let t = dodai_core::themes::cyberpunk::yellow();
        assert_eq!(heat_color(&t, 0.0), t.palette.surface_raised);
        assert_eq!(heat_color(&t, 1.0), t.palette.primary);
        assert_eq!(heat_color(&t, -5.0), heat_color(&t, 0.0));
        assert_eq!(heat_color(&t, 9.0), heat_color(&t, 1.0));
    }
}
