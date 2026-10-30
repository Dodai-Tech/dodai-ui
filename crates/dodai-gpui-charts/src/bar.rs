//! [`BarChart`].

use crate::common::*;
use dodai_gpui::paint::{shaped, ShapeStyle};
use dodai_gpui::panel::DodaiTooltip;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, relative, App, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── BarChart ───────────────────────────────

/// Bar chart: vertical or horizontal, grouped or stacked.
#[derive(IntoElement)]
pub struct BarChart {
    follow: dodai_gpui::follow::Follow,
    annotations: Vec<crate::annotate::Annotation>,
    id: SharedString,
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    horizontal: bool,
    stacked: bool,
    legend: bool,
    toggleable: bool,
    y_max: Option<f32>,
    fmt: Option<NumFmt>,
    value_labels: bool,
    bar_width: f32,
    title: Option<SharedString>,
    axes: crate::common::AxisTitles,
    corners: Option<dodai_core::Corners>,
}

dodai_gpui::corners_setter!(BarChart);

impl BarChart {
    dodai_gpui::follow_api!();
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(mut self, a: impl IntoIterator<Item = crate::annotate::Annotation>) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    /// Create a new [`BarChart`].
    pub fn new(id: impl Into<SharedString>, series: impl IntoIterator<Item = Series>) -> Self {
        let id: SharedString = id.into();
        Self {
            follow: dodai_gpui::follow::Follow::named(&id),
            annotations: Vec::new(),
            corners: None,
            id,
            series: series.into_iter().collect(),
            labels: Vec::new(),
            height: 200.0,
            horizontal: false,
            stacked: false,
            legend: true,
            toggleable: false,
            y_max: None,
            fmt: None,
            value_labels: false,
            bar_width: 32.0,
            title: None,
            axes: crate::common::AxisTitles::default(),
        }
    }
    /// Fix the value-axis maximum (default: fit data).
    pub fn max(mut self, m: f32) -> Self {
        self.y_max = Some(m);
        self
    }
    /// Custom number formatter for axis labels, tooltips and value labels.
    pub fn format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.fmt = Some(Rc::new(f));
        self
    }
    /// Print the value above each bar (vertical, grouped only).
    pub fn value_labels(mut self, v: bool) -> Self {
        self.value_labels = v;
        self
    }
    /// Maximum bar thickness in px (default 32).
    pub fn bar_width(mut self, w: f32) -> Self {
        self.bar_width = w;
        self
    }
    /// Title above the chart.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// Name of the horizontal axis, centred under the plot (e.g. "Seconds").
    pub fn x_title(mut self, t: impl Into<SharedString>) -> Self {
        self.axes.x = Some(t.into());
        self
    }
    /// Name of the vertical axis, over its value labels (e.g. a unit, "Mbps").
    pub fn y_title(mut self, t: impl Into<SharedString>) -> Self {
        self.axes.y = Some(t.into());
        self
    }
    /// Labels, one per item.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = l.into_iter().map(Into::into).collect();
        self
    }
    /// Height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Lay out horizontally.
    pub fn horizontal(mut self, h: bool) -> Self {
        self.horizontal = h;
        self
    }
    /// Stack the series instead of grouping them.
    pub fn stacked(mut self, s: bool) -> Self {
        self.stacked = s;
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
    /// Clicking a legend entry hides or shows that series; the axis refits to what is left.
    pub fn toggleable(mut self, on: bool) -> Self {
        self.toggleable = on;
        self
    }
}

/// Max of per-category totals (stacked) or single values (grouped). Pure.
pub fn bar_max(series: &[Series], stacked: bool) -> f32 {
    let n = series.iter().map(|s| s.values.len()).max().unwrap_or(0);
    (0..n)
        .map(|i| {
            let it = series
                .iter()
                .map(|s| s.values.get(i).copied().unwrap_or(0.0).max(0.0));
            if stacked {
                it.sum::<f32>()
            } else {
                it.fold(0.0, f32::max)
            }
        })
        .fold(0.0, f32::max)
}

impl BarChart {
    /// One bar of series `si` in category `i`, `frac` of its cell long.
    fn bar(
        &self,
        t: &dodai_core::Theme,
        si: usize,
        i: usize,
        v: f32,
        frac: f32,
    ) -> gpui::Stateful<gpui::Div> {
        let s = &self.series[si];
        let cs = t.chart_style();
        let ink = cs.mark_outline;
        let style = ShapeStyle::from_tokens(t, Some(series_color(t, s, si)), ink, false, false);
        let label = self.labels.get(i).cloned().unwrap_or_default();
        let el = div()
            .id((SharedString::from(format!("{}-{si}-{i}", self.id)), i))
            .tooltip(DodaiTooltip::text(format!(
                "{} {label}: {}",
                s.name,
                fmt_with(&self.fmt, v)
            )))
            .child(shaped(&t.shape, style).size_full());
        if self.horizontal {
            el.h_full().w(relative(frac))
        } else {
            el.w_full().h(relative(frac))
        }
    }

    /// One category: its bar, or a bar per series (or a stack of them).
    fn cell(&self, t: &dodai_core::Theme, i: usize, bar_top: f32) -> gpui::Div {
        let h = self.horizontal;
        let mut cell = div()
            .flex_1()
            .size_full()
            .flex()
            .gap(px(t.spacing.xs / 2.0))
            .p(px(t.spacing.xs));
        if h {
            // Bars grow right; stacked bars lay segments left→right.
            cell = cell.items_center();
            if self.stacked {
                let mut stack = div().h_full().max_h(px(t.sizes().chart_bar)).flex();
                for (si, s) in self.series.iter().enumerate() {
                    let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                    stack = stack.child(self.bar(t, si, i, v, v / bar_top.max(f32::EPSILON)));
                }
                cell = cell.child(
                    div()
                        .h_full()
                        .w_full()
                        .flex()
                        .items_center()
                        .child(stack.w_full()),
                );
                cell = cell.flex_col();
            } else {
                cell = cell.flex_col().justify_center();
                for (si, s) in self.series.iter().enumerate() {
                    let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                    cell = cell.child(
                        div()
                            .w_full()
                            .flex_1()
                            .min_h_0()
                            .max_h(px(t.sizes().chart_cell))
                            .child(self.bar(t, si, i, v, norm(v, 0.0, bar_top))),
                    );
                }
            }
        } else {
            cell = cell.items_end().justify_center();
            if self.stacked {
                let total: f32 = self
                    .series
                    .iter()
                    .map(|s| s.values.get(i).copied().unwrap_or(0.0).max(0.0))
                    .sum();
                let mut stack = div()
                    .h(relative(norm(total, 0.0, bar_top)))
                    .w_full()
                    .max_w(px(self.bar_width * 1.5))
                    .flex()
                    .flex_col_reverse();
                for (si, s) in self.series.iter().enumerate() {
                    let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                    stack = stack.child(
                        self.bar(t, si, i, v, v / total.max(f32::EPSILON))
                            .h(relative(v / total.max(f32::EPSILON))),
                    );
                }
                cell = cell.child(stack);
            } else {
                for (si, s) in self.series.iter().enumerate() {
                    let v = s.values.get(i).copied().unwrap_or(0.0).max(0.0);
                    cell = cell.child(
                        div()
                            .h_full()
                            .flex_1()
                            .max_w(px(self.bar_width))
                            .flex()
                            .flex_col()
                            .justify_end()
                            .children(self.value_labels.then(|| {
                                label_style(t, div())
                                    .flex()
                                    .justify_center()
                                    .child(fmt_with(&self.fmt, v))
                            }))
                            .child(
                                div()
                                    .w_full()
                                    .h(relative(norm(v, 0.0, bar_top)))
                                    .child(self.bar(t, si, i, v, 1.0)),
                            ),
                    );
                }
            }
        }
        cell
    }

    /// The plot area: gridlines behind the bars.
    fn plot(&self, t: &dodai_core::Theme, ticks: &[f32], top: f32, cats: gpui::Div) -> gpui::Div {
        let h = self.horizontal;
        let cs = t.chart_style();
        let grid = hsla(cs.grid_color);
        // Gridlines behind the bars (vertical orientation: horizontal lines).
        let grid_fracs: Vec<f32> = ticks
            .iter()
            .map(|v| norm(*v, 0.0, top))
            .filter(|f| *f <= 1.0)
            .collect();
        let mut plot = div().relative().w_full().h(px(self.height));
        if cs.grid_dashed {
            let (fr, style) = (grid_fracs.clone(), cs);
            plot = plot.child(
                gpui::canvas(
                    |_, _, _| (),
                    move |b: gpui::Bounds<gpui::Pixels>, _, w, _| {
                        let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                        for f in &fr {
                            let (x, y) = (bw * f, bh * (1.0 - f));
                            let (p, q) = if h {
                                ((x, 0.0), (x, bh))
                            } else {
                                ((0.0, y), (bw, y))
                            };
                            crate::common::grid_line(w, b.origin, p, q, &style);
                        }
                    },
                )
                .absolute()
                .size_full(),
            );
        }
        for f in grid_fracs.iter().filter(|_| !cs.grid_dashed) {
            let line = div().absolute().bg(grid);
            plot = plot.child(if h {
                line.left(relative(*f))
                    .top_0()
                    .h_full()
                    .w(px(cs.grid_width))
            } else {
                line.bottom(relative(*f))
                    .left_0()
                    .w_full()
                    .h(px(cs.grid_width))
            });
        }
        if !h {
            let n = self
                .series
                .iter()
                .map(|s| s.values.len())
                .max()
                .unwrap_or(0);
            plot = plot.child(crate::annotate::layer(
                t,
                &self.annotations,
                (0.0, top),
                &crate::annotate::category_x(n),
            ));
        }
        plot = plot.child(cats);
        plot
    }

    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = dodai_gpui::apply_corners(t, self.corners);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "BarChart");
        self.series = crate::common::follow_series(window, cx, &self.follow, &self.series);
        let mut legend_all = None;
        if self.toggleable {
            let all = std::mem::take(&mut self.series);
            let (visible, h, pinned) = crate::common::toggled(window, cx, &self.id, all);
            self.series = visible;
            legend_all = Some((pinned, h));
        }
        let ticks = nice_ticks(
            0.0,
            self.y_max
                .unwrap_or_else(|| bar_max(&self.series, self.stacked)),
            4,
        );
        let top = *ticks.last().unwrap_or(&1.0);
        let (_, top) =
            dodai_gpui::follow::follow_range(window, cx, &self.follow, "top", (0.0, top));
        let n = self
            .series
            .iter()
            .map(|s| s.values.len())
            .max()
            .unwrap_or(0);
        let h = self.horizontal;
        let _grid = hsla(t.palette.border);

        let bar_top = top / g.max(1e-3);
        let mut cats = div().flex().size_full();
        cats = if h { cats.flex_col() } else { cats };
        let fid = format!("{}-focus", self.id);
        let focus =
            crate::focus::Focus::new(window, cx, &dodai_gpui::follow::Follow::named(&fid), n);
        for i in 0..n {
            let f = focus.item(i);
            let item = div()
                .id((SharedString::from(fid.clone()), i))
                .flex_1()
                .size_full()
                // Without these the cells' automatic minimum (their full height) overflows the
                // plot when categories stack (horizontal bars).
                .min_h_0()
                .min_w_0()
                .relative()
                .top(px(-4.0 * f.lift))
                .opacity(f.alpha)
                .child(self.cell(&t, i, bar_top));
            cats = cats.child(focus.bind(item, i));
        }

        // A live window slides left between samples: the cells sit in a clipped strip one slot
        // wider than the plot, shifted by the scroll fraction.
        let scroll = self.series.iter().find_map(crate::common::Series::scroll);
        if scroll.is_some() {
            dodai_gpui::follow::next_frame(window);
        }
        let cats = match scroll {
            Some(sc) if !h && n > 1 => {
                let visible = (n - 1) as f32;
                div().absolute().size_full().overflow_hidden().child(
                    div()
                        .absolute()
                        .top_0()
                        .h_full()
                        .w(relative(n as f32 / visible))
                        .left(relative(-sc / visible))
                        .child(cats),
                )
            }
            _ => cats,
        };
        let plot = self.plot(&t, &ticks, top, cats);

        let labels: Vec<SharedString> = (0..n)
            .map(|i| self.labels.get(i).cloned().unwrap_or_default())
            .collect();
        let body = if h {
            let ycol = label_style(&t, div())
                .w(px(t.sizes().axis_gutter_lg))
                .h(px(self.height))
                .flex()
                .flex_col()
                .children(labels.into_iter().map(|l| {
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pr(px(t.spacing.xs))
                        .child(l)
                }));
            let xrow = label_style(&t, div())
                .ml(px(t.sizes().axis_gutter_lg))
                .flex()
                .justify_between()
                .children(ticks.iter().map(|v| div().child(fmt_with(&self.fmt, *v))));
            div()
                .flex()
                .flex_col()
                .w_full()
                .child(div().flex().child(ycol).child(div().flex_1().child(plot)))
                .child(xrow)
        } else {
            framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                labels,
                plot,
            )
        };
        let gutter = if self.horizontal {
            t.sizes().axis_gutter_lg
        } else {
            t.sizes().axis_gutter
        };
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(self.axes.around(&t, gutter, body));
        if let (true, Some((pinned, h))) = (self.legend, legend_all) {
            if pinned.len() > 1 {
                root =
                    root.child(Legend::of_series(&t, &pinned).toggle(format!("{}-lg", self.id), h));
            }
        } else if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        titled(&t, self.title.clone(), root)
    }
}

dodai_gpui::entry_render!(BarChart);
