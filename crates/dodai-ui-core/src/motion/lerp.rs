//! The [`Lerp`] trait: anything that can be blended.

/// Linear blend between two values. `t = 0` is `self`, `t = 1` is `other`; `t` may leave
/// `0..=1` (springs overshoot), so implementations must not clamp.
pub trait Lerp: Sized {
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(&self, o: &f32, t: f32) -> f32 {
        self + (o - self) * t
    }
}

impl Lerp for (f32, f32) {
    fn lerp(&self, o: &Self, t: f32) -> Self {
        (self.0.lerp(&o.0, t), self.1.lerp(&o.1, t))
    }
}

impl Lerp for (f32, f32, f32) {
    fn lerp(&self, o: &Self, t: f32) -> Self {
        (
            self.0.lerp(&o.0, t),
            self.1.lerp(&o.1, t),
            self.2.lerp(&o.2, t),
        )
    }
}

impl<const N: usize> Lerp for [f32; N] {
    fn lerp(&self, o: &Self, t: f32) -> Self {
        let mut out = *self;
        for i in 0..N {
            out[i] = self[i].lerp(&o[i], t);
        }
        out
    }
}
