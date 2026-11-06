//! The chronograph and heads-up looks of [`MenuRing`] (a rotary selector in a watch case and a
//! drum of rows on a heads-up display) and the dispatch to the others. Colours from the theme.

use super::*;
use crate::shapes::{chamfered, chrono_case, chrono_glass, circle, hud_ring, Ink};

/// Turn of row `i` of `n` on the selector, fanned around three o'clock.
fn row_turn(i: usize, n: usize) -> f32 {
    let spread = 0.075;
    0.25 + (i as f32 - (n.max(1) - 1) as f32 / 2.0) * spread
}

impl MenuRing {
    pub(super) fn detailed(self, t: &Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Sport | Look::Chronograph => self.chrono(t),
        }
    }

    /// A rotary switch: a knurled knob with a pointer, a mark per row and the rows fanned out.
    fn chrono(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (kx, ky) = (c - s * 0.19, c + s * 0.02);
        let kr = s * 0.12;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let pressed = self.pressed;
        let chosen = self.chosen;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for i in 0..n {
                let a = row_turn(i, n);
                let on = i == sel;
                stroke(
                    w,
                    o,
                    &[
                        polar(kx, ky, kr * 1.35, a),
                        polar(kx, ky, kr * if on { 1.75 } else { 1.6 }, a),
                    ],
                    if on { s * 0.01 } else { s * 0.005 },
                    if on {
                        ink.primary
                    } else if chosen == Some(i) {
                        ink.info
                    } else {
                        ink.muted
                    },
                    false,
                );
            }
            stroke(
                w,
                o,
                &turn_arc(
                    kx,
                    ky,
                    kr * 1.3,
                    row_turn(0, n) - 0.02,
                    row_turn(n - 1, n) + 0.02,
                ),
                s * 0.003,
                scale_alpha(ink.text, 0.4),
                false,
            );
            // The knob: knurled rim, cap and pointer.
            fill(
                w,
                o,
                &circle(kx, ky, kr * 1.12),
                scale_alpha(ink.text, 0.12),
            );
            for k in 0..40 {
                let a = k as f32 / 40.0;
                stroke(
                    w,
                    o,
                    &[polar(kx, ky, kr, a), polar(kx, ky, kr * 1.12, a)],
                    s * 0.004,
                    scale_alpha(ink.text, 0.35),
                    false,
                );
            }
            fill(w, o, &circle(kx, ky, kr), ink.face);
            stroke(
                w,
                o,
                &circle(kx, ky, kr),
                if pressed { s * 0.012 } else { s * 0.005 },
                if pressed { ink.primary } else { ink.line },
                true,
            );
            let a = row_turn(sel, n);
            stroke(
                w,
                o,
                &[polar(kx, ky, kr * 0.25, a), polar(kx, ky, kr * 0.9, a)],
                s * 0.014,
                ink.primary,
                false,
            );
            fill(w, o, &circle(kx, ky, kr * 0.12), ink.text);
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(
            canvas(
                |_, _, _| (),
                move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
            )
            .absolute()
            .size_full(),
        );
        if !self.heading.is_empty() {
            el = el.child(
                text_box(c, s * 0.16, s * 0.6)
                    .text_size(px(s * 0.045))
                    .text_color(ink.info)
                    .child(self.heading.clone()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let (x, y) = polar(kx, ky, kr * 1.95, row_turn(i, n));
            let on = i == sel;
            el = el.child(
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y - s * 0.035))
                    .whitespace_nowrap()
                    .text_size(px(if on { s * 0.058 } else { s * 0.048 }))
                    .text_color(if self.chosen == Some(i) {
                        ink.info
                    } else if on {
                        ink.text
                    } else {
                        ink.muted
                    })
                    .when(on, |d| d.font_weight(FontWeight::MEDIUM))
                    .child(r.clone()),
            );
        }
        el.into_any_element()
    }

    /// A drum: the selected row large in a chamfered frame, the others shrinking and fading above
    /// and below, a scroll track and the position.
    fn hud(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let (fw, fh) = (s * 0.5, s * 0.13);
        let pitch = s * 0.1;
        let pressed = self.pressed;
        let (tx, t0, t1) = (c + s * 0.32, c - s * 0.2, c + s * 0.2);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            let frame = chamfered(c - fw / 2.0, c - fh / 2.0, fw, fh, fh * 0.3);
            fill(
                w,
                o,
                &frame,
                scale_alpha(ink.primary, if pressed { 0.35 } else { 0.12 }),
            );
            stroke(w, o, &frame, s * 0.006, ink.primary, true);
            for sx in [-1.0_f32, 1.0] {
                let x = c + sx * (fw / 2.0 + s * 0.02);
                let k = s * 0.022;
                fill(
                    w,
                    o,
                    &[(x, c), (x + sx * k, c - k), (x + sx * k, c + k)],
                    ink.primary,
                );
            }
            // Scroll track with a thumb for the selected row.
            stroke(
                w,
                o,
                &[(tx, t0), (tx, t1)],
                s * 0.006,
                scale_alpha(ink.text, 0.2),
                false,
            );
            let len = (t1 - t0) / n as f32;
            let y = t0 + len * sel as f32;
            stroke(
                w,
                o,
                &[(tx, y + s * 0.004), (tx, y + len - s * 0.004)],
                s * 0.012,
                ink.primary,
                false,
            );
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(
            canvas(
                |_, _, _| (),
                move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
            )
            .absolute()
            .size_full(),
        );
        if !self.heading.is_empty() {
            el = el.child(
                text_box(c, s * 0.18, s * 0.6)
                    .text_size(px(s * 0.038))
                    .text_color(ink.primary)
                    .child(self.heading.to_uppercase()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let d = i as f32 - sel as f32;
            if d.abs() > 2.0 {
                continue;
            }
            let k = 1.0 - d.abs() * 0.22;
            let y = c + d * pitch + d.signum() * s * 0.025;
            let col = if self.chosen == Some(i) {
                ink.info
            } else {
                ink.text
            };
            let size = s * 0.058 * k;
            el = el.child(
                text_box(c, y - size * 0.5, fw)
                    .text_size(px(size))
                    .line_height(px(size))
                    .text_color(if d == 0.0 {
                        col
                    } else {
                        scale_alpha(col, 0.35 + 0.3 * k)
                    })
                    .when(d == 0.0, |e| e.font_weight(FontWeight::MEDIUM))
                    .child(r.to_uppercase()),
            );
        }
        el.child(
            text_box(c, c + s * 0.3, s * 0.3)
                .text_size(px(s * 0.032))
                .text_color(ink.muted)
                .child(format!("{} / {}", sel + 1, self.rows.len())),
        )
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_fan_out_evenly_around_three_o_clock() {
        assert_eq!(row_turn(0, 1), 0.25);
        assert!(
            (row_turn(0, 3) + row_turn(2, 3) - 0.5).abs() < 1e-6,
            "symmetric"
        );
        assert!(row_turn(0, 3) < row_turn(1, 3));
        assert_eq!(row_turn(0, 0), 0.25, "no rows is not a panic");
    }
}
