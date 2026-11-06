//! The parts of the [`Look::Nixie`] instruments: stacked-digit glow tubes behind a wire mesh,
//! bargraph tubes, neon bulbs, a ring of glowing cathodes and a riveted chassis. Pure geometry
//! returns points; the painters take every colour from an [`Ink`].
//!
//! [`Look::Nixie`]: crate::look::Look::Nixie

use crate::draw::turn_arc;
use crate::shapes::{circle, Ink};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{
    canvas, div, px, Bounds, FontWeight, Hsla, ParentElement, Pixels, Point, Styled, Window,
};

/// The glow of a tube: the theme's warning colour pulled towards its danger colour, the orange
/// of neon.
pub(crate) fn glow(ink: Ink) -> Hsla {
    mix(ink.warn, ink.danger, 0.4)
}

/// A rectangle with corners rounded by `r`, as a polygon. Pure.
pub(crate) fn rounded(x: f32, y: f32, wd: f32, h: f32, r: f32) -> Vec<(f32, f32)> {
    let r = r.min(wd / 2.0).min(h / 2.0).max(0.0);
    let mut p = turn_arc(x + wd - r, y + r, r, 0.0, 0.25);
    p.extend(turn_arc(x + wd - r, y + h - r, r, 0.25, 0.5));
    p.extend(turn_arc(x + r, y + h - r, r, 0.5, 0.75));
    p.extend(turn_arc(x + r, y + r, r, 0.75, 1.0));
    p
}

/// The glass of a tube `(x, y, wd, h)`: a domed top on straight sides. Pure.
pub(crate) fn envelope(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    let r = wd / 2.0;
    let mut p = turn_arc(x + r, y + r, r, 0.75, 1.25);
    p.extend([(x + wd, y + h), (x, y + h)]);
    p
}

/// Paint the chassis of a square of `s`: a dark panel with rounded corners, four rivets and a
/// recessed window `inset` in from the edge.
pub(crate) fn chassis(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) {
    let panel = rounded(0.0, 0.0, s, s, s * 0.06);
    fill(w, o, &panel, ink.face);
    fill(w, o, &panel, scale_alpha(ink.text, 0.07));
    stroke(w, o, &panel, s * 0.004, ink.line, true);
    let k = s * 0.035;
    let window = rounded(k, k, s - 2.0 * k, s - 2.0 * k, s * 0.04);
    fill(w, o, &window, ink.face);
    stroke(w, o, &window, s * 0.003, scale_alpha(ink.text, 0.12), true);
    for (x, y) in [(k, k), (s - k, k), (k, s - k), (s - k, s - k)] {
        fill(w, o, &circle(x, y, s * 0.009), scale_alpha(ink.text, 0.3));
    }
}

/// Paint a soft orange glow round `(x, y)`, `rx` by `ry`: layered ellipses.
pub(crate) fn haze(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y): (f32, f32),
    (rx, ry): (f32, f32),
    col: Hsla,
) {
    for k in 1..=5 {
        let f = k as f32 / 5.0;
        let e: Vec<(f32, f32)> = circle(0.0, 0.0, 1.0)
            .into_iter()
            .map(|(a, b)| (x + a * rx * f, y + b * ry * f))
            .collect();
        fill(w, o, &e, scale_alpha(col, 0.07));
    }
}

/// How wide a cell of tube text is for `c` at tube height `h`: tubes for figures and letters,
/// narrow neon dots for `.`, `,` and `:`, a gap for a space. Pure.
pub(crate) fn cell_width(c: char, h: f32) -> f32 {
    match c {
        '.' | ',' | ':' => h * 0.16,
        ' ' => h * 0.2,
        _ => h * 0.56,
    }
}

/// Width of `text` laid out as tubes of height `h`, gaps included. Pure.
pub(crate) fn tubes_width(text: &str, h: f32) -> f32 {
    let n = text.chars().count();
    text.chars().map(|c| cell_width(c, h)).sum::<f32>() + h * 0.04 * n.saturating_sub(1) as f32
}

/// `text` as a row of glow tubes of height `h`: each figure glows in `col` over a faint stack of
/// unlit cathodes behind a wire mesh; dots and colons are neon dots between the tubes.
pub(crate) fn tubes(text: &str, h: f32, col: Hsla, ink: Ink) -> gpui::Div {
    let mut row = div().flex().items_end().gap(px(h * 0.04));
    for c in text.chars() {
        let wd = cell_width(c, h);
        let paint = move |w: &mut Window, o: Point<Pixels>| match c {
            '.' | ',' => {
                fill(w, o, &circle(wd / 2.0, h * 0.86, h * 0.04), col);
            }
            ':' => {
                for y in [h * 0.42, h * 0.7] {
                    fill(w, o, &circle(wd / 2.0, y, h * 0.04), col);
                }
            }
            ' ' => {}
            _ => {
                let glass = envelope(0.0, 0.0, wd, h * 0.92);
                fill(w, o, &glass, scale_alpha(ink.text, 0.05));
                haze(w, o, (wd / 2.0, h * 0.5), (wd * 0.5, h * 0.4), col);
                // The wire mesh in front of the cathodes.
                let mut x = wd * 0.12;
                while x < wd * 0.9 {
                    stroke(
                        w,
                        o,
                        &[(x, h * 0.16), (x, h * 0.86)],
                        (h * 0.006).max(0.5),
                        scale_alpha(ink.text, 0.08),
                        false,
                    );
                    x += wd * 0.13;
                }
                stroke(
                    w,
                    o,
                    &glass,
                    (h * 0.012).max(0.8),
                    scale_alpha(ink.text, 0.25),
                    true,
                );
                // A highlight down the glass and the dark socket at the foot.
                stroke(
                    w,
                    o,
                    &[(wd * 0.18, h * 0.2), (wd * 0.18, h * 0.6)],
                    (h * 0.015).max(0.8),
                    scale_alpha(ink.text, 0.18),
                    false,
                );
                fill(
                    w,
                    o,
                    &[(0.0, h * 0.92), (wd, h * 0.92), (wd, h), (0.0, h)],
                    ink.line,
                );
            }
        };
        let digit = !matches!(c, '.' | ',' | ':' | ' ');
        let mut cell = div()
            .relative()
            .w(px(wd))
            .h(px(h))
            .flex()
            .items_center()
            .justify_center()
            .child(
                canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
                )
                .absolute()
                .size_full(),
            );
        if digit {
            let fs = h * 0.62;
            cell = cell
                .child(
                    div()
                        .absolute()
                        .top(px(h * 0.12))
                        .w(px(wd))
                        .flex()
                        .justify_center()
                        .text_size(px(fs))
                        .font_weight(FontWeight::THIN)
                        .text_color(scale_alpha(ink.text, 0.07))
                        .child("8"),
                )
                .child(
                    div()
                        .absolute()
                        .top(px(h * 0.12))
                        .w(px(wd))
                        .flex()
                        .justify_center()
                        .text_size(px(fs))
                        .font_weight(FontWeight::LIGHT)
                        .text_color(col)
                        .child(c.to_string()),
                );
        }
        row = row.child(cell);
    }
    row
}

/// [`tubes`] placed with its centre on `cx` and its top at `y`.
pub(crate) fn tubes_at(text: &str, cx: f32, y: f32, h: f32, col: Hsla, ink: Ink) -> gpui::Div {
    div()
        .absolute()
        .left(px(cx - tubes_width(text, h) / 2.0))
        .top(px(y))
        .child(tubes(text, h, col, ink))
}

/// Paint a bargraph tube over `(x, y, wd, h)`: a glass capsule whose glowing column reaches `f`,
/// along the length (upwards when `vertical`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn bargraph(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    f: f32,
    vertical: bool,
    col: Hsla,
    ink: Ink,
) {
    let f = f.clamp(0.0, 1.0);
    let thick = if vertical { wd } else { h };
    let glass = rounded(x, y, wd, h, thick / 2.0);
    fill(w, o, &glass, scale_alpha(ink.text, 0.05));
    let k = thick * 0.28;
    let (cx0, cy0, cw, ch) = if vertical {
        let len = (h - 2.0 * k) * f;
        (x + k, y + h - k - len, wd - 2.0 * k, len)
    } else {
        (x + k, y + k, (wd - 2.0 * k) * f, h - 2.0 * k)
    };
    if f > 0.0 {
        let column = rounded(cx0, cy0, cw, ch, thick * 0.2);
        for g in 1..=3 {
            let e = thick * 0.12 * g as f32;
            fill(
                w,
                o,
                &rounded(
                    cx0 - e,
                    cy0 - e,
                    cw + 2.0 * e,
                    ch + 2.0 * e,
                    thick * 0.3 + e,
                ),
                scale_alpha(col, 0.08),
            );
        }
        fill(w, o, &column, col);
        let core = if vertical {
            rounded(cx0 + cw * 0.35, cy0, cw * 0.3, ch, cw * 0.15)
        } else {
            rounded(cx0, cy0 + ch * 0.35, cw, ch * 0.3, ch * 0.15)
        };
        fill(w, o, &core, scale_alpha(mix(col, ink.text, 0.6), 0.8));
    }
    stroke(
        w,
        o,
        &glass,
        (thick * 0.06).max(0.8),
        scale_alpha(ink.text, 0.3),
        true,
    );
}

/// Paint a neon bulb of radius `r` at `(cx, cy)`: a ring, a domed glass and, when `lit`, a glowing
/// fill in `col` with a bright core.
#[allow(clippy::too_many_arguments)]
pub(crate) fn bulb(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    col: Hsla,
    lit: bool,
    ink: Ink,
) {
    if lit {
        for k in 1..=4 {
            fill(
                w,
                o,
                &circle(cx, cy, r * (1.0 + 0.15 * k as f32)),
                scale_alpha(col, 0.07),
            );
        }
    }
    fill(w, o, &circle(cx, cy, r), ink.line);
    fill(
        w,
        o,
        &circle(cx, cy, r * 0.82),
        if lit { col } else { scale_alpha(col, 0.14) },
    );
    if lit {
        fill(
            w,
            o,
            &circle(cx, cy, r * 0.42),
            scale_alpha(mix(col, ink.text, 0.65), 0.5),
        );
    }
    let mut glint = turn_arc(cx, cy, r * 0.66, 0.82, 0.97);
    glint.extend(turn_arc(cx, cy, r * 0.5, 0.97, 0.82));
    fill(w, o, &glint, scale_alpha(ink.text, 0.3));
}

/// The cathode a ring of `n` lights for `f` of a turn: the nearest one, wrapping round. Pure.
pub(crate) fn cathode(f: f32, n: usize) -> usize {
    let n = n.max(1);
    let i = (f.rem_euclid(1.0) * n as f32).round() as usize;
    i % n
}

/// Paint a ring of `n` cathodes of radius `r` round `(cx, cy)`, the one for `f` of a turn glowing
/// in `col` with a fading tail on either side.
#[allow(clippy::too_many_arguments)]
pub(crate) fn dekatron(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    n: usize,
    f: f32,
    col: Hsla,
    ink: Ink,
) {
    let n = n.max(1);
    fill(w, o, &circle(cx, cy, r * 1.12), scale_alpha(ink.text, 0.04));
    stroke(
        w,
        o,
        &circle(cx, cy, r * 1.12),
        r * 0.012,
        scale_alpha(ink.text, 0.25),
        true,
    );
    let lit = cathode(f, n);
    for i in 0..n {
        let (x, y) = crate::draw::polar(cx, cy, r, i as f32 / n as f32);
        let d = (i as i32 - lit as i32).rem_euclid(n as i32);
        let near = d.min(n as i32 - d);
        fill(w, o, &circle(x, y, r * 0.035), scale_alpha(ink.text, 0.22));
        if near <= 2 {
            let k = 1.0 - near as f32 * 0.4;
            haze(w, o, (x, y), (r * 0.12 * k, r * 0.12 * k), col);
            fill(w, o, &circle(x, y, r * 0.04 * k), scale_alpha(col, k));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dots_and_colons_sit_between_narrower_than_tubes() {
        let h = 100.0;
        assert!(cell_width('.', h) < cell_width('8', h));
        assert_eq!(cell_width(':', h), cell_width('.', h));
        assert_eq!(tubes_width("", h), 0.0);
        assert!((tubes_width("12", h) - (2.0 * cell_width('1', h) + h * 0.04)).abs() < 1e-4);
    }

    #[test]
    fn the_cathode_ring_wraps_round() {
        assert_eq!(cathode(0.0, 30), 0);
        assert_eq!(cathode(0.5, 30), 15);
        assert_eq!(
            cathode(0.999, 30),
            0,
            "just short of a turn is the first cathode"
        );
        assert_eq!(cathode(-0.25, 4), 3);
        assert_eq!(cathode(0.3, 0), 0);
    }

    #[test]
    fn a_rounded_rect_stays_in_its_box() {
        let p = rounded(10.0, 20.0, 30.0, 40.0, 100.0);
        assert!(p
            .iter()
            .all(|(x, y)| (9.99..=40.01).contains(x) && (19.99..=60.01).contains(y)));
        let e = envelope(0.0, 0.0, 10.0, 30.0);
        assert!(e.iter().all(|p| p.1 >= -1e-4 && p.1 <= 30.0));
    }
}
