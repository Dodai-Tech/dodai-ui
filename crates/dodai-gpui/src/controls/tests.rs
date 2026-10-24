#[test]
fn checkbox_variants_have_sane_geometry() {
    for v in CheckboxVariant::ALL {
        let (target, bx) = v.dims();
        assert!(target >= bx, "{v:?}: target smaller than box");
        assert!(!v.label().is_empty());
    }
    assert_eq!(CheckboxVariant::default().dims(), (40.0, 18.0));
    assert_eq!(CheckboxVariant::Large.dims(), (48.0, 24.0));
}

use super::*;
use crate::hsla;
use crate::testing::{light_rounded, square};
use dodai::themes;

#[test]
fn clamp_and_fraction() {
    assert_eq!(clamp01(2.0), 1.0);
    assert_eq!(clamp01(f32::NAN), 0.0);
    assert_eq!(slider_fraction(5.0, 0.0, 10.0), 0.5);
    assert_eq!(slider_fraction(5.0, 3.0, 3.0), 0.0);
}

#[test]
fn ticks_follow_step() {
    assert_eq!(tick_fractions(0.0, 10.0, Some(5.0)), vec![0.0, 0.5, 1.0]);
    assert!(tick_fractions(0.0, 10.0, None).is_empty());
    assert!(tick_fractions(0.0, 1000.0, Some(1.0)).is_empty());
}

#[test]
fn slider_maps_pointer() {
    // bounds 100..300 (w=200), thumb 20: centre travels 110..290.
    assert_eq!(slider_value(100.0, 100.0, 200.0, 20.0, 0.0, 1.0, None), 0.0);
    assert!((slider_value(200.0, 100.0, 200.0, 20.0, 0.0, 1.0, None) - 0.5).abs() < 1e-6);
    assert_eq!(slider_value(999.0, 100.0, 200.0, 20.0, 0.0, 1.0, None), 1.0);
    let v = slider_value(180.0, 100.0, 200.0, 20.0, 0.0, 10.0, Some(2.0));
    assert_eq!(v, 4.0);
}

#[test]
fn toggle_style_tokens() {
    for t in [light_rounded(), square(), themes::cyberpunk::yellow()] {
        let on = toggle_style(&t, true, ControlState::Idle);
        let off = toggle_style(&t, false, ControlState::Idle);
        assert_ne!(on.fill, off.fill);
        assert_eq!(on.fill, Some(hsla(t.palette.primary)));
        assert_eq!(on.glow.is_some(), t.effects.glow_enabled());
        assert!(off.glow.is_none());
    }
}
