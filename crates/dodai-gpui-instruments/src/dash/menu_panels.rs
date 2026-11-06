//! The glass-cockpit and nixie looks of [`MenuRing`]: a display page whose rows sit beside the
//! soft keys, the selected one boxed in cyan; and a panel of neon bulbs beside engraved rows,
//! the selected bulb lit. Colours from the theme.

use super::*;
use crate::mfd::{page, rect};
use crate::shapes::Ink;
use crate::tubes::{bulb, chassis, glow};

/// A canvas filling its parent.
fn layer(paint: impl Fn(&mut Window, gpui::Point<Pixels>) + 'static) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

/// Top and pitch of `n` rows on a page of `s`, below a heading if there is one. Pure.
pub(crate) fn rows_layout(s: f32, n: usize, heading: bool) -> (f32, f32) {
    let top = if heading { s * 0.22 } else { s * 0.12 };
    let pitch = ((s * 0.86 - top) / n.max(1) as f32).min(s * 0.15);
    (top, pitch)
}

impl MenuRing {
    /// A display page: the heading over a rule, each row beside a soft key with a caret, the
    /// selected row boxed in cyan (filled while pressed) and the chosen one marked `ON`.
    pub(super) fn cockpit(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let (top, pitch) = rows_layout(s, n, !self.heading.is_empty());
        let has_heading = !self.heading.is_empty();
        let (x0, x1) = (s * 0.1, s * 0.86);
        let pressed = self.pressed;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            page(w, o, s, ink);
            if has_heading {
                stroke(
                    w,
                    o,
                    &[(x0, s * 0.18), (x1, s * 0.18)],
                    s * 0.003,
                    scale_alpha(ink.text, 0.4),
                    false,
                );
            }
            for i in 0..n {
                let y = top + pitch * (i as f32 + 0.5);
                // The caret from the soft key on the bezel.
                fill(
                    w,
                    o,
                    &[
                        (s * 0.052, y - s * 0.016),
                        (s * 0.072, y),
                        (s * 0.052, y + s * 0.016),
                    ],
                    scale_alpha(ink.text, if i == sel { 1.0 } else { 0.5 }),
                );
            }
            let y = top + pitch * sel as f32;
            let frame = rect(x0, y + pitch * 0.12, x1 - x0, pitch * 0.76);
            if pressed {
                fill(w, o, &frame, scale_alpha(ink.info, 0.25));
            }
            stroke(w, o, &frame, s * 0.006, ink.info, true);
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if has_heading {
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(s * 0.1))
                    .text_size(px(s * 0.042))
                    .text_color(ink.text)
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
                    .text_size(px((pitch * 0.4).min(s * 0.05)))
                    .text_color(if on { ink.info } else { ink.text })
                    .child(r.to_uppercase())
                    .when(self.chosen == Some(i), |d| {
                        d.child(div().text_color(ink.ok).child("ON"))
                    }),
            );
        }
        el.into_any_element()
    }

    /// A panel of neon bulbs: one per row beside its engraved name, the selected bulb lit, the
    /// chosen row in the glow colour, the heading on a strip.
    pub(super) fn nixie(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let has_heading = !self.heading.is_empty();
        let (top, pitch) = rows_layout(s, n, has_heading);
        let neon = glow(ink);
        let pressed = self.pressed;
        let bx = s * 0.17;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            for i in 0..n {
                let y = top + pitch * (i as f32 + 0.5);
                let r = (pitch * 0.26).min(s * 0.04);
                let lit = i == sel;
                bulb(
                    w,
                    o,
                    (bx, y),
                    r * if lit && pressed { 1.15 } else { 1.0 },
                    neon,
                    lit,
                    ink,
                );
            }
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if has_heading {
            el = el.child(
                text_box(s / 2.0, s * 0.1, s * 0.7)
                    .text_size(px(s * 0.036))
                    .text_color(ink.muted)
                    .child(self.heading.to_uppercase()),
            );
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let col = if self.chosen == Some(i) {
                neon
            } else if i == sel {
                ink.text
            } else {
                ink.muted
            };
            el = el.child(
                div()
                    .absolute()
                    .left(px(bx + s * 0.08))
                    .top(px(y))
                    .h(px(pitch))
                    .flex()
                    .items_center()
                    .text_size(px((pitch * 0.38).min(s * 0.048)))
                    .text_color(col)
                    .child(r.to_uppercase()),
            );
        }
        el.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_fit_under_the_heading() {
        let (top, pitch) = rows_layout(100.0, 4, true);
        assert!(top > rows_layout(100.0, 4, false).0);
        assert!(top + pitch * 4.0 <= 86.0 + 1e-4);
        let (_, one) = rows_layout(100.0, 1, false);
        assert!(one <= 15.0 + 1e-4, "a single row is not stretched");
        assert!(rows_layout(100.0, 0, false).1 > 0.0);
    }
}
