//! Radar (spider) chart.

use crate::common::*;
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement as _, IntoElement,
    ParentElement, Pixels, SharedString, Styled, Window,
};

// ─────────────────────────────── RadarChart ───────────────────────────────

/// Radar (spider) chart: one polygon per series over shared axes.
#[derive(IntoElement)]
pub struct RadarChart {
    follow: dodai_gpui::follow::Follow,
    glow: crate::common::ChartGlow,
    axes: Vec<SharedString>,
    series: Vec<Series>,
    size: f32,
    max: Option<f32>,
    legend: bool,
}

impl RadarChart {
    dodai_gpui::follow_api!();
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::common::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    #[track_caller]
    /// Create a new [`RadarChart`].
    pub fn new<S: Into<SharedString>>(
        axes: impl IntoIterator<Item = S>,
        series: impl IntoIterator<Item = Series>,
    ) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            glow: crate::common::ChartGlow::Theme,
            axes: axes.into_iter().map(Into::into).collect(),
            series: series.into_iter().collect(),
            size: 220.0,
            max: None,
            legend: true,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Maximum value.
    pub fn max(mut self, m: f32) -> Self {
        self.max = Some(m);
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

impl RadarChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "RadarChart");
        self.series = crate::common::follow_series(window, cx, &self.follow, &self.series);
        let t = theme_of(cx);
        let n = self.axes.len().max(3);
        let max = self.max.unwrap_or_else(|| {
            *nice_ticks(0.0, y_range(&self.series, true, None).1, 4)
                .last()
                .unwrap_or(&1.0)
        });
        let grid = hsla(t.palette.border);
        let style = t.chart_style();
        let glow = self.glow.resolve(&t);
        let size = self.size;
        let focus = crate::focus::Focus::new(window, cx, &self.follow, self.axes.len());
        let foc: Vec<crate::focus::ItemFocus> =
            (0..self.axes.len()).map(|i| focus.item(i)).collect();
        let foc_c = foc.clone();
        let polys: Vec<(Hsla, Vec<f32>)> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| (hsla(series_color(&t, s, i)), s.values.clone()))
            .collect();
        let cvs = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let c = size / 2.0;
                let r = c - 4.0;
                for ring in 1..=4 {
                    let rr = r * ring as f32 / 4.0;
                    for i in 0..n {
                        grid_line(
                            w,
                            b.origin,
                            polar(c, c, rr, i as f32 / n as f32),
                            polar(c, c, rr, (i + 1) as f32 / n as f32),
                            &style,
                        );
                    }
                }
                for i in 0..n {
                    let lf = foc_c.get(i).copied().unwrap_or(crate::focus::ItemFocus {
                        alpha: 1.0,
                        lift: 0.0,
                    });
                    stroke(
                        w,
                        b.origin,
                        &[(c, c), polar(c, c, r, i as f32 / n as f32)],
                        1.0 + 1.5 * lf.lift,
                        scale_alpha(grid, lf.alpha),
                        false,
                    );
                }
                for (col, vals) in &polys {
                    let p: Vec<_> = (0..n)
                        .map(|i| {
                            polar(
                                c,
                                c,
                                r * norm(vals.get(i).copied().unwrap_or(0.0), 0.0, max) * g,
                                i as f32 / n as f32,
                            )
                        })
                        .collect();
                    fill(w, b.origin, &p, scale_alpha(*col, 0.2));
                    let mut closed = p.clone();
                    closed.push(p[0]);
                    glow_stroke(w, b.origin, &closed, 2.0, *col, glow);
                    for (i, q) in p.iter().enumerate() {
                        let lf = foc_c.get(i).map_or(0.0, |f| f.lift);
                        if lf > 0.02 {
                            fill(
                                w,
                                b.origin,
                                &crate::common::marker_points(
                                    crate::common::Marker::Dot,
                                    q.0,
                                    q.1,
                                    2.0 + 2.5 * lf,
                                ),
                                *col,
                            );
                        }
                    }
                }
            },
        )
        .absolute()
        .size_full();
        // Axis labels positioned just outside the web.
        let mut web = div().relative().w(px(size)).h(px(size)).child(cvs);
        for (i, a) in self.axes.iter().enumerate() {
            let (x, y) = polar(0.5, 0.5, 0.5, i as f32 / n as f32);
            let lf = foc[i];
            web = web.child(
                focus
                    .bind(label_style(&t, div()).id(("radar-axis", i)), i)
                    .opacity(lf.alpha)
                    .absolute()
                    .left(relative(x))
                    .top(relative(y))
                    .child(a.clone()),
            );
        }
        let mut root = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .p(px(t.sizes().chart_pad))
            .child(web);
        if self.legend && self.series.len() > 1 {
            root = root.child(Legend::of_series(&t, &self.series));
        }
        root
    }
}

dodai_gpui::entry_render!(RadarChart);
