//! The glass-cockpit and race looks of [`MenuRing`]: a display page whose rows sit beside the
//! soft keys, the selected one boxed in cyan; and a race plate of slanted tabs, the selected one
//! filled in the primary colour. Colours from the theme.

use super::*;
use crate::mfd::{page, rect};
use crate::race::{plate, skew, tag, tag_text};
use crate::shapes::Ink;

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

    /// A race plate: the heading on a slanted tag, each row a slanted tab, the selected one
    /// filled in the primary colour (pushed in while pressed) and the chosen one with a green
    /// edge.
    pub(super) fn race(self, t: &Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let n = self.rows.len().max(1);
        let sel = self.selected.min(n - 1);
        let has_heading = !self.heading.is_empty();
        let (top, pitch) = rows_layout(s, n, has_heading);
        let head = (s * 0.08, s * 0.1, s * 0.56, s * 0.075);
        let pressed = self.pressed;
        let chosen = self.chosen;
        let (x0, wd) = (s * 0.1, s * 0.8);
        let tab = move |i: usize| {
            let inset = if i == sel && pressed { s * 0.012 } else { 0.0 };
            (
                x0 + inset,
                top + pitch * i as f32 + pitch * 0.1 + inset,
                wd - 2.0 * inset,
                pitch * 0.8 - 2.0 * inset,
            )
        };
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            plate(w, o, s, ink);
            if has_heading {
                tag(w, o, head, ink.primary);
            }
            for i in 0..n {
                let (x, y, tw, th) = tab(i);
                let p = skew(x, y, tw, th);
                if i == sel {
                    fill(w, o, &p, ink.primary);
                } else {
                    fill(w, o, &p, scale_alpha(ink.text, 0.05));
                    stroke(w, o, &p, s * 0.004, ink.line, true);
                }
                if chosen == Some(i) {
                    fill(w, o, &crate::race::lean_edge(x, y, s * 0.025, th), ink.ok);
                }
            }
        };
        let mut el = div().relative().w(px(s)).h(px(s)).child(layer(paint));
        if has_heading {
            el = el.child(tag_text(head, ink.on_primary, &self.heading));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let (x, y, tw, th) = tab(i);
            let col = if i == sel { ink.on_primary } else { ink.text };
            el = el.child(
                div()
                    .absolute()
                    .left(px(x + th * 0.5))
                    .top(px(y))
                    .w(px(tw - th * 0.8))
                    .h(px(th))
                    .flex()
                    .items_center()
                    .text_size(px((th * 0.5).min(s * 0.05)))
                    .font_weight(FontWeight::BOLD)
                    .italic()
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
