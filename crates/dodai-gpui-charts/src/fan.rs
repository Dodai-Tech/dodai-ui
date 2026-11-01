//! [`SpanFan`]: lifespans as spokes fanned round an arc.

use crate::common::{
    chart_cfg, fmt_with, nice_ticks, plot_box, titled, tracked_canvas, CanvasText, Groups, NumFmt,
};
use crate::focus::{readout, Focus};
use dodai_charts_core::fan::{self, FanOrder, Spoke};
use dodai_charts_core::math::{polar, turn_arc};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::{hsla, theme_of};
use gpui::{
    div, px, App, Bounds, Hsla, IntoElement, ParentElement, Pixels, SharedString, Styled, Window,
};
use std::cell::Cell;
use std::rc::Rc;

/// One item of a [`SpanFan`]: a named lifespan, with an optional peak stretch, a magnitude
/// and a group.
#[derive(Clone, Debug)]
pub struct FanSpan {
    label: SharedString,
    span: fan::Span,
    group: Option<SharedString>,
}

impl FanSpan {
    /// The lifespan `start..end` of `label`.
    pub fn new(label: impl Into<SharedString>, start: f32, end: f32) -> Self {
        Self {
            label: label.into(),
            span: fan::Span {
                start,
                end,
                peak: None,
                magnitude: 1.0,
            },
            group: None,
        }
    }
    /// Its peak, `from..to` (clipped to the lifespan): drawn as a heavier stretch.
    pub fn peak(mut self, from: f32, to: f32) -> Self {
        self.span.peak = Some((from, to));
        self
    }
    /// Its size (area, count, ...): sets the end marker's *area* and the label's size.
    pub fn magnitude(mut self, m: f32) -> Self {
        self.span.magnitude = m;
        self
    }
    /// Its group: spans of one group share a colour and a legend entry.
    pub fn group(mut self, g: impl Into<SharedString>) -> Self {
        self.group = Some(g.into());
        self
    }
}

/// Span fan: one spoke per item, fanned round an arc from a hub. A spoke's length is the
/// item's lifespan on one shared scale (guide rings mark it), a heavier stretch its peak, the
/// dot at its tip its magnitude (by area) and the label grows with it. Hover a spoke for its
/// numbers; click to keep it picked.
#[derive(IntoElement)]
pub struct SpanFan {
    follow: dodai_gpui::follow::Follow,
    items: Vec<FanSpan>,
    sweep: f32,
    order: FanOrder,
    hub: f32,
    height: f32,
    fmt: Option<NumFmt>,
    magnitude_fmt: Option<NumFmt>,
    filter: bool,
    title: Option<SharedString>,
}
chart_cfg!(SpanFan);

impl SpanFan {
    dodai_gpui::follow_api!();
    /// Create a new [`SpanFan`] of `items`.
    #[track_caller]
    pub fn new(items: impl IntoIterator<Item = FanSpan>) -> Self {
        Self {
            follow: dodai_gpui::follow::Follow::here(),
            items: items.into_iter().collect(),
            sweep: 0.75,
            order: FanOrder::Given,
            hub: 0.16,
            height: 520.0,
            fmt: None,
            magnitude_fmt: None,
            filter: true,
            title: None,
        }
    }
    /// The arc the spokes fill, in turns centred on the top (default 0.75, `0.05..=1`).
    pub fn sweep(mut self, turns: f32) -> Self {
        self.sweep = if turns.is_finite() {
            turns.clamp(0.05, 1.0)
        } else {
            0.75
        };
        self
    }
    /// The order the spokes go round in (default: as given).
    pub fn order(mut self, o: FanOrder) -> Self {
        self.order = o;
        self
    }
    /// The hub's radius, as a share of the fan's (default 0.16, `0..=0.6`).
    pub fn hub(mut self, h: f32) -> Self {
        self.hub = if h.is_finite() {
            h.clamp(0.0, 0.6)
        } else {
            0.16
        };
        self
    }
    /// Let the legend hide and show groups (default on; needs two groups or more).
    pub fn filter(mut self, on: bool) -> Self {
        self.filter = on;
        self
    }
    /// Formatter for magnitudes in the readout.
    pub fn magnitude_format(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.magnitude_fmt = Some(Rc::new(f));
        self
    }
}

/// Room kept round the fan for the labels, in px.
const LABEL_ROOM: f32 = 60.0;

/// How far the fan reaches below its centre, as a share of its radius.
fn below(sweep: f32, hub: f32) -> f32 {
    let arc = if sweep >= 0.5 {
        (std::f32::consts::PI * (1.0 - sweep)).cos().max(0.0)
    } else {
        0.0
    };
    arc.max(hub)
}

/// Fan geometry in a canvas `w`×`h`: centre, hub radius and rim radius, with [`LABEL_ROOM`]
/// all round. The centre drops when the arc leaves the bottom open, so the fan fills the box.
fn frame(w: f32, h: f32, sweep: f32, hub: f32) -> ((f32, f32), f32, f32) {
    let r = (w / 2.0 - LABEL_ROOM)
        .min((h - 2.0 * LABEL_ROOM) / (1.0 + below(sweep, hub)))
        .max(8.0);
    ((w / 2.0, LABEL_ROOM + r), r * hub, r)
}

impl SpanFan {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let g = dodai_gpui::motion::grow_for(window, cx, &self.follow, "SpanFan");
        let groups = Groups::new(
            window,
            cx,
            &self.follow,
            self.items.iter().map(|i| i.group.as_ref()),
            self.filter,
        );
        let (items, slots) = groups.keep(std::mem::take(&mut self.items));
        self.items = items;
        let spans: Vec<fan::Span> = self.items.iter().map(|i| i.span).collect();
        let spokes: Vec<Spoke> = fan::layout(&spans, self.sweep, self.order);
        let focus = Focus::new(window, cx, &self.follow, spokes.len());
        let looks: Vec<_> = (0..spokes.len()).map(|k| focus.item(k)).collect();
        let cols: Vec<Hsla> = slots
            .iter()
            .map(|s| hsla(t.palette.series_at(*s)))
            .collect();
        let labels: Vec<SharedString> = self.items.iter().map(|i| i.label.clone()).collect();
        let longest = spans
            .iter()
            .map(|s| (s.end - s.start).abs())
            .filter(|v| v.is_finite())
            .fold(0.0f32, f32::max);
        let ticks: Vec<f32> = if longest > 0.0 {
            nice_ticks(0.0, longest, 4)
                .into_iter()
                .filter(|v| *v > 0.0 && *v <= longest)
                .collect()
        } else {
            Vec::new()
        };
        let tick_text: Vec<String> = ticks.iter().map(|v| fmt_with(&self.fmt, *v)).collect();
        let (sweep, hub) = (self.sweep, self.hub);
        let (guide, muted) = (hsla(t.palette.border), hsla(t.palette.text_muted));
        let small = CanvasText::of(&t);
        let base = t.typography.size_xs;
        let display = t.clone();
        let bounds: Rc<Cell<Bounds<Pixels>>> = Default::default();
        let paint_spokes = spokes.clone();
        let cv = tracked_canvas(&bounds, move |b: Bounds<Pixels>, w, cx| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let ((ox, oy), r0, r1) = frame(bw, bh, sweep, hub);
            let reach = r1 - r0;
            let (a, e) = if sweep >= 1.0 {
                (0.0, 1.0)
            } else {
                (-sweep / 2.0, sweep / 2.0)
            };
            // Guide rings on the length scale, labelled in the open part of the arc.
            stroke(w, b.origin, &turn_arc(ox, oy, r0, a, e), 1.0, guide, false);
            for (v, s) in ticks.iter().zip(&tick_text) {
                let r = r0 + reach * v / longest;
                stroke(
                    w,
                    b.origin,
                    &turn_arc(ox, oy, r, a, e),
                    1.0,
                    scale_alpha(guide, 0.6),
                    false,
                );
                if sweep < 1.0 {
                    small.beside(w, cx, s, (b.origin, polar(ox, oy, r, e)), (0.0, 1.0), muted);
                }
            }
            // Spokes first, then labels by priority: the focused one, then by size.
            let mut tips = Vec::with_capacity(paint_spokes.len());
            for (k, s) in paint_spokes.iter().enumerate() {
                let lk = looks[k];
                let col = scale_alpha(cols[s.item], lk.alpha);
                let len = s.len * g;
                let at = |f: f32| polar(ox, oy, r0 + reach * f, s.turn);
                let tip = at(len);
                // The ruler: hub to rim, faint.
                stroke(
                    w,
                    b.origin,
                    &[at(0.0), at(1.0)],
                    1.0,
                    scale_alpha(guide, 0.35 * lk.alpha),
                    false,
                );
                stroke(
                    w,
                    b.origin,
                    &[at(0.0), tip],
                    1.5 + lk.lift * 1.5,
                    col,
                    false,
                );
                if let Some((p, q)) = s.peak {
                    let (p, q) = (p.min(len), q.min(len));
                    if q > p {
                        stroke(
                            w,
                            b.origin,
                            &[at(p), at(q)],
                            5.0 + lk.lift * 2.0,
                            col,
                            false,
                        );
                    }
                }
                // The magnitude dot (area scale).
                let dot = (2.5 + s.mark * 9.0 + lk.lift * 2.0) * g.min(1.0);
                fill(w, b.origin, &turn_arc(tip.0, tip.1, dot, 0.0, 1.0), col);
                tips.push((tip, dot, col));
            }
            let mut rank: Vec<usize> = (0..paint_spokes.len()).collect();
            rank.sort_by(|a, b| {
                let key = |k: usize| looks[k].lift * 10.0 + paint_spokes[k].mark;
                key(*b).total_cmp(&key(*a))
            });
            let mut taken = Vec::with_capacity(rank.len());
            for k in rank {
                let (s, (tip, dot, col)) = (&paint_spokes[k], tips[k]);
                let u = polar(0.0, 0.0, 1.0, s.turn);
                let at_label = (tip.0 + u.0 * (dot + 4.0), tip.1 + u.1 * (dot + 4.0));
                let size = base * (0.85 + 0.75 * s.mark) + looks[k].lift * 2.0;
                CanvasText::display(&display, size).beside_free(
                    w,
                    cx,
                    (&labels[s.item], col),
                    (b.origin, at_label),
                    u,
                    &mut taken,
                );
            }
        })
        .absolute()
        .size_full();
        let hit_spokes = spokes.clone();
        let fan_box = plot_box(
            self.follow.sub("fan").element_id(window, cx),
            self.height,
            cv,
        );
        let fan_box = focus.pointer(fan_box, &bounds, move |x, y, w, h| {
            let ((ox, oy), r0, r1) = frame(w, h, sweep, hub);
            let (dx, dy) = (x - ox, y - oy);
            let d = (dx * dx + dy * dy).sqrt();
            if d < r0 * 0.5 || d > r1 + LABEL_ROOM {
                return None;
            }
            let turn = (dy.atan2(dx) / std::f32::consts::TAU + 0.25).rem_euclid(1.0);
            fan::hit(&hit_spokes, sweep, turn)
        });
        let text = focus.shown(cx).and_then(|k| spokes.get(k)).map(|s| {
            let it = &self.items[s.item];
            let sp = it.span;
            let (a, b) = (sp.start.min(sp.end), sp.start.max(sp.end));
            let mut line = format!(
                "{} · {} – {} ({})",
                it.label,
                fmt_with(&self.fmt, a),
                fmt_with(&self.fmt, b),
                fmt_with(&self.fmt, b - a)
            );
            if let Some((p, q)) = sp.peak {
                line += &format!(
                    " · peak {} – {}",
                    fmt_with(&self.fmt, p),
                    fmt_with(&self.fmt, q)
                );
            }
            line + " · " + &fmt_with(&self.magnitude_fmt, sp.magnitude)
        });
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .w_full()
            .child(fan_box)
            .child(readout(&t, text, &groups.hint("Hover a spoke")))
            .children(groups.legend(&t));
        titled(&t, self.title, body)
    }
}

dodai_gpui::entry_render!(SpanFan);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fan_fits_its_box_and_the_hub_sits_inside() {
        for (w, h) in [
            (600.0, 520.0),
            (300.0, 300.0),
            (1200.0, 400.0),
            (10.0, 10.0),
        ] {
            for sweep in [0.3, 0.5, 0.75, 1.0] {
                let ((ox, oy), r0, r1) = frame(w, h, sweep, 0.16);
                assert!(r0 < r1 && r1 > 0.0 && ox.is_finite() && oy.is_finite());
                if w >= 300.0 {
                    assert!(ox - r1 >= 0.0 && ox + r1 <= w, "{w}x{h} {sweep}");
                    assert!(oy - r1 >= 0.0, "{w}x{h} {sweep}");
                    assert!(oy + r1 * below(sweep, 0.16) <= h, "{w}x{h} {sweep}");
                }
            }
        }
    }

    #[test]
    fn groups_share_a_colour_slot_in_order_of_appearance() {
        let items = [
            FanSpan::new("a", 0.0, 1.0).group("x"),
            FanSpan::new("b", 0.0, 1.0).group("y"),
            FanSpan::new("c", 0.0, 1.0).group("x"),
        ];
        let (slots, names) = crate::common::group_slots(items.iter().map(|i| i.group.as_ref()));
        assert_eq!(slots, [0, 1, 0]);
        assert_eq!(names, ["x", "y"]);
    }
}
