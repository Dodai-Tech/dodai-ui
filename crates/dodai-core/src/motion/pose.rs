//! [`Pose`]: the one thing an adapter has to apply.

use crate::motion::Lerp;

/// How a widget is drawn right now, relative to its resting state.
///
/// Every mixin produces a `Pose`; poses [`layer`](Pose::layer) on top of each other, so an
/// adapter applies a single pose per widget however many mixins are stacked.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    /// Multiplies the widget's opacity.
    pub opacity: f32,
    /// Translation in logical pixels.
    pub dx: f32,
    pub dy: f32,
    /// Uniform scale about the centre.
    pub scale: f32,
    /// Rotation in degrees, clockwise.
    pub rotate: f32,
    /// Extra glow / highlight strength `0..=1` (the theme decides what that looks like).
    pub glow: f32,
}

impl Pose {
    /// Resting state: nothing changes.
    pub const IDENTITY: Pose = Pose {
        opacity: 1.0,
        dx: 0.0,
        dy: 0.0,
        scale: 1.0,
        rotate: 0.0,
        glow: 0.0,
    };
    pub const fn opacity(opacity: f32) -> Pose {
        Pose {
            opacity,
            ..Pose::IDENTITY
        }
    }
    pub const fn offset(dx: f32, dy: f32) -> Pose {
        Pose {
            dx,
            dy,
            ..Pose::IDENTITY
        }
    }
    pub const fn scaled(scale: f32) -> Pose {
        Pose {
            scale,
            ..Pose::IDENTITY
        }
    }
    pub const fn rotated(rotate: f32) -> Pose {
        Pose {
            rotate,
            ..Pose::IDENTITY
        }
    }
    pub const fn glowing(glow: f32) -> Pose {
        Pose {
            glow,
            ..Pose::IDENTITY
        }
    }
    /// `self` with `top` applied on top: opacity and scale multiply, offsets and rotation
    /// add, glow takes the stronger of the two.
    pub fn layer(self, top: Pose) -> Pose {
        Pose {
            opacity: self.opacity * top.opacity,
            dx: self.dx + top.dx,
            dy: self.dy + top.dy,
            scale: self.scale * top.scale,
            rotate: self.rotate + top.rotate,
            glow: self.glow.max(top.glow),
        }
    }
    /// The pose for users who asked for reduced motion: nothing moves, scales or spins; only
    /// opacity and glow (state feedback) remain.
    pub fn reduced(self) -> Pose {
        Pose {
            dx: 0.0,
            dy: 0.0,
            scale: 1.0,
            rotate: 0.0,
            ..self
        }
    }
    /// Is this (nearly) the resting state?
    pub fn is_identity(&self) -> bool {
        let e = 1e-3;
        (self.opacity - 1.0).abs() < e
            && self.dx.abs() < e
            && self.dy.abs() < e
            && (self.scale - 1.0).abs() < e
            && self.rotate.abs() < e
            && self.glow.abs() < e
    }
}

impl Default for Pose {
    fn default() -> Self {
        Pose::IDENTITY
    }
}

impl Lerp for Pose {
    fn lerp(&self, o: &Pose, t: f32) -> Pose {
        Pose {
            opacity: self.opacity.lerp(&o.opacity, t),
            dx: self.dx.lerp(&o.dx, t),
            dy: self.dy.lerp(&o.dy, t),
            scale: self.scale.lerp(&o.scale, t),
            rotate: self.rotate.lerp(&o.rotate, t),
            glow: self.glow.lerp(&o.glow, t),
        }
    }
}
