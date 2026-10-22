//! Every design crate offers the same entry points: `light()`, `dark()`, `variants()`, `SPEC`
//! and `shape_presets()`.

use dodai::themes::{cyberpunk, material, neobrutalism, aurora};
use dodai_core::Mode;

#[test]
fn every_design_has_the_common_surface() {
    let designs = [
        (
            "cyberpunk",
            cyberpunk::light(),
            cyberpunk::dark(),
            cyberpunk::variants().len(),
            cyberpunk::shape_presets().len(),
        ),
        (
            "material",
            material::light(),
            material::dark(),
            material::variants().len(),
            material::shape_presets().len(),
        ),
        (
            "neobrutalism",
            neobrutalism::light(),
            neobrutalism::dark(),
            neobrutalism::variants().len(),
            neobrutalism::shape_presets().len(),
        ),
        (
            "aurora",
            aurora::light(),
            aurora::dark(),
            aurora::variants().len(),
            aurora::shape_presets().len(),
        ),
    ];
    for (family, light, dark, variants, shapes) in designs {
        assert_eq!(light.meta.family, family);
        assert_eq!(dark.meta.family, family);
        assert_eq!(light.meta.mode, Mode::Light, "{family}");
        assert_eq!(dark.meta.mode, Mode::Dark, "{family}");
        assert!(variants >= 2, "{family} variants");
        assert!(shapes >= 1, "{family} shape presets");
    }
}

#[test]
fn only_designs_with_an_opinion_suggest_a_surface_mix() {
    for sys in dodai::themes::systems() {
        let t = (sys.themes)().into_iter().next().unwrap();
        let fx = t.surface_preset();
        // A preset is either empty or valid (every amount within 0..=1).
        for v in [fx.strength, fx.glitch, fx.sheen, fx.flicker, fx.bloom] {
            assert!((0.0..=1.0).contains(&v), "{}: {v}", sys.name);
        }
    }
}
