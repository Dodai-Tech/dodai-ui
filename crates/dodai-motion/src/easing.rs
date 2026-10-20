//! Easing functions.

/// Timing curve mapping linear progress `0..1` to eased progress `0..1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    /// Slow start, accelerates.
    EaseIn,
    /// Fast start, decelerates.
    EaseOut,
    /// Accelerates then decelerates.
    EaseInOut,
    /// CSS-style `cubic-bezier(x1, y1, x2, y2)`.
    CubicBezier(f32, f32, f32, f32),
    /// Damped spring (mass 1): `stiffness` and `damping` as in physics
    /// engines. Under-damped springs overshoot `1.0`; entry effects clamp the
    /// overshoot, value-in animations (charts, sliders) show it.
    Spring {
        stiffness: f32,
        damping: f32,
    },
    /// Ball drop: falls to the target and bounces to rest.
    Bounce,
    /// Decaying wobble around the target (rubber band).
    Elastic,
    /// Overshoots the target by roughly `overshoot * 10%` and eases back (`1.7` ≈ 10%).
    Back(f32),
    /// Pulls back first, then accelerates past the start (windup).
    Anticipate(f32),
}

impl Easing {
    /// Bouncy spring preset (overshoots about 15%).
    pub const SPRING: Easing = Easing::Spring {
        stiffness: 170.0,
        damping: 12.0,
    };
    /// Fast and tight: settles quickly with a small overshoot.
    pub const SPRING_STIFF: Easing = Easing::Spring {
        stiffness: 400.0,
        damping: 26.0,
    };
    /// Slow and loose: large, lingering wobble.
    pub const SPRING_WOBBLY: Easing = Easing::Spring {
        stiffness: 120.0,
        damping: 6.0,
    };
    /// Heavily damped: no overshoot, soft landing.
    pub const SPRING_GENTLE: Easing = Easing::Spring {
        stiffness: 100.0,
        damping: 20.0,
    };

    /// Eased progress for linear `t` (clamped to `0..=1`). Pure.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => bezier(0.42, 0.0, 1.0, 1.0, t),
            Easing::EaseOut => bezier(0.0, 0.0, 0.58, 1.0, t),
            Easing::EaseInOut => bezier(0.42, 0.0, 0.58, 1.0, t),
            Easing::CubicBezier(a, b, c, d) => bezier(a, b, c, d, t),
            Easing::Spring { stiffness, damping } => spring(stiffness, damping, t),
            Easing::Bounce => bounce(t),
            Easing::Elastic => elastic(t),
            Easing::Back(o) => {
                let u = t - 1.0;
                1.0 + u * u * ((o + 1.0) * u + o)
            }
            Easing::Anticipate(o) => t * t * ((o + 1.0) * t - o),
        }
    }
}

/// Closed-form damped spring step response at time `t` (seconds), snapped to
/// exactly `1.0` at `t == 1`.
pub(crate) fn spring(k: f32, c: f32, t: f32) -> f32 {
    let k = k.max(1.0);
    let w0 = k.sqrt();
    let z = c.max(0.0) / (2.0 * w0);
    let x = if z < 1.0 {
        let wd = w0 * (1.0 - z * z).sqrt();
        let e = (-z * w0 * t).exp();
        1.0 - e * ((wd * t).cos() + (z * w0 / wd) * (wd * t).sin())
    } else {
        let e = (-w0 * t).exp();
        1.0 - e * (1.0 + w0 * t)
    };
    // Blend the last 5% to the target so the curve ends exactly at 1.
    let snap = ((t - 0.95) / 0.05).clamp(0.0, 1.0);
    x + (1.0 - x) * snap
}

/// Ball-drop bounce (easeOutBounce). Pure.
fn bounce(t: f32) -> f32 {
    let (n, d) = (7.5625, 2.75);
    if t < 1.0 / d {
        n * t * t
    } else if t < 2.0 / d {
        let t = t - 1.5 / d;
        n * t * t + 0.75
    } else if t < 2.5 / d {
        let t = t - 2.25 / d;
        n * t * t + 0.9375
    } else {
        let t = t - 2.625 / d;
        n * t * t + 0.984375
    }
}

/// Elastic ease-out: decaying sine around the target. Pure.
fn elastic(t: f32) -> f32 {
    if t <= 0.0 || t >= 1.0 {
        return t;
    }
    let c = std::f32::consts::TAU / 3.0;
    2f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * c).sin() + 1.0
}

/// Solve a CSS cubic-bezier for `y` at time `x` by bisection.
fn bezier(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    let at = |s: f32, a: f32, b: f32| {
        let u = 1.0 - s;
        3.0 * u * u * s * a + 3.0 * u * s * s * b + s * s * s
    };
    let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
    for _ in 0..24 {
        let mid = (lo + hi) / 2.0;
        if at(mid, x1.clamp(0.0, 1.0), x2.clamp(0.0, 1.0)) < x {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    at((lo + hi) / 2.0, y1, y2)
}
