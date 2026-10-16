//! [`ThemeSwitcherBar`]: one button per available theme.

use crate::button::Button;
use crate::{select_theme, ActiveDodai};
use gpui::{px, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window};
use dodai_core::ButtonVariant;

/// Row of buttons, one per theme in the global switcher; the active one is filled.
#[derive(IntoElement)]
pub struct ThemeSwitcherBar {
    id: ElementId,
}

impl ThemeSwitcherBar {
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
