//! Shared building blocks for instruments: vehicle dashboards, process (industrial) screens and
//! medical monitors.
//!
//! A [`Scale`] maps a value range to `0..=1` and carries coloured [`Zone`]s and tick marks.
//! [`Severity`] ranks how bad a reading is; the toolkit maps it to a colour. [`Limits`] turn alarm
//! setpoints (low-low, low, high, high-high) into zones and a severity per reading. [`Alarm`]
//! tracks acknowledgement and tells a lamp when to flash. Everything is pure and renderer-free.

/// How serious a reading or alarm is, in increasing order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Severity {
    /// Within limits.
    #[default]
    Normal,
    /// Informational, no action required.
    Advisory,
    /// Attention needed soon.
    Caution,
    /// Immediate action needed.
    Alarm,
}

impl Severity {
    /// Flash rate of an unacknowledged alarm in Hz (`0` = steady). Medical and process
    /// conventions flash higher priority faster; advisories never flash.
    pub fn flash_hz(self) -> f32 {
        match self {
            Severity::Normal | Severity::Advisory => 0.0,
            Severity::Caution => 1.0,
            Severity::Alarm => 2.0,
        }
    }
}

/// A band of a scale, `from..to` in value units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zone {
    pub from: f32,
    pub to: f32,
    pub severity: Severity,
}

/// One tick mark of a scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tick {
    pub value: f32,
    /// Position along the scale, `0..=1`.
    pub frac: f32,
    /// A labelled major tick (otherwise a minor one).
    pub major: bool,
}

/// A value range with tick marks and severity zones.
#[derive(Debug, Clone, PartialEq)]
pub struct Scale {
    pub min: f32,
    pub max: f32,
    /// Number of major intervals.
    pub majors: usize,
    /// Minor intervals per major interval (`1` = no minor ticks).
    pub minors: usize,
    pub zones: Vec<Zone>,
}

impl Scale {
    /// A scale over `min..max` with five major intervals and no minor ticks.
    pub fn new(min: f32, max: f32) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            min,
            max: if max > min { max } else { min + 1.0 },
            majors: 5,
            minors: 1,
            zones: Vec::new(),
        }
    }

    /// Major and minor tick density.
    pub fn ticks_per(mut self, majors: usize, minors: usize) -> Self {
        self.majors = majors.max(1);
        self.minors = minors.max(1);
        self
    }

    /// Adds a zone (value units); `from`/`to` are ordered and clamped to the scale.
    pub fn zone(mut self, from: f32, to: f32, severity: Severity) -> Self {
        let (a, b) = if from <= to { (from, to) } else { (to, from) };
        self.zones.push(Zone {
            from: a.max(self.min),
            to: b.min(self.max),
            severity,
        });
        self
    }

    /// Position of `v` along the scale, clamped to `0..=1`.
    pub fn fraction(&self, v: f32) -> f32 {
        if v.is_nan() {
            return 0.0;
        }
        ((v - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
    }

    /// Value at a position `0..=1`.
    pub fn value_at(&self, frac: f32) -> f32 {
        self.min + (self.max - self.min) * frac.clamp(0.0, 1.0)
    }

    /// The worst severity of every zone containing `v` (`Normal` when none does).
    pub fn severity_at(&self, v: f32) -> Severity {
        self.zones
            .iter()
            .filter(|z| v >= z.from && v <= z.to)
            .map(|z| z.severity)
            .max()
            .unwrap_or_default()
    }

    /// Major and minor ticks from `min` to `max`.
    pub fn ticks(&self) -> Vec<Tick> {
        let total = self.majors * self.minors;
        (0..=total)
            .map(|i| {
                let frac = i as f32 / total as f32;
                Tick {
                    value: self.value_at(frac),
                    frac,
                    major: i % self.minors == 0,
                }
            })
            .collect()
    }

    /// Zones as `(start, end, severity)` fractions, in drawing order (low severity first so the
    /// worst band is painted on top).
    pub fn zone_spans(&self) -> Vec<(f32, f32, Severity)> {
        let mut v: Vec<_> = self
            .zones
            .iter()
            .map(|z| (self.fraction(z.from), self.fraction(z.to), z.severity))
            .filter(|(a, b, _)| b > a)
            .collect();
        v.sort_by_key(|z| z.2);
        v
    }
}

/// Alarm setpoints in value units; any may be unset.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Limits {
    pub low_low: Option<f32>,
    pub low: Option<f32>,
    pub high: Option<f32>,
    pub high_high: Option<f32>,
}

impl Limits {
    /// Severity of a reading: beyond a `*_low`/`*_high` setpoint it is a caution, beyond a
    /// double setpoint an alarm. A NaN reading is an alarm (the sensor is not delivering).
    pub fn severity(&self, v: f32) -> Severity {
        if v.is_nan() {
            return Severity::Alarm;
        }
        if self.high_high.is_some_and(|h| v >= h) || self.low_low.is_some_and(|l| v <= l) {
            Severity::Alarm
        } else if self.high.is_some_and(|h| v >= h) || self.low.is_some_and(|l| v <= l) {
            Severity::Caution
        } else {
            Severity::Normal
        }
    }

    /// Paints these limits onto a scale as zones.
    pub fn zones_on(&self, mut scale: Scale) -> Scale {
        let (min, max) = (scale.min, scale.max);
        if let Some(l) = self.low_low {
            scale = scale.zone(min, l, Severity::Alarm);
        }
        if let Some(l) = self.low {
            scale = scale.zone(self.low_low.unwrap_or(min), l, Severity::Caution);
        }
        if let Some(h) = self.high_high {
            scale = scale.zone(h, max, Severity::Alarm);
        }
        if let Some(h) = self.high {
            scale = scale.zone(h, self.high_high.unwrap_or(max), Severity::Caution);
        }
        scale
    }
}

/// Acknowledgement state of one alarm point.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Alarm {
    /// Current severity of the condition.
    pub severity: Severity,
    /// The operator has acknowledged it.
    pub acknowledged: bool,
}

impl Alarm {
    /// Feeds a new reading's severity: a worse condition always demands a fresh acknowledgement;
    /// a return to normal clears it.
    pub fn update(&mut self, now: Severity) {
        if now > self.severity || now == Severity::Normal {
            self.acknowledged = false;
        }
        self.severity = now;
    }

    /// Operator acknowledge.
    pub fn acknowledge(&mut self) {
        self.acknowledged = true;
    }

    /// Whether the lamp is lit at `seconds`: flashing while unacknowledged, steady once
    /// acknowledged, and a 50% duty cycle at [`Severity::flash_hz`].
    pub fn lit(&self, seconds: f32) -> bool {
        if self.severity == Severity::Normal {
            return false;
        }
        let hz = self.severity.flash_hz();
        self.acknowledged || hz == 0.0 || (seconds * hz).fract() < 0.5
    }
}

/// Angle in degrees of `frac` on a dial that starts at `start` and sweeps `sweep` degrees
/// clockwise (0° = 12 o'clock).
pub fn dial_angle(frac: f32, start: f32, sweep: f32) -> f32 {
    start + sweep * frac.clamp(0.0, 1.0)
}

/// Fixed-decimal number for a readout, with a trailing unit when given. NaN reads `--`.
pub fn readout(v: f32, decimals: usize, unit: &str) -> String {
    if !v.is_finite() {
        return "--".into();
    }
    if unit.is_empty() {
        format!("{v:.decimals$}")
    } else {
        format!("{v:.decimals$} {unit}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp() -> Scale {
        Limits {
            low: Some(10.0),
            high: Some(80.0),
            high_high: Some(95.0),
            ..Limits::default()
        }
        .zones_on(Scale::new(0.0, 100.0))
    }

    #[test]
    fn fraction_clamps_and_survives_nan() {
        let s = Scale::new(0.0, 200.0);
        assert_eq!(s.fraction(50.0), 0.25);
        assert_eq!(s.fraction(-9.0), 0.0);
        assert_eq!(s.fraction(900.0), 1.0);
        assert_eq!(s.fraction(f32::NAN), 0.0);
        assert_eq!(Scale::new(5.0, 5.0).max, 6.0);
    }

    #[test]
    fn ticks_cover_the_range_with_marked_majors() {
        let t = Scale::new(0.0, 100.0).ticks_per(4, 5).ticks();
        assert_eq!(t.len(), 21);
        assert_eq!(t[0].value, 0.0);
        assert_eq!(t[20].value, 100.0);
        assert_eq!(t.iter().filter(|k| k.major).count(), 5);
    }

    #[test]
    fn limits_become_zones_and_severity() {
        let s = temp();
        assert_eq!(s.severity_at(50.0), Severity::Normal);
        assert_eq!(s.severity_at(5.0), Severity::Caution);
        assert_eq!(s.severity_at(85.0), Severity::Caution);
        assert_eq!(s.severity_at(99.0), Severity::Alarm);
        let spans = s.zone_spans();
        assert!(spans.windows(2).all(|w| w[0].2 <= w[1].2), "worst on top");
    }

    #[test]
    fn limits_rate_a_reading() {
        let l = Limits {
            low_low: Some(5.0),
            high: Some(80.0),
            high_high: Some(95.0),
            ..Limits::default()
        };
        assert_eq!(l.severity(50.0), Severity::Normal);
        assert_eq!(l.severity(80.0), Severity::Caution);
        assert_eq!(l.severity(95.0), Severity::Alarm);
        assert_eq!(l.severity(5.0), Severity::Alarm);
        assert_eq!(l.severity(f32::NAN), Severity::Alarm);
    }

    #[test]
    fn alarm_needs_fresh_acknowledgement_when_it_worsens() {
        let mut a = Alarm::default();
        a.update(Severity::Caution);
        a.acknowledge();
        a.update(Severity::Caution);
        assert!(a.acknowledged);
        a.update(Severity::Alarm);
        assert!(!a.acknowledged);
        a.acknowledge();
        a.update(Severity::Caution); // improving keeps the acknowledgement
        assert!(a.acknowledged);
        a.update(Severity::Normal);
        assert!(!a.acknowledged && !a.lit(0.0));
    }

    #[test]
    fn lamp_flashes_until_acknowledged() {
        let mut a = Alarm::default();
        a.update(Severity::Alarm); // 2 Hz: 0.25 s on, 0.25 s off
        assert!(a.lit(0.1) && !a.lit(0.3));
        a.acknowledge();
        assert!(a.lit(0.3));
        let mut adv = Alarm::default();
        adv.update(Severity::Advisory);
        assert!(adv.lit(0.3));
    }

    #[test]
    fn dial_and_readout() {
        assert_eq!(dial_angle(0.5, -135.0, 270.0), 0.0);
        assert_eq!(dial_angle(2.0, -135.0, 270.0), 135.0);
        assert_eq!(readout(72.456, 1, "bpm"), "72.5 bpm");
        assert_eq!(readout(f32::NAN, 0, "°C"), "--");
        assert_eq!(readout(3.0, 0, ""), "3");
    }
}
