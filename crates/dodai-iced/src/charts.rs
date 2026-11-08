//! Charts for iced, drawn on a canvas from `dodai-charts-core` geometry.
//!
//! Layout is a pure function ([`ChartKind::layout`]) from data and size to a list of
//! [`Prim`]s in pixel coordinates, so it is unit-tested without a renderer; the canvas
//! program only maps each primitive to a path and a theme colour.

use dodai_core::math::{nice_ticks, norm, smooth_points, turn_arc};
use iced::{
    mouse,
    widget::canvas::{self, Canvas, Geometry, Path, Stroke},
    Color, Element, Length, Point, Rectangle, Renderer, Size, Theme as IcedBaseTheme,
};

use crate::{color, IcedTheme};

/// Which colour a primitive takes from the theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ink {
    /// Series colour `i` (wraps).
    Series(usize),
    /// Gridlines and tracks.
    Grid,
}

/// One drawable, in pixels relative to the chart's top-left.
#[derive(Debug, Clone, PartialEq)]
pub enum Prim {
    /// Open polyline.
    Line(Vec<(f32, f32)>, Ink, f32),
    /// Filled polygon at `alpha`.
    Fill(Vec<(f32, f32)>, Ink, f32),
    /// Filled rectangle `(x, y, w, h)`.
    Rect(f32, f32, f32, f32, Ink),
}

/// A chart and its data.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartKind {
    /// Tiny trend line with an area fill.
    Sparkline(Vec<f32>),
    /// Multi-series lines on a gridded axis range.
    Line(Vec<Vec<f32>>),
    /// Grouped bars: one group per category, one bar per series.
    Bars(Vec<Vec<f32>>),
    /// A 0..1 value on a 270° arc.
    Gauge(f32),
}

fn range(series: &[Vec<f32>], zero: bool) -> (f32, f32, Vec<f32>) {
    let all = series.iter().flatten().copied();
    let lo = all.clone().fold(f32::MAX, f32::min);
    let hi = all.fold(f32::MIN, f32::max);
    let (lo, hi) = if lo > hi { (0.0, 1.0) } else { (lo, hi) };
    let lo = if zero { lo.min(0.0) } else { lo };
    let t = nice_ticks(lo, hi, 4);
    let (a, b) = (*t.first().unwrap_or(&lo), *t.last().unwrap_or(&hi));
    (a, b, t)
}

impl ChartKind {
    /// Primitives for a `w`×`h` chart. Pure.
    pub fn layout(&self, w: f32, h: f32) -> Vec<Prim> {
        let mut out = Vec::new();
        match self {
            Self::Sparkline(v) => {
                let one = [v.clone()];
                let (lo, hi, _) = range(&one, false);
                let n = v.len();
                let pts: Vec<_> = v
                    .iter()
                    .enumerate()
                    .map(|(i, x)| {
                        let fx = if n > 1 {
                            i as f32 / (n - 1) as f32
                        } else {
                            0.5
                        };
                        (fx * w, h - norm(*x, lo, hi) * h)
                    })
                    .collect();
                if pts.len() > 1 {
                    let mut poly = smooth_points(&pts, 6);
                    out.push(Prim::Fill(
                        {
                            let mut p = poly.clone();
                            p.push((w, h));
                            p.push((0.0, h));
                            p
                        },
                        Ink::Series(0),
                        0.16,
                    ));
                    out.push(Prim::Line(std::mem::take(&mut poly), Ink::Series(0), 1.5));
                }
            }
            Self::Line(series) => {
                let (lo, hi, ticks) = range(series, false);
                for t in &ticks {
                    let y = h - norm(*t, lo, hi) * h;
                    out.push(Prim::Line(vec![(0.0, y), (w, y)], Ink::Grid, 1.0));
                }
                for (si, s) in series.iter().enumerate() {
                    let m = s.len();
                    let pts: Vec<_> = s
                        .iter()
                        .enumerate()
                        .map(|(i, v)| ((i as f32 + 0.5) / m as f32 * w, h - norm(*v, lo, hi) * h))
                        .collect();
                    if pts.len() > 1 {
                        out.push(Prim::Line(smooth_points(&pts, 8), Ink::Series(si), 2.0));
                    }
                }
            }
            Self::Bars(series) => {
                let (lo, hi, ticks) = range(series, true);
                for t in &ticks {
                    let y = h - norm(*t, lo, hi) * h;
                    out.push(Prim::Line(vec![(0.0, y), (w, y)], Ink::Grid, 1.0));
                }
                let groups = series.iter().map(Vec::len).max().unwrap_or(0);
                let k = series.len().max(1);
                let slot = w / groups.max(1) as f32;
                let bar = slot * 0.7 / k as f32;
                let zero_y = h - norm(0.0, lo, hi) * h;
                for (si, s) in series.iter().enumerate() {
                    for (gi, v) in s.iter().enumerate() {
                        let x = gi as f32 * slot + slot * 0.15 + si as f32 * bar;
                        let y = h - norm(*v, lo, hi) * h;
                        out.push(Prim::Rect(
                            x,
                            y.min(zero_y),
                            bar,
                            (zero_y - y).abs(),
                            Ink::Series(si),
                        ));
                    }
                }
            }
            Self::Gauge(f) => {
                let f = f.clamp(0.0, 1.0);
                let (c, r) = (w.min(h) / 2.0, w.min(h) / 2.0 - 6.0);
                let (a0, span) = (0.625, 0.75); // 225° start, 270° sweep, in turns
                out.push(Prim::Line(turn_arc(c, c, r, a0, a0 + span), Ink::Grid, 8.0));
                if f > 0.0 {
                    out.push(Prim::Line(
                        turn_arc(c, c, r, a0, a0 + span * f),
                        Ink::Series(0),
                        8.0,
                    ));
                }
            }
        }
        out
    }
}

struct Chart {
    kind: ChartKind,
    series: Vec<Color>,
    grid: Color,
}

impl<Message> canvas::Program<Message> for Chart {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &Renderer,
        _: &IcedBaseTheme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let ink = |i: Ink| match i {
            Ink::Series(n) => self.series[n % self.series.len()],
            Ink::Grid => self.grid,
        };
        let path = |pts: &[(f32, f32)], close: bool| {
            Path::new(|b| {
                for (i, (x, y)) in pts.iter().enumerate() {
                    let p = Point::new(*x, *y);
                    if i == 0 {
                        b.move_to(p);
                    } else {
                        b.line_to(p);
                    }
                }
                if close {
                    b.close();
                }
            })
        };
        for p in self.kind.layout(bounds.width, bounds.height) {
            match p {
                Prim::Line(pts, i, wd) => frame.stroke(
                    &path(&pts, false),
                    Stroke::default().with_color(ink(i)).with_width(wd),
                ),
                Prim::Fill(pts, i, a) => frame.fill(&path(&pts, true), Color { a, ..ink(i) }),
                Prim::Rect(x, y, w, h, i) => {
                    frame.fill(&Path::rectangle(Point::new(x, y), Size::new(w, h)), ink(i));
                }
            }
        }
        vec![frame.into_geometry()]
    }
}

impl IcedTheme {
    /// A chart of `kind`, `height` px tall and as wide as its container.
    pub fn chart<'a, Message: 'a>(&self, kind: ChartKind, height: f32) -> Element<'a, Message> {
        let p = &self.theme.palette;
        Canvas::new(Chart {
            kind,
            series: p.series.iter().map(|c| color(*c)).collect(),
            grid: color(p.border),
        })
        .width(Length::Fill)
        .height(Length::Fixed(height))
        .into()
    }

    /// A compact trend line (see [`ChartKind::Sparkline`]).
    pub fn sparkline<'a, Message: 'a>(
        &self,
        values: impl IntoIterator<Item = f32>,
    ) -> Element<'a, Message> {
        self.chart(ChartKind::Sparkline(values.into_iter().collect()), 36.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparkline_has_area_and_line_inside_the_box() {
        let p = ChartKind::Sparkline(vec![1.0, 3.0, 2.0, 5.0]).layout(100.0, 30.0);
        assert_eq!(p.len(), 2);
        for prim in &p {
            let pts = match prim {
                Prim::Line(p, ..) | Prim::Fill(p, ..) => p,
                Prim::Rect(..) => unreachable!(),
            };
            assert!(pts
                .iter()
                .all(|(x, y)| *x >= -0.5 && *x <= 100.5 && *y >= -2.0 && *y <= 32.0));
        }
    }

    #[test]
    fn bars_sit_on_the_zero_line() {
        let p = ChartKind::Bars(vec![vec![2.0, 4.0]]).layout(100.0, 100.0);
        let rects: Vec<_> = p.iter().filter(|p| matches!(p, Prim::Rect(..))).collect();
        assert_eq!(rects.len(), 2);
        if let (Prim::Rect(_, y0, _, h0, _), Prim::Rect(_, y1, _, h1, _)) = (rects[0], rects[1]) {
            assert!((y0 + h0 - (y1 + h1)).abs() < 1e-3);
            assert!(h1 > h0);
        }
    }

    #[test]
    fn gauge_arc_grows_with_the_value() {
        let len = |f| match &ChartKind::Gauge(f).layout(80.0, 80.0)[..] {
            [_, Prim::Line(p, ..)] => p.len(),
            _ => 0,
        };
        assert!(len(0.9) > len(0.3));
        assert_eq!(ChartKind::Gauge(0.0).layout(80.0, 80.0).len(), 1);
    }

    #[test]
    fn empty_data_draws_only_what_exists() {
        assert!(ChartKind::Line(vec![]).layout(50.0, 50.0).len() <= 5);
        assert!(ChartKind::Sparkline(vec![]).layout(50.0, 20.0).is_empty());
    }
}
