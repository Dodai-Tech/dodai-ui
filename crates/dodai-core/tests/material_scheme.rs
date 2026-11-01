//! Seed-generated Material schemes stay readable whatever the seed, style and mode.
#![cfg(feature = "material")]

use dodai_core::material::{preset_themes, Scheme, SchemeStyle, PRESETS};
use dodai_core::{Color, Mode, ThemeMeta};

/// Seeds across the hue circle, plus grey, near-black and near-white extremes.
const SEEDS: [u32; 9] = [
    0x6750a4, 0xb4004e, 0xc77700, 0x2e7d32, 0x0061a4, 0x8fd1b0, 0x808080, 0x050505, 0xfafafa,
];

fn meta(mode: Mode) -> ThemeMeta {
    ThemeMeta {
        id: "test-scheme",
        name: "Test",
        family: "material",
        description: "test",
        mode,
    }
}

fn every_scheme() -> impl Iterator<Item = Scheme> {
    SEEDS.into_iter().flat_map(|seed| {
        SchemeStyle::ALL.into_iter().flat_map(move |style| {
            [Mode::Light, Mode::Dark].map(|mode| Scheme::new(Color::hex(seed), style, mode))
        })
    })
}

#[test]
fn text_and_accent_pairs_keep_their_contrast() {
    for s in every_scheme() {
        let p = s.theme(meta(s.mode)).palette;
        let what = format!("{:?} {:?} {:?}", s.seed, s.style, s.mode);
        assert!(p.text.contrast(p.background) >= 7.0, "text: {what}");
        assert!(p.text.contrast(p.surface) >= 4.5, "text on surface: {what}");
        assert!(p.text_muted.contrast(p.background) >= 3.0, "muted: {what}");
        assert!(
            p.on_primary.contrast(p.primary) >= 4.5,
            "on primary: {what}"
        );
        let r = s.roles();
        for (on, bg, name) in [
            (
                r.on_primary_container,
                r.primary_container,
                "primary container",
            ),
            (
                r.on_secondary_container,
                r.secondary_container,
                "secondary container",
            ),
            (
                r.on_tertiary_container,
                r.tertiary_container,
                "tertiary container",
            ),
            (r.on_tertiary, r.tertiary, "tertiary"),
            (r.on_error_container, r.error_container, "error container"),
            (r.inverse_on_surface, r.inverse_surface, "inverse surface"),
        ] {
            assert!(on.contrast(bg) >= 4.5, "{name}: {what}");
        }
    }
}

#[test]
fn surfaces_step_in_order_and_follow_the_mode() {
    for s in every_scheme() {
        let r = s.roles();
        let l = [
            r.surface_container_lowest,
            r.surface_container_low,
            r.surface_container,
            r.surface_container_high,
            r.surface_container_highest,
        ]
        .map(|c| c.luminance());
        let what = format!("{:?} {:?} {:?}", s.seed, s.style, s.mode);
        // Light schemes darken towards "highest", dark ones lighten.
        for w in l.windows(2) {
            match s.mode {
                Mode::Light => assert!(w[0] >= w[1], "{what}: {l:?}"),
                Mode::Dark => assert!(w[0] <= w[1], "{what}: {l:?}"),
            }
        }
        let bg = s.theme(meta(s.mode)).palette.background.luminance();
        match s.mode {
            Mode::Light => assert!(bg > 0.5, "{what}"),
            Mode::Dark => assert!(bg < 0.1, "{what}"),
        }
    }
}

#[test]
fn monochrome_is_grey_and_the_scheme_is_deterministic() {
    let s = Scheme::new(Color::hex(0xb4004e), SchemeStyle::Monochrome, Mode::Light);
    let p = s.theme(meta(Mode::Light)).palette;
    for c in [p.primary, p.background, p.surface, p.text] {
        let spread = c.r.max(c.g).max(c.b) - c.r.min(c.g).min(c.b);
        assert!(spread <= 3, "{c:?}");
    }
    assert_eq!(s.roles(), s.roles());
}

#[test]
fn presets_have_unique_ids_and_both_modes() {
    let themes = preset_themes();
    assert_eq!(themes.len(), 2 * PRESETS.len());
    let mut ids: Vec<_> = themes.iter().map(|t| t.meta.id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), themes.len());
    for (i, t) in themes.iter().enumerate() {
        let want = if i % 2 == 0 { Mode::Light } else { Mode::Dark };
        assert_eq!(t.meta.mode, want, "{}", t.meta.id);
        assert!(t.meta.id.starts_with("material-"), "{}", t.meta.id);
    }
}

#[test]
fn style_names_are_distinct() {
    let mut n: Vec<_> = SchemeStyle::ALL.iter().map(|s| s.name()).collect();
    n.sort_unstable();
    n.dedup();
    assert_eq!(n.len(), SchemeStyle::ALL.len());
}
