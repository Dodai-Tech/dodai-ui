//! Physically based motion, in closed form.
//!
//! Every model here is an exact function of time (no per-frame integration), so it is
//! deterministic, stable at any frame rate and can be sampled, composed as a
//! [`Clip`](crate::Clip) or driven by the host's clock.

/// A damped spring (`mass * x'' + damping * x' + stiffness * x = 0`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

impl Spring {
    pub const fn new(stiffness: f32, damping: f32, mass: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass,
        }
    }
    /// Designer-friendly: `response` is the time in seconds of one natural period (lower is
    /// snappier); `bounce` is `0` for critically damped, towards `1` for very bouncy and
    /// negative for sluggish (over-damped).
    pub fn from_response(response: f32, bounce: f32) -> Self {
        let w = std::f32::consts::TAU / response.max(1e-3);
        let zeta = (1.0 - bounce).clamp(0.02, 4.0);
        Self::new(w * w, 2.0 * zeta * w, 1.0)
    }
    /// Natural angular frequency.
    pub fn omega(&self) -> f32 {
        (self.stiffness / self.mass.max(1e-6)).sqrt()
    }
    /// Damping ratio: `< 1` overshoots, `1` is critical, `> 1` creeps.
    pub fn zeta(&self) -> f32 {
        self.damping / (2.0 * (self.stiffness * self.mass).max(1e-12).sqrt())
    }
    /// Exact `(position, velocity)` `t` seconds after leaving `x0` with velocity `v0`
    /// towards `target`.
    pub fn at(&self, t: f32, x0: f32, v0: f32, target: f32) -> (f32, f32) {
        let t = t.max(0.0);
        let (w, z) = (self.omega(), self.zeta());
        let d = x0 - target;
        if (z - 1.0).abs() < 1e-4 {
            let e = (-w * t).exp();
            let b = v0 + w * d;
            (target + e * (d + b * t), e * (v0 - w * b * t))
        } else if z < 1.0 {
            let wd = w * (1.0 - z * z).sqrt();
            let e = (-z * w * t).exp();
            let (s, c) = (wd * t).sin_cos();
            let b = (v0 + z * w * d) / wd;
            (
                target + e * (d * c + b * s),
                e * ((b * wd - z * w * d) * c + (-d * wd - z * w * b) * s),
            )
        } else {
            let s = w * (z * z - 1.0).sqrt();
            let (r1, r2) = (-z * w + s, -z * w - s);
            let c2 = (v0 - r1 * d) / (r2 - r1);
            let c1 = d - c2;
            let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
            (target + c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        }
    }
    /// Seconds until the spring stays within `eps` of the target (position and velocity),
    /// capped at 10 s.
    pub fn settle_time(&self, x0: f32, v0: f32, target: f32, eps: f32) -> f32 {
        let step = 1.0 / 240.0;
        let mut last_out = 0.0;
        for i in 0..2400 {
            let t = i as f32 * step;
            let (x, v) = self.at(t, x0, v0, target);
            if (x - target).abs() > eps || v.abs() > eps {
                last_out = t + step;
            }
        }
        last_out
    }
}

/// A spring heading for a target, retargetable mid-flight without losing velocity.
/// Like [`Tween`](crate::Tween) it takes the time as a parameter.
#[derive(Clone, Copy, Debug)]
pub struct SpringState {
    spring: Spring,
    x0: f32,
    v0: f32,
    target: f32,
    t0: f32,
}

impl SpringState {
    pub fn new(spring: Spring, value: f32) -> Self {
        Self {
            spring,
            x0: value,
            v0: 0.0,
            target: value,
            t0: 0.0,
        }
    }
    pub fn value(&self, now: f32) -> f32 {
        self.spring
            .at(now - self.t0, self.x0, self.v0, self.target)
            .0
    }
    pub fn velocity(&self, now: f32) -> f32 {
        self.spring
            .at(now - self.t0, self.x0, self.v0, self.target)
            .1
    }
    pub fn target(&self) -> f32 {
        self.target
    }
    /// Head for `target` from the current position *and* velocity (no-op if unchanged).
    pub fn retarget(&mut self, target: f32, now: f32) {
        if (target - self.target).abs() > f32::EPSILON {
            let (x, v) = self.spring.at(now - self.t0, self.x0, self.v0, self.target);
            *self = Self {
                x0: x,
                v0: v,
                target,
                t0: now,
                ..*self
            };
        }
    }
    /// Kick the spring: add `dv` to its velocity now (a tap, a flick).
    pub fn impulse(&mut self, dv: f32, now: f32) {
        let (x, v) = self.spring.at(now - self.t0, self.x0, self.v0, self.target);
        self.x0 = x;
        self.v0 = v + dv;
        self.t0 = now;
    }
    pub fn settled(&self, now: f32, eps: f32) -> bool {
        let (x, v) = self.spring.at(now - self.t0, self.x0, self.v0, self.target);
        (x - self.target).abs() <= eps && v.abs() <= eps
    }
}

/// Inertia: a fling that slows under exponential friction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decay {
    pub x0: f32,
    pub v0: f32,
    /// Friction rate in 1/s (higher stops sooner). `~4` feels like a scroll view.
    pub friction: f32,
}

impl Decay {
    pub fn new(x0: f32, v0: f32, friction: f32) -> Self {
        Self { x0, v0, friction }
    }
    pub fn position(&self, t: f32) -> f32 {
        let k = self.friction.max(1e-3);
        self.x0 + self.v0 * (1.0 - (-k * t.max(0.0)).exp()) / k
    }
    pub fn velocity(&self, t: f32) -> f32 {
        self.v0 * (-self.friction.max(1e-3) * t.max(0.0)).exp()
    }
    /// Where it comes to rest.
    pub fn rest(&self) -> f32 {
        self.x0 + self.v0 / self.friction.max(1e-3)
    }
    /// Seconds until the speed falls below `min_speed`.
    pub fn duration(&self, min_speed: f32) -> f32 {
        let k = self.friction.max(1e-3);
        if self.v0.abs() <= min_speed {
            0.0
        } else {
            (self.v0.abs() / min_speed.max(1e-3)).ln() / k
        }
    }
}

/// The value in `points` nearest to `x`, or `x` if there are none. Pair with
/// [`Decay::rest`] to land a fling on pages or snap points.
pub fn nearest(points: &[f32], x: f32) -> f32 {
    let mut best = x;
    let mut best_d = f32::INFINITY;
    for &p in points {
        let d = (p - x).abs();
        if d < best_d {
            best_d = d;
            best = p;
        }
    }
    best
}

/// A ball dropped from `height` onto the ground (`0`), losing energy at each bounce.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounce {
    pub height: f32,
    pub gravity: f32,
    /// Speed kept after each impact, `0..1` (`0.5` is a tennis ball).
    pub restitution: f32,
}

impl Bounce {
    pub fn new(height: f32, gravity: f32, restitution: f32) -> Self {
        Self {
            height,
            gravity,
            restitution: restitution.clamp(0.0, 0.99),
        }
    }
    /// Height above the ground `t` seconds after release.
    pub fn at(&self, t: f32) -> f32 {
        let g = self.gravity.max(1e-3);
        let fall = (2.0 * self.height / g).sqrt();
        let mut t = t.max(0.0);
        if t < fall {
            return self.height - 0.5 * g * t * t;
        }
        t -= fall;
        let mut v = g * fall * self.restitution;
        for _ in 0..64 {
            let flight = 2.0 * v / g;
            if flight < 1e-3 {
                return 0.0;
            }
            if t < flight {
                return v * t - 0.5 * g * t * t;
            }
            t -= flight;
            v *= self.restitution;
        }
        0.0
    }
    /// Time of the last visible bounce (peak below `0.5 px`).
    pub fn duration(&self) -> f32 {
        let g = self.gravity.max(1e-3);
        let mut total = (2.0 * self.height / g).sqrt();
        let mut v = g * total * self.restitution;
        for _ in 0..64 {
            if v * v / (2.0 * g) < 0.5 {
                break;
            }
            total += 2.0 * v / g;
            v *= self.restitution;
        }
        total
    }
}

/// Resistance past an edge: `offset` px of drag past a limit of `limit` px moves the content
/// by less, approaching `limit` asymptotically (the iOS overscroll curve). Sign-preserving.
pub fn rubber_band(offset: f32, limit: f32, resistance: f32) -> f32 {
    let limit = limit.max(1e-3);
    let x = offset.abs();
    let y = (1.0 - 1.0 / (x * resistance / limit + 1.0)) * limit;
    y.copysign(offset)
}

/// Frame-rate independent smoothing: move `current` towards `target` at `rate` (1/s).
/// `damp(a, b, rate, dt)` called every frame with the frame time follows the same curve at
/// 30 or 240 fps (unlike `lerp(a, b, 0.1)`).
pub fn damp(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    target + (current - target) * (-rate * dt.max(0.0)).exp()
}

/// A decaying oscillation: error shake, doorbell wobble.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shake {
    pub frequency: f32,
    /// Decay rate in 1/s.
    pub decay: f32,
}

impl Shake {
    pub fn new(frequency: f32, decay: f32) -> Self {
        Self { frequency, decay }
    }
    /// `-1..=1`, starting at `0`.
    pub fn at(&self, t: f32) -> f32 {
        let t = t.max(0.0);
        (-self.decay * t).exp() * (std::f32::consts::TAU * self.frequency * t).sin()
    }
    pub fn duration(&self) -> f32 {
        (100.0f32).ln() / self.decay.max(1e-3)
    }
}
