//! Window-free tests: token → value mapping, geometry, generator sync.
use dodai_core::{themes::ThemeBuilder, Corners, Mode, Shape, Theme, ThemeMeta, Typography};
use dodai_slint::*;

fn spec(id: &'static str, shape: Shape, glow: f32, blur: f32, caps: bool) -> Theme {
    ThemeBuilder {
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
        shape,
        glow,
        glow_blur: blur,
        hard_shadow: 0.0,
        ..ThemeBuilder::neutral(ThemeMeta {
            id,
            name: id,
            family: "test",
            description: "",
            mode: Mode::Light,
        })
    }
    .build()
}
fn shape(radius: f32) -> Shape {
    Shape {
        corners: Corners::NONE,
        radius,
        chamfer: 0.0,
        scoop: false,
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
fn scoop_follows_the_shape() {
    let mut t = dodai_core::themes::cyberpunk::yellow();
    assert!(!ThemeValues::from_theme(&t).scoop);
    t.shape.scoop = true;
    assert!(ThemeValues::from_theme(&t).scoop);
}

#[test]
fn shape_flags_follow_kind() {
    let c = ThemeValues::from_theme(&dodai_core::themes::cyberpunk::yellow());
    assert!(c.chamfered && c.chamfer > 0.0 && (c.cut_tl || c.cut_tr || c.cut_br || c.cut_bl));
    let r = ThemeValues::from_theme(&rounded());
    assert!(!r.chamfered && r.radius == 8.0 && !r.cut_br);
    let s = ThemeValues::from_theme(&square());
    assert!(!s.chamfered && s.radius == 0.0 && s.uppercase && !r.uppercase);
}

#[test]
fn outline_geometry_per_kind() {
    let t = dodai_core::themes::cyberpunk::yellow();
    assert!(outline(&t.shape, 100.0, 40.0).len() > 4);
    assert_eq!(outline(&rounded().shape, 100.0, 40.0).len(), 4);
    assert_eq!(outline(&square().shape, 100.0, 40.0).len(), 4);
}

#[test]
fn glow_intensity_applies() {
    let t = dodai_core::themes::cyberpunk::yellow();
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
    for t in [dodai_core::themes::cyberpunk::yellow(), rounded(), square()] {
        let v = ThemeValues::from_theme(&t);
        assert_eq!(v.button_bg.len(), 28);
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
    let generated = generate_theme_slint(&dodai_core::themes::cyberpunk::yellow());
    let committed = include_str!("../ui/theme.slint");
    assert_eq!(
        generated, committed,
        "regenerate: cargo run -p dodai-slint --example slint_gen > crates/dodai-slint/ui/theme.slint"
    );
    for n in property_names() {
        assert!(generated.contains(&format!(" {n}:")));
    }
}

#[test]
fn components_have_no_theme_id_branching() {
    let src = COMPONENTS_SLINT;
    for t in dodai_core::themes::all() {
        assert!(!src.contains(t.id()), "components mention {}", t.id());
    }
}
