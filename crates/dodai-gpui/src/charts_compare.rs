//! Comparison charts: [`Bullet`], [`PairedBars`], [`Lollipop`] (and its
//! dumbbell form) and [`SlopeChart`].

use crate::charts::{
    chart_cfg, down_color, fmt_with, framed, label_style, nice_ticks, norm, titled, up_color,
    Legend, NumFmt,
};
use crate::paint::{scale_alpha, stroke};
use crate::panel::DodaiTooltip;
use crate::{hsla, theme_of};
use dodai_core::Color;
use gpui::{
    canvas, div, px, relative, App, Bounds, InteractiveElement, IntoElement, ParentElement, Pixels,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ─────────────────────────────── Bullet ───────────────────────────────

/// Bullet graph: a value bar over qualitative range bands with a target tick.
/// A compact KPI row; stack several for a dashboard.
#[derive(IntoElement)]
pub struct Bullet {
    label: SharedString,
    value: f32,
    target: Option<f32>,
    ranges: Vec<f32>,
    max: Option<f32>,
    color: Option<Color>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl Bullet {
    /// Create a new [`Bullet`].
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value: crate::charts::finite_or_zero(value),
            target: None,
            ranges: Vec::new(),
            max: None,
            color: None,
            height: 28.0,
            fmt: None,
            title: None,
        }
    }
    /// Target marker position.
    pub fn target(mut self, t: f32) -> Self {
        self.target = Some(t);
        self
    }
    /// Upper bounds of the qualitative bands, e.g. `[150, 225, 300]` for
    /// poor / fair / good.
    pub fn ranges(mut self, r: impl IntoIterator<Item = f32>) -> Self {
        self.ranges = r.into_iter().filter(|v| v.is_finite()).collect();
        self.ranges.sort_by(f32::total_cmp);
        self
    }
    /// Scale maximum (default: the largest of value, target and ranges).
    pub fn max(mut self, m: f32) -> Self {
        self.max = Some(m);
        self
    }
    /// Value bar colour (default: the theme's primary).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}
chart_cfg!(Bullet);

impl Bullet {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "Bullet");
        let max = self
            .max
            .unwrap_or_else(|| {
                self.ranges
                    .iter()
                    .copied()
                    .chain([self.value, self.target.unwrap_or(0.0)])
                    .fold(0.0f32, f32::max)
            })
            .max(f32::MIN_POSITIVE);
        let band = hsla(t.palette.text_muted);
        let mut track = div().relative().flex_1().h(px(self.height));
        let mut prev = 0.0;
        let n = self.ranges.len().max(1);
        let edges: Vec<f32> = if self.ranges.is_empty() {
            vec![max]
        } else {
            self.ranges.clone()
        };
        for (i, hi) in edges.iter().enumerate() {
            let (a, b) = (norm(prev, 0.0, max), norm(*hi, 0.0, max));
            // Darkest band is the lowest ("poor"), lightest the highest.
            let alpha = 0.34 - 0.24 * (i as f32 / (n - 1).max(1) as f32);
            track = track.child(
                div()
                    .absolute()
                    .left(relative(a))
                    .w(relative(b - a))
                    .h_full()
                    .bg(scale_alpha(band, alpha)),
            );
            prev = *hi;
        }
        let col = hsla(self.color.unwrap_or(t.palette.primary));
        track = track.child(
            div()
                .id(SharedString::from(format!("bullet-{}", self.label)))
                .absolute()
                .left_0()
                .top(relative(0.33))
                .h(relative(0.34))
                .w(relative(norm(self.value, 0.0, max) * g))
                .bg(col)
                .tooltip(DodaiTooltip::text(format!(
                    "{}: {}",
                    self.label,
                    fmt_with(&self.fmt, self.value)
                ))),
        );
        if let Some(tg) = self.target {
            track = track.child(
                div()
                    .absolute()
                    .left(relative(norm(tg, 0.0, max)))
                    .ml(px(-1.0))
                    .top(relative(0.15))
                    .h(relative(0.7))
                    .w(px(2.0))
                    .bg(hsla(t.palette.text)),
            );
        }
        let row = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.md))
            .w_full()
            .child(
                label_style(&t, div())
                    .w(px(t.sizes().axis_gutter_lg * 1.6))
                    .flex_none()
                    .text_color(hsla(t.palette.text))
                    .child(self.label.clone()),
            )
            .child(track)
            .child(
                label_style(&t, div())
                    .w(px(t.sizes().axis_gutter_lg))
                    .flex_none()
                    .child(fmt_with(&self.fmt, self.value)),
            );
        titled(&t, self.title.clone(), div().w_full().child(row))
    }
}
crate::entry_render!(Bullet);

// ───────────────────────────── PairedBars ─────────────────────────────

/// Before/after bars per row. Each row is scaled to its own larger value, so
/// measures in different units compare by change, not magnitude.
#[derive(IntoElement)]
pub struct PairedBars {
    follow: crate::instruments_motion::Follow,
    rows: Vec<(SharedString, f32, f32)>,
    names: (SharedString, SharedString),
    shared: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl PairedBars {
    crate::instruments_motion::follow_api!();
    /// Create a new [`PairedBars`] from `(label, before, after)` rows.
    #[track_caller]
    pub fn new<S: Into<SharedString>>(rows: impl IntoIterator<Item = (S, f32, f32)>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            rows: rows
                .into_iter()
                .map(|(l, a, b)| {
                    (
                        l.into(),
                        crate::charts::finite_or_zero(a),
                        crate::charts::finite_or_zero(b),
                    )
                })
                .collect(),
            names: ("Before".into(), "After".into()),
            shared: false,
            height: 0.0,
            fmt: None,
            title: None,
        }
    }
    /// Names of the two states (legend and tooltips).
    pub fn names(mut self, a: impl Into<SharedString>, b: impl Into<SharedString>) -> Self {
        self.names = (a.into(), b.into());
        self
    }
    /// Use one scale for every row instead of one per row.
    pub fn shared_scale(mut self, on: bool) -> Self {
        self.shared = on;
        self
    }
}
chart_cfg!(PairedBars);

impl PairedBars {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_triples(window, cx, &self.follow, &mut self.rows);
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "PairedBars");
        let (ca, cb) = (t.palette.series_at(0), t.palette.series_at(1));
        let global = self
            .rows
            .iter()
            .flat_map(|r| [r.1.abs(), r.2.abs()])
            .fold(f32::MIN_POSITIVE, f32::max);
        let mut col = div().flex().flex_col().gap(px(t.spacing.md)).w_full();
        for (i, (label, a, b)) in self.rows.iter().enumerate() {
            let m = if self.shared {
                global
            } else {
                a.abs().max(b.abs()).max(f32::MIN_POSITIVE)
            };
            let delta = if *a != 0.0 {
                (b - a) / a.abs() * 100.0
            } else {
                0.0
            };
            let dc = if b >= a { up_color(&t) } else { down_color(&t) };
            let bar = |k: usize, v: f32, c: Color, who: &SharedString| {
                div()
                    .id((SharedString::from(format!("pb-{i}")), k))
                    .h(px(t.sizes().legend_swatch + 2.0))
                    .w(relative((v.abs() / m).clamp(0.0, 1.0) * g))
                    .bg(hsla(c))
                    .tooltip(DodaiTooltip::text(format!(
                        "{label} · {who}: {}",
                        fmt_with(&self.fmt, v)
                    )))
            };
            col = col.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.md))
                    .child(
                        label_style(&t, div())
                            .w(px(t.sizes().axis_gutter_lg * 1.4))
                            .flex_none()
                            .text_color(hsla(t.palette.text))
                            .child(label.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(bar(0, *a, ca, &self.names.0))
                            .child(bar(1, *b, cb, &self.names.1)),
                    )
                    .child(
                        label_style(&t, div())
                            .w(px(t.sizes().axis_gutter_lg * 1.2))
                            .flex_none()
                            .text_color(dc)
                            .child(format!("{delta:+.0}%")),
                    ),
            );
        }
        let legend = Legend::new([(self.names.0.clone(), ca), (self.names.1.clone(), cb)]);
        titled(
            &t,
            self.title.clone(),
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .w_full()
                .child(col)
                .child(legend),
        )
    }
}
crate::entry_render!(PairedBars);

// ─────────────────────────────── Lollipop ───────────────────────────────

/// Stems with dots; with [`Lollipop::compare`] a dumbbell (two dots joined by
/// a bar) that shows change per category.
#[derive(IntoElement)]
pub struct Lollipop {
    follow: crate::instruments_motion::Follow,
    id: SharedString,
    items: Vec<(SharedString, f32)>,
    other: Vec<f32>,
    names: (SharedString, SharedString),
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl Lollipop {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Lollipop`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            id: id.into(),
            items: items
                .into_iter()
                .map(|(l, v)| (l.into(), crate::charts::finite_or_zero(v)))
                .collect(),
            other: Vec::new(),
            names: ("A".into(), "B".into()),
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
    /// Second value per category: draws a dumbbell between the two.
    pub fn compare(mut self, values: impl IntoIterator<Item = f32>) -> Self {
        self.other = values
            .into_iter()
            .map(crate::charts::finite_or_zero)
            .collect();
        self
    }
    /// Names of the two states in dumbbell mode.
    pub fn names(mut self, a: impl Into<SharedString>, b: impl Into<SharedString>) -> Self {
        self.names = (a.into(), b.into());
        self
    }
}
chart_cfg!(Lollipop);

impl Lollipop {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_pairs(window, cx, &self.follow, &mut self.items);
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "Lollipop");
        let n = self.items.len().max(1);
        let all: Vec<f32> = self
            .items
            .iter()
            .map(|i| i.1)
            .chain(self.other.iter().copied())
            .collect();
        let lo = all.iter().copied().fold(0.0f32, f32::min);
        let hi = all.iter().copied().fold(0.0f32, f32::max);
        let ticks = nice_ticks(lo, hi.max(lo + f32::EPSILON), 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let grid = hsla(t.palette.border);
        let (ca, cb) = (hsla(t.palette.series_at(0)), hsla(t.palette.series_at(1)));
        let dumbbell = !self.other.is_empty();
        let a_vals: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        let b_vals = self.other.clone();
        let tick_fracs: Vec<f32> = ticks.iter().map(|v| norm(*v, lo, hi)).collect();
        let zero = norm(0.0, lo, hi);
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for f in &tick_fracs {
                    let y = bh - f * bh;
                    stroke(w, b.origin, &[(0.0, y), (bw, y)], 1.0, grid, false);
                }
                for (i, a) in a_vals.iter().enumerate() {
                    let x = (i as f32 + 0.5) / n as f32 * bw;
                    let ya = bh - (zero + (norm(*a, lo, hi) - zero) * g) * bh;
                    if dumbbell {
                        let bv = b_vals.get(i).copied().unwrap_or(*a);
                        let yb = bh - (zero + (norm(bv, lo, hi) - zero) * g) * bh;
                        stroke(
                            w,
                            b.origin,
                            &[(x, ya), (x, yb)],
                            3.0,
                            scale_alpha(ca, 0.55),
                            false,
                        );
                    } else {
                        stroke(w, b.origin, &[(x, bh - zero * bh), (x, ya)], 2.0, ca, false);
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let d = t.sizes().scatter_dot * 1.6;
        let mut plot = div().relative().w_full().h(px(self.height)).child(cv);
        let dot =
            |plot: gpui::Div, k: usize, i: usize, v: f32, c: gpui::Hsla, who: &SharedString| {
                let fx = (i as f32 + 0.5) / n as f32;
                let fy = 1.0 - (zero + (norm(v, lo, hi) - zero) * g);
                plot.child(
                    div()
                        .id((SharedString::from(format!("{}-{k}", self.id)), i))
                        .absolute()
                        .left(relative(fx))
                        .top(relative(fy))
                        .ml(px(-d / 2.0))
                        .mt(px(-d / 2.0))
                        .w(px(d))
                        .h(px(d))
                        .rounded_full()
                        .bg(c)
                        .tooltip(DodaiTooltip::text(format!(
                            "{} · {who}: {}",
                            self.items[i].0,
                            fmt_with(&self.fmt, v)
                        ))),
                )
            };
        for (i, (_, v)) in self.items.iter().enumerate() {
            plot = dot(plot, 0, i, *v, ca, &self.names.0);
            if dumbbell {
                let bv = self.other.get(i).copied().unwrap_or(*v);
                plot = dot(plot, 1, i, bv, cb, &self.names.1);
            }
        }
        let labels: Vec<SharedString> = self.items.iter().map(|i| i.0.clone()).collect();
        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(framed(
                &t,
                ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
                labels,
                plot,
            ));
        if dumbbell {
            root = root.child(Legend::new([
                (self.names.0.clone(), t.palette.series_at(0)),
                (self.names.1.clone(), t.palette.series_at(1)),
            ]));
        }
        titled(&t, self.title.clone(), root)
    }
}
crate::entry_render!(Lollipop);

// ─────────────────────────────── SlopeChart ───────────────────────────────

/// Two-state slope chart: one line per item from its left to its right value,
/// coloured by direction. Good for rankings and before/after change.
#[derive(IntoElement)]
pub struct SlopeChart {
    items: Vec<(SharedString, f32, f32)>,
    headers: (SharedString, SharedString),
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}

impl SlopeChart {
    /// Create a new [`SlopeChart`] from `(label, left, right)` items.
    pub fn new<S: Into<SharedString>>(items: impl IntoIterator<Item = (S, f32, f32)>) -> Self {
        Self {
            items: items
                .into_iter()
                .map(|(l, a, b)| {
                    (
                        l.into(),
                        crate::charts::finite_or_zero(a),
                        crate::charts::finite_or_zero(b),
                    )
                })
                .collect(),
            headers: ("Before".into(), "After".into()),
            height: 240.0,
            fmt: None,
            title: None,
        }
    }
    /// Column headers.
    pub fn headers(mut self, a: impl Into<SharedString>, b: impl Into<SharedString>) -> Self {
        self.headers = (a.into(), b.into());
        self
    }
}
chart_cfg!(SlopeChart);

impl SlopeChart {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = crate::motion::grow(window, cx, "SlopeChart");
        let vals = self.items.iter().flat_map(|i| [i.1, i.2]);
        let lo = vals.clone().fold(f32::INFINITY, f32::min);
        let hi = vals.fold(f32::NEG_INFINITY, f32::max);
        let (lo, hi) = if lo.is_finite() && hi > lo {
            let pad = (hi - lo) * 0.08;
            (lo - pad, hi + pad)
        } else {
            (0.0, 1.0)
        };
        let muted = hsla(t.palette.text_muted);
        let dir = |a: f32, b: f32| {
            if b > a {
                up_color(&t)
            } else if b < a {
                down_color(&t)
            } else {
                muted
            }
        };
        let lines: Vec<(f32, f32, gpui::Hsla)> = self
            .items
            .iter()
            .map(|(_, a, b)| (norm(*a, lo, hi), norm(*b, lo, hi), dir(*a, *b)))
            .collect();
        let lines2 = lines.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                for (a, c, col) in &lines2 {
                    let (ya, yc) = (bh - a * bh, bh - (a + (c - a) * g) * bh);
                    stroke(w, b.origin, &[(0.0, ya), (bw * g, yc)], 2.0, *col, false);
                }
            },
        )
        .absolute()
        .size_full();
        let gutter = px(t.sizes().axis_gutter_lg * 2.2);
        let side = |left: bool| {
            let mut c = label_style(&t, div())
                .relative()
                .w(gutter)
                .flex_none()
                .h(px(self.height));
            for (i, (name, a, b)) in self.items.iter().enumerate() {
                let (y, v) = if left {
                    (lines[i].0, *a)
                } else {
                    (lines[i].1, *b)
                };
                let text = if left {
                    format!("{name} {}", fmt_with(&self.fmt, v))
                } else {
                    format!("{} {name}", fmt_with(&self.fmt, v))
                };
                c = c.child(
                    div()
                        .absolute()
                        .top(relative(1.0 - y))
                        .mt(px(-7.0))
                        .w_full()
                        .flex()
                        .when_left(left)
                        .text_color(hsla(t.palette.text))
                        .child(text),
                );
            }
            c
        };
        let head = label_style(&t, div())
            .flex()
            .justify_between()
            .child(self.headers.0.clone())
            .child(self.headers.1.clone());
        let body = div()
            .flex()
            .w_full()
            .child(side(true).pr(px(t.spacing.sm)))
            .child(div().relative().flex_1().h(px(self.height)).child(cv))
            .child(side(false).pl(px(t.spacing.sm)));
        titled(
            &t,
            self.title.clone(),
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .w_full()
                .child(head)
                .child(body),
        )
    }
}
crate::entry_render!(SlopeChart);

/// Alignment helper: left gutters hug the line, right gutters start at it.
trait WhenLeft: Styled + Sized {
    fn when_left(self, left: bool) -> Self {
        if left {
            self.justify_end()
        } else {
            self
        }
    }
}
impl WhenLeft for gpui::Div {}
