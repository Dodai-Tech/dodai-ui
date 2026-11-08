//! The glass cockpit's extra layouts on a narrow display page: the tape, an airspeed-style tape
//! scrolling past a boxed readout, and the column, an engine-indication bar with a pointer. Both
//! take the page's colours: white scales, a cyan caption, the value in the zone's colour past a
//! limit and in inverse video past the alarm limit. Every colour comes from the theme.

use super::layouts::layer;
use super::rich::{zone_colours, FINE};
use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::dial::severity_color;
use crate::gpui::mfd::{glow, panel, readout_box, trail};
use crate::gpui::pod_math as pm;
use crate::gpui::shapes::Ink;
use crate::instrument::{readout, Severity};
use dodai_gpui::paint::{fill, fill_rect, scale_alpha, stroke, stroke_rect};
use dodai_gpui::theme_of;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// Height of the wide cockpit bar page, as a share of its width.
pub(crate) const BAR_H: f32 = 0.42;

/// Width of the narrow cockpit pages, as a share of their height.
pub(crate) const PAGE_W: f32 = 0.45;

/// The share of the scale the tape shows at once.
const TAPE_SPAN: f32 = 0.5;

/// Whether a mark at `y` shows on a tape running from `top` to `bottom`, kept `margin` clear of
/// both ends. Pure.
pub(crate) fn on_tape(y: f32, (top, bottom): (f32, f32), margin: f32) -> bool {
    y >= top + margin && y <= bottom - margin
}

impl GaugeFace {
    /// The value's colour on a cockpit page and whether it is past the alarm limit.
    fn page_colour(&self, t: &dodai_core::Theme, ink: &Ink) -> (Hsla, bool) {
        match pm::severity_at(&self.spans(), self.frac()) {
            Severity::Normal => (ink.text, false),
            sev => (severity_color(t, sev), sev == Severity::Alarm),
        }
    }

    /// The caption in cyan at the top of a narrow page and the unit at its foot.
    fn page_legends(&self, ink: &Ink, wd: f32, s: f32) -> [AnyElement; 2] {
        [
            text_box(wd / 2.0, s * 0.045, wd * 0.9)
                .text_size(px(s * 0.036))
                .text_color(ink.info)
                .child(self.caption().to_uppercase())
                .into_any_element(),
            text_box(wd / 2.0, s * 0.9, wd * 0.9)
                .text_size(px(s * 0.032))
                .text_color(ink.muted)
                .child(self.unit.to_uppercase())
                .into_any_element(),
        ]
    }

    /// An airspeed-style tape: the scale scrolls past a fixed readout box whose notch points at
    /// the ticks, the zones run as bands down the tape's edge.
    pub(super) fn cockpit_tape(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let wd = s * PAGE_W;
        let (col, alarm) = self.page_colour(&t, &ink);
        let (min, max) = (self.scale.min, self.scale.max);
        let value = self.scale.value_at(self.frac());
        let (top, bottom) = (s * 0.11, s * 0.87);
        let mid = (top + bottom) / 2.0;
        let per = (bottom - top) / ((max - min).abs().max(f32::EPSILON) * TAPE_SPAN);
        let y_of = move |v: f32| mid - (v - value) * per;
        let total = self.scale.majors.max(1) * FINE;
        let marks: Vec<(f32, bool, f32)> = (0..=total)
            .map(|i| {
                let v = min + (max - min) * i as f32 / total as f32;
                (y_of(v), i % FINE == 0, v)
            })
            .collect();
        let zones: Vec<(f32, f32, Hsla)> = zone_colours(&self, &t)
            .into_iter()
            .map(|(a, z, c)| {
                let (ya, yz) = (y_of(min + (max - min) * a), y_of(min + (max - min) * z));
                (yz.clamp(top, bottom), ya.clamp(top, bottom), c)
            })
            .collect();
        let (tx, tw) = (s * 0.04, s * 0.27);
        let x_tick = tx + s * 0.03;
        let (bx, bh) = (x_tick + s * 0.1, s * 0.1);
        let bw = wd - bx - s * 0.04;
        let ticks: Vec<(f32, bool)> = marks.iter().map(|m| (m.0, m.1)).collect();
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            panel(w, o, (wd, s), ink);
            fill_rect(
                w,
                o,
                (tx, top, tw, bottom - top),
                0.0,
                scale_alpha(ink.text, 0.05),
            );
            for (y0, y1, zc) in &zones {
                if y1 > y0 {
                    fill_rect(w, o, (tx, *y0, s * 0.018, y1 - y0), 0.0, *zc);
                }
            }
            let lw = (s * 0.005).max(1.0);
            for (y, major) in &ticks {
                if on_tape(*y, (top, bottom), 0.0) {
                    let len = if *major { s * 0.05 } else { s * 0.025 };
                    stroke(
                        w,
                        o,
                        &[(x_tick, *y), (x_tick + len, *y)],
                        lw,
                        ink.text,
                        false,
                    );
                }
            }
            for y in [top, bottom] {
                stroke(
                    w,
                    o,
                    &[(tx, y), (tx + tw, y)],
                    lw,
                    scale_alpha(ink.text, 0.5),
                    false,
                );
            }
            readout_box(w, o, (bx, mid - bh / 2.0, bw, bh), true, col, ink);
            if alarm {
                fill_rect(w, o, (bx, mid - bh / 2.0, bw, bh), 0.0, col);
            }
        };
        let numerals: Vec<AnyElement> = marks
            .into_iter()
            .filter(|(y, major, _)| {
                *major && on_tape(*y, (top, bottom), s * 0.025) && (y - mid).abs() > bh * 0.75
            })
            .map(|(y, _, v)| {
                div()
                    .absolute()
                    .left(px(x_tick + s * 0.065))
                    .top(px(y - s * 0.022))
                    .text_size(px(s * 0.036))
                    .text_color(ink.text)
                    .child(readout(v, 0, ""))
                    .into_any_element()
            })
            .collect();
        let fg = if alarm { ink.ground } else { col };
        div()
            .relative()
            .w(px(wd))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left(px(bx))
                    .top(px(mid - bh / 2.0))
                    .w(px(bw - s * 0.015))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(bh * 0.62))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(self.figure()),
            )
            .children(self.page_legends(&ink, wd, s))
            .into_any_element()
    }

    /// An engine-indication column: a bar filled to the value as a trail brightening upwards,
    /// a white scale with its majors, the zones on the scale with limit lines across the bar, a
    /// pointer at the value and the figure boxed at the top.
    pub(super) fn cockpit_column(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let wd = s * PAGE_W;
        let (col, alarm) = self.page_colour(&t, &ink);
        let frac = self.frac();
        let zones = zone_colours(&self, &t);
        let (top, bottom) = (s * 0.3, s * 0.86);
        let y_of = move |f: f32| bottom - (bottom - top) * f.clamp(0.0, 1.0);
        let (bar_x, bar_w) = (s * 0.15, s * 0.07);
        let x_s = bar_x + bar_w + s * 0.01;
        let majors = self.scale.majors.max(1);
        let (bx, by, bw, bh) = (s * 0.06, s * 0.1, wd - s * 0.12, s * 0.1);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            panel(w, o, (wd, s), ink);
            let lw = (s * 0.005).max(1.0);
            let tint = if col == ink.text { ink.info } else { col };
            for a in trail(frac, 20) {
                let y = y_of(frac);
                let y0 = y_of(a);
                fill_rect(
                    w,
                    o,
                    (bar_x, y, bar_w, y0 - y),
                    0.0,
                    scale_alpha(tint, 0.03),
                );
            }
            stroke_rect(
                w,
                o,
                (bar_x, top, bar_w, bottom - top),
                0.0,
                lw,
                scale_alpha(ink.text, 0.3),
            );
            stroke(w, o, &[(x_s, top), (x_s, bottom)], lw, ink.text, false);
            for (a, z, zc) in &zones {
                glow(
                    w,
                    o,
                    &[(x_s + lw * 1.6, y_of(*a)), (x_s + lw * 1.6, y_of(*z))],
                    lw * 2.2,
                    *zc,
                    false,
                );
                stroke(
                    w,
                    o,
                    &[(bar_x - lw, y_of(*a)), (x_s + lw * 3.0, y_of(*a))],
                    lw * 1.2,
                    *zc,
                    false,
                );
            }
            for i in 0..=majors {
                let y = y_of(i as f32 / majors as f32);
                stroke(w, o, &[(x_s, y), (x_s - s * 0.025, y)], lw, ink.text, false);
            }
            let y = y_of(frac);
            glow(w, o, &[(bar_x, y), (x_s, y)], lw * 1.4, col, false);
            let k = s * 0.022;
            let tip = x_s + lw * 3.5;
            let p = [(tip, y), (tip + k * 2.0, y - k), (tip + k * 2.0, y + k)];
            fill(w, o, &p, col);
            glow(w, o, &p, lw * 0.6, col, true);
            readout_box(w, o, (bx, by, bw, bh), false, col, ink);
            if alarm {
                fill_rect(w, o, (bx, by, bw, bh), 0.0, col);
            }
        };
        let numerals = super::rich::major_labels(&self)
            .into_iter()
            .map(move |(f, label)| {
                div()
                    .absolute()
                    .left(px(s * 0.02))
                    .top(px(y_of(f) - s * 0.02))
                    .w(px(bar_x - s * 0.035))
                    .flex()
                    .justify_end()
                    .text_size(px(s * 0.032))
                    .text_color(scale_alpha(ink.text, 0.8))
                    .child(label)
                    .into_any_element()
            });
        let fg = if alarm { ink.ground } else { col };
        div()
            .relative()
            .w(px(wd))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left(px(bx))
                    .top(px(by))
                    .w(px(bw - s * 0.02))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(bh * 0.62))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(self.figure()),
            )
            .children(self.page_legends(&ink, wd, s))
            .into_any_element()
    }

    /// An engine-indication bar on a wide page: the caption in cyan and the unit at the top
    /// left, the figure boxed at the top right, and a bar filled to the value as a trail
    /// brightening to the right over a white scale with its majors, the zones on the scale with
    /// limit lines across the bar and a pointer under the scale at the value.
    pub(super) fn cockpit_bar(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * BAR_H;
        let (col, alarm) = self.page_colour(&t, &ink);
        let frac = self.frac();
        let zones = zone_colours(&self, &t);
        let (x0, x1) = (s * 0.08, s * 0.92);
        let x_of = move |f: f32| x0 + (x1 - x0) * f.clamp(0.0, 1.0);
        let (bar_y, bar_h) = (h * 0.42, s * 0.055);
        let y_s = bar_y + bar_h + s * 0.008;
        let majors = self.scale.majors.max(1);
        let (bw, bh) = (s * 0.26, s * 0.09);
        let (bx, by) = (x1 - bw, s * 0.035);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            panel(w, o, (s, h), ink);
            let lw = (s * 0.004).max(1.0);
            let tint = if col == ink.text { ink.info } else { col };
            for a in trail(frac, 20) {
                let x = x_of(a);
                fill_rect(
                    w,
                    o,
                    (x, bar_y, x_of(frac) - x, bar_h),
                    0.0,
                    scale_alpha(tint, 0.03),
                );
            }
            stroke_rect(
                w,
                o,
                (x0, bar_y, x1 - x0, bar_h),
                0.0,
                lw,
                scale_alpha(ink.text, 0.3),
            );
            stroke(w, o, &[(x0, y_s), (x1, y_s)], lw, ink.text, false);
            for (a, z, zc) in &zones {
                glow(
                    w,
                    o,
                    &[(x_of(*a), y_s + lw * 1.6), (x_of(*z), y_s + lw * 1.6)],
                    lw * 2.2,
                    *zc,
                    false,
                );
                stroke(
                    w,
                    o,
                    &[(x_of(*a), bar_y - lw), (x_of(*a), y_s + lw * 3.0)],
                    lw * 1.2,
                    *zc,
                    false,
                );
            }
            for i in 0..=majors {
                let x = x_of(i as f32 / majors as f32);
                stroke(w, o, &[(x, y_s), (x, y_s + s * 0.02)], lw, ink.text, false);
            }
            let x = x_of(frac);
            glow(w, o, &[(x, bar_y), (x, y_s)], lw * 1.4, col, false);
            let k = s * 0.018;
            let tip = y_s + lw * 3.5;
            let p = [(x, tip), (x - k, tip + k * 2.0), (x + k, tip + k * 2.0)];
            fill(w, o, &p, col);
            glow(w, o, &p, lw * 0.6, col, true);
            readout_box(w, o, (bx, by, bw, bh), false, col, ink);
            if alarm {
                fill_rect(w, o, (bx, by, bw, bh), 0.0, col);
            }
        };
        let numerals = super::rich::major_labels(&self)
            .into_iter()
            .map(move |(f, label)| {
                text_box(x_of(f), y_s + s * 0.06, s * 0.12)
                    .text_size(px(s * 0.03))
                    .text_color(scale_alpha(ink.text, 0.8))
                    .child(label)
                    .into_any_element()
            });
        let fg = if alarm { ink.ground } else { col };
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(by))
                    .flex()
                    .flex_col()
                    .gap(px(s * 0.006))
                    .child(
                        div()
                            .text_size(px(s * 0.034))
                            .text_color(ink.info)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.028))
                            .text_color(ink.muted)
                            .child(self.unit.to_uppercase()),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .left(px(bx))
                    .top(px(by))
                    .w(px(bw - s * 0.02))
                    .h(px(bh))
                    .flex()
                    .items_center()
                    .justify_end()
                    .text_size(px(bh * 0.62))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(fg)
                    .child(self.figure()),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tape_mark_shows_only_between_its_ends() {
        assert!(on_tape(50.0, (10.0, 90.0), 5.0));
        assert!(!on_tape(12.0, (10.0, 90.0), 5.0), "too near the top");
        assert!(!on_tape(95.0, (10.0, 90.0), 0.0), "past the foot");
    }
}
