//! Window-free tests: token → value mapping, geometry, generator sync.
use legendary_core::{themes::Spec, Corners, Mode, Shape, Spacing, Theme, ThemeMeta, Typography};
use legendary_slint::*;

fn spec(id: &'static str, shape: Shape, glow: f32, blur: f32, caps: bool) -> Theme {
    Spec {
        meta: ThemeMeta {
            id,
            name: id,
            family: "test",
            description: "",
            mode: Mode::Light,
        },
        mode: Mode::Light,
        background: 0xf6f7f9,
        surface: 0xffffff,
        raised: 0xeceef2,
        border: 0xd5d9e0,
        border_strong: 0xaab1bd,
        text: 0x14171c,
        muted: 0x4a5260,
        disabled: 0x8a92a0,
        primary: 0x1f5fd6,
        primary_hover: 0x1a50b5,
        on_primary: 0xffffff,
        secondary: 0x0a7d6e,
        on_secondary: 0xffffff,
        accent: 0xb45309,
        info: 0x1f6fd6,
        success: 0x15803d,
        warning: 0xb45309,
        danger: 0xc81e3a,
        series: [
            0x1f5fd6, 0x0a7d6e, 0xb45309, 0xc81e3a, 0x7c3aed, 0x0e7490, 0xbe185d, 0x4a5260,
        ],
        typography: Typography {
            display: "\"Inter\", sans-serif",
            body: "Inter",
            mono: "Menlo",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: caps,
            label_tracking: 0.0,
        },
        spacing: Spacing {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 20.0,
            xl: 32.0,
        },
        shape,
        glow,
        glow_blur: blur,
        scanlines: false,
        hard_shadow: 0.0,
    }
    .build()
}
fn shape(radius: f32) -> Shape {
    Shape {
        corners: Corners::NONE,
        radius,
        chamfer: 0.0,
        border_width: 1.0,
        control_height: 36.0,
    }
}
fn rounded() -> Theme {
    spec("r", shape(8.0), 0.0, 0.0, false)
}
fn square() -> Theme {
    spec("s", shape(0.0), 0.0, 0.0, true)
}

#[test]
fn shape_flags_follow_kind() {
    let c = ThemeValues::from_theme(&Theme::default());
    assert!(c.chamfered && c.chamfer > 0.0 && (c.cut_tl || c.cut_tr || c.cut_br || c.cut_bl));
    let r = ThemeValues::from_theme(&rounded());
    assert!(!r.chamfered && r.radius == 8.0 && !r.cut_br);
    let s = ThemeValues::from_theme(&square());
    assert!(!s.chamfered && s.radius == 0.0 && s.uppercase && !r.uppercase);
}

#[test]
fn outline_geometry_per_kind() {
    let t = Theme::default();
    assert!(outline(&t.shape, 100.0, 40.0).len() > 4);
    assert_eq!(outline(&rounded().shape, 100.0, 40.0).len(), 4);
    assert_eq!(outline(&square().shape, 100.0, 40.0).len(), 4);
}

#[test]
fn glow_intensity_applies() {
    let t = Theme::default();
    let on = ThemeValues::from_theme(&t);
    assert!(on.glow_enabled && on.glow_radius > 0.0);
    let off = ThemeValues::from_theme(&t.clone().with_glow_intensity(0.0));
    assert!(!off.glow_enabled && off.glow_radius == 0.0);
    let strong = ThemeValues::from_theme(&t.with_glow_intensity(2.0));
    assert!(strong.glow_radius > on.glow_radius);
    assert!(!ThemeValues::from_theme(&rounded()).glow_enabled);
}

#[test]
fn table_sizes_and_button_tokens() {
    for t in [Theme::default(), rounded(), square()] {
        let v = ThemeValues::from_theme(&t);
        assert_eq!(v.button_bg.len(), 24);
        assert_eq!(v.tag_bg.len(), 8);
        assert_eq!(v.toggle_track.len(), 8);
        assert_eq!(v.progress_fill.len(), 5);
        assert_eq!(v.list_bg.len(), 4);
        let b = t.button(BUTTON_VARIANTS[0], STATES[0]);
        let bg = b.background.unwrap();
        assert_eq!(v.button_bg[0], [bg.r, bg.g, bg.b, bg.a]);
        // outline idle has no fill
        assert_eq!(v.button_bg[2 * 4], TRANSPARENT);
    }
}

#[test]
fn font_family_first_unquoted() {
    assert_eq!(first_family("\"Inter\", sans-serif"), "Inter");
    assert_eq!(ThemeValues::from_theme(&rounded()).font_mono, "Menlo");
}

#[test]
fn generator_matches_committed_theme_file() {
    let generated = generate_theme_slint(&Theme::default());
    let committed = include_str!("../ui/theme.slint");
    assert_eq!(
        generated, committed,
        "regenerate: cargo run -p legendary-slint --example slint_gen > crates/legendary-slint/ui/theme.slint"
    );
    for n in property_names() {
        assert!(generated.contains(&format!(" {n}:")));
    }
}

#[test]
fn components_have_no_theme_id_branching() {
    let src = COMPONENTS_SLINT;
    for t in legendary_core::themes::all() {
        assert!(!src.contains(t.id()), "components mention {}", t.id());
    }
}
