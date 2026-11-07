//! The halo look of the pods: a halo of light round a dark glass, gradient arcs, glowing
//! dots and blades, pills of glass and thin type. Every colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::frames::{glow_arc, glow_dot, halo, light_blade, pill};
use crate::shapes::Ink;
use dodai_gpui::paint::stroke_circle;

/// Thin large type for a halo figure.
fn figure(
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    col: Hsla,
    text: impl Into<SharedString>,
) -> gpui::Div {
    label(x, y, w, size, col, text).font_weight(FontWeight::LIGHT)
}

/// The small spaced heading of a halo pod.
fn heading(x: f32, y: f32, s: f32, ink: Ink, text: &str) -> gpui::Div {
    label(x, y, s * 0.6, s * 0.03, ink.primary, text.to_uppercase())
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four quadrant arcs of light round the glass, each filled to its pressure, the figures
    /// inside them in thin type.
    pub(super) fn futuristic(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let quads = [0.875_f32, 0.125, 0.625, 0.375];
        let reads: Vec<(Option<f32>, bool)> = self
            .bars
            .iter()
            .map(|b| (pm::pressure_frac(*b, self.low), pm::is_low(*b, self.low)))
            .collect();
        let reads_paint = reads.clone();
        let r_arc = c - s * 0.13;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            for (&q, (f, low)) in quads.iter().zip(&reads_paint) {
                let (a0, a1) = (q - 0.09, q + 0.09);
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r_arc, a0, a1),
                    s * 0.018,
                    scale_alpha(ink.text, 0.07),
                    false,
                );
                let col = if *low { ink.warn } else { ink.primary };
                if let Some(f) = f {
                    // Upper arcs fill clockwise from the middle line, lower ones mirror them.
                    let (from, to) = if q < 0.5 {
                        (a0, a0 + (a1 - a0) * f)
                    } else {
                        (a1 - (a1 - a0) * f, a1)
                    };
                    glow_arc(w, o, (c, c), r_arc, (from, to), s * 0.01, (ink.info, col));
                }
            }
            stroke(
                w,
                o,
                &[(c - s * 0.2, c), (c + s * 0.2, c)],
                s * 0.002,
                scale_alpha(ink.primary, 0.3),
                false,
            );
            stroke(
                w,
                o,
                &[(c, c - s * 0.2), (c, c + s * 0.2)],
                s * 0.002,
                scale_alpha(ink.primary, 0.3),
                false,
            );
        };
        let mut el = case(s).child(layer(paint));
        let names = ["FL", "FR", "RL", "RR"];
        for ((&q, (_, low)), (name, bar)) in
            quads.iter().zip(&reads).zip(names.iter().zip(self.bars))
        {
            let (x, y) = polar(c, c, s * 0.19, q);
            el = el
                .child(figure(
                    x,
                    y - s * 0.05,
                    s * 0.2,
                    s * 0.075,
                    if *low { ink.warn } else { ink.text },
                    pm::pressure_text(bar),
                ))
                .child(label(
                    x,
                    y + s * 0.035,
                    s * 0.1,
                    s * 0.026,
                    ink.muted,
                    *name,
                ));
        }
        el = el.child(heading(c, s * 0.075, s, ink, &self.heading));
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                s * 0.86,
                s * 0.6,
                s * 0.026,
                ink.muted,
                self.note.clone(),
            ));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// Every row in a pill of glass: the icon in a glowing ring, the value in thin type, a level
    /// as a line of light ending in a dot.
    pub(super) fn futuristic(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let pitch = (s * 0.5 / n as f32).min(s * 0.13);
        let top = c - pitch * n as f32 / 2.0
            + if self.heading.is_empty() {
                0.0
            } else {
                s * 0.03
            };
        let (x0, x1) = (c - s * 0.3, c + s * 0.3);
        let rows: Vec<Option<(f32, Hsla)>> = self
            .rows
            .iter()
            .map(|r| r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            for (i, bar) in rows.iter().enumerate() {
                let y = top + pitch * i as f32;
                let h = pitch * 0.8;
                let p = pill(x0, y, x1 - x0, h);
                fill(w, o, &p, scale_alpha(ink.text, 0.05));
                stroke(w, o, &p, s * 0.003, scale_alpha(ink.primary, 0.3), true);
                let (ix, iy) = (x0 + h / 2.0, y + h / 2.0);
                stroke_circle(
                    w,
                    o,
                    (ix, iy),
                    h * 0.36,
                    s * 0.003,
                    scale_alpha(ink.info, 0.6),
                );
                if let Some((f, col)) = bar {
                    let (bx0, bx1) = (x0 + h * 1.1, x1 - h * 0.5);
                    let by = y + h * 0.82;
                    stroke(
                        w,
                        o,
                        &[(bx0, by), (bx1, by)],
                        s * 0.003,
                        scale_alpha(ink.text, 0.12),
                        false,
                    );
                    let fx = bx0 + (bx1 - bx0) * f;
                    stroke(
                        w,
                        o,
                        &[(bx0, by), (fx, by)],
                        s * 0.009,
                        scale_alpha(*col, 0.25),
                        false,
                    );
                    stroke(w, o, &[(bx0, by), (fx, by)], s * 0.003, *col, false);
                    glow_dot(w, o, (fx, by), s * 0.006, *col, ink.text);
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(heading(c, top - s * 0.06, s, ink, &self.heading));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let h = pitch * 0.8;
            el = el
                .child(
                    div()
                        .absolute()
                        .left(px(x0 + h / 2.0 - h * 0.22))
                        .top(px(y + h / 2.0 - h * 0.22))
                        .child(r.icon.element(h * 0.44, ink.text)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x0 + h * 1.1))
                        .top(px(y + h * (if r.bar.is_some() { 0.1 } else { 0.22 })))
                        .w(px(x1 - x0 - h * 1.6))
                        .flex()
                        .justify_end()
                        .whitespace_nowrap()
                        .text_size(px(s * 0.042))
                        .font_weight(FontWeight::LIGHT)
                        .text_color(ink.text)
                        .child(r.value.clone()),
                );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// The symbol in an orb of the tone with rings pulsing out of it, the title in thin type.
    pub(super) fn futuristic(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let (ox, oy, or) = (c, c - s * 0.1, s * 0.075);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            for k in 1..=4 {
                stroke_circle(
                    w,
                    o,
                    (ox, oy),
                    or * (1.0 + k as f32 * 0.38),
                    s * 0.003,
                    scale_alpha(col, 0.5 / k as f32),
                );
            }
            glow_dot(
                w,
                o,
                (ox, oy),
                or,
                scale_alpha(col, 0.85),
                scale_alpha(col, 0.0),
            );
            glow_arc(
                w,
                o,
                (c, c),
                c - s * 0.16,
                (0.62, 0.88),
                s * 0.006,
                (col, ink.info),
            );
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(ox - or * 0.6))
                    .top(px(oy - or * 0.6))
                    .child(self.icon.element(or * 1.2, ink.face)),
            )
            .child(figure(
                c,
                c + s * 0.06,
                s * 0.6,
                s * 0.06,
                ink.text,
                self.title.clone(),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(c - s * 0.25))
                    .top(px(c + s * 0.16))
                    .w(px(s * 0.5))
                    .text_center()
                    .text_size(px(s * 0.032))
                    .text_color(ink.muted)
                    .child(self.text.clone()),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

impl GForcePod {
    /// Rings of light, the force as a blade from the middle to a glowing dot, and an arc on the
    /// rim pointing the way the force pulls.
    pub(super) fn futuristic(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let reach = s * 0.26;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let (dx, dy) = (c + gx * reach, c - gy * reach);
        let len = (gx * gx + gy * gy).sqrt();
        // The turn the force points at (0 = forward, clockwise).
        let turn = gx.atan2(gy) / std::f32::consts::TAU;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_in = halo(w, o, s, ink);
            for k in 1..=3 {
                stroke_circle(
                    w,
                    o,
                    (c, c),
                    reach * k as f32 / 3.0,
                    s * 0.003,
                    scale_alpha(ink.primary, 0.12 + 0.1 * k as f32),
                );
            }
            stroke(
                w,
                o,
                &[(c - reach, c), (c + reach, c)],
                s * 0.002,
                scale_alpha(ink.text, 0.2),
                false,
            );
            stroke(
                w,
                o,
                &[(c, c - reach), (c, c + reach)],
                s * 0.002,
                scale_alpha(ink.text, 0.2),
                false,
            );
            if len > 0.02 {
                glow_arc(
                    w,
                    o,
                    (c, c),
                    r_in - s * 0.02,
                    (turn - 0.06, turn + 0.06),
                    s * 0.01,
                    (ink.info, ink.primary),
                );
                light_blade(
                    w,
                    o,
                    (c, c),
                    (0.0, reach * len.min(1.0)),
                    turn,
                    s * 0.005,
                    ink.primary,
                    ink.text,
                );
            } else {
                glow_dot(w, o, (dx, dy), s * 0.008, ink.primary, ink.text);
            }
        };
        case(s)
            .child(layer(paint))
            .child(heading(c, s * 0.12, s, ink, "G-force"))
            .child(figure(
                c,
                c + reach + s * 0.0,
                s * 0.3,
                s * 0.06,
                ink.text,
                format!(
                    "{} g",
                    pm::g_text(pm::g_total(self.lateral, self.longitudinal))
                ),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    /// The cover framed by a ring of light that fills with the progress, the spectrum as radial
    /// beams round it, the title in thin type.
    pub(super) fn futuristic(self, ink: Ink, cover: Option<AnyElement>) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let (vx, vy) = (c, c - s * 0.08);
        let side = s * 0.18;
        let f = pm::played(&self.time);
        let beams = pm::spectrum(&self.title, 48);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            let r0 = side * 0.78;
            for (i, b) in beams.iter().enumerate() {
                let a = i as f32 / beams.len() as f32;
                stroke(
                    w,
                    o,
                    &[polar(vx, vy, r0, a), polar(vx, vy, r0 + s * 0.06 * b, a)],
                    s * 0.006,
                    scale_alpha(
                        dodai_gpui::motion::mix(ink.info, ink.primary, *b),
                        0.4 + 0.5 * b,
                    ),
                    false,
                );
            }
            stroke_circle(
                w,
                o,
                (vx, vy),
                r0 - s * 0.012,
                s * 0.004,
                scale_alpha(ink.text, 0.1),
            );
            glow_arc(
                w,
                o,
                (vx, vy),
                r0 - s * 0.012,
                (0.0, f.max(0.003)),
                s * 0.006,
                (ink.info, ink.primary),
            );
        };
        let cover = cover.unwrap_or_else(|| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(scale_alpha(ink.primary, 0.25))
                .child(Icon::new(IconId::Music).size(side * 0.5).color(ink.text))
                .into_any_element()
        });
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(vx - side / 2.0))
                    .top(px(vy - side / 2.0))
                    .w(px(side))
                    .h(px(side))
                    .overflow_hidden()
                    .rounded(px(side * 0.12))
                    .child(cover),
            )
            .child(figure(
                c,
                c + s * 0.15,
                s * 0.6,
                s * 0.05,
                ink.text,
                self.title.clone(),
            ))
            .child(label(
                c,
                c + s * 0.22,
                s * 0.6,
                s * 0.03,
                ink.muted,
                self.artist.clone(),
            ))
            .child(label(
                c,
                c + s * 0.27,
                s * 0.4,
                s * 0.028,
                ink.primary,
                self.time.clone(),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

impl CompassPod {
    /// A ring of degrees turning under a fixed chevron of light, the bearing in thin type.
    pub(super) fn futuristic(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let turn = self.heading / 360.0;
        let r_ring = c - s * 0.14;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            halo(w, o, s, ink);
            for i in 0..120 {
                let a = i as f32 / 120.0 - turn;
                let major = i % 10 == 0;
                let near =
                    (((i as f32 / 120.0 - turn).rem_euclid(1.0) + 0.5).rem_euclid(1.0) - 0.5).abs();
                let glow = (1.0 - near * 8.0).clamp(0.0, 1.0);
                stroke(
                    w,
                    o,
                    &[
                        polar(c, c, r_ring - if major { s * 0.035 } else { s * 0.018 }, a),
                        polar(c, c, r_ring, a),
                    ],
                    if major { s * 0.005 } else { s * 0.003 },
                    scale_alpha(
                        dodai_gpui::motion::mix(ink.muted, ink.primary, glow),
                        0.4 + 0.6 * glow,
                    ),
                    false,
                );
            }
            glow_arc(
                w,
                o,
                (c, c),
                r_ring + s * 0.015,
                (-0.05, 0.05),
                s * 0.008,
                (ink.info, ink.primary),
            );
            let k = s * 0.025;
            let tip = (c, c - r_ring + s * 0.05);
            fill(
                w,
                o,
                &[
                    tip,
                    (tip.0 - k, tip.1 + k * 1.2),
                    (tip.0, tip.1 + k * 0.6),
                    (tip.0 + k, tip.1 + k * 1.2),
                ],
                ink.primary,
            );
            stroke_circle(
                w,
                o,
                (c, c),
                s * 0.15,
                s * 0.003,
                scale_alpha(ink.primary, 0.3),
            );
        };
        let mut el = case(s).child(layer(paint));
        for (k, name) in ["N", "E", "S", "W"].iter().enumerate() {
            let (x, y) = polar(c, c, r_ring - s * 0.07, k as f32 / 4.0 - turn);
            let col = if k == 0 { ink.danger } else { ink.text };
            el = el.child(label(x, y - s * 0.022, s * 0.08, s * 0.036, col, *name));
        }
        el.child(figure(
            c,
            c - s * 0.06,
            s * 0.3,
            s * 0.09,
            ink.text,
            format!("{:03.0}°", self.heading),
        ))
        .child(label(
            c,
            c + s * 0.05,
            s * 0.2,
            s * 0.032,
            ink.primary,
            pm::cardinal(self.heading),
        ))
        .child(label(
            c,
            c + s * 0.24,
            s * 0.3,
            s * 0.028,
            ink.muted,
            self.height.clone(),
        ))
        .into_any_element()
    }
}
