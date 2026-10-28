//! [`ThemeSwitcherBar`] (one button per theme) and [`ThemeTree`].

use crate::button::Button;
use crate::hsla;
use crate::list::ListItem;
use crate::select::{family_title, DesignSystemSelector};
use crate::{select_theme, ActiveDodai};
use dodai_core::ButtonVariant;
use gpui::{px, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window};

/// Row of buttons, one per theme in the global switcher; the active one is filled.
#[derive(IntoElement)]
pub struct ThemeSwitcherBar {
    id: ElementId,
}

impl ThemeSwitcherBar {
    /// Create a new [`ThemeSwitcherBar`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self { id: id.into() }
    }
}

impl RenderOnce for ThemeSwitcherBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let lt = cx.dodai();
        let gap = lt.theme().spacing.sm;
        let active = lt.switcher().index();
        let names: Vec<_> = lt.switcher().themes().iter().map(|t| t.name()).collect();
        let mut row = gpui::div().flex().flex_wrap().gap(px(gap));
        for (i, name) in names.into_iter().enumerate() {
            row = row.child(
                Button::new((self.id.clone(), i.to_string()), name)
                    .variant(if i == active {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| select_theme(cx, i)),
            );
        }
        row
    }
}

/// Design systems with their themes as children. Few systems render as an
/// expandable tree; more than `dropdown_over` collapse into a dropdown with the
/// active system's themes listed below. New systems appear automatically.
struct TreeExpanded(bool);
impl gpui::Global for TreeExpanded {}

/// Whether the active design system's children are shown (default `true`).
pub fn tree_expanded(cx: &App) -> bool {
    cx.try_global::<TreeExpanded>().is_none_or(|g| g.0)
}

/// Show / hide the active design system's children in every
/// [`ThemeTree`]. Call with `true` to re-open it from elsewhere
/// (e.g. when a section is picked); the caller must notify its view.
pub fn set_tree_expanded(cx: &mut App, open: bool) {
    cx.set_global(TreeExpanded(open));
    cx.refresh_windows();
}

/// Tree of design systems and their themes for picking a theme.
#[derive(IntoElement)]
pub struct ThemeTree {
    id: ElementId,
    dropdown_over: usize,
    props: Vec<(&'static str, gpui::AnyElement)>,
}

impl ThemeTree {
    /// Create a new [`ThemeTree`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            dropdown_over: 4,
            props: Vec::new(),
        }
    }
    /// Family-specific settings (e.g. glow, scanlines), shown under that family's
    /// themes while it is the active design system.
    pub fn properties(mut self, family: &'static str, el: impl IntoElement) -> Self {
        self.props.push((family, el.into_any_element()));
        self
    }
    /// Switch to a dropdown when there are more than `n` design systems.
    pub fn dropdown_over(mut self, n: usize) -> Self {
        self.dropdown_over = n;
        self
    }
}

impl RenderOnce for ThemeTree {
    fn render(mut self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = crate::theme_of(cx);
        // Expanded state of the active family's children (collapsible).
        let is_expanded = tree_expanded(cx);
        let lt = cx.dodai();
        let sw = lt.switcher();
        let (cur_idx, cur_fam) = (sw.index(), sw.current_family());
        let fams = sw.families();
        let as_dropdown = fams.len() > self.dropdown_over;
        let mut tree = gpui::div().flex().flex_col().gap(px(2.0));
        if as_dropdown {
            tree = tree.child(DesignSystemSelector::new((self.id.clone(), "dd")));
        }
        for (fi, fam) in fams.into_iter().enumerate() {
            let open = fam == cur_fam;
            if !as_dropdown {
                let first = sw.family_indices(fam)[0];
                tree = tree.child(
                    ListItem::new(
                        (self.id.clone(), format!("fam{fi}")),
                        format!(
                            "{} {}",
                            if open && is_expanded { "▾" } else { "▸" },
                            family_title(fam)
                        ),
                    )
                    .selected(open)
                    .on_click(move |_, _, cx| {
                        // Active family: toggle its children. Other: switch + expand.
                        set_tree_expanded(cx, if open { !tree_expanded(cx) } else { true });
                        if !open {
                            select_theme(cx, first);
                        }
                    }),
                );
            }
            if open && (is_expanded || as_dropdown) {
                let mut kids = gpui::div()
                    .ml(px(if as_dropdown { 0.0 } else { t.spacing.md }))
                    .pl(px(t.spacing.md))
                    // Material: nesting is shown by indent alone, no guide line.
                    .border_l(px(if t.look().tonal { 0.0 } else { 1.0 }))
                    .border_color(hsla(t.palette.border_strong))
                    .flex()
                    .flex_col()
                    .gap(px(2.0));
                for &ti in &sw.family_indices(fam) {
                    kids = kids.child(
                        ListItem::new(
                            (self.id.clone(), format!("th{ti}")),
                            short_name(sw.themes()[ti].name()),
                        )
                        .selected(ti == cur_idx)
                        .on_click(move |_, _, cx| select_theme(cx, ti)),
                    );
                }
                if let Some(i) = self.props.iter().position(|(f, _)| *f == fam) {
                    let (_, el) = self.props.remove(i);
                    kids = kids.child(
                        gpui::div()
                            .pt(px(t.spacing.sm))
                            .flex()
                            .flex_col()
                            .gap(px(t.spacing.xs))
                            .child(
                                gpui::div()
                                    .text_size(px(t.typography.size_xs))
                                    .text_color(hsla(t.palette.text_muted))
                                    .child("PROPERTIES"),
                            )
                            .child(el),
                    );
                }
                tree = tree.child(kids);
            }
        }
        tree
    }
}

/// Theme name without its design-system prefix (`"Cyberpunk · Red"` → `"Red"`).
fn short_name(name: &str) -> String {
    name.split_once(" · ").map_or(name, |(_, r)| r).to_string()
}

#[cfg(test)]
mod tests {
    use super::short_name;

    #[test]
    fn strips_family_prefix() {
        assert_eq!(short_name("Cyberpunk · Red"), "Red");
        assert_eq!(short_name("Plain"), "Plain");
    }
}
