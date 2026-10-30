//! Pure token → CSS-class mapping (no Yew, trivially unit-testable).
//! Class names target the stylesheet shipped by `dodai-topcoat`.

use dodai_core::{ButtonVariant, Tone};

/// Join non-empty class fragments with spaces.
pub fn join(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Class for a [`ButtonVariant`].
pub fn button(v: ButtonVariant) -> &'static str {
    match v {
        ButtonVariant::Primary => "dodai-btn dodai-btn--primary",
        ButtonVariant::Secondary => "dodai-btn dodai-btn--secondary",
        ButtonVariant::Outline => "dodai-btn dodai-btn--outline",
        ButtonVariant::OutlineSecondary => "dodai-btn dodai-btn--outline-secondary",
        ButtonVariant::Danger => "dodai-btn dodai-btn--danger",
        ButtonVariant::Ghost => "dodai-btn dodai-btn--ghost",
        ButtonVariant::Scrim => "dodai-btn dodai-btn--scrim",
    }
}

/// Modifier class for a status [`Tone`].
pub fn tone(t: Tone) -> &'static str {
    match t {
        Tone::Info => "",
        Tone::Success => "dodai-tag--success",
        Tone::Warning => "dodai-tag--warning",
        Tone::Danger => "dodai-tag--danger",
    }
}

/// Progress-bar tone modifier.
pub fn progress_tone(t: Tone) -> &'static str {
    match t {
        Tone::Info => "",
        Tone::Success => "dodai-progress--success",
        Tone::Warning => "dodai-progress--warning",
        Tone::Danger => "dodai-progress--danger",
    }
}

/// Clamp a percentage to `0..=100`.
pub fn clamp_pct(v: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;
    use dodai_core::ShapeKind;

    #[test]
    fn every_variant_has_distinct_class() {
        use ButtonVariant::*;
        let v = [Primary, Secondary, Outline, OutlineSecondary, Danger, Ghost];
        let set: std::collections::HashSet<_> = v.iter().map(|b| button(*b)).collect();
        assert_eq!(set.len(), v.len());
    }

    #[test]
    fn tones_map() {
        assert_eq!(tone(Tone::Info), "");
        assert_eq!(tone(Tone::Danger), "dodai-tag--danger");
        assert_eq!(progress_tone(Tone::Success), "dodai-progress--success");
        assert_eq!(join(&["a", "", "b"]), "a b");
        assert_eq!(clamp_pct(150.0), 100.0);
        assert_eq!(clamp_pct(f32::NAN), 0.0);
    }

    fn light() -> dodai_core::Theme {
        use dodai_core::themes::ThemeBuilder;
        use dodai_core::{Mode, ThemeMeta};
        let s = ThemeBuilder::neutral(ThemeMeta {
            id: "test-light",
            name: "Test Light",
            family: "test",
            description: "",
            mode: Mode::Light,
        });
        let rounded = s.build();
        assert_eq!(rounded.shape.kind(), ShapeKind::Rounded);
        rounded
    }

    #[test]
    fn css_follows_shape_and_effects() {
        let l = light();
        let css = theme_css_for(&l);
        assert!(css.contains("--dodai-label-case:none;"));
        assert!(css.contains("--dodai-glow-filter:none;"));
        for t in themes::all() {
            let c = dodai_topcoat::clip_path(&t.shape);
            assert_eq!(
                c != "none",
                t.shape.kind() == ShapeKind::Chamfered,
                "{}",
                t.id()
            );
        }
        assert_eq!(dodai_topcoat::clip_path(&l.shape), "none");
    }

    fn theme_css_for(t: &dodai_core::Theme) -> String {
        crate::theme::theme_css(t)
    }

    #[test]
    fn square_theme_has_square_kind() {
        let mut t = light();
        t.shape.radius = 0.0;
        assert_eq!(t.shape.kind(), ShapeKind::Square);
    }

    #[test]
    fn components_render_themed_html() {
        use yew::prelude::*;
        #[function_component]
        fn App() -> Html {
            html! { <crate::ThemeProvider theme_id="nope"><crate::Button variant={ButtonVariant::Danger}>{"x"}</crate::Button>
            <crate::Progress value={150.0} /></crate::ThemeProvider> }
        }
        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();
        let out = rt.block_on(yew::ServerRenderer::<App>::new().render());
        assert!(out.contains("dodai-btn dodai-btn--danger"));
        assert!(out.contains("--v:100%"));
        assert!(out.contains("data-theme="));
        assert!(out.contains("--dodai-clip") || out.contains("dodai-root"));
    }
}
