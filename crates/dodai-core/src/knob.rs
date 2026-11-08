//! The rotary knob: sweep geometry, drag maths and resolved colours, shared by every toolkit.
//!
//! Drag up or right to increase, down or left to decrease; the sweep is 270° starting at the
//! bottom-left. Colours come from the toggle and progress tokens and the glow from the effects,
//! so a knob follows every theme without knowing which one is active.

use crate::{Color, ControlState, Theme};

/// Start of the sweep, degrees clockwise from +x (screen space): bottom-left.
pub const KNOB_START_DEG: f32 = 135.0;
/// Total sweep in degrees.
pub const KNOB_SWEEP_DEG: f32 = 270.0;
/// Pixels of drag that cover the whole range.
pub const KNOB_DRAG_PX: f32 = 160.0;

/// Angle in radians (screen space, clockwise from +x) for a 0..1 fraction.
pub fn knob_angle(frac: f32) -> f32 {
    (KNOB_START_DEG + frac.clamp(0.0, 1.0) * KNOB_SWEEP_DEG).to_radians()
}

/// New value after dragging `dx`/`dy` px (screen space; up and right increase) from `start`.
/// `step` snaps to multiples from `min`.
pub fn knob_value(start: f32, dx: f32, dy: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let mut v = start + (dx - dy) / KNOB_DRAG_PX * (max - min);
    if let Some(s) = step.filter(|s| *s > 0.0) {
        v = min + ((v - min) / s).round() * s;
    }
    v.clamp(min.min(max), max.max(min))
}

/// Points of the arc from fraction `a` to `b` on a circle of radius `r` centred on `(cx, cy)`.
pub fn arc_points(cx: f32, cy: f32, r: f32, a: f32, b: f32) -> Vec<(f32, f32)> {
    let n = (((b - a).abs() * KNOB_SWEEP_DEG / 6.0).ceil() as usize).max(2);
    (0..=n)
        .map(|i| {
            let th = knob_angle(a + (b - a) * i as f32 / n as f32);
            (cx + r * th.cos(), cy + r * th.sin())
        })
        .collect()
}

/// Where the parts of a knob of diameter `size` sit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobGeometry {
    /// Stroke width of the track and value arcs.
    pub ring: f32,
    /// Radius of the arcs' centre line.
    pub r_ring: f32,
    /// Radius of the body disc.
    pub r_body: f32,
    /// Stroke width of the indicator tick.
    pub tick: f32,
}

impl KnobGeometry {
    /// The geometry of a knob of diameter `size`.
    pub fn new(size: f32) -> Self {
        let ring = (size * 0.09).clamp(3.0, 8.0);
        let r_ring = size / 2.0 - ring / 2.0 - 1.0;
        Self {
            ring,
            r_ring,
            r_body: r_ring - ring / 2.0 - size * 0.06,
            tick: (size * 0.05).max(2.0),
        }
    }
}

/// Resolved colours of a knob.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KnobStyle {
    /// Body disc fill.
    pub body: Color,
    /// Body disc edge.
    pub body_edge: Color,
    /// Empty part of the arc.
    pub track: Color,
    /// Value arc.
    pub fill: Color,
    /// Indicator tick.
    pub tick: Color,
    /// Glow colour and radius, when the design glows and the knob is enabled.
    pub glow: Option<(Color, f32)>,
    /// Border width of the body.
    pub border_width: f32,
}

impl Theme {
    /// The colours of a knob.
    pub fn knob(&self, disabled: bool) -> KnobStyle {
        let state = if disabled {
            ControlState::Disabled
        } else {
            ControlState::Idle
        };
        let (on, off, prog) = (
            self.toggle(true, state),
            self.toggle(false, state),
            self.progress(None),
        );
        KnobStyle {
            body: off.track,
            body_edge: off.border,
            track: prog.track,
            fill: prog.fill,
            tick: on.thumb,
            glow: (!disabled && self.effects.glow_enabled())
                .then(|| (self.effects.glow_color(), self.effects.glow_radius())),
            border_width: self.shape.border_width.max(1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dragging_up_or_right_increases_and_down_or_left_decreases() {
        assert!(knob_value(0.5, 0.0, -16.0, 0.0, 1.0, None) > 0.5);
        assert!(knob_value(0.5, 16.0, 0.0, 0.0, 1.0, None) > 0.5);
        assert!(knob_value(0.5, 0.0, 16.0, 0.0, 1.0, None) < 0.5);
        assert!(knob_value(0.5, -16.0, 0.0, 0.0, 1.0, None) < 0.5);
    }

    #[test]
    fn a_full_drag_covers_the_range_and_clamps() {
        assert_eq!(knob_value(0.0, KNOB_DRAG_PX, 0.0, 0.0, 1.0, None), 1.0);
        assert_eq!(knob_value(0.5, 1e6, 0.0, 0.0, 1.0, None), 1.0);
        assert_eq!(knob_value(0.5, -1e6, 0.0, 0.0, 1.0, None), 0.0);
        assert_eq!(knob_value(2.0, 0.0, -KNOB_DRAG_PX, 0.0, 10.0, None), 10.0);
    }

    #[test]
    fn steps_snap_from_the_minimum() {
        let v = knob_value(0.0, 0.26 * KNOB_DRAG_PX, 0.0, 0.0, 1.0, Some(0.25));
        assert_eq!(v, 0.25);
        let v = knob_value(1.0, 0.1 * KNOB_DRAG_PX, 0.0, 1.0, 5.0, Some(1.0));
        assert_eq!(v, 1.0);
    }

    #[test]
    fn the_sweep_runs_clockwise_from_bottom_left_over_the_top() {
        let s = knob_angle(0.0);
        assert!(
            (s.cos() + std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3
                && (s.sin() - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3
        );
        let top = knob_angle(0.5);
        assert!(top.cos().abs() < 1e-3 && (top.sin() + 1.0).abs() < 1e-3);
        let e = knob_angle(1.0);
        assert!(
            (e.cos() - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3
                && (e.sin() - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-3
        );
        assert_eq!(knob_angle(-3.0), knob_angle(0.0));
    }

    #[test]
    fn arcs_start_and_end_on_the_circle() {
        let p = arc_points(10.0, 10.0, 5.0, 0.0, 1.0);
        assert!(p.len() > 20);
        for (x, y) in &p {
            assert!((((x - 10.0).powi(2) + (y - 10.0).powi(2)).sqrt() - 5.0).abs() < 1e-3);
        }
    }

    #[test]
    fn geometry_nests_the_body_inside_the_ring() {
        for size in [40.0, 72.0, 96.0, 200.0] {
            let g = KnobGeometry::new(size);
            assert!(g.r_body > 0.0 && g.r_body < g.r_ring && g.r_ring + g.ring / 2.0 <= size / 2.0);
        }
    }

    #[test]
    fn the_style_dims_and_loses_its_glow_when_disabled() {
        for t in crate::themes::all() {
            assert!(t.knob(true).glow.is_none(), "{}", t.id());
            let k = t.knob(false);
            assert!(k.border_width >= 1.0);
        }
    }
}
