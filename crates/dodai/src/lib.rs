//! # dodai
//!
//! One dependency for everything Dodai UI ships: the framework-agnostic
//! tokens ([`dodai_core`], re-exported at the root), every design system and
//! the shaders. Each part is a Cargo feature, so you can also take only what
//! you need, or depend on the individual crates directly:
//!
//! | feature | crate |
//! |---|---|
//! | `cyberpunk` | [`dodai-cyberpunk`](https://docs.rs/dodai-cyberpunk) (+ `dodai-cyberpunk-icons`) |
//! | `material` | [`dodai-material`](https://docs.rs/dodai-material) (+ `dodai-material-icons`) |
//! | `shaders` | [`dodai-shaders`](https://docs.rs/dodai-shaders) |
//!
//! ```
//! let mut sw = dodai::switcher();
//! assert!(sw.themes().len() >= 1);
//! let _ = sw.cycle_next();
//! ```

pub use dodai_core::*;

#[cfg(feature = "cyberpunk")]
pub use dodai_cyberpunk as cyberpunk;
#[cfg(feature = "material")]
pub use dodai_material as material;
#[cfg(feature = "shaders")]
pub use dodai_shaders as shaders;

#[cfg(feature = "icons")]
pub mod icons;

/// Built-in themes of the enabled design systems.
pub mod themes {
    pub use dodai_core::themes::{neutral, Spec};

    #[cfg(feature = "cyberpunk")]
    pub use dodai_cyberpunk as cyberpunk;
    #[cfg(feature = "material")]
    pub use dodai_material as material;

    /// Every theme of the enabled design systems, in switcher order
    /// (Cyberpunk, then Material). Falls back to [`neutral`] if none is enabled.
    pub fn all() -> Vec<crate::Theme> {
        #[allow(unused_mut)]
        let mut v: Vec<crate::Theme> = Vec::new();
        #[cfg(feature = "cyberpunk")]
        v.extend(dodai_cyberpunk::variants());
        #[cfg(feature = "material")]
        v.extend(dodai_material::variants());
        if v.is_empty() {
            v.push(neutral());
        }
        v
    }

    /// Look up a built-in theme by id.
    pub fn by_id(id: &str) -> Option<crate::Theme> {
        all().into_iter().find(|t| t.id() == id)
    }
}

/// A [`ThemeSwitcher`] over [`themes::all`].
pub fn switcher() -> ThemeSwitcher {
    ThemeSwitcher::with_themes(themes::all())
}
