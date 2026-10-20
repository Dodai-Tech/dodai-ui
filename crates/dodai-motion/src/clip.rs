//! [`Clip`]: a value over time, composable.

use crate::{Bounce, Decay, Easing, Lerp, Motion, Shake, Spring};
use std::sync::Arc;

/// A value of type `T` as a function of seconds, with a duration. After the end it holds its
/// final value. Clips are cheap to clone (`Arc`) and `Send + Sync`.
///
/// Compose them: [`then`](Clip::then) (sequence), [`zip`](Clip::zip) (parallel),
/// [`delay`](Clip::delay), [`repeat`](Clip::repeat), [`ping_pong`](Clip::ping_pong),
/// [`speed`](Clip::speed), [`reverse`](Clip::reverse), [`map`](Clip::map),
/// [`stagger`](Clip::stagger). Springs, decay, bounce and keyframes are all clips, so they mix
/// freely.
pub struct Clip<T> {
    duration: f32,
    f: Arc<dyn Fn(f32) -> T + Send + Sync>,
}

impl<T> Clone for Clip<T> {
    fn clone(&self) -> Self {
        Self {
            duration: self.duration,
            f: self.f.clone(),
        }
    }
}

impl<T: 'static> Clip<T> {
    /// A clip from any function of time (`0..=duration`).
    pub fn from_fn(duration: f32, f: impl Fn(f32) -> T + Send + Sync + 'static) -> Self {
        Self {
            duration: duration.max(0.0),
            f: Arc::new(f),
        }
    }
    /// Length in seconds (`INFINITY` for [`forever`](Clip::forever)).
    pub fn duration(&self) -> f32 {
        self.duration
    }
    /// The value `t` seconds in (clamped to the clip's span).
    pub fn at(&self, t: f32) -> T {
        (self.f)(t.clamp(0.0, self.duration))
    }
    pub fn done(&self, t: f32) -> bool {
        t >= self.duration
    }
    /// Play `self`, then `next`.
    pub fn then(self, next: Clip<T>) -> Clip<T> {
        let d = self.duration;
        Clip::from_fn(d + next.duration, move |t| {
            if t < d {
                self.at(t)
            } else {
                next.at(t - d)
            }
        })
    }
    /// Wait `secs` (holding the first frame) before playing.
    pub fn delay(self, secs: f32) -> Clip<T> {
        let s = secs.max(0.0);
        Clip::from_fn(self.duration + s, move |t| self.at(t - s))
    }
    /// Play `n` times in a row.
    pub fn repeat(self, n: u32) -> Clip<T> {
        let d = self.duration.max(1e-6);
        let n = n.max(1);
        Clip::from_fn(d * n as f32, move |t| {
            if t >= d * n as f32 {
                self.at(d)
            } else {
                self.at(t % d)
            }
        })
    }
    /// Play over and over, never finishing.
    pub fn forever(self) -> Clip<T> {
        let d = self.duration.max(1e-6);
        Clip::from_fn(f32::INFINITY, move |t| self.at(t % d))
    }
    /// Forwards then backwards.
    pub fn ping_pong(self) -> Clip<T> {
        let d = self.duration;
        Clip::from_fn(2.0 * d, move |t| {
            self.at(if t < d { t } else { 2.0 * d - t })
        })
    }
    pub fn reverse(self) -> Clip<T> {
        let d = self.duration;
        Clip::from_fn(d, move |t| self.at(d - t))
    }
    /// Play `k` times faster (`k > 1`) or slower (`k < 1`).
    pub fn speed(self, k: f32) -> Clip<T> {
        let k = k.max(1e-3);
        Clip::from_fn(self.duration / k, move |t| self.at(t * k))
    }
    /// Transform every value.
    pub fn map<U: 'static>(self, f: impl Fn(T) -> U + Send + Sync + 'static) -> Clip<U> {
        Clip::from_fn(self.duration, move |t| f(self.at(t)))
    }
    /// Play `self` and `other` at the same time, combining their values. Lasts as long as the
    /// longer one; the shorter holds its last value.
    pub fn zip_with<U: 'static, V: 'static>(
        self,
        other: Clip<U>,
        f: impl Fn(T, U) -> V + Send + Sync + 'static,
    ) -> Clip<V> {
        Clip::from_fn(self.duration.max(other.duration), move |t| {
            f(self.at(t), other.at(t))
        })
    }
    /// Parallel composition as a pair.
    pub fn zip<U: 'static>(self, other: Clip<U>) -> Clip<(T, U)> {
        self.zip_with(other, |a, b| (a, b))
    }
    /// `n` clips, clip `i` delayed by `i * gap` seconds: the classic cascade.
    pub fn stagger(n: usize, gap: f32, make: impl Fn(usize) -> Clip<T>) -> Vec<Clip<T>> {
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            v.push(make(i).delay(gap * i as f32));
        }
        v
    }
}

impl<T: Clone + Send + Sync + 'static> Clip<T> {
    /// A value that never changes.
    pub fn constant(v: T) -> Clip<T> {
        Clip::from_fn(0.0, move |_| v.clone())
    }
}

impl<T: Lerp + Clone + Send + Sync + 'static> Clip<T> {
    /// Ease from `from` to `to` with a [`Motion`]'s duration and easing.
    pub fn tween(from: T, to: T, motion: Motion) -> Clip<T> {
        let d = motion.duration.as_secs_f32().max(1e-3);
        Clip::from_fn(d, move |t| from.lerp(&to, motion.eased(t / d)))
    }
    /// Ease over `secs` seconds with `easing`.
    pub fn ease(from: T, to: T, secs: f32, easing: Easing) -> Clip<T> {
        let d = secs.max(1e-3);
        Clip::from_fn(d, move |t| from.lerp(&to, easing.apply(t / d)))
    }
    /// Keyframes: `(time in 0..=1, value, easing into this frame)`, spread over `secs`.
    /// Frames should be sorted by time; the first frame's easing is ignored.
    pub fn keyframes(secs: f32, frames: Vec<(f32, T, Easing)>) -> Clip<T> {
        let d = secs.max(1e-3);
        Clip::from_fn(d, move |t| {
            let p = t / d;
            let n = frames.len();
            if n == 0 {
                panic!("keyframes needs at least one frame");
            }
            if p <= frames[0].0 {
                return frames[0].1.clone();
            }
            for i in 1..n {
                if p <= frames[i].0 {
                    let (t0, a, _) = &frames[i - 1];
                    let (t1, b, e) = &frames[i];
                    let local = (p - t0) / (t1 - t0).max(1e-6);
                    return a.lerp(b, e.apply(local));
                }
            }
            frames[n - 1].1.clone()
        })
    }
}

impl Clip<f32> {
    /// A spring released from `from` with velocity `v0` towards `to`.
    pub fn spring(spring: Spring, from: f32, to: f32, v0: f32) -> Clip<f32> {
        let d = spring.settle_time(from, v0, to, 0.002);
        Clip::from_fn(d, move |t| spring.at(t, from, v0, to).0)
    }
    /// A fling that coasts to rest.
    pub fn decay(decay: Decay) -> Clip<f32> {
        let d = decay.duration(0.5);
        Clip::from_fn(d, move |t| decay.position(t))
    }
    /// A dropped ball: height above the ground.
    pub fn bounce(b: Bounce) -> Clip<f32> {
        Clip::from_fn(b.duration(), move |t| b.at(t))
    }
    /// A decaying oscillation.
    pub fn shake(s: Shake, amplitude: f32) -> Clip<f32> {
        Clip::from_fn(s.duration(), move |t| s.at(t) * amplitude)
    }
}

impl Clip<crate::Pose> {
    /// This clip for reduced-motion users (see [`Pose::reduced`](crate::Pose::reduced)).
    pub fn reduced(self) -> Clip<crate::Pose> {
        self.map(|p| p.reduced())
    }
}
