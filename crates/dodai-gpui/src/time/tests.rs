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
fn auto_face_follows_shape_not_theme_id() {
    use dodai_core::ShapeKind::*;
    assert!(ClockFace::Auto.is_digital(Chamfered));
    assert!(!ClockFace::Auto.is_digital(Rounded));
    assert!(!ClockFace::Auto.is_digital(Square));
    assert!(ClockFace::Digital.is_digital(Rounded));
    assert!(!ClockFace::Analog.is_digital(Chamfered));
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
