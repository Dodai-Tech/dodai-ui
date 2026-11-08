//! The vintage, segment-LCD and halo looks of [`MenuRing`]: a bakelite rotary selector on a
//! plate, an LCD list with the row inverted, and a curved list of glass with a scroll arc of
//! light. Colours from the theme.

use super::*;
use crate::gpui::frames::{aged, ghost, glow_arc, halo, lcd, name_plate, pill, plate};
use crate::gpui::shapes::{chamfered, chrono_glass, Ink};
use dodai_gpui::paint::{fill_circle, stroke_circle};

/// Turn of position `i` of `n` on the vintage selector, fanned over the top of the knob. Pure.
pub(crate) fn dial_turn(i: usize, n: usize) -> f32 {
    let n = n.max(1);
    if n == 1 {
        return 0.0;
    }
    let spread = (0.36 / (n - 1) as f32).min(0.14);
    (i as f32 - (n - 1) as f32 / 2.0) * spread
}

/// A canvas filling its parent.
fn layer(paint: impl Fn(&mut Window, gpui::Point<Pixels>) + 'static) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

impl MenuRing {
    /// A rotary selector: positions engraved round the top of a fluted knob, its pointer on the
    /// selected row, the heading on a plate.
    pub(super) fn vintage(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (kx, ky, kr) = (c, c + s * 0.11, s * 0.1);
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let (pressed, chosen) = (self.pressed, self.chosen);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = plate(w, o, s, ink);
            for i in 0..n {
                let a = dial_turn(i, n);
                let on = i == sel;
                stroke(
                    w,
                    o,
                    &[polar(kx, ky, kr * 1.25, a), polar(kx, ky, kr * 1.6, a)],
                    if on { s * 0.008 } else { s * 0.004 },
                    if chosen == Some(i) {
                        ink.info
                    } else {
                        ink.text
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
                    kr * 1.25,
                    dial_turn(0, n) - 0.02,
                    dial_turn(n - 1, n) + 0.02,
                ),
                s * 0.003,
                ink.text,
                false,
            );
            // The knob: a shadow, fluted rim, a dome and the pointer bar.
            fill_circle(
                w,
                o,
                (kx + s * 0.006, ky + s * 0.01),
                kr,
                scale_alpha(ink.text, 0.15),
            );
            fill_circle(w, o, (kx, ky), kr, ink.text);
            for i in 0..24 {
                let a = i as f32 / 24.0;
                stroke(
                    w,
                    o,
                    &[polar(kx, ky, kr * 0.82, a), polar(kx, ky, kr, a)],
                    s * 0.006,
                    aged(ink),
                    false,
                );
            }
            fill_circle(w, o, (kx, ky), kr * 0.78, ink.text);
            fill_circle(
                w,
                o,
                (kx - kr * 0.2, ky - kr * 0.25),
                kr * 0.3,
                scale_alpha(aged(ink), 0.12),
            );
            let a = dial_turn(sel, n);
            stroke(
                w,
                o,
                &[polar(kx, ky, -kr * 0.3, a), polar(kx, ky, kr * 0.95, a)],
                s * 0.014,
                if pressed { ink.primary } else { aged(ink) },
                false,
            );
            name_plate(w, o, (c - s * 0.13, c + s * 0.25, s * 0.26, s * 0.06), ink);
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        for (i, r) in self.rows.iter().enumerate() {
            let (x, y) = polar(kx, ky, kr * 2.25, dial_turn(i, n));
            let on = i == sel;
            el = el.child(
                text_box(x, y - s * 0.027, s * 0.26)
                    .text_size(px(if on { s * 0.05 } else { s * 0.044 }))
                    .text_color(if self.chosen == Some(i) {
                        ink.info
                    } else if on {
                        ink.text
                    } else {
                        ink.muted
                    })
                    .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .child(r.clone()),
            );
        }
        let heading = if self.heading.is_empty() {
            SharedString::from(format!("{} / {}", sel + 1, self.rows.len()))
        } else {
            self.heading.clone()
        };
        el.child(
            text_box(c, c + s * 0.263, s * 0.24)
                .text_size(px(s * 0.03))
                .text_color(ink.text)
                .child(heading.to_uppercase()),
        )
        .into_any_element()
    }

    /// An LCD list: the selected row inverted, the chosen one ticked, and a segment scroll bar.
    pub(super) fn digital(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let off = ghost(ink.primary, t.is_dark());
        let top = if self.heading.is_empty() {
            s * 0.12
        } else {
            s * 0.2
        };
        let pitch = ((s * 0.84 - top) / n as f32).min(s * 0.12);
        let (x0, x1) = (s * 0.1, s * 0.84);
        let pressed = self.pressed;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, s), s, ink);
            let y = top + pitch * sel as f32;
            let bar = chamfered(x0, y + pitch * 0.08, x1 - x0, pitch * 0.84, pitch * 0.15);
            if pressed {
                stroke(w, o, &bar, s * 0.014, scale_alpha(ink.primary, 0.3), true);
            }
            fill(w, o, &bar, ink.primary);
            let (sx, h) = (s * 0.87, (pitch * n as f32) / n as f32);
            for i in 0..n {
                let cell = [
                    (sx, top + h * i as f32 + h * 0.1),
                    (sx + s * 0.025, top + h * i as f32 + h * 0.1),
                    (sx + s * 0.025, top + h * (i as f32 + 0.9)),
                    (sx, top + h * (i as f32 + 0.9)),
                ];
                fill(w, o, &cell, if i == sel { ink.primary } else { off });
            }
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(
                text_box(s / 2.0, s * 0.08, s * 0.7)
                    .text_size(px(s * 0.034))
                    .text_color(ink.primary)
                    .child(self.heading.to_uppercase()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let on = i == sel;
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0 + s * 0.03))
                    .top(px(y))
                    .w(px(x1 - x0 - s * 0.06))
                    .h(px(pitch))
                    .flex()
                    .items_center()
                    .justify_between()
                    .text_size(px(s * 0.05))
                    .text_color(if on { ink.face } else { ink.primary })
                    .child(r.to_uppercase())
                    .when(self.chosen == Some(i), |d| {
                        d.child(
                            div()
                                .text_color(if on { ink.face } else { ink.info })
                                .child("✓"),
                        )
                    }),
            );
        }
        el.into_any_element()
    }

    /// A curved list of glass: the selected row in a lit pill, the others bending away and
    /// fading, a scroll arc of light on the rim.
    pub(super) fn futuristic(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let pitch = s * 0.105;
        let (pw, ph) = (s * 0.5, s * 0.11);
        let pressed = self.pressed;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_in = halo(w, o, s, ink);
            let p = pill(c - pw / 2.0, c - ph / 2.0, pw, ph);
            fill(
                w,
                o,
                &p,
                scale_alpha(ink.primary, if pressed { 0.35 } else { 0.14 }),
            );
            stroke(w, o, &p, s * 0.006, scale_alpha(ink.primary, 0.25), true);
            stroke(w, o, &p, s * 0.0025, ink.primary, true);
            // The scroll arc: the track on the right, lit for the selected row's share.
            let (a0, a1) = (0.17_f32, 0.33_f32);
            let seg = (a1 - a0) / n as f32;
            stroke(
                w,
                o,
                &turn_arc(c, c, r_in - s * 0.02, a0, a1),
                s * 0.006,
                scale_alpha(ink.text, 0.1),
                false,
            );
            glow_arc(
                w,
                o,
                (c, c),
                r_in - s * 0.02,
                (a0 + seg * sel as f32, a0 + seg * (sel as f32 + 1.0)),
                s * 0.008,
                (ink.info, ink.primary),
            );
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(
                text_box(c, s * 0.16, s * 0.6)
                    .text_size(px(s * 0.03))
                    .text_color(ink.primary)
                    .child(self.heading.to_uppercase()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let d = i as f32 - sel as f32;
            if d.abs() > 2.0 {
                continue;
            }
            let k = 1.0 - d.abs() * 0.25;
            let size = s * 0.052 * k;
            // Rows bend away to the right, like a list on a drum.
            let x = c + d * d * s * 0.035;
            let y = c + d * pitch;
            let col = if self.chosen == Some(i) {
                ink.info
            } else {
                ink.text
            };
            el = el.child(
                text_box(x, y - size * 0.5, pw)
                    .text_size(px(size))
                    .line_height(px(size))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(if d == 0.0 {
                        col
                    } else {
                        scale_alpha(col, 0.25 + 0.35 * k)
                    })
                    .child(r.clone()),
            );
        }
        el.into_any_element()
    }

    /// A line menu: a hairline ring round a plain list, the selected row in the text colour over
    /// a hairline in the primary colour, the others fading away from it.
    pub(super) fn line(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let pitch = s * 0.11;
        let pressed = self.pressed;
        let hair = (s * 0.003).max(0.6);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            stroke_circle(w, o, (c, c), s * 0.47, hair, scale_alpha(ink.text, 0.2));
            let y = c + s * 0.04;
            stroke(
                w,
                o,
                &[(c - s * 0.2, y), (c + s * 0.2, y)],
                if pressed { hair * 3.0 } else { hair * 1.5 },
                ink.primary,
                false,
            );
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(
                text_box(c, s * 0.17, s * 0.6)
                    .text_size(px(s * 0.03))
                    .text_color(ink.muted)
                    .child(self.heading.to_uppercase()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let d = i as f32 - sel as f32;
            if d.abs() > 2.0 {
                continue;
            }
            let size = s * if d == 0.0 { 0.058 } else { 0.044 };
            let col = match (d == 0.0, self.chosen == Some(i)) {
                (_, true) => ink.primary,
                (true, false) => ink.text,
                (false, false) => scale_alpha(ink.muted, 1.0 - d.abs() * 0.3),
            };
            el = el.child(
                text_box(c, c + d * pitch - size * 0.55, s * 0.7)
                    .text_size(px(size))
                    .line_height(px(size))
                    .font_weight(FontWeight::LIGHT)
                    .text_color(col)
                    .child(r.clone()),
            );
        }
        el.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_fan_symmetrically_over_the_top() {
        assert_eq!(dial_turn(0, 1), 0.0);
        assert!((dial_turn(0, 3) + dial_turn(2, 3)).abs() < 1e-6);
        assert!(dial_turn(0, 5) < dial_turn(1, 5));
        assert!(
            dial_turn(0, 20) >= -0.18 - 1e-6,
            "many rows stay over the top"
        );
        assert_eq!(dial_turn(0, 0), 0.0);
    }
}
