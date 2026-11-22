//! Calendar-aware time axis: unix-time ↔ civil date conversion and tick selection.
//!
//! Timestamps are seconds since the unix epoch (UTC) as `f64`. A fixed UTC offset in seconds
//! stands in for a time zone (there is no zone database): pass `0` for UTC or the local offset.
//! Ticks land on calendar boundaries (whole minutes, hours, local midnights, Mondays, month and
//! year starts), never on arbitrary multiples of the span.

use crate::calendar::{days_in_month, weekday};

/// A broken-down civil date and time (proleptic Gregorian).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Civil {
    pub year: i32,
    /// 1–12.
    pub month: u32,
    /// 1–31.
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
}

const DAY: i64 = 86_400;
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

/// Days since 1970-01-01 of a civil date (Hinnant's algorithm). Pure.
pub fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let y = i64::from(year) - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (i64::from(month) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(day) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Civil date of a day count since 1970-01-01. Pure.
pub fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y as i32, m, d)
}

/// Break a unix timestamp (plus a UTC offset in seconds) into a civil date and time.
pub fn civil(t: f64, offset_secs: i32) -> Civil {
    let local = t.floor() as i64 + i64::from(offset_secs);
    let days = local.div_euclid(DAY);
    let rem = local.rem_euclid(DAY);
    let (year, month, day) = civil_from_days(days);
    Civil {
        year,
        month,
        day,
        hour: (rem / 3600) as u32,
        minute: (rem % 3600 / 60) as u32,
        second: (rem % 60) as u32,
    }
}

/// Unix timestamp of a civil date and time interpreted with a UTC offset. Pure.
pub fn unix(c: Civil, offset_secs: i32) -> f64 {
    let local = days_from_civil(c.year, c.month, c.day) * DAY
        + i64::from(c.hour) * 3600
        + i64::from(c.minute) * 60
        + i64::from(c.second);
    (local - i64::from(offset_secs)) as f64
}

/// `2024-03-04 14:30:05`. Pure.
pub fn format_datetime(t: f64, offset_secs: i32) -> String {
    let c = civil(t, offset_secs);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        c.year, c.month, c.day, c.hour, c.minute, c.second
    )
}

/// One axis tick: position (unix seconds) and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    pub t: f64,
    pub label: String,
}

/// The spacing between ticks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Seconds(u32),
    Minutes(u32),
    Hours(u32),
    Days(u32),
    Weeks,
    Months(u32),
    Years(u32),
}

impl Step {
    /// Approximate length in seconds (months 30.44 days, years 365.25), for choosing a step.
    pub fn approx_secs(self) -> f64 {
        match self {
            Step::Seconds(n) => f64::from(n),
            Step::Minutes(n) => f64::from(n) * 60.0,
            Step::Hours(n) => f64::from(n) * 3600.0,
            Step::Days(n) => f64::from(n) * 86_400.0,
            Step::Weeks => 7.0 * 86_400.0,
            Step::Months(n) => f64::from(n) * 2_629_800.0,
            Step::Years(n) => f64::from(n) * 31_557_600.0,
        }
    }
}

fn candidates() -> Vec<Step> {
    let mut v = vec![];
    v.extend([1, 2, 5, 10, 15, 30].map(Step::Seconds));
    v.extend([1, 2, 5, 10, 15, 30].map(Step::Minutes));
    v.extend([1, 2, 3, 6, 12].map(Step::Hours));
    v.extend([Step::Days(1), Step::Days(2), Step::Weeks]);
    v.extend([1, 2, 3, 6].map(Step::Months));
    let mut base = 1u32;
    while base <= 1_000_000 {
        v.extend([base, 2 * base, 5 * base].map(Step::Years));
        base *= 10;
    }
    v
}

/// The finest [`Step`] that gives at most `max_ticks` ticks across `span` seconds. Pure.
pub fn choose_step(span: f64, max_ticks: usize) -> Step {
    let max = max_ticks.max(2) as f64;
    candidates()
        .into_iter()
        .find(|s| span / s.approx_secs() <= max)
        .unwrap_or(Step::Years(1_000_000))
}

/// Ticks for `[start, end]` with at most about `max_ticks` labels, on calendar boundaries.
/// Returns nothing for an empty, reversed or non-finite range. Pure.
pub fn ticks(start: f64, end: f64, max_ticks: usize, offset_secs: i32) -> Vec<Tick> {
    if !(start.is_finite() && end.is_finite()) || end <= start {
        return Vec::new();
    }
    let step = choose_step(end - start, max_ticks);
    ticks_with(start, end, step, offset_secs)
}

/// Ticks for `[start, end]` at an explicit [`Step`]. Pure.
pub fn ticks_with(start: f64, end: f64, step: Step, offset_secs: i32) -> Vec<Tick> {
    if !(start.is_finite() && end.is_finite()) || end < start {
        return Vec::new();
    }
    let off = f64::from(offset_secs);
    let mut at: Vec<f64> = Vec::new();
    match step {
        Step::Seconds(_) | Step::Minutes(_) | Step::Hours(_) | Step::Days(_) => {
            let len = step.approx_secs();
            let mut t = ((start + off) / len).ceil() * len - off;
            while t <= end + 1e-6 && at.len() < 10_000 {
                at.push(t);
                t += len;
            }
        }
        Step::Weeks => {
            // First local Monday at or after `start`.
            let local_day = ((start + off) / 86_400.0).ceil() as i64;
            let dow = weekday_of_days(local_day);
            let mut d = local_day + i64::from((7 - dow) % 7);
            while (d * DAY) as f64 - off <= end && at.len() < 10_000 {
                at.push((d * DAY) as f64 - off);
                d += 7;
            }
        }
        Step::Months(n) => {
            let c = civil(start, offset_secs);
            let n = n.max(1);
            let mut idx = i64::from(c.year) * 12 + i64::from(c.month) - 1;
            idx = (idx + i64::from(n) - 1).div_euclid(i64::from(n)) * i64::from(n);
            loop {
                let (y, m) = ((idx.div_euclid(12)) as i32, (idx.rem_euclid(12)) as u32 + 1);
                let t = month_start(y, m, offset_secs);
                if t > end || at.len() >= 10_000 {
                    break;
                }
                if t >= start {
                    at.push(t);
                }
                idx += i64::from(n);
            }
        }
        Step::Years(n) => {
            let c = civil(start, offset_secs);
            let n = i64::from(n.max(1));
            let mut y = i64::from(c.year).div_euclid(n) * n;
            loop {
                let t = month_start(y as i32, 1, offset_secs);
                if t > end || at.len() >= 10_000 {
                    break;
                }
                if t >= start {
                    at.push(t);
                }
                y += n;
            }
        }
    }
    at.into_iter()
        .map(|t| Tick {
            label: label(t, step, offset_secs),
            t,
        })
        .collect()
}

fn month_start(y: i32, m: u32, off: i32) -> f64 {
    (days_from_civil(y, m, 1) * DAY) as f64 - f64::from(off)
}

/// Monday = 0 for a day count since the epoch.
fn weekday_of_days(days: i64) -> u8 {
    // 1970-01-01 was a Thursday (3).
    (days + 3).rem_euclid(7) as u8
}

/// The label for a tick at `t` for a given step. Midnight ticks of sub-day steps show the date.
pub fn label(t: f64, step: Step, offset_secs: i32) -> String {
    let c = civil(t, offset_secs);
    let month = MONTHS[(c.month - 1) as usize];
    match step {
        Step::Seconds(_) if c.hour == 0 && c.minute == 0 && c.second == 0 => {
            format!("{month} {}", c.day)
        }
        Step::Seconds(_) => format!("{:02}:{:02}:{:02}", c.hour, c.minute, c.second),
        Step::Minutes(_) | Step::Hours(_) if c.hour == 0 && c.minute == 0 => {
            format!("{month} {}", c.day)
        }
        Step::Minutes(_) | Step::Hours(_) => format!("{:02}:{:02}", c.hour, c.minute),
        Step::Days(_) | Step::Weeks => {
            let wd = WEEKDAYS[weekday(c.year, c.month, c.day) as usize];
            if matches!(step, Step::Days(1)) {
                format!("{wd} {}", c.day)
            } else {
                format!("{month} {}", c.day)
            }
        }
        Step::Months(_) if c.month == 1 => c.year.to_string(),
        Step::Months(_) => month.to_string(),
        Step::Years(_) => c.year.to_string(),
    }
}

/// Fractional position of `t` within `[start, end]`, clamped to 0..=1. Pure.
pub fn frac(t: f64, start: f64, end: f64) -> f32 {
    if end <= start {
        return 0.5;
    }
    (((t - start) / (end - start)).clamp(0.0, 1.0)) as f32
}

/// Days in the month of a timestamp (re-export convenience for callers laying out month bands).
pub fn month_len(t: f64, offset_secs: i32) -> u32 {
    let c = civil(t, offset_secs);
    days_in_month(c.year, c.month)
}

#[cfg(test)]
mod tests {
    use super::*;

    const T2024: f64 = 1_704_067_200.0; // 2024-01-01 00:00:00 UTC

    #[test]
    fn epoch_and_known_dates_round_trip() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(days_from_civil(2024, 1, 1) * DAY, T2024 as i64);
        for d in [-800_000, -1, 0, 59, 60, 11_016, 19_782, 2_932_896] {
            let (y, m, dd) = civil_from_days(d);
            assert_eq!(days_from_civil(y, m, dd), d, "day {d}");
        }
    }

    #[test]
    fn leap_day_and_formatting() {
        let t = unix(
            Civil {
                year: 2024,
                month: 2,
                day: 29,
                hour: 23,
                minute: 59,
                second: 58,
            },
            0,
        );
        assert_eq!(format_datetime(t, 0), "2024-02-29 23:59:58");
        assert_eq!(format_datetime(t + 2.0, 0), "2024-03-01 00:00:00");
        // +02:00 pushes it over midnight.
        assert_eq!(format_datetime(t, 7200), "2024-03-01 01:59:58");
        assert_eq!(unix(civil(t, -18_000), -18_000), t);
    }

    #[test]
    fn negative_timestamps_work() {
        assert_eq!(format_datetime(-1.0, 0), "1969-12-31 23:59:59");
    }

    #[test]
    fn step_grows_with_the_span() {
        assert_eq!(choose_step(60.0, 6), Step::Seconds(10));
        assert_eq!(choose_step(3600.0, 6), Step::Minutes(10));
        assert_eq!(choose_step(86_400.0, 6), Step::Hours(6));
        assert_eq!(choose_step(86_400.0 * 365.0, 6), Step::Months(2));
        assert_eq!(choose_step(86_400.0 * 365.0 * 30.0, 6), Step::Years(5));
    }

    #[test]
    fn hour_ticks_sit_on_the_hour_and_mark_midnight() {
        let t = ticks(T2024 - 3.0 * 3600.0 + 123.0, T2024 + 9.0 * 3600.0, 6, 0);
        assert!(t.iter().all(|k| k.t % 3600.0 == 0.0));
        assert!(t.iter().any(|k| k.label == "Jan 1"), "{t:?}");
        assert!(t.len() <= 7);
    }

    #[test]
    fn offsets_shift_the_boundaries() {
        // In UTC+05:30 a "day" starts at 18:30 UTC the previous day.
        let t = ticks_with(T2024, T2024 + 3.0 * 86_400.0, Step::Days(1), 19_800);
        assert_eq!(t[0].t, T2024 + 86_400.0 - 19_800.0);
    }

    #[test]
    fn months_and_years_start_on_the_first() {
        let t = ticks_with(
            T2024 + 86_400.0 * 10.0,
            T2024 + 86_400.0 * 200.0,
            Step::Months(1),
            0,
        );
        let labels: Vec<_> = t.iter().map(|k| k.label.as_str()).collect();
        assert_eq!(labels, ["Feb", "Mar", "Apr", "May", "Jun", "Jul"]);
        let y = ticks_with(T2024 - 1.0, T2024 * 4.0, Step::Years(10), 0);
        assert_eq!(y[0].label, "2030");
        assert!(y.iter().all(|k| k.label.ends_with('0')));
    }

    #[test]
    fn weeks_start_on_monday() {
        let t = ticks_with(T2024 + 1.0, T2024 + 86_400.0 * 30.0, Step::Weeks, 0);
        for k in &t {
            let c = civil(k.t, 0);
            assert_eq!(weekday(c.year, c.month, c.day), 0);
        }
        assert_eq!(civil(t[0].t, 0).day, 8); // 2024-01-08 is the first Monday after 01-01+1s
    }

    #[test]
    fn degenerate_ranges_are_empty() {
        assert!(ticks(5.0, 5.0, 6, 0).is_empty());
        assert!(ticks(9.0, 1.0, 6, 0).is_empty());
        assert!(ticks(f64::NAN, 1.0, 6, 0).is_empty());
        assert!(ticks(0.0, f64::INFINITY, 6, 0).is_empty());
    }

    #[test]
    fn frac_clamps() {
        assert_eq!(frac(5.0, 0.0, 10.0), 0.5);
        assert_eq!(frac(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(frac(1.0, 3.0, 3.0), 0.5);
    }
}
