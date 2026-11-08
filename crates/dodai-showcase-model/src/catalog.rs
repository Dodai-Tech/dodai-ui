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
    /// Sidebar entry (also what the filter and the scroll-spy match).
    pub title: &'static str,
    /// Title on the panel itself (often shorter than the sidebar entry).
    pub heading: &'static str,
    /// Whether the active theme shows it (a design-only component or effect).
    pub shown: fn(&Theme) -> bool,
    /// Builds the demo.
    pub build: fn(&Ctx) -> Node,
}

impl Section {
    /// A section every design shows, its panel titled like its sidebar entry.
    pub const fn new(category: &'static str, title: &'static str, build: fn(&Ctx) -> Node) -> Self {
        Self {
            category,
            title,
            heading: title,
            shown: |_| true,
            build,
        }
    }

    /// A panel title other than the sidebar entry.
    pub const fn heading(mut self, heading: &'static str) -> Self {
        self.heading = heading;
        self
    }

    /// Shown only for themes where `shown` holds.
    pub const fn when(mut self, shown: fn(&Theme) -> bool) -> Self {
        self.shown = shown;
        self
    }
}

/// The categories this port shows, in sidebar order. (Charts, Instruments and Shaders are
/// not ported to the other toolkits yet.)
pub const CATEGORIES: [&str; 10] = [
    "Foundations",
    "Shape",
    "Surface effects",
    "Actions",
    "Data display",
    "Navigation",
    "Inputs",
    "Loading",
    "Animation",
    "Shaders",
];

/// Every section, grouped by category in [`CATEGORIES`] order.
pub fn sections() -> &'static [Section] {
    static ALL: std::sync::LazyLock<Vec<Section>> = std::sync::LazyLock::new(|| {
        let mut all = Vec::new();
        all.extend(page::foundations::SECTIONS);
        all.extend(page::shape::SECTIONS);
        all.extend(page::surface::SECTIONS);
        all.extend(page::actions::SECTIONS);
        all.extend(page::data::SECTIONS);
        all.extend(page::navigation::SECTIONS);
        all.extend(page::inputs::SECTIONS);
        all.extend(page::loading::SECTIONS);
        all.extend(page::animation::SECTIONS);
        all.extend(page::shaders::SECTIONS);
        all
    });
    &ALL
}
