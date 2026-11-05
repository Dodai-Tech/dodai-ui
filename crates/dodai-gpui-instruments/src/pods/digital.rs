//! The digital look of the pods: an LCD module with seven-segment figures, ghost segments,
//! segment bars, a dot matrix and a heading tape. Every colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::frames::{ghost, lcd, lit_segments, seg_bar, seg_text};
use crate::shapes::{chamfered, Ink};

/// The heading of a digital pod: small capitals in the accent near the top of the glass.
fn title(s: f32, ink: Ink, text: &str) -> gpui::Div {
    label(
        s / 2.0,
        s * 0.065,
        s * 0.7,
        s * 0.034,
        ink.primary,
        text.to_uppercase(),
    )
}

/// A whole bar of `n` segments in `col` up to `f`.
fn bar_of(n: usize, f: f32, col: Hsla) -> Vec<Option<Hsla>> {
    lit_segments(n, f, move |_| col)
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four framed cells, each with its corner, a seven-segment pressure and a segment bar.
    pub(super) fn digital(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let off = ghost(ink.primary, t.is_dark());
        let (bw, bh) = (s * 0.37, s * 0.3);
        let cells = [
            (c - s * 0.01 - bw, s * 0.13),
            (c + s * 0.01, s * 0.13),
            (c - s * 0.01 - bw, s * 0.13 + bh + s * 0.02),
            (c + s * 0.01, s * 0.13 + bh + s * 0.02),
        ];
        let reads: Vec<(Option<f32>, bool)> = self
            .bars
            .iter()
            .map(|b| (pm::pressure_frac(*b, self.low), pm::is_low(*b, self.low)))
            .collect();
        let reads_paint = reads.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            for (&(x, y), (f, low)) in cells.iter().zip(&reads_paint) {
                let col = if *low { ink.warn } else { ink.primary };
                let frame = chamfered(x, y, bw, bh, s * 0.02);
                stroke(w, o, &frame, s * 0.003, scale_alpha(col, 0.5), true);
                if *low {
                    fill(w, o, &frame, scale_alpha(ink.warn, 0.08));
                }
                seg_bar(
                    w,
                    o,
                    (x + bw * 0.1, y + bh * 0.78, bw * 0.8, bh * 0.08),
                    &bar_of(10, f.unwrap_or(0.0), col),
                    off,
                );
            }
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(title(s, ink, &self.heading));
        let names = ["FL", "FR", "RL", "RR"];
        for ((&(x, y), (_, low)), (name, bar)) in
            cells.iter().zip(&reads).zip(names.iter().zip(self.bars))
        {
            let col = if *low { ink.warn } else { ink.primary };
            el = el
                .child(
                    div()
                        .absolute()
                        .left(px(x + bw * 0.08))
                        .top(px(y + bh * 0.06))
                        .text_size(px(s * 0.026))
                        .text_color(ink.muted)
                        .child(if *low {
                            format!("{name} LOW")
                        } else {
                            (*name).to_string()
                        }),
                )
                .child(seg_text(
                    &pm::pressure_text(bar),
                    x + bw / 2.0,
                    y + bh * 0.28,
                    bh * 0.38,
                    col,
                ));
        }
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                s * 0.86,
                s * 0.8,
                s * 0.028,
                ink.muted,
                self.note.to_uppercase(),
            ));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// Every row as icon, seven-segment figure, unit and an optional segment bar.
    pub(super) fn digital(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let off = ghost(ink.primary, t.is_dark());
        let n = self.rows.len().max(1);
        let top = s * 0.13;
        let pitch = ((s * 0.76) / n as f32).min(s * 0.2);
        let (x0, x1) = (s * 0.09, s * 0.91);
        let rows: Vec<Option<(f32, Hsla)>> = self
            .rows
            .iter()
            .map(|r| r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            for (i, bar) in rows.iter().enumerate() {
                let y = top + pitch * i as f32;
                if i > 0 {
                    let mut x = x0;
                    while x < x1 {
                        stroke(
                            w,
                            o,
                            &[(x, y), (x + s * 0.008, y)],
                            s * 0.003,
                            scale_alpha(ink.primary, 0.3),
                            false,
                        );
                        x += s * 0.016;
                    }
                }
                if let Some((f, col)) = bar {
                    seg_bar(
                        w,
                        o,
                        (
                            x0 + s * 0.1,
                            y + pitch * 0.7,
                            x1 - x0 - s * 0.1,
                            pitch * 0.1,
                        ),
                        &bar_of(16, *f, *col),
                        off,
                    );
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(title(s, ink, &self.heading));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let has_bar = r.bar.is_some();
            let mid = y + pitch * if has_bar { 0.36 } else { 0.5 };
            let h = (pitch * 0.34).min(s * 0.075);
            let (fig, rest) = pm::split_figure(&r.value);
            let value: AnyElement = if fig.is_empty() {
                div()
                    .text_size(px(s * 0.045))
                    .text_color(ink.primary)
                    .child(r.value.to_uppercase())
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_end()
                    .gap(px(s * 0.012))
                    .child(
                        crate::segment::SegmentDisplay::new(fig.to_string())
                            .height(h)
                            .color(ink.primary),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.03))
                            .text_color(ink.muted)
                            .child(rest.to_uppercase()),
                    )
                    .into_any_element()
            };
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(mid - h * 0.6))
                    .w(px(x1 - x0))
                    .h(px(h * 1.2))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(r.icon.element(s * 0.055, ink.primary))
                    .child(value),
            );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// The symbol inverted in a block of the tone, the title in capitals and a striped bar.
    pub(super) fn digital(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let off = ghost(col, t.is_dark());
        let (bw, bh) = (s * 0.26, s * 0.2);
        let (bx, by) = (c - bw / 2.0, s * 0.12);
        let stripes: Vec<Option<Hsla>> = (0..14).map(|i| (i % 2 == 0).then_some(col)).collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            let block = chamfered(bx, by, bw, bh, s * 0.02);
            stroke(w, o, &block, s * 0.012, scale_alpha(col, 0.2), true);
            fill(w, o, &block, col);
            seg_bar(w, o, (s * 0.12, s * 0.8, s * 0.76, s * 0.03), &stripes, off);
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(c - bh * 0.35))
                    .top(px(by + bh * 0.15))
                    .child(self.icon.element(bh * 0.7, ink.face)),
            )
            .child(label(
                c,
                s * 0.37,
                s * 0.8,
                s * 0.05,
                col,
                self.title.to_uppercase(),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.12))
                    .top(px(s * 0.47))
                    .w(px(s * 0.76))
                    .text_center()
                    .text_size(px(s * 0.035))
                    .text_color(ink.muted)
                    .child(self.text.to_uppercase()),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

/// How bright the dot at grid offset `(i, j)` is when the force sits at `(gx, gy)` on a grid of
/// `half` dots each way: 1 on the nearest dot, fading over about two dots. Pure.
pub(crate) fn matrix_glow(i: i32, j: i32, gx: f32, gy: f32, half: i32) -> f32 {
    let h = half.max(1) as f32;
    let (dx, dy) = (i as f32 - gx * h, j as f32 + gy * h);
    (1.0 - (dx * dx + dy * dy).sqrt() / 2.0).clamp(0.0, 1.0)
}

impl GForcePod {
    /// A dot matrix with brighter axes, the force lit as a soft blob, the total in segments.
    pub(super) fn digital(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let cy = c - s * 0.06;
        let half = 6;
        let step = s * 0.05;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let off = ghost(ink.primary, t.is_dark());
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            for j in -half..=half {
                for i in -half..=half {
                    let (x, y) = (c + i as f32 * step, cy + j as f32 * step);
                    let k = matrix_glow(i, j, gx, gy, half);
                    let axis = i == 0 || j == 0;
                    let r = step * 0.28;
                    let sq = [
                        (x - r, y - r),
                        (x + r, y - r),
                        (x + r, y + r),
                        (x - r, y + r),
                    ];
                    let base = if axis {
                        scale_alpha(ink.primary, 0.35)
                    } else {
                        off
                    };
                    fill(w, o, &sq, base);
                    if k > 0.0 {
                        fill(w, o, &sq, scale_alpha(ink.primary, k));
                    }
                }
            }
        };
        let total = pm::g_text(pm::g_total(self.lateral, self.longitudinal));
        case(s)
            .child(layer(paint))
            .child(title(s, ink, "G-force"))
            .child(seg_text(&total, c, s * 0.76, s * 0.09, ink.primary))
            .child(label(
                s * 0.2,
                s * 0.79,
                s * 0.2,
                s * 0.026,
                ink.muted,
                format!("LAT {}", pm::g_text(self.lateral.abs())),
            ))
            .child(label(
                s * 0.8,
                s * 0.79,
                s * 0.2,
                s * 0.026,
                ink.muted,
                format!("LON {}", pm::g_text(self.longitudinal.abs())),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    /// A segment spectrum, the title in capitals, the time in segments and a progress bar.
    pub(super) fn digital(self, ink: Ink, dark: bool) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let f = pm::played(&self.time);
        let off = ghost(ink.primary, dark);
        let bands = pm::spectrum(&self.title, 14);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            let (x0, y0, wd, h) = (s * 0.12, s * 0.13, s * 0.76, s * 0.26);
            let cw = wd / bands.len() as f32;
            for (i, b) in bands.iter().enumerate() {
                let rows = 9;
                let lit = pm::segments_lit(*b, rows);
                for r in 0..rows {
                    let y = y0 + h - (r as f32 + 1.0) * h / rows as f32;
                    let cell = [
                        (x0 + cw * i as f32, y),
                        (x0 + cw * (i as f32 + 0.75), y),
                        (x0 + cw * (i as f32 + 0.75), y + h / rows as f32 * 0.7),
                        (x0 + cw * i as f32, y + h / rows as f32 * 0.7),
                    ];
                    let on = if r + 1 == rows { ink.warn } else { ink.primary };
                    fill(w, o, &cell, if r < lit { on } else { off });
                }
            }
            seg_bar(
                w,
                o,
                (s * 0.12, s * 0.8, s * 0.76, s * 0.03),
                &bar_of(24, f, ink.primary),
                off,
            );
        };
        case(s)
            .child(layer(paint))
            .child(label(
                c,
                s * 0.43,
                s * 0.76,
                s * 0.042,
                ink.primary,
                self.title.to_uppercase(),
            ))
            .child(label(
                c,
                s * 0.5,
                s * 0.76,
                s * 0.03,
                ink.muted,
                self.artist.to_uppercase(),
            ))
            .child(seg_text(
                pm::elapsed(&self.time),
                c,
                s * 0.58,
                s * 0.13,
                ink.primary,
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

/// The ticks of a heading tape every 5° within `half` degrees of `heading`: each as its offset
/// from the heading in degrees and its bearing in `0..360`. Pure.
pub(crate) fn tape_ticks(heading: f32, half: f32) -> Vec<(f32, i32)> {
    let first = ((heading - half) / 5.0).ceil() as i32 * 5;
    let last = ((heading + half) / 5.0).floor() as i32 * 5;
    (first..=last)
        .step_by(5)
        .map(|d| (d as f32 - heading, d.rem_euclid(360)))
        .collect()
}

impl CompassPod {
    /// The bearing in large segments with its point, a scrolling tape of ticks under a marker,
    /// and the height.
    pub(super) fn digital(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let heading = self.heading;
        let (tx0, tx1, ty) = (s * 0.1, s * 0.9, s * 0.62);
        // The tape shows 90° across its width.
        let per = (tx1 - tx0) / 90.0;
        let ticks: Vec<(f32, i32)> = tape_ticks(heading, 45.0)
            .into_iter()
            .map(|(off, deg)| (c + off * per, deg))
            .collect();
        let ticks_paint = ticks.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            stroke(
                w,
                o,
                &[(tx0, ty), (tx1, ty)],
                s * 0.003,
                scale_alpha(ink.primary, 0.5),
                false,
            );
            for &(x, deg) in &ticks_paint {
                let h = if deg % 30 == 0 {
                    s * 0.05
                } else if deg % 10 == 0 {
                    s * 0.032
                } else {
                    s * 0.018
                };
                stroke(w, o, &[(x, ty - h), (x, ty)], s * 0.005, ink.primary, false);
            }
            let k = s * 0.022;
            fill(
                w,
                o,
                &[
                    (c, ty + s * 0.008),
                    (c - k, ty + k * 1.8),
                    (c + k, ty + k * 1.8),
                ],
                ink.warn,
            );
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(title(s, ink, "Heading"))
            .child(seg_text(
                &format!("{:03.0}°", heading),
                c - s * 0.08,
                s * 0.16,
                s * 0.16,
                ink.primary,
            ))
            .child(label(
                c + s * 0.28,
                s * 0.2,
                s * 0.16,
                s * 0.08,
                ink.primary,
                pm::cardinal(heading),
            ));
        for &(x, deg) in ticks.iter().filter(|t| t.1 % 30 == 0) {
            let name = match deg {
                0 => "N".to_string(),
                90 => "E".to_string(),
                180 => "S".to_string(),
                270 => "W".to_string(),
                d => (d / 10).to_string(),
            };
            el = el.child(label(x, ty - s * 0.1, s * 0.08, s * 0.03, ink.muted, name));
        }
        el.child(label(
            c,
            s * 0.78,
            s * 0.6,
            s * 0.032,
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
    fn the_tape_wraps_round_north() {
        let t = tape_ticks(2.0, 10.0);
        assert_eq!(t.iter().map(|x| x.1).collect::<Vec<_>>(), [355, 0, 5, 10]);
        assert!((t[1].0 + 2.0).abs() < 1e-4, "north is 2° to the left");
        assert!(t.iter().all(|x| x.0.abs() <= 10.0));
    }

    #[test]
    fn the_matrix_lights_the_dot_under_the_force() {
        assert_eq!(matrix_glow(0, 0, 0.0, 0.0, 6), 1.0);
        assert_eq!(matrix_glow(6, 0, 1.0, 0.0, 6), 1.0);
        // Forward force (positive) sits above the middle, where rows count down.
        assert_eq!(matrix_glow(0, -6, 0.0, 1.0, 6), 1.0);
        assert_eq!(matrix_glow(-6, 0, 1.0, 0.0, 6), 0.0);
        assert!(matrix_glow(1, 0, 0.0, 0.0, 6) > 0.0);
    }
}
