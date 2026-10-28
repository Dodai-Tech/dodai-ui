//! Flow charts: funnel and waterfall.

use crate::charts::{chart_cfg, fill, fmt_with, framed, nice_ticks, norm, titled, NumFmt};
use crate::paint::{shaped, ShapeStyle};
use crate::panel::DodaiTooltip;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::{Color, Tone};
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Running `(start, end)` for each delta of a waterfall. Pure.
pub fn waterfall_steps(deltas: &[f32]) -> Vec<(f32, f32)> {
    let mut acc = 0.0;
    deltas
        .iter()
        .map(|d| {
            let s = acc;
            acc += d;
            (s, acc)
        })
        .collect()
}

// ─────────────────────────────── Funnel ───────────────────────────────

/// Funnel of decreasing stages, drawn as joined trapezoids.
#[derive(IntoElement)]
pub struct Funnel {
    follow: crate::instruments_motion::Follow,
    stages: Vec<(SharedString, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(Funnel);

impl Funnel {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Funnel`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(stages: impl IntoIterator<Item = (S, f32)>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            stages: stages.into_iter().map(|(a, b)| (a.into(), b)).collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl Funnel {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_pairs(window, cx, &self.follow, &mut self.stages);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Funnel");
        let t = theme_of(cx);
        let n = self.stages.len().max(1);
        let maxv = self
            .stages
            .iter()
            .map(|s| s.1)
            .fold(f32::MIN_POSITIVE, f32::max);
        let fr: Vec<f32> = self
            .stages
            .iter()
            .map(|s| (s.1 / maxv).clamp(0.0, 1.0))
            .collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let focus = crate::charts_focus::Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|i| focus.item(i)).collect();
        let fr2 = fr.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
                let sh = bh / n as f32;
                for (i, col) in cols.iter().enumerate() {
                    let cur = fr2.get(i).copied().unwrap_or(0.0);
                    let next = fr2.get(i + 1).copied().unwrap_or(cur * 0.8);
                    let gi = crate::motion::reveal(g, i, n);
                    let lk = looks[i];
                    let pop = 1.0 + 0.05 * lk.lift;
                    let (w0, w1) = (cur * bw * gi * pop, next * bw * gi * pop);
                    let (y0, y1) = (i as f32 * sh + 1.0, (i + 1) as f32 * sh - 1.0);
                    let c = bw / 2.0;
                    fill(
                        w,
                        b.origin,
                        &[
                            (c - w0 / 2.0, y0),
                            (c + w0 / 2.0, y0),
                            (c + w1 / 2.0, y1),
                            (c - w1 / 2.0, y1),
                        ],
                        crate::paint::scale_alpha(*col, lk.alpha),
                    );
                }
            },
        )
        .absolute()
        .size_full();
        let row_key = self.follow.element_id(window, cx);
        let mut labels = div().absolute().size_full().flex().flex_col();
        for (i, (name, v)) in self.stages.iter().enumerate() {
            let pct = fr[i] * 100.0;
            labels = labels.child(
                focus.bind(
                    div()
                        .id(gpui::ElementId::from((
                            row_key.clone(),
                            SharedString::from(format!("row{i}")),
                        )))
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .font_family(first_family(t.typography.mono))
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.on_primary))
                        .child(format!("{name} · {} ({pct:.0}%)", fmt_with(&self.fmt, *v))),
                    i,
                ),
            );
        }
        let root = div()
            .relative()
            .w_full()
            .h(px(self.height))
            .child(cv)
            .child(labels);
        titled(&t, self.title, div().w_full().child(root))
    }
}

crate::entry_render!(Funnel);

// ─────────────────────────────── Waterfall ───────────────────────────────

/// Waterfall of deltas; increases green, decreases red, optional total bar.
#[derive(IntoElement)]
pub struct Waterfall {
    follow: crate::instruments_motion::Follow,
    annotations: Vec<crate::charts_annotate::Annotation>,
    id: SharedString,
    steps: Vec<(SharedString, f32)>,
    total: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Waterfall);
chart_cfg!(Waterfall);

impl Waterfall {
    crate::instruments_motion::follow_api!();
    /// Threshold lines and highlighted bands drawn over the plot.
    pub fn annotations(
        mut self,
        a: impl IntoIterator<Item = crate::charts_annotate::Annotation>,
    ) -> Self {
        self.annotations = a.into_iter().collect();
        self
    }
    /// Create a new [`Waterfall`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        steps: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            annotations: Vec::new(),
            corners: None,
            id: id.into(),
            steps: steps
                .into_iter()
                .map(|(a, b)| (a.into(), crate::charts::finite_or_zero(b)))
                .collect(),
            total: true,
            height: 200.0,
            fmt: None,
            title: None,
        }
    }
    /// Append a "Total" bar (default on).
    pub fn total(mut self, on: bool) -> Self {
        self.total = on;
        self
    }
}

impl Waterfall {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_pairs(window, cx, &self.follow, &mut self.steps);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Waterfall");
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let deltas: Vec<f32> = self.steps.iter().map(|s| s.1).collect();
        let st = waterfall_steps(&deltas);
        let end = st.last().map_or(0.0, |s| s.1);
        let lo = st.iter().flat_map(|s| [s.0, s.1]).fold(0.0f32, f32::min);
        let hi = st.iter().flat_map(|s| [s.0, s.1]).fold(0.0f32, f32::max);
        let ticks = nice_ticks(lo, hi, 4);
        let (lo, hi) = (*ticks.first().unwrap_or(&lo), *ticks.last().unwrap_or(&hi));
        let grid = hsla(t.palette.border);
        let mut plot = div().relative().w_full().h(px(self.height));
        plot = plot.child(crate::charts_annotate::layer(
            &t,
            &self.annotations,
            (lo, hi),
            &crate::charts_annotate::category_x(self.steps.len() + usize::from(self.total)),
        ));

        for v in &ticks {
            plot = plot.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .h(px(1.0))
                    .bottom(relative(norm(*v, lo, hi)))
                    .bg(grid),
            );
        }
        let mut bars: Vec<(SharedString, f32, f32, Color, f32)> = self
            .steps
            .iter()
            .zip(&st)
            .map(|((n, d), (a, b))| {
                let c = if *d >= 0.0 {
                    t.palette.tone(Tone::Success)
                } else {
                    t.palette.tone(Tone::Danger)
                };
                (n.clone(), a.min(*b), a.max(*b), c, *d)
            })
            .collect();
        if self.total {
            bars.push((
                "Total".into(),
                end.min(0.0),
                end.max(0.0),
                t.palette.primary,
                end,
            ));
        }
        let mut row = div().flex().size_full();
        let names: Vec<SharedString> = bars.iter().map(|b| b.0.clone()).collect();
        let focus = crate::charts_focus::Focus::new(window, cx, &self.follow, bars.len());
        for (i, (n, a, b, c, d)) in bars.into_iter().enumerate() {
            let lf = focus.item(i);
            let style = ShapeStyle::from_tokens(&t, Some(c), None, false, false);
            let nb = names.len();
            let (fa, fb) = (norm(a, lo, hi), norm(b, lo, hi));
            let fb = fa + (fb - fa) * crate::motion::reveal(g, i, nb);
            row = row.child(
                div().relative().flex_1().h_full().child(
                    focus
                        .bind(div().id((self.id.clone(), i)), i)
                        .opacity(lf.alpha)
                        .absolute()
                        .left(relative(0.15 - 0.05 * lf.lift))
                        .w(relative(0.7 + 0.1 * lf.lift))
                        .bottom(relative(fa))
                        .h(relative((fb - fa).max(0.004)))
                        .tooltip(DodaiTooltip::text(format!(
                            "{n}: {}{}",
                            if d > 0.0 && i < self.steps.len() {
                                "+"
                            } else {
                                ""
                            },
                            fmt_with(&self.fmt, d)
                        )))
                        .child(shaped(&t.shape, style).size_full()),
                ),
            );
        }
        plot = plot.child(row);
        let root = div().flex().flex_col().w_full().child(framed(
            &t,
            ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect(),
            names,
            plot,
        ));
        titled(&t, self.title, root)
    }
}

crate::entry_render!(Waterfall);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waterfall_accumulates() {
        assert_eq!(
            waterfall_steps(&[5.0, -2.0, 4.0]),
            vec![(0.0, 5.0), (5.0, 3.0), (3.0, 7.0)]
        );
    }
}
