//! Behaviour tests for the motion primitives.

use crate::*;
use std::time::Duration;
#[test]
fn entry_set_per_effect_duration() {
    let d = Duration::from_millis(200);
    let s = EntrySet::NONE
        .with(EntryKind::Fade)
        .with_ms(EntryKind::Rise, 400);
    assert!((s.duration(d).as_secs_f32() * 1000.0 - 400.0).abs() < 0.01);
    let (o, _, y) = s.frame(0.5, d); // fade done, rise half-way
    assert!((o - 0.5).abs() < 1e-6 && (y - 7.0).abs() < 1e-6);
    let (o1, _, _) = s.frame(1.0, d);
    assert!((o1 - 1.0).abs() < 1e-6);
}

#[test]
fn entry_set_composes() {
    let s = EntrySet::NONE.with(EntryKind::Fade).with(EntryKind::Rise);
    let (o, x, y) = s.frame(0.5, Duration::from_millis(200));
    assert!((o - 0.25).abs() < 1e-6 && x == 0.0 && (y - 7.0).abs() < 1e-6);
    assert!(s
        .toggled(EntryKind::Fade)
        .toggled(EntryKind::Rise)
        .is_empty());
}

#[test]
fn easing_endpoints_and_shape() {
    for e in [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
        Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
    ] {
        assert!(e.apply(0.0).abs() < 1e-3);
        assert!((e.apply(1.0) - 1.0).abs() < 1e-3);
    }
    assert!(Easing::EaseIn.apply(0.3) < 0.3);
    assert!(Easing::EaseOut.apply(0.3) > 0.3);
    assert!((Easing::EaseInOut.apply(0.5) - 0.5).abs() < 1e-2);
}

#[test]
fn physics_easings_hit_endpoints() {
    for e in [
        Easing::Bounce,
        Easing::Elastic,
        Easing::Back(1.7),
        Easing::Anticipate(1.7),
        Easing::SPRING_STIFF,
        Easing::SPRING_WOBBLY,
        Easing::SPRING_GENTLE,
    ] {
        assert!(e.apply(0.0).abs() < 1e-3, "{e:?} start");
        assert!((e.apply(1.0) - 1.0).abs() < 1e-3, "{e:?} end");
    }
    assert!(Easing::Back(1.7).apply(0.7) > 1.0, "back overshoots");
    assert!(
        Easing::Anticipate(1.7).apply(0.2) < 0.0,
        "anticipate pulls back"
    );
    assert!(Easing::Bounce.apply(0.9) <= 1.0);
}

#[test]
fn spring_overshoots_and_settles() {
    let e = Easing::SPRING;
    assert!(e.apply(0.0).abs() < 1e-4);
    assert!((e.apply(1.0) - 1.0).abs() < 1e-4);
    assert!((0..100).any(|i| e.apply(i as f32 / 100.0) > 1.02));
}

#[test]
fn curves_compose() {
    let lin = || Curve::from(Easing::Linear);
    let w = lin().window(0.5, 1.0);
    assert_eq!(w.apply(0.25), 0.0);
    assert!((w.apply(0.75) - 0.5).abs() < 1e-5);
    assert!((lin().reverse().apply(0.25) - 0.75).abs() < 1e-5);
    assert!((lin().ping_pong().apply(0.5) - 1.0).abs() < 1e-5);
    assert!(lin().ping_pong().apply(1.0).abs() < 1e-5);
    let seq = lin().then(lin().reverse());
    assert!((seq.apply(0.5) - 1.0).abs() < 1e-5);
    assert!((seq.apply(1.0) - 0.5).abs() < 1e-5);
    assert!(
        (lin().pipe(Easing::EaseIn.into()).apply(0.3) - Easing::EaseIn.apply(0.3)).abs() < 1e-5
    );
}

#[test]
fn stagger_windows_cover_timeline() {
    let (s0, _) = stagger_window(0, 4, 0.5);
    let (_, e3) = stagger_window(3, 4, 0.5);
    assert!(s0.abs() < 1e-6 && (e3 - 1.0).abs() < 1e-5);
    assert!(stagger_window(1, 4, 0.0).0 > stagger_window(0, 4, 0.0).0);
}

#[test]
fn speed_scales_duration() {
    let m = Motion::default().millis(200).speed(2.0);
    assert_eq!(m.duration, Duration::from_millis(100));
}

#[test]
fn tween_is_driven_by_the_callers_clock() {
    let m = Motion::new(Duration::from_millis(100), Easing::Linear);
    let mut t = Tween::new(0.0, m);
    t.to(10.0, m, 1.0);
    assert!((t.value(1.0)).abs() < 1e-5);
    assert!((t.value(1.05) - 5.0).abs() < 1e-3);
    assert!(t.is_done(1.2) && (t.value(9.0) - 10.0).abs() < 1e-5);
    t.to(0.0, m, 1.05); // retarget mid-flight: starts from 5
    assert!((t.value(1.05) - 5.0).abs() < 1e-3);
}

#[test]
fn entry_frame_needs_only_a_clock() {
    let m = Motion::new(Duration::from_millis(200), Easing::Linear)
        .entry(EntrySet::of(EntryKind::Rise))
        .delay(Duration::from_millis(100));
    assert_eq!(m.entry_frame(0.0).0, 0.0, "waits for the delay");
    assert!((m.entry_frame(0.2).0 - 0.5).abs() < 1e-3);
    assert_eq!(m.entry_frame(5.0), (1.0, 0.0, 0.0));
}

#[test]
fn easing_css_forms() {
    assert_eq!(Easing::EaseOut.to_css(), "ease-out");
    assert_eq!(
        Easing::CubicBezier(0.2, 0.0, 0.0, 1.0).to_css(),
        "cubic-bezier(0.2, 0, 0, 1)"
    );
    let s = Easing::SPRING.to_css();
    assert!(s.starts_with("linear(0,") && s.ends_with("1)"), "{s}");
}

#[test]
fn curves_and_motion_are_send_sync() {
    fn ok<T: Send + Sync>() {}
    ok::<Curve>();
    ok::<Motion>();
    ok::<Tween>();
}
