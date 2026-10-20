//! Cyberpunk-only motion mixins: glitch, neon flicker, scan sweep, chamfer wipe, text
//! decrypt and hologram. Pure and deterministic (a seed and a time give the same frame
//! everywhere), toolkit-independent: an adapter paints what these describe.

use dodai_core::dodai_motion::{Clip, Easing, Pose};

/// Small integer hash: the same `(seed, n)` always gives the same bits.
fn hash(seed: u32, n: u32) -> u32 {
    let mut x = seed.wrapping_mul(0x9E37_79B1) ^ n.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^ (x >> 15)
}

/// Uniform `0..1` from `(seed, n)`.
fn unit(seed: u32, n: u32) -> f32 {
    (hash(seed, n) >> 8) as f32 / (1u32 << 24) as f32
}

/// One frame of glitch: how to displace and split the widget right now.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct GlitchFrame {
    /// Is a glitch happening?
    pub active: bool,
    /// Horizontal RGB channel split in px (red goes `+`, blue `-`).
    pub split: f32,
    /// Whole-widget horizontal jitter in px.
    pub jitter: f32,
    /// A slice of the widget displaced sideways: top as a `0..1` fraction…
    pub slice_y: f32,
    /// …its height as a fraction…
    pub slice_h: f32,
    /// …and how far it moves, px.
    pub slice_dx: f32,
}

/// Bursty glitching: in `rate` steps per second each step glitches with `probability`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Glitch {
    pub seed: u32,
    pub rate: f32,
    pub probability: f32,
    /// Strength multiplier on the pixel offsets.
    pub intensity: f32,
}

impl Glitch {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rate: 14.0,
            probability: 0.18,
            intensity: 1.0,
        }
    }
    pub fn frame(&self, t: f32) -> GlitchFrame {
        let step = (t.max(0.0) * self.rate) as u32;
        if unit(self.seed, step) >= self.probability {
            return GlitchFrame::default();
        }
        let r = |k: u32| unit(self.seed ^ 0xA5A5, step.wrapping_mul(7).wrapping_add(k));
        let s = self.intensity;
        GlitchFrame {
            active: true,
            split: (2.0 + 4.0 * r(1)) * s,
            jitter: (r(2) - 0.5) * 6.0 * s,
            slice_y: r(3) * 0.8,
            slice_h: 0.05 + 0.15 * r(4),
            slice_dx: (r(5) - 0.5) * 24.0 * s,
        }
    }
}

/// Neon sign flicker: brightness `floor..=1`, mostly steady with occasional stutters.
/// `severity` `0..1` sets how often.
pub fn flicker(t: f32, seed: u32, severity: f32, floor: f32) -> f32 {
    let step = (t.max(0.0) * 24.0) as u32;
    let slow = unit(seed, step / 24); // a bad second: stutters cluster
    let bad = slow < severity;
    if bad && unit(seed ^ 0x55, step) < 0.45 {
        floor + (1.0 - floor) * unit(seed ^ 0x77, step)
    } else {
        1.0
    }
}

/// A bright line sweeping the widget top to bottom, `0..=1`, looping every `period` s.
pub fn scan_sweep(period: f32) -> Clip<f32> {
    Clip::ease(0.0, 1.0, period, Easing::Linear).forever()
}

/// Reveal progress `0..=1` for a wipe along the chamfered corners: clip the widget's polygon
/// to this fraction.
pub fn chamfer_wipe(secs: f32) -> Clip<f32> {
    Clip::ease(0.0, 1.0, secs, Easing::CubicBezier(0.1, 0.9, 0.2, 1.0))
}

/// What one character of a decrypting string shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decrypt {
    /// Not reached yet (draw nothing or a placeholder).
    Hidden,
    /// Still scrambled: show glyph number `n` of your scramble alphabet (take it `%` its
    /// length).
    Scramble(u32),
    /// Resolved: show the real character.
    Done,
}

/// State of character `i` of `n` when the whole string is `progress` (`0..=1`) decrypted at
/// time `t`. Characters resolve left to right; those just ahead of the front churn.
pub fn decrypt(i: usize, n: usize, progress: f32, t: f32, seed: u32) -> Decrypt {
    let n = n.max(1) as f32;
    let front = progress.clamp(0.0, 1.0) * (n + 4.0) - 4.0; // 4 chars of churn ahead
    let i = i as f32;
    if i < front {
        Decrypt::Done
    } else if i < front + 4.0 {
        Decrypt::Scramble(hash(seed ^ i as u32, (t.max(0.0) * 30.0) as u32))
    } else {
        Decrypt::Hidden
    }
}

/// Hologram: translucent with slow brightness drift and rare sideways jumps.
pub fn hologram(t: f32, seed: u32) -> Pose {
    let drift = 0.06 * (t * 5.0).sin();
    let jump = if unit(seed, (t.max(0.0) * 10.0) as u32) < 0.06 {
        (unit(seed ^ 9, (t.max(0.0) * 10.0) as u32) - 0.5) * 4.0
    } else {
        0.0
    };
    Pose {
        opacity: (0.82 + drift) * flicker(t, seed, 0.3, 0.6),
        dx: jump,
        ..Pose::IDENTITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glitch_is_deterministic_and_bursty() {
        let g = Glitch::new(7);
        assert_eq!(g.frame(1.234), g.frame(1.234));
        let active = (0..2000)
            .filter(|i| g.frame(*i as f32 / 100.0).active)
            .count();
        assert!(
            active > 100 && active < 900,
            "about the set probability: {active}"
        );
        let f = (0..2000)
            .map(|i| g.frame(i as f32 / 100.0))
            .find(|f| f.active)
            .unwrap();
        assert!(f.split >= 2.0 && f.slice_h > 0.04 && f.slice_y + f.slice_h <= 1.0);
        assert_ne!(Glitch::new(1).frame(0.5), Glitch::new(1).frame(5.0).clone());
        let off = Glitch {
            probability: 0.0,
            ..g
        };
        assert!((0..500).all(|i| !off.frame(i as f32 / 50.0).active));
    }

    #[test]
    fn flicker_stays_in_range_and_dips_only_when_severe() {
        let all: Vec<f32> = (0..4000)
            .map(|i| flicker(i as f32 / 100.0, 3, 0.5, 0.4))
            .collect();
        assert!(all.iter().all(|b| (0.4..=1.0).contains(b)));
        assert!(all.iter().any(|b| *b < 0.95), "dips");
        assert!(
            (0..4000).all(|i| flicker(i as f32 / 100.0, 3, 0.0, 0.4) == 1.0),
            "severity 0 is steady"
        );
    }

    #[test]
    fn sweep_and_wipe() {
        let s = scan_sweep(2.0);
        assert!(s.duration().is_infinite() && (s.at(1.0) - 0.5).abs() < 1e-5);
        let w = chamfer_wipe(0.4);
        assert!(w.at(0.0) < 1e-5);
        assert!(
            (w.at(0.4) - 1.0).abs() < 1e-5 && w.at(0.1) > 0.25,
            "front-loaded"
        );
    }

    #[test]
    fn decrypt_resolves_left_to_right() {
        let n = 12;
        assert!((0..n).all(|i| decrypt(i, n, 0.0, 0.0, 1) == Decrypt::Hidden
            || matches!(decrypt(i, n, 0.0, 0.0, 1), Decrypt::Scramble(_))));
        assert!((0..n).all(|i| decrypt(i, n, 1.0, 0.0, 1) == Decrypt::Done));
        let mid: Vec<_> = (0..n).map(|i| decrypt(i, n, 0.5, 0.2, 1)).collect();
        let first_unresolved = mid.iter().position(|d| *d != Decrypt::Done).unwrap();
        assert!(
            mid[first_unresolved..].iter().all(|d| *d != Decrypt::Done),
            "no gaps"
        );
        assert!(mid.iter().any(|d| matches!(d, Decrypt::Scramble(_))));
        assert_eq!(decrypt(5, n, 0.3, 0.7, 2), decrypt(5, n, 0.3, 0.7, 2));
    }

    #[test]
    fn hologram_is_translucent_and_bounded() {
        for i in 0..500 {
            let p = hologram(i as f32 / 20.0, 4);
            assert!(p.opacity > 0.3 && p.opacity < 1.0 && p.dx.abs() <= 2.0 && p.dy == 0.0);
        }
    }
}
