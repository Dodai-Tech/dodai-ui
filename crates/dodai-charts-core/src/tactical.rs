//! Tactical display logic: contacts with an affiliation, their map symbols, threat wedges and
//! the mission countdown. Pure, so every toolkit draws the same picture.

use crate::instrument::Severity;
use crate::math::{polar, turn_arc};

/// Whose side a contact is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Affiliation {
    /// Own forces (circle).
    Friendly,
    /// Enemy (diamond).
    Hostile,
    /// Not involved (square).
    Neutral,
    /// Not yet identified (triangle).
    Unknown,
}

impl Affiliation {
    /// Short name for track lists.
    pub fn name(self) -> &'static str {
        match self {
            Self::Friendly => "FRIEND",
            Self::Hostile => "HOSTILE",
            Self::Neutral => "NEUTRAL",
            Self::Unknown => "UNKNOWN",
        }
    }
}

/// A tracked contact: `bearing` and `heading` in turns (0 = up, clockwise), `range` in `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    /// Direction from own position, in turns.
    pub bearing: f32,
    /// Distance from own position, `0..=1` of the plot radius.
    pub range: f32,
    /// Side of the contact.
    pub affiliation: Affiliation,
    /// Direction of travel, in turns, when known.
    pub heading: Option<f32>,
    /// Track number shown next to the symbol.
    pub id: u32,
}

impl Contact {
    /// A contact at `bearing` turns and `range` of the radius.
    pub fn new(bearing: f32, range: f32, affiliation: Affiliation) -> Self {
        Self {
            bearing,
            range: range.clamp(0.0, 1.0),
            affiliation,
            heading: None,
            id: 0,
        }
    }
    /// Sets the direction of travel.
    pub fn heading(mut self, turns: f32) -> Self {
        self.heading = Some(turns);
        self
    }
    /// Sets the track number.
    pub fn id(mut self, id: u32) -> Self {
        self.id = id;
        self
    }
    /// Bearing in whole degrees, `0..360`. Pure.
    pub fn bearing_deg(&self) -> u32 {
        (self.bearing.rem_euclid(1.0) * 360.0).round() as u32 % 360
    }
    /// Distance in the plot's own unit when the outer ring is `max`. Pure.
    pub fn distance(&self, max: f32) -> f32 {
        self.range * max
    }
    /// Position on a plot centred at `(cx, cy)` with radius `r`. Pure.
    pub fn position(&self, cx: f32, cy: f32, r: f32) -> (f32, f32) {
        polar(cx, cy, r * self.range, self.bearing)
    }
}

/// The closed map symbol of an affiliation around `(x, y)` with radius `r`. Pure.
pub fn symbol(a: Affiliation, x: f32, y: f32, r: f32) -> Vec<(f32, f32)> {
    match a {
        Affiliation::Friendly => (0..16).map(|i| polar(x, y, r, i as f32 / 16.0)).collect(),
        Affiliation::Hostile => vec![
            (x, y - r * 1.2),
            (x + r * 1.2, y),
            (x, y + r * 1.2),
            (x - r * 1.2, y),
        ],
        Affiliation::Neutral => vec![
            (x - r, y - r),
            (x + r, y - r),
            (x + r, y + r),
            (x - r, y + r),
        ],
        Affiliation::Unknown => vec![
            (x, y - r * 1.2),
            (x + r * 1.1, y + r * 0.8),
            (x - r * 1.1, y + r * 0.8),
        ],
    }
}

/// A ring sector between radii `r0` and `r1`, centred on `bearing` and `width` turns wide. Pure.
pub fn wedge(cx: f32, cy: f32, r0: f32, r1: f32, bearing: f32, width: f32) -> Vec<(f32, f32)> {
    let (a, b) = (bearing - width / 2.0, bearing + width / 2.0);
    let mut p = turn_arc(cx, cy, r1, a, b);
    p.extend(turn_arc(cx, cy, r0, b, a));
    p
}

/// Severity of a threat from its strength `0..=1`. Pure.
pub fn threat_severity(level: f32) -> Severity {
    match level {
        l if l <= 0.0 => Severity::Normal,
        l if l < 0.35 => Severity::Advisory,
        l if l < 0.7 => Severity::Caution,
        _ => Severity::Alarm,
    }
}

/// A countdown as `T-HH:MM:SS` (negative `secs` count up as `T+`). Pure.
pub fn countdown(secs: i64) -> String {
    let (sign, s) = if secs >= 0 { ('-', secs) } else { ('+', -secs) };
    format!("T{sign}{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// How urgent the remaining time is: the last minute is an alarm, the last five a caution. Pure.
pub fn timer_severity(secs: i64) -> Severity {
    match secs {
        s if s <= 60 => Severity::Alarm,
        s if s <= 300 => Severity::Caution,
        _ => Severity::Normal,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearing_and_distance_read_in_plot_units() {
        let c = Contact::new(0.25, 0.5, Affiliation::Hostile);
        assert_eq!(c.bearing_deg(), 90);
        assert!((c.distance(80.0) - 40.0).abs() < 1e-4);
    }

    #[test]
    fn countdown_formats_both_directions() {
        assert_eq!(countdown(3725), "T-01:02:05");
        assert_eq!(countdown(-5), "T+00:00:05");
    }

    #[test]
    fn timer_gets_more_urgent() {
        assert_eq!(timer_severity(900), Severity::Normal);
        assert_eq!(timer_severity(120), Severity::Caution);
        assert_eq!(timer_severity(10), Severity::Alarm);
    }

    #[test]
    fn contact_north_is_straight_up() {
        let (x, y) = Contact::new(0.0, 0.5, Affiliation::Friendly).position(10.0, 10.0, 8.0);
        assert!((x - 10.0).abs() < 1e-4 && (y - 6.0).abs() < 1e-4);
    }

    #[test]
    fn threat_strength_maps_to_severity() {
        assert_eq!(threat_severity(0.0), Severity::Normal);
        assert_eq!(threat_severity(0.5), Severity::Caution);
        assert_eq!(threat_severity(0.9), Severity::Alarm);
    }
}
