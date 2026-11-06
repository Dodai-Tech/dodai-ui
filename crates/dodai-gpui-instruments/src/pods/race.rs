//! The race look of the pods: a carbon plate with the heading on a slanted tag. Tyre pressures
//! on slanted tiles with a status edge, rows over segment bars, a notice between hazard bands, a
//! friction circle for G-force, an equaliser for media and a heading tape for the compass. Every
//! colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::race::{
    figure, hazard, lean_edge, plate, seg_bar, skew, tag, tag_text, tape_marks, LEAN,
};
use crate::shapes::{circle, Ink};

/// The heading tag of a race pod of `s`.
fn head(s: f32) -> (f32, f32, f32, f32) {
    (s * 0.08, s * 0.1, s * 0.56, s * 0.075)
}

/// A line of leaning text with its left edge at `x`, its top at `y`.
fn lean_text(
    x: f32,
    y: f32,
    wd: f32,
    size: f32,
    col: Hsla,
    text: impl Into<SharedString>,
) -> gpui::Div {
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(wd))
        .overflow_hidden()
        .whitespace_nowrap()
        .text_size(px(size))
        .italic()
        .text_color(col)
        .child(text.into())
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four slanted tiles, front on top, each with its pressure big and leaning and an edge that
    /// turns amber when the tyre is low.
    pub(super) fn race(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (tw, th) = (s * 0.4, s * 0.23);
        let tiles = [
            (s * 0.07, s * 0.24),
            (s * 0.53, s * 0.24),
            (s * 0.07, s * 0.53),
            (s * 0.53, s * 0.53),
        ];
        let lows: Vec<bool> = self.bars.iter().map(|b| pm::is_low(*b, self.low)).collect();
        let lows_p = lows.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            tag(w, o, head(s), ink.primary);
            for (i, &(x, y)) in tiles.iter().enumerate() {
                let tile = skew(x, y, tw, th);
                fill(w, o, &tile, scale_alpha(ink.text, 0.05));
                stroke(w, o, &tile, s * 0.004, ink.line, true);
                let edge = if lows_p[i] { ink.warn } else { ink.ok };
                fill(w, o, &lean_edge(x, y, s * 0.03, th), edge);
            }
        };
        let mut el =
            case(s)
                .child(layer(paint))
                .child(tag_text(head(s), ink.on_primary, &self.heading));
        let names = ["FL", "FR", "RL", "RR"];
        for (i, &(x, y)) in tiles.iter().enumerate() {
            let col = if lows[i] { ink.warn } else { ink.text };
            let k = th * LEAN;
            el = el
                .child(lean_text(
                    x + k + s * 0.03,
                    y + s * 0.015,
                    tw * 0.6,
                    s * 0.032,
                    if lows[i] { ink.warn } else { ink.muted },
                    if lows[i] {
                        format!("{} LOW", names[i])
                    } else {
                        names[i].to_string()
                    },
                ))
                .child(
                    div()
                        .absolute()
                        .left(px(x))
                        .top(px(y + th * 0.3))
                        .w(px(tw - k * 0.6 - s * 0.03))
                        .flex()
                        .justify_end()
                        .child(figure(pm::pressure_text(self.bars[i]), th * 0.5, col)),
                );
        }
        let foot = if self.note.is_empty() {
            self.unit.to_uppercase()
        } else {
            self.note.to_uppercase()
        };
        el.child(label(c, s * 0.83, s * 0.84, s * 0.034, ink.muted, foot).italic())
            .into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// Each row's icon and leaning figure, a bar as a row of slanted segments under it.
    pub(super) fn race(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let has_heading = !self.heading.is_empty();
        let top = if has_heading { s * 0.22 } else { s * 0.12 };
        let pitch = ((s * 0.88 - top) / n as f32).min(s * 0.2);
        let (x0, x1) = (s * 0.08, s * 0.92);
        let bars: Vec<Option<(f32, Hsla)>> = self
            .rows
            .iter()
            .map(|r| r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            if has_heading {
                tag(w, o, head(s), ink.primary);
            }
            for (i, bar) in bars.iter().enumerate() {
                if let Some((f, col)) = bar {
                    let y = top + pitch * i as f32 + pitch * 0.6;
                    let col = *col;
                    seg_bar(
                        w,
                        o,
                        (x0, y, x1 - x0, pitch * 0.2),
                        *f,
                        16,
                        false,
                        |_| col,
                        ink,
                    );
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if has_heading {
            el = el.child(tag_text(head(s), ink.on_primary, &self.heading));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let h = pitch * if r.bar.is_some() { 0.55 } else { 0.8 };
            let size = (h * 0.72).min(s * 0.075);
            let (fig, rest) = pm::split_figure(&r.value);
            let value = if fig.is_empty() {
                div().child(figure(r.value.to_uppercase(), size * 0.8, ink.text))
            } else {
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(s * 0.012))
                    .child(figure(fig.to_string(), size, ink.text))
                    .child(
                        div()
                            .text_size(px(size * 0.45))
                            .italic()
                            .text_color(ink.muted)
                            .child(rest.to_uppercase()),
                    )
            };
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(y))
                    .w(px(x1 - x0))
                    .h(px(h))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(r.icon.element(size * 0.8, ink.muted))
                    .child(value),
            );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// Hazard bands above and below, the symbol dark on a slanted tag in the tone, the title big
    /// and leaning in the tone and the message under it.
    pub(super) fn race(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let badge = (c - s * 0.15, s * 0.24, s * 0.3, s * 0.19);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            hazard(w, o, (s * 0.06, s * 0.1, s * 0.8, s * 0.06), col);
            hazard(w, o, (s * 0.06, s * 0.84, s * 0.88, s * 0.06), col);
            tag(w, o, badge, col);
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(badge.0))
                    .top(px(badge.1))
                    .w(px(badge.2))
                    .h(px(badge.3))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(self.icon.element(badge.3 * 0.62, ink.ground)),
            )
            .child(
                text_box(c, s * 0.47, s * 0.86)
                    .flex()
                    .justify_center()
                    .child(figure(self.title.to_uppercase(), s * 0.062, col)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.1))
                    .top(px(s * 0.6))
                    .w(px(s * 0.8))
                    .text_center()
                    .text_size(px(s * 0.04))
                    .italic()
                    .text_color(ink.muted)
                    .child(self.text.clone()),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

impl GForcePod {
    /// A friction circle: rings at a third, two thirds and the limit, a cross, the force as a dot
    /// in the primary colour, and the total big and leaning under it.
    pub(super) fn race(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (cy, r) = (s * 0.49, s * 0.24);
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            tag(w, o, head(s), ink.primary);
            for k in 1..=3 {
                let a = if k == 3 { 0.45 } else { 0.18 };
                stroke(
                    w,
                    o,
                    &circle(c, cy, r * k as f32 / 3.0),
                    s * 0.004,
                    scale_alpha(ink.text, a),
                    true,
                );
            }
            let faint = scale_alpha(ink.text, 0.18);
            stroke(w, o, &[(c - r, cy), (c + r, cy)], s * 0.003, faint, false);
            stroke(w, o, &[(c, cy - r), (c, cy + r)], s * 0.003, faint, false);
            // Forward force points up, rightward force to the right.
            let (dx, dy) = (c + gx * r, cy - gy * r);
            stroke(
                w,
                o,
                &[(c, cy), (dx, dy)],
                s * 0.006,
                scale_alpha(ink.primary, 0.5),
                false,
            );
            fill(
                w,
                o,
                &circle(dx, dy, s * 0.032),
                scale_alpha(ink.primary, 0.25),
            );
            fill(w, o, &circle(dx, dy, s * 0.02), ink.primary);
        };
        let total = pm::g_text(pm::g_total(self.lateral, self.longitudinal));
        case(s)
            .child(layer(paint))
            .child(tag_text(head(s), ink.on_primary, "G-force"))
            .child(
                text_box(c, s * 0.75, s * 0.4)
                    .flex()
                    .justify_center()
                    .items_baseline()
                    .gap(px(s * 0.01))
                    .child(figure(total, s * 0.1, ink.text))
                    .child(
                        div()
                            .text_size(px(s * 0.04))
                            .italic()
                            .text_color(ink.muted)
                            .child("G"),
                    ),
            )
            .child(lean_text(
                s * 0.08,
                s * 0.8,
                s * 0.3,
                s * 0.032,
                ink.muted,
                format!("LAT {}", pm::g_text(self.lateral.abs())),
            ))
            .child(
                lean_text(
                    s * 0.62,
                    s * 0.8,
                    s * 0.3,
                    s * 0.032,
                    ink.muted,
                    format!("LON {}", pm::g_text(self.longitudinal.abs())),
                )
                .text_right(),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    /// The title big and leaning with the artist under it, an equaliser of slanted bars, and the
    /// progress as a row of segments with the elapsed time.
    pub(super) fn race(self, ink: Ink) -> AnyElement {
        let s = self.size;
        let f = pm::played(&self.time);
        let bands = pm::spectrum(&self.title, 12);
        let (x0, x1) = (s * 0.08, s * 0.92);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            let (y0, h) = (s * 0.4, s * 0.26);
            // The bars lean, so the last one's top reaches `h * LEAN` past its foot.
            let cw = (x1 - x0 - h * LEAN) / bands.len() as f32;
            for (i, b) in bands.iter().enumerate() {
                let bh = h * b.clamp(0.08, 1.0);
                let (x, bw) = (x0 + cw * i as f32, cw * 0.62);
                fill(w, o, &lean_edge(x, y0, bw, h), scale_alpha(ink.text, 0.06));
                // The lit part is the foot of the same bar, so its left edge moves in with it.
                fill(
                    w,
                    o,
                    &lean_edge(x, y0 + h - bh, bw, bh),
                    scale_alpha(ink.primary, 0.55 + 0.45 * b),
                );
            }
            seg_bar(
                w,
                o,
                (x0, s * 0.72, x1 - x0, s * 0.045),
                f,
                24,
                false,
                |_| ink.primary,
                ink,
            );
        };
        case(s)
            .child(layer(paint))
            .child(
                lean_text(
                    x0,
                    s * 0.12,
                    x1 - x0,
                    s * 0.07,
                    ink.text,
                    self.title.clone(),
                )
                .font_weight(FontWeight::BOLD),
            )
            .child(lean_text(
                x0,
                s * 0.23,
                x1 - x0,
                s * 0.04,
                ink.muted,
                self.artist.clone(),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(s * 0.79))
                    .w(px(x1 - x0))
                    .flex()
                    .justify_end()
                    .child(figure(
                        pm::elapsed(&self.time).to_string(),
                        s * 0.07,
                        ink.text,
                    )),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

impl CompassPod {
    /// The heading big and leaning with the cardinal on a slanted tag, over a heading tape that
    /// slides under a fixed pointer.
    pub(super) fn race(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let heading = pm::bearing(self.heading);
        let (x0, x1, ty) = (s * 0.08, s * 0.92, s * 0.6);
        let per_deg = (x1 - x0) / 120.0;
        let marks = tape_marks(heading, 60.0, 5.0);
        let card = (s * 0.62, s * 0.2, s * 0.24, s * 0.12);
        let marks_p = marks.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            tag(w, o, card, ink.primary);
            stroke(
                w,
                o,
                &[(x0, ty), (x1, ty)],
                s * 0.004,
                scale_alpha(ink.text, 0.5),
                false,
            );
            for &(off, deg) in &marks_p {
                let x = c + off * per_deg;
                let len = if deg % 30 == 0 {
                    s * 0.05
                } else if deg % 10 == 0 {
                    s * 0.032
                } else {
                    s * 0.018
                };
                stroke(
                    w,
                    o,
                    &[(x, ty), (x, ty + len)],
                    s * 0.004,
                    scale_alpha(ink.text, 0.7),
                    false,
                );
            }
            fill(
                w,
                o,
                &[
                    (c, ty - s * 0.005),
                    (c - s * 0.03, ty - s * 0.05),
                    (c + s * 0.03, ty - s * 0.05),
                ],
                ink.primary,
            );
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.08))
                    .top(px(s * 0.16))
                    .w(px(s * 0.5))
                    .flex()
                    .justify_end()
                    .child(figure(format!("{heading:03.0}°"), s * 0.18, ink.text)),
            )
            .child(tag_text(card, ink.on_primary, pm::cardinal(heading)));
        for (off, deg) in marks.into_iter().filter(|(_, d)| d % 30 == 0) {
            let name = match deg {
                0 => "N".to_string(),
                90 => "E".to_string(),
                180 => "S".to_string(),
                270 => "W".to_string(),
                d => d.to_string(),
            };
            el = el.child(
                label(
                    c + off * per_deg,
                    ty + s * 0.065,
                    s * 0.12,
                    s * 0.038,
                    ink.muted,
                    name,
                )
                .italic(),
            );
        }
        el.child(
            label(
                c,
                s * 0.83,
                s * 0.7,
                s * 0.034,
                ink.muted,
                format!("ALT {}", self.height.to_uppercase()),
            )
            .italic(),
        )
        .into_any_element()
    }
}
