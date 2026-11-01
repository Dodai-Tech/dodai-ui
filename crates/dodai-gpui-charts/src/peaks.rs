//! [`PeakRange`]: overlapping series as a range of mountains on one horizon.

use crate::common::{
    chart_cfg, fmt_with, many_series, plot_box, tracked_canvas, CanvasText, NumFmt, Series,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::peaks;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// Peak range: each series a solid mountain on one scale, the tallest at the back so smaller
/// peaks stay visible in front. With [`PeakRange::lanes`] the series stand on several
/// horizons, one a little above the other, so a crowded range opens up into rows. Each
/// summit carries its name; hover a mountain for its peak, click to keep it picked.
#[derive(IntoElement)]
pub struct PeakRange {
    follow: dodai_gpui::follow::Follow,
    series: Vec<Series>,
    x_labels: Vec<SharedString>,
    lanes: usize,
    cap: f32,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(PeakRange);

impl PeakRange {
    dodai_gpui::follow_api!();
    /// Create a new [`PeakRange`]; every series is sampled on the same evenly spaced x grid.
    #[track_caller]
    pub fn new(series: impl IntoIterator<Item = Series>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            series: series.into_iter().collect(),
            x_labels: Vec::new(),
            lanes: 1,
            cap: 1.0,
            height: 320.0,
            fmt: None,
            title: None,
        }
    }
    /// Labels spread evenly under the horizon (months, years, ...).
    pub fn x_labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.x_labels = l.into_iter().map(Into::into).collect();
        self
    }
    /// Stand the series on `n` horizons (series `i` on lane `i % n`; lane 0 is lowest and
    /// in front). Default 1.
    pub fn lanes(mut self, n: usize) -> Self {
        self.lanes = n.clamp(1, 8);
        self
    }
    /// Light the top of each mountain above this share of its summit (default 1: off).
    pub fn cap(mut self, c: f32) -> Self {
        self.cap = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }
}

/// A lighter shade of `c` for the lit caps.
fn lit(c: Hsla) -> Hsla {
    Hsla {
        l: (c.l + (1.0 - c.l) * 0.45).min(0.97),
        ..c
    }
}

/// The outline of `values` across `w` px rising from `base` (y down), `scale` px per unit,
/// with every sample raised to at least `floor` units.
fn ridge(values: &[f32], w: f32, base: f32, scale: f32, floor: f32) -> Vec<(f32, f32)> {
    let n = values.len();
    let step = if n > 1 { w / (n - 1) as f32 } else { 0.0 };
    values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let v = if v.is_finite() { v.max(floor) } else { floor };
            (i as f32 * step, base - v * scale)
        })
        .collect()
}

/// Height of one lane step, as a share of the tallest summit.
const LANE_STEP: f32 = 0.09;

impl PeakRange {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "PeakRange");
        let n = self.series.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        let values: Vec<Vec<f32>> = self.series.iter().map(|s| s.values.clone()).collect();
        let summits: Vec<Option<(usize, f32)>> = values.iter().map(|v| peaks::summit(v)).collect();
        let top = summits.iter().flatten().map(|s| s.1).fold(0.0f32, f32::max);
        let step = top * LANE_STEP;
        let base: Vec<f32> = (0..n).map(|i| (i % self.lanes) as f32 * step).collect();
        let order = peaks::lane_order(&values, &base);
        let ymax = (top + step * (self.lanes - 1) as f32).max(f32::MIN_POSITIVE);
        let cols: Vec<Hsla> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| hsla(s.color.unwrap_or_else(|| many_series(&t, i))))
            .collect();
        let names: Vec<SharedString> = self.series.iter().map(|s| s.name.clone()).collect();
        let (axis, ink) = (hsla(t.palette.border_strong), hsla(t.palette.text));
        let cap = self.cap;
        let label = CanvasText::display(&t, t.typography.size_xs).bold();
        let room = label.size_line() + 4.0;
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let (pv, po, ps, pbase) = (values.clone(), order.clone(), summits.clone(), base.clone());
        let lanes = self.lanes;
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let scale = (bh - room).max(0.0) / ymax;
            for l in 0..lanes {
                let y = bh - l as f32 * step * scale;
                stroke(
                    w,
                    b.origin,
                    &[(0.0, y), (bw, y)],
                    1.0,
                    scale_alpha(axis, 0.5),
                    false,
                );
            }
            for &i in &po {
                let v = &pv[i];
                if v.len() < 2 {
                    continue;
                }
                let lk = looks[i];
                let floor = bh - pbase[i] * scale;
                let line = ridge(v, bw, floor, scale * g, 0.0);
                let mut body = line.clone();
                body.push((bw, floor));
                body.push((0.0, floor));
                fill(w, b.origin, &body, scale_alpha(cols[i], lk.alpha));
                if let Some((_, s)) = ps[i].filter(|s| s.1 > 0.0 && cap < 1.0) {
                    let f = s * cap;
                    let mut top = ridge(v, bw, floor, scale * g, f);
                    top.extend(ridge(&[f, f], bw, floor, scale * g, f).into_iter().rev());
                    fill(w, b.origin, &top, scale_alpha(lit(cols[i]), lk.alpha));
                }
                if lk.lift > 0.01 {
                    stroke(
                        w,
                        b.origin,
                        &line,
                        1.0 + lk.lift,
                        scale_alpha(ink, lk.lift),
                        false,
                    );
                }
            }
            stroke(w, b.origin, &[(0.0, bh), (bw, bh)], 1.0, axis, false);
            if g < 0.95 {
                return;
            }
            // Names on the summits: the focused first, then the tallest.
            let mut rank = po.clone();
            rank.sort_by(|a, c| {
                let key = |k: usize| looks[k].lift * 1e9 + ps[k].map_or(0.0, |s| s.1);
                key(*c).total_cmp(&key(*a))
            });
            let mut taken = Vec::new();
            for i in rank {
                let Some((at, s)) = ps[i] else { continue };
                let nv = pv[i].len();
                let x = if nv > 1 {
                    at as f32 / (nv - 1) as f32 * bw
                } else {
                    bw / 2.0
                };
                let y = bh - (pbase[i] + s.max(0.0)) * scale - 2.0;
                let x = x.clamp(30.0, (bw - 30.0).max(30.0));
                let c = scale_alpha(ink, looks[i].alpha.max(0.5));
                let spots = [
                    ((x, y), (0.0, -1.0)),
                    ((x + 4.0, y + 8.0), (1.0, -1.0)),
                    ((x - 4.0, y + 8.0), (-1.0, -1.0)),
                ];
                label.place_free(w, cx, (&names[i], c), b.origin, &spots, &mut taken);
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("range").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let span = h - room;
            if w <= 0.0 || span <= 0.0 {
                return None;
            }
            // Heights are read off the grown mountains, bases off the fixed lanes.
            let v = (h - y) / span * ymax;
            let grown: Vec<Vec<f32>> = values
                .iter()
                .map(|s| s.iter().map(|x| x * g).collect())
                .collect();
            peaks::hit(&grown, &base, &order, x / w, v)
        });
        let xl = self.x_labels.clone();
        let text = focus.shown(cx).and_then(|k| {
            let s = self.series.get(k)?;
            let (at, v) = summits[k]?;
            let n = s.values.len().max(1);
            let when = match xl.len() {
                0 => String::new(),
                m => format!(" in {}", xl[(at * m / n).min(m - 1)]),
            };
            Some(format!(
                "{} · peak {}{when}",
                s.name,
                fmt_with(&self.fmt, v)
            ))
        });
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot);
        if !self.x_labels.is_empty() {
            body = body.child(crate::common::slot_labels(&t, self.x_labels.clone()));
        }
        let body = body.child(readout(&t, text, "Hover a peak"));
        crate::common::titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(PeakRange);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ridges_rise_from_the_base_and_respect_the_floor() {
        let r = ridge(&[0.0, 2.0, f32::NAN], 100.0, 50.0, 10.0, 1.0);
        assert_eq!(r, [(0.0, 40.0), (50.0, 30.0), (100.0, 40.0)]);
        assert_eq!(ridge(&[3.0], 100.0, 50.0, 1.0, 0.0), [(0.0, 47.0)]);
    }
}
