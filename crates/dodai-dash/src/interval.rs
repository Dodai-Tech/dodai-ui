//! Time spans written the way people write them: `5s`, `15m`, `1h`, `7d`, `2w`.

use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A positive span of time. Text form: an integer and a unit, `s`, `m`, `h`, `d` or `w`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Interval(Duration);

/// Why an interval did not parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntervalError {
    /// The text was empty.
    Empty,
    /// The number part was missing or not an integer.
    BadNumber(String),
    /// The unit was not one of `s`, `m`, `h`, `d`, `w`.
    BadUnit(String),
    /// The span was zero.
    Zero,
}

impl fmt::Display for IntervalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "empty interval"),
            Self::BadNumber(s) => write!(f, "`{s}` is not a whole number"),
            Self::BadUnit(s) => write!(f, "`{s}` is not a unit (use s, m, h, d or w)"),
            Self::Zero => write!(f, "an interval must be longer than zero"),
        }
    }
}

impl std::error::Error for IntervalError {}

impl Interval {
    /// An interval of `secs` whole seconds.
    pub const fn secs(secs: u64) -> Self {
        Self(Duration::from_secs(secs))
    }

    /// The span as a [`Duration`].
    pub const fn duration(self) -> Duration {
        self.0
    }

    /// The span in whole seconds.
    pub const fn as_secs(self) -> u64 {
        self.0.as_secs()
    }
}

impl FromStr for Interval {
    type Err = IntervalError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text.is_empty() {
            return Err(IntervalError::Empty);
        }
        let split = text
            .find(|c: char| !c.is_ascii_digit())
            .ok_or_else(|| IntervalError::BadUnit(String::new()))?;
        let (number, unit) = text.split_at(split);
        let n: u64 = number
            .parse()
            .map_err(|_| IntervalError::BadNumber(number.to_string()))?;
        let per = match unit {
            "s" => 1,
            "m" => 60,
            "h" => 3_600,
            "d" => 86_400,
            "w" => 604_800,
            other => return Err(IntervalError::BadUnit(other.to_string())),
        };
        let secs = n.saturating_mul(per);
        if secs == 0 {
            return Err(IntervalError::Zero);
        }
        Ok(Self::secs(secs))
    }
}

impl fmt::Display for Interval {
    /// The largest unit that divides the span exactly, so `3600s` prints as `1h`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.as_secs();
        for (per, unit) in [(604_800, 'w'), (86_400, 'd'), (3_600, 'h'), (60, 'm')] {
            if s % per == 0 {
                return write!(f, "{}{unit}", s / per);
            }
        }
        write!(f, "{s}s")
    }
}

impl Serialize for Interval {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let text = String::deserialize(d)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Interval {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "Interval".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "string",
            "pattern": "^[0-9]+[smhdw]$",
            "description": "A span of time: a whole number and a unit (s, m, h, d, w), for example 5s, 1h, 7d."
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_unit() {
        assert_eq!("5s".parse(), Ok(Interval::secs(5)));
        assert_eq!("15m".parse(), Ok(Interval::secs(900)));
        assert_eq!("1h".parse(), Ok(Interval::secs(3_600)));
        assert_eq!("7d".parse(), Ok(Interval::secs(604_800)));
        assert_eq!("2w".parse(), Ok(Interval::secs(1_209_600)));
    }

    #[test]
    fn rejects_bad_text() {
        assert_eq!("".parse::<Interval>(), Err(IntervalError::Empty));
        assert!(matches!(
            "h".parse::<Interval>(),
            Err(IntervalError::BadNumber(_))
        ));
        assert!(matches!(
            "5".parse::<Interval>(),
            Err(IntervalError::BadUnit(_))
        ));
        assert!(matches!(
            "5x".parse::<Interval>(),
            Err(IntervalError::BadUnit(_))
        ));
        assert_eq!("0s".parse::<Interval>(), Err(IntervalError::Zero));
        assert!(matches!(
            "-5s".parse::<Interval>(),
            Err(IntervalError::BadNumber(_))
        ));
    }

    #[test]
    fn prints_the_largest_exact_unit() {
        assert_eq!(Interval::secs(3_600).to_string(), "1h");
        assert_eq!(Interval::secs(5_400).to_string(), "90m");
        assert_eq!(Interval::secs(90).to_string(), "90s");
        assert_eq!(Interval::secs(172_800).to_string(), "2d");
    }

    #[test]
    fn display_and_parse_round_trip() {
        for s in [1, 59, 60, 61, 3_600, 86_400, 604_800, 700_000] {
            let i = Interval::secs(s);
            assert_eq!(i.to_string().parse::<Interval>(), Ok(i));
        }
    }
}
