//! Light and motion effects: depth, tint, ripples, wash, gloss, brackets, pulse, chase, sheen,
//! glitch and the rest.

use crate::canvas::Canvas;
use crate::geom::{noise, outline_of};
use crate::{scale_alpha, Rgba, SurfacePaint};

/// Inner shadow: dark rings pulled in from the edge, fading inward.
pub(crate) fn depth(cv: &mut Canvas, sp: &SurfacePaint) {
    let d = sp.fx.depth.clamp(0.0, 1.0);
    let n = 6;
    for k in 0..n {
        let f = 1.0 - k as f32 / n as f32;
        cv.outline_inset(
            cv.inset + 1.0 + 2.0 * k as f32,
            2.0,
            scale_alpha(sp.shade, 0.55 * d * f * f),
        );
    }
}

/// Material: a tonal wash of the accent, strongest at the top (the elevation tint).
pub(crate) fn tint(cv: &mut Canvas, sp: &SurfacePaint) {
    let t = sp.fx.tint.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    let step = 3.0;
    let mut y = 0.0;
    while y < h {
        let f = 1.0 - y / h;
        cv.quad(
            0.0,
            y,
            w,
            step,
            scale_alpha(sp.neon, (0.02 + 0.10 * f * f) * t),
        );
        y += step;
    }
}

/// Draw one frame (`t` in `0..1`) of a click ripple of the given style.
pub(crate) fn press_frame(
    cv: &mut Canvas,
    style: crate::PressStyle,
    accent: Rgba,
    ink: Rgba,
    k: f32,
    at: (f32, f32),
    t: f32,
) {
    use crate::PressStyle as P;
    let (w, h) = (cv.w, cv.h);
    // Ease-out spread, and a fade that holds for the first third.
    let spread = 1.0 - (1.0 - t).powi(3);
    let fade = 1.0 - ((t - 0.3).max(0.0) / 0.7).powi(2);
    let far = |x: f32, y: f32| {
        let dx = x.max(w - x);
        let dy = y.max(h - y);
        (dx * dx + dy * dy).sqrt()
    };
    let (cx, cy) = (w / 2.0, h / 2.0);
    match style {
        P::Ink | P::Centre => {
            let (px0, py0) = if style == P::Ink { at } else { (cx, cy) };
            let r = far(px0, py0) * spread;
            let c = scale_alpha(ink, 0.18 * k * fade);
            let mut y = cv.inset;
            while y < h - cv.inset {
                let dy = y + 0.5 - py0;
                if dy.abs() < r {
                    let half = (r * r - dy * dy).sqrt();
                    cv.quad(px0 - half, y, 2.0 * half, 1.0, c);
                }
                y += 1.0;
            }
        }
        P::Ring | P::Echo => {
            let waves = if style == P::Ring { 1 } else { 3 };
            for i in 0..waves {
                let ti = (t - 0.12 * i as f32).clamp(0.0, 1.0);
                if ti <= 0.0 {
                    continue;
                }
                let r = far(at.0, at.1) * (1.0 - (1.0 - ti).powi(3));
                let f = (1.0 - ti) * (1.0 - ti);
                cv.ring_clipped(at.0, at.1, r, 2.0, scale_alpha(accent, 0.6 * k * f));
            }
        }
        P::Sweep => {
            // A soft vertical band moving away from the touch point towards the far side.
            let dir = if at.0 < cx { 1.0 } else { -1.0 };
            let travel = w * spread;
            let x = at.0 + dir * travel;
            let band = (w * 0.25).max(12.0);
            let n = 10;
            for j in 0..n {
                let f = (j as f32 + 0.5) / n as f32;
                let a = (std::f32::consts::PI * f).sin();
                let seg = band / n as f32;
                cv.quad(
                    x - band / 2.0 + j as f32 * seg,
                    0.0,
                    seg + 0.5,
                    h,
                    scale_alpha(accent, 0.4 * k * a * fade),
                );
            }
        }
        P::Glow => {
            // Stacked discs make a soft radial glow that only fades.
            let r0 = h.max(24.0);
            let layers = 6;
            for j in (0..layers).rev() {
                let r = r0 * (0.35 + 0.65 * (j + 1) as f32 / layers as f32);
                let c = scale_alpha(accent, 0.14 * k * (1.0 - t));
                let mut y = cv.inset;
                while y < h - cv.inset {
                    let dy = y + 0.5 - at.1;
                    if dy.abs() < r {
                        let half = (r * r - dy * dy).sqrt();
                        cv.quad(at.0 - half, y, 2.0 * half, 1.0, c);
                    }
                    y += 1.0;
                }
            }
        }
    }
}

/// Neo-brutalism: a hard offset block along the bottom and right inner edges.
pub(crate) fn pop(cv: &mut Canvas, sp: &SurfacePaint) {
    let k = sp.fx.pop.clamp(0.0, 1.0);
    // Thinner on small controls so the block never touches a label.
    let t = if cv.h < 64.0 {
        2.0 + 1.5 * k
    } else {
        3.0 + 4.0 * k
    };
    let (w, h, i) = (cv.w, cv.h, cv.inset);
    let c = scale_alpha(sp.ink, 0.9);
    cv.quad(i + t, h - i - t, w - 2.0 * i - t, t, c);
    cv.quad(w - i - t, i + t, t, h - 2.0 * i - 2.0 * t, c);
}

/// Neo-brutalism: translucent masking tape across the top-left and bottom-right corners.
pub(crate) fn tape(cv: &mut Canvas, sp: &SurfacePaint) {
    let k = sp.fx.tape.clamp(0.0, 1.0);
    let (w, h, i) = (cv.w, cv.h, cv.inset);
    // Tape is for containers; on a button it would sit over the label.
    if h < 64.0 {
        return;
    }
    let l = (w.min(h) * 0.22).clamp(14.0, 30.0);
    let t = l * 0.45;
    let c = scale_alpha(sp.neon, 0.75 * k);
    let edge = scale_alpha(sp.ink, 0.35 * k);
    let band = |cv: &mut Canvas, a: (f32, f32), b: (f32, f32), d: (f32, f32)| {
        let q = [a, b, (b.0 + d.0, b.1 + d.1), (a.0 + d.0, a.1 + d.1)];
        cv.polygon(&q, c);
        cv.polyline(&[q[0], q[1]], 1.0, edge);
        cv.polyline(&[q[2], q[3]], 1.0, edge);
    };
    band(cv, (i, i + l), (i + l, i), (-0.0, t));
    band(cv, (w - i - l, h - i), (w - i, h - i - l), (0.0, -t));
}

/// The colour of a three-stop gradient at `t` (0 = first stop, 1 = last), opaque.
pub(crate) fn wash_colour(stops: &[Rgba; 3], t: f32) -> Rgba {
    let t = t.clamp(0.0, 1.0);
    let (a, b, t) = if t < 0.5 {
        (stops[0], stops[1], t * 2.0)
    } else {
        (stops[1], stops[2], t * 2.0 - 1.0)
    };
    Rgba {
        r: a.r + (b.r - a.r) * t,
        g: a.g + (b.g - a.g) * t,
        b: a.b + (b.b - a.b) * t,
        a: 1.0,
    }
}

/// A hairline rim along the whole outline, coloured by the design's gradient at 135° and
/// brightest at the top left, where the light falls in; a softer second line sits just inside.
pub(crate) fn rim(cv: &mut Canvas, sp: &SurfacePaint) {
    let k = sp.fx.rim.clamp(0.0, 1.0);
    let d = cv.inset * 0.5 + 0.5;
    let (w, h) = (cv.w - 2.0 * d, cv.h - 2.0 * d);
    if w < 4.0 || h < 4.0 {
        return;
    }
    let pts: Vec<(f32, f32)> = cv
        .shape
        .outline(w, h)
        .into_iter()
        .map(|(x, y)| (x + d, y + d))
        .collect();
    let n = pts.len();
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let t = (((a.0 + b.0) / 2.0) / cv.w + ((a.1 + b.1) / 2.0) / cv.h) / 2.0;
        let c = wash_colour(&sp.wash, t);
        let lit = (1.0 - t).powf(1.4);
        cv.polyline(&[a, b], 1.25, scale_alpha(c, k * (0.25 + 0.75 * lit)));
        cv.polyline(&[a, b], 3.0, scale_alpha(c, k * 0.12 * lit));
    }
}

/// A three-colour gradient laid over the surface at 135°, in coarse cells so it stays cheap.
/// It is faint on purpose: frosted glass is tinted by it, not covered.
pub(crate) fn wash(cv: &mut Canvas, sp: &SurfacePaint) {
    let luma = |c: Rgba| 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
    let wash_luma = sp.wash.iter().map(|c| luma(*c)).sum::<f32>() / 3.0;
    // A wash darker than a bright surface (a pale primary button) would only grey it, so it
    // fades out in proportion to how much it would darken.
    let hole = luma(sp.hole);
    let darken = if hole > 0.6 {
        (hole - wash_luma).max(0.0)
    } else {
        0.0
    };
    let k = sp.fx.wash.clamp(0.0, 1.0) * (1.0 - 0.8 * darken.min(1.0));
    let (w, h) = (cv.w, cv.h);
    let cols = (w / 10.0).clamp(2.0, 20.0) as usize;
    // Rows are coarse in the middle but 2 px tall through the rounded corners, where each cell
    // is clipped to the curve; coarse rows there would turn it into steps.
    let corner = cv.shape.radius.min(h / 2.0).ceil();
    let mut ys = vec![0.0_f32];
    while *ys.last().unwrap() < h - 0.5 {
        let y = *ys.last().unwrap();
        let step = if y < corner || y >= h - corner {
            2.0
        } else {
            (h / 80.0).max(6.0)
        };
        ys.push((y + step).min(h));
    }
    let rows = ys.len() - 1;
    let edge = |n: usize, len: f32, of: usize| (len * n as f32 / of as f32).round();
    let at = |t: f32| wash_colour(&sp.wash, t);
    for j in 0..rows {
        for i in 0..cols {
            let t = ((i as f32 + 0.5) / cols as f32 + ((ys[j] + ys[j + 1]) / 2.0) / h) / 2.0;
            let (x0, x1) = (edge(i, w, cols), edge(i + 1, w, cols));
            let (y0, y1) = (ys[j], ys[j + 1]);
            // Lit from the top left: the wash is a little stronger where the light comes in.
            let lit = 1.3 - 0.6 * t;
            cv.quad(x0, y0, x1 - x0, y1 - y0, scale_alpha(at(t), 0.24 * k * lit));
        }
    }
}

/// A clear-coat reflection across the top, cut on a slant like a windscreen glint.
pub(crate) fn gloss(cv: &mut Canvas, sp: &SurfacePaint) {
    let g = sp.fx.gloss.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    // A reflection is always lighter than the surface, so it brightens the background tone.
    let lit = Rgba::WHITE;
    let band = h * 0.5;
    let step = 1.0;
    let mut y = 0.0;
    while y < band {
        let t = y / band;
        let len = w * (1.0 - 0.45 * t);
        cv.quad(
            0.0,
            y,
            len,
            step,
            scale_alpha(lit, 0.45 * g * (1.0 - t) * (1.0 - t)),
        );
        y += step;
    }
    cv.quad(0.0, cv.inset, w, 1.0, scale_alpha(lit, 0.6 * g));
}

/// A thin line of light all around the border, following every corner, with a
/// brighter spot that travels along it and a tail that fades out behind.
pub(crate) fn strip(cv: &mut Canvas, sp: &SurfacePaint) {
    let s = sp.fx.strip.clamp(0.0, 1.0);
    let d = cv.inset + 1.0;
    let (w, h) = (cv.w - 2.0 * d, cv.h - 2.0 * d);
    if w < 8.0 || h < 8.0 {
        return;
    }
    let mut sh = cv.shape.clone();
    sh.radius = (sh.radius - d).max(0.0);
    let mut pts: Vec<(f32, f32)> = outline_of(&sh, w, h)
        .into_iter()
        .map(|(x, y)| (x + d, y + d))
        .collect();
    pts.push(pts[0]);
    let mut cum = vec![0.0_f32];
    for k in 1..pts.len() {
        let (a, b) = (pts[k - 1], pts[k]);
        cum.push(cum[k - 1] + ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt());
    }
    let total = cum[cum.len() - 1];
    // Point at distance `t` along the closed outline.
    let at = |t: f32| -> (f32, f32) {
        let t = t.rem_euclid(total);
        let k = cum.partition_point(|&c| c <= t).clamp(1, cum.len() - 1);
        let f = (t - cum[k - 1]) / (cum[k] - cum[k - 1]).max(1e-3);
        (
            pts[k - 1].0 + (pts[k].0 - pts[k - 1].0) * f,
            pts[k - 1].1 + (pts[k].1 - pts[k - 1].1) * f,
        )
    };
    // The whole border carries a thin line of light, so it reads as one outline all around.
    cv.polyline(&pts, 1.5, scale_alpha(sp.neon, 0.4 * s));
    let len = (total * 0.28).clamp(40.0, 260.0);
    let head = (cv.time / sp.tune.sheen_period.max(0.8)).fract() * total;
    let n = 24;
    let seg = len / n as f32;
    for j in 0..n {
        // The head is bright and the tail fades.
        let f = (j + 1) as f32 / n as f32;
        let t0 = head - len + j as f32 * seg;
        let run: Vec<(f32, f32)> = (0..=2).map(|q| at(t0 + seg * q as f32 / 2.0)).collect();
        cv.polyline(&run, 3.5, scale_alpha(sp.neon, 0.18 * s * f * f));
        cv.polyline(&run, 1.5, scale_alpha(sp.neon, 0.95 * s * f * f));
    }
}

/// Light falling inward from the edges, in the neon colour.
pub(crate) fn edge_light(cv: &mut Canvas, sp: &SurfacePaint) {
    let e = sp.fx.edge.clamp(0.0, 1.0);
    let n = 5;
    for k in 0..n {
        let f = 1.0 - k as f32 / n as f32;
        cv.outline_inset(
            cv.inset + 0.5 + 1.6 * k as f32,
            1.6,
            scale_alpha(sp.neon, 0.5 * e * f * f),
        );
    }
}

/// Bracket marks in the four corners (kept clear of cut corners).
pub(crate) fn brackets(cv: &mut Canvas, sp: &SurfacePaint) {
    let b = sp.fx.brackets.clamp(0.0, 1.0);
    let (w, h) = (cv.w, cv.h);
    let arm = (w.min(h) * 0.22).clamp(5.0, 12.0);
    let m = cv.inset + 3.0;
    let c = scale_alpha(sp.neon, 0.9 * b);
    let left = |cv: &Canvas, y: f32| cv.span(y).map_or(m, |s| s.0.max(0.0) + m);
    let right = |cv: &Canvas, y: f32| cv.span(y).map_or(w - m, |s| s.1.min(w) - m);
    let tl = left(cv, m).max(left(cv, m + arm));
    let tr = right(cv, m).min(right(cv, m + arm));
    let bl = left(cv, h - m).max(left(cv, h - m - arm));
    let br = right(cv, h - m).min(right(cv, h - m - arm));
    cv.quad(tl, m, arm, 1.5, c);
    cv.quad(tl, m, 1.5, arm, c);
    cv.quad(tr - arm, m, arm, 1.5, c);
    cv.quad(tr - 1.5, m, 1.5, arm, c);
    cv.quad(bl, h - m - 1.5, arm, 1.5, c);
    cv.quad(bl, h - m - arm, 1.5, arm, c);
    cv.quad(br - arm, h - m - 1.5, arm, 1.5, c);
    cv.quad(br - 1.5, h - m - arm, 1.5, arm, c);
}

/// Breathing light along the border.
pub(crate) fn pulse(cv: &mut Canvas, sp: &SurfacePaint) {
    let p = sp.fx.pulse.clamp(0.0, 1.0);
    let phase = cv.time / sp.tune.pulse_period.max(0.4) * std::f32::consts::TAU;
    let a = 0.5 + 0.5 * phase.sin();
    let d = cv.inset * 0.5;
    cv.outline_inset(d, cv.inset + 4.0, scale_alpha(sp.neon, 0.12 * p * a));
    cv.outline_inset(
        d,
        cv.inset + 1.0,
        scale_alpha(sp.neon, (0.15 + 0.7 * a) * p),
    );
}

/// A bright segment (with a fading tail) running around the border.
pub(crate) fn chase(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let k = sp.fx.chase.clamp(0.0, 1.0);
    let d = cv.inset * 0.5;
    let (w, h) = (cv.w - 2.0 * d, cv.h - 2.0 * d);
    if w < 4.0 || h < 4.0 {
        return;
    }
    let pts: Vec<(f32, f32)> = cv
        .shape
        .outline(w, h)
        .into_iter()
        .map(|(x, y)| (x + d, y + d))
        .collect();
    let n = pts.len();
    let mut cum = vec![0.0_f32];
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        cum.push(cum[i] + ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt());
    }
    let total = cum[n].max(1.0);
    let at = |s: f32| {
        let s = s.rem_euclid(total);
        let i = cum.partition_point(|c| *c <= s).clamp(1, n) - 1;
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        let t = (s - cum[i]) / (cum[i + 1] - cum[i]).max(1e-3);
        (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t)
    };
    let head = ((cv.time / sp.tune.chase_period.max(0.5) + noise(seed, 5)).fract()) * total;
    let len = (total * 0.16).max(18.0);
    let parts = 3;
    for j in 0..parts {
        let (s0, s1) = (
            head - len * (j + 1) as f32 / parts as f32,
            head - len * j as f32 / parts as f32,
        );
        let seg: Vec<(f32, f32)> = (0..=6)
            .map(|i| at(s0 + (s1 - s0) * i as f32 / 6.0))
            .collect();
        let f = (parts - j) as f32 / parts as f32;
        cv.polyline(&seg, cv.inset + 3.0, scale_alpha(sp.neon, 0.15 * k * f));
        cv.polyline(&seg, cv.inset + 0.5, scale_alpha(sp.neon, 0.9 * k * f * f));
    }
}

pub(crate) fn sheen(cv: &mut Canvas, sp: &SurfacePaint) {
    let phase = (cv.time / sp.tune.sheen_period.max(0.5)).fract();
    if phase > 0.45 {
        return;
    }
    let t = phase / 0.45;
    let (w, h) = (cv.w, cv.h);
    let band = (w * 0.16).max(12.0);
    let skew = 0.6;
    let cx = -band + (w + h * skew + 2.0 * band) * t;
    let mut y = 0.0;
    while y < h {
        let x = cx - (y - h / 2.0) * skew;
        cv.quad(x, y, band, 2.0, scale_alpha(sp.light, 0.14 * sp.fx.sheen));
        y += 2.0;
    }
}

pub(crate) fn glitch(cv: &mut Canvas, sp: &SurfacePaint, seed: i32) {
    let g = sp.fx.glitch.clamp(0.0, 1.0);
    let period = sp.tune.glitch_period.max(0.5);
    let burst = sp.tune.glitch_burst + 0.25 * g;
    let t = (cv.time + noise(seed, 3) * period) % period;
    if t > burst {
        return;
    }
    let frame = (cv.time * 24.0) as i32;
    let dx = 1.0 + 3.0 * g;
    cv.outline(-dx, 1.0, scale_alpha(sp.a, 0.7));
    cv.outline(dx, 1.0, scale_alpha(sp.b, 0.7));
    let (w, h) = (cv.w, cv.h);
    let slices = 2 + (g * 2.0) as i32;
    for i in 0..slices {
        let y = noise(frame, seed + i) * (h - 4.0);
        let th = 2.0 + noise(i, frame) * (4.0 + 6.0 * g);
        let off = (noise(frame + 1, seed + i) - 0.5) * 2.0 * (4.0 + 12.0 * g);
        let c = if i % 2 == 0 { sp.a } else { sp.b };
        cv.quad(off, y, w, th, scale_alpha(c, 0.35));
        cv.quad(-off, y + th, w * 0.4, 1.0, scale_alpha(sp.light, 0.4));
    }
}
