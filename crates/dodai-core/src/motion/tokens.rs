//! Material 3 motion tokens: the sixteen duration steps and the named easing curves.
//!
//! Toolkit-agnostic: durations are plain seconds, easings are [`Easing`] values. Other design
//! systems can ship their own scale; this one is what `md.sys.motion` defines.

use crate::motion::Easing;

/// The `md.sys.motion.duration.*` scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurationToken {
    Short1,
    Short2,
    Short3,
    Short4,
    Medium1,
    Medium2,
    Medium3,
    Medium4,
    Long1,
    Long2,
    Long3,
    Long4,
    ExtraLong1,
    ExtraLong2,
    ExtraLong3,
    ExtraLong4,
}

impl DurationToken {
    /// All steps, shortest first.
    pub const ALL: [DurationToken; 16] = [
        Self::Short1,
        Self::Short2,
        Self::Short3,
        Self::Short4,
        Self::Medium1,
        Self::Medium2,
        Self::Medium3,
        Self::Medium4,
        Self::Long1,
        Self::Long2,
        Self::Long3,
        Self::Long4,
        Self::ExtraLong1,
        Self::ExtraLong2,
        Self::ExtraLong3,
        Self::ExtraLong4,
    ];

    /// Milliseconds.
    pub const fn millis(self) -> u32 {
        match self {
            Self::Short1 => 50,
            Self::Short2 => 100,
            Self::Short3 => 150,
            Self::Short4 => 200,
            Self::Medium1 => 250,
            Self::Medium2 => 300,
            Self::Medium3 => 350,
            Self::Medium4 => 400,
            Self::Long1 => 450,
            Self::Long2 => 500,
            Self::Long3 => 550,
            Self::Long4 => 600,
            Self::ExtraLong1 => 700,
            Self::ExtraLong2 => 800,
            Self::ExtraLong3 => 900,
            Self::ExtraLong4 => 1000,
        }
    }

    /// Seconds, as the rest of the crate expects.
    pub fn secs(self) -> f32 {
        self.millis() as f32 / 1000.0
    }

    /// Token name as in the spec, e.g. `"short1"`, `"extra-long4"`.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Short1 => "short1",
            Self::Short2 => "short2",
            Self::Short3 => "short3",
            Self::Short4 => "short4",
            Self::Medium1 => "medium1",
            Self::Medium2 => "medium2",
            Self::Medium3 => "medium3",
            Self::Medium4 => "medium4",
            Self::Long1 => "long1",
            Self::Long2 => "long2",
            Self::Long3 => "long3",
            Self::Long4 => "long4",
            Self::ExtraLong1 => "extra-long1",
            Self::ExtraLong2 => "extra-long2",
            Self::ExtraLong3 => "extra-long3",
            Self::ExtraLong4 => "extra-long4",
        }
    }
}

/// The `md.sys.motion.easing.*` curves.
pub mod material_easing {
    use crate::motion::Easing;
    pub const EMPHASIZED: Easing = Easing::CubicBezier(0.2, 0.0, 0.0, 1.0);
    pub const EMPHASIZED_DECELERATE: Easing = Easing::CubicBezier(0.05, 0.7, 0.1, 1.0);
    pub const EMPHASIZED_ACCELERATE: Easing = Easing::CubicBezier(0.3, 0.0, 0.8, 0.15);
    pub const STANDARD: Easing = Easing::CubicBezier(0.2, 0.0, 0.0, 1.0);
    pub const STANDARD_DECELERATE: Easing = Easing::CubicBezier(0.0, 0.0, 0.0, 1.0);
    pub const STANDARD_ACCELERATE: Easing = Easing::CubicBezier(0.3, 0.0, 1.0, 1.0);
    pub const LINEAR: Easing = Easing::Linear;
}

/// A duration paired with a curve.
pub fn token_motion(d: DurationToken, easing: Easing) -> crate::motion::Motion {
    crate::motion::Motion::new(std::time::Duration::from_millis(d.millis() as u64), easing)
}

#[cfg(test)]
mod token_tests {
    use super::*;

    #[test]
    fn durations_rise_in_fifty_ms_steps_then_hundreds() {
        let ms: Vec<u32> = DurationToken::ALL.iter().map(|d| d.millis()).collect();
        assert!(ms.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(
            (ms[0], ms[3], ms[4], ms[11], ms[12], ms[15]),
            (50, 200, 250, 600, 700, 1000)
        );
        assert_eq!(DurationToken::Medium2.secs(), 0.3);
    }

    #[test]
    fn named_curves_run_from_zero_to_one() {
        for e in [
            material_easing::EMPHASIZED,
            material_easing::EMPHASIZED_DECELERATE,
            material_easing::EMPHASIZED_ACCELERATE,
            material_easing::STANDARD_DECELERATE,
            material_easing::STANDARD_ACCELERATE,
        ] {
            assert!(e.apply(0.0).abs() < 1e-3 && (e.apply(1.0) - 1.0).abs() < 1e-3);
        }
    }
}
