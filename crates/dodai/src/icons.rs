//! Toolkit-neutral icons: the right glyph for a theme, as SVG or as a coverage mask, so any
//! adapter can draw it. Icons are named by the typed [`IconId`]; there are no name strings.
//!
//! Each design system draws every [`IconId`] in its own style (see [`dodai_icons`]):
//! Cyberpunk's hand-drawn straight segments, Material 3's Material Symbols, Neo-Brutalism's bold
//! Phosphor. Which one a theme uses is declared by its `Look::icons`.
//!
//! ```
//! # #[cfg(all(feature = "cyberpunk", feature = "material"))] {
//! use dodai::icons::IconId;
//! let t = dodai::themes::neutral();
//! let g = dodai::icons::glyph(IconId::Home, &t, false).unwrap();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(g.coverage(24).len(), 24 * 24);
//! # }
//! ```

use crate::Theme;

pub use dodai_icons::{Glyph, IconId, Pack, Points, STROKE};

#[cfg(feature = "cyberpunk")]
pub use dodai_icons::Modifier;

/// Resolve `id` for `theme`'s design system. `filled` picks the filled form where there is one.
pub fn glyph(id: IconId, theme: &Theme, filled: bool) -> Option<Glyph> {
    dodai_icons::glyph_for(id, theme.look().icons, filled)
}
