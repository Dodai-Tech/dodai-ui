//! Pure time maths and text formatting (no GPUI).

use std::time::{SystemTime, UNIX_EPOCH};

/// `HH:MM:SS` from whole seconds (wraps at 24 h).
pub fn hms(secs: u64) -> String {
    let s = secs % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// Chronometer text: `MM:SS.cc`, or `H:MM:SS.cc` from one hour on.
pub fn stopwatch_text(ms: u64) -> String {
    let (cs, s) = (ms / 10 % 100, ms / 1000);
    if s >= 3600 {
        format!("{}:{:02}:{:02}.{:02}", s / 3600, s / 60 % 60, s % 60, cs)
    } else {
        format!("{:02}:{:02}.{:02}", s / 60, s % 60, cs)
    }
}

/// Countdown text, rounding **up** so it reads `00:00` only when done.
pub fn countdown_text(ms: u64) -> String {
    let s = ms.div_ceil(1000);
    if s >= 3600 {
        hms(s)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

/// Clock-hand angles in radians, clockwise from 12 o'clock, for seconds since
/// midnight: `(hour, minute, second)`.
pub fn hand_angles(day_secs: f64) -> (f32, f32, f32) {
    let t = std::f64::consts::TAU;
    let d = day_secs.rem_euclid(86_400.0);
    (
        ((d / 43_200.0) * t) as f32,
        ((d % 3600.0) / 3600.0 * t) as f32,
        ((d % 60.0) / 60.0 * t) as f32,
    )
}

/// Split totals (ms at each lap press) into per-lap durations.
pub fn lap_splits(totals: &[u64]) -> Vec<u64> {
    let mut prev = 0;
    totals
        .iter()
        .map(|&t| {
            let d = t.saturating_sub(prev);
            prev = t;
            d
        })
        .collect()
}

/// BPM from tap timestamps (seconds). Uses the last 8 intervals; `None` below two taps.
pub fn tap_bpm(taps: &[f64]) -> Option<f32> {
    if taps.len() < 2 {
        return None;
    }
    let from = taps.len().saturating_sub(9);
    let w = &taps[from..];
    let span = w[w.len() - 1] - w[0];
    if span <= 0.0 {
        return None;
    }
    Some((60.0 * (w.len() - 1) as f64 / span) as f32)
}

/// Steadiness of the taps: standard deviation of the intervals relative to
/// their mean (0 = perfectly even). `None` below three taps.
pub fn tap_jitter(taps: &[f64]) -> Option<f32> {
    if taps.len() < 3 {
        return None;
    }
    let iv: Vec<f64> = taps.windows(2).map(|w| w[1] - w[0]).collect();
    let mean = iv.iter().sum::<f64>() / iv.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let var = iv.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / iv.len() as f64;
    Some((var.sqrt() / mean) as f32)
}

/// Italian tempo marking for a BPM.
pub fn tempo_name(bpm: f32) -> &'static str {
    match bpm {
        b if b < 40.0 => "Grave",
        b if b < 60.0 => "Largo",
        b if b < 76.0 => "Adagio",
        b if b < 108.0 => "Andante",
        b if b < 120.0 => "Moderato",
        b if b < 156.0 => "Allegro",
        b if b < 176.0 => "Vivace",
        _ => "Presto",
    }
}

/// Beat index (counting from 0) and fraction through it after `elapsed` seconds.
pub fn beat_phase(elapsed: f64, bpm: f32) -> (u64, f32) {
    let beats = elapsed.max(0.0) * bpm as f64 / 60.0;
    (beats.floor() as u64, beats.fract() as f32)
}

/// Seconds since midnight for a Unix time and UTC offset in minutes.
pub fn day_secs(unix: f64, offset_min: i32) -> f64 {
    (unix + offset_min as f64 * 60.0).rem_euclid(86_400.0)
}

/// Lit segments (bits a,b,c,d,e,f,g = 0..6) of a seven-segment digit.
pub fn segments(d: u8) -> u8 {
    const T: [u8; 10] = [
        0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
        0b1111111, 0b1101111,
    ];
    T[(d % 10) as usize]
}

/// Civil date `(year, month 1-12, day 1-31)` from days since 1970-01-01.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Weekday name from days since 1970-01-01 (a Thursday).
pub fn weekday_name(days: i64) -> &'static str {
    ["THU", "FRI", "SAT", "SUN", "MON", "TUE", "WED"][days.rem_euclid(7) as usize]
}

/// `THU · 14 MAY 2025` for a Unix time and UTC offset in minutes.
pub fn date_text(unix: f64, offset_min: i32) -> String {
    const M: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    let days = ((unix + offset_min as f64 * 60.0) / 86_400.0).floor() as i64;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{} · {} {} {}",
        weekday_name(days),
        d,
        M[(m - 1) as usize],
        y
    )
}

pub(super) fn unix_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// The machine's current UTC offset in minutes.
pub fn local_utc_offset_minutes() -> i32 {
    chrono::Local::now().offset().local_minus_utc() / 60
}
