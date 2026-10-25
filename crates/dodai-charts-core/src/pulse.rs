//! Physiological-style signal generators and beat detection for pulse /
//! vital-sign monitors. Everything is a pure function of time, so a toolkit
//! can evaluate a trace at any instant without storing a buffer.

use std::f32::consts::TAU;

/// What the signal looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Kind {
    /// Electrocardiogram (P, QRS, T).
    #[default]
    Ecg,
    /// Photoplethysmogram (SpO₂ pulse): a smooth wave with a dicrotic notch.
    Pleth,
    /// Arterial pressure: fast upstroke, notch, slow decay.
    Art,
    /// Respiration: slow asymmetric wave.
    Resp,
    /// EEG-like: a mix of incommensurate bands.
    Eeg,
    /// Plain sine.
    Sine,
    /// Square wave.
    Square,
    /// Triangle wave.
    Triangle,
}

/// Rhythm of an [`Kind::Ecg`] trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rhythm {
    /// Regular sinus rhythm.
    #[default]
    Normal,
    /// Atrial fibrillation: irregular beats, no P wave, fibrillatory baseline.
    AFib,
    /// ST-segment elevation after the QRS.
    StElevation,
    /// Ventricular fibrillation: chaotic, no discernible beats.
    VFib,
    /// Flat line with faint noise.
    Asystole,
}

/// A configured signal: evaluate it with [`Signal::at`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Signal {
    pub kind: Kind,
    pub rhythm: Rhythm,
    /// Beats (or breaths, cycles) per minute.
    pub rate: f32,
    /// Beat-to-beat timing variation `0..=1` (heart-rate variability).
    pub variability: f32,
    /// Random noise amplitude (fraction of full scale).
    pub noise: f32,
    /// Slow baseline drift amplitude (fraction of full scale).
    pub wander: f32,
    /// Distinguishes two otherwise identical signals.
    pub seed: f32,
}

impl Default for Signal {
    fn default() -> Self {
        Self {
            kind: Kind::Ecg,
            rhythm: Rhythm::Normal,
            rate: 72.0,
            variability: 0.0,
            noise: 0.0,
            wander: 0.0,
            seed: 0.0,
        }
    }
}

/// Deterministic hash of `x` to `0..1`. Pure.
pub fn hash(x: f32) -> f32 {
    ((x * 127.1 + 311.7).sin() * 43758.545).fract().abs()
}

/// Smooth value noise in `-1..1` (interpolated hash). Pure.
pub fn smooth_noise(x: f32) -> f32 {
    let (i, f) = (x.floor(), x - x.floor());
    let s = f * f * (3.0 - 2.0 * f);
    (hash(i) + (hash(i + 1.0) - hash(i)) * s) * 2.0 - 1.0
}

fn gauss(x: f32, c: f32, w: f32, a: f32) -> f32 {
    a * (-((x - c) / w).powi(2)).exp()
}

/// One ECG period (`x` in cycles), about `-0.4..=1.0`. `st` lifts the segment
/// between QRS and T, `p` scales the P wave. Pure.
pub fn ecg_shape(x: f32, st: f32, p: f32) -> f32 {
    let x = x.rem_euclid(1.0);
    let st_seg = if (0.42..0.6).contains(&x) {
        st * ((x - 0.42) / 0.04).min(1.0)
    } else {
        0.0
    };
    gauss(x, 0.18, 0.04, 0.18 * p)
        + gauss(x, 0.36, 0.012, -0.22)
        + gauss(x, 0.39, 0.012, 1.0)
        + gauss(x, 0.42, 0.012, -0.35)
        + gauss(x, 0.62, 0.06, 0.28)
        + st_seg
}

/// Phase `0..1` within the beat containing `tau` seconds, for a rate in
/// beats per minute and a timing `variability` (`0` regular, `1` very
/// irregular). Beat starts are jittered deterministically, never crossing. Pure.
pub fn beat_phase(tau: f32, rate: f32, variability: f32, seed: f32) -> f32 {
    let period = 60.0 / rate.max(1.0);
    let v = variability.clamp(0.0, 1.0);
    let start = |k: f32| k * period + (hash(k + seed * 17.0) - 0.5) * v * period * 0.8;
    let k0 = (tau / period).floor();
    for k in [k0 - 1.0, k0, k0 + 1.0] {
        let (a, b) = (start(k), start(k + 1.0));
        if tau >= a && tau < b {
            return (tau - a) / (b - a);
        }
    }
    (tau / period).fract()
}

impl Signal {
    /// Value at `tau` seconds, nominally `-1..=1`. Pure.
    pub fn at(&self, tau: f32) -> f32 {
        let cycles = tau * self.rate / 60.0;
        let base = match self.kind {
            Kind::Ecg => self.ecg(tau),
            Kind::Pleth => {
                let x = beat_phase(tau, self.rate, self.variability, self.seed);
                (gauss(x, 0.22, 0.1, 1.0) + gauss(x, 0.55, 0.09, 0.32)) * 1.6 - 0.7
            }
            Kind::Art => {
                let x = beat_phase(tau, self.rate, self.variability, self.seed);
                let rise = (x / 0.12).min(1.0);
                let decay = (-(x - 0.12).max(0.0) * 2.6).exp();
                (rise * decay + gauss(x, 0.42, 0.025, 0.12)) * 1.5 - 0.75
            }
            Kind::Resp => {
                let x = cycles.fract();
                (x * TAU).sin() * 0.6 + (x * TAU * 2.0).sin() * 0.1
            }
            Kind::Eeg => {
                let t = tau + self.seed * 13.0;
                0.35 * (t * 6.0 * TAU * 0.4).sin()
                    + 0.3 * (t * 10.3 * TAU * 0.4).sin()
                    + 0.2 * (t * 19.7 * TAU * 0.4).sin()
                    + 0.35 * smooth_noise(t * 9.0)
            }
            Kind::Sine => (cycles * TAU).sin() * 0.8,
            Kind::Square => {
                if cycles.fract() < 0.5 {
                    0.7
                } else {
                    -0.7
                }
            }
            Kind::Triangle => (1.0 - 4.0 * (cycles.fract() - 0.5).abs()) * 0.8,
        };
        let drift = self.wander * (tau * 0.35 * TAU).sin();
        let noise = self.noise * smooth_noise(tau * 90.0 + self.seed * 31.0);
        base + drift + noise
    }

    fn ecg(&self, tau: f32) -> f32 {
        let x = beat_phase(
            tau,
            self.rate,
            if self.rhythm == Rhythm::AFib {
                self.variability.max(0.9)
            } else {
                self.variability
            },
            self.seed,
        );
        match self.rhythm {
            Rhythm::Normal => ecg_shape(x, 0.0, 1.0),
            Rhythm::StElevation => ecg_shape(x, 0.22, 1.0),
            Rhythm::AFib => {
                ecg_shape(x, 0.0, 0.0) + 0.05 * smooth_noise(tau * 38.0 + self.seed * 5.0)
            }
            Rhythm::VFib => {
                let t = tau + self.seed * 7.0;
                0.55 * (t * 4.6 * TAU * 0.5).sin() * (0.6 + 0.4 * smooth_noise(t * 1.3))
                    + 0.25 * (t * 9.1 * TAU * 0.5).sin()
            }
            Rhythm::Asystole => 0.015 * smooth_noise(tau * 20.0 + self.seed),
        }
    }
}

/// Indices of local maxima above `threshold` at least `min_gap` samples
/// apart (keeping the taller of two close peaks). Pure.
pub fn peaks(values: &[f32], threshold: f32, min_gap: usize) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for i in 1..values.len().saturating_sub(1) {
        let v = values[i];
        if v.is_finite() && v >= threshold && v > values[i - 1] && v >= values[i + 1] {
            match out.last().copied() {
                Some(l) if i - l < min_gap => {
                    if v > values[l] {
                        *out.last_mut().unwrap() = i;
                    }
                }
                _ => out.push(i),
            }
        }
    }
    out
}

/// Mean rate per minute from peak indices at `samples_per_sec`. Pure.
pub fn rate_from_peaks(peaks: &[usize], samples_per_sec: f32) -> Option<f32> {
    if peaks.len() < 2 || samples_per_sec <= 0.0 {
        return None;
    }
    let span = (peaks[peaks.len() - 1] - peaks[0]) as f32 / samples_per_sec;
    (span > 0.0).then(|| (peaks.len() - 1) as f32 / span * 60.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(sig: &Signal, secs: f32, hz: f32) -> Vec<f32> {
        (0..(secs * hz) as usize)
            .map(|i| sig.at(i as f32 / hz))
            .collect()
    }

    #[test]
    fn normal_ecg_rate_is_recovered_from_its_peaks() {
        let sig = Signal {
            rate: 72.0,
            ..Signal::default()
        };
        let v = sample(&sig, 20.0, 250.0);
        let p = peaks(&v, 0.6, 50);
        let bpm = rate_from_peaks(&p, 250.0).unwrap();
        assert!((bpm - 72.0).abs() < 2.0, "bpm {bpm}");
    }

    #[test]
    fn variability_keeps_beats_ordered_and_average_rate() {
        let sig = Signal {
            rate: 60.0,
            variability: 1.0,
            ..Signal::default()
        };
        let v = sample(&sig, 60.0, 200.0);
        let p = peaks(&v, 0.6, 40);
        let bpm = rate_from_peaks(&p, 200.0).unwrap();
        assert!((bpm - 60.0).abs() < 4.0, "bpm {bpm}");
        let gaps: Vec<usize> = p.windows(2).map(|w| w[1] - w[0]).collect();
        assert!(gaps.iter().max() != gaps.iter().min());
    }

    #[test]
    fn beat_phase_stays_in_unit_range() {
        for i in 0..2000 {
            let x = beat_phase(i as f32 * 0.013, 90.0, 1.0, 3.0);
            assert!((0.0..=1.0).contains(&x), "{x}");
        }
    }

    #[test]
    fn st_elevation_lifts_the_segment() {
        let n = ecg_shape(0.5, 0.0, 1.0);
        let e = ecg_shape(0.5, 0.22, 1.0);
        assert!(e - n > 0.15);
    }

    #[test]
    fn asystole_is_nearly_flat_and_all_kinds_are_finite() {
        let flat = Signal {
            rhythm: Rhythm::Asystole,
            ..Signal::default()
        };
        assert!(sample(&flat, 5.0, 100.0).iter().all(|v| v.abs() < 0.05));
        for kind in [
            Kind::Ecg,
            Kind::Pleth,
            Kind::Art,
            Kind::Resp,
            Kind::Eeg,
            Kind::Sine,
            Kind::Square,
            Kind::Triangle,
        ] {
            let s = Signal {
                kind,
                noise: 0.1,
                wander: 0.1,
                ..Signal::default()
            };
            assert!(sample(&s, 3.0, 100.0)
                .iter()
                .all(|v| v.is_finite() && v.abs() < 2.5));
        }
    }

    #[test]
    fn peaks_respects_gap_and_threshold() {
        let v = [0.0, 1.0, 0.0, 0.9, 0.0, 0.2, 0.0, 1.0, 0.0];
        assert_eq!(peaks(&v, 0.5, 3), vec![1, 7]);
        assert_eq!(peaks(&v, 0.5, 1), vec![1, 3, 7]);
        assert_eq!(rate_from_peaks(&[1], 10.0), None);
    }
}
