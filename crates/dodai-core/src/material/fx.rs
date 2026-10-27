//! Material-only motion mixins: state layers, ripple, container transform, fade-through and
//! shared axis, following the M3 motion spec. Pure and toolkit-independent.

use crate::dodai_motion::{Clip, Easing, Pose};

/// M3 emphasized easing.
const EMPHASIZED: Easing = Easing::CubicBezier(0.2, 0.0, 0.0, 1.0);

/// The overlay opacity M3 draws over a component for its interaction state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateLayer {
    Hover,
    Focus,
    Press,
    Dragged,
}

impl StateLayer {
    pub fn opacity(self) -> f32 {
        match self {
            StateLayer::Hover => 0.08,
            StateLayer::Focus | StateLayer::Press => 0.10,
            StateLayer::Dragged => 0.16,
        }
    }
}

/// Overlay opacity for the combination of states currently active (the strongest wins; M3
/// does not stack them).
pub fn state_layer(hover: bool, focus: bool, press: bool, dragged: bool) -> f32 {
    let mut o = 0.0f32;
    for (on, s) in [
        (hover, StateLayer::Hover),
        (focus, StateLayer::Focus),
        (press, StateLayer::Press),
        (dragged, StateLayer::Dragged),
    ] {
        if on {
            o = o.max(s.opacity());
        }
    }
    o
}

/// Radius a ripple needs to cover a `w` × `h` widget from a press at `origin`.
pub fn ripple_radius(origin: (f32, f32), w: f32, h: f32) -> f32 {
    let dx = origin.0.max(w - origin.0);
    let dy = origin.1.max(h - origin.1);
    (dx * dx + dy * dy).sqrt()
}

/// Press ripple: `(radius px, opacity)` over time. It grows to cover the widget while holding
/// at the pressed-state opacity, then fades.
pub fn ripple(origin: (f32, f32), w: f32, h: f32) -> Clip<(f32, f32)> {
    let r = ripple_radius(origin, w, h);
    let grow = Clip::ease(0.0, r, 0.45, EMPHASIZED);
    let alpha = Clip::keyframes(
        0.45 + 0.15,
        vec![
            (0.0, 0.0f32, Easing::Linear),
            (0.1, 0.12, Easing::EaseOut),
            (0.75, 0.12, Easing::Linear),
            (1.0, 0.0, Easing::EaseIn),
        ],
    );
    grow.zip(alpha)
}

/// Container transform: `(rect, content cross-fade)` where the rect `[x, y, w, h]` morphs from
/// `from` to `to` and the cross-fade goes `0..=1` (old content out, new content in).
pub fn container_transform(from: [f32; 4], to: [f32; 4]) -> Clip<([f32; 4], f32)> {
    Clip::ease(from, to, 0.3, EMPHASIZED).zip(Clip::ease(0.0, 1.0, 0.15, Easing::Linear).delay(0.1))
}

/// Fade through: the old content fades out, then the new one scales in from `92 %`. Returns
/// `(outgoing, incoming)` poses.
pub fn fade_through() -> Clip<(Pose, Pose)> {
    let out = Clip::ease(1.0, 0.0, 0.09, Easing::Linear).map(Pose::opacity);
    let inc = Clip::ease(0.0, 1.0, 0.21, EMPHASIZED)
        .zip(Clip::ease(0.92, 1.0, 0.21, EMPHASIZED))
        .map(|(o, s)| Pose {
            opacity: o,
            scale: s,
            ..Pose::IDENTITY
        })
        .delay(0.09);
    out.zip(inc)
}

/// Shared axis along x: both pages slide `distance` px while the old fades out and the new
/// fades in. `forward` moves left to right. Returns `(outgoing, incoming)` poses.
pub fn shared_axis(distance: f32, forward: bool) -> Clip<(Pose, Pose)> {
    let s = if forward { 1.0 } else { -1.0 };
    let out = Clip::ease(0.0, -s * distance, 0.3, EMPHASIZED)
        .zip(Clip::ease(1.0, 0.0, 0.09, Easing::Linear))
        .map(|(dx, o)| Pose {
            dx,
            opacity: o,
            ..Pose::IDENTITY
        });
    let inc = Clip::ease(s * distance, 0.0, 0.3, EMPHASIZED)
        .zip(Clip::ease(0.0, 1.0, 0.21, Easing::Linear).delay(0.09))
        .map(|(dx, o)| Pose {
            dx,
            opacity: o,
            ..Pose::IDENTITY
        });
    out.zip(inc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_layers_follow_the_spec_and_do_not_stack() {
        assert_eq!(state_layer(false, false, false, false), 0.0);
        assert_eq!(state_layer(true, false, false, false), 0.08);
        assert_eq!(state_layer(true, true, false, false), 0.10);
        assert_eq!(state_layer(true, true, true, true), 0.16);
    }

    #[test]
    fn ripple_covers_the_widget_and_fades_out() {
        let r = ripple_radius((0.0, 0.0), 300.0, 400.0);
        assert!((r - 500.0).abs() < 1e-3, "farthest corner");
        assert!((ripple_radius((150.0, 200.0), 300.0, 400.0) - 250.0).abs() < 1e-3);
        let c = ripple((10.0, 10.0), 100.0, 40.0);
        let (r0, a0) = c.at(0.0);
        assert!(r0.abs() < 1e-3 && a0.abs() < 1e-4);
        let (rm, am) = c.at(0.3);
        assert!(rm > 0.0 && (am - 0.12).abs() < 1e-3);
        let (re, ae) = c.at(c.duration());
        assert!(re >= ripple_radius((10.0, 10.0), 100.0, 40.0) - 1e-2 && ae.abs() < 1e-4);
    }

    #[test]
    fn container_transform_morphs_and_crossfades() {
        let c = container_transform([0.0, 0.0, 40.0, 40.0], [10.0, 20.0, 300.0, 500.0]);
        let (r0, f0) = c.at(0.0);
        assert!(
            r0.iter()
                .zip([0.0, 0.0, 40.0, 40.0])
                .all(|(a, b)| (a - b).abs() < 1e-3)
                && f0 == 0.0
        );
        let (r1, f1) = c.at(c.duration());
        assert!(
            r1.iter()
                .zip([10.0, 20.0, 300.0, 500.0])
                .all(|(a, b)| (a - b).abs() < 1e-3)
                && (f1 - 1.0).abs() < 1e-4
        );
        assert!(c.at(0.15).0[2] > 100.0, "emphasized is front-loaded");
    }

    #[test]
    fn fade_through_hands_over_in_order() {
        let c = fade_through();
        let (out, inc) = c.at(0.0);
        assert!((out.opacity - 1.0).abs() < 1e-4 && inc.opacity.abs() < 1e-4);
        let (out, inc) = c.at(0.09);
        assert!(
            out.opacity < 0.01 && inc.opacity < 0.01,
            "gap between: nothing overlaps"
        );
        let (out, inc) = c.at(c.duration());
        assert!(out.opacity.abs() < 1e-3 && inc.is_identity());
        assert!(c.at(0.1).1.scale < 1.0, "scales in");
    }

    #[test]
    fn shared_axis_direction() {
        let fwd = shared_axis(30.0, true);
        let (out, inc) = fwd.at(0.15);
        assert!(
            out.dx < 0.0 && inc.dx > 0.0,
            "forward: old leaves left, new arrives from the right"
        );
        let back = shared_axis(30.0, false);
        let (out, inc) = back.at(0.15);
        assert!(out.dx > 0.0 && inc.dx < 0.0);
        let (out, inc) = fwd.at(fwd.duration());
        assert!(out.opacity.abs() < 1e-3 && inc.is_identity());
    }
}
