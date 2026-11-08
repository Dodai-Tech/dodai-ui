//! Civil-date helpers for calendar heatmaps (proleptic Gregorian).

/// Weekday of a date: `0` Monday … `6` Sunday. Pure.
pub fn weekday(year: i32, month: u32, day: u32) -> u32 {
    // Sakamoto's algorithm gives 0 = Sunday.
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let m = month.clamp(1, 12) as usize;
    let y = if m < 3 { year - 1 } else { year };
    let sun0 = (y + y / 4 - y / 100 + y / 400 + T[m - 1] + day as i32).rem_euclid(7);
    ((sun0 + 6) % 7) as u32
}

/// Whether `year` is a leap year. Pure.
pub fn is_leap(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// Days in `month` (1–12) of `year`. Pure.
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// The date `n` days after `(year, month, day)`. Pure.
pub fn add_days(year: i32, month: u32, day: u32, n: u32) -> (i32, u32, u32) {
    let (mut y, mut m, mut d) = (year, month.clamp(1, 12), day.max(1));
    for _ in 0..n {
        d += 1;
        if d > days_in_month(y, m) {
            d = 1;
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
    }
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_weekdays() {
        assert_eq!(weekday(2024, 1, 1), 0); // Monday
        assert_eq!(weekday(2000, 1, 1), 5); // Saturday
        assert_eq!(weekday(2023, 12, 31), 6); // Sunday
    }

    #[test]
    fn month_lengths_and_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2024, 4), 30);
    }

    #[test]
    fn add_days_rolls_months_and_years() {
        assert_eq!(add_days(2024, 1, 31, 1), (2024, 2, 1));
        assert_eq!(add_days(2023, 12, 31, 1), (2024, 1, 1));
        assert_eq!(add_days(2024, 1, 1, 365), (2024, 12, 31));
    }
}
