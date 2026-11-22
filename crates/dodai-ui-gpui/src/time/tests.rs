use super::*;
#[test]
fn auto_face_follows_shape_not_theme_id() {
    use dodai_ui_core::ShapeKind::*;
    assert!(ClockFace::Auto.is_digital(Chamfered));
    assert!(!ClockFace::Auto.is_digital(Rounded));
    assert!(!ClockFace::Auto.is_digital(Square));
    assert!(ClockFace::Digital.is_digital(Rounded));
    assert!(!ClockFace::Analog.is_digital(Chamfered));
}

#[test]
fn the_local_offset_is_a_real_zone() {
    let m = local_utc_offset_minutes();
    // UTC-12 to UTC+14, and on whole quarter hours.
    assert!((-12 * 60..=14 * 60).contains(&m), "{m}");
    assert_eq!(m % 15, 0, "{m}");
}
