//! Charts made of stacked rows: [`BumpChart`] (standings over time), [`RidgelineChart`]
//! (overlapping distributions) and [`HorizonChart`] (folded bands in short strips).

use crate::common::{
    chart_cfg, down_color, fill, fmt_with, label_column, label_style, labels_of, nice_ticks, norm,
    series_color, smooth_points, stroke, tick_row, titled, up_color, NumFmt, Series,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::{density, horizon, rank};
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::panel::DodaiTooltip;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// The height (0..=1) of density `d` at value `v`, linearly interpolated; 0 outside its range.
fn density_at(d: &density::Density, v: f32) -> f32 {
    if d.y.len() < 2
        || d.hi.partial_cmp(&d.lo) != Some(std::cmp::Ordering::Greater)
        || !(d.lo..=d.hi).contains(&v)
    {
        return 0.0;
    }
    let f = (v - d.lo) / (d.hi - d.lo) * (d.y.len() - 1) as f32;
    let i = (f as usize).min(d.y.len() - 2);
    let t = f - i as f32;
    d.y[i] * (1.0 - t) + d.y[i + 1] * t
}

/// The ridge under `(x, y)` in a `w` x `h` ridgeline of `curves` over `lo..hi`, ridges
/// `rise` rows tall: the front-most ridge whose shape covers the point, else the row the
/// point is in.
fn ridge_hit(
    curves: &[density::Density],
    (lo, hi): (f32, f32),
    rise: f32,
    (x, y, w, h): (f32, f32, f32, f32),
) -> Option<usize> {
    let m = curves.len();
    if m == 0 || w <= 0.0 || h <= 0.0 || hi.partial_cmp(&lo) != Some(std::cmp::Ordering::Greater) {
        return None;
    }
    let row = h / (m as f32 - 1.0 + rise);
    let v = lo + x / w * (hi - lo);
    let base = |k: usize| (k as f32 + rise) * row;
    (0..m)
        .rev()
        .find(|&k| y <= base(k) && y >= base(k) - density_at(&curves[k], v) * row * rise)
        .or_else(|| (0..m).find(|&k| y <= base(k) && y > base(k) - row))
}

/// A label column `w` px wide with one row label per `(text, y fraction from the top)`.
fn row_labels(
    t: &dodai_core::Theme,
    w: f32,
    h: f32,
    rows: impl IntoIterator<Item = (SharedString, f32)>,
    end: bool,
) -> gpui::Div {
    let line = (t.typography.size_xs * 1.5).ceil();
    let mut col = label_style(t, div())
        .relative()
        .w(px(w))
        .flex_none()
        .h(px(h));
    for (text, y) in rows {
        let cell = div()
            .absolute()
            .top(relative(y))
            .mt(px(-line / 2.0))
            .w_full()
            .h(px(line))
            .flex()
            .items_center()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_color(hsla(t.palette.text))
            .child(text);
        col = col.child(if end { cell.justify_end() } else { cell });
    }
    col
}

// ─────────────────────────────── BumpChart ───────────────────────────────

/// Bump chart: every series' rank (1 = highest value) at each step, so overtakes read at a
/// glance. Names sit at both ends, beside their first and last place.
#[derive(IntoElement)]
pub struct BumpChart {
    follow: dodai_gpui::follow::Follow,
    series: Vec<Series>,
    labels: Vec<SharedString>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(BumpChart);

impl BumpChart {
    dodai_gpui::follow_api!();
    /// Create a new [`BumpChart`] from the raw values; the chart ranks them per step.
    #[track_caller]
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        let series: Vec<Series> = series.into_iter().collect();
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            height: (series.len().max(2) as f32 * 30.0).max(120.0),
            series,
            labels: Vec::new(),
            fmt: None,
            title: None,
        }
    }
    /// Step labels under the chart, one per value.
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
}

impl BumpChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.series = crate::common::follow_series(window, cx, &self.follow, &self.series);
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "BumpChart");
        let rows: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let ranks = rank::ranks(&rows);
        let (m, n) = (self.series.len().max(1), ranks.first().map_or(0, Vec::len));
        let y_of = move |r: usize| (r as f32 + 0.5) / m as f32;
        let cols: Vec<Hsla> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| hsla(series_color(&t, s, i)))
            .collect();
        let paths: Vec<(Hsla, Vec<f32>)> = ranks
            .iter()
            .zip(&cols)
            .map(|(r, c)| (*c, r.iter().map(|r| y_of(*r)).collect()))
            .collect();
        let bg = hsla(t.palette.surface);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let x = |i: usize| (i as f32 + 0.5) / n.max(1) as f32 * bw;
                let mut shown = b;
                shown.size.width = b.size.width * g.clamp(0.0, 1.0);
                w.with_content_mask(Some(gpui::ContentMask { bounds: shown }), |w| {
                    for (c, ys) in &paths {
                        let p: Vec<(f32, f32)> =
                            ys.iter().enumerate().map(|(i, y)| (x(i), y * bh)).collect();
                        if p.len() > 1 {
                            stroke(w, b.origin, &smooth_points(&p, 10), 3.0, *c, false);
                        }
                        for &(px_, py) in &p {
                            let dot = crate::common::marker_points(
                                crate::common::Marker::Dot,
                                px_,
                                py,
                                5.5,
                            );
                            fill(w, b.origin, &dot, *c);
                            let hole = crate::common::marker_points(
                                crate::common::Marker::Dot,
                                px_,
                                py,
                                2.5,
                            );
                            fill(w, b.origin, &hole, bg);
                        }
                    }
                });
            },
        )
        .absolute()
        .size_full();
        // Hover a step to read every value in it.
        let mut plot = div().relative().flex_1().h(px(self.height)).child(cv);
        for i in 0..n {
            let tip = self
                .series
                .iter()
                .zip(&ranks)
                .filter_map(|(s, r)| {
                    let v = fmt_with(&self.fmt, *s.values.get(i)?);
                    Some((*r.get(i)?, (s.name.clone(), v)))
                })
                .collect::<std::collections::BTreeMap<_, _>>()
                .into_iter()
                .map(|(r, (name, v))| format!("{}. {name}: {v}", r + 1))
                .collect::<Vec<_>>()
                .join("\n");
            let head = self
                .labels
                .get(i)
                .map(|l| format!("{l}\n"))
                .unwrap_or_default();
            plot = plot.child(
                div()
                    .id(SharedString::from(format!("bump-{i}")))
                    .absolute()
                    .top_0()
                    .h_full()
                    .left(relative(i as f32 / n as f32))
                    .w(relative(1.0 / n as f32))
                    .tooltip(DodaiTooltip::text(format!("{head}{tip}"))),
            );
        }
        let gutter = label_column(&t, self.series.iter().map(|s| s.name.as_ref()));
        let ends = |at: usize| {
            ranks
                .iter()
                .zip(&self.series)
                .filter_map(|(r, s)| r.get(at).map(|r| (s.name.clone(), y_of(*r))))
                .collect::<Vec<_>>()
        };
        let body = div()
            .flex()
            .w_full()
            .gap(px(t.spacing.sm))
            .child(row_labels(&t, gutter, self.height, ends(0), true))
            .child(plot)
            .child(row_labels(
                &t,
                gutter,
                self.height,
                ends(n.saturating_sub(1)),
                false,
            ));
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(body);
        if self.labels.len() == n {
            root = root.child(
                label_style(&t, div())
                    .flex()
                    .mx(px(gutter + t.spacing.sm))
                    .children(
                        self.labels
                            .iter()
                            .map(|l| div().flex_1().flex().justify_center().child(l.clone())),
                    ),
            );
        }
        titled(&t, self.title.clone(), root)
    }
}

dodai_gpui::entry_render!(BumpChart);

// ─────────────────────────────── RidgelineChart ───────────────────────────────

/// Ridgeline: one smoothed distribution per group on a shared value axis, each row overlapping
/// the one above, so shifts between groups (months, versions, regions) read as a landscape.
#[derive(IntoElement)]
pub struct RidgelineChart {
    follow: dodai_gpui::follow::Follow,
    groups: Vec<(SharedString, Vec<f32>)>,
    overlap: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(RidgelineChart);

impl RidgelineChart {
    dodai_gpui::follow_api!();
    /// Create a new [`RidgelineChart`] from `(label, samples)` groups, drawn top to bottom.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(groups: impl IntoIterator<Item = (S, Vec<f32>)>) -> Self {
        let groups: Vec<(SharedString, Vec<f32>)> =
            groups.into_iter().map(|(l, v)| (l.into(), v)).collect();
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            height: (groups.len().max(2) as f32 * 34.0).max(140.0),
            groups,
            overlap: 1.8,
            fmt: None,
            title: None,
        }
    }
    /// How tall a ridge may rise, in row heights (default 1.8; 1 = no overlap).
    pub fn overlap(mut self, o: f32) -> Self {
        self.overlap = if o.is_finite() {
            o.clamp(0.5, 4.0)
        } else {
            1.8
        };
        self
    }
}

impl RidgelineChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "RidgelineChart");
        let curves: Vec<density::Density> = self
            .groups
            .iter()
            .map(|(_, v)| density::kde(v, 96))
            .collect();
        let live = curves.iter().filter(|d| !d.y.is_empty());
        let ticks = match dodai_charts_core::extent(live.flat_map(|d| [d.lo, d.hi])) {
            Some((lo, hi)) if hi > lo => nice_ticks(lo, hi, 5),
            _ => vec![0.0, 1.0],
        };
        let (lo, hi) = (
            *ticks.first().unwrap_or(&0.0),
            *ticks.last().unwrap_or(&1.0),
        );
        let m = self.groups.len().max(1);
        let overlap = self.overlap;
        let cols: Vec<Hsla> = (0..m).map(|i| hsla(t.palette.series_at(i))).collect();
        let (bg, grid) = (hsla(t.palette.surface), hsla(t.palette.border));
        let focus = Focus::new(window, cx, &self.follow, m);
        let looks: Vec<_> = (0..m).map(|k| focus.item(k)).collect();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let hit_curves = curves.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                // Room on top for the first ridge to rise into.
                let row = bh / (m as f32 - 1.0 + overlap);
                // The top row first, so each lower ridge covers the one behind it.
                for (k, d) in curves.iter().enumerate() {
                    let base = (k as f32 + overlap) * row;
                    stroke(w, b.origin, &[(0.0, base), (bw, base)], 1.0, grid, false);
                    if d.y.len() < 2 {
                        continue;
                    }
                    let span = (d.y.len() - 1) as f32;
                    let top: Vec<(f32, f32)> =
                        d.y.iter()
                            .enumerate()
                            .map(|(i, y)| {
                                let v = d.lo + (d.hi - d.lo) * i as f32 / span;
                                (norm(v, lo, hi) * bw, base - y * row * overlap * g)
                            })
                            .collect();
                    let mut poly = top.clone();
                    poly.push((top[top.len() - 1].0, base));
                    poly.push((top[0].0, base));
                    let lk = looks[k];
                    fill(w, b.origin, &poly, bg);
                    fill(w, b.origin, &poly, scale_alpha(cols[k], 0.55 * lk.alpha));
                    let c = scale_alpha(cols[k], lk.alpha.max(0.35));
                    stroke(w, b.origin, &top, 1.5 + lk.lift * 1.5, c, false);
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("ridges").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            ridge_hit(&hit_curves, (lo, hi), overlap, (x, y, w, h))
        });
        let text = focus.shown(cx).and_then(|k| {
            let (name, v) = self.groups.get(k)?;
            let mut v: Vec<f32> = v.iter().copied().filter(|x| x.is_finite()).collect();
            if v.is_empty() {
                return Some(format!("{name} · no data"));
            }
            v.sort_by(f32::total_cmp);
            let f = |x: f32| fmt_with(&self.fmt, x);
            let (n, mid) = (v.len(), v.len() / 2);
            let median = if n % 2 == 0 {
                (v[mid - 1] + v[mid]) / 2.0
            } else {
                v[mid]
            };
            Some(format!(
                "{name} · median {} · range {} – {} · {n} values",
                f(median),
                f(v[0]),
                f(v[n - 1])
            ))
        });
        let gutter = label_column(&t, self.groups.iter().map(|g| g.0.as_ref()));
        let row_frac = 1.0 / (m as f32 - 1.0 + overlap);
        let labels = self
            .groups
            .iter()
            .enumerate()
            .map(|(k, (l, _))| (l.clone(), (k as f32 + overlap) * row_frac - row_frac * 0.3));
        let x_row = tick_row(&t, &ticks, &self.fmt);
        let body = div()
            .flex()
            .w_full()
            .gap(px(t.spacing.sm))
            .child(row_labels(&t, gutter, self.height, labels, true))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .child(plot)
                    .child(x_row),
            );
        let root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(body)
            .child(readout(&t, text, "Hover a ridge"));
        titled(&t, self.title.clone(), root)
    }
}

dodai_gpui::entry_render!(RidgelineChart);

// ─────────────────────────────── HorizonChart ───────────────────────────────

/// Horizon chart: each series in a short strip, its range folded into stacked bands (deeper
/// colour = further from zero; below zero in the falling colour). Many series fit in little
/// height and stay comparable, as every strip shares one band size.
#[derive(IntoElement)]
pub struct HorizonChart {
    follow: dodai_gpui::follow::Follow,
    series: Vec<Series>,
    bands: usize,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(HorizonChart);

impl HorizonChart {
    dodai_gpui::follow_api!();
    /// Create a new [`HorizonChart`], one strip per series.
    #[track_caller]
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        let series: Vec<Series> = series.into_iter().collect();
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            height: series.len().max(1) as f32 * 32.0,
            series,
            bands: 3,
            fmt: None,
            title: None,
        }
    }
    /// How many bands each strip folds into (default 3, 1..=5).
    pub fn bands(mut self, n: usize) -> Self {
        self.bands = n.clamp(1, 5);
        self
    }
}

impl HorizonChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.series = crate::common::follow_series(window, cx, &self.follow, &self.series);
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "HorizonChart");
        let bands = self.bands;
        let all: Vec<f32> = self.series.iter().flat_map(|s| s.values.clone()).collect();
        let band = horizon::band_for(&all, bands);
        let rows: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let m = rows.len().max(1);
        let grid = hsla(t.palette.border);
        // Deeper bands closer to the full colour, so the eye reads depth as colour. Opaque
        // blends with the surface: translucent columns would show seams where they meet.
        let shades = |tone: dodai_core::Tone| -> Vec<Hsla> {
            let c = t.palette.tone(tone);
            (0..bands)
                .map(|k| {
                    let depth = 0.3 + 0.7 * (k + 1) as f32 / bands as f32;
                    hsla(t.palette.surface.mix(c, depth))
                })
                .collect()
        };
        let (up, down) = (
            shades(dodai_core::Tone::Success),
            shades(dodai_core::Tone::Danger),
        );
        let strips = rows.len();
        let focus = Focus::new(window, cx, &self.follow, strips);
        let looks: Vec<_> = (0..strips).map(|k| focus.item(k)).collect();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let strip = bh / m as f32;
                for (r, vals) in rows.iter().enumerate() {
                    let top = r as f32 * strip;
                    if r > 0 {
                        stroke(w, b.origin, &[(0.0, top), (bw, top)], 1.0, grid, false);
                    }
                    let n = vals.len().max(1);
                    let cw = bw / n as f32;
                    for (i, v) in vals.iter().enumerate() {
                        let x0 = i as f32 * cw;
                        let x1 = x0 + cw + 0.5;
                        let c = if *v < 0.0 { &down } else { &up };
                        for (k, f) in horizon::fold(*v * g, band, bands).into_iter().enumerate() {
                            if f <= 0.0 {
                                break;
                            }
                            let h = f * strip;
                            // Above zero grows up from the strip's floor, below hangs from its top.
                            let (y0, y1) = if *v < 0.0 {
                                (top, top + h)
                            } else {
                                (top + strip - h, top + strip)
                            };
                            let a = looks[r].alpha.max(0.3);
                            fill(
                                w,
                                b.origin,
                                &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)],
                                scale_alpha(c[k], a),
                            );
                        }
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("strips").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |_, y, _, h| {
            (h > 0.0 && (0.0..h).contains(&y))
                .then(|| (y / h * m as f32) as usize)
                .filter(|r| *r < strips)
        });
        let text = focus.shown(cx).and_then(|k| {
            let s = self.series.get(k)?;
            let v: Vec<f32> = s.values.iter().copied().filter(|x| x.is_finite()).collect();
            let f = |x: f32| fmt_with(&self.fmt, x);
            let last = *v.last()?;
            let (low, high) = dodai_charts_core::extent(v.iter().copied())?;
            Some(format!(
                "{} · latest {} · low {} · high {}",
                s.name,
                f(last),
                f(low),
                f(high)
            ))
        });
        let names = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| (s.name.clone(), (i as f32 + 0.5) / m as f32));
        let gutter = label_column(&t, self.series.iter().map(|s| s.name.as_ref()));
        let legend = label_style(&t, div())
            .flex()
            .gap(px(t.spacing.md))
            .ml(px(gutter + t.spacing.sm))
            .child(format!("Each band {}", fmt_with(&self.fmt, band)))
            .child(div().text_color(up_color(&t)).child("▲ above zero"))
            .child(div().text_color(down_color(&t)).child("▼ below zero"));
        let body = div()
            .flex()
            .w_full()
            .gap(px(t.spacing.sm))
            .child(row_labels(&t, gutter, self.height, names, true))
            .child(div().flex_1().child(plot));
        titled(
            &t,
            self.title.clone(),
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .w_full()
                .child(body)
                .child(legend)
                .child(readout(&t, text, "Hover a strip")),
        )
    }
}

dodai_gpui::entry_render!(HorizonChart);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bump_chart_is_tall_enough_for_its_rows() {
        let c = BumpChart::new((0..10).map(|i| Series::new(format!("s{i}"), [i as f32])));
        assert!(c.height >= 10.0 * 30.0);
        let r = RidgelineChart::new([("a", vec![1.0, 2.0])]).overlap(f32::NAN);
        assert_eq!(r.overlap, 1.8);
        assert_eq!(HorizonChart::new([]).bands(9).bands, 5);
    }

    #[test]
    fn ridges_are_hit_front_first_then_by_row() {
        let d = |lo: f32, hi: f32| density::Density {
            lo,
            hi,
            y: vec![1.0, 1.0],
        };
        let curves = [d(0.0, 10.0), d(0.0, 5.0)];
        // Two rows rising 2 rows tall in 100 px: row 33.3, bases at 66.7 and 100.
        let at = |x, y| ridge_hit(&curves, (0.0, 10.0), 2.0, (x, y, 100.0, 100.0));
        assert_eq!(
            at(20.0, 60.0),
            Some(1),
            "the front ridge covers the back one"
        );
        assert_eq!(at(80.0, 60.0), Some(0), "only the back ridge reaches here");
        assert_eq!(at(80.0, 90.0), Some(1), "no shape: the row the point is in");
        assert_eq!(at(80.0, 10.0), Some(0));
        assert_eq!(
            ridge_hit(&[], (0.0, 1.0), 2.0, (1.0, 1.0, 10.0, 10.0)),
            None
        );
        assert_eq!(density_at(&d(0.0, 1.0), 2.0), 0.0);
    }
}
