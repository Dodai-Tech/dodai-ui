//! The parts of the [`Look::Race`] instruments, a motorsport data display: a dark plate with a
//! carbon twill and a cut corner, slanted segment bars that rise like a rev bar, slanted tags
//! and big leaning figures. Pure geometry returns points; the painters take every colour from an
//! [`Ink`].
//!
//! [`Look::Race`]: crate::look::Look::Race

use crate::shapes::Ink;
use dodai_gpui::paint::{fill, fill_batch, scale_alpha, stroke, stroke_batch};
use gpui::{div, px, FontWeight, Hsla, ParentElement, Pixels, Point, SharedString, Styled, Window};

/// How far the slanted shapes lean: the shift to the right per unit of height.
pub(crate) const LEAN: f32 = 0.3;

/// The corner cut off the top right of a plate, as a share of its side.
pub(crate) const CUT: f32 = 0.12;

/// A parallelogram over `(x, y, wd, h)` leaning right: its top edge starts `LEAN * h` in, its
/// bottom edge ends `LEAN * h` short. Pure.
pub(crate) fn skew(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    let k = (h * LEAN).min(wd / 2.0).max(0.0);
    vec![(x + k, y), (x + wd, y), (x + wd - k, y + h), (x, y + h)]
}

/// The outline of a plate `wd` by `h` with its top-right corner cut by `cut`. Pure.
pub(crate) fn plate_outline(wd: f32, h: f32, cut: f32) -> Vec<(f32, f32)> {
    let cut = cut.min(wd).min(h).max(0.0);
    vec![(0.0, 0.0), (wd - cut, 0.0), (wd, cut), (wd, h), (0.0, h)]
}

/// The lines of a twill on a plate `wd` by `h` with its top-right corner cut by `cut`: lines
/// rising to the right every `step`, each kept inside the plate. Pure.
pub(crate) fn twill_in((wd, h): (f32, f32), cut: f32, step: f32) -> Vec<Vec<(f32, f32)>> {
    let step = step.max(1.0);
    let n = ((wd + h) / step).ceil() as usize;
    (1..n)
        .filter_map(|i| {
            // The line x + y = c, clipped to the plate and to the cut (x - y <= wd - cut).
            let c = step * i as f32;
            let x0 = (c - h).max(0.0);
            let x1 = wd.min(c).min((wd - cut + c) / 2.0);
            (x1 - x0 > step * 0.2).then(|| vec![(x0, c - x0), (x1, c - x1)])
        })
        .collect()
}

/// The segments of a bar over `(x, y, wd, h)`: `n` slanted blocks with gaps. A rising bar grows
/// from under half its height at the left to the full height at the right, its blocks sharing a
/// baseline, like a rev bar. Pure.
pub(crate) fn segments(
    (x, y, wd, h): (f32, f32, f32, f32),
    n: usize,
    rising: bool,
) -> Vec<Vec<(f32, f32)>> {
    let n = n.max(1);
    let pitch = wd / n as f32;
    let gap = pitch * 0.22;
    (0..n)
        .map(|i| {
            let sh = if rising {
                h * (0.4 + 0.6 * (i as f32 + 1.0) / n as f32)
            } else {
                h
            };
            let x0 = x + pitch * i as f32;
            // The slant of a block follows the bar's height so the blocks stay parallel.
            let k = h * LEAN * 0.5 * sh / h;
            vec![
                (x0 + k, y + h - sh),
                (x0 + pitch - gap + k, y + h - sh),
                (x0 + pitch - gap, y + h),
                (x0, y + h),
            ]
        })
        .collect()
}

/// How many of `n` segments a bar at `f` lights: none at zero, all from one. Pure.
pub(crate) fn lit_count(f: f32, n: usize) -> usize {
    if !f.is_finite() {
        return 0;
    }
    ((f.clamp(0.0, 1.0) * n as f32).round() as usize).min(n)
}

/// Paint a race plate filling a square of `s`: the ground colour with a faint carbon twill, the
/// top-right corner cut, a frame, and three speed stripes in the primary colour at the top left.
pub(crate) fn plate(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) {
    plate_in(w, o, (s, s), s, ink);
}

/// Paint a race plate `wd` by `h` for a gauge of size `s`, see [`plate`].
pub(crate) fn plate_in(w: &mut Window, o: Point<Pixels>, (wd, h): (f32, f32), s: f32, ink: Ink) {
    let cut = wd.min(h) * CUT;
    let outline = plate_outline(wd, h, cut);
    fill(w, o, &outline, ink.ground);
    stroke_batch(
        w,
        o,
        &twill_in((wd, h), cut, s * 0.016),
        (s * 0.0025).max(0.5),
        scale_alpha(ink.text, 0.03),
        false,
    );
    stroke(
        w,
        o,
        &outline,
        (s * 0.003).max(0.6),
        scale_alpha(ink.text, 0.3),
        true,
    );
    // The edge of the cut in the primary colour, as a painted livery line.
    stroke(
        w,
        o,
        &[(wd - cut, 0.0), (wd, cut)],
        (s * 0.012).max(1.0),
        ink.primary,
        false,
    );
    stripes(
        w,
        o,
        (s * 0.06, s * 0.035),
        s * 0.022,
        s * 0.13,
        ink.primary,
    );
}

/// Paint three speed stripes at `(x, y)`, `h` high and `wd` long, fading out.
pub(crate) fn stripes(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y): (f32, f32),
    h: f32,
    wd: f32,
    col: Hsla,
) {
    for (i, a) in [1.0, 0.55, 0.25].into_iter().enumerate() {
        let dx = (wd + h) * i as f32 * 0.55;
        fill(
            w,
            o,
            &skew(x + dx, y, wd * (1.0 - 0.25 * i as f32), h),
            scale_alpha(col, a),
        );
    }
}

/// Paint a segment bar over `(x, y, wd, h)` lit to `f`: each lit block in `colour(at)` with `at`
/// its middle as a share of the bar, the dark blocks faint.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seg_bar(
    w: &mut Window,
    o: Point<Pixels>,
    rect: (f32, f32, f32, f32),
    f: f32,
    n: usize,
    rising: bool,
    colour: impl Fn(f32) -> Hsla,
    ink: Ink,
) {
    let segs = segments(rect, n, rising);
    let lit = lit_count(f, segs.len());
    let dark: Vec<Vec<(f32, f32)>> = segs[lit..].to_vec();
    fill_batch(w, o, &dark, scale_alpha(ink.text, 0.08));
    let count = segs.len() as f32;
    for (i, seg) in segs.iter().take(lit).enumerate() {
        fill(w, o, seg, colour((i as f32 + 0.5) / count));
    }
}

/// Paint a slanted tag over `(x, y, wd, h)` in `col`.
pub(crate) fn tag(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    col: Hsla,
) {
    fill(w, o, &skew(x, y, wd, h), col);
}

/// The text of a tag over `(x, y, wd, h)`: bold leaning capitals in `col`, centred.
pub(crate) fn tag_text(
    (x, y, wd, h): (f32, f32, f32, f32),
    col: Hsla,
    text: impl Into<SharedString>,
) -> gpui::Div {
    let text: SharedString = text.into();
    div()
        .absolute()
        .left(px(x))
        .top(px(y))
        .w(px(wd))
        .h(px(h))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(h * 0.6))
        .font_weight(FontWeight::BOLD)
        .italic()
        .text_color(col)
        .child(SharedString::from(text.to_uppercase()))
}

/// A big leaning figure of `size` in `col`.
pub(crate) fn figure(text: impl Into<SharedString>, size: f32, col: Hsla) -> gpui::Div {
    div()
        .text_size(px(size))
        .line_height(px(size * 1.05))
        .font_weight(FontWeight::BOLD)
        .italic()
        .text_color(col)
        .child(text.into())
}

/// The leaning left edge of a slanted shape over `(x, y, _, h)`, `wd` wide: a strip parallel to
/// the shape's left side. Pure.
pub(crate) fn lean_edge(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    let k = h * LEAN;
    vec![(x + k, y), (x + k + wd, y), (x + wd, y + h), (x, y + h)]
}

/// Paint a hazard band over `(x, y, wd, h)`: slanted blocks in `col` with equal gaps.
pub(crate) fn hazard(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    col: Hsla,
) {
    let n = ((wd / (h * 1.2)).floor() as usize).max(1);
    let blocks: Vec<Vec<(f32, f32)>> = (0..n)
        .step_by(2)
        .map(|i| lean_edge(x + wd * i as f32 / n as f32, y, wd / n as f32, h))
        .collect();
    fill_batch(w, o, &blocks, col);
}

/// The marks of a heading tape centred on `heading`: every multiple of `step` degrees within
/// `half` degrees either side, as `(offset from the centre in degrees, bearing 0..360)`. Pure.
pub(crate) fn tape_marks(heading: f32, half: f32, step: f32) -> Vec<(f32, u32)> {
    let valid = heading.is_finite() && step.is_finite() && half.is_finite();
    if !valid || step <= 0.0 || half <= 0.0 {
        return Vec::new();
    }
    let first = ((heading - half) / step).ceil() as i32;
    let last = ((heading + half) / step).floor() as i32;
    (first..=last)
        .map(|k| {
            let deg = k as f32 * step;
            (deg - heading, deg.rem_euclid(360.0).round() as u32 % 360)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_skew_leans_right_and_keeps_its_box() {
        let p = skew(0.0, 0.0, 10.0, 4.0);
        assert_eq!(p[0], (4.0 * LEAN, 0.0), "the top edge starts in");
        assert_eq!(p[3], (0.0, 4.0), "the bottom edge starts at the left");
        assert!(p.iter().all(|q| q.0 >= 0.0 && q.0 <= 10.0));
        let thin = skew(0.0, 0.0, 1.0, 100.0);
        assert!(
            thin.iter().all(|q| q.0 >= 0.0 && q.0 <= 1.0),
            "a thin tag still fits"
        );
    }

    #[test]
    fn the_twill_stays_inside_the_plate() {
        let (s, cut) = (100.0, 12.0);
        let lines = twill_in((s, s), cut, 5.0);
        assert!(lines.len() > 20);
        for l in &lines {
            for &(x, y) in l {
                assert!((-1e-3..=s + 1e-3).contains(&x) && (-1e-3..=s + 1e-3).contains(&y));
                assert!(x - y <= s - cut + 1e-3, "no line crosses the cut corner");
            }
        }
    }

    #[test]
    fn a_wide_plate_keeps_its_twill_inside() {
        let (wd, h, cut) = (200.0, 80.0, 9.0);
        for l in twill_in((wd, h), cut, 5.0) {
            for (x, y) in l {
                assert!((-1e-3..=wd + 1e-3).contains(&x) && (-1e-3..=h + 1e-3).contains(&y));
                assert!(x - y <= wd - cut + 1e-3);
            }
        }
    }

    #[test]
    fn a_rising_bar_grows_to_the_right_on_one_baseline() {
        let segs = segments((0.0, 0.0, 100.0, 20.0), 10, true);
        assert_eq!(segs.len(), 10);
        let top = |s: &Vec<(f32, f32)>| s[0].1;
        assert!(top(&segs[0]) > top(&segs[9]), "the first block is lower");
        assert_eq!(top(&segs[9]), 0.0, "the last block is full height");
        assert!(segs.iter().all(|s| s[3].1 == 20.0), "one baseline");
        let flat = segments((0.0, 0.0, 100.0, 20.0), 4, false);
        assert!(flat.iter().all(|s| s[0].1 == 0.0));
        assert_eq!(segments((0.0, 0.0, 10.0, 2.0), 0, false).len(), 1);
    }

    #[test]
    fn a_bar_lights_its_share() {
        assert_eq!(lit_count(0.0, 10), 0);
        assert_eq!(lit_count(0.5, 10), 5);
        assert_eq!(lit_count(1.5, 10), 10);
        assert_eq!(lit_count(f32::NAN, 10), 0);
    }

    #[test]
    fn a_tape_wraps_round_north() {
        let marks = tape_marks(350.0, 20.0, 10.0);
        let degs: Vec<u32> = marks.iter().map(|m| m.1).collect();
        assert_eq!(degs, vec![330, 340, 350, 0, 10]);
        assert_eq!(marks[2].0, 0.0, "the heading sits in the middle");
        assert!(marks.iter().all(|m| m.0.abs() <= 20.0));
        assert!(tape_marks(f32::NAN, 20.0, 10.0).is_empty());
        assert!(tape_marks(0.0, 20.0, 0.0).is_empty());
    }

    #[test]
    fn an_edge_follows_the_lean() {
        let e = lean_edge(0.0, 0.0, 2.0, 10.0);
        assert_eq!(e[0].0 - e[3].0, 10.0 * LEAN);
        assert_eq!(e[1].0 - e[0].0, 2.0);
    }

    #[test]
    fn a_plate_cuts_its_top_right_corner() {
        let p = plate_outline(10.0, 10.0, 2.0);
        assert!(p.contains(&(8.0, 0.0)) && p.contains(&(10.0, 2.0)));
        assert!(!p.contains(&(10.0, 0.0)));
    }
}
