//! The drawing helpers instruments share with charts: geometry from `dodai-charts-core` and the
//! glow from `dodai-gpui`.

pub(crate) use dodai_charts_core::math::circle_points;
pub(crate) use dodai_charts_core::{fmt_num, norm, polar, turn_arc};
pub(crate) use dodai_gpui::plot::{glow_stroke, rect, ChartGlow};

use dodai_gpui::paint::{fill_circle, scale_alpha, stroke_batch};
use gpui::{Hsla, Pixels, Point, Window};

/// Open lines gathered by colour and width, painted as one path per group: a ring of 240
/// ticks in two shades costs two draws instead of 240.
#[derive(Default)]
pub(crate) struct Strokes {
    groups: Vec<StrokeGroup>,
}

/// One colour and width of [`Strokes`], with its lines.
type StrokeGroup = (Hsla, f32, Vec<Vec<(f32, f32)>>);

impl Strokes {
    /// Add the line `p`, `width` wide, in `colour`.
    pub(crate) fn add(&mut self, p: Vec<(f32, f32)>, width: f32, colour: Hsla) {
        match self
            .groups
            .iter_mut()
            .find(|g| g.0 == colour && g.1 == width)
        {
            Some(g) => g.2.push(p),
            None => self.groups.push((colour, width, vec![p])),
        }
    }

    /// Paint every group, in the order its first line was added.
    pub(crate) fn paint(self, w: &mut Window, o: Point<Pixels>) {
        for (colour, width, lines) in self.groups {
            stroke_batch(w, o, &lines, width, colour, false);
        }
    }

    /// How many paths [`Strokes::paint`] draws. Pure.
    #[cfg(test)]
    fn paths(&self) -> usize {
        self.groups.len()
    }
}

/// Paint a sunburst finish on a disc of radius `r` round `(cx, cy)`: faint rays round
/// the centre, as on a brushed dial.
pub(crate) fn sunburst(w: &mut Window, o: Point<Pixels>, (cx, cy): (f32, f32), r: f32, ink: Hsla) {
    let hair = (r * 0.004).max(0.5);
    // One path for all the rays: one tessellation and one draw instead of 180.
    let rays: Vec<Vec<(f32, f32)>> = (0..180)
        .map(|i| {
            let a = i as f32 / 180.0;
            vec![polar(cx, cy, r * 0.3, a), polar(cx, cy, r * 0.98, a)]
        })
        .collect();
    stroke_batch(w, o, &rays, hair, scale_alpha(ink, 0.025), false);
}

/// Paint a polished hub of radius `r` at `(cx, cy)`: a bright cap, a ring of the face colour
/// and a centre in the hand's colour.
pub(crate) fn hub(
    w: &mut Window,
    o: Point<Pixels>,
    (cx, cy): (f32, f32),
    r: f32,
    (cap, face, hand): (Hsla, Hsla, Hsla),
) {
    fill_circle(w, o, (cx, cy), r, cap);
    fill_circle(w, o, (cx, cy), r * 0.72, face);
    fill_circle(w, o, (cx, cy), r * 0.32, hand);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strokes_group_by_colour_and_width() {
        let (a, b) = (
            gpui::hsla(0.0, 0.0, 1.0, 1.0),
            gpui::hsla(0.0, 0.0, 1.0, 0.5),
        );
        let mut s = Strokes::default();
        for i in 0..240 {
            s.add(
                vec![(0.0, 0.0), (1.0, 1.0)],
                1.0,
                if i % 20 == 0 { a } else { b },
            );
        }
        assert_eq!(s.paths(), 2);
        s.add(vec![(0.0, 0.0), (1.0, 1.0)], 2.0, a);
        assert_eq!(s.paths(), 3);
    }
}
