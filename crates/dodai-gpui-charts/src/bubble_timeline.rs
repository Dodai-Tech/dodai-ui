//! [`BubbleTimeline`]: events on a time axis as bubbles sized by how big each was, piled up
//! from the axis so none overlap, coloured by group; the legend filters the groups.

use crate::common::{
    chart_cfg, fill, fmt_with, marker_points, norm, plot_box, stroke, tick_row, titled,
    tracked_canvas, CanvasText, Groups, Marker, NumFmt,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::pack::{pile_up, swarm_sized};
use dodai_gpui::paint::scale_alpha;
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// One event of a [`BubbleTimeline`]: a name, when it happened (any number: a year, a day
/// count) and how big it was (drawn as the bubble's area).
#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    label: SharedString,
    at: f32,
    size: f32,
    group: Option<SharedString>,
    note: Option<SharedString>,
}

impl Bubble {
    /// An event `label` at time `at`, of magnitude `size`.
    pub fn new(label: impl Into<SharedString>, at: f32, size: f32) -> Self {
        Self {
            label: label.into(),
            at,
            size,
            group: None,
            note: None,
        }
    }
    /// The group it belongs to: one colour and one legend entry per group.
    pub fn group(mut self, g: impl Into<SharedString>) -> Self {
        self.group = Some(g.into());
        self
    }
    /// A line of detail shown when the bubble is hovered or picked.
    pub fn note(mut self, n: impl Into<SharedString>) -> Self {
        self.note = Some(n.into());
        self
    }
}

/// Bubble timeline: every event a bubble at its time, its area the event's size, piled up
/// from the time axis (the biggest rest on it, smaller ones settle above) so nothing overlaps.
/// Bubbles big enough carry their name and size inside; the others get their name beside
/// them where it fits. Click a legend entry to hide or show its group; hover or click a bubble
/// for its details.
#[derive(IntoElement)]
pub struct BubbleTimeline {
    follow: dodai_gpui::follow::Follow,
    items: Vec<Bubble>,
    max_radius: f32,
    size_label: Option<SharedString>,
    x_fmt: Option<NumFmt>,
    filter: bool,
    centred: bool,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(BubbleTimeline);

impl BubbleTimeline {
    dodai_gpui::follow_api!();
    /// Create a new [`BubbleTimeline`] from its events, in any order.
    #[track_caller]
    pub fn new(items: impl IntoIterator<Item = Bubble>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            items: items.into_iter().collect(),
            max_radius: 48.0,
            size_label: None,
            x_fmt: None,
            filter: true,
            centred: false,
            height: 420.0,
            fmt: None,
            title: None,
        }
    }
    /// Radius in px of the biggest bubble (default 48, `6..=120`). Shrunk to fit if the pile
    /// would outgrow the chart.
    pub fn max_radius(mut self, r: f32) -> Self {
        self.max_radius = if r.is_finite() {
            r.clamp(6.0, 120.0)
        } else {
            48.0
        };
        self
    }
    /// What the size measures, for the details line ("records lost").
    pub fn size_label(mut self, l: impl Into<SharedString>) -> Self {
        self.size_label = Some(l.into());
        self
    }
    /// How times are written on the axis and in the details (default: plain numbers).
    pub fn x_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.x_fmt = Some(Rc::new(f));
        self
    }
    /// Spread the bubbles either side of the middle (a sized beeswarm) instead of piling them
    /// up from the axis (default off).
    pub fn centred(mut self, on: bool) -> Self {
        self.centred = on;
        self
    }
    /// Whether clicking a legend entry hides its group (default on).
    pub fn filter(mut self, on: bool) -> Self {
        self.filter = on;
        self
    }
}

/// Bubble centres and radii `(x, y, r)` for `(time, size)` items in a `bw` x `bh` canvas whose
/// times run over `lo..hi`: piled up from the bottom, the biggest `rmax` across (less if the
/// pile would not fit the height).
fn lay_out(
    items: &[(f32, f32)],
    (lo, hi): (f32, f32),
    (bw, bh): (f32, f32),
    (rmax, centred): (f32, bool),
) -> Vec<(f32, f32, f32)> {
    let smax = dodai_charts_core::peak_from(items.iter().map(|i| i.1), f32::MIN_POSITIVE);
    let mut rmax = rmax.min(bw / 4.0).max(1.0);
    let mut out = Vec::new();
    for _ in 0..4 {
        let span = (bw - 2.0 * rmax).max(1.0);
        let xs: Vec<f32> = items
            .iter()
            .map(|i| rmax + norm(i.0, lo, hi) * span)
            .collect();
        // Tiny events still get a dot you can find and hover.
        let rs: Vec<f32> = items
            .iter()
            .map(|i| dodai_charts_core::area_radius(i.1, smax, 0.0, rmax).max(2.5))
            .collect();
        let gap: Vec<f32> = rs.iter().map(|r| r + 1.0).collect();
        let (top, room) = if centred {
            let ys = swarm_sized(&xs, &gap);
            let top = ys
                .iter()
                .zip(&rs)
                .map(|(y, r)| y.abs() + r)
                .fold(0.0, f32::max);
            let mid = bh / 2.0;
            out = (0..items.len())
                .map(|i| (xs[i], mid + ys[i], rs[i]))
                .collect();
            (top, mid)
        } else {
            let ys = pile_up(&xs, &gap);
            let top = ys.iter().zip(&rs).map(|(y, r)| y + r).fold(0.0, f32::max);
            out = (0..items.len())
                .map(|i| (xs[i], bh - ys[i], rs[i]))
                .collect();
            (top, bh)
        };
        if top <= room || bh <= 0.0 {
            break;
        }
        // Areas scale with the square, so shrink the radii by the root of the overflow.
        rmax *= (room / top).sqrt().clamp(0.5, 0.97);
    }
    out
}

impl BubbleTimeline {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "BubbleTimeline");
        let groups = Groups::new(
            window,
            cx,
            &self.follow,
            self.items.iter().map(|b| b.group.as_ref()),
            self.filter,
        );
        let slots = &groups.slots;
        // Indices of the shown, drawable events.
        let shown: Vec<usize> = (0..self.items.len())
            .filter(|&i| {
                let b = &self.items[i];
                b.at.is_finite() && b.size.is_finite() && b.size > 0.0 && groups.shown(i)
            })
            .collect();
        let (tlo, thi, ticks) = dodai_charts_core::tick_span(self.items.iter().map(|b| b.at), 8);
        let n = shown.len();
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        let fills: Vec<Hsla> = shown
            .iter()
            .map(|&i| hsla(t.palette.series_at(slots[i])))
            .collect();
        let inks: Vec<Hsla> = shown
            .iter()
            .map(|&i| hsla(t.on(t.palette.series_at(slots[i]))))
            .collect();
        let data: Vec<(f32, f32)> = shown
            .iter()
            .map(|&i| (self.items[i].at, self.items[i].size))
            .collect();
        let names: Vec<SharedString> = shown.iter().map(|&i| self.items[i].label.clone()).collect();
        let sizes: Vec<String> = shown
            .iter()
            .map(|&i| fmt_with(&self.fmt, self.items[i].size))
            .collect();
        let (ink, axis, surface) = (
            hsla(t.palette.text),
            hsla(t.palette.border),
            hsla(t.palette.surface),
        );
        let inside = CanvasText::display(&t, t.typography.size_xs).bold();
        let value = CanvasText::display(&t, t.typography.size_xs);
        let beside = CanvasText::display(&t, t.typography.size_xs);
        let rmax = self.max_radius;
        let centred = self.centred;
        let placed: Rc<RefCell<Vec<(f32, f32, f32)>>> = Default::default();
        let painted = placed.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let at = lay_out(&data, (tlo, thi), (bw, bh), (rmax, centred));
            stroke(
                w,
                b.origin,
                &[(0.0, bh - 0.5), (bw, bh - 0.5)],
                1.0,
                axis,
                false,
            );
            // Paint big first so small bubbles stay visible on top.
            let mut order: Vec<usize> = (0..at.len()).collect();
            order.sort_by(|a, c| at[*c].2.total_cmp(&at[*a].2));
            for &i in &order {
                let (x, y, r) = at[i];
                let gi = dodai_core::motion::reveal(g, i, at.len());
                if gi <= 0.0 {
                    continue;
                }
                let lk = looks[i];
                let rr = (r + lk.lift * 2.0) * gi;
                let dot = marker_points(Marker::Dot, x, y, rr);
                fill(w, b.origin, &dot, scale_alpha(fills[i], 0.9 * lk.alpha));
                stroke(
                    w,
                    b.origin,
                    &dot,
                    1.0,
                    scale_alpha(surface, 0.7 * lk.alpha),
                    true,
                );
            }
            *painted.borrow_mut() = at.clone();
            if g < 1.0 {
                return;
            }
            // Names: inside bubbles that hold them, else beside, the focused first then
            // the biggest; every bubble keeps its own room. The canvas edges count as taken.
            let mut taken: Vec<[f32; 4]> = vec![
                [-1e5, -1e5, 0.0, 1e5],
                [bw, -1e5, 1e5, 1e5],
                [-1e5, -1e5, 1e5, 0.0],
            ];
            let mut outside = Vec::new();
            for &i in &order {
                let (x, y, r) = at[i];
                let c = scale_alpha(inks[i], looks[i].alpha);
                let lh = inside.size_line();
                if r * 2.0 > lh * 2.4
                    && inside.centred(w, cx, &names[i], (b.origin, (x, y - lh * 0.45)), r * 1.8, c)
                {
                    value.centred(w, cx, &sizes[i], (b.origin, (x, y + lh * 0.55)), r * 1.8, c);
                } else if r * 2.0 > lh * 1.2
                    && inside.centred(w, cx, &names[i], (b.origin, (x, y)), r * 1.8, c)
                {
                } else {
                    outside.push(i);
                }
                taken.push([x - r, y - r, x + r, y + r]);
            }
            outside.sort_by(|a, c| looks[*c].lift.total_cmp(&looks[*a].lift));
            for i in outside {
                let (x, y, r) = at[i];
                let c = scale_alpha(ink, looks[i].alpha.max(0.4));
                let r = r + 2.0;
                let spots = [
                    ((x, y - r), (0.0, -1.0)),
                    ((x + r, y), (1.0, 0.0)),
                    ((x - r, y), (-1.0, 0.0)),
                ];
                beside.place_free(w, cx, (&names[i], c), b.origin, &spots, &mut taken);
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("bubbles").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = focus.pointer(plot, &bounds, move |x, y, _, _| {
            // The smallest bubble under the pointer: small ones sit on top.
            let at = placed.borrow();
            (0..at.len())
                .filter(|&i| (at[i].0 - x).powi(2) + (at[i].1 - y).powi(2) <= at[i].2 * at[i].2)
                .min_by(|a, c| at[*a].2.total_cmp(&at[*c].2))
        });
        let text = focus.shown(cx).and_then(|k| {
            let b = &self.items[*shown.get(k)?];
            let mut line = format!("{} · {}", b.label, fmt_with(&self.x_fmt, b.at));
            if let Some(gr) = &b.group {
                line += &format!(" · {gr}");
            }
            line += &format!(" · {}", fmt_with(&self.fmt, b.size));
            if let Some(l) = &self.size_label {
                line += &format!(" {l}");
            }
            if let Some(n) = &b.note {
                line += &format!(" · {n}");
            }
            Some(line)
        });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w_full()
            .child(plot)
            .child(
                div()
                    .px(px(rmax.min(48.0)))
                    .child(tick_row(&t, &ticks, &self.x_fmt)),
            )
            .children(groups.legend(&t))
            .child(readout(&t, text, &groups.hint("Hover a bubble")));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(BubbleTimeline);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bubbles_fit_the_canvas_and_keep_their_order_in_time() {
        let items: Vec<(f32, f32)> = (0..40)
            .map(|i| (2000.0 + (i % 10) as f32, 1.0 + i as f32))
            .collect();
        let at = lay_out(&items, (2000.0, 2010.0), (600.0, 200.0), (60.0, false));
        for (i, (x, y, r)) in at.iter().enumerate() {
            assert!(*x >= 0.0 && *x <= 600.0, "bubble {i} off the side");
            assert!(*y + r <= 200.0 + 1e-2, "bubble {i} below the axis");
        }
        assert!(at[0].0 < at[9].0, "later events sit further right");
        assert!(lay_out(&[], (0.0, 1.0), (10.0, 10.0), (5.0, false)).is_empty());
    }

    #[test]
    fn the_builder_keeps_its_options_sane() {
        let c = BubbleTimeline::new([Bubble::new("a", 1.0, 2.0).group("g").note("n")])
            .max_radius(f32::NAN)
            .filter(false);
        assert_eq!((c.max_radius, c.filter), (48.0, false));
        assert_eq!(c.items[0].note.as_ref().map(|n| n.as_ref()), Some("n"));
    }
}
