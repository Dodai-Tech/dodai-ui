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
//! | `neobrutalism` | [`dodai-neobrutalism`](https://docs.rs/dodai-neobrutalism) |
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
#[cfg(feature = "neobrutalism")]
pub use dodai_neobrutalism as neobrutalism;
#[cfg(feature = "shaders")]
pub use dodai_shaders as shaders;

#[cfg(feature = "icons")]
pub mod icons;

/// Built-in themes of the enabled design systems.
pub mod themes {
    pub use dodai_core::themes::{neutral, ThemeBuilder};

    #[cfg(feature = "cyberpunk")]
    pub use dodai_cyberpunk as cyberpunk;
    #[cfg(feature = "material")]
    pub use dodai_material as material;
    #[cfg(feature = "neobrutalism")]
    pub use dodai_neobrutalism as neobrutalism;

    /// A registered design system: its family id, display name and themes.
    #[derive(Clone, Copy)]
    pub struct DesignSystem {
        /// Family id shared by its themes (`Theme::meta.family`).
        pub family: &'static str,
        /// Display name.
        pub name: &'static str,
        /// Builds its themes (light/dark/accent variants).
        pub themes: fn() -> Vec<crate::Theme>,
    }

    /// The enabled design systems, in switcher order. **The one place a new design system is
    /// registered**: add a feature-gated entry here; the switcher, `all()`, `by_id()` and the
    /// `dist` export pick it up.
    #[allow(unused_mut, clippy::vec_init_then_push)]
    pub fn systems() -> Vec<DesignSystem> {
        let mut v: Vec<DesignSystem> = Vec::new();
        #[cfg(feature = "cyberpunk")]
        v.push(DesignSystem {
            family: "cyberpunk",
            name: "Cyberpunk",
            themes: dodai_cyberpunk::variants,
        });
        #[cfg(feature = "material")]
        v.push(DesignSystem {
            family: "material",
            name: "Material 3",
            themes: dodai_material::variants,
        });
        #[cfg(feature = "neobrutalism")]
        v.push(DesignSystem {
            family: "neobrutalism",
            name: "Neo-Brutalism",
            themes: dodai_neobrutalism::variants,
        });
        v
    }

    /// Every theme of the enabled design systems, in switcher order.
    /// Falls back to [`neutral`] if none is enabled.
    pub fn all() -> Vec<crate::Theme> {
        let mut v: Vec<crate::Theme> = systems().iter().flat_map(|s| (s.themes)()).collect();
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
