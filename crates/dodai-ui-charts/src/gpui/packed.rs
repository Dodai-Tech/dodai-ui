//! Packed charts in the infographic style: [`ProportionalSquares`] (one square per item, area by value) and [`CirclePack`] (a
//! hierarchy as circles inside circles).

use crate::gpui::common::{
    chart_cfg, fill, fmt_with, marker_points, plot_box, stroke, titled, tracked_canvas, CanvasText,
    Groups, Marker, NumFmt,
};
use crate::gpui::focus::{readout, Focus};
use crate::gpui::hierarchy::ChartNode;
use crate::pack::{circle_pack, square_pack, Disc};
use dodai_ui_gpui::paint::scale_alpha;
use dodai_ui_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// Groups of labelled items: `(group, [(label, values…)])`.
type Grouped<T> = Vec<(SharedString, Vec<(SharedString, T)>)>;

fn grouped<G: Into<SharedString>, L: Into<SharedString>, T>(
    groups: impl IntoIterator<Item = (G, Vec<(L, T)>)>,
) -> Grouped<T> {
    groups
        .into_iter()
        .map(|(g, items)| {
            (
                g.into(),
                items.into_iter().map(|(l, v)| (l.into(), v)).collect(),
            )
        })
        .collect()
}

// ─────────────────────────────── ProportionalSquares ───────────────────────────────

/// One square per item, its area proportional to the value, packed largest first: sizes that
/// differ by orders of magnitude stay comparable side by side ("the billion-pound-o-gram").
#[derive(IntoElement)]
pub struct ProportionalSquares {
    follow: dodai_ui_gpui::follow::Follow,
    groups: Grouped<f32>,
    filter: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(ProportionalSquares);

impl ProportionalSquares {
    dodai_ui_gpui::follow_api!();
    /// Let the legend hide and show groups (default on; needs two groups or more).
    pub fn filter(mut self, on: bool) -> Self {
        self.filter = on;
        self
    }
    /// Create new [`ProportionalSquares`] from `(group, [(label, value)])`; each group takes a
    /// series colour.
    #[track_caller]
    pub fn new<G: Into<SharedString>, L: Into<SharedString>>(
        groups: impl IntoIterator<Item = (G, Vec<(L, f32)>)>,
    ) -> Self {
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            groups: grouped(groups),
            filter: true,
            height: 260.0,
            fmt: None,
            title: None,
        }
    }
}

impl ProportionalSquares {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let flat: Vec<f32> = self
            .groups
            .iter()
            .flat_map(|g| g.1.iter().map(|i| i.1))
            .collect();
        let mut shown =
            dodai_ui_gpui::follow::follow_all(window, cx, &self.follow, &flat).into_iter();
        for (_, v) in self.groups.iter_mut().flat_map(|g| g.1.iter_mut()) {
            *v = shown.next().unwrap_or(*v);
        }
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "ProportionalSquares");
        let groups = Groups::new(
            window,
            cx,
            &self.follow,
            self.groups.iter().map(|g| Some(&g.0)),
            self.filter,
        );
        let items: Vec<(usize, SharedString, f32)> = self
            .groups
            .iter()
            .enumerate()
            .flat_map(|(k, (_, it))| it.iter().map(move |(l, v)| (k, l.clone(), *v)))
            .collect();
        let total: f32 = items
            .iter()
            .map(|i| i.2)
            .filter(|v| v.is_finite() && *v > 0.0)
            .sum();
        // Shares stay of the whole, with groups hidden or not.
        let items: Vec<_> = items.into_iter().filter(|i| groups.shown(i.0)).collect();
        let focus = Focus::new(window, cx, &self.follow, items.len());
        let looks: Vec<_> = (0..items.len()).map(|i| focus.item(i)).collect();
        let cols: Vec<Hsla> = (0..self.groups.len().max(1))
            .map(|i| hsla(t.palette.series_at(i)))
            .collect();
        let inks: Vec<Hsla> = (0..self.groups.len().max(1))
            .map(|i| hsla(t.on(t.palette.series_at(i))))
            .collect();
        let text = CanvasText::of(&t);
        let gap = t.spacing.xs / 2.0;
        let fmt = self.fmt.clone();
        let rects: Rc<RefCell<Vec<(f32, f32, f32)>>> = Default::default();
        let painted = rects.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_items = items.clone();
        let hatcher = crate::gpui::common::Hatcher::new(&t);
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let values: Vec<f32> = paint_items.iter().map(|i| i.2).collect();
            let sq = square_pack(&values, bh / bw.max(1.0));
            // Largest first, as packed.
            let mut order: Vec<usize> = (0..sq.len()).collect();
            order.sort_by(|a, b| sq[*b].side.total_cmp(&sq[*a].side));
            let at: Vec<(f32, f32, f32)> = sq
                .iter()
                .map(|s| (s.x * bw, s.y * bw, s.side * bw))
                .collect();
            for (rank, &i) in order.iter().enumerate() {
                let (x, y, side) = at[i];
                let gi = dodai_ui_core::motion::reveal(g, rank, order.len());
                let s = (side - gap) * gi;
                if s <= 0.5 {
                    continue;
                }
                let lk = looks[i];
                let k = paint_items[i].0;
                let (cx_, cy) = (x + side / 2.0, y + side / 2.0);
                let (x0, y0) = (cx_ - s / 2.0, cy - s / 2.0);
                let square = crate::gpui::common::rect(x0, y0, s, s);
                fill(w, b.origin, &square, scale_alpha(cols[k], 0.9 * lk.alpha));
                hatcher.over(w, b.origin, &square, k);
                if gi < 1.0 {
                    continue;
                }
                let room = s - 8.0;
                let label = &paint_items[i].1;
                let value = fmt_with(&fmt, paint_items[i].2);
                // Label and value when two lines fit, else the label alone.
                let line = text.size_line();
                if s >= line * 3.0
                    && text.centred(
                        w,
                        cx,
                        label,
                        (b.origin, (cx_, cy - line / 2.0)),
                        room,
                        inks[k],
                    )
                {
                    text.centred(
                        w,
                        cx,
                        &value,
                        (b.origin, (cx_, cy + line / 2.0)),
                        room,
                        inks[k],
                    );
                } else if s >= line * 1.6 {
                    text.centred(w, cx, label, (b.origin, (cx_, cy)), room, inks[k]);
                }
            }
            *painted.borrow_mut() = at;
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("squares").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, y, _, _| {
            rects.borrow().iter().position(|(rx, ry, s)| {
                *s > 0.0 && (*rx..rx + s).contains(&x) && (*ry..ry + s).contains(&y)
            })
        });
        let names: Vec<SharedString> = self.groups.iter().map(|g| g.0.clone()).collect();
        let shown = focus.shown(cx).and_then(|i| {
            let (k, l, v) = items.get(i)?;
            let share = if total > 0.0 { v / total * 100.0 } else { 0.0 };
            Some(format!(
                "{l} ({}) · {} ({share:.1}%)",
                names.get(*k)?,
                fmt_with(&self.fmt, *v)
            ))
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(plot)
            .children(groups.legend(&t).filter(|_| names.len() > 1))
            .child(readout(&t, shown, &groups.hint("Hover a square")));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(ProportionalSquares);

// ─────────────────────────────── CirclePack ───────────────────────────────

/// A hierarchy as circles inside circles: each leaf a circle sized (by area) by its value,
/// each branch a ring around its children. Shows the parts and the whole at once, like a
/// treemap with softer edges.
#[derive(IntoElement)]
pub struct CirclePack {
    follow: dodai_ui_gpui::follow::Follow,
    root: ChartNode,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(CirclePack);

impl CirclePack {
    dodai_ui_gpui::follow_api!();
    /// Create a new [`CirclePack`] of everything below `root`.
    #[track_caller]
    pub fn new(root: ChartNode) -> Self {
        Self {
            follow: dodai_ui_gpui::follow::Follow::here(),
            root,
            height: 300.0,
            fmt: None,
            title: None,
        }
    }
}

/// "a › b › c · value (share)" for disc `i`.
fn disc_text(ds: &[Disc], i: usize, total: f32, fmt: &Option<NumFmt>) -> Option<String> {
    let d = ds.get(i)?;
    let mut path = vec![d.label.clone()];
    let mut at = d.parent;
    while let Some(p) = at {
        path.push(ds[p].label.clone());
        at = ds[p].parent;
    }
    path.reverse();
    let share = if total > 0.0 {
        d.value / total * 100.0
    } else {
        0.0
    };
    Some(format!(
        "{} · {} ({share:.1}%)",
        path.join(" › "),
        fmt_with(fmt, d.value)
    ))
}

impl CirclePack {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_ui_gpui::motion::grow_for(window, cx, &self.follow, "CirclePack");
        let mut discs = circle_pack(&self.root);
        // New data glides: every circle eases to its new place and size.
        let flat: Vec<f32> = discs.iter().flat_map(|d| [d.x, d.y, d.r]).collect();
        let mut shown =
            dodai_ui_gpui::follow::follow_all(window, cx, &self.follow.sub("discs"), &flat)
                .into_iter();
        for d in &mut discs {
            d.x = shown.next().unwrap_or(d.x);
            d.y = shown.next().unwrap_or(d.y);
            d.r = shown.next().unwrap_or(d.r);
        }
        let total = self.root.total();
        let levels = discs.iter().map(|d| d.depth + 1).max().unwrap_or(1);
        let focus = Focus::new(window, cx, &self.follow, discs.len());
        let looks: Vec<_> = (0..discs.len()).map(|i| focus.item(i)).collect();
        // Leaves in their branch colour, paler with depth; branches a faint wash with a rim.
        let style: Vec<(Hsla, Hsla, Hsla)> = discs
            .iter()
            .map(|d| {
                let base = t.palette.series_at(d.branch);
                if d.leaf {
                    let c = base.mix(t.palette.surface, (0.15 * d.depth as f32).min(0.45));
                    (hsla(c), hsla(c), hsla(t.on(c)))
                } else {
                    let wash = base.mix(t.palette.surface, 0.86);
                    (
                        hsla(wash),
                        hsla(base.mix(t.palette.surface, 0.5)),
                        hsla(t.palette.text),
                    )
                }
            })
            .collect();
        let text = CanvasText::of(&t);
        let geo: Vec<(f32, f32, f32, usize, bool, SharedString)> = discs
            .iter()
            .map(|d| {
                (
                    d.x,
                    d.y,
                    d.r,
                    d.depth,
                    d.leaf,
                    SharedString::from(d.label.clone()),
                )
            })
            .collect();
        let hit: Rc<Cell<(f32, f32, f32)>> = Default::default();
        let ph = hit.clone();
        let hit_geo: Vec<(f32, f32, f32)> = discs.iter().map(|d| (d.x, d.y, d.r)).collect();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let hatcher = crate::gpui::common::Hatcher::new(&t);
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let side = bw.min(bh);
            let (ox, oy) = ((bw - side) / 2.0, (bh - side) / 2.0);
            ph.set((ox, oy, side));
            for (i, (x, y, r, depth, leaf, label)) in geo.iter().enumerate() {
                // Levels open outwards in: rings first, then what they hold.
                let gi = dodai_ui_core::motion::reveal(g, *depth, levels);
                if gi <= 0.0 {
                    continue;
                }
                let lk = looks[i];
                let (cx_, cy, rr) = (ox + x * side, oy + y * side, r * side * gi);
                let ring = marker_points(Marker::Dot, cx_, cy, rr);
                let (body, rim, ink) = style[i];
                let alpha = if *leaf { 0.9 } else { 1.0 } * lk.alpha;
                fill(w, b.origin, &ring, scale_alpha(body, alpha));
                if *leaf {
                    hatcher.over(w, b.origin, &ring, i);
                }
                if !leaf || lk.lift > 0.0 {
                    let rim = if *leaf { ink } else { rim };
                    stroke(
                        w,
                        b.origin,
                        &ring,
                        1.0 + lk.lift,
                        scale_alpha(rim, lk.alpha),
                        true,
                    );
                }
                if gi < 1.0 {
                    continue;
                }
                if *leaf {
                    text.centred(w, cx, label, (b.origin, (cx_, cy)), 2.0 * rr - 6.0, ink);
                } else if *depth == 0 {
                    // A branch's name sits just inside its top edge.
                    let at = (cx_, cy - rr + text.size_line());
                    text.centred(w, cx, label, (b.origin, at), rr * 1.2, ink);
                }
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("pack").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, y, _, _| {
            let (ox, oy, side) = hit.get();
            let (u, v) = ((x - ox) / side.max(1.0), (y - oy) / side.max(1.0));
            // Children follow their parents: the last circle holding the pointer is the deepest.
            hit_geo
                .iter()
                .rposition(|(cx_, cy, r)| (cx_ - u).powi(2) + (cy - v).powi(2) <= r * r)
        });
        let shown = focus
            .shown(cx)
            .and_then(|i| disc_text(&discs, i, total, &self.fmt));
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, shown, "Hover a circle"));
        titled(&t, self.title, body)
    }
}

dodai_ui_gpui::entry_render!(CirclePack);
