//! Calendar maths for date and time pickers: dates, ranges, month grids, wrapping steps.

/// Calendar date (proleptic Gregorian).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    /// Year.
    pub year: i32,
    /// Month, 1-12.
    pub month: u32,
    /// Day of the month, 1-31.
    pub day: u32,
}

impl Date {
    /// Create a new [`Date`].
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }
}

/// Selected span of days (inclusive). `end` is `None` while the second click is pending.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    /// First day.
    pub start: Option<Date>,
    /// Last day.
    pub end: Option<Date>,
}

impl DateRange {
    /// `(from, to)` once both ends are picked.
    pub fn span(&self) -> Option<(Date, Date)> {
        Some((self.start?, self.end?))
    }
    /// Whether `d` lies inside the picked span (ends included).
    pub fn contains(&self, d: Date) -> bool {
        match (self.start, self.end) {
            (Some(a), Some(b)) => a <= d && d <= b,
            (Some(a), None) => a == d,
            _ => false,
        }
    }
}

/// Next range after clicking `d`: the first click starts, the second ends
/// (clicking an earlier day restarts there), a third click starts over.
pub fn range_pick(r: DateRange, d: Date) -> DateRange {
    match (r.start, r.end) {
        (Some(a), None) if d >= a => DateRange {
            start: Some(a),
            end: Some(d),
        },
        _ => DateRange {
            start: Some(d),
            end: None,
        },
    }
}

/// English month names, January first.
pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
/// Two-letter weekday names, Monday first.
pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Whether `y` is a Gregorian leap year.
pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Number of days in month `m` (1-12) of year `y`.
pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(y) => 29,
        _ => 28,
    }
}

/// Weekday of a date, Monday = 0 … Sunday = 6.
pub fn weekday(y: i32, m: u32, d: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let sun0 = (yy + yy / 4 - yy / 100 + yy / 400 + T[(m - 1) as usize] + d as i32).rem_euclid(7);
    ((sun0 + 6) % 7) as u32
}

/// Month as rows of 7 cells (Monday first); `None` pads outside the month.
pub fn month_grid(y: i32, m: u32) -> Vec<[Option<u32>; 7]> {
    let lead = weekday(y, m, 1) as usize;
    let n = days_in_month(y, m);
    let mut rows = Vec::new();
    let mut day = 1u32;
    let mut first = true;
    while day <= n {
        let mut row = [None; 7];
        for (i, c) in row.iter_mut().enumerate() {
            if first && i < lead {
                continue;
            }
            if day <= n {
                *c = Some(day);
                day += 1;
            }
        }
        first = false;
        rows.push(row);
    }
    rows
}

/// Previous / next month of `(year, month)`.
pub fn shift_month(y: i32, m: u32, delta: i32) -> (i32, u32) {
    let idx = y * 12 + (m as i32 - 1) + delta;
    (idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u32)
}

/// Step `hour` by `delta`, wrapping within 24 h.
pub fn step_hour(h: u32, delta: i32) -> u32 {
    (h as i32 + delta).rem_euclid(24) as u32
}

/// Step `minute` by `delta` (wrapping 0-59).
pub fn step_minute(mn: u32, delta: i32) -> u32 {
    (mn as i32 + delta).rem_euclid(60) as u32
}

/// `(hour12, is_pm)` for a 24-hour value.
pub fn to_12h(h: u32) -> (u32, bool) {
    let h12 = match h % 12 {
        0 => 12,
        v => v,
    };
    (h12, h >= 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_pick_cycle() {
        let d = |n| Date::new(2025, 3, n);
        let r = range_pick(DateRange::default(), d(10));
        assert_eq!((r.start, r.end), (Some(d(10)), None));
        let r = range_pick(r, d(5)); // earlier restarts
        assert_eq!((r.start, r.end), (Some(d(5)), None));
        let r = range_pick(r, d(9));
        assert_eq!(r.span(), Some((d(5), d(9))));
        assert!(r.contains(d(7)) && !r.contains(d(10)));
        let r = range_pick(r, d(20)); // third click restarts
        assert_eq!((r.start, r.end), (Some(d(20)), None));
    }

    #[test]
    fn leap_and_lengths() {
        assert!(is_leap(2024) && !is_leap(1900) && is_leap(2000));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 4), 30);
    }

    #[test]
    fn weekdays_known_dates() {
        // 2024-01-01 Monday, 2000-01-01 Saturday, 2077-12-10 Friday.
        assert_eq!(weekday(2024, 1, 1), 0);
        assert_eq!(weekday(2000, 1, 1), 5);
        assert_eq!(weekday(2077, 12, 10), 4);
    }

    #[test]
    fn grid_covers_every_day_once() {
        for (y, m) in [(2024, 2), (2023, 2), (2024, 9), (2077, 12)] {
            let days: Vec<u32> = month_grid(y, m)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect();
            assert_eq!(days, (1..=days_in_month(y, m)).collect::<Vec<_>>());
        }
        assert_eq!(month_grid(2024, 1)[0][0], Some(1));
    }
}
