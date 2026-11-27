//! [`VoronoiChart`]: the plane split into one cell per point, each holding what is nearest it.

use crate::gpui::common::{chart_cfg, fmt_with, plot_box, tracked_canvas, CanvasText, NumFmt};
use crate::gpui::focus::{readout, Focus};
use crate::voronoi;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// One point of a [`VoronoiChart`]: a label at `(x, y)`, in a group that sets its colour.
#[derive(Clone, Debug)]
pub struct VoronoiPoint {
    label: SharedString,
    x: f32,
    y: f32,
    group: usize,
}

impl VoronoiPoint {
    /// The point `label` at `(x, y)`, in group 0.
    pub fn new(label: impl Into<SharedString>, x: f32, y: f32) -> Self {
        Self {
            label: label.into(),
            x,
            y,
            group: 0,
        }
    }
    /// The series colour the point's cell takes.
    pub fn group(mut self, g: usize) -> Self {
        self.group = g;
        self
    }
}

/// Voronoi diagram: the plot is split into one cell per point, holding every place nearer to
/// that point than to any other. Hover anywhere to snap to the nearest point and read it; click
/// to keep it picked. `links(true)` adds the Delaunay triangulation (points whose cells touch).
#[derive(IntoElement)]
pub struct VoronoiChart {
    follow: dodai_gpui::follow::Follow,
    points: Vec<VoronoiPoint>,
    links: bool,
    labels: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(VoronoiChart);

impl VoronoiChart {
    dodai_gpui::follow_api!();
    /// Create a new [`VoronoiChart`] of `points`.
    #[track_caller]
    pub fn new(points: impl IntoIterator<Item = VoronoiPoint>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            points: points.into_iter().collect(),
            links: false,
            labels: true,
            height: 420.0,
            fmt: None,
            title: None,
        }
    }
    /// Draw the Delaunay links between neighbouring points (default: off).
    pub fn links(mut self, on: bool) -> Self {
        self.links = on;
        self
    }
    /// Name each point on the plot (default: on).
    pub fn labels(mut self, on: bool) -> Self {
        self.labels = on;
        self
    }
}

/// The data's own range, padded, as `(min_x, min_y, span_x, span_y)`.
fn frame(points: &[VoronoiPoint]) -> (f32, f32, f32, f32) {
    let xs = crate::extent(points.iter().map(|p| p.x).filter(|v| v.is_finite()));
    let ys = crate::extent(points.iter().map(|p| p.y).filter(|v| v.is_finite()));
    let pad = |(lo, hi): (f32, f32)| {
        let span = (hi - lo).max(1e-6);
        (lo - span * 0.08, span * 1.16)
    };
    let (x0, sx) = pad(xs.unwrap_or((0.0, 1.0)));
    let (y0, sy) = pad(ys.unwrap_or((0.0, 1.0)));
    (x0, y0, sx, sy)
}

/// The points in canvas pixels for a `w`×`h` box; data `y` grows upward.
fn sites(points: &[VoronoiPoint], w: f32, h: f32) -> Vec<(f32, f32)> {
    let (x0, y0, sx, sy) = frame(points);
    points
        .iter()
        .map(|p| ((p.x - x0) / sx * w, h - (p.y - y0) / sy * h))
        .collect()
}

impl VoronoiChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "VoronoiChart");
        let n = self.points.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        let pastel = |gi: usize| t.palette.series_at(gi).lighten(0.5);
        let fills: Vec<Hsla> = self.points.iter().map(|p| hsla(pastel(p.group))).collect();
        let fills_hot: Vec<Hsla> = self
            .points
            .iter()
            .map(|p| hsla(t.palette.series_at(p.group).lighten(0.25)))
            .collect();
        let dots: Vec<Hsla> = self
            .points
            .iter()
            .map(|p| hsla(t.palette.series_at(p.group)))
            .collect();
        let edge = hsla(t.on(pastel(0)));
        let ink = hsla(t.palette.text);
        let page = hsla(t.palette.background);
        let size = t.typography.size_xs;
        let display = t.clone();
        let points = self.points.clone();
        let (links, labels) = (self.links, self.labels);
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_points = points.clone();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let a = g.min(1.0);
            let s = sites(&paint_points, bw, bh);
            let d = voronoi::diagram(&s, bw, bh);
            for (k, cell) in d.cells.iter().enumerate().filter(|(_, c)| c.len() > 2) {
                let tone = if looks[k].lift > 0.01 {
                    fills_hot[k]
                } else {
                    fills[k]
                };
                fill(w, b.origin, cell, scale_alpha(tone, looks[k].alpha * a));
                stroke(w, b.origin, cell, 1.5, scale_alpha(edge, 0.6 * a), true);
            }
            if links {
                for &(i, j) in &d.links {
                    stroke(
                        w,
                        b.origin,
                        &[s[i], s[j]],
                        1.0,
                        scale_alpha(ink, 0.45 * a),
                        false,
                    );
                }
            }
            let name = CanvasText::display(&display, size);
            for (k, p) in s.iter().enumerate() {
                if d.cells[k].is_empty() {
                    continue;
                }
                let r = (3.5 + 2.5 * looks[k].lift) * a;
                fill_circle(w, b.origin, *p, r, scale_alpha(dots[k], a));
                stroke_circle(w, b.origin, *p, r, 1.5, scale_alpha(page, a));
                if labels && g > 0.9 {
                    name.centred(
                        w,
                        cx,
                        &paint_points[k].label,
                        (b.origin, (p.0, p.1 - r - size)),
                        80.0,
                        scale_alpha(ink, looks[k].alpha),
                    );
                }
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("plot").element_id(window, cx),
            self.height,
            cv,
        );
        let hit_points = points.clone();
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            // every spot belongs to its nearest point
            voronoi::nearest(&sites(&hit_points, w, h), (x, y))
        });
        let line = focus.shown(cx).and_then(|k| points.get(k)).map(|p| {
            format!(
                "{} · {}, {}",
                p.label,
                fmt_with(&self.fmt, p.x),
                fmt_with(&self.fmt, p.y)
            )
        });
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, line, "Hover anywhere: the nearest point"));
        crate::gpui::common::titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(VoronoiChart);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_land_inside_the_box_with_y_up() {
        let pts = [
            VoronoiPoint::new("a", 0.0, 0.0),
            VoronoiPoint::new("b", 10.0, 10.0),
        ];
        let s = sites(&pts, 200.0, 100.0);
        assert!(s
            .iter()
            .all(|p| (0.0..=200.0).contains(&p.0) && (0.0..=100.0).contains(&p.1)));
        assert!(s[1].1 < s[0].1, "a larger y sits higher");
        assert!(s[1].0 > s[0].0);
    }

    #[test]
    fn one_point_or_none_do_not_break_the_frame() {
        assert_eq!(sites(&[], 10.0, 10.0).len(), 0);
        let s = sites(&[VoronoiPoint::new("a", 5.0, 5.0)], 10.0, 10.0);
        assert!(s[0].0.is_finite() && s[0].1.is_finite());
    }
}
