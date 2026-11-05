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

#[cfg(test)]
mod tests {
    use super::*;

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
    fn pod_diameter_has_a_floor() {
        assert_eq!(pod_diameter(10.0), 120.0);
        assert_eq!(pod_diameter(240.0), 240.0);
        assert_eq!(pod_diameter(f32::NAN), 300.0);
    }
}
