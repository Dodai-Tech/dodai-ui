//! One module per showcase category; each builds only its own widgets.

mod actions;
mod charts_sec;
mod data_display;
mod foundations_sec;
mod inputs;
mod navigation;
mod shape;

use super::*;

/// Every category, in sidebar order (Shaders last: it is the biggest).
pub(crate) const CATEGORIES: [&str; 10] = [
    "Foundations",
    "Shape",
    "Actions",
    "Data display",
    "Navigation",
    "Inputs",
    "Charts",
    "Loading",
    "Animation",
    "Shaders",
];

impl Showcase {
    /// Build the sections of one category (nothing else is constructed).
    pub(crate) fn sections_of(&mut self, cat: &str, cx: &mut Context<Self>) -> Vec<Section> {
        match cat {
            "Foundations" => self.sections_foundations_sec(cx),
            "Shape" => self.sections_shape(cx),
            "Actions" => self.sections_actions(cx),
            "Data display" => self.sections_data_display(cx),
            "Navigation" => self.sections_navigation(cx),
            "Inputs" => self.sections_inputs(cx),
            "Charts" => self.sections_charts_sec(cx),
            "Loading" => {
                let p = self.loading_section(cx);
                vec![(
                    "Loading",
                    "Spinner · Indeterminate · Skeleton",
                    p.into_any_element(),
                    "Loading",
                )]
            }
            "Animation" => {
                let (els, titles) = self.animation_sections(cx);
                els.into_iter()
                    .zip(titles)
                    .map(|(el, k)| ("Animation", k, el, k))
                    .collect()
            }
            _ => {
                let me = cx.entity();
                let mut v = Vec::new();
                self.shader_sections(&me, &mut v, cx);
                v
            }
        }
    }

    /// `(category, title)` of every section, built once up front.
    pub(crate) fn section_meta(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Vec<(&'static str, &'static str)> {
        let mut out = Vec::new();
        for cat in CATEGORIES {
            for (c, title, _, _) in self.sections_of(cat, cx) {
                out.push((c, title));
            }
        }
        out
    }
}
