//! The vintage look of the pods: a screwed plate with a polished bezel round a warm face,
//! engraved scales, spade hands, lamp lenses, a reel-to-reel deck and a compass card. Every
//! colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::gpui::frames::{aged, name_plate, plate};
use crate::gpui::shapes::{band, chrono_glass, star, Ink};
use dodai_gpui::paint::{fill_circle, stroke_circle};

/// A small engraved half dial at `(x, y)` of radius `r` over the top half: ticks, a band to
/// `low` in the warning colour and a thin hand at `f`.
fn half_dial(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    (x, y, r): (f32, f32, f32),
    low: f32,
    f: Option<f32>,
    hand: Hsla,
    ink: Ink,
) {
    let (start, span) = (0.75_f32, 0.5_f32);
    fill_circle(w, o, (x, y), r, scale_alpha(ink.text, 0.05));
    stroke_circle(w, o, (x, y), r, r * 0.04, scale_alpha(ink.text, 0.35));
    fill(
        w,
        o,
        &band(x, y, r * 0.66, r * 0.76, start, start + span * low),
        scale_alpha(ink.warn, 0.85),
    );
    for i in 0..=20 {
        let a = start + span * i as f32 / 20.0;
        let major = i % 5 == 0;
        stroke(
            w,
            o,
            &[
                polar(x, y, if major { r * 0.62 } else { r * 0.7 }, a),
                polar(x, y, r * 0.8, a),
            ],
            if major { r * 0.05 } else { r * 0.025 },
            ink.text,
            false,
        );
    }
    if let Some(f) = f {
        let a = start + span * f;
        stroke(
            w,
            o,
            &[polar(x, y, -r * 0.12, a), polar(x, y, r * 0.78, a)],
            r * 0.05,
            hand,
            false,
        );
    }
    fill_circle(w, o, (x, y), r * 0.09, ink.text);
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four engraved half dials on a plate, the low band of each in the warning colour.
    pub(super) fn vintage(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let low_f = pm::pressure_frac(Some(self.low), self.low).unwrap_or(0.5);
        let r = s * 0.122;
        let centres = [
            (c - s * 0.135, c - s * 0.03),
            (c + s * 0.135, c - s * 0.03),
            (c - s * 0.135, c + s * 0.2),
            (c + s * 0.135, c + s * 0.2),
        ];
        let reads: Vec<(Option<f32>, bool)> = self
            .bars
            .iter()
            .map(|b| (pm::pressure_frac(*b, self.low), pm::is_low(*b, self.low)))
            .collect();
        let reads_paint = reads.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            name_plate(w, o, (c - s * 0.15, c - s * 0.27, s * 0.3, s * 0.065), ink);
            for (&(x, y), (f, low)) in centres.iter().zip(&reads_paint) {
                let hand = if *low { ink.warn } else { ink.text };
                half_dial(w, o, (x, y, r), low_f, *f, hand, ink);
            }
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint)).child(label(
            c,
            c - s * 0.262,
            s * 0.28,
            s * 0.034,
            ink.text,
            self.heading.to_uppercase(),
        ));
        let names = ["FL", "FR", "RL", "RR"];
        for ((&(x, y), (_, low)), (name, bar)) in
            centres.iter().zip(&reads).zip(names.iter().zip(self.bars))
        {
            el = el
                .child(label(x, y - r * 0.5, r, s * 0.026, ink.muted, *name))
                .child(label(
                    x,
                    y + r * 0.12,
                    r * 2.0,
                    s * 0.045,
                    if *low { ink.warn } else { ink.text },
                    pm::pressure_text(bar),
                ));
        }
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                c + s * 0.315,
                s * 0.55,
                s * 0.028,
                ink.muted,
                self.note.clone(),
            ));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// Rows engraved on the face between double rules, a level as a small scale with a pointer.
    pub(super) fn vintage(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let pitch = (s * 0.46 / n as f32).min(s * 0.12);
        let top = c - pitch * n as f32 / 2.0 + s * 0.04;
        let (x0, x1) = (c - s * 0.27, c + s * 0.27);
        let (rx0, rx1) = (x0 + s * 0.08, x1 - s * 0.17);
        let rows: Vec<(Option<(f32, Hsla)>, bool)> = self
            .rows
            .iter()
            .map(|r| (r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))), r.span))
            .collect();
        let has_heading = !self.heading.is_empty();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            if has_heading {
                name_plate(w, o, (c - s * 0.15, top - s * 0.1, s * 0.3, s * 0.06), ink);
            }
            for (i, (bar, span)) in rows.iter().enumerate() {
                let y = top + pitch * (i as f32 + 0.5);
                let y_rule = y + pitch / 2.0;
                if i + 1 < rows.len() {
                    for d in [-s * 0.004, s * 0.004] {
                        stroke(
                            w,
                            o,
                            &[(x0, y_rule + d), (x1, y_rule + d)],
                            s * 0.002,
                            scale_alpha(ink.text, 0.3),
                            false,
                        );
                    }
                }
                if let Some((f, col)) = bar {
                    let base = y + pitch * 0.18;
                    for k in 0..=10 {
                        let x = rx0 + (rx1 - rx0) * k as f32 / 10.0;
                        let h = if k % 5 == 0 { pitch * 0.2 } else { pitch * 0.1 };
                        stroke(
                            w,
                            o,
                            &[(x, base - h), (x, base)],
                            s * 0.003,
                            ink.text,
                            false,
                        );
                    }
                    stroke(
                        w,
                        o,
                        &[(rx0, base), (rx1, base)],
                        s * 0.003,
                        ink.text,
                        false,
                    );
                    let fx = rx0 + (rx1 - rx0) * f;
                    let k = s * 0.014;
                    fill(
                        w,
                        o,
                        &[
                            (fx, base - s * 0.002),
                            (fx - k, base + k * 1.6),
                            (fx + k, base + k * 1.6),
                        ],
                        *col,
                    );
                    if *span {
                        fill_circle(w, o, (rx0, base), s * 0.007, ink.info);
                        fill_circle(w, o, (rx1, base), s * 0.007, ink.danger);
                    }
                }
            }
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint));
        if has_heading {
            el = el.child(label(
                c,
                top - s * 0.093,
                s * 0.28,
                s * 0.032,
                ink.text,
                self.heading.to_uppercase(),
            ));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * (i as f32 + 0.5);
            el = el
                .child(
                    div()
                        .absolute()
                        .left(px(x0))
                        .top(px(y - s * 0.032))
                        .child(r.icon.element(s * 0.055, ink.text)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x1 - s * 0.16))
                        .top(px(y - s * 0.03))
                        .w(px(s * 0.16))
                        .flex()
                        .justify_end()
                        .whitespace_nowrap()
                        .text_size(px(s * 0.042))
                        .text_color(ink.text)
                        .child(r.value.clone()),
                );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// A lamp lens in a polished ring, lit in the tone, the title on a plate and the advice under.
    pub(super) fn vintage(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let (lx, ly, lr) = (c, c - s * 0.1, s * 0.12);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            // The lamp: a knurled ring, the lit lens and a glint.
            fill_circle(w, o, (lx, ly), lr * 1.3, scale_alpha(ink.text, 0.2));
            for i in 0..48 {
                let a = i as f32 / 48.0;
                stroke(
                    w,
                    o,
                    &[polar(lx, ly, lr * 1.12, a), polar(lx, ly, lr * 1.28, a)],
                    s * 0.003,
                    scale_alpha(ink.text, 0.35),
                    false,
                );
            }
            fill_circle(w, o, (lx, ly), lr * 1.08, ink.line);
            for k in 0..6 {
                let rr = lr * (1.0 - k as f32 * 0.12);
                fill_circle(w, o, (lx, ly), rr, scale_alpha(col, 0.22));
            }
            stroke_circle(w, o, (lx, ly), lr, s * 0.004, col);
            let mut glint = turn_arc(lx, ly, lr * 0.85, 0.8, 0.97);
            glint.extend(turn_arc(lx, ly, lr * 0.65, 0.97, 0.8));
            fill(w, o, &glint, scale_alpha(ink.text, 0.35));
            name_plate(w, o, (c - s * 0.2, c + s * 0.08, s * 0.4, s * 0.07), ink);
            chrono_glass(w, o, s, r_face, ink);
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(lx - lr * 0.5))
                    .top(px(ly - lr * 0.5))
                    .child(self.icon.element(lr, ink.text)),
            )
            .child(label(
                c,
                c + s * 0.093,
                s * 0.36,
                s * 0.036,
                ink.text,
                self.title.to_uppercase(),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(c - s * 0.26))
                    .top(px(c + s * 0.18))
                    .w(px(s * 0.52))
                    .text_center()
                    .text_size(px(s * 0.033))
                    .text_color(ink.muted)
                    .child(self.text.clone()),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

impl GForcePod {
    /// A bubble level: engraved rings with their g, a fine cross and a ball with a glint.
    pub(super) fn vintage(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let cy = c - s * 0.03;
        let reach = s * 0.25;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let (bx, by) = (c + gx * reach, cy - gy * reach);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            fill_circle(w, o, (c, cy), reach, scale_alpha(ink.text, 0.04));
            for k in 1..=4 {
                stroke_circle(
                    w,
                    o,
                    (c, cy),
                    reach * k as f32 / 4.0,
                    if k == 4 { s * 0.005 } else { s * 0.0025 },
                    scale_alpha(ink.text, if k == 4 { 0.8 } else { 0.4 }),
                );
            }
            for (ax, ay) in [(1.0_f32, 0.0_f32), (0.0, 1.0)] {
                stroke(
                    w,
                    o,
                    &[
                        (c - ax * reach, cy - ay * reach),
                        (c + ax * reach, cy + ay * reach),
                    ],
                    s * 0.003,
                    scale_alpha(ink.text, 0.5),
                    false,
                );
                for i in -10..=10 {
                    let d = reach * i as f32 / 10.0;
                    let k = if i % 5 == 0 { s * 0.012 } else { s * 0.006 };
                    let (x, y) = (c + ax * d, cy + ay * d);
                    stroke(
                        w,
                        o,
                        &[(x - ay * k, y - ax * k), (x + ay * k, y + ax * k)],
                        s * 0.002,
                        ink.text,
                        false,
                    );
                }
            }
            // The ball: a shadow, the body and a glint.
            let br = s * 0.04;
            fill_circle(
                w,
                o,
                (bx + br * 0.25, by + br * 0.35),
                br,
                scale_alpha(ink.text, 0.15),
            );
            fill_circle(w, o, (bx, by), br, ink.text);
            fill_circle(w, o, (bx - br * 0.35, by - br * 0.35), br * 0.3, aged(ink));
            name_plate(w, o, (c - s * 0.15, c + s * 0.25, s * 0.3, s * 0.065), ink);
            chrono_glass(w, o, s, r_face, ink);
        };
        let lim = self.limit;
        case(s)
            .child(layer(paint))
            .child(label(
                c + reach * 0.5 + s * 0.02,
                cy - s * 0.035,
                s * 0.1,
                s * 0.026,
                ink.muted,
                pm::g_text(lim * 0.5),
            ))
            .child(label(
                c + reach + s * 0.035,
                cy - s * 0.035,
                s * 0.1,
                s * 0.026,
                ink.muted,
                pm::g_text(lim),
            ))
            .child(label(
                c,
                c + s * 0.258,
                s * 0.28,
                s * 0.04,
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
    /// A reel-to-reel deck: two reels whose tape packs trade size as the track plays, the tape
    /// over two guides, and the title on a plate.
    pub(super) fn vintage(self, ink: Ink) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let f = pm::played(&self.time);
        let (ly, rr) = (c - s * 0.09, s * 0.125);
        let reels = [(c - s * 0.14, f), (c + s * 0.14, 1.0 - f)];
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            let guides = [(c - s * 0.07, c + s * 0.05), (c + s * 0.07, c + s * 0.05)];
            for (i, &(x, used)) in reels.iter().enumerate() {
                // The pack shrinks on the feeding reel and grows on the other.
                let pack = rr * (0.42 + 0.5 * (1.0 - used));
                fill_circle(w, o, (x, ly), rr, scale_alpha(ink.text, 0.06));
                stroke_circle(w, o, (x, ly), rr, s * 0.004, scale_alpha(ink.text, 0.5));
                fill_circle(w, o, (x, ly), pack, scale_alpha(ink.text, 0.32));
                for g in 1..4 {
                    stroke_circle(
                        w,
                        o,
                        (x, ly),
                        pack * g as f32 / 4.0 + rr * 0.1,
                        s * 0.0015,
                        scale_alpha(ink.text, 0.2),
                    );
                }
                // The hub: three windows and a centre.
                fill_circle(w, o, (x, ly), rr * 0.36, aged(ink));
                for k in 0..3 {
                    let a = k as f32 / 3.0 + i as f32 * 0.1 + used * 2.0;
                    let (hx, hy) = polar(x, ly, rr * 0.2, a);
                    fill_circle(w, o, (hx, hy), rr * 0.08, ink.text);
                }
                fill_circle(w, o, (x, ly), rr * 0.06, ink.text);
                // Tape from the pack down to its guide.
                let side = if i == 0 { -1.0 } else { 1.0 };
                let (gx, gy) = guides[i];
                stroke(
                    w,
                    o,
                    &[(x + side * pack, ly), (gx + side * s * 0.012, gy)],
                    s * 0.004,
                    scale_alpha(ink.text, 0.7),
                    false,
                );
            }
            for &(gx, gy) in &guides {
                fill_circle(w, o, (gx, gy), s * 0.012, ink.text);
            }
            stroke(
                w,
                o,
                &[
                    (guides[0].0, guides[0].1 + s * 0.012),
                    (guides[1].0, guides[1].1 + s * 0.012),
                ],
                s * 0.004,
                scale_alpha(ink.text, 0.7),
                false,
            );
            // The head between the guides.
            fill(
                w,
                o,
                &crate::gpui::shapes::chamfered(
                    c - s * 0.03,
                    c + s * 0.065,
                    s * 0.06,
                    s * 0.03,
                    s * 0.008,
                ),
                scale_alpha(ink.text, 0.5),
            );
            name_plate(w, o, (c - s * 0.24, c + s * 0.13, s * 0.48, s * 0.075), ink);
            chrono_glass(w, o, s, r_face, ink);
        };
        case(s)
            .child(layer(paint))
            .child(label(
                c,
                c + s * 0.145,
                s * 0.44,
                s * 0.036,
                ink.text,
                self.title.clone(),
            ))
            .child(label(
                c,
                c + s * 0.225,
                s * 0.5,
                s * 0.03,
                ink.muted,
                self.artist.clone(),
            ))
            .child(label(
                c,
                c + s * 0.27,
                s * 0.4,
                s * 0.028,
                ink.muted,
                self.time.clone(),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

impl CompassPod {
    /// A compass card that turns: degree ring with numbers, a two-tone eight-point star, a fixed
    /// lubber line and the bearing on a plate.
    pub(super) fn vintage(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let turn = self.heading / 360.0;
        let r_ring = c - s * 0.115;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            stroke_circle(w, o, (c, c), r_ring, s * 0.003, ink.text);
            stroke_circle(w, o, (c, c), r_ring - s * 0.06, s * 0.003, ink.text);
            for i in 0..72 {
                let a = i as f32 / 72.0 - turn;
                let len = if i % 18 == 0 {
                    s * 0.035
                } else if i % 2 == 0 {
                    s * 0.02
                } else {
                    s * 0.012
                };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r_ring - len, a), polar(c, c, r_ring, a)],
                    s * 0.003,
                    ink.text,
                    false,
                );
            }
            // Two-tone star: every point has a dark half and a light half.
            for (long, short, off) in [(s * 0.14, s * 0.03, 0.125), (s * 0.2, s * 0.04, 0.0)] {
                for q in 0..4 {
                    let a = q as f32 / 4.0 + off - turn;
                    let tip = polar(c, c, long, a);
                    let left = polar(c, c, short, a - 0.125);
                    let right = polar(c, c, short, a + 0.125);
                    fill(w, o, &[(c, c), left, tip], ink.text);
                    fill(w, o, &[(c, c), tip, right], aged(ink));
                    stroke(w, o, &[left, tip, right], s * 0.003, ink.text, false);
                }
            }
            fill(w, o, &star(c, c, s * 0.02, s * 0.01, -turn, 4), ink.warn);
            // The lubber line: fixed, over the card.
            stroke(
                w,
                o,
                &[(c, c - r_ring - s * 0.03), (c, c - r_ring + s * 0.05)],
                s * 0.008,
                ink.danger,
                false,
            );
            name_plate(w, o, (c - s * 0.11, c + s * 0.22, s * 0.22, s * 0.06), ink);
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint));
        for (k, name) in ["N", "E", "S", "W"].iter().enumerate() {
            let (x, y) = polar(c, c, r_ring - s * 0.085, k as f32 / 4.0 - turn);
            el = el.child(label(x, y - s * 0.025, s * 0.08, s * 0.04, ink.text, *name));
        }
        el.child(label(
            c,
            c + s * 0.227,
            s * 0.2,
            s * 0.034,
            ink.text,
            format!("{:03.0}° {}", self.heading, pm::cardinal(self.heading)),
        ))
        .into_any_element()
    }
}
