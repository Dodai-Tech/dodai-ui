//! Glass for components: see-through fills and frosted panes from the app-wide surface
//! ([`ThemeProvider`](crate::ThemeProvider)'s `surface`) and a component's own
//! [`Surface`](dodai_ui_core::Surface).

use crate::use_theme;
use dodai_ui_core::{Surface, SurfaceClass};
use yew::prelude::*;

/// What a `class` surface needs to be glass: the extra CSS class (`" dodai-glass"`, or empty)
/// and the inline style variables (`None` when the fill stays as designed). Needs a
/// [`ThemeProvider`](crate::ThemeProvider); without one the fill stays as designed.
#[hook]
pub fn use_glass(class: SurfaceClass, over: &Surface) -> (&'static str, Option<AttrValue>) {
    let Some(ctx) = use_theme() else {
        return ("", None);
    };
    let s = dodai_ui_topcoat::glass_style(&ctx.theme, class, over, ctx.surface);
    if s.is_empty() {
        ("", None)
    } else {
        (" dodai-glass", Some(AttrValue::from(s)))
    }
}
