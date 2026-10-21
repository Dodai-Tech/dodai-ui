//! Theme provider, stylesheet scoping and theme-switching handle.

use dioxus::prelude::*;
use dodai::themes;
use dodai_core::{Theme, ThemeSwitcher};

/// Scope the topcoat multi-theme stylesheet to a `.dodai-root` element instead of `:root`,
/// so it works in any renderer (no access to `<html>` needed).
pub fn scoped_stylesheet(themes: &[Theme]) -> String {
    let css = dodai_topcoat::multi_theme_stylesheet(themes)
        .replace(":root[data-theme=", ".dodai-root[data-theme=")
        .replacen(":root{", ".dodai-root{", 1);
    format!(
        "{css}\n.dodai-root{{position:relative;min-height:100vh;background:var(--dodai-background);\
color:var(--dodai-text);font-family:var(--dodai-font-body);font-size:var(--dodai-size-md);line-height:1.5}}\n\
.dodai-root::after{{content:\"\";position:fixed;inset:0;pointer-events:none;z-index:9999;\
opacity:var(--dodai-scanlines);background:repeating-linear-gradient(to bottom,transparent 0,transparent 2px,rgba(0,0,0,.28) 3px)}}\n"
    )
}

/// The themes with the given glow intensity applied (`None` = as designed).
pub fn themes_with_glow(list: &[Theme], glow: Option<f32>) -> Vec<Theme> {
    list.iter()
        .cloned()
        .map(|t| match glow {
            Some(g) => t.with_glow_intensity(g),
            None => t,
        })
        .collect()
}

/// Handle to the active theme; get it with [`use_theme`].
#[derive(Clone, Copy)]
pub struct ThemeHandle {
    switcher: Signal<ThemeSwitcher>,
    glow: Signal<f32>,
}

impl ThemeHandle {
    /// Active theme id (the `data-theme` value).
    pub fn id(&self) -> &'static str {
        self.switcher.read().current().id()
    }
    /// Active theme with the current glow intensity applied.
    pub fn theme(&self) -> Theme {
        self.switcher
            .read()
            .current()
            .clone()
            .with_glow_intensity(*self.glow.read())
    }
    /// `(id, name)` of every theme.
    pub fn list(&self) -> Vec<(&'static str, &'static str)> {
        self.switcher
            .read()
            .themes()
            .iter()
            .map(|t| (t.id(), t.name()))
            .collect()
    }
    /// Cycle to the next theme.
    pub fn next(&mut self) {
        self.switcher.write().cycle_next();
    }
    /// Cycle to the previous theme.
    pub fn prev(&mut self) {
        self.switcher.write().cycle_prev();
    }
    /// Select by id; returns false when unknown.
    pub fn select(&mut self, id: &str) -> bool {
        self.switcher.write().select(id)
    }
    /// Current glow intensity (1 = as designed).
    pub fn glow(&self) -> f32 {
        *self.glow.read()
    }
    /// Set glow intensity (clamped at 0).
    pub fn set_glow(&mut self, g: f32) {
        self.glow.set(g.max(0.0));
    }
}

/// Access the [`ThemeHandle`] provided by [`ThemeProvider`].
pub fn use_theme() -> ThemeHandle {
    use_context::<ThemeHandle>()
}

/// Properties for [`ThemeProvider`].
#[derive(Props, Clone, PartialEq)]
pub struct ThemeProviderProps {
    /// Themes to offer; defaults to [`themes::all`].
    #[props(default)]
    pub themes: Option<Vec<Theme>>,
    /// Initial theme id.
    #[props(default)]
    pub initial: Option<String>,
    /// Initial glow intensity.
    #[props(default = 1.0)]
    pub glow: f32,
    /// Extra class on the root element.
    #[props(default)]
    pub class: String,
    pub children: Element,
}

/// Installs the stylesheet, a `data-theme` root element and the [`ThemeHandle`] context.
#[component]
pub fn ThemeProvider(props: ThemeProviderProps) -> Element {
    let handle = use_context_provider(|| {
        let mut sw = ThemeSwitcher::with_themes(props.themes.clone().unwrap_or_else(themes::all));
        if let Some(id) = &props.initial {
            sw.select(id);
        }
        ThemeHandle {
            switcher: Signal::new(sw),
            glow: Signal::new(props.glow.max(0.0)),
        }
    });
    let list = handle.switcher.read().themes().to_vec();
    let css = scoped_stylesheet(&themes_with_glow(&list, Some(handle.glow())));
    rsx! {
        style { "{css}" }
        div { class: "dodai-root {props.class}", "data-theme": handle.id(), {props.children} }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use dodai_core::themes::ThemeBuilder;
    use dodai_core::{ButtonVariant, ShapeKind, Theme};
    use dodai_core::{Corners, Mode, Shape, Spacing, ThemeMeta, Typography};

    fn light() -> Theme {
        ThemeBuilder {
            meta: ThemeMeta {
                id: "test-light",
                name: "Test Light",
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
                display: "Inter",
                body: "Inter",
                mono: "Menlo",
                size_xs: 11.0,
                size_sm: 13.0,
                size_md: 15.0,
                size_lg: 18.0,
                size_xl: 24.0,
                size_xxl: 36.0,
                uppercase_labels: false,
                label_tracking: 0.0,
            },
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape: Shape {
                corners: Corners::NONE,
                radius: 8.0,
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

    fn square() -> Theme {
        ThemeBuilder {
            meta: ThemeMeta {
                id: "test-square",
                name: "Test Light",
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
                display: "Inter",
                body: "Inter",
                mono: "Menlo",
                size_xs: 11.0,
                size_sm: 13.0,
                size_md: 15.0,
                size_lg: 18.0,
                size_xl: 24.0,
                size_xxl: 36.0,
                uppercase_labels: false,
                label_tracking: 0.0,
            },
            spacing: Spacing {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 20.0,
                xl: 32.0,
            },
            shape: Shape {
                corners: Corners::NONE,
                radius: 0.0,
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

    fn html(app: fn() -> Element) -> String {
        let mut v = VirtualDom::new(app);
        v.rebuild_in_place();
        dioxus_ssr::render(&v)
    }

    fn all_buttons() -> Element {
        rsx! {
            Button {} Button { variant: ButtonVariant::Secondary } Button { variant: ButtonVariant::Outline }
            Button { variant: ButtonVariant::OutlineSecondary } Button { variant: ButtonVariant::Danger }
            Button { variant: ButtonVariant::Ghost } Button { disabled: true }
        }
    }

    #[test]
    fn button_variants_and_disabled() {
        let h = html(all_buttons);
        for c in [
            "--primary",
            "--secondary",
            "--outline\"",
            "--outline-secondary",
            "--danger",
            "--ghost",
        ] {
            assert!(h.contains(&format!("dodai-btn{c}")) || h.contains(c), "{c}");
        }
        assert!(h.contains("disabled"));
    }

    #[test]
    fn slider_and_progress_fill() {
        fn app() -> Element {
            rsx! { Slider { value: 0.25 } Progress { value: 2.0 } }
        }
        let h = html(app);
        assert!(h.contains("--v:25%"));
        assert!(h.contains("--v:100%"));
    }

    #[test]
    fn scoped_css_has_no_root_theme_selectors() {
        let list = [light(), square()];
        let css = scoped_stylesheet(&list);
        assert!(css.contains(".dodai-root[data-theme=\"test-light\"]"));
        assert!(css.contains(".dodai-root[data-theme=\"test-square\"]"));
        assert!(!css.contains(":root"));
    }

    #[test]
    fn shape_derives_from_tokens() {
        let (l, s) = (light(), square());
        assert_eq!(l.shape.kind(), ShapeKind::Rounded);
        assert_eq!(s.shape.kind(), ShapeKind::Square);
        let cl = scoped_stylesheet(&[l]);
        let cs = scoped_stylesheet(&[s]);
        assert!(cl.contains("--dodai-radius:8px"));
        assert!(cs.contains("--dodai-radius:0px"));
        let cp = scoped_stylesheet(&[dodai::themes::all()[0].clone()]);
        assert!(cp.contains("--dodai-chamfer:16px"));
    }

    #[test]
    fn glow_intensity_applied() {
        let base = dodai::themes::all();
        let off = themes_with_glow(&base, Some(0.0));
        assert!(off.iter().all(|t| !t.effects.glow_enabled()));
        assert_eq!(themes_with_glow(&base, None), base);
    }

    #[test]
    fn uppercase_follows_token() {
        assert!(scoped_stylesheet(&[light()]).contains("--dodai-label-case:none"));
    }

    #[test]
    fn no_theme_id_branching_in_source() {
        for src in [include_str!("components.rs"), include_str!("provider.rs")] {
            let body = src.split("#[cfg(test)]").next().unwrap();
            assert!(!body.contains("cyberpunk") && !body.contains("yellow"));
        }
    }

    #[cfg(feature = "icons")]
    #[test]
    fn icon_renders_svg() {
        use dodai::icons::IconId;
        fn app() -> Element {
            rsx! {
                ThemeProvider { Icon { icon: IconId::Home } Icon { icon: IconId::Airplane, filled: true } }
            }
        }
        let h = html(app);
        assert_eq!(h.matches("<svg").count(), 2, "{h}");
        assert!(h.contains("dodai-icon"));
    }
}
