//! The pages and sections of the showcase, in sidebar order.

use dodai_core::Theme;

use crate::node::Node;
use crate::sections as page;
use crate::state::Values;

/// Everything a section needs to build its demo.
pub struct Ctx<'a> {
    /// The active theme (control colour applied).
    pub theme: &'a Theme,
    /// The demos' values (for sections that show a pick's result).
    pub values: &'a Values,
}

/// One titled panel of a category.
pub struct Section {
    /// The category (page) it belongs to; one of [`CATEGORIES`].
    pub category: &'static str,
    /// Panel title and sidebar entry.
    pub title: &'static str,
    /// Builds the demo.
    pub build: fn(&Ctx) -> Node,
}

/// The categories this port shows, in sidebar order. (Charts, Instruments and Shaders are
/// not ported to the other toolkits yet.)
pub const CATEGORIES: [&str; 6] = [
    "Foundations",
    "Actions",
    "Data display",
    "Navigation",
    "Inputs",
    "Loading",
];

/// Every section, grouped by category in [`CATEGORIES`] order.
pub fn sections() -> &'static [Section] {
    static ALL: std::sync::LazyLock<Vec<Section>> = std::sync::LazyLock::new(|| {
        let mut all = Vec::new();
        all.extend(page::foundations::SECTIONS);
        all.extend(page::actions::SECTIONS);
        all.extend(page::data::SECTIONS);
        all.extend(page::navigation::SECTIONS);
        all.extend(page::inputs::SECTIONS);
        all.extend(page::loading::SECTIONS);
        all
    });
    &ALL
}
