//! Glass for components: see-through fills and frosted panes from the app-wide surface
//! ([`ThemeProvider`](crate::ThemeProvider)'s `surface`) and a component's own
//! [`Surface`](dodai_core::Surface).

use dodai_core::{ResolvedSurface, Surface, SurfaceClass, Theme};
use leptos::prelude::*;

/// The theme and surface mix a [`ThemeProvider`](crate::ThemeProvider) hands to its subtree.
#[derive(Clone)]
pub struct GlassContext {
    /// The active theme.
    pub theme: Theme,
    /// The app-wide surface.
    pub fx: ResolvedSurface,
}

/// The CSS class (`" dodai-glass"`, or empty) and inline style variables a `class` surface needs
/// to be glass. Empty without a [`ThemeProvider`](crate::ThemeProvider), or when the fill stays
/// as designed.
pub fn glass(class: SurfaceClass, over: &Surface) -> (&'static str, String) {
    let Some(g) = use_context::<GlassContext>() else {
        return ("", String::new());
    };
    let s = dodai_topcoat::glass_style(&g.theme, class, over, g.fx);
    if s.is_empty() {
        ("", s)
    } else {
        (" dodai-glass", s)
    }
}
