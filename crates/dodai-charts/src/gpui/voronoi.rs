//! [`VoronoiChart`]: the plane split into one cell per point, each holding what is nearest it.

use crate::gpui::common::{chart_cfg, fmt_with, plot_box, tracked_canvas, CanvasText, NumFmt};
use crate::gpui::focus::{readout, Focus};
use crate::gpui::grid::heat_color_with;
use crate::voronoi::{self, Raster, Site, Spec, NONE};
use crate::Gradient;
use dodai_gpui::paint::{fill, fill_circle, fill_rect, scale_alpha, stroke, stroke_circle};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

pub use crate::voronoi::Metric as VoronoiMetric;

/// One point of a [`VoronoiChart`]: a label at `(x, y)`, in a group that sets its colour.
#[derive(Clone, Debug)]
pub struct VoronoiPoint {
    label: SharedString,
    x: f32,
    y: f32,
    group: usize,
    weight: Option<f32>,
    value: Option<f32>,
    category: Option<usize>,
}

impl VoronoiPoint {
    /// The point `label` at `(x, y)`, in group 0.
    pub fn new(label: impl Into<SharedString>, x: f32, y: f32) -> Self {
        Self {
            label: label.into(),
            x,
            y,
            group: 0,
            weight: None,
            value: None,
            category: None,
        }
    }
    /// The series colour the point's cell takes.
    pub fn group(mut self, g: usize) -> Self {
        self.group = g;
        self
    }
    /// The point's weight: a heavier point owns a larger cell when the chart is `weighted`.
    pub fn weight(mut self, w: f32) -> Self {
        self.weight = Some(w);
        self
    }
    /// A value the cell is coloured by when the chart has a `gradient` (a heatmap).
    pub fn value(mut self, v: f32) -> Self {
        self.value = Some(v);
        self
    }
    /// The class of the point: same-class cells merge into a territory and `knn` votes use it.
    pub fn category(mut self, c: usize) -> Self {
        self.category = Some(c);
        self
    }
}

#[derive(Clone, Copy)]
struct Cfg {
    links: bool,
    labels: bool,
    borders: bool,
    show_sites: bool,
    shade: bool,
    metric: voronoi::Metric,
    weighted: bool,
    farthest: bool,
    knn: usize,
    relax: usize,
    merge: bool,
    clip_circle: bool,
    phase: f32,
    gradient: Option<Gradient>,
    probe: Option<(f32, f32)>,
    ring: bool,
}

impl Default for Cfg {
    fn default() -> Self {
        Self {
            links: false,
            labels: true,
            borders: true,
            show_sites: true,
            shade: false,
            metric: voronoi::Metric::Euclidean,
            weighted: false,
            farthest: false,
            knn: 1,
            relax: 0,
            merge: true,
            clip_circle: false,
            phase: 0.0,
            gradient: None,
            probe: None,
            ring: false,
        }
    }
}

/// Voronoi diagram: the plot is split into one cell per point, holding every place nearer to
/// that point than to any other. Hover anywhere to snap to the nearest point and read it.
///
/// A plain diagram draws exact polygons. The variants below are computed as a pixel field:
/// [`gradient`](Self::gradient) (a heatmap by point value), [`category`](VoronoiPoint::category)
/// territories with [`merge`](Self::merge)d borders, [`knn`](Self::knn) regions,
/// [`metric`](Self::metric)s, [`farthest`](Self::farthest) site, power diagrams
/// ([`weighted`](Self::weighted)), Lloyd [`relax`](Self::relax)ing, a [`probe`](Self::probe)
/// site, a [`clip_circle`](Self::clip_circle) disc, [`ring`](Self::ring) pies and
/// [`stipple`](Self::stipple) mosaics.
#[derive(IntoElement)]
pub struct VoronoiChart {
    follow: dodai_gpui::follow::Follow,
    points: Vec<VoronoiPoint>,
    cfg: Cfg,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(VoronoiChart);

macro_rules! setter {
    ($(#[$m:meta])* $name:ident, $field:ident: $ty:ty) => {
        $(#[$m])*
        pub fn $name(mut self, v: $ty) -> Self {
            self.cfg.$field = v;
            self
        }
    };
}

impl VoronoiChart {
    dodai_gpui::follow_api!();
    /// Create a new [`VoronoiChart`] of `points`.
    #[track_caller]
    pub fn new(points: impl IntoIterator<Item = VoronoiPoint>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            points: points.into_iter().collect(),
            cfg: Cfg::default(),
            height: 420.0,
            fmt: None,
            title: None,
        }
    }
    /// A pie drawn as a Voronoi diagram: one weighted site per `(label, value)` on a ring,
    /// inside a disc, so the wedges follow the values.
    #[track_caller]
    pub fn ring<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32)>) -> Self {
        let points = items
            .into_iter()
            .enumerate()
            .map(|(i, (l, v))| {
                VoronoiPoint::new(l, 0.0, 0.0)
                    .weight(v)
                    .value(v)
                    .category(i)
            })
            .collect::<Vec<_>>();
        let mut c = Self::new(points);
        c.cfg.ring = true;
        c.cfg.weighted = true;
        c.cfg.clip_circle = true;
        c.cfg.merge = false;
        c
    }
    /// A mosaic: `n` points sampled from `density[row][col]` (row 0 on top), relaxed `relax`
    /// times so they crowd where the density is high, each coloured by the density there.
    #[track_caller]
    pub fn stipple(density: &[Vec<f32>], n: usize, relax: usize, seed: u64) -> Self {
        let pts = voronoi::stipple(density, n, relax, seed)
            .into_iter()
            .map(|(x, y, d)| VoronoiPoint::new("", x, y).value(d))
            .collect::<Vec<_>>();
        let mut c = Self::new(pts);
        c.cfg.gradient = Some(Gradient::Viridis);
        c.cfg.labels = false;
        c.cfg.show_sites = false;
        c.cfg.borders = false;
        c
    }
    setter!(
        /// Draw the Delaunay links between neighbouring points (default: off).
        links, links: bool
    );
    setter!(
        /// Name each point on the plot (default: on).
        labels, labels: bool
    );
    setter!(
        /// Draw the borders between cells and territories (default: on).
        borders, borders: bool
    );
    setter!(
        /// Draw a dot on each point (default: on).
        show_sites, show_sites: bool
    );
    setter!(
        /// Fade each pixel with its distance from its point (default: off).
        shade_by_distance, shade: bool
    );
    setter!(
        /// The distance that decides the nearest point (default: Euclidean).
        metric, metric: voronoi::Metric
    );
    setter!(
        /// Heavier points own larger cells: a power diagram (default: off).
        weighted, weighted: bool
    );
    setter!(
        /// Each spot takes its farthest point instead of its nearest (default: off).
        farthest, farthest: bool
    );
    setter!(
        /// Colour each spot by the majority category among its `k` nearest points.
        knn, knn: usize
    );
    setter!(
        /// Lloyd steps: points move to their cells' centres (up to 30), evening the cells out.
        relax, relax: usize
    );
    setter!(
        /// Same-category cells share one region with no border between them (default: on).
        merge, merge: bool
    );
    setter!(
        /// Fill only the inscribed disc (default: off).
        clip_circle, clip_circle: bool
    );
    setter!(
        /// Drift the points around their places; 0 leaves them (default: 0).
        phase, phase: f32
    );
    /// Colour each cell by its point's value along `g`.
    pub fn gradient(mut self, g: Gradient) -> Self {
        self.cfg.gradient = Some(g);
        self
    }
    /// A temporary extra site at the data position `(x, y)`, like a cursor.
    pub fn probe(mut self, x: f32, y: f32) -> Self {
        self.cfg.probe = Some((x, y));
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

/// The sites of the field, in canvas pixels.
fn site_list(points: &[VoronoiPoint], cfg: &Cfg, w: f32, h: f32) -> Vec<Site> {
    if cfg.ring {
        let v: Vec<f32> = points.iter().map(|p| p.weight.unwrap_or(0.0)).collect();
        return voronoi::ring_sites(&v, w, h);
    }
    sites(points, w, h)
        .into_iter()
        .zip(points)
        .map(|((x, y), p)| Site {
            x,
            y,
            weight: p.weight,
            category: p.category,
        })
        .collect()
}

/// What the pure layer is asked for. Positions are in raster pixels: canvas pixels over `cell`.
fn spec_of(points: &[VoronoiPoint], cfg: &Cfg, w: f32, h: f32, cell: f32) -> Spec {
    let (x0, y0, sx, sy) = frame(points);
    Spec {
        metric: cfg.metric,
        weighted: cfg.weighted,
        farthest: cfg.farthest,
        knn: cfg.knn,
        relax: cfg.relax,
        merge: cfg.merge,
        clip_circle: cfg.clip_circle,
        phase: cfg.phase,
        probe: cfg
            .probe
            .map(|(x, y)| ((x - x0) / sx * w / cell, (h - (y - y0) / sy * h) / cell)),
    }
}

/// Side of one raster pixel in canvas pixels. Runs of one owner paint as one quad, so a fine
/// cell is cheap and the edges stay smooth; a distance fade paints every pixel, so it is coarser.
fn cell_of(_w: f32, _h: f32, shade: bool) -> f32 {
    if shade {
        4.0
    } else {
        2.0
    }
}

thread_local! {
    static LAST: RefCell<Option<(String, Rc<Raster>)>> = const { RefCell::new(None) };
}

/// The field for a `w`×`h` plot, kept from the last call when nothing changed (the canvas and
/// the hover test both ask every frame).
fn field(points: &[VoronoiPoint], cfg: &Cfg, w: f32, h: f32) -> (Rc<Raster>, f32) {
    let cell = cell_of(w, h, cfg.shade);
    let (rw, rh) = ((w / cell).ceil().max(1.0), (h / cell).ceil().max(1.0));
    let list: Vec<Site> = site_list(points, cfg, w, h)
        .into_iter()
        .map(|s| Site {
            x: s.x / cell,
            y: s.y / cell,
            ..s
        })
        .collect();
    let spec = spec_of(points, cfg, w, h, cell);
    let key = format!("{list:?}|{spec:?}|{rw}x{rh}");
    let hit = LAST.with(|l| {
        l.borrow()
            .as_ref()
            .filter(|(k, _)| *k == key)
            .map(|(_, r)| r.clone())
    });
    let r = hit.unwrap_or_else(|| {
        let r = Rc::new(voronoi::raster(&list, rw as usize, rh as usize, &spec));
        LAST.with(|l| *l.borrow_mut() = Some((key, r.clone())));
        r
    });
    (r, cell)
}

impl VoronoiChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "VoronoiChart");
        let n = self.points.len();
        let cfg = self.cfg;
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        // The value range a gradient spans.
        let (lo, hi) = crate::extent(self.points.iter().filter_map(|p| p.value))
            .map_or((0.0, 1.0), |(a, b)| (a, b.max(a + 1e-6)));
        let tone = |p: &VoronoiPoint, lift: f32| match (cfg.gradient, p.value) {
            (Some(gr), Some(v)) => {
                let c = heat_color_with(&t, gr, (v - lo) / (hi - lo));
                hsla(c.lighten(0.15 * lift))
            }
            _ => {
                let s = t.palette.series_at(p.category.unwrap_or(p.group));
                hsla(s.lighten(0.5 - 0.25 * lift))
            }
        };
        let fills: Vec<Hsla> = self.points.iter().map(|p| tone(p, 0.0)).collect();
        let fills_hot: Vec<Hsla> = self.points.iter().map(|p| tone(p, 1.0)).collect();
        let dots: Vec<Hsla> = self
            .points
            .iter()
            .map(|p| hsla(t.palette.series_at(p.category.unwrap_or(p.group))))
            .collect();
        let cat_fills: Vec<Hsla> = (0..16)
            .map(|c| hsla(t.palette.series_at(c).lighten(0.5)))
            .collect();
        let edge = hsla(t.on(t.palette.series_at(0).lighten(0.5)));
        let ink = hsla(t.palette.text);
        // labels sit on pastel cells, or on gradient colours that suit the page's text
        let label_ink = if cfg.gradient.is_some() { ink } else { edge };
        let page = hsla(t.palette.background);
        let size = t.typography.size_xs;
        let display = t.clone();
        let points = self.points.clone();
        let plain = !cfg.ring && !cfg.shade && spec_of(&points, &cfg, 1.0, 1.0, 1.0).is_plain();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_points = points.clone();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let a = g.min(1.0);
            let name = CanvasText::display(&display, size);
            // where each dot sits, and the edges between neighbours
            let (dot_at, link_at): Drawn = if plain {
                let s = sites(&paint_points, bw, bh);
                let d = voronoi::diagram(&s, bw, bh);
                for (k, cell) in d.cells.iter().enumerate().filter(|(_, c)| c.len() > 2) {
                    let c = if looks[k].lift > 0.01 {
                        fills_hot[k]
                    } else {
                        fills[k]
                    };
                    fill(w, b.origin, cell, scale_alpha(c, looks[k].alpha * a));
                    if cfg.borders {
                        stroke(w, b.origin, cell, 1.5, scale_alpha(edge, 0.6 * a), true);
                    }
                }
                (s, d.links)
            } else {
                let (r, cell) = field(&paint_points, &cfg, bw, bh);
                paint_raster(
                    w,
                    b.origin,
                    &r,
                    cell,
                    (bw, bh),
                    &Paint {
                        cfg: &cfg,
                        n,
                        fills: &fills,
                        fills_hot: &fills_hot,
                        cat_fills: &cat_fills,
                        looks: &looks,
                        edge,
                        ink,
                        a,
                    },
                );
                if cfg.clip_circle {
                    let (cx0, cy0, rr) = (bw / 2.0, bh / 2.0, bw.min(bh) / 2.0);
                    stroke_circle(w, b.origin, (cx0, cy0), rr, 1.5, scale_alpha(edge, 0.6 * a));
                }
                let real = r.sites.len() - usize::from(cfg.probe.is_some());
                let at: Vec<(f32, f32)> =
                    r.sites.iter().map(|p| (p.0 * cell, p.1 * cell)).collect();
                let links = r
                    .links
                    .iter()
                    .filter(|(_, c)| *c >= 2)
                    .map(|(p, _)| *p)
                    .filter(|(i, j)| *i < real && *j < real)
                    .collect();
                if cfg.probe.is_some() {
                    if let Some(p) = at.last() {
                        stroke_circle(w, b.origin, *p, 7.0, 2.0, scale_alpha(ink, a));
                        fill_circle(w, b.origin, *p, 2.5, scale_alpha(ink, a));
                    }
                }
                (at, links)
            };
            if cfg.links {
                for &(i, j) in &link_at {
                    stroke(
                        w,
                        b.origin,
                        &[dot_at[i], dot_at[j]],
                        1.0,
                        scale_alpha(ink, 0.45 * a),
                        false,
                    );
                }
            }
            for (k, p) in dot_at.iter().enumerate().take(n) {
                let r = (3.5 + 2.5 * looks[k].lift) * a;
                if cfg.show_sites {
                    fill_circle(w, b.origin, *p, r, scale_alpha(dots[k], a));
                    stroke_circle(w, b.origin, *p, r, 1.5, scale_alpha(page, a));
                }
                if cfg.labels && g > 0.9 && !paint_points[k].label.is_empty() {
                    name.centred(
                        w,
                        cx,
                        &paint_points[k].label,
                        (b.origin, (p.0, p.1 - r - size)),
                        80.0,
                        scale_alpha(label_ink, looks[k].alpha),
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
            if plain {
                // every spot belongs to its nearest point
                voronoi::nearest(&sites(&hit_points, w, h), (x, y))
            } else {
                let (r, cell) = field(&hit_points, &cfg, w, h);
                r.owner_at(x / cell, y / cell)
                    .filter(|k| *k < hit_points.len())
            }
        });
        let line = focus.shown(cx).and_then(|k| points.get(k)).map(|p| {
            let at = format!(
                "{} · {}, {}",
                p.label,
                fmt_with(&self.fmt, p.x),
                fmt_with(&self.fmt, p.y)
            );
            match p.value {
                Some(v) => format!("{at} · {}", fmt_with(&self.fmt, v)),
                None => at,
            }
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

/// Where the dots sit, and which pairs of them to link.
type Drawn = (Vec<(f32, f32)>, Vec<(usize, usize)>);

/// What [`paint_raster`] colours with.
struct Paint<'a> {
    cfg: &'a Cfg,
    n: usize,
    fills: &'a [Hsla],
    fills_hot: &'a [Hsla],
    cat_fills: &'a [Hsla],
    looks: &'a [crate::gpui::focus::ItemFocus],
    edge: Hsla,
    ink: Hsla,
    a: f32,
}

/// Paints a field as runs of squares, then the borders where the region changes.
fn paint_raster(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    r: &Raster,
    cell: f32,
    (bw, bh): (f32, f32),
    p: &Paint,
) {
    for py in 0..r.h {
        let mut x = 0;
        while x < r.w {
            let i = py * r.w + x;
            let own = r.own[i];
            if own == NONE {
                x += 1;
                continue;
            }
            let cat = r.cat[i];
            let mut e = x + 1;
            if !p.cfg.shade {
                while e < r.w && r.own[py * r.w + e] == own && r.cat[py * r.w + e] == cat {
                    e += 1;
                }
            }
            let k = own as usize;
            let (c, alpha) = if cat != NONE {
                (p.cat_fills[cat as usize % p.cat_fills.len()], p.a)
            } else if k >= p.n {
                (p.ink, 0.3 * p.a)
            } else {
                let look = &p.looks[k];
                let c = if look.lift > 0.01 {
                    p.fills_hot[k]
                } else {
                    p.fills[k]
                };
                (c, look.alpha * p.a)
            };
            let fade = if p.cfg.shade {
                1.0 - 0.6 * (r.dist[i] / (r.spacing * 0.9).max(1.0)).clamp(0.0, 1.0)
            } else {
                1.0
            };
            let (x0, y0) = (x as f32 * cell, py as f32 * cell);
            let width = ((e - x) as f32 * cell).min(bw - x0);
            fill_rect(
                w,
                o,
                (x0, y0, width, cell.min(bh - y0)),
                0.0,
                scale_alpha(c, alpha * fade),
            );
            x = e;
        }
    }
    if !p.cfg.borders {
        return;
    }
    let line = scale_alpha(p.edge, 0.6 * p.a);
    for py in 0..r.h {
        for x in 0..r.w {
            let i = py * r.w + x;
            if r.own[i] == NONE {
                continue;
            }
            let g = r.group[i];
            if x + 1 < r.w && r.own[i + 1] != NONE && r.group[i + 1] != g {
                fill_rect(
                    w,
                    o,
                    ((x + 1) as f32 * cell - 0.75, py as f32 * cell, 1.5, cell),
                    0.0,
                    line,
                );
            }
            if py + 1 < r.h && r.own[i + r.w] != NONE && r.group[i + r.w] != g {
                fill_rect(
                    w,
                    o,
                    (x as f32 * cell, (py + 1) as f32 * cell - 0.75, cell, 1.5),
                    0.0,
                    line,
                );
            }
        }
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
