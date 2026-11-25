//! # dodai-ui
//!
//! One dependency for the Dodai UI design systems. The engine ([`core`]: tokens, themes, shapes,
//! component recipes) is always there; designs, building blocks and toolkits are features:
//!
//! ```toml
//! dodai-ui = { version = "0.1", features = ["iced", "cyberpunk", "material"] }
//! ```
//!
//! ```ignore
//! use dodai_ui::core::ThemeSwitcher;
//! use dodai_ui::iced::IcedTheme;
//! ```
//!
//! | feature | module |
//! | --- | --- |
//! | `dodai`, `cyberpunk`, `material`, `neobrutalism` (`all-designs`) | the designs inside [`core`] |
//! | `icons`, `charts`, `dash`, `fx`, `frost`, `shaders` | the module of the same name (`dodai_ui::icons`, `dodai_ui::charts`, ...) |
//! | `iced`, `gpui`, `egui`, `bevy`, `slint`, `dioxus`, `leptos`, `yew`, `tui`, `topcoat` | the toolkit of that name |
//!
//! Freya and xilem live in their own workspaces (their GPU stacks do not unify with the others)
//! and are used through `dodai-freya` and `dodai-xilem` directly.
#![forbid(unsafe_code)]

pub use dodai_core as core;
pub use dodai_core::*;

#[cfg(feature = "charts")]
pub use dodai_charts as charts;
#[cfg(feature = "dash")]
pub use dodai_dash as dash;
#[cfg(feature = "frost")]
pub use dodai_frost as frost;
#[cfg(feature = "fx")]
pub use dodai_fx as fx;
#[cfg(feature = "icons")]
pub use dodai_icons as icons;
#[cfg(feature = "shaders")]
pub use dodai_shaders as shaders;

#[cfg(feature = "bevy")]
pub use dodai_bevy as bevy;
#[cfg(feature = "dioxus")]
pub use dodai_dioxus as dioxus;
#[cfg(feature = "egui")]
pub use dodai_egui as egui;
#[cfg(feature = "gpui")]
pub use dodai_gpui as gpui;
#[cfg(feature = "iced")]
pub use dodai_iced as iced;
#[cfg(feature = "leptos")]
pub use dodai_leptos as leptos;
#[cfg(feature = "slint")]
pub use dodai_slint as slint;
#[cfg(feature = "topcoat")]
pub use dodai_topcoat as topcoat;
#[cfg(feature = "tui")]
pub use dodai_tui as tui;
#[cfg(feature = "yew")]
pub use dodai_yew as yew;
