//! The parts of the [`Look::Cockpit`] instruments: a display page with soft keys on its bezel,
//! engine-indication arcs with a filled sweep, boxed readouts with a rolling-drum window, bugs and a heading arc. Pure
//! geometry returns points; the painters take every colour from an [`Ink`].
//!
//! [`Look::Cockpit`]: crate::look::Look::Cockpit

use crate::draw::{polar, turn_arc};
use crate::shapes::{rounded, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use gpui::{Hsla, Pixels, Point, Window};

/// The rectangle `(x, y, wd, h)` as a polygon. Pure.
pub(crate) fn rect(x: f32, y: f32, wd: f32, h: f32) -> Vec<(f32, f32)> {
    vec![(x, y), (x + wd, y), (x + wd, y + h), (x, y + h)]
}

/// The bezel's width round the glass, as a fraction of the page.
pub(crate) const BEZEL: f32 = 0.055;

/// The heights of the soft keys down each side, as fractions of the page.
pub(crate) const KEYS: [f32; 5] = [0.2, 0.35, 0.5, 0.65, 0.8];

/// The page size from which the bezel shows its keys, sensor and rocker.
const DETAIL_FROM: f32 = 150.0;

/// Paint a display page filling a square of `s`: a rounded charcoal bezel shaded lighter at the
/// top, a light sensor over the glass, a brightness rocker under it and a row of raised soft keys
/// on each side, and the dark glass glowing towards the middle with a soft sheen across its top
/// left.
/// Returns the glass rectangle.
pub(crate) fn page(w: &mut Window, o: Point<Pixels>, s: f32, ink: Ink) -> (f32, f32, f32, f32) {
    let lw = (s * 0.004).max(1.0);
    let rr = s * 0.035;
    let bezel = rounded(0.0, 0.0, s, s, rr);
    // A neutral charcoal: the glass colour lifted a little towards the text, so the bezel
    // takes no tint from the theme's surfaces.
    fill(w, o, &bezel, ink.ground);
    fill(w, o, &bezel, scale_alpha(ink.text, 0.09));
    // The bezel catches the light from above: lighter bands towards the top.
    for i in 1..=4 {
        let h = s * (0.5 - 0.1 * i as f32);
        fill(
            w,
            o,
            &rounded(0.0, 0.0, s, h, rr),
            scale_alpha(ink.text, 0.012),
        );
    }
    stroke(w, o, &bezel, lw * 1.5, ink.ground, true);
    // The bevel: a lit edge along the top.
    stroke(
        w,
        o,
        &[(rr, lw * 1.2), (s - rr, lw * 1.2)],
        lw,
        scale_alpha(ink.text, 0.28),
        false,
    );
    let k = BEZEL * s;
    // On a small page the sensor, rocker and keys would only be specks: leave them off.
    let detailed = s >= DETAIL_FROM;
    if detailed {
        // The light sensor over the glass and the brightness rocker under it.
        fill(
            w,
            o,
            &crate::shapes::circle(s * 0.5, k * 0.5, k * 0.13),
            ink.ground,
        );
        let (rw, rh) = (s * 0.14, k * 0.42);
        let ry = s - k * 0.5 - rh / 2.0;
        key(w, o, (s * 0.5 - rw / 2.0, ry, rw, rh), ink);
        stroke(
            w,
            o,
            &[(s * 0.5, ry + rh * 0.2), (s * 0.5, ry + rh * 0.8)],
            lw,
            ink.ground,
            false,
        );
    }
    let glass = (k, k, s - 2.0 * k, s - 2.0 * k);
    let g = rect(glass.0, glass.1, glass.2, glass.3);
    fill(w, o, &g, ink.ground);
    // The picture glows a little towards the middle, so the edges of the glass fall away. The
    // glow and the sheen are in the face colour, lighter than the glass in light and dark themes.
    for i in 1..=8 {
        let d = glass.2 * 0.035 * i as f32;
        fill(
            w,
            o,
            &rounded(
                glass.0 + d,
                glass.1 + d,
                glass.2 - 2.0 * d,
                glass.3 - 2.0 * d,
                d * 2.0,
            ),
            scale_alpha(ink.face, 0.07),
        );
    }
    // The back-light falls from the top of the glass: bands growing down from the top edge,
    // tinted towards the cyan of the page's legends.
    for i in 1..=8 {
        let h = glass.3 * 0.07 * i as f32;
        fill(
            w,
            o,
            &rect(glass.0, glass.1, glass.2, h),
            scale_alpha(ink.info, 0.007),
        );
    }
    // A soft sheen: nested bands, each a little fainter than the glass under the last.
    for i in 0..5 {
        fill(
            w,
            o,
            &sheen(glass, 0.3 + 0.1 * i as f32),
            scale_alpha(ink.face, 0.16),
        );
    }
    // The glass sits in the bezel: a dark rim with a light lip under it.
    stroke(w, o, &g, lw * 1.5, scale_alpha(ink.ground, 0.9), true);
    stroke(
        w,
        o,
        &[
            (glass.0, glass.1 + glass.3 + lw),
            (glass.0 + glass.2, glass.1 + glass.3 + lw),
        ],
        lw,
        scale_alpha(ink.text, 0.15),
        false,
    );
    if detailed {
        for y in KEYS {
            for x in [k * 0.18, s - k * 0.78] {
                key(w, o, (x, y * s - s * 0.024, k * 0.6, s * 0.048), ink);
            }
        }
    }
    glass
}

/// Paint a raised key `(x, y, wd, h)` on the bezel: rounded, lit along the top, shadowed under.
fn key(w: &mut Window, o: Point<Pixels>, (x, y, wd, h): (f32, f32, f32, f32), ink: Ink) {
    let r = wd.min(h) * 0.25;
    fill(w, o, &rounded(x, y + h * 0.12, wd, h, r), ink.ground);
    fill(w, o, &rounded(x, y, wd, h, r), scale_alpha(ink.text, 0.14));
    fill(
        w,
        o,
        &rounded(x, y, wd, h * 0.45, r),
        scale_alpha(ink.text, 0.06),
    );
}

/// The sheen on a display's glass `(x, y, wd, h)`: a band from the top edge down the left side
/// reaching `k` of the way along each, as a polygon. Pure.
pub(crate) fn sheen((x, y, wd, h): (f32, f32, f32, f32), k: f32) -> Vec<(f32, f32)> {
    let k = k.clamp(0.0, 1.0);
    vec![(x, y), (x + wd * k, y), (x, y + h * k)]
}

/// The turn of the engine-indication arc at `f` of the scale: it opens at nine o'clock and runs
/// clockwise over the top to just past three o'clock. Pure.
pub(crate) fn arc_turn(f: f32) -> f32 {
    ARC.0 + ARC.1 * f.clamp(0.0, 1.0)
}

/// Start and span of the engine-indication arc, in turns.
pub(crate) const ARC: (f32, f32) = (0.75, 0.55);

/// Stroke `pts` in `col` as a lit line on the glass: two faint wider passes under it give it a
/// glow.
pub(crate) fn glow(
    w: &mut Window,
    o: Point<Pixels>,
    pts: &[(f32, f32)],
    lw: f32,
    col: Hsla,
    closed: bool,
) {
    stroke(w, o, pts, lw * 4.0, scale_alpha(col, 0.07), closed);
    stroke(w, o, pts, lw * 2.2, scale_alpha(col, 0.14), closed);
    stroke(w, o, pts, lw, col, closed);
}

/// How many slices build up the sweep's trail: enough that no step shows.
const TRAIL: usize = 20;

/// Where the `n` slices of a sweep to `f` start: from the arc's start, packed ever closer
/// towards `f`, so the stacked slices brighten smoothly up to the needle. Pure.
pub(crate) fn trail(f: f32, n: usize) -> impl Iterator<Item = f32> {
    let f = f.clamp(0.0, 1.0);
    (0..n).map(move |i| f * (i as f32 / n as f32).sqrt())
}

/// Paint a lit tile `(x, y, wd, h)` with corners of radius `r`: a pane of glass shading up from
/// its foot in `col`, under a glowing frame, the frame full strength when `alert`.
pub(crate) fn tile(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    r: f32,
    col: Hsla,
    alert: bool,
) {
    let pane = rounded(x, y, wd, h, r);
    fill(
        w,
        o,
        &pane,
        scale_alpha(col, if alert { 0.08 } else { 0.04 }),
    );
    // Bands rising from the foot, each shorter, so the pane brightens downwards.
    for i in 1..=6 {
        let bh = h * (1.0 - i as f32 * 0.14);
        fill(
            w,
            o,
            &rounded(x, y + h - bh, wd, bh, r),
            scale_alpha(col, 0.025),
        );
    }
    let lw = (h * 0.025).max(1.0);
    glow(
        w,
        o,
        &pane,
        lw,
        scale_alpha(col, if alert { 1.0 } else { 0.55 }),
        true,
    );
    // A light catch along the top edge of the pane.
    stroke(
        w,
        o,
        &[(x + r, y + lw * 1.5), (x + wd - r, y + lw * 1.5)],
        lw * 0.8,
        scale_alpha(col, 0.35),
        false,
    );
}

/// The slice of an engine-indication arc between the fractions `a` and `f`: a pie slice from
/// the centre. Pure.
pub(crate) fn sector(cx: f32, cy: f32, r: f32, a: f32, f: f32) -> Vec<(f32, f32)> {
    let mut p = vec![(cx, cy)];
    p.extend(turn_arc(cx, cy, r, arc_turn(a.min(f)), arc_turn(f)));
    p
}

/// Paint an engine-indication arc around `(cx, cy)` of radius `r`: the sweep to `f` filled as a
/// trail brightening towards the needle (cyan, or `col` once that is not the text colour), the
/// white arc with its majors, the zones `(from, to, colour)` as wide bands with a radial limit
/// line, and a thin glowing needle in `col`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn engine_arc(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    f: f32,
    majors: usize,
    zones: &[(f32, f32, Hsla)],
    col: Hsla,
    ink: Ink,
) {
    let lw = (r * 0.03).max(1.0);
    if f > 0.0 {
        // The sweep takes the colour of the value once it is past a limit.
        let tint = if col == ink.text { ink.info } else { col };
        // Layered slices, each starting further along, so the sweep brightens towards the
        // needle like a trail behind it.
        for a in trail(f, TRAIL) {
            fill(w, o, &sector(cx, cy, r, a, f), scale_alpha(tint, 0.03));
        }
    }
    stroke(
        w,
        o,
        &turn_arc(cx, cy, r, ARC.0, ARC.0 + ARC.1),
        lw,
        ink.text,
        false,
    );
    for &(a, z, zc) in zones {
        glow(
            w,
            o,
            &turn_arc(cx, cy, r + lw * 1.6, arc_turn(a), arc_turn(z)),
            lw * 2.2,
            zc,
            false,
        );
        let t = arc_turn(a);
        stroke(
            w,
            o,
            &[polar(cx, cy, r * 0.82, t), polar(cx, cy, r + lw * 2.8, t)],
            lw * 1.2,
            zc,
            false,
        );
    }
    let majors = majors.max(1);
    for i in 0..=majors {
        let t = arc_turn(i as f32 / majors as f32);
        stroke(
            w,
            o,
            &[polar(cx, cy, r, t), polar(cx, cy, r * 0.88, t)],
            lw,
            ink.text,
            false,
        );
    }
    let t = arc_turn(f);
    glow(
        w,
        o,
        &[polar(cx, cy, r * 0.12, t), polar(cx, cy, r * 1.04, t)],
        lw * 1.4,
        col,
        false,
    );
}

/// Paint a readout box: the glass colour inside a white frame, an optional pointer notch on its
/// left edge pointing at a scale.
pub(crate) fn readout_box(
    w: &mut Window,
    o: Point<Pixels>,
    (x, y, wd, h): (f32, f32, f32, f32),
    notch: bool,
    frame: Hsla,
    ink: Ink,
) {
    let mut p = vec![(x, y), (x + wd, y), (x + wd, y + h), (x, y + h)];
    if notch {
        let k = h * 0.3;
        p.extend([
            (x, y + h / 2.0 + k),
            (x - k, y + h / 2.0),
            (x, y + h / 2.0 - k),
        ]);
    }
    framed(w, o, &p, (h * 0.05).max(1.0), frame, ink);
}

/// Paint the polygon `p` as a readout: the glass colour inside a frame in `frame`.
pub(crate) fn framed(
    w: &mut Window,
    o: Point<Pixels>,
    p: &[(f32, f32)],
    lw: f32,
    frame: Hsla,
    ink: Ink,
) {
    fill(w, o, p, ink.ground);
    glow(w, o, p, lw, frame, true);
}

/// A rolling-drum readout box `(x, y, wd, h)`: its last `cw` stands `e` taller above and below,
/// a window on the drum that rolls the last digit. Pure.
pub(crate) fn drum_box((x, y, wd, h): (f32, f32, f32, f32), cw: f32, e: f32) -> Vec<(f32, f32)> {
    let cx = x + wd - cw.clamp(0.0, wd);
    vec![
        (x, y),
        (cx, y),
        (cx, y - e),
        (x + wd, y - e),
        (x + wd, y + h + e),
        (cx, y + h + e),
        (cx, y + h),
        (x, y + h),
    ]
}

/// The digits a rolling drum shows round `d`: the next one above, `d`, and the one before
/// under it. `None` when `d` is not a digit. Pure.
pub(crate) fn drum_digits(d: char) -> Option<[char; 3]> {
    let n = d.to_digit(10)?;
    let at = |k: u32| char::from_digit(k % 10, 10).unwrap_or(d);
    Some([at(n + 1), d, at(n + 9)])
}

/// A bug (a target marker) at `(x, y)` pointing down at a scale: a notched block `k` wide. Pure.
pub(crate) fn bug(x: f32, y: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x - k, y - k * 0.8),
        (x + k, y - k * 0.8),
        (x + k, y),
        (x + k * 0.35, y),
        (x, y - k * 0.4),
        (x - k * 0.35, y),
        (x - k, y),
    ]
}

/// A dotted leader from `x0` to `x1` at `y`, as dot centres every `step`. Pure.
pub(crate) fn leader(x0: f32, x1: f32, y: f32, step: f32) -> Vec<(f32, f32)> {
    let step = step.max(1.0);
    let n = ((x1 - x0) / step).floor().max(0.0) as usize;
    (0..n).map(|i| (x0 + step * (i as f32 + 0.5), y)).collect()
}

/// Paint a horizontal scale from `x0` to `x1` at `y` with a filled triangle pointer at `f`, in
/// `col`, ticks at quarters.
pub(crate) fn pointer_scale(
    w: &mut Window,
    o: Point<Pixels>,
    (x0, x1, y): (f32, f32, f32),
    f: f32,
    k: f32,
    col: Hsla,
    ink: Ink,
) {
    stroke(w, o, &[(x0, y), (x1, y)], k * 0.25, ink.text, false);
    for i in 0..=4 {
        let x = x0 + (x1 - x0) * i as f32 / 4.0;
        stroke(w, o, &[(x, y), (x, y + k * 0.8)], k * 0.25, ink.text, false);
    }
    let x = x0 + (x1 - x0) * f.clamp(0.0, 1.0);
    fill(
        w,
        o,
        &[(x, y - k * 0.2), (x - k, y - k * 1.6), (x + k, y - k * 1.6)],
        col,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_engine_arc_opens_at_nine_and_runs_clockwise() {
        assert_eq!(arc_turn(0.0), 0.75);
        assert!((arc_turn(1.0) - 1.3).abs() < 1e-6);
        assert_eq!(arc_turn(2.0), arc_turn(1.0), "the arc stops at its end");
    }

    #[test]
    fn the_sheen_stays_on_the_glass() {
        let g = (10.0, 10.0, 80.0, 80.0);
        assert!(sheen(g, 0.7)
            .iter()
            .all(|p| p.0 >= 10.0 && p.0 <= 90.0 && p.1 >= 10.0 && p.1 <= 90.0));
        assert!(sheen(g, 9.0).iter().all(|p| p.0 <= 90.0 && p.1 <= 90.0));
    }

    #[test]
    fn the_trail_packs_towards_the_needle() {
        let a: Vec<f32> = trail(0.8, 10).collect();
        assert_eq!(a.len(), 10);
        assert_eq!(a[0], 0.0);
        assert!(a.iter().all(|&x| (0.0..0.8).contains(&x)));
        let gaps: Vec<f32> = a.windows(2).map(|p| p[1] - p[0]).collect();
        assert!(
            gaps.windows(2).all(|g| g[1] <= g[0]),
            "the slices crowd towards the needle"
        );
        assert_eq!(trail(-1.0, 3).collect::<Vec<_>>(), vec![0.0; 3]);
    }

    #[test]
    fn a_sector_runs_from_the_centre_between_its_fractions() {
        let p = sector(50.0, 50.0, 40.0, 0.25, 0.75);
        assert_eq!(p[0], (50.0, 50.0));
        let first = polar(50.0, 50.0, 40.0, arc_turn(0.25));
        let last = polar(50.0, 50.0, 40.0, arc_turn(0.75));
        let near =
            |a: (f32, f32), b: (f32, f32)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3;
        assert!(near(p[1], first));
        assert!(near(p[p.len() - 1], last));
        // A start past the end gives an empty slice, not one running backwards.
        let q = sector(50.0, 50.0, 40.0, 0.9, 0.5);
        assert!(q[1..]
            .iter()
            .all(|&pt| near(pt, last) || near(pt, polar(50.0, 50.0, 40.0, arc_turn(0.5)))));
    }

    #[test]
    fn the_drum_window_stands_taller_at_the_end() {
        let p = drum_box((0.0, 10.0, 40.0, 10.0), 8.0, 3.0);
        let top = p.iter().map(|q| q.1).fold(f32::MAX, f32::min);
        let bottom = p.iter().map(|q| q.1).fold(f32::MIN, f32::max);
        assert_eq!((top, bottom), (7.0, 23.0));
        assert!(p.iter().all(|q| q.0 >= 0.0 && q.0 <= 40.0));
        assert!(p.iter().filter(|q| q.1 < 10.0).all(|q| q.0 >= 32.0));
    }

    #[test]
    fn the_drum_rolls_round_its_digit() {
        assert_eq!(drum_digits('8'), Some(['9', '8', '7']));
        assert_eq!(drum_digits('9'), Some(['0', '9', '8']));
        assert_eq!(drum_digits('0'), Some(['1', '0', '9']));
        assert_eq!(drum_digits('-'), None);
    }

    #[test]
    fn a_leader_fits_between_its_ends() {
        let dots = leader(0.0, 10.0, 5.0, 2.0);
        assert_eq!(dots.len(), 5);
        assert!(dots.iter().all(|d| d.0 > 0.0 && d.0 < 10.0 && d.1 == 5.0));
        assert!(leader(5.0, 1.0, 0.0, 1.0).is_empty());
    }

    #[test]
    fn a_bug_points_down_at_its_spot() {
        let b = bug(10.0, 20.0, 4.0);
        assert!(b.iter().all(|p| p.1 <= 20.0));
        assert!(b.iter().any(|p| (p.0 - 10.0).abs() < 1e-6));
    }
}
