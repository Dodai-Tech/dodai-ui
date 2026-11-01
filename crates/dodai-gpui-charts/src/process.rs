//! Process charts: [`ControlChart`] (is the process stable?) and [`EventTimeline`] (which
//! state was each lane in, when?).

use crate::common::{
    chart_cfg, down_color, fill, fmt_with, label_column, label_style, labels_of, marker_points,
    nice_ticks, norm, plot_box, stroke, tick_row, titled, tracked_canvas, Legend, Marker, NumFmt,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::control::{limits, signals, RUN};
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

// ─────────────────────────────── ControlChart ───────────────────────────────

/// Control chart (individuals): the values against their centre line and ±3σ control limits
/// (σ from the moving range). Points outside the limits, or in a run of nine on one side,
/// are flagged: the process changed.
#[derive(IntoElement)]
pub struct ControlChart {
    follow: dodai_gpui::follow::Follow,
    values: Vec<f32>,
    labels: Vec<SharedString>,
    sigma: f32,
    baseline: Option<usize>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ControlChart);

impl ControlChart {
    dodai_gpui::follow_api!();
    /// Create a new [`ControlChart`] of one measurement per sample, in time order.
    #[track_caller]
    pub fn new(values: impl IntoIterator<Item = f32>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            values: values.into_iter().collect(),
            labels: Vec::new(),
            sigma: 3.0,
            baseline: None,
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
    /// Sample labels under the chart (shown when there is one per value).
    pub fn labels<S: Into<SharedString>>(mut self, l: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels_of(l);
        self
    }
    /// Set the centre and limits from the first `n` samples only (a known stable period),
    /// then judge every sample against them. Default: all samples.
    pub fn baseline(mut self, n: usize) -> Self {
        self.baseline = (n >= 2).then_some(n);
        self
    }
    /// How many σ the limits sit from the centre (default 3, `1..=6`).
    pub fn sigma(mut self, k: f32) -> Self {
        self.sigma = if k.is_finite() {
            k.clamp(1.0, 6.0)
        } else {
            3.0
        };
        self
    }
}

/// Why sample `v` is flagged, if it is.
fn verdict(v: f32, flagged: bool, lo: f32, hi: f32) -> &'static str {
    if !flagged {
        "in control"
    } else if v > hi {
        "above the upper limit"
    } else if v < lo {
        "below the lower limit"
    } else {
        "part of a run on one side"
    }
}

impl ControlChart {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        self.values =
            dodai_gpui::follow::follow_all(window, cx, &self.follow.sub("v"), &self.values);
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "ControlChart");
        let base = self
            .baseline
            .map_or(self.values.len(), |n| n.min(self.values.len()));
        let l = limits(&self.values[..base], self.sigma);
        let flags = signals(&self.values, &l);
        let (lo, hi) = self
            .values
            .iter()
            .copied()
            .filter(|v| v.is_finite())
            .chain([l.lower, l.upper])
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), v| {
                (a.min(v), b.max(v))
            });
        let ticks = if hi > lo {
            nice_ticks(lo, hi, 4)
        } else {
            vec![lo - 1.0, lo + 1.0]
        };
        let (y0, y1) = (ticks[0], ticks[ticks.len() - 1]);
        let n = self.values.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let shown = focus.shown(cx);
        let (ink, bad, band, mid) = (
            hsla(t.palette.primary),
            down_color(&t),
            hsla(t.palette.primary),
            hsla(t.palette.text_muted),
        );
        let style = t.chart_style();
        let vals = self.values.clone();
        let paint_flags = flags.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let y = |v: f32| bh * (1.0 - norm(v, y0, y1));
            let x = |i: usize| (i as f32 + 0.5) / n.max(1) as f32 * bw;
            let (yu, yl, ym) = (y(l.upper), y(l.lower), y(l.mean));
            fill(
                w,
                b.origin,
                &[(0.0, yu), (bw, yu), (bw, yl), (0.0, yl)],
                scale_alpha(band, 0.08),
            );
            for v in [l.upper, l.lower] {
                crate::common::grid_line(w, b.origin, (0.0, y(v)), (bw, y(v)), &style);
            }
            stroke(w, b.origin, &[(0.0, ym), (bw, ym)], 1.5, mid, false);
            let reach = ((n as f32 * g).ceil() as usize).min(n);
            let p: Vec<(f32, f32)> = vals
                .iter()
                .take(reach)
                .enumerate()
                .filter(|(_, v)| v.is_finite())
                .map(|(i, v)| (x(i), y(*v)))
                .collect();
            stroke(w, b.origin, &p, 1.5, scale_alpha(ink, 0.8), false);
            for (i, v) in vals.iter().take(reach).enumerate() {
                if !v.is_finite() {
                    continue;
                }
                let (c, r) = if paint_flags[i] {
                    (bad, 4.5)
                } else {
                    (ink, 2.5)
                };
                let r = if shown == Some(i) { r + 2.0 } else { r };
                fill(w, b.origin, &marker_points(Marker::Dot, x(i), y(*v), r), c);
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("ctl").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, _, w, _| {
            let i = (x / w.max(1.0) * n as f32) as usize;
            (i < n).then_some(i)
        });
        let flagged = flags.iter().filter(|f| **f).count();
        let text = shown.and_then(|i| {
            let v = *self.values.get(i)?;
            let name = self
                .labels
                .get(i)
                .map_or_else(|| format!("#{}", i + 1), |s| s.to_string());
            Some(format!(
                "{name} · {} · {}",
                fmt_with(&self.fmt, v),
                verdict(v, flags[i], l.lower, l.upper)
            ))
        });
        let from = if base < n {
            format!(" (from the first {base})")
        } else {
            String::new()
        };
        let hint = format!(
            "Centre {}{from} · limits {} – {} · {flagged} flagged (outside, or a run of {RUN})",
            fmt_with(&self.fmt, l.mean),
            fmt_with(&self.fmt, l.lower),
            fmt_with(&self.fmt, l.upper),
        );
        let x_labels = if self.labels.len() == n {
            self.labels.clone()
        } else {
            Vec::new()
        };
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(crate::common::framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                x_labels,
                div().w_full().child(plot),
            ))
            .child(readout(&t, text, &hint));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(ControlChart);

// ─────────────────────────────── EventTimeline ───────────────────────────────

/// One span of an [`EventTimeline`] lane: from `start` to `end` in `state`.
#[derive(Clone, Debug)]
pub struct StateSpan {
    /// Start time (any unit; format with [`EventTimeline::format`]).
    pub start: f32,
    /// End time.
    pub end: f32,
    /// The state the lane was in.
    pub state: SharedString,
}

impl StateSpan {
    /// A span from `start` to `end` (swapped if reversed) in `state`.
    pub fn new(start: f32, end: f32, state: impl Into<SharedString>) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
            state: state.into(),
        }
    }
}

/// Event timeline: one lane per system or person, coloured by the state it was in over time
/// (up, degraded, down; on shift, on call). Every state keeps one colour across lanes.
#[derive(IntoElement)]
pub struct EventTimeline {
    follow: dodai_gpui::follow::Follow,
    lanes: Vec<(SharedString, Vec<StateSpan>)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(EventTimeline);

impl EventTimeline {
    dodai_gpui::follow_api!();
    /// Create a new [`EventTimeline`] from `(lane, spans)`.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        lanes: impl IntoIterator<Item = (S, Vec<StateSpan>)>,
    ) -> Self {
        let lanes: Vec<(SharedString, Vec<StateSpan>)> =
            lanes.into_iter().map(|(l, s)| (l.into(), s)).collect();
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            height: (lanes.len().max(1) as f32 * 28.0).max(84.0),
            lanes,
            fmt: None,
            title: None,
        }
    }
}

/// The distinct states in first-seen order.
fn states_of(lanes: &[(SharedString, Vec<StateSpan>)]) -> Vec<SharedString> {
    let mut out: Vec<SharedString> = Vec::new();
    for s in lanes.iter().flat_map(|l| &l.1) {
        if !out.contains(&s.state) {
            out.push(s.state.clone());
        }
    }
    out
}

impl EventTimeline {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "EventTimeline");
        let states = states_of(&self.lanes);
        let (lo, hi) = self
            .lanes
            .iter()
            .flat_map(|l| &l.1)
            .flat_map(|s| [s.start, s.end])
            .filter(|v| v.is_finite())
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(a, b), v| {
                (a.min(v), b.max(v))
            });
        let ticks = if hi > lo {
            nice_ticks(lo, hi, 6)
        } else {
            vec![0.0, 1.0]
        };
        let (t0, t1) = (ticks[0], ticks[ticks.len() - 1]);
        // Flat spans: (lane, start, end, state index).
        let spans: Vec<(usize, f32, f32, usize)> = self
            .lanes
            .iter()
            .enumerate()
            .flat_map(|(k, (_, ss))| {
                let states = &states;
                ss.iter()
                    .filter(|s| s.start.is_finite() && s.end.is_finite())
                    .map(move |s| {
                        (
                            k,
                            s.start,
                            s.end,
                            states.iter().position(|x| *x == s.state).unwrap_or(0),
                        )
                    })
            })
            .collect();
        let nl = self.lanes.len().max(1);
        let focus = Focus::new(window, cx, &self.follow, spans.len());
        let looks: Vec<_> = (0..spans.len()).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..states.len())
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let paint_spans = spans.clone();
        let hit_spans = spans.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let lane = bh / nl as f32;
            // Time sweeps in from the left.
            let edge = g * bw;
            for (i, &(k, a, e, s)) in paint_spans.iter().enumerate() {
                let lk = looks[i];
                let x0 = norm(a, t0, t1) * bw;
                let x1 = (norm(e, t0, t1) * bw).min(edge);
                if x1 - x0 < 0.5 {
                    continue;
                }
                let pad = lane * 0.15 - lk.lift * 2.0;
                let (y0, y1) = (k as f32 * lane + pad, (k + 1) as f32 * lane - pad);
                fill(
                    w,
                    b.origin,
                    &[(x0, y0), (x1 - 1.0, y0), (x1 - 1.0, y1), (x0, y1)],
                    scale_alpha(cols[s], lk.alpha),
                );
            }
        })
        .absolute()
        .size_full();
        let plot = div()
            .id(self.follow.sub("lanes").element_id(window, cx))
            .relative()
            .flex_1()
            .h(px(self.height))
            .child(cv);
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let k = (y / (h / nl as f32)) as usize;
            let v = t0 + (t1 - t0) * x / w.max(1.0);
            hit_spans
                .iter()
                .position(|&(l, a, e, _)| l == k && (a..=e).contains(&v))
        });
        let text = focus.shown(cx).and_then(|i| {
            let &(k, a, e, s) = spans.get(i)?;
            Some(format!(
                "{} · {} · {} – {} ({})",
                self.lanes.get(k)?.0,
                states.get(s)?,
                fmt_with(&self.fmt, a),
                fmt_with(&self.fmt, e),
                fmt_with(&self.fmt, e - a)
            ))
        });
        let gutter = label_column(&t, self.lanes.iter().map(|l| l.0.as_ref()));
        let mut names = label_style(&t, div())
            .relative()
            .w(px(gutter))
            .flex_none()
            .h(px(self.height));
        for (i, (name, _)) in self.lanes.iter().enumerate() {
            names = names.child(
                div()
                    .absolute()
                    .top(relative(i as f32 / nl as f32))
                    .h(relative(1.0 / nl as f32))
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_end()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_color(hsla(t.palette.text))
                    .child(name.clone()),
            );
        }
        let legend = Legend::new(
            states
                .iter()
                .enumerate()
                .map(|(i, s)| (s.clone(), t.palette.series_at(i))),
        );
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(div().flex().gap(px(t.spacing.sm)).child(names).child(plot))
            .child(
                div()
                    .ml(px(gutter + t.spacing.sm))
                    .child(tick_row(&t, &ticks, &self.fmt)),
            )
            .child(legend)
            .child(readout(&t, text, "Hover a span"));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(EventTimeline);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_flag_says_why() {
        assert_eq!(verdict(5.0, false, 0.0, 10.0), "in control");
        assert_eq!(verdict(11.0, true, 0.0, 10.0), "above the upper limit");
        assert_eq!(verdict(-1.0, true, 0.0, 10.0), "below the lower limit");
        assert_eq!(verdict(6.0, true, 0.0, 10.0), "part of a run on one side");
    }

    #[test]
    fn states_keep_their_first_seen_order() {
        let lanes = vec![
            (
                "a".into(),
                vec![
                    StateSpan::new(0.0, 1.0, "up"),
                    StateSpan::new(2.0, 1.0, "down"),
                ],
            ),
            (
                "b".into(),
                vec![
                    StateSpan::new(0.0, 1.0, "down"),
                    StateSpan::new(1.0, 3.0, "idle"),
                ],
            ),
        ];
        let s: Vec<String> = states_of(&lanes).iter().map(|s| s.to_string()).collect();
        assert_eq!(s, ["up", "down", "idle"]);
        assert_eq!((lanes[0].1[1].start, lanes[0].1[1].end), (1.0, 2.0));
    }
}
