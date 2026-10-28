//! Part-to-whole charts: treemap and radial bars.

use crate::charts::{
    chart_cfg, fmt_with, glow_stroke, label_style, norm, stroke, titled, turn_arc, NumFmt,
};
use crate::paint::{scale_alpha, shaped, ShapeStyle};
use crate::panel::DodaiTooltip;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    canvas, div, px, relative, App, Bounds, Hsla, InteractiveElement, IntoElement, ParentElement,
    Pixels, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Treemap rectangles `(x, y, w, h)` for `values` inside `w × h`, in input
/// order; each area is proportional to its value. Pure.
pub fn treemap_layout(values: &[f32], w: f32, h: f32) -> Vec<(f32, f32, f32, f32)> {
    let mut items: Vec<(usize, f32)> = values
        .iter()
        .enumerate()
        .map(|(i, v)| (i, v.max(0.0)))
        .collect();
    items.sort_by(|a, b| b.1.total_cmp(&a.1));
    let mut out = vec![(0.0, 0.0, 0.0, 0.0); values.len()];
    split(&items, 0.0, 0.0, w, h, &mut out);
    out
}

fn split(items: &[(usize, f32)], x: f32, y: f32, w: f32, h: f32, out: &mut [(f32, f32, f32, f32)]) {
    match items {
        [] => {}
        [(i, _)] => out[*i] = (x, y, w, h),
        _ => {
            let total: f32 = items.iter().map(|i| i.1).sum();
            if total <= 0.0 {
                for (i, _) in items {
                    out[*i] = (x, y, 0.0, 0.0);
                }
                return;
            }
            let (mut acc, mut k) = (0.0, 0);
            while k < items.len() - 1 && acc + items[k].1 / 2.0 < total / 2.0 {
                acc += items[k].1;
                k += 1;
            }
            let k = k.clamp(1, items.len() - 1);
            let first: f32 = items[..k].iter().map(|i| i.1).sum();
            let f = first / total;
            if w >= h {
                split(&items[..k], x, y, w * f, h, out);
                split(&items[k..], x + w * f, y, w * (1.0 - f), h, out);
            } else {
                split(&items[..k], x, y, w, h * f, out);
                split(&items[k..], x, y + h * f, w, h * (1.0 - f), out);
            }
        }
    }
}

// ─────────────────────────────── Treemap ───────────────────────────────

/// Treemap: rectangle area ∝ value.
#[derive(IntoElement)]
pub struct Treemap {
    follow: crate::instruments_motion::Follow,
    id: SharedString,
    items: Vec<(SharedString, f32)>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Treemap);
chart_cfg!(Treemap);

impl Treemap {
    crate::instruments_motion::follow_api!();
    /// Create a new [`Treemap`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(
        id: impl Into<SharedString>,
        items: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            corners: None,
            id: id.into(),
            items: items
                .into_iter()
                .map(|(a, b)| (a.into(), crate::charts::finite_or_zero(b)))
                .collect(),
            height: 220.0,
            fmt: None,
            title: None,
        }
    }
}

impl Treemap {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_pairs(window, cx, &self.follow, &mut self.items);
        let g = crate::motion::grow_for(window, cx, &self.follow, "Treemap");
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let vals: Vec<f32> = self.items.iter().map(|i| i.1).collect();
        // Layout in unit space, rendered with relative sizes.
        let rects = treemap_layout(&vals, 1.0, 1.0);
        let mut plot = div().relative().w_full().h(px(self.height));
        let fid = format!("{}-focus", self.id);
        let focus = crate::charts_focus::Focus::new(
            window,
            cx,
            &crate::instruments_motion::Follow::named(&fid),
            self.items.len(),
        );
        for (i, ((x, y, w, h), (name, v))) in rects.iter().zip(&self.items).enumerate() {
            let f = focus.item(i);
            let style =
                ShapeStyle::from_tokens(&t, Some(t.palette.series_at(i)), None, false, false);
            let gi = crate::motion::reveal(g, i, self.items.len());
            let tile = div()
                .id((self.id.clone(), i))
                .opacity(f.alpha)
                .absolute()
                .left(relative(*x + *w * (1.0 - gi) / 2.0))
                .top(relative(*y + *h * (1.0 - gi) / 2.0))
                .w(relative(*w * gi))
                .h(relative(*h * gi))
                .p(px(1.5 - 1.2 * f.lift))
                .tooltip(DodaiTooltip::text(format!(
                    "{name}: {}",
                    fmt_with(&self.fmt, *v)
                )))
                .child(
                    shaped(&t.shape, style)
                        .size_full()
                        .overflow_hidden()
                        .p(px(t.spacing.xs))
                        .font_family(first_family(t.typography.mono))
                        .text_size(px(t.typography.size_xs))
                        .text_color(hsla(t.palette.on_primary))
                        .child(format!("{name}\n{}", fmt_with(&self.fmt, *v))),
                );
            plot = plot.child(focus.bind(tile, i));
        }
        titled(&t, self.title, div().w_full().child(plot))
    }
}

crate::entry_render!(Treemap);

// ─────────────────────────────── RadialBar ───────────────────────────────

/// Concentric progress rings, one per `(name, value, max)`.
#[derive(IntoElement)]
pub struct RadialBar {
    follow: crate::instruments_motion::Follow,
    glow: crate::charts::ChartGlow,
    rings: Vec<(SharedString, f32, f32)>,
    size: f32,
    legend: bool,
}

impl RadialBar {
    crate::instruments_motion::follow_api!();
    /// The glow ("shadow") under the lines: the theme's own, none, or a radius.
    pub fn glow(mut self, g: crate::charts::ChartGlow) -> Self {
        self.glow = g;
        self
    }
    /// Create a new [`RadialBar`].
    #[track_caller]
    pub fn new<S: Into<SharedString>>(rings: impl IntoIterator<Item = (S, f32, f32)>) -> Self {
        Self {
            follow: crate::instruments_motion::Follow::here(),
            glow: crate::charts::ChartGlow::Theme,
            rings: rings
                .into_iter()
                .map(|(a, b, c)| (a.into(), b, c))
                .collect(),
            size: 160.0,
            legend: true,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Show or hide the legend.
    pub fn legend(mut self, l: bool) -> Self {
        self.legend = l;
        self
    }
}

/// Which ring of a [`RadialBar`] lies under a pointer `d` px from its centre, if any. Rings are
/// laid out exactly as painted: outermost first, each `ring` px thick with a 2 px step.
fn ring_hit(d: f32, size: f32, n: usize) -> Option<usize> {
    let (c, ring) = ring_metrics(size, n);
    (0..n).find(|i| {
        let r = c - ring * (*i as f32 + 0.5) - 2.0 * *i as f32 - 1.0;
        r > 1.0 && (d - r).abs() <= ring / 2.0
    })
}

/// Centre and ring thickness for `n` rings in a box of `size` px.
fn ring_metrics(size: f32, n: usize) -> (f32, f32) {
    (size / 2.0, (size / 2.0 / (n as f32 + 1.0)).min(14.0))
}

impl RadialBar {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        crate::instruments_motion::follow_triples(window, cx, &self.follow, &mut self.rings);
        let t = theme_of(cx);
        let g = crate::motion::grow_for(window, cx, &self.follow, "RadialBar");
        let focus = crate::charts_focus::Focus::new(window, cx, &self.follow, self.rings.len());
        let n = self.rings.len().max(1);
        let size = self.size;
        let fr: Vec<f32> = self.rings.iter().map(|r| norm(r.1, 0.0, r.2) * g).collect();
        let cols: Vec<Hsla> = (0..n).map(|i| hsla(t.palette.series_at(i))).collect();
        let track = hsla(t.palette.border);
        let glow = self.glow.resolve(&t);
        let looks: Vec<(f32, f32)> = (0..self.rings.len())
            .map(|i| (focus.item(i).alpha, focus.item(i).lift))
            .collect();
        let bounds: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>> = Default::default();
        let paint_bounds = bounds.clone();
        let cv = canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                paint_bounds.set(b);
                let (c, ring) = ring_metrics(size, n);
                for i in 0..fr.len() {
                    let r = c - ring * (i as f32 + 0.5) - 2.0 * i as f32 - 1.0;
                    if r <= 1.0 {
                        break;
                    }
                    let (alpha, lift) = looks.get(i).copied().unwrap_or((1.0, 0.0));
                    let thick = ring - 2.0 + lift * 2.0;
                    stroke(
                        w,
                        b.origin,
                        &turn_arc(c, c, r, 0.0, 0.999),
                        thick,
                        scale_alpha(track, 0.6),
                        false,
                    );
                    if fr[i] > 0.001 {
                        glow_stroke(
                            w,
                            b.origin,
                            &turn_arc(c, c, r, 0.0, fr[i].min(0.999)),
                            thick,
                            scale_alpha(cols[i], alpha),
                            glow,
                        );
                    }
                }
            },
        )
        .absolute()
        .size_full();
        let count = self.rings.len();
        let (st_move, st_click, st_leave) = (focus.state(), focus.state(), focus.state());
        let (b_move, b_click) = (bounds.clone(), bounds);
        let at = move |b: &std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
                       p: gpui::Point<Pixels>|
              -> Option<usize> {
            let b = b.get();
            let (dx, dy): (f32, f32) = ((p.x - b.origin.x).into(), (p.y - b.origin.y).into());
            ring_hit(
                ((dx - size / 2.0).powi(2) + (dy - size / 2.0).powi(2)).sqrt(),
                size,
                count,
            )
        };
        let sid = self.follow.sub("rb").element_id(window, cx).to_string();
        // The ring under the pointer (or the selected one) is read out in the middle.
        let s = focus.state().read(cx).to_owned();
        let shown = s.hover.or(s.sel).filter(|i| *i < self.rings.len());
        let mut disc = div()
            .id(SharedString::from(format!("{sid}-disc")))
            .relative()
            .w(px(size))
            .h(px(size))
            .child(cv)
            .on_mouse_move(move |e, w, cx| {
                crate::charts_focus::Focus::set_hover(&st_move, at(&b_move, e.position), w, cx)
            })
            .on_hover(move |over, w, cx| {
                if !*over {
                    crate::charts_focus::Focus::set_hover(&st_leave, None, w, cx);
                }
            })
            .on_click(move |e, w, cx| {
                crate::charts_focus::Focus::toggle(&st_click, at(&b_click, e.position()), w, cx)
            });
        if let Some(i) = shown {
            let (name, v, m) = &self.rings[i];
            disc = disc.child(
                div()
                    .absolute()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .font_family(first_family(t.typography.display))
                    .text_color(hsla(t.palette.text))
                    .child(
                        div()
                            .text_size(px(t.typography.size_lg))
                            .child(format!("{:.0}%", norm(*v, 0.0, *m) * 100.0)),
                    )
                    .child(
                        div()
                            .text_size(px(t.typography.size_sm))
                            .text_color(hsla(t.palette.text_muted))
                            .child(name.clone()),
                    ),
            );
        }
        let mut root = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.lg))
            .child(disc);
        if self.legend {
            root = root.child(div().flex().flex_col().gap(px(t.spacing.xs)).children(
                self.rings.iter().enumerate().map(|(i, (n, v, m))| {
                    let row = label_style(&t, div())
                        .id(SharedString::from(format!("{sid}-leg-{i}")))
                        .flex()
                        .items_center()
                        .gap(px(t.spacing.xs))
                        .child(div().w(px(10.0)).h(px(10.0)).bg(cols_at(&t, i)))
                        .child(format!("{n} · {:.0}%", norm(*v, 0.0, *m) * 100.0));
                    focus.bind(row, i)
                }),
            ));
        }
        root
    }
}

crate::entry_render!(RadialBar);

fn cols_at(t: &Theme, i: usize) -> Hsla {
    hsla(t.palette.series_at(i))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_hit_finds_the_painted_ring() {
        let (c, ring) = ring_metrics(160.0, 3);
        for i in 0..3 {
            let r = c - ring * (i as f32 + 0.5) - 2.0 * i as f32 - 1.0;
            assert_eq!(ring_hit(r, 160.0, 3), Some(i));
        }
        assert_eq!(ring_hit(0.0, 160.0, 3), None);
        assert_eq!(ring_hit(500.0, 160.0, 3), None);
        assert_eq!(ring_hit(10.0, 160.0, 0), None);
    }

    #[test]
    fn treemap_areas_proportional_and_tile() {
        let v = [6.0, 3.0, 2.0, 1.0];
        let r = treemap_layout(&v, 12.0, 6.0);
        let total: f32 = r.iter().map(|(_, _, w, h)| w * h).sum();
        assert!((total - 72.0).abs() < 1e-3);
        for ((_, _, w, h), val) in r.iter().zip(v) {
            assert!((w * h - val / 12.0 * 72.0).abs() < 1e-3);
        }
    }
}
