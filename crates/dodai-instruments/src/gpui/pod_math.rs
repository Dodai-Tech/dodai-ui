//! Pure helpers behind the dash pods: they turn any input (NaN, infinite, out of range) into a
//! value that is safe to draw. No drawing here, so they are plain unit tests.

/// `v`, or `fallback` when it is NaN or infinite.
pub(crate) fn finite(v: f32, fallback: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        fallback
    }
}

/// A fraction in `0..=1` (NaN gives 0).
pub(crate) fn unit(v: f32) -> f32 {
    finite(v, 0.0).clamp(0.0, 1.0)
}

/// A percentage in `0..=100` (NaN gives 0).
pub(crate) fn percent(v: f32) -> f32 {
    finite(v, 0.0).clamp(0.0, 100.0)
}

/// A bearing in `0..360` degrees (NaN gives 0, so north).
pub(crate) fn bearing(deg: f32) -> f32 {
    let d = finite(deg, 0.0).rem_euclid(360.0);
    // rem_euclid can round a tiny negative up to exactly 360.
    if d >= 360.0 {
        0.0
    } else {
        d
    }
}

/// The eight-point name of a bearing.
pub(crate) fn cardinal(deg: f32) -> &'static str {
    const NAMES: [&str; 8] = ["N", "NE", "E", "SE", "S", "SW", "W", "NW"];
    NAMES[((bearing(deg) + 22.5) / 45.0) as usize % 8]
}

/// How many of `cells` are lit at `percent` (rounded).
pub(crate) fn lit_cells(percent_value: f32, cells: usize) -> usize {
    ((percent(percent_value) / 100.0 * cells as f32).round() as usize).min(cells)
}

/// The offset of a g reading on its axis, `-1..=1` of the way to `limit`.
pub(crate) fn g_offset(g: f32, limit: f32) -> f32 {
    let limit = finite(limit, 1.5).max(0.1);
    (finite(g, 0.0) / limit).clamp(-1.0, 1.0)
}

/// A g value as text, without a sign (the sign is the side it is written on).
pub(crate) fn g_text(g: f32) -> String {
    format!("{:.1}", finite(g, 0.0).abs())
}

/// A tyre pressure is low when it is a real number under `low`.
pub(crate) fn is_low(bar: Option<f32>, low: f32) -> bool {
    matches!(bar, Some(v) if v.is_finite() && v < low)
}

/// A tyre pressure as text; no reading (or NaN) is `---`.
pub(crate) fn pressure_text(bar: Option<f32>) -> String {
    match bar {
        Some(v) if v.is_finite() => format!("{v:.1}"),
        _ => "---".to_string(),
    }
}

/// A pod diameter: at least 120 px, 300 when not a number.
pub(crate) fn pod_diameter(s: f32) -> f32 {
    finite(s, 300.0).max(120.0)
}

/// The worst severity of the zones that contain fraction `f` (`Normal` when none does).
pub(crate) fn severity_at(
    spans: &[(f32, f32, crate::instrument::Severity)],
    f: f32,
) -> crate::instrument::Severity {
    use crate::instrument::Severity;
    let f = unit(f);
    spans
        .iter()
        .filter(|(a, z, _)| f >= *a && f <= *z)
        .map(|s| s.2)
        .fold(
            Severity::Normal,
            |worst, s| if s > worst { s } else { worst },
        )
}

/// How many of `n` segments are lit for fraction `f`: any value above the minimum lights the first.
pub(crate) fn segments_lit(f: f32, n: usize) -> usize {
    let f = unit(f);
    if f <= 0.0 {
        0
    } else {
        ((f * n as f32).ceil() as usize).clamp(1, n)
    }
}

/// Seconds in a `m:ss` or `h:mm:ss` time; `None` when it does not parse.
pub(crate) fn seconds(t: &str) -> Option<f32> {
    let mut total = 0.0_f32;
    let mut parts = 0;
    for p in t.trim().split(':') {
        let v: f32 = p.trim().parse().ok()?;
        if !v.is_finite() || v < 0.0 {
            return None;
        }
        total = total * 60.0 + v;
        parts += 1;
    }
    (parts >= 2).then_some(total)
}

/// How far through a track an `elapsed | total` line is, `0..=1` (0 when it does not parse).
pub(crate) fn played(time: &str) -> f32 {
    let mut it = time.split('|');
    match (it.next().and_then(seconds), it.next().and_then(seconds)) {
        (Some(a), Some(b)) if b > 0.0 => (a / b).clamp(0.0, 1.0),
        _ => 0.0,
    }
}

/// Where a tyre pressure sits on a dial running from half to one and a half times `low` (a
/// typical 2 bar when no limit is set).
pub(crate) fn pressure_frac(bar: Option<f32>, low: f32) -> Option<f32> {
    let low = if low.is_finite() && low > 0.0 {
        low
    } else {
        2.0
    };
    let v = bar.filter(|v| v.is_finite())?;
    Some(((v - low * 0.5) / low).clamp(0.0, 1.0))
}

/// `n` bar heights in `0.2..=1` that look like a spectrum, the same for the same `seed` text.
pub(crate) fn spectrum(seed: &str, n: usize) -> Vec<f32> {
    let mut h: u32 = 2_166_136_261;
    for b in seed.bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(16_777_619);
    }
    (0..n)
        .map(|i| {
            h ^= h << 13;
            h ^= h >> 17;
            h ^= h << 5;
            // Louder in the low bands, like music.
            let tilt = 1.0 - 0.5 * i as f32 / n.max(1) as f32;
            (0.2 + 0.8 * (h % 1000) as f32 / 1000.0 * tilt).clamp(0.2, 1.0)
        })
        .collect()
}

/// A reading split into its figure and the rest: `"12.4 l/100km"` → `("12.4", "l/100km")`. The
/// figure is the leading run of digits, `.`, `:` and `-`; without a digit in it the whole text is
/// the rest.
pub(crate) fn split_figure(text: &str) -> (&str, &str) {
    let end = text
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_digit() || matches!(c, '.' | ':' | '-')))
        .map_or(text.len(), |(i, _)| i);
    let (fig, rest) = text.split_at(end);
    if fig.chars().any(|c| c.is_ascii_digit()) {
        (fig, rest.trim_start())
    } else {
        ("", text)
    }
}

/// The elapsed part of a `"m:ss | m:ss"` time, trimmed; the whole text without a `|`.
pub(crate) fn elapsed(time: &str) -> &str {
    time.split('|').next().unwrap_or("").trim()
}

/// The total of two forces in g (NaN counts as 0).
pub(crate) fn g_total(lateral: f32, longitudinal: f32) -> f32 {
    finite(lateral, 0.0).hypot(finite(longitudinal, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_parse_and_give_the_share_played() {
        assert_eq!(seconds("1:30"), Some(90.0));
        assert_eq!(seconds("1:00:05"), Some(3605.0));
        assert_eq!(seconds("abc"), None);
        assert_eq!(seconds("90"), None, "a bare number is not a time");
        assert_eq!(played("1:30 | 3:00"), 0.5);
        assert_eq!(played("0:00 | 0:00"), 0.0);
        assert_eq!(played("9:00 | 3:00"), 1.0);
        assert_eq!(played("live"), 0.0);
    }

    #[test]
    fn pressure_dial_runs_around_the_limit() {
        assert_eq!(pressure_frac(Some(2.0), 2.0), Some(0.5));
        assert_eq!(pressure_frac(Some(0.0), 2.0), Some(0.0));
        assert_eq!(pressure_frac(Some(9.0), 2.0), Some(1.0));
        assert_eq!(pressure_frac(None, 2.0), None);
        assert_eq!(
            pressure_frac(Some(2.0), 0.0),
            Some(0.5),
            "no limit: around 2 bar"
        );
        assert_eq!(pressure_frac(Some(f32::NAN), 2.0), None);
    }

    #[test]
    fn spectrum_is_stable_and_in_range() {
        let a = spectrum("Song", 16);
        assert_eq!(a, spectrum("Song", 16));
        assert_ne!(a, spectrum("Other", 16));
        assert!(a.iter().all(|v| (0.2..=1.0).contains(v)));
    }

    #[test]
    fn unit_and_percent_clamp_and_survive_nan() {
        assert_eq!(unit(f32::NAN), 0.0);
        assert_eq!(unit(-3.0), 0.0);
        assert_eq!(unit(4.0), 1.0);
        assert_eq!(unit(f32::INFINITY), 0.0);
        assert_eq!(percent(f32::NAN), 0.0);
        assert_eq!(percent(250.0), 100.0);
        assert_eq!(percent(-1.0), 0.0);
    }

    #[test]
    fn bearing_wraps_and_never_reaches_360() {
        assert_eq!(bearing(370.0), 10.0);
        assert_eq!(bearing(-90.0), 270.0);
        assert_eq!(bearing(f32::NAN), 0.0);
        assert!(bearing(-1e-9) < 360.0);
    }

    #[test]
    fn cardinal_points_cover_the_circle() {
        assert_eq!(cardinal(0.0), "N");
        assert_eq!(cardinal(22.4), "N");
        assert_eq!(cardinal(22.6), "NE");
        assert_eq!(cardinal(90.0), "E");
        assert_eq!(cardinal(180.0), "S");
        assert_eq!(cardinal(343.0), "N");
        assert_eq!(cardinal(315.0), "NW");
        assert_eq!(cardinal(-45.0), "NW");
        assert_eq!(cardinal(f32::NAN), "N");
    }

    #[test]
    fn lit_cells_round_and_stay_in_range() {
        assert_eq!(lit_cells(0.0, 10), 0);
        assert_eq!(lit_cells(48.0, 10), 5);
        assert_eq!(lit_cells(100.0, 10), 10);
        assert_eq!(lit_cells(900.0, 10), 10);
        assert_eq!(lit_cells(f32::NAN, 10), 0);
        assert_eq!(lit_cells(50.0, 0), 0);
    }

    #[test]
    fn g_offset_clamps_to_the_cross() {
        assert_eq!(g_offset(0.75, 1.5), 0.5);
        assert_eq!(g_offset(9.0, 1.5), 1.0);
        assert_eq!(g_offset(-9.0, 1.5), -1.0);
        assert_eq!(g_offset(f32::NAN, 1.5), 0.0);
        assert_eq!(g_offset(1.0, 0.0), 1.0, "a zero limit is raised to 0.1");
        assert_eq!(g_text(-0.84), "0.8");
        assert_eq!(g_text(f32::NAN), "0.0");
    }

    #[test]
    fn tyre_text_and_low_flag() {
        assert!(is_low(Some(2.0), 2.3));
        assert!(!is_low(Some(2.3), 2.3));
        assert!(!is_low(None, 2.3));
        assert!(!is_low(Some(f32::NAN), 2.3));
        assert_eq!(pressure_text(Some(2.34)), "2.3");
        assert_eq!(pressure_text(None), "---");
        assert_eq!(pressure_text(Some(f32::NAN)), "---");
    }

    #[test]
    fn severity_and_segments() {
        use crate::instrument::Severity;
        let spans = [(0.8, 1.0, Severity::Alarm), (0.6, 0.9, Severity::Caution)];
        assert_eq!(severity_at(&spans, 0.1), Severity::Normal);
        assert_eq!(severity_at(&spans, 0.7), Severity::Caution);
        assert_eq!(severity_at(&spans, 0.85), Severity::Alarm);
        assert_eq!(severity_at(&spans, f32::NAN), Severity::Normal);
        assert_eq!(segments_lit(0.0, 20), 0);
        assert_eq!(segments_lit(0.01, 20), 1);
        assert_eq!(segments_lit(0.5, 20), 10);
        assert_eq!(segments_lit(7.0, 20), 20);
        assert_eq!(segments_lit(f32::NAN, 20), 0);
    }

    #[test]
    fn pod_diameter_has_a_floor() {
        assert_eq!(pod_diameter(10.0), 120.0);
        assert_eq!(pod_diameter(240.0), 240.0);
        assert_eq!(pod_diameter(f32::NAN), 300.0);
    }

    #[test]
    fn a_reading_splits_into_figure_and_unit() {
        assert_eq!(split_figure("12.4 l/100km"), ("12.4", "l/100km"));
        assert_eq!(split_figure("08:45"), ("08:45", ""));
        assert_eq!(split_figure("523 km"), ("523", "km"));
        assert_eq!(split_figure("--- km"), ("", "--- km"));
        assert_eq!(split_figure("Eco"), ("", "Eco"));
        assert_eq!(split_figure(""), ("", ""));
    }

    #[test]
    fn elapsed_is_the_part_before_the_bar() {
        assert_eq!(elapsed("1:05 | 3:40"), "1:05");
        assert_eq!(elapsed(" 0:00 "), "0:00");
        assert_eq!(elapsed(""), "");
    }

    #[test]
    fn total_force_is_the_length_of_the_vector() {
        assert!((g_total(0.3, 0.4) - 0.5).abs() < 1e-6);
        assert_eq!(g_total(f32::NAN, 0.0), 0.0);
    }
}
