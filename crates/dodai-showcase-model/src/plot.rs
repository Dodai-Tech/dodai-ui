//! Line plots as data: coloured polylines in pixel space, so a toolkit only strokes them.
//!
//! The easing and physics sections of the Animation page build these; they are also the
//! first step towards charts, which are polylines and rectangles too.

use dodai_core::motion::Easing;
use dodai_core::{Color, Theme};

/// One stroked polyline in the plot's pixel space (origin top-left).
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    /// Stroke colour (alpha included).
    pub color: Color,
    /// Stroke width px.
    pub width: f32,
    /// Points px.
    pub points: Vec<(f32, f32)>,
}

/// A plot: a `w`×`h` px box and the lines drawn in it.
#[derive(Debug, Clone, PartialEq)]
pub struct Plot {
    /// Width px.
    pub w: f32,
    /// Height px.
    pub h: f32,
    /// Lines, back to front.
    pub lines: Vec<Line>,
}

/// Maps data coordinates (`x` in `0..=1`, `y` in `lo..=hi`) to a padded pixel box.
#[derive(Debug, Clone, Copy)]
struct Frame {
    w: f32,
    h: f32,
    pad: f32,
    lo: f32,
    hi: f32,
}

impl Frame {
    fn at(&self, (x, y): (f32, f32)) -> (f32, f32) {
        let span = (self.hi - self.lo).max(f32::EPSILON);
        (
            self.pad + x * (self.w - 2.0 * self.pad),
            self.pad + (1.0 - (y - self.lo) / span) * (self.h - 2.0 * self.pad),
        )
    }
    fn line(&self, color: Color, width: f32, pts: &[(f32, f32)]) -> Line {
        Line {
            color,
            width,
            points: pts.iter().map(|p| self.at(*p)).collect(),
        }
    }
}

/// CSS cubic-bezier control points of an easing, when it has them.
pub fn control_points(e: Easing) -> Option<(f32, f32, f32, f32)> {
    Some(match e {
        Easing::Linear => (0.0, 0.0, 1.0, 1.0),
        Easing::EaseIn => (0.42, 0.0, 1.0, 1.0),
        Easing::EaseOut => (0.0, 0.0, 0.58, 1.0),
        Easing::EaseInOut => (0.42, 0.0, 0.58, 1.0),
        Easing::CubicBezier(a, b, c, d) => (a, b, c, d),
        Easing::Spring { .. }
        | Easing::Bounce
        | Easing::Elastic
        | Easing::Back(_)
        | Easing::Anticipate(_) => return None,
    })
}

/// `f` sampled `n + 1` times over `0..=1`.
fn sample(n: usize, f: impl Fn(f32) -> f32) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|k| {
            let x = k as f32 / n as f32;
            (x, f(x))
        })
        .collect()
}

/// A square plot of an easing curve: the frame, the diagonal, the control handles (for the
/// bezier curves) and the curve itself.
pub fn easing(theme: &Theme, e: Easing, side: f32) -> Plot {
    easing_at(theme, e, side, None)
}

/// A dot on a plot: a short fat stroke at `at`.
fn dot(f: &Frame, color: Color, at: (f32, f32)) -> Line {
    let (x, y) = f.at(at);
    Line {
        color,
        width: 11.0,
        points: vec![(x - 0.3, y), (x + 0.3, y)],
    }
}

/// [`easing`] while it plays: the curve drawn up to `progress` (`0..=1`) with a dot at its tip.
/// `None` draws the whole curve without a dot.
pub fn easing_at(theme: &Theme, e: Easing, side: f32, progress: Option<f32>) -> Plot {
    let p = &theme.palette;
    let (border, accent, muted) = (p.border, p.primary, p.text_muted);
    let ctrl = control_points(e);
    let hi = if ctrl.is_some() { 1.0 } else { 1.4 };
    let f = Frame {
        w: side,
        h: side,
        pad: 10.0,
        lo: 0.0,
        hi,
    };
    let mut lines = vec![
        f.line(
            border,
            1.0,
            &[(0.0, 0.0), (1.0, 0.0), (1.0, hi), (0.0, hi), (0.0, 0.0)],
        ),
        f.line(border.with_alpha(0.5), 1.0, &[(0.0, 0.0), (1.0, 1.0)]),
    ];
    match ctrl {
        Some((x1, y1, x2, y2)) => {
            lines.push(f.line(muted, 1.0, &[(0.0, 0.0), (x1, y1)]));
            lines.push(f.line(muted, 1.0, &[(1.0, 1.0), (x2, y2)]));
        }
        None => lines.push(f.line(muted.with_alpha(0.5), 1.0, &[(0.0, 1.0), (1.0, 1.0)])),
    }
    let upto = progress.map_or(1.0, |p| p.clamp(0.0, 1.0));
    let curve = sample(48, |x| x * upto)
        .into_iter()
        .map(|(x, _)| (x, e.apply(x).clamp(0.0, hi)))
        .collect::<Vec<_>>();
    lines.push(f.line(accent, 3.0, &curve));
    if let Some(&tip) = curve.last().filter(|_| progress.is_some()) {
        lines.push(dot(&f, accent, tip));
    }
    Plot {
        w: side,
        h: side,
        lines,
    }
}

/// A one-dimensional track `w`×`h` px with a dot at `value` (`0..=1`, may overshoot a little):
/// how an eased value would move something along a line.
pub fn track(theme: &Theme, w: f32, h: f32, value: f32) -> Plot {
    let f = Frame {
        w,
        h,
        pad: 8.0,
        lo: 0.0,
        hi: 1.0,
    };
    Plot {
        w,
        h,
        lines: vec![
            f.line(theme.palette.border, 2.0, &[(0.0, 0.5), (1.0, 0.5)]),
            dot(&f, theme.palette.primary, (value.clamp(-0.2, 1.4), 0.5)),
        ],
    }
}

/// [`lane`] with a dot on the curve at `at` (`0..=1`).
pub fn lane_at(
    theme: &Theme,
    (w, h): (f32, f32),
    (lo, hi): (f32, f32),
    at: f32,
    f: impl Fn(f32) -> f32,
) -> Plot {
    let mut p = lane(theme, w, h, lo, hi, &f);
    let fr = Frame {
        w,
        h,
        pad: 4.0,
        lo,
        hi,
    };
    p.lines
        .push(dot(&fr, theme.palette.primary, (at, f(at).clamp(lo, hi))));
    p
}

/// Where a looping demo is, `0..=1` of `period` seconds, holding the end for `hold` of it.
pub fn cycle(seconds: f32, period: f32, hold: f32) -> f32 {
    ((seconds % period) / (period * (1.0 - hold))).min(1.0)
}

/// A plot of `f` over `0..=1` with its value range `lo..=hi`, and a baseline at zero.
pub fn lane(theme: &Theme, w: f32, h: f32, lo: f32, hi: f32, f: impl Fn(f32) -> f32) -> Plot {
    let fr = Frame {
        w,
        h,
        pad: 4.0,
        lo,
        hi,
    };
    Plot {
        w,
        h,
        lines: vec![
            fr.line(theme.palette.border, 2.0, &[(0.0, 0.0), (1.0, 0.0)]),
            fr.line(theme.palette.primary, 2.0, &sample(80, f)),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn theme() -> Theme {
        Theme::default()
    }

    #[test]
    fn an_easing_plot_ends_where_the_curve_ends() {
        let p = easing(&theme(), Easing::Linear, 200.0);
        let curve = p.lines.last().unwrap();
        let (x0, y0) = curve.points[0];
        let (x1, y1) = *curve.points.last().unwrap();
        assert_eq!((x0, y0), (10.0, 190.0));
        assert_eq!((x1, y1), (190.0, 10.0));
        assert_eq!((p.w, p.h), (200.0, 200.0));
    }

    #[test]
    fn bezier_curves_show_their_handles_and_physical_ones_the_target_line() {
        let bez = easing(&theme(), Easing::EaseInOut, 200.0);
        let spring = easing(&theme(), Easing::SPRING, 200.0);
        // frame + diagonal + two handles + curve, against frame + diagonal + target + curve
        assert_eq!(bez.lines.len(), 5);
        assert_eq!(spring.lines.len(), 4);
        assert!(control_points(Easing::SPRING).is_none());
    }

    #[test]
    fn every_point_of_a_lane_stays_inside_its_box() {
        let p = lane(&theme(), 150.0, 70.0, 0.0, 1.4, |x| x * 1.4);
        for l in &p.lines {
            for (x, y) in &l.points {
                assert!((0.0..=150.0).contains(x) && (0.0..=70.0).contains(y));
            }
        }
    }
}
