//! [`ShardMap`]: an outline cut into one convex shard per item, sized by value and grouped.

use crate::gpui::common::{
    chart_cfg, fmt_with, plot_box, tracked_canvas, CanvasText, Legend, NumFmt,
};
use crate::gpui::focus::{readout, Focus};
use crate::shards::{self, Outline, Poly, Shard};
use dodai_ui_gpui::paint::{fill, scale_alpha, stroke};
use dodai_ui_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// One group of a [`ShardMap`]: a name and its `(label, value)` items.
#[derive(Clone, Debug)]
pub struct ShardGroup {
    name: SharedString,
    items: Vec<(SharedString, f32)>,
}

impl ShardGroup {
    /// The group `name` of `items`.
    pub fn new<S: Into<SharedString>>(
        name: impl Into<SharedString>,
        items: impl IntoIterator<Item = (S, f32)>,
    ) -> Self {
        Self {
            name: name.into(),
            items: items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
        }
    }
}

/// Shard map (a Voronoi treemap): an outline (a circle by default, or a hexagon or square)
/// cut into one convex cell per item, its area proportional to the value; each group's items
/// share a region, ringed by a heavier border. Big shards carry their name and value. Hover a shard for its share;
/// click to keep it picked.
#[derive(IntoElement)]
pub struct ShardMap {
    follow: dodai_ui_gpui::follow::Follow,
    groups: Vec<ShardGroup>,
    outline: Outline,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ShardMap);

impl ShardMap {
    dodai_ui_gpui::follow_api!();
    /// Create a new [`ShardMap`] of `groups`.
    #[track_caller]
    pub fn new(groups: impl IntoIterator<Item = ShardGroup>) -> Self {
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            groups: groups.into_iter().collect(),
            outline: Outline::Circle,
            height: 520.0,
            fmt: None,
            title: None,
        }
    }
    /// The outline to fill (default: a circle).
    pub fn outline(mut self, o: Outline) -> Self {
        self.outline = o;
        self
    }
}

/// Unit-outline coordinates to canvas pixels: centred, radius `r`.
fn place(p: &[(f32, f32)], (cx, cy): (f32, f32), r: f32) -> Poly {
    p.iter().map(|(x, y)| (cx + x * r, cy + y * r)).collect()
}

/// The horizontal chord of convex `p` at height `y`: its left and right ends.
fn chord(p: &[(f32, f32)], y: f32) -> Option<(f32, f32)> {
    let xs: Vec<f32> = (0..p.len())
        .filter_map(|i| {
            let (a, b) = (p[i], p[(i + 1) % p.len()]);
            let (lo, hi) = (a.1.min(b.1), a.1.max(b.1));
            (lo <= y && y <= hi && hi > lo).then(|| a.0 + (b.0 - a.0) * (y - a.1) / (b.1 - a.1))
        })
        .collect();
    crate::extent(xs).filter(|(l, r)| l < r)
}

/// The layout for `values` in `outline`, kept between frames: the Voronoi relaxation is too
/// slow to redo on every frame of an animation.
fn cached_layout(
    window: &mut Window,
    cx: &mut App,
    follow: &dodai_ui_gpui::follow::Follow,
    values: &[Vec<f32>],
    outline: &[(f32, f32)],
) -> Rc<(Vec<Shard>, Vec<Poly>)> {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    for g in values {
        g.len().hash(&mut h);
        g.iter().for_each(|v| v.to_bits().hash(&mut h));
    }
    outline.len().hash(&mut h);
    let key = h.finish();
    let id = follow.sub("layout").element_id(window, cx);
    let slot = window.with_id(id, |w| {
        w.use_keyed_state("shards", cx, |_, _| {
            None::<(u64, Rc<(Vec<Shard>, Vec<Poly>)>)>
        })
    });
    if let Some((k, laid)) = slot.read(cx).as_ref() {
        if *k == key {
            return laid.clone();
        }
    }
    let laid = Rc::new(shards::layout(values, outline));
    let keep = laid.clone();
    slot.update(cx, |s, _| *s = Some((key, keep)));
    laid
}

/// The canvas centre and outline radius for a `w`×`h` box.
fn fit(w: f32, h: f32) -> ((f32, f32), f32) {
    ((w / 2.0, h / 2.0), (w.min(h) / 2.0 - 4.0).max(1.0))
}

impl ShardMap {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "ShardMap");
        let values: Vec<Vec<f32>> = self
            .groups
            .iter()
            .map(|g| g.items.iter().map(|i| i.1).collect())
            .collect();
        let outline = self.outline.polygon();
        let laid = cached_layout(window, cx, &self.follow, &values, &outline);
        let (cells, regions) = (laid.0.clone(), laid.1.clone());
        let n = cells.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        // Pastel cells, a little stronger when focused; text and borders in whichever of the
        // theme's ink and page colours reads best on them.
        let pastel = |gi: usize| t.palette.series_at(gi).lighten(0.45);
        let fills: Vec<Hsla> = cells.iter().map(|c| hsla(pastel(c.group))).collect();
        let fills_hot: Vec<Hsla> = cells
            .iter()
            .map(|c| hsla(t.palette.series_at(c.group).lighten(0.2)))
            .collect();
        let inks: Vec<Hsla> = cells.iter().map(|c| hsla(t.on(pastel(c.group)))).collect();
        let lines = inks.clone();
        let region_lines: Vec<Hsla> = (0..self.groups.len())
            .map(|gi| hsla(t.on(pastel(gi))))
            .collect();
        let total: f32 = values
            .iter()
            .flatten()
            .filter(|v| v.is_finite() && **v > 0.0)
            .sum();
        let text: Vec<(SharedString, String)> = cells
            .iter()
            .map(|c| {
                let (l, v) = &self.groups[c.group].items[c.item];
                (l.clone(), fmt_with(&self.fmt, *v))
            })
            .collect();
        let gap = hsla(t.palette.text);
        let (big, small) = (t.typography.size_md, t.typography.size_xs);
        let display = t.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_cells: Vec<Shard> = cells.clone();
        let hatcher = crate::gpui::common::Hatcher::new(&t);
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (centre, r) = fit(b.size.width.into(), b.size.height.into());
            let r = r * (0.85 + 0.15 * g.min(1.0));
            let a = g.min(1.0);
            for (k, c) in paint_cells.iter().enumerate() {
                if c.poly.len() < 3 {
                    continue;
                }
                let lk = looks[k];
                let p = place(&c.poly, centre, r);
                let tone = if lk.lift > 0.01 {
                    fills_hot[k]
                } else {
                    fills[k]
                };
                fill(w, b.origin, &p, scale_alpha(tone, lk.alpha * a));
                hatcher.over(w, b.origin, &p, k);
                stroke(w, b.origin, &p, 1.0, scale_alpha(lines[k], 0.55 * a), true);
            }
            for (gi, region) in regions.iter().enumerate().filter(|(_, p)| p.len() > 2) {
                let edge = region_lines.get(gi).copied().unwrap_or(gap);
                stroke(
                    w,
                    b.origin,
                    &place(region, centre, r),
                    3.0,
                    scale_alpha(edge, a),
                    true,
                );
            }
            if g < 0.9 {
                return;
            }
            for (k, c) in paint_cells.iter().enumerate() {
                if c.poly.len() < 3 {
                    continue;
                }
                let p = place(&c.poly, centre, r);
                let (x, y) = shards::centroid(&p);
                let side = shards::area(&p).sqrt();
                let room = chord(&p, y).map_or(0.0, |(l, r)| r - l) - 8.0;
                let size = (side * 0.11).clamp(small * 0.8, big);
                let name = CanvasText::display(&display, size);
                let value = CanvasText::display(&display, size * 1.35).bold();
                let ink = scale_alpha(inks[k], looks[k].alpha);
                let (lh, vh) = (name.size_line(), value.size_line());
                if side < (lh + vh) * 1.4 {
                    continue;
                }
                if name.centred(w, cx, &text[k].0, (b.origin, (x, y - vh * 0.45)), room, ink) {
                    value.centred(w, cx, &text[k].1, (b.origin, (x, y + lh * 0.45)), room, ink);
                }
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("map").element_id(window, cx),
            self.height,
            cv,
        );
        let unit = cells.clone();
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let ((cx, cy), r) = fit(w, h);
            shards::hit(&unit, ((x - cx) / r, (y - cy) / r))
        });
        let line = focus.shown(cx).and_then(|k| cells.get(k)).map(|c| {
            let grp = &self.groups[c.group];
            let (l, v) = &grp.items[c.item];
            let share = if total > 0.0 { v / total * 100.0 } else { 0.0 };
            format!(
                "{l} · {} · {share:.1}% of all · {}",
                fmt_with(&self.fmt, *v),
                grp.name
            )
        });
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, line, "Hover a shard"))
            .child(Legend::new(
                self.groups
                    .iter()
                    .enumerate()
                    .map(|(i, g)| (g.name.clone(), t.palette.series_at(i))),
            ));
        crate::gpui::common::titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(ShardMap);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_outline_fits_the_box() {
        let ((cx, cy), r) = fit(600.0, 400.0);
        assert_eq!((cx, cy, r), (300.0, 200.0, 196.0));
        assert!(fit(0.0, f32::NAN).1 >= 1.0);
        assert_eq!(place(&[(1.0, -1.0)], (10.0, 10.0), 5.0), [(15.0, 5.0)]);
        let sq = [(0.0, 0.0), (4.0, 0.0), (4.0, 2.0), (0.0, 2.0)];
        assert_eq!(chord(&sq, 1.0), Some((0.0, 4.0)));
        assert_eq!(chord(&sq, 3.0), None);
    }
}
