//! Component token resolution and palette helpers, for every built-in theme.
#![cfg(all(
    feature = "cyberpunk",
    feature = "material",
    feature = "neobrutalism",
    feature = "aurora"
))]
use dodai_core::{themes, ButtonVariant, ControlState, Tone};

const STATES: [ControlState; 5] = [
    ControlState::Idle,
    ControlState::Hovered,
    ControlState::Pressed,
    ControlState::Focused,
    ControlState::Disabled,
];

#[test]
fn buttons_resolve_in_every_state_and_solid_fills_are_readable() {
    for t in themes::all() {
        for s in STATES {
            for v in [ButtonVariant::Primary, ButtonVariant::Danger] {
                let b = t.button(v, s);
                if let (Some(bg), false) = (b.background, s == ControlState::Disabled) {
                    assert!(
                        b.foreground.contrast(bg) >= 3.0,
                        "{} {v:?} {s:?} label on fill",
                        t.id()
                    );
                }
            }
        }
    }
}

#[test]
fn toggles_differ_between_on_and_off() {
    for t in themes::all() {
        let (off, on) = (
            t.toggle(false, ControlState::Idle),
            t.toggle(true, ControlState::Idle),
        );
        assert_ne!(off, on, "{}: on looks like off", t.id());
    }
}

#[test]
fn tones_map_to_distinct_palette_colours() {
    for t in themes::all() {
        let p = &t.palette;
        let cs = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger].map(|x| p.tone(x));
        for i in 0..cs.len() {
            for j in i + 1..cs.len() {
                assert_ne!(cs[i], cs[j], "{}: tones {i}/{j} equal", t.id());
            }
        }
    }
}

#[test]
fn series_wraps_and_effect_colours_are_translucent() {
    for t in themes::all() {
        let p = &t.palette;
        assert_eq!(p.series_at(0), p.series_at(p.series.len()));
        let (shadow, scan, _) = p.effect_colours(false);
        assert!(shadow.a < 255 && scan.a < 255, "{}", t.id());
    }
}

#[test]
fn derived_roles_are_stable() {
    for t in themes::all() {
        let p = t.palette.clone();
        let d = p.clone().with_derived_roles(false);
        assert_eq!(d.with_derived_roles(false).roles.list(), {
            p.with_derived_roles(false).roles.list()
        });
    }
}

#[test]
fn density_keeps_every_design_consistent() {
    for t in themes::all() {
        let dense = t.clone().with_density(0.8);
        assert!(
            dense.sizes().selection_box <= t.sizes().selection_box,
            "{}",
            t.id()
        );
        assert!(
            dense.shape.control_height < t.shape.control_height,
            "{}",
            t.id()
        );
    }
}
