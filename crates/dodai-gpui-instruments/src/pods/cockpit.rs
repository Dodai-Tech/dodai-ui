//! The glass-cockpit look of the pods: display pages with a title band, four pressure boxes split
//! by a cross,
//! dotted leaders to green values, an alert page with clear and recall legends led to the bezel keys, a cross of rings
//! with a bug, a radio panel with active and standby lines, and a heading arc under a boxed
//! readout. Every colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::mfd::{bug, leader, page, pointer_scale, readout_box, rect, BEZEL, KEYS};
use crate::shapes::{circle, Ink};

/// The page title at the top left with its rule; the pod's text sits under it.
fn title(s: f32, ink: Ink, text: &str, right: &str) -> gpui::Div {
    div()
        .absolute()
        .left(px(s * 0.09))
        .top(px(s * 0.075))
        .w(px(s * 0.82))
        .flex()
        .justify_between()
        .text_size(px(s * 0.04))
        .child(div().text_color(ink.text).child(text.to_uppercase()))
        .child(div().text_color(ink.info).child(right.to_uppercase()))
}

/// Paint the page's title band across the top of the glass, ruled off from the page under it.
fn rule(w: &mut Window, o: gpui::Point<Pixels>, s: f32, ink: Ink) {
    let (k, y) = (s * BEZEL, s * 0.14);
    fill(
        w,
        o,
        &rect(k, k, s - 2.0 * k, y - k),
        scale_alpha(ink.text, 0.07),
    );
    stroke(
        w,
        o,
        &[(k, y), (s - k, y)],
        s * 0.003,
        scale_alpha(ink.text, 0.4),
        false,
    );
}

/// Paint a soft-key legend box `(x, y, wd, h)` on the glass with a lead to the bezel key level
/// with it, on the left edge when `left`.
fn legend_box(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    s: f32,
    (x, y, wd, h): (f32, f32, f32, f32),
    left: bool,
    ink: Ink,
) {
    let col = scale_alpha(ink.text, 0.6);
    let lw = s * 0.004;
    stroke(w, o, &rect(x, y, wd, h), lw, col, true);
    let (from, to) = if left {
        (s * BEZEL, x)
    } else {
        (x + wd, s * (1.0 - BEZEL))
    };
    stroke(
        w,
        o,
        &[(from, y + h / 2.0), (to, y + h / 2.0)],
        lw,
        col,
        false,
    );
}

/// Text in a box `(x, y, wd, h)`, centred.
fn boxed(
    x: f32,
    y: f32,
    wd: f32,
    h: f32,
    size: f32,
    col: Hsla,
    text: impl Into<SharedString>,
) -> gpui::Div {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(wd))
        .h(px(h))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size))
        .text_color(col)
        .child(text.into())
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four pressure boxes in quadrants split by a faint cross, front on top: cyan when good,
    /// the frame and figure amber when low.
    pub(super) fn cockpit(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let lows: Vec<bool> = self.bars.iter().map(|b| pm::is_low(*b, self.low)).collect();
        let (bw, bh) = (s * 0.26, s * 0.12);
        let (top, bottom, mid) = (s * 0.27, s * 0.56, s * 0.475);
        let boxes: Vec<(f32, f32)> = [
            (c - s * 0.2, top),
            (c + s * 0.2, top),
            (c - s * 0.2, bottom),
            (c + s * 0.2, bottom),
        ]
        .iter()
        .map(|&(x, y)| (x - bw / 2.0, y))
        .collect();
        let (boxes_p, lows_p) = (boxes.clone(), lows.clone());
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            rule(w, o, s, ink);
            let faint = scale_alpha(ink.text, 0.25);
            stroke(
                w,
                o,
                &[(c, s * 0.2), (c, s * 0.76)],
                s * 0.003,
                faint,
                false,
            );
            stroke(
                w,
                o,
                &[(s * 0.12, mid), (s * 0.88, mid)],
                s * 0.003,
                faint,
                false,
            );
            for (i, &(bx, by)) in boxes_p.iter().enumerate() {
                let frame = if lows_p[i] {
                    ink.warn
                } else {
                    scale_alpha(ink.text, 0.7)
                };
                readout_box(w, o, (bx, by, bw, bh), false, frame, ink);
            }
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(title(s, ink, &self.heading, &self.unit));
        let names = ["FL", "FR", "RL", "RR"];
        for (i, &(bx, by)) in boxes.iter().enumerate() {
            let col = if lows[i] { ink.warn } else { ink.ok };
            el = el
                .child(boxed(
                    bx,
                    by,
                    bw,
                    bh,
                    bh * 0.55,
                    col,
                    pm::pressure_text(self.bars[i]),
                ))
                .child(label(
                    bx + bw / 2.0,
                    if i < 2 {
                        by - s * 0.045
                    } else {
                        by + bh + s * 0.01
                    },
                    bw,
                    s * 0.03,
                    if lows[i] { ink.warn } else { ink.info },
                    if lows[i] {
                        format!("{} LOW", names[i])
                    } else {
                        names[i].to_string()
                    },
                ));
        }
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                s * 0.82,
                s * 0.8,
                s * 0.032,
                ink.info,
                self.note.to_uppercase(),
            ));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// A systems list: each row's icon in cyan, a dotted leader to its value in green, and a
    /// bar as a scale with a pointer under it.
    pub(super) fn cockpit(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let has_heading = !self.heading.is_empty();
        let top = if has_heading { s * 0.17 } else { s * 0.1 };
        let pitch = ((s * 0.86 - top) / n as f32).min(s * 0.2);
        let (x0, x1) = (s * 0.1, s * 0.9);
        let rows: Vec<(Option<(f32, Hsla)>, f32)> = self
            .rows
            .iter()
            .map(|r| {
                // Room for the value at the right end of the leader.
                let text_w = (r.value.chars().count() as f32 + 1.0) * s * 0.026;
                (r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))), text_w)
            })
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            if has_heading {
                rule(w, o, s, ink);
            }
            for (i, (bar, text_w)) in rows.iter().enumerate() {
                let y = top + pitch * i as f32;
                let mid = y + pitch * if bar.is_some() { 0.32 } else { 0.5 };
                for d in leader(x0 + s * 0.1, x1 - text_w, mid + s * 0.012, s * 0.02) {
                    fill(
                        w,
                        o,
                        &circle(d.0, d.1, s * 0.0035),
                        scale_alpha(ink.text, 0.5),
                    );
                }
                if let Some((f, col)) = bar {
                    pointer_scale(
                        w,
                        o,
                        (x0 + s * 0.1, x1, y + pitch * 0.78),
                        *f,
                        s * 0.012,
                        *col,
                        ink,
                    );
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if has_heading {
            el = el.child(title(s, ink, &self.heading, ""));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let mid = y + pitch * if r.bar.is_some() { 0.32 } else { 0.5 };
            let h = s * 0.06;
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(mid - h / 2.0))
                    .w(px(x1 - x0))
                    .h(px(h))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(r.icon.element(s * 0.05, ink.info))
                    .child(
                        div()
                            .text_size(px(s * 0.042))
                            .text_color(ink.ok)
                            .child(r.value.to_uppercase()),
                    ),
            );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// An alert page: the title in inverse video in the tone, the symbol beside the message in
    /// the tone, and the clear and recall keys at the foot.
    pub(super) fn cockpit(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let col = tone_or(t, self.tone, ink);
        let (hx, hy, hw, hh) = (s * 0.09, s * 0.1, s * 0.82, s * 0.11);
        // The clear and recall legends sit level with the bottom keys of the bezel.
        let (kw, kh) = (s * 0.2, s * 0.08);
        let ky = s * KEYS[4] - kh / 2.0;
        let keys = [(s * 0.1, "CLR"), (s * 0.7, "RCL")];
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            fill(w, o, &rect(hx, hy, hw, hh), col);
            for (i, (x, _)) in keys.into_iter().enumerate() {
                legend_box(w, o, s, (x, ky, kw, kh), i == 0, ink);
            }
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(hx + s * 0.03))
                    .top(px(hy))
                    .h(px(hh))
                    .flex()
                    .items_center()
                    .text_size(px(s * 0.05))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(ink.ground)
                    .child(self.title.to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.09))
                    .top(px(s * 0.28))
                    .w(px(s * 0.82))
                    .flex()
                    .gap(px(s * 0.04))
                    .child(self.icon.element(s * 0.14, col))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(s * 0.042))
                            .text_color(col)
                            .child(self.text.to_uppercase()),
                    ),
            );
        for (x, k) in keys {
            el = el.child(boxed(x, ky, kw, kh, s * 0.036, ink.info, k));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

impl GForcePod {
    /// A cross of dashed rings at half and the whole limit, the force as a bug-coloured diamond,
    /// the total in a box and the two axes in cyan.
    pub(super) fn cockpit(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let cy = s * 0.5;
        let r = s * 0.28;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let (bx, by, bw, bh) = (s * 0.62, s * 0.08, s * 0.28, s * 0.1);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            for (k, rr) in [(0, r * 0.5), (1, r)] {
                let n = 48;
                for i in (0..n).step_by(2) {
                    let a = i as f32 / n as f32;
                    stroke(
                        w,
                        o,
                        &turn_arc(c, cy, rr, a, a + 1.0 / n as f32),
                        s * 0.004,
                        scale_alpha(ink.text, if k == 1 { 0.7 } else { 0.4 }),
                        false,
                    );
                }
            }
            for (a, b) in [
                ((c - r * 1.1, cy), (c + r * 1.1, cy)),
                ((c, cy - r * 1.1), (c, cy + r * 1.1)),
            ] {
                stroke(w, o, &[a, b], s * 0.003, scale_alpha(ink.text, 0.5), false);
            }
            let (x, y) = (c + gx * r, cy - gy * r);
            let k = s * 0.03;
            fill(
                w,
                o,
                &[(x, y - k), (x + k, y), (x, y + k), (x - k, y)],
                ink.accent,
            );
            readout_box(
                w,
                o,
                (bx, by, bw, bh),
                false,
                scale_alpha(ink.text, 0.7),
                ink,
            );
        };
        let total = pm::g_text(pm::g_total(self.lateral, self.longitudinal));
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.09))
                    .top(px(s * 0.1))
                    .text_size(px(s * 0.04))
                    .text_color(ink.text)
                    .child("G-FORCE"),
            )
            .child(boxed(
                bx,
                by,
                bw,
                bh,
                bh * 0.6,
                ink.ok,
                format!("{total} G"),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.09))
                    .top(px(s * 0.85))
                    .w(px(s * 0.82))
                    .flex()
                    .justify_between()
                    .text_size(px(s * 0.036))
                    .text_color(ink.info)
                    .child(format!("LAT {}", pm::g_text(self.lateral.abs())))
                    .child(format!("LON {}", pm::g_text(self.longitudinal.abs()))),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    /// A radio panel: the title as the active line in a cyan box, the artist as the standby
    /// line, the elapsed time in green and the progress as a scale with a pointer.
    pub(super) fn cockpit(self, ink: Ink) -> AnyElement {
        let s = self.size;
        let f = pm::played(&self.time);
        let (x0, x1) = (s * 0.09, s * 0.91);
        let (ay, ah) = (s * 0.24, s * 0.13);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            rule(w, o, s, ink);
            readout_box(
                w,
                o,
                (x0 + s * 0.16, ay, x1 - x0 - s * 0.16, ah),
                false,
                ink.info,
                ink,
            );
            pointer_scale(w, o, (x0, x1, s * 0.8), f, s * 0.014, ink.ok, ink);
        };
        let line = |y: f32, tag: &'static str, text: SharedString, col: Hsla, size: f32| {
            div()
                .absolute()
                .left(px(x0))
                .top(px(y))
                .w(px(x1 - x0))
                .h(px(ah))
                .flex()
                .items_center()
                .gap(px(s * 0.04))
                .child(
                    div()
                        .w(px(s * 0.13))
                        .text_size(px(s * 0.032))
                        .text_color(ink.muted)
                        .child(tag),
                )
                .child(
                    div()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_size(px(size))
                        .text_color(col)
                        .child(text),
                )
        };
        let total = self.time.split('|').nth(1).unwrap_or("").trim().to_string();
        case(s)
            .child(layer(paint))
            .child(title(s, ink, "Audio", ""))
            .child(line(
                ay,
                "ACT",
                self.title.to_uppercase().into(),
                ink.info,
                s * 0.05,
            ))
            .child(line(
                ay + ah + s * 0.03,
                "STBY",
                self.artist.to_uppercase().into(),
                ink.text,
                s * 0.042,
            ))
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(s * 0.6))
                    .w(px(x1 - x0))
                    .flex()
                    .items_end()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(s * 0.08))
                            .text_color(ink.ok)
                            .child(pm::elapsed(&self.time).to_string()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.04))
                            .text_color(ink.muted)
                            .child(total),
                    ),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

/// Where bearing `deg` sits on a heading arc centred on `heading`: its offset in degrees, or
/// `None` beyond `half` either side. Pure.
pub(crate) fn arc_offset(deg: f32, heading: f32, half: f32) -> Option<f32> {
    let off = (deg - heading + 540.0).rem_euclid(360.0) - 180.0;
    (off.abs() <= half).then_some(off)
}

impl CompassPod {
    /// A heading arc: the top of a rose that turns under a fixed pointer, ticks every 5°, names
    /// every 30°, the bearing in a box with a notch and the cardinal in cyan.
    pub(super) fn cockpit(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (ox, oy, r) = (c, s * 0.98, s * 0.6);
        let half = 50.0;
        let heading = pm::bearing(self.heading);
        let (bw, bh) = (s * 0.2, s * 0.1);
        let (bx, by) = (c - bw / 2.0, s * 0.17);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            stroke(
                w,
                o,
                &turn_arc(ox, oy, r, -half / 360.0, half / 360.0),
                s * 0.004,
                ink.text,
                false,
            );
            for d in (0..360).step_by(5) {
                let Some(off) = arc_offset(d as f32, heading, half) else {
                    continue;
                };
                let a = off / 360.0;
                let len = if d % 10 == 0 { s * 0.035 } else { s * 0.018 };
                stroke(
                    w,
                    o,
                    &[polar(ox, oy, r, a), polar(ox, oy, r - len, a)],
                    s * 0.004,
                    ink.text,
                    false,
                );
            }
            // The fixed pointer under the box and the box with its notch.
            fill(
                w,
                o,
                &[
                    (c, oy - r + s * 0.004),
                    (c - s * 0.02, by + bh + s * 0.02),
                    (c + s * 0.02, by + bh + s * 0.02),
                ],
                ink.text,
            );
            readout_box(w, o, (bx, by, bw, bh), false, ink.text, ink);
            fill(w, o, &bug(c, by - s * 0.012, s * 0.025), ink.accent);
        };
        let mut el = case(s).child(layer(paint)).child(boxed(
            bx,
            by,
            bw,
            bh,
            bh * 0.62,
            ink.text,
            format!("{:03.0}", heading),
        ));
        for d in (0..360).step_by(30) {
            let Some(off) = arc_offset(d as f32, heading, half - 5.0) else {
                continue;
            };
            let (x, y) = polar(ox, oy, r - s * 0.08, off / 360.0);
            let name = match d {
                0 => "N".to_string(),
                90 => "E".to_string(),
                180 => "S".to_string(),
                270 => "W".to_string(),
                d => (d / 10).to_string(),
            };
            el = el.child(label(x, y - s * 0.02, s * 0.1, s * 0.04, ink.text, name));
        }
        el.child(label(
            c,
            s * 0.56,
            s * 0.4,
            s * 0.09,
            ink.info,
            pm::cardinal(heading),
        ))
        .child(label(
            c,
            s * 0.8,
            s * 0.7,
            s * 0.034,
            ink.muted,
            format!("ALT {}", self.height.to_uppercase()),
        ))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_heading_arc_shows_bearings_near_the_heading() {
        assert_eq!(arc_offset(10.0, 350.0, 50.0), Some(20.0));
        assert_eq!(arc_offset(340.0, 10.0, 50.0), Some(-30.0));
        assert_eq!(arc_offset(180.0, 0.0, 50.0), None);
        assert_eq!(arc_offset(45.0, 45.0, 0.0), Some(0.0));
    }
}
