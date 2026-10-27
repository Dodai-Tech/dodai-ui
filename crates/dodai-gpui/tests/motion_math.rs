//! Pure motion helpers: colour mixing and optional-colour fading.

use dodai_gpui::motion::{lerp, mix, mix_opt};
use gpui::{hsla, Rgba};

fn rgba(h: gpui::Hsla) -> Rgba {
    h.into()
}

#[test]
fn lerp_hits_both_ends_and_the_middle() {
    assert_eq!(lerp(2.0, 10.0, 0.0), 2.0);
    assert_eq!(lerp(2.0, 10.0, 1.0), 10.0);
    assert!((lerp(2.0, 10.0, 0.5) - 6.0).abs() < 1e-5);
}

#[test]
fn mix_ends_match_inputs() {
    let (a, b) = (hsla(0.0, 1.0, 0.5, 1.0), hsla(0.6, 1.0, 0.5, 1.0));
    let (m0, m1) = (rgba(mix(a, b, 0.0)), rgba(mix(a, b, 1.0)));
    assert!((m0.r - rgba(a).r).abs() < 1e-3 && (m1.b - rgba(b).b).abs() < 1e-3);
}

#[test]
fn mix_opt_fades_a_missing_side_through_transparency() {
    let c = hsla(0.3, 0.5, 0.5, 1.0);
    assert!(mix_opt(None, None, 0.5).is_none());
    assert!((mix_opt(Some(c), None, 0.25).unwrap().a - 0.75).abs() < 1e-5);
    assert!((mix_opt(None, Some(c), 0.25).unwrap().a - 0.25).abs() < 1e-5);
}
