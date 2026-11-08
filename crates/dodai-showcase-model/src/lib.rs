//! # dodai-showcase-model
//!
//! The Dodai showcase as data. The gpui showcase builds its widgets directly; the other
//! toolkits (bevy, iced, dioxus, ...) show the same pages by walking this model:
//!
//! * [`Showcase`] is the state (theme, design, mode, colour, filter, every demo's value) and
//!   [`Showcase::update`] the single place events land, so behaviour is tested once.
//! * [`catalog::sections`] lists the sections of each category; a section builds a [`Node`]
//!   tree from the active theme.
//! * [`Node`] is the demo widget vocabulary a toolkit has to draw.
//!
//! Nothing here depends on a toolkit.

pub mod catalog;
pub mod md;
pub mod media;
pub mod node;
pub mod plot;
mod sections;
pub mod shader_state;
pub mod state;
pub mod widgets;

pub use catalog::{sections, Ctx, Section, CATEGORIES};
pub use node::{b, Axis, Bind, Gap, Node, ShaderSpec, Stack, TextRole};
pub use state::{Action, ModePref, Showcase, Value, Values, SPEEDS};
