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
