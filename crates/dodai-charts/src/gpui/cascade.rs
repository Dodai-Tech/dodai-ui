//! [`SplitFlow`]: a quantity flowing down through levels, splitting at each node.

use crate::cascade::{self, Block, Node};
use crate::gpui::common::{
    chart_cfg, fmt_with, group_slots, overlaps, plot_box, tracked_canvas, CanvasText, NumFmt,
};
use crate::gpui::focus::{readout, Focus};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// One node of a [`SplitFlow`]: a named amount, optionally split off another node.
#[derive(Clone, Debug)]
pub struct FlowStep {
    label: SharedString,
    value: f32,
    from: Option<SharedString>,
    group: Option<SharedString>,
}

impl FlowStep {
    /// The amount `value` named `label`; a root unless [`FlowStep::from`] names its source.
    pub fn new(label: impl Into<SharedString>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
            from: None,
            group: None,
        }
    }
    /// The node this one splits off (by label; it must come earlier in the list).
    pub fn from(mut self, parent: impl Into<SharedString>) -> Self {
        self.from = Some(parent.into());
        self
    }
    /// Its group: nodes of a group share a colour (all the losses, say).
    pub fn group(mut self, g: impl Into<SharedString>) -> Self {
        self.group = Some(g.into());
        self
    }
}

/// Split flow: a quantity pouring down through levels. Every block is as wide as its amount
/// (one scale throughout) and hangs under the slice of its parent it came from, joined by a
/// ribbon in its own colour, so the flow reads as one stream. Blocks show their name and share
/// of the total; every end branch runs on to a shared bottom row, captioned with its share and
/// amount.
/// Hover a block or ribbon for its numbers; click to keep it picked.
#[derive(IntoElement)]
pub struct SplitFlow {
    follow: dodai_gpui::follow::Follow,
    steps: Vec<FlowStep>,
    max_width: Option<f32>,
    height: f32,
    fmt: Option<NumFmt>,
    title: Option<SharedString>,
}
chart_cfg!(SplitFlow);

impl SplitFlow {
    dodai_gpui::follow_api!();
    /// Create a new [`SplitFlow`] of `steps`.
    #[track_caller]
    pub fn new(steps: impl IntoIterator<Item = FlowStep>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            steps: steps.into_iter().collect(),
            max_width: None,
            height: 460.0,
            fmt: None,
            title: None,
        }
    }
    /// Keep the flow at most `w` px wide, centred (default: the full width). With a tall
    /// [`height`](Self::height) this gives the long, narrow poster form, where the ribbons
    /// carry the eye down.
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = (w.is_finite() && w > 0.0).then_some(w.max(120.0));
        self
    }
}

/// Gap between blocks of a row, block height and smallest ribbon drop, in px.
const GAP: f32 = 6.0;
const BLOCK: f32 = 46.0;
const DROP: f32 = 28.0;
/// Opacity of the ribbons and of the pass-through strips that carry an end branch down: lighter
/// than the blocks, so blocks of one group stay apart instead of melting into one slab.
const STREAM: f32 = 0.5;

/// The `Node`s of `steps`: parents resolved by label among the earlier steps.
fn nodes(steps: &[FlowStep]) -> Vec<Node> {
    steps
        .iter()
        .enumerate()
        .map(|(i, s)| Node {
            parent: s
                .from
                .as_ref()
                .and_then(|p| steps[..i].iter().rposition(|q| &q.label == p)),
            value: s.value,
        })
        .collect()
}

/// Top of level `l` and the drop between levels, for `depth` levels in `h` px (the last row
/// keeps a line under it for amounts).
fn rows(h: f32, depth: usize, foot: f32) -> (f32, impl Fn(usize) -> f32) {
    let depth = depth.max(1);
    let drop = if depth > 1 {
        ((h - foot - BLOCK * depth as f32) / (depth - 1) as f32).max(DROP)
    } else {
        0.0
    };
    (drop, move |l: usize| l as f32 * (BLOCK + drop))
}

/// The node at `(x, y)`: a block, or the ribbon running into it.
fn hit(blocks: &[Block], top: &dyn Fn(usize) -> f32, (x, y): (f32, f32)) -> Option<usize> {
    blocks.iter().enumerate().find_map(|(i, b)| {
        let y0 = top(b.level);
        if (y0..=y0 + BLOCK).contains(&y) && (b.x0..=b.x1).contains(&x) {
            return Some(i);
        }
        let (s0, s1) = b.source?;
        let from = top(b.level - 1) + BLOCK;
        if !(from..y0).contains(&y) || y0 <= from {
            return None;
        }
        let t = (y - from) / (y0 - from);
        let e = t * t * (3.0 - 2.0 * t);
        let (l, r) = (s0 + (b.x0 - s0) * e, s1 + (b.x1 - s1) * e);
        (l..=r).contains(&x).then_some(i)
    })
}

/// `nodes` with every end node carried down to the last level by pass-through copies, so
/// all outcomes land on one bottom row. Also returns, per node, the step it shows and whether
/// it is such a copy.
fn reach_bottom(nodes: &[Node]) -> (Vec<Node>, Vec<usize>, Vec<bool>) {
    let lv = cascade::levels(nodes);
    let last = lv.iter().copied().max().unwrap_or(0);
    let mut out = nodes.to_vec();
    let mut origin: Vec<usize> = (0..nodes.len()).collect();
    let mut pass = vec![false; nodes.len()];
    for i in 0..nodes.len() {
        if nodes.iter().any(|m| m.parent == Some(i)) {
            continue;
        }
        let mut from = i;
        for _ in lv[i]..last {
            out.push(Node {
                parent: Some(from),
                value: nodes[i].value,
            });
            origin.push(i);
            pass.push(true);
            from = out.len() - 1;
        }
    }
    (out, origin, pass)
}

impl SplitFlow {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "SplitFlow");
        let steps = nodes(&self.steps);
        let n = steps.len();
        let (all, origin, pass) = reach_bottom(&steps);
        let depth = cascade::levels(&all).into_iter().max().map_or(0, |m| m + 1);
        let focus = Focus::new(window, cx, &self.follow, n);
        let looks: Vec<_> = (0..n).map(|k| focus.item(k)).collect();
        let (slots, _) = group_slots(self.steps.iter().map(|s| s.group.as_ref()));
        let cols: Vec<Hsla> = slots
            .iter()
            .map(|s| hsla(t.palette.series_at(*s)))
            .collect();
        let inks: Vec<Hsla> = slots
            .iter()
            .map(|s| hsla(t.on(t.palette.series_at(*s))))
            .collect();
        let total: f32 = steps
            .iter()
            .filter(|n| n.parent.is_none() && n.value.is_finite() && n.value > 0.0)
            .map(|n| n.value)
            .sum();
        let pct = |v: f32| {
            if total > 0.0 && v.is_finite() {
                format!("{:.0}%", v / total * 100.0)
            } else {
                String::new()
            }
        };
        let names: Vec<String> = self.steps.iter().map(|s| s.label.to_uppercase()).collect();
        let shares: Vec<String> = self.steps.iter().map(|s| pct(s.value)).collect();
        let amounts: Vec<String> = self
            .steps
            .iter()
            .map(|s| fmt_with(&self.fmt, s.value))
            .collect();
        let head = CanvasText::display(&t, t.typography.size_xs).bold();
        let big = CanvasText::display(&t, t.typography.size_xl);
        let small = CanvasText::display(&t, t.typography.size_xs);
        let (muted, seam) = (hsla(t.palette.text_muted), hsla(t.palette.background));
        let foot = head.size_line() + big.size_line() + small.size_line() + 6.0;
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let (pall, porigin, ppass) = (all.clone(), origin.clone(), pass.clone());
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let blocks = cascade::layout(&pall, bw, GAP);
            let (_, top) = rows(bh, depth, foot);
            // Rows pour in top down as the chart grows.
            let shown = |l: usize| (g * depth as f32 - l as f32).clamp(0.0, 1.0);
            for (i, bl) in blocks.iter().enumerate() {
                let (Some(src), a) = (bl.source, shown(bl.level)) else {
                    continue;
                };
                let o = porigin[i];
                if a <= 0.0 {
                    continue;
                }
                let y0 = top(bl.level - 1) + BLOCK;
                let rib = cascade::ribbon(src, (bl.x0, bl.x1), y0, top(bl.level), 20);
                fill(
                    w,
                    b.origin,
                    &rib,
                    scale_alpha(cols[o], STREAM * a * looks[o].alpha),
                );
            }
            let mut taken = Vec::new();
            let mut pills: Vec<[f32; 4]> = Vec::new();
            for (i, bl) in blocks.iter().enumerate() {
                let o = porigin[i];
                let a = shown(bl.level) * looks[o].alpha;
                if a <= 0.0 || bl.x1 <= bl.x0 {
                    continue;
                }
                let y = top(bl.level);
                let r = dodai_gpui::plot::rect(bl.x0, y, bl.x1 - bl.x0, BLOCK);
                let carried = ppass[i] && bl.level + 1 < depth;
                fill(
                    w,
                    b.origin,
                    &r,
                    scale_alpha(cols[o], if carried { STREAM * a } else { a }),
                );
                if !ppass[i] {
                    // A seam where the flow enters a new stage.
                    let seam_line = [(bl.x0, y + 1.0), (bl.x1, y + 1.0)];
                    stroke(
                        w,
                        b.origin,
                        &seam_line,
                        2.0,
                        scale_alpha(seam, 0.7 * a),
                        false,
                    );
                }
                let (mid, wid) = ((bl.x0 + bl.x1) / 2.0, bl.x1 - bl.x0 - 8.0);
                let ink = scale_alpha(inks[o], a);
                let bottom = bl.level + 1 == depth;
                if !ppass[i] && !bottom {
                    let fits = head.centred(
                        w,
                        cx,
                        &names[o],
                        (b.origin, (mid, y + BLOCK * 0.28)),
                        wid,
                        ink,
                    );
                    if fits {
                        let at = (b.origin, (mid, y + BLOCK * 0.66));
                        big.centred(w, cx, &shares[o], at, wid, ink);
                    } else {
                        // Too narrow for its name: a pill on its ribbon, which may stick
                        // out past it, as long as it misses the other pills.
                        let next = top(bl.level + 1);
                        let text = format!("{} {}", names[o], shares[o]);
                        let (lw, lh) = head.measure(w, &text);
                        // Try a few heights down the ribbon; keep the pill inside the chart.
                        let half = lw / 2.0 + 6.0;
                        let pill_x = mid.clamp(half, (bw - half).max(half));
                        let free = [0.45, 0.7, 0.2].into_iter().find_map(|f| {
                            let py = y + BLOCK + (next - y - BLOCK) * f;
                            let bx = [
                                pill_x - half,
                                py - lh / 2.0 - 3.0,
                                pill_x + half,
                                py + lh / 2.0 + 3.0,
                            ];
                            pills.iter().all(|p| !overlaps(&bx, p)).then_some((py, bx))
                        });
                        if let Some((py, bx)) = free {
                            let r =
                                dodai_gpui::plot::rect(bx[0], bx[1], bx[2] - bx[0], bx[3] - bx[1]);
                            fill(w, b.origin, &r, scale_alpha(cols[o], a));
                            stroke(w, b.origin, &r, 1.5, scale_alpha(seam, a), true);
                            head.centred(w, cx, &text, (b.origin, (pill_x, py)), lw + 2.0, ink);
                            pills.push(bx);
                        } else {
                            head.centred(
                                w,
                                cx,
                                &shares[o],
                                (b.origin, (mid, y + BLOCK / 2.0)),
                                wid,
                                ink,
                            );
                        }
                    }
                }
                if bottom {
                    // Outcomes: name, share and amount under the bottom row.
                    let c = scale_alpha(cols[o], a);
                    let y1 = y + BLOCK + 3.0;
                    // The name when it fits, else just the share and amount.
                    head.place_free(
                        w,
                        cx,
                        (&names[o], c),
                        b.origin,
                        &[((mid, y1), (0.0, 1.0))],
                        &mut taken,
                    );
                    // Shares and amounts keep their own rows, so a name that did not fit
                    // does not push them into its neighbours' names.
                    let y2 = y1 + head.size_line();
                    let r = big.place_free(
                        w,
                        cx,
                        (&shares[o], c),
                        b.origin,
                        &[((mid, y2), (0.0, 1.0))],
                        &mut taken,
                    );
                    if r.is_some() {
                        small.place_free(
                            w,
                            cx,
                            (&amounts[o], scale_alpha(muted, a)),
                            b.origin,
                            &[((mid, y2 + big.size_line()), (0.0, 1.0))],
                            &mut taken,
                        );
                    }
                }
            }
        })
        .absolute()
        .size_full();
        let plot = plot_box(
            self.follow.sub("flow").element_id(window, cx),
            self.height,
            cv,
        );
        let plot = match self.max_width {
            Some(mw) => plot.max_w(px(mw)).mx_auto(),
            None => plot,
        };
        let plot = focus.pointer(plot, &bounds, move |x, y, w, h| {
            let blocks = cascade::layout(&all, w, GAP);
            let (_, top) = rows(h, depth, foot);
            hit(&blocks, &top, (x, y)).map(|k| origin[k])
        });
        let text = focus
            .shown(cx)
            .and_then(|k| self.steps.get(k).map(|s| (k, s)))
            .map(|(k, s)| {
                let mut line = format!("{} · {}", s.label, fmt_with(&self.fmt, s.value));
                if let Some(p) = steps[k].parent {
                    let pv = self.steps[p].value;
                    if pv > 0.0 {
                        line +=
                            &format!(" · {:.0}% of {}", s.value / pv * 100.0, self.steps[p].label);
                    }
                }
                line + " · " + &pct(s.value) + " of the total"
            });
        let body = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(plot)
            .child(readout(&t, text, "Hover a block"));
        crate::gpui::common::titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(SplitFlow);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_nodes_are_carried_to_the_bottom_row() {
        let steps = [
            Node {
                parent: None,
                value: 10.0,
            },
            Node {
                parent: Some(0),
                value: 6.0,
            },
            Node {
                parent: Some(0),
                value: 4.0,
            },
            Node {
                parent: Some(1),
                value: 6.0,
            },
        ];
        let (all, origin, pass) = reach_bottom(&steps);
        assert_eq!(all.len(), 5);
        assert_eq!((all[4].parent, origin[4], pass[4]), (Some(2), 2, true));
        assert_eq!(cascade::levels(&all)[4], 2);
    }

    #[test]
    fn parents_resolve_by_earlier_label_only() {
        let steps = [
            FlowStep::new("b", 1.0).from("a"),
            FlowStep::new("a", 2.0),
            FlowStep::new("c", 1.0).from("a"),
        ];
        let n = nodes(&steps);
        assert_eq!((n[0].parent, n[2].parent), (None, Some(1)));
    }

    #[test]
    fn rows_fill_the_height_and_hits_find_blocks_and_ribbons() {
        let (drop, top) = rows(300.0, 3, 20.0);
        assert_eq!(drop, (300.0 - 20.0 - 3.0 * BLOCK) / 2.0);
        assert_eq!(top(2), 2.0 * (BLOCK + drop));
        let nodes = [
            Node {
                parent: None,
                value: 10.0,
            },
            Node {
                parent: Some(0),
                value: 4.0,
            },
        ];
        let blocks = cascade::layout(&nodes, 200.0, GAP);
        let mid_x = (blocks[1].x0 + blocks[1].x1) / 2.0;
        assert_eq!(hit(&blocks, &top, (mid_x, top(1) + 5.0)), Some(1));
        assert_eq!(hit(&blocks, &top, (100.0, 5.0)), Some(0));
        let src = blocks[1].source.unwrap();
        assert_eq!(
            hit(&blocks, &top, ((src.0 + src.1) / 2.0, BLOCK + 1.0)),
            Some(1)
        );
        assert_eq!(hit(&blocks, &top, (199.0, BLOCK + drop - 1.0)), None);
    }
}
