//! Composable progress curves, stagger and interpolation.

use crate::motion::Easing;
use std::sync::Arc;

/// Composable progress curve `0..1 -> 0..1`. Cheap to clone; build from an
/// [`Easing`] or closure, then combine: [`window`](Curve::window) (stagger /
/// delay), [`reverse`](Curve::reverse), [`ping_pong`](Curve::ping_pong),
/// [`then`](Curve::then), [`pipe`](Curve::pipe).
#[derive(Clone)]
pub struct Curve(Arc<dyn Fn(f32) -> f32 + Send + Sync>);

impl From<Easing> for Curve {
    fn from(e: Easing) -> Self {
        Curve(Arc::new(move |t| e.apply(t)))
    }
}

impl Curve {
    pub fn new(f: impl Fn(f32) -> f32 + Send + Sync + 'static) -> Self {
        Curve(Arc::new(f))
    }
    pub fn apply(&self, t: f32) -> f32 {
        (self.0)(t.clamp(0.0, 1.0))
    }
    /// Run only within `[start, end]` of the timeline: `0` before, `1` after.
    pub fn window(self, start: f32, end: f32) -> Curve {
        let span = (end - start).max(1e-6);
        Curve::new(move |t| self.apply(((t - start) / span).clamp(0.0, 1.0)))
    }
    /// Play backwards.
    pub fn reverse(self) -> Curve {
        Curve::new(move |t| self.apply(1.0 - t))
    }
    /// Forward in the first half, back in the second.
    pub fn ping_pong(self) -> Curve {
        Curve::new(move |t| self.apply(1.0 - (2.0 * t - 1.0).abs()))
    }
    /// `self` over the first half (output `0..0.5`), then `next` over the
    /// second (output `0.5..1`).
    pub fn then(self, next: Curve) -> Curve {
        Curve::new(move |t| {
            if t < 0.5 {
                self.apply(2.0 * t) * 0.5
            } else {
                0.5 + next.apply(2.0 * t - 1.0) * 0.5
            }
        })
    }
    /// Feed this curve's output through `next`.
    pub fn pipe(self, next: Curve) -> Curve {
        Curve::new(move |t| next.apply(self.apply(t)))
    }
    /// Curve for item `i` of `n` in a staggered group; `overlap` in `0..1`
    /// is how much neighbouring items run concurrently.
    pub fn stagger(self, i: usize, n: usize, overlap: f32) -> Curve {
        let (start, end) = stagger_window(i, n, overlap);
        self.window(start, end)
    }
    /// Map the eased progress onto `a..b`.
    pub fn value(&self, t: f32, a: f32, b: f32) -> f32 {
        lerp(a, b, self.apply(t))
    }
}

/// `(start, end)` timeline window of item `i` of `n`. Pure.
pub fn stagger_window(i: usize, n: usize, overlap: f32) -> (f32, f32) {
    if n <= 1 {
        return (0.0, 1.0);
    }
    let o = overlap.clamp(0.0, 0.95);
    let len = 1.0 / (1.0 + (n - 1) as f32 * (1.0 - o));
    let start = i as f32 * len * (1.0 - o);
    (start, start + len)
}

/// Staggered reveal: progress `0..=1` of item `i` of `n` while the whole chart
/// is at `g` (items overlap; later items start later). Pure.
pub fn reveal(g: f32, i: usize, n: usize) -> f32 {
    let start = 0.6 * i as f32 / n.max(1) as f32;
    ((g - start) / 0.4).clamp(0.0, 1.0)
}

/// Linear interpolation. Pure.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
