//! Every design crate offers the same entry points: `light()`, `dark()`, `variants()`, `SPEC`
//! and `shape_presets()`.
#![cfg(all(
    feature = "cyberpunk",
    feature = "material",
    feature = "neobrutalism",
    feature = "dodai"
))]

use dodai_core::themes::{cyberpunk, dodai, material, neobrutalism};
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
            "dodai",
            dodai::light(),
            dodai::dark(),
            dodai::variants().len(),
            dodai::shape_presets().len(),
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
    for sys in dodai_core::themes::systems() {
        let t = (sys.themes)().into_iter().next().unwrap();
        let fx = t.signature_surface();
        // A preset is either empty or valid (every amount within 0..=1).
        for v in [fx.strength, fx.glitch, fx.sheen, fx.flicker, fx.bloom] {
            assert!((0.0..=1.0).contains(&v), "{}: {v}", sys.name);
        }
    }
}

#[test]
fn a_design_suggests_only_textures_and_effects_it_owns() {
    for sys in dodai_core::themes::systems() {
        let t = (sys.themes)().into_iter().next().unwrap();
        let fx = t.signature_surface();
        let owned = t.surface_textures();
        for tex in [fx.texture, fx.container_texture] {
            assert!(
                tex == dodai_core::Texture::None || owned.contains(&tex),
                "{}: preset uses {tex:?}, which it does not own",
                sys.name
            );
        }
        // Everything the preset asks for is something the design owns.
        assert_eq!(
            fx.limited(owned, t.surface_effects()),
            fx,
            "{}: the preset uses a texture or effect it does not own",
            sys.name
        );
        if !t.surface_effects().any() {
            assert_eq!(
                (fx.glitch, fx.sheen, fx.flicker, fx.bloom),
                (0.0, 0.0, 0.0, 0.0),
                "{}",
                sys.name
            );
        }
        // Whatever the app turned on, a design never draws what it does not own.
        let all_on = dodai_core::ResolvedSurface {
            texture: dodai_core::Texture::Circuit,
            container_texture: dodai_core::Texture::Halftone,
            glitch: 1.0,
            ..dodai_core::ResolvedSurface::NONE
        }
        .limited(owned, t.surface_effects());
        for tex in [all_on.texture, all_on.container_texture] {
            assert!(tex == dodai_core::Texture::None || owned.contains(&tex));
        }
    }
}

#[test]
fn only_dodai_is_frosted_by_default_and_every_design_can_be() {
    for sys in dodai_core::themes::systems() {
        let t = (sys.themes)().into_iter().next().unwrap();
        let fx = t.default_surface();
        // Only the glass of the preset is on by default, never a texture or an effect.
        assert_eq!(fx.frost, t.signature_surface().frost, "{}", sys.name);
        assert_eq!(fx.glitch + fx.sheen + fx.flicker + fx.bloom, 0.0);
        assert_eq!(fx.texture, dodai_core::Texture::None);
        let frosted = fx.frost > 0.0;
        assert_eq!(
            frosted,
            sys.name.to_lowercase().contains("dodai"),
            "{}",
            sys.name
        );
    }
}

#[test]
fn one_surface_api_follows_the_design_until_the_app_sets_one() {
    use dodai_core::Surface;
    let dodai = dodai::dark();
    let cyber = cyberpunk::yellow();

    // Unset: the design's own glass.
    assert_eq!(
        dodai.resolve_surface(&Surface::INHERIT),
        dodai.default_surface()
    );
    assert_eq!(cyber.resolve_surface(&Surface::INHERIT).frost, 0.0);

    // Frosted glass turns it on in any design, and effects mix on top of it.
    let mix = Surface::frosted().glitch(0.6);
    let fx = cyber.resolve_surface(&mix);
    assert!(fx.frost > 0.0 && fx.glitch == 0.6);

    // `NONE` turns everything off, glass included.
    assert_eq!(
        dodai.resolve_surface(&Surface::NONE),
        dodai_core::ResolvedSurface::NONE
    );

    // Merging: the later one wins where it sets something.
    let both = Surface::frosted().and(Surface::new().glitch(0.6));
    assert_eq!(both, mix);

    // A resolved mix converts back into exactly itself.
    let sig = dodai.signature_surface();
    assert_eq!(dodai.resolve_surface(&Surface::from(sig)), sig);
}

#[test]
fn surface_modes_read_and_set_every_design() {
    use dodai_core::{Surface, SurfaceMode};
    for t in [dodai::dark(), cyberpunk::dark(), material::light()] {
        assert_eq!(SurfaceMode::of(&t, &Surface::INHERIT), SurfaceMode::Design);
        for m in SurfaceMode::ALL {
            if m == SurfaceMode::Custom {
                continue;
            }
            let s = m.apply(&t, &Surface::INHERIT);
            let got = SurfaceMode::of(&t, &s);
            // Where the design is already frosted, Frosted is what Design is.
            let same = got == m || (m == SurfaceMode::Frosted && got == SurfaceMode::Design);
            assert!(same, "{m:?} read back as {got:?}");
        }
        assert_eq!(
            SurfaceMode::of(&t, &Surface::frosted().glitch(0.31)),
            SurfaceMode::Custom
        );
    }
}
