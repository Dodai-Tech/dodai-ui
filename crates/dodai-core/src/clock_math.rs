//! Pure time maths and text formatting for clocks, timers, stopwatches and tap tempo
//! (toolkit-free).

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

/// Seconds since the Unix epoch, now (0 when the clock is before it).
pub fn unix_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(hms(3725), "01:02:05");
        assert_eq!(hms(86_400 + 5), "00:00:05");
        assert_eq!(stopwatch_text(61_230), "01:01.23");
        assert_eq!(stopwatch_text(3_661_000), "1:01:01.00");
        assert_eq!(countdown_text(0), "00:00");
        assert_eq!(countdown_text(1), "00:01");
        assert_eq!(countdown_text(60_000), "01:00");
        assert_eq!(countdown_text(3_600_000), "01:00:00");
    }

    #[test]
    fn hands() {
        let (h, m, s) = hand_angles(3.0 * 3600.0);
        assert!((h - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert!(m.abs() < 1e-5 && s.abs() < 1e-5);
        let (_, m, s) = hand_angles(30.0 * 60.0 + 15.0);
        assert!((s - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert!(m > std::f32::consts::PI);
        assert!((hand_angles(-1.0).2 - hand_angles(86_399.0).2).abs() < 1e-5);
    }

    #[test]
    fn splits() {
        assert_eq!(lap_splits(&[1000, 2500, 2600]), vec![1000, 1500, 100]);
        assert!(lap_splits(&[]).is_empty());
    }

    #[test]
    fn bpm_from_taps() {
        assert_eq!(tap_bpm(&[1.0]), None);
        let taps: Vec<f64> = (0..6).map(|i| i as f64 * 0.5).collect();
        assert!((tap_bpm(&taps).unwrap() - 120.0).abs() < 0.01);
        // only the last 8 intervals count: slow start is forgotten
        let mut v = vec![0.0, 3.0];
        v.extend((0..9).map(|i| 3.0 + (i + 1) as f64 * 0.6));
        assert!((tap_bpm(&v).unwrap() - 100.0).abs() < 0.01);
    }

    #[test]
    fn jitter() {
        let even: Vec<f64> = (0..5).map(|i| i as f64 * 0.5).collect();
        assert!(tap_jitter(&even).unwrap() < 1e-6);
        assert!(tap_jitter(&[0.0, 0.5, 1.5, 1.6]).unwrap() > 0.3);
        assert_eq!(tap_jitter(&[0.0, 1.0]), None);
    }

    #[test]
    fn tempo_and_beats() {
        assert_eq!(tempo_name(60.0), "Adagio");
        assert_eq!(tempo_name(128.0), "Allegro");
        assert_eq!(tempo_name(200.0), "Presto");
        let (i, f) = beat_phase(1.25, 120.0);
        assert_eq!(i, 2);
        assert!((f - 0.5).abs() < 1e-5);
    }

    #[test]
    fn seven_segments() {
        assert_eq!(segments(8), 0b1111111);
        assert_eq!(segments(1).count_ones(), 2);
        assert_eq!(segments(7).count_ones(), 3);
        assert_eq!(segments(13), segments(3)); // wraps
    }

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(weekday_name(0), "THU");
        assert_eq!(weekday_name(19_723), "MON"); // 2024-01-01
        assert_eq!(date_text(86_400.0 * 19_723.0 + 5.0, 0), "MON · 1 JAN 2024");
        assert_eq!(date_text(86_400.0 * 19_723.0, -60), "SUN · 31 DEC 2023");
    }

    #[test]
    fn day_seconds() {
        assert_eq!(day_secs(86_400.0 * 3.0 + 10.0, 0), 10.0);
        assert_eq!(day_secs(10.0, -1), 86_400.0 - 50.0);
        assert_eq!(day_secs(0.0, 60), 3600.0);
    }
}
