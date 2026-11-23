//! Progress bars drawn the way a design system draws them ([`ProgressStyle`]) and the
//! Material 3 Expressive wavy indeterminate bar.

use crate::hsla;
use dodai_core::{ProgressStyle, Theme, Tone};
use gpui::{canvas, div, point, px, size, Bounds, ParentElement, PathBuilder, Styled, Window};
use std::f32::consts::TAU;

type Origin = (f32, f32);

fn rect(window: &mut Window, o: Origin, r: (f32, f32, f32, f32), c: gpui::Hsla, radius: f32) {
    let (x, y, w, h) = r;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let b = Bounds::new(point(px(o.0 + x), px(o.1 + y)), size(px(w), px(h)));
    window.paint_quad(gpui::fill(b, c).corner_radii(px(radius)));
}

fn poly(window: &mut Window, o: Origin, pts: [(f32, f32); 4], c: gpui::Hsla) {
    let mut pb = PathBuilder::fill();
    for (i, (x, y)) in pts.iter().enumerate() {
        let p = point(px(o.0 + x), px(o.1 + y));
        if i == 0 {
            pb.move_to(p);
        } else {
            pb.line_to(p);
        }
    }
    pb.close();
    if let Ok(p) = pb.build() {
        window.paint_path(p, c);
    }
}

fn with_alpha(mut c: gpui::Hsla, a: f32) -> gpui::Hsla {
    c.a *= a.clamp(0.0, 1.0);
    c
}

/// Height of a bar in `style`.
pub fn styled_height(style: ProgressStyle) -> f32 {
    match style {
        ProgressStyle::Hud => 16.0,
        ProgressStyle::Stripes => 22.0,
        ProgressStyle::Stones => 18.0,
    }
}

const HUD_CELLS: usize = 26;
const STONES: usize = 12;

fn hud(
    window: &mut Window,
    o: Origin,
    w: f32,
    h: f32,
    frac: Option<f32>,
    c: (gpui::Hsla, gpui::Hsla),
    now: f32,
) {
    let (track, fill) = c;
    let n = HUD_CELLS;
    let slant = h * 0.4;
    let slot = (w - slant) / n as f32;
    let gap = 3.0;
    let lit = frac.map(|f| f.clamp(0.0, 1.0) * n as f32);
    let comet = (now * 0.9).fract() * (n as f32 + 8.0) - 4.0;
    for i in 0..n {
        let x0 = i as f32 * slot;
        let x1 = x0 + slot - gap;
        let cell = [(x0 + slant, 0.0), (x1 + slant, 0.0), (x1, h), (x0, h)];
        let a = match lit {
            Some(l) => (l - i as f32).clamp(0.0, 1.0),
            None => {
                let d = comet - i as f32;
                if (0.0..8.0).contains(&d) {
                    1.0 - d / 8.0
                } else {
                    0.0
                }
            }
        };
        poly(window, o, cell, with_alpha(track, 0.55));
        if a > 0.0 {
            poly(window, o, cell, with_alpha(fill, a));
        }
        // The head flickers and throws a halo.
        let head = match lit {
            Some(l) => l > 0.0 && l < n as f32 && (l - 0.001).floor() as i64 == i as i64,
            None => (0.0..1.0).contains(&(comet - i as f32)),
        };
        if head {
            let flick = 0.55 + 0.45 * (now * 18.0).sin();
            let g = 3.0;
            poly(
                window,
                o,
                [
                    (x0 + slant - g, -g),
                    (x1 + slant + g, -g),
                    (x1 + g, h + g),
                    (x0 - g, h + g),
                ],
                with_alpha(fill, 0.28 * flick),
            );
        }
    }
}

fn stripes(
    window: &mut Window,
    o: Origin,
    w: f32,
    h: f32,
    frac: Option<f32>,
    c: (gpui::Hsla, gpui::Hsla, gpui::Hsla),
    now: f32,
) {
    let (track, fill, line) = c;
    let bw = 2.5;
    rect(window, o, (0.0, 0.0, w, h), line, 0.0);
    rect(window, o, (bw, bw, w - 2.0 * bw, h - 2.0 * bw), track, 0.0);
    let (ix0, ix1) = (bw, w - bw);
    let (iy, ih) = (bw, h - 2.0 * bw);
    let active = match frac {
        Some(f) => ix0 + (ix1 - ix0) * f.clamp(0.0, 1.0),
        None => ix1,
    };
    if active <= ix0 {
        return;
    }
    rect(
        window,
        o,
        (ix0, iy, active - ix0, ih),
        with_alpha(fill, 0.45),
        0.0,
    );
    let sw = ih * 0.9;
    let period = sw * 2.0;
    let off = (now * 36.0) % period;
    let mut x = ix0 - ih - period;
    while x < active + period {
        let sx = x + off;
        let cl = |v: f32| v.clamp(ix0, active);
        poly(
            window,
            o,
            [
                (cl(sx + ih), iy),
                (cl(sx + ih + sw), iy),
                (cl(sx + sw), iy + ih),
                (cl(sx), iy + ih),
            ],
            fill,
        );
        x += period;
    }
    // A hard cut-off edge at the end of a determinate fill.
    if frac.is_some() {
        rect(window, o, (active - 2.0, iy, 2.0, ih), line, 0.0);
    }
}

fn stones(
    window: &mut Window,
    o: Origin,
    w: f32,
    h: f32,
    frac: Option<f32>,
    c: (gpui::Hsla, gpui::Hsla),
    now: f32,
) {
    let (track, fill) = c;
    let n = STONES;
    let slot = w / n as f32;
    let side = (slot - 5.0).min(h);
    let lit = frac.map(|f| f.clamp(0.0, 1.0) * n as f32);
    for i in 0..n {
        let cx = slot * (i as f32 + 0.5);
        let mut s = side;
        let a = match lit {
            Some(l) => {
                if l > 0.0 && l < n as f32 && (l - 0.001).floor() as i64 == i as i64 {
                    s *= 0.86 + 0.14 * (now * 5.0).sin();
                }
                (l - i as f32).clamp(0.0, 1.0)
            }
            None => 0.2 + 0.8 * (now * 4.0 - i as f32 * 0.6).sin().max(0.0),
        };
        rect(
            window,
            o,
            (cx - side / 2.0, (h - side) / 2.0, side, side),
            track,
            5.0,
        );
        if a > 0.0 {
            rect(
                window,
                o,
                (cx - s / 2.0, (h - s) / 2.0, s, s),
                with_alpha(fill, a),
                5.0,
            );
        }
    }
}

/// A bar in `style`: `frac` is `Some(0..=1)` when determinate and `None` when it just runs.
pub fn styled_bar(
    t: &Theme,
    style: ProgressStyle,
    frac: Option<f32>,
    tone: Option<Tone>,
) -> gpui::Div {
    let k = t.progress(tone);
    let (track, fill, line) = (hsla(k.track), hsla(k.fill), hsla(k.border));
    let h = styled_height(style);
    div().w_full().h(px(h)).relative().child(
        canvas(
            |_, _, _| (),
            move |b, _, window, _| {
                let o: Origin = (b.origin.x.into(), b.origin.y.into());
                let w: f32 = b.size.width.into();
                let now = crate::display::elapsed_secs();
                match style {
                    ProgressStyle::Hud => hud(window, o, w, h, frac, (track, fill), now),
                    ProgressStyle::Stripes => {
                        stripes(window, o, w, h, frac, (track, fill, line), now)
                    }
                    ProgressStyle::Stones => stones(window, o, w, h, frac, (track, fill), now),
                }
                crate::motion::next_frame(window);
            },
        )
        .absolute()
        .size_full(),
    )
}

/// Material 3 Expressive wavy indeterminate bar: a travelling wave sliding along a thin track.
pub fn wavy_indeterminate(t: &Theme, tone: Option<Tone>) -> gpui::Div {
    let k = t.progress(tone);
    let (track, fill) = (hsla(k.track), hsla(k.fill));
    div().w_full().h(px(t.sizes().chart_cell)).relative().child(
        canvas(
            |_, _, _| (),
            move |b, _, window, _| {
                let w: f32 = b.size.width.into();
                let (ox, cy) = (b.origin.x, b.origin.y + b.size.height / 2.0);
                let stroke = 4.0;
                let now = crate::display::elapsed_secs();
                let (l, sw) = crate::feedback::indeterminate_segment((now / 1.6).fract());
                let (x0, x1) = (l * w, (l + sw) * w);
                let mut pb = PathBuilder::stroke(px(stroke));
                pb.move_to(point(ox, cy));
                pb.line_to(point(ox + px(w), cy));
                if let Ok(p) = pb.build() {
                    window.paint_path(p, track);
                }
                if x1 - x0 > 1.0 {
                    let steps = ((x1 - x0) / 2.0).ceil().max(2.0) as usize;
                    let mut pb = PathBuilder::stroke(px(stroke));
                    for i in 0..=steps {
                        let x = x0 + (x1 - x0) * i as f32 / steps as f32;
                        let y = 3.0 * (x / 36.0 * TAU - now * 4.0).sin();
                        let p = point(ox + px(x), cy + px(y));
                        if i == 0 {
                            pb.move_to(p);
                        } else {
                            pb.line_to(p);
                        }
                    }
                    if let Ok(p) = pb.build() {
                        window.paint_path(p, fill);
                    }
                }
                crate::motion::next_frame(window);
            },
        )
        .absolute()
        .size_full(),
    )
}
