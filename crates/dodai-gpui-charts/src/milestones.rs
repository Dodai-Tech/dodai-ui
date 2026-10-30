//! [`TimeScatter`]: labelled milestones over time, with an optional record frontier, group
//! trails, reference lines and bubble sizes.

use crate::common::{
    chart_cfg, fmt_with, framed_row, group_slots, nice_ticks, norm, tick_row, turn_arc, CanvasText,
    Legend, NumFmt,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::milestones;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    canvas, div, px, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// One point of a [`TimeScatter`]: a labelled value at a time.
#[derive(Clone, Debug)]
pub struct Milestone {
    label: SharedString,
    x: f32,
    y: f32,
    size: Option<f32>,
    group: Option<SharedString>,
    hollow: bool,
}

impl Milestone {
    /// `label` at time `x` with value `y`.
    pub fn new(label: impl Into<SharedString>, x: f32, y: f32) -> Self {
        Self {
            label: label.into(),
            x,
            y,
            size: None,
            group: None,
            hollow: false,
        }
    }
    /// A size: the point becomes a bubble whose *area* follows it.
    pub fn size(mut self, s: f32) -> Self {
        self.size = Some(s);
        self
    }
    /// Its group: points of a group share a colour, a legend entry and (with trails) a line.
    pub fn group(mut self, g: impl Into<SharedString>) -> Self {
        self.group = Some(g.into());
        self
    }
    /// Draw it as a ring instead of a dot (a second, yes/no property).
    pub fn hollow(mut self, h: bool) -> Self {
        self.hollow = h;
        self
    }
}

/// A labelled horizontal line across a [`TimeScatter`].
#[derive(Clone, Debug)]
struct Reference {
    y: f32,
    label: SharedString,
}

/// Time scatter: labelled milestones at `(time, value)`. Options draw the record frontier (a
/// staircase of the best value so far), join each group's points in time order (trails), add
/// labelled reference lines, and size points as bubbles. Labels are placed largest first and
/// skipped where they would collide; hover a point for its numbers, click to keep it.
#[derive(IntoElement)]
pub struct TimeScatter {
    follow: dodai_gpui::follow::Follow,
    points: Vec<Milestone>,
    refs: Vec<Reference>,
    frontier: bool,
    trend: bool,
    trails: bool,
    height: f32,
    fmt: Option<NumFmt>,
    x_fmt: Option<NumFmt>,
    size_fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(TimeScatter);

impl TimeScatter {
    dodai_gpui::follow_api!();
    /// Create a new [`TimeScatter`] of `points`.
    #[track_caller]
    pub fn new(points: impl IntoIterator<Item = Milestone>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            points: points.into_iter().collect(),
            refs: Vec::new(),
            frontier: false,
            trend: false,
            trails: false,
            height: 420.0,
            fmt: None,
            x_fmt: None,
            size_fmt: None,
            title: None,
        }
    }
    /// Draw the record frontier: the best value so far, as a staircase.
    pub fn frontier(mut self, on: bool) -> Self {
        self.frontier = on;
        self
    }
    /// Draw the least-squares trend line through all points.
    pub fn trend(mut self, on: bool) -> Self {
        self.trend = on;
        self
    }
    /// Join each group's points in time order.
    pub fn trails(mut self, on: bool) -> Self {
        self.trails = on;
        self
    }
    /// A dashed reference line at value `y`, labelled `label` (a target, a break-even, ...).
    pub fn reference(mut self, y: f32, label: impl Into<SharedString>) -> Self {
        self.refs.push(Reference {
            y,
            label: label.into(),
        });
        self
    }
    /// Formatter for times (the x axis).
    pub fn x_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.x_fmt = Some(Rc::new(f));
        self
    }
    /// Formatter for bubble sizes in the readout.
    pub fn size_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.size_fmt = Some(Rc::new(f));
        self
    }
}

/// Radius of the smallest and largest bubble, and of an unsized point, in px.
const R_MIN: f32 = 3.5;
const R_MAX: f32 = 18.0;
const R_DOT: f32 = 4.5;

/// The finite extent of `v`, widened by `also`; `0..1` when there is none.
fn extent(v: impl Iterator<Item = f32>) -> (f32, f32) {
    let (l, h) = v
        .filter(|v| v.is_finite())
        .fold((f32::INFINITY, f32::NEG_INFINITY), |(l, h), v| {
            (l.min(v), h.max(v))
        });
    if l <= h {
        (l, h)
    } else {
        (0.0, 1.0)
    }
}

/// Dashes along the horizontal line `y` from `x0` to `x1`.
fn dashes(x0: f32, x1: f32, y: f32) -> Vec<[(f32, f32); 2]> {
    let mut out = Vec::new();
    let mut x = x0;
    while x < x1 && out.len() < 600 {
        out.push([(x, y), ((x + 6.0).min(x1), y)]);
        x += 10.0;
    }
    out
}

impl TimeScatter {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "TimeScatter");
        let n = self.points.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        let (slots, groups) = group_slots(self.points.iter().map(|p| p.group.as_ref()));
        let cols: Vec<Hsla> = slots
            .iter()
            .map(|s| hsla(t.palette.series_at(*s)))
            .collect();
        let (xl, xh) = extent(self.points.iter().map(|p| p.x));
        let (yl, yh) = extent(
            self.points
                .iter()
                .map(|p| p.y)
                .chain(self.refs.iter().map(|r| r.y)),
        );
        let (xt, yt) = (nice_ticks(xl, xh, 5), nice_ticks(yl, yh, 5));
        let (x0, x1) = (xt[0], xt[xt.len() - 1]);
        let (y0, y1) = (yt[0], yt[yt.len() - 1]);
        let biggest = self
            .points
            .iter()
            .filter_map(|p| p.size)
            .filter(|s| s.is_finite())
            .fold(0.0f32, f32::max);
        let radius: Vec<f32> = self
            .points
            .iter()
            .map(|p| match p.size {
                Some(s) if biggest > 0.0 && s.is_finite() && s > 0.0 => {
                    R_MIN + (s / biggest).sqrt() * (R_MAX - R_MIN)
                }
                Some(_) if biggest > 0.0 => R_MIN,
                _ => R_DOT,
            })
            .collect();
        let unit: Vec<(f32, f32)> = self
            .points
            .iter()
            .map(|p| (norm(p.x, x0, x1), 1.0 - norm(p.y, y0, y1)))
            .collect();
        let xy: Vec<(f32, f32)> = self.points.iter().map(|p| (p.x, p.y)).collect();
        let stairs: Vec<(f32, f32)> = if self.frontier {
            milestones::frontier(&xy)
                .into_iter()
                .map(|(x, y)| (norm(x, x0, x1), 1.0 - norm(y, y0, y1)))
                .collect()
        } else {
            Vec::new()
        };
        let trend: Vec<(f32, f32)> = self
            .trend
            .then(|| milestones::trend(&xy))
            .flatten()
            .map(|(k, c)| {
                [x0, x1]
                    .map(|x| (norm(x, x0, x1), 1.0 - norm(k * x + c, y0, y1)))
                    .to_vec()
            })
            .unwrap_or_default();
        let trails = if self.trails {
            milestones::trails(&xy, &slots)
        } else {
            Vec::new()
        };
        let refs: Vec<(f32, SharedString)> = self
            .refs
            .iter()
            .filter(|r| r.y.is_finite())
            .map(|r| (1.0 - norm(r.y, y0, y1), r.label.clone()))
            .collect();
        let labels: Vec<SharedString> = self.points.iter().map(|p| p.label.clone()).collect();
        let hollow: Vec<bool> = self.points.iter().map(|p| p.hollow).collect();
        let (grid, muted, ink) = (
            hsla(t.palette.border),
            hsla(t.palette.text_muted),
            hsla(t.palette.text),
        );
        let small = CanvasText::of(&t);
        let name = CanvasText::display(&t, t.typography.size_xs);
        let surface = hsla(t.palette.surface);
        let (xf, yf): (Vec<f32>, Vec<f32>) = (
            xt.iter().map(|v| norm(*v, x0, x1)).collect(),
            yt.iter().map(|v| 1.0 - norm(*v, y0, y1)).collect(),
        );
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let pb = bounds.clone();
        let (paint_unit, paint_radius) = (unit.clone(), radius.clone());
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, cx| {
                pb.set(b);
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let at = |(u, v): (f32, f32)| (u * bw, v * bh);
                for f in &yf {
                    stroke(
                        w,
                        b.origin,
                        &[(0.0, f * bh), (bw, f * bh)],
                        1.0,
                        scale_alpha(grid, 0.6),
                        false,
                    );
                }
                for f in &xf {
                    stroke(
                        w,
                        b.origin,
                        &[(f * bw, 0.0), (f * bw, bh)],
                        1.0,
                        scale_alpha(grid, 0.35),
                        false,
                    );
                }
                let mut taken: Vec<[f32; 4]> = Vec::new();
                for (v, label) in &refs {
                    let y = v * bh;
                    for d in dashes(0.0, bw, y) {
                        stroke(w, b.origin, &d, 1.0, muted, false);
                    }
                    // A boxed label at the left end, over the line.
                    let (lw, lh) = small.measure(w, label);
                    let r = dodai_gpui::plot::rect(2.0, y - lh / 2.0 - 2.0, lw + 10.0, lh + 4.0);
                    fill(w, b.origin, &r, surface);
                    stroke(w, b.origin, &r, 1.0, muted, true);
                    small.beside_free(
                        w,
                        cx,
                        (label, ink),
                        (b.origin, (7.0, y)),
                        (1.0, 0.0),
                        &mut taken,
                    );
                }
                for trail in &trails {
                    let line: Vec<(f32, f32)> = trail.iter().map(|i| at(paint_unit[*i])).collect();
                    let c = cols[trail[0]];
                    stroke(
                        w,
                        b.origin,
                        &line,
                        1.5,
                        scale_alpha(c, 0.5 * g.min(1.0)),
                        false,
                    );
                }
                if trend.len() == 2 {
                    let line: Vec<(f32, f32)> = trend.iter().map(|p| at(*p)).collect();
                    stroke(
                        w,
                        b.origin,
                        &line,
                        1.0,
                        scale_alpha(ink, 0.6 * g.min(1.0)),
                        false,
                    );
                }
                if stairs.len() > 1 {
                    let line: Vec<(f32, f32)> = stairs.iter().map(|p| at(*p)).collect();
                    stroke(
                        w,
                        b.origin,
                        &line,
                        1.5,
                        scale_alpha(ink, 0.55 * g.min(1.0)),
                        false,
                    );
                }
                // Big bubbles first so small ones stay on top.
                let mut order: Vec<usize> = (0..paint_unit.len()).collect();
                order.sort_by(|a, c| paint_radius[*c].total_cmp(&paint_radius[*a]));
                for &i in &order {
                    let lk = looks[i];
                    let c = scale_alpha(cols[i], lk.alpha);
                    let r = (paint_radius[i] + lk.lift * 3.0)
                        * dodai_gpui::motion::reveal(g, i, paint_unit.len());
                    if r <= 0.0 {
                        continue;
                    }
                    let (x, y) = at(paint_unit[i]);
                    let ring = turn_arc(x, y, r, 0.0, 1.0);
                    if hollow[i] {
                        stroke(w, b.origin, &ring, 2.0, c, true);
                    } else {
                        fill(w, b.origin, &ring, scale_alpha(c, 0.85));
                        stroke(
                            w,
                            b.origin,
                            &ring,
                            1.0,
                            scale_alpha(surface, 0.8 * lk.alpha),
                            true,
                        );
                    }
                }
                // Bubbles keep their own room, then labels go to the first free side: the
                // focused point first, then the biggest.
                for (i, u) in paint_unit.iter().enumerate() {
                    let (x, y) = at(*u);
                    let r = paint_radius[i] * g.min(1.0);
                    taken.push([x - r, y - r, x + r, y + r]);
                }
                let mut rank: Vec<usize> = (0..paint_unit.len()).collect();
                rank.sort_by(|a, c| {
                    let key = |k: usize| looks[k].lift * 100.0 + paint_radius[k];
                    key(*c).total_cmp(&key(*a))
                });
                for i in rank {
                    let (x, y) = at(paint_unit[i]);
                    let r = paint_radius[i] * g.min(1.0) + 3.0;
                    let col = scale_alpha(ink, lk_alpha(looks[i].alpha));
                    let spots = [
                        ((x + r, y), (1.0, 0.0)),
                        ((x, y + r), (0.0, 1.0)),
                        ((x - r, y), (-1.0, 0.0)),
                        ((x, y - r), (0.0, -1.0)),
                    ];
                    // Bubbles' own boxes stand aside for their labels.
                    let own = taken
                        .iter()
                        .position(|t| *t == [x - r + 3.0, y - r + 3.0, x + r - 3.0, y + r - 3.0]);
                    let mine = own.map(|k| taken.remove(k));
                    name.place_free(w, cx, (&labels[i], col), b.origin, &spots, &mut taken);
                    if let Some(m) = mine {
                        taken.push(m);
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("plot").element_id(window, cx))
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv);
        let hit_unit = unit.clone();
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let px: Vec<(f32, f32)> = hit_unit.iter().map(|(u, v)| (u * w, v * h)).collect();
            milestones::nearest(&px, (x, y), R_MAX + 4.0)
        });
        let text = focus.shown(cx).and_then(|k| self.points.get(k)).map(|p| {
            let mut line = format!(
                "{} · {} · {}",
                p.label,
                fmt_with(&self.x_fmt, p.x),
                fmt_with(&self.fmt, p.y)
            );
            if let Some(s) = p.size {
                line += &format!(" · {}", fmt_with(&self.size_fmt, s));
            }
            if let Some(g) = &p.group {
                line += &format!(" · {g}");
            }
            line
        });
        let x_row = tick_row(&t, &xt, &self.x_fmt);
        let y_labels = yt.iter().map(|v| fmt_with(&self.fmt, *v)).collect();
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(framed_row(
                &t,
                y_labels,
                Some(x_row),
                div().w_full().child(plot),
            ))
            .child(readout(&t, text, "Hover a point"));
        if !groups.is_empty() {
            body = body.child(Legend::new(
                groups
                    .iter()
                    .enumerate()
                    .map(|(i, n)| (n.clone(), t.palette.series_at(i))),
            ));
        }
        crate::common::titled(&t, self.title, body)
    }
}

/// Label opacity for a point painted at `alpha`: dimmed points keep a readable name.
fn lk_alpha(alpha: f32) -> f32 {
    alpha.max(0.5)
}

dodai_gpui::entry_render!(TimeScatter);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extent_and_dashes_are_bounded() {
        assert_eq!(extent([f32::NAN, 2.0, -1.0].into_iter()), (-1.0, 2.0));
        assert_eq!(extent(std::iter::empty()), (0.0, 1.0));
        assert_eq!(dashes(0.0, 25.0, 3.0).len(), 3);
        assert!(dashes(0.0, f32::INFINITY, 0.0).len() <= 600);
        assert!(dashes(0.0, f32::NAN, 0.0).is_empty());
    }
}
