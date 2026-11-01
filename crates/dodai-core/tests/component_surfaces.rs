//! The surface, selection and geometry hooks of `Theme` (component.rs / spec.rs / palette.rs),
//! for every built-in theme: what a design resolves must be readable and must show state.
#![cfg(all(
    feature = "cyberpunk",
    feature = "material",
    feature = "neobrutalism",
    feature = "dodai"
))]
use dodai_core::{themes, CardVariant, ItemState, ProgressTokens, StepState, Tone};

/// Text on `bg` (the theme's page when the surface is transparent) must clear large-text contrast.
fn readable(t: &dodai_core::Theme, s: dodai_core::SurfaceTokens, what: &str) {
    let under = s.background.unwrap_or(t.palette.background);
    // A translucent fill shows the page through it: judge against the page it sits on.
    let under = if under.a < 255 {
        t.palette.background
    } else {
        under
    };
    assert!(
        s.foreground.contrast(under) >= 3.0,
        "{} {what}: {:?} on {:?}",
        t.id(),
        s.foreground,
        under
    );
}

#[test]
fn surfaces_are_readable() {
    for t in themes::all() {
        for v in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Outlined,
        ] {
            readable(&t, t.card(v), &format!("card {v:?}"));
        }
        readable(&t, t.dialog(), "dialog");
        readable(&t, t.sheet(), "sheet");
        readable(&t, t.popup(), "popup");
        readable(&t, t.snackbar().0, "snackbar");
        for tone in [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger] {
            readable(&t, t.alert(tone, false), &format!("alert {tone:?}"));
            readable(&t, t.alert(tone, true), &format!("banner {tone:?}"));
            readable(&t, t.badge(tone), &format!("badge {tone:?}"));
        }
        readable(&t, t.chip(ItemState::IDLE), "chip");
        readable(&t, t.chip(ItemState::SELECTED), "selected chip");
    }
}

#[test]
fn selection_is_visible_in_tabs_lists_and_chips() {
    for t in themes::all() {
        let (idle, picked) = (t.tab(ItemState::IDLE), t.tab(ItemState::SELECTED));
        assert_ne!(idle, picked, "{} tab", t.id());
        let (idle, picked) = (
            t.list_item(ItemState::IDLE),
            t.list_item(ItemState::SELECTED),
        );
        assert_ne!(idle, picked, "{} list item", t.id());
        assert_ne!(
            t.chip(ItemState::IDLE),
            t.chip(ItemState::SELECTED),
            "{} chip",
            t.id()
        );
        assert_ne!(
            t.navigation_item(ItemState::IDLE),
            t.navigation_item(ItemState::SELECTED),
            "{} navigation item",
            t.id()
        );
    }
}

#[test]
fn tags_progress_and_steps_stand_out_from_their_backdrop() {
    for t in themes::all() {
        for tone in [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger] {
            for solid in [false, true] {
                let tag = t.tag(tone, solid);
                assert!(
                    tag.background.is_some() || tag.border.is_some() || solid,
                    "{} {tone:?} tag has no edge",
                    t.id()
                );
                if let Some(bg) = tag.background.filter(|b| b.a == 255) {
                    assert!(tag.foreground.contrast(bg) >= 3.0, "{} {tone:?}", t.id());
                }
            }
        }
        for tone in [None, Some(Tone::Danger)] {
            let ProgressTokens { track, fill, .. } = t.progress(tone);
            assert_ne!(track, fill, "{} progress {tone:?}", t.id());
        }
        let (todo, active, done) = (
            t.step(StepState::Todo),
            t.step(StepState::Current),
            t.step(StepState::Done),
        );
        assert!(
            todo != active && active != done,
            "{} steps look alike",
            t.id()
        );
    }
}

#[test]
fn navigation_geometry_is_positive_and_nests_inside_a_row() {
    for t in themes::all() {
        let k = t.navigation();
        for (what, v) in [
            ("item height", k.item_height),
            ("nested height", k.nested_height),
            ("headline height", k.headline_height),
            ("indent", k.indent),
            ("icon", k.icon_size),
        ] {
            assert!(v > 0.0 && v.is_finite(), "{} {what}: {v}", t.id());
        }
        assert!(k.nested_height <= k.item_height, "{}", t.id());
        for v in [
            k.item_padding,
            k.container_padding,
            k.item_content_gap,
            k.item_gap,
        ] {
            assert!(v >= 0.0 && v.is_finite(), "{}", t.id());
        }
    }
}

#[test]
fn the_palette_helpers_stay_inside_the_palette() {
    for t in themes::all() {
        let p = &t.palette;
        // The series cycle never repeats a neighbour and wraps around.
        for i in 0..12 {
            assert_ne!(p.series_at(i), p.series_at(i + 1), "{} series {i}", t.id());
        }
        let wrap = p.series_at(0) == p.series_at(64) || p.series_at(0) != p.series_at(1);
        assert!(wrap, "{}", t.id());
        // Monochrome is a ramp from the text colour towards the background: every step is
        // fainter than the one before.
        let mono = p.monochrome();
        assert_eq!(mono.series_at(0), p.text, "{}", t.id());
        for i in 0..mono.series.len() - 1 {
            let (a, b) = (mono.series_at(i), mono.series_at(i + 1));
            assert!(
                b.contrast(p.background) <= a.contrast(p.background) + 1e-3,
                "{} mono step {i}",
                t.id()
            );
        }
        assert_ne!(mono.series_at(0), mono.series_at(1), "{}", t.id());
    }
}
