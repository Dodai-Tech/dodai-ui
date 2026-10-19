//! Pure token/variant → CSS class mapping (no framework needed, unit-testable).

use dodai_core::{ButtonVariant, Tone};

/// Join non-empty class names.
pub fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Class for a button variant.
pub fn button(v: ButtonVariant) -> &'static str {
    match v {
        ButtonVariant::Primary => "dodai-btn dodai-btn--primary",
        ButtonVariant::Secondary => "dodai-btn dodai-btn--secondary",
        ButtonVariant::Outline => "dodai-btn dodai-btn--outline",
        ButtonVariant::OutlineSecondary => "dodai-btn dodai-btn--outline-secondary",
        ButtonVariant::Danger => "dodai-btn dodai-btn--danger",
        ButtonVariant::Ghost => "dodai-btn dodai-btn--ghost",
    }
}

/// Tag colour: a semantic [`Tone`] or a brand colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TagTone {
    /// Info tone.
    #[default]
    Info,
    /// Success tone.
    Success,
    /// Warning tone.
    Warning,
    /// Danger tone.
    Danger,
    /// Brand primary.
    Primary,
    /// Brand secondary.
    Secondary,
}

impl From<Tone> for TagTone {
    fn from(t: Tone) -> Self {
        match t {
            Tone::Info => Self::Info,
            Tone::Success => Self::Success,
            Tone::Warning => Self::Warning,
            Tone::Danger => Self::Danger,
        }
    }
}

/// Class for a tag/badge.
pub fn tag(tone: TagTone, soft: bool, badge: bool) -> String {
    let base = if badge { "dodai-badge" } else { "dodai-tag" };
    let t = match tone {
        TagTone::Info => "",
        TagTone::Success => "dodai-tag--success",
        TagTone::Warning => "dodai-tag--warning",
        TagTone::Danger => "dodai-tag--danger",
        TagTone::Primary => "dodai-tag--primary",
        TagTone::Secondary => "dodai-tag--secondary",
    };
    join(&[base, t, if soft { "dodai-tag--soft" } else { "" }])
}

/// Class for a progress bar of the given tone (`None` = brand primary).
pub fn progress(tone: Option<Tone>) -> &'static str {
    match tone {
        None | Some(Tone::Info) => "dodai-progress",
        Some(Tone::Success) => "dodai-progress dodai-progress--success",
        Some(Tone::Warning) => "dodai-progress dodai-progress--warning",
        Some(Tone::Danger) => "dodai-progress dodai-progress--danger",
    }
}

/// Class for a tab.
pub fn tab(active: bool) -> &'static str {
    if active {
        "dodai-tab dodai-tab--active"
    } else {
        "dodai-tab"
    }
}

/// Class for a panel.
pub fn panel(focused: bool, raised: bool) -> String {
    join(&[
        "dodai-panel",
        if focused { "dodai-panel--focus" } else { "" },
        if raised { "dodai-panel--raised" } else { "" },
    ])
}

/// Clamp to 0..=100 and format as a CSS percentage.
pub fn percent(v: f32) -> String {
    let v = if v.is_nan() { 0.0 } else { v.clamp(0.0, 100.0) };
    format!("{v}%")
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai::themes;
    use dodai_core::themes::Spec;
    use dodai_core::Theme;
    use dodai_core::{Corners, Mode, Shape, ShapeKind, Spacing, ThemeMeta, Typography};
    use dodai_topcoat::theme_vars;

    #[test]
    fn button_classes_distinct() {
        use ButtonVariant::*;
        let all = [Primary, Secondary, Outline, OutlineSecondary, Danger, Ghost];
        let set: std::collections::HashSet<_> = all.iter().map(|v| button(*v)).collect();
        assert_eq!(set.len(), all.len());
        assert!(all.iter().all(|v| button(*v).starts_with("dodai-btn ")));
    }

    #[test]
    fn tag_panel_progress_tab() {
        assert_eq!(tag(TagTone::Info, false, false), "dodai-tag");
        assert_eq!(
            tag(TagTone::Danger, true, true),
            "dodai-badge dodai-tag--danger dodai-tag--soft"
        );
        assert_eq!(TagTone::from(Tone::Success), TagTone::Success);
        assert_eq!(panel(true, false), "dodai-panel dodai-panel--focus");
        assert_eq!(
            progress(Some(Tone::Danger)),
            "dodai-progress dodai-progress--danger"
        );
        assert_eq!(tab(true), "dodai-tab dodai-tab--active");
        assert_eq!(percent(150.0), "100%");
        assert_eq!(percent(f32::NAN), "0%");
    }

    fn spec(radius: f32, label_upper: bool) -> Theme {
        let ty = Typography {
            display: "Inter",
            body: "Inter",
            mono: "Menlo",
            size_xs: 11.0,
            size_sm: 13.0,
            size_md: 15.0,
            size_lg: 18.0,
            size_xl: 24.0,
            size_xxl: 36.0,
            uppercase_labels: label_upper,
            label_tracking: 0.0,
        };
        Spec {
            meta: ThemeMeta {
                id: "t",
                name: "T",
                family: "t",
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
            typography: ty,
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape: Shape {
                corners: Corners::NONE,
                radius,
                chamfer: 0.0,
                border_width: 1.0,
                control_height: 40.0,
            },
            glow: 0.0,
            glow_blur: 0.0,
            scanlines: false,
            hard_shadow: 2.0,
        }
        .build()
    }

    #[test]
    fn vars_follow_shape_kind() {
        let rounded = spec(8.0, false);
        let square = spec(0.0, false);
        assert_eq!(rounded.shape.kind(), ShapeKind::Rounded);
        assert_eq!(square.shape.kind(), ShapeKind::Square);
        assert_ne!(theme_vars(&rounded), theme_vars(&square));
        let cp = themes::all()
            .into_iter()
            .find(|t| t.shape.kind() == ShapeKind::Chamfered)
            .unwrap();
        assert!(theme_vars(&cp).contains("clip"));
    }

    #[test]
    fn install_css_covers_all_themes_and_glow() {
        let all = themes::all();
        let css = crate::install_css_with(&all, &[0.0, 1.0]);
        for t in &all {
            assert!(css.contains(&format!("[data-theme=\"{}\"]", t.id())));
        }
        assert!(css.contains("[data-glow=\"0\"]"));
        let off = theme_vars(&all[0].clone().with_glow_intensity(0.0));
        assert_ne!(off, theme_vars(&all[0]));
    }

    #[test]
    fn labels_follow_token_not_id() {
        assert_ne!(theme_vars(&spec(8.0, true)), theme_vars(&spec(8.0, false)));
        let src = include_str!("components/display.rs");
        assert!(!src.contains("cyberpunk") && !src.contains(".id()"));
    }

    #[test]
    fn ssr_renders_components() {
        use leptos::prelude::*;
        let html = view! { <crate::Button variant=ButtonVariant::Danger disabled=true>"x"</crate::Button> }.to_html();
        assert!(html.contains("dodai-btn--danger") && html.contains("disabled"));
        let html = view! { <crate::Progress value=250.0/> }.to_html();
        assert!(html.contains("--v:100%"));
    }
}
