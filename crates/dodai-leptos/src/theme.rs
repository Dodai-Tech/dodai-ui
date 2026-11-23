//! Theme installation: stylesheet generation and [`ThemeProvider`].

use dodai_core::Theme;
use dodai_topcoat::{multi_theme_stylesheet, theme_vars, COMPONENTS_CSS};
use leptos::prelude::*;
use leptos::reactive::owner::Owner;
use leptos::tachys::reactive_graph::OwnedView;

/// Glow intensities addressable through `data-glow` on `<html>`.
pub const GLOW_LEVELS: [f32; 5] = [0.0, 0.5, 1.0, 1.5, 2.0];

/// CRT scanline overlay, enabled by `data-scanlines="1"` (set from
/// `Effects::scanlines`).
pub const SCANLINES_CSS: &str = ":root[data-scanlines=\"1\"] body::after{content:\"\";position:fixed;inset:0;pointer-events:none;z-index:9999;\
background:repeating-linear-gradient(to bottom,rgba(0,0,0,.18) 0,rgba(0,0,0,.18) 1px,transparent 1px,transparent 3px)}";

/// Scanline stylesheet fragment.
pub fn scanlines_css() -> &'static str {
    SCANLINES_CSS
}

/// Stylesheet for runtime theme switching: each theme addressable as
/// `:root[data-theme="<id>"]`, components CSS, scanline overlay.
pub fn install_css(themes: &[Theme]) -> String {
    format!("{}\n{}", multi_theme_stylesheet(themes), SCANLINES_CSS)
}

/// Like [`install_css`] plus per-theme glow overrides
/// `:root[data-theme=…][data-glow="<n>"]` for each intensity in `levels`
/// (values are multipliers; `Theme::with_glow_intensity(base * n)`).
pub fn install_css_with(themes: &[Theme], levels: &[f32]) -> String {
    let mut s = install_css(themes);
    // Overrides must follow the base rules; components CSS only reads vars.
    s = s.replace(COMPONENTS_CSS, "");
    for t in themes {
        for l in levels {
            let v = theme_vars(&t.clone().with_glow_intensity(t.effects.glow_intensity * l));
            s.push_str(&format!(
                ":root[data-theme=\"{}\"][data-glow=\"{l}\"]{{{v}}}\n",
                t.id()
            ));
        }
    }
    s.push_str(COMPONENTS_CSS);
    s
}

/// Wraps children in a `data-theme` scope and emits the stylesheet.
///
/// `themes` is every selectable theme; `theme` the active id.
#[component]
pub fn ThemeProvider(
    /// All themes to generate CSS for.
    themes: Vec<Theme>,
    /// Active theme id (`data-theme`).
    #[prop(into)]
    theme: String,
    /// The app-wide surface: containers and input fields it frosts turn into glass over the
    /// page. Default: the design's own glass (frost is opt-in).
    #[prop(into, optional)]
    surface: dodai_core::Surface,
    children: Children,
) -> impl IntoView {
    let css = install_css(&themes);
    // The children are built under their own owner that carries the context: components run
    // when the view renders, after this function has returned.
    let owner = Owner::current().map_or_else(Owner::new, |o| o.child());
    let body = owner.with(|| {
        if let Some(t) = themes.iter().find(|t| t.id() == theme) {
            provide_context(crate::GlassContext {
                theme: t.clone(),
                fx: t.resolve_surface(&surface),
            });
        }
        children()
    });
    let body = OwnedView::new_with_owner(body, owner);
    let scan = themes
        .iter()
        .find(|t| t.id() == theme)
        .is_some_and(|t| t.effects.scanlines);
    let backdrop = themes
        .iter()
        .find(|t| t.id() == theme)
        .map_or(String::new(), dodai_topcoat::root_style);
    view! {
        <div class="dodai-root" data-theme=theme style=backdrop data-scanlines=if scan { "1" } else { "0" }>
            <style>{css}</style>
            {body}
        </div>
    }
}
