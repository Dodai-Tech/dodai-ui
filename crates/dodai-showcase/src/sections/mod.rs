//! One module per showcase category; each builds only its own widgets.

mod actions;
mod charts_layered;
mod charts_more;
mod charts_sec;
mod charts_status;
mod data_display;
mod foundations_sec;
mod inputs;
mod instruments_sec;
mod navigation;
pub(crate) mod shape;
mod throughput;

use super::*;

pub(crate) use foundations_sec::{icon_total, icon_tree, IconEntry, IconPick};
pub(crate) use throughput::WINDOW as THROUGHPUT_WINDOW;

/// Every category, in sidebar order (Shaders last: it is the biggest).
pub(crate) const CATEGORIES: [&str; 12] = [
    "Foundations",
    "Shape",
    "Surface effects",
    "Actions",
    "Data display",
    "Navigation",
    "Inputs",
    "Charts",
    "Instruments",
    "Loading",
    "Animation",
    "Shaders",
];

impl Showcase {
    /// The panel every section starts from: title, design chips, replay scope.
    pub(crate) fn section(&self, title: &'static str, cx: &mut Context<Self>) -> Panel {
        Panel::new()
            .title(title.to_string())
            .scope(Some(self.scope_of(title, cx)))
            .scope_key(self.sec_replay.get(title).copied().unwrap_or(0))
    }

    /// Picks the selection animation the demo components are given ("Design's own" passes
    /// `None`); only the value travels, the components do the animating.
    pub(crate) fn animation_picker(&self, id: &'static str, cx: &mut Context<Self>) -> Segmented {
        use dodai_gpui::motion::SelectionMotion;
        let me = cx.entity();
        let at = self
            .animation
            .and_then(|a| SelectionMotion::ALL.iter().position(|&b| b == a))
            .map_or(0, |i| i + 1);
        Segmented::new(
            id,
            std::iter::once("Design's own").chain(SelectionMotion::ALL.map(|a| a.label())),
        )
        .selected(at)
        .on_select(move |i, _, cx| {
            me.update(cx, |s, cx| {
                s.animation = i.checked_sub(1).map(|i| SelectionMotion::ALL[i]);
                cx.notify();
            })
        })
    }

    /// The group builders of a category. Each builds a few related sections; they are called
    /// only when one of their sections is on screen.
    pub(crate) fn groups_of(cat: &str) -> Vec<Group> {
        match cat {
            "Foundations" => Self::groups_foundations_sec(),
            "Shape" | "Surface effects" => Self::groups_shape(),
            "Actions" => Self::groups_actions(),
            "Data display" => Self::groups_data_display(),
            "Navigation" => Self::groups_navigation(),
            "Inputs" => Self::groups_inputs(),
            "Charts" => vec![Self::sections_charts_sec],
            "Instruments" => vec![Self::sections_instruments],
            "Loading" => vec![Self::loading_group],
            "Animation" => vec![Self::animation_group],
            _ => vec![Self::shaders_group],
        }
    }

    /// One group, without the sections that belong to another design (see `catalog::owner`) or
    /// another category (the Shape and Surface-effects pages share their builders).
    pub(crate) fn build_group(
        &mut self,
        cat: &str,
        group: Group,
        cx: &mut Context<Self>,
    ) -> Vec<Section> {
        let family = theme_of(cx).meta.family;
        let mut v = group(self, cx);
        v.retain(|(c, title, _, _)| {
            *c == cat && super::catalog::owner(title).is_none_or(|o| o.contains(&family))
        });
        v
    }

    fn loading_group(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let p = self.loading_section(cx);
        let t = theme_of(cx);
        let li = Panel::new()
            .title("Loading indicator")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.lg))
                    .child(LoadingIndicator::new("li-plain"))
                    .child(LoadingIndicator::new("li-contained").contained(true)),
            )
            .into_any_element();
        vec![
            (
                "Loading",
                "Spinner · Indeterminate · Skeleton",
                p.into_any_element(),
                "Loading",
            ),
            ("Loading", "Loading indicator", li, "Loading indicator"),
        ]
    }

    fn animation_group(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let (els, titles) = self.animation_sections(cx);
        els.into_iter()
            .zip(titles)
            .map(|(el, k)| ("Animation", k, el, k))
            .collect()
    }

    fn shaders_group(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let me = cx.entity();
        let mut v = Vec::new();
        self.shader_sections(&me, &mut v, cx);
        v
    }

    /// `(category, title)` of every section, built once up front. Also records each section's
    /// key and the group that builds it (`self.section_group`).
    pub(crate) fn section_meta(
        &mut self,
        cx: &mut Context<Self>,
    ) -> Vec<(&'static str, &'static str)> {
        let mut out = Vec::new();
        self.section_group.clear();
        for cat in CATEGORIES {
            for (gi, group) in Self::groups_of(cat).into_iter().enumerate() {
                for (c, title, _, key) in self.build_group(cat, group, cx) {
                    out.push((c, title));
                    self.section_group.insert(title, (cat, gi, key));
                }
            }
        }
        out
    }
}
