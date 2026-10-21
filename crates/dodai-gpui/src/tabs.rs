//! [`Tabs`]: a row of selectable tabs driven by `theme.tab` tokens.

use crate::interact::{track, use_interact};
use crate::paint::{bottom_band_canvas, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::Theme;
use gpui::{
    div, px, App, ElementId, Hsla, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

/// Resolved tab paint: shape style, text colour, optional indicator colour.
pub fn tab_style(theme: &Theme, active: bool, hovered: bool) -> (ShapeStyle, Hsla, Option<Hsla>) {
    let k = theme.tab(active, hovered);
    (
        ShapeStyle::from_tokens(theme, k.background, None, active, false),
        hsla(k.foreground),
        k.indicator.map(hsla),
    )
}

type SelectHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Tab strip. Controlled: pass `active` and update it in `on_select`.
#[derive(IntoElement)]
pub struct Tabs {
    id: ElementId,
    labels: Vec<SharedString>,
    active: usize,
    on_select: Option<SelectHandler>,
    corners: Option<dodai_core::Corners>,
    icons: Vec<crate::icons::IconName>,
    secondary: bool,
    scrollable: bool,
}

crate::corners_setter!(Tabs);

impl Tabs {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            labels: Vec::new(),
            active: 0,
            on_select: None,
            icons: Vec::new(),
            secondary: false,
            scrollable: false,
        }
    }
    /// Scrollable tabs: keep every tab at its natural width and scroll the strip sideways when
    /// it overflows (instead of squeezing the labels).
    pub fn scrollable(mut self, on: bool) -> Self {
        self.scrollable = on;
        self
    }
    /// One icon per tab, stacked above the label (icon-only when there are no labels).
    pub fn icons(mut self, icons: impl IntoIterator<Item = crate::icons::IconName>) -> Self {
        self.icons = icons.into_iter().collect();
        self
    }
    /// Secondary tabs: an on-surface label and a full-width underline, for sub-navigation.
    pub fn secondary(mut self, on: bool) -> Self {
        self.secondary = on;
        self
    }
    /// Tab labels in order.
    pub fn labels<S: Into<SharedString>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }
    /// Index of the active tab.
    pub fn active(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    crate::callback!(
        /// Called with the clicked tab index.
        on_select, usize
    );
}

impl Tabs {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut row = div().flex().gap(px(t.spacing.xs));
        let count = self.labels.len().max(self.icons.len());
        for i in 0..count {
            let label = self.labels.get(i);
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let active = i == self.active;
            let p = crate::motion::transition(window, cx, id.clone(), active);
            let (mut style, fg, indicator) = tab_style(&t, active, hovered);
            let (on_s, on_fg, _) = tab_style(&t, true, hovered);
            let (off_s, off_fg, _) = tab_style(&t, false, hovered);
            style.fill = crate::motion::mix_opt(off_s.fill, on_s.fill, p);
            let fg = if p > 0.0 && p < 1.0 {
                crate::motion::mix(off_fg, on_fg, p)
            } else {
                fg
            };
            let indicator = indicator
                .or_else(|| tab_style(&t, true, hovered).2.filter(|_| p > 0.01))
                .map(|c| c.opacity(p));
            let fg = if self.secondary && active {
                hsla(t.palette.roles.on_surface)
            } else {
                fg
            };
            let stacked = label.is_some() && self.icons.get(i).is_some();
            let height = if stacked && !self.secondary {
                t.shape.control_height * 1.6
            } else {
                t.shape.control_height
            };
            let mut tab = track(shaped_id(id, &t.shape, style), &st)
                .h(px(height))
                .px(px(t.spacing.md))
                .flex()
                .items_center()
                .cursor_pointer()
                .font_family(first_family(t.typography.body))
                .text_size(px(t.typography.size_md))
                .text_color(fg);
            if t.look().tonal {
                tab = crate::text::typeset(tab, &t, dodai_core::TypeRole::TitleSmall);
            }
            if stacked && !self.secondary {
                tab = tab.flex_col().justify_center().gap(px(2.0));
            } else {
                tab = tab.gap(px(t.spacing.sm));
            }
            if let Some(icon) = self.icons.get(i) {
                tab = tab.child(crate::icons::Icon::new(*icon).size(20.0).color(fg));
            }
            if let Some(label) = label {
                tab = tab.child(format_label(&t.typography, label));
            }
            if let Some(c) = indicator {
                tab = tab.child(bottom_band_canvas(&t.shape, t.shape.border_width, c));
            }
            if let Some(f) = self.on_select.clone() {
                tab = tab.on_click(move |_, w, cx| f(i, w, cx));
            }
            if self.scrollable {
                tab = tab.flex_none().whitespace_nowrap();
            }
            row = row.child(tab);
        }
        if self.scrollable {
            return gpui::InteractiveElement::id(div(), (self.id.clone(), "scroll"))
                .w_full()
                .overflow_x_scroll()
                .child(row.flex_none())
                .into_any_element();
        }
        row.into_any_element()
    }
}

crate::entry_render!(Tabs);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::light_rounded;

    #[test]
    fn active_tab_uses_indicator_tokens() {
        let t = light_rounded();
        let (s, fg, ind) = tab_style(&t, true, false);
        // M3 primary tab: no fill, `primary` label and indicator.
        assert!(s.fill.is_none());
        assert_eq!(fg, hsla(t.palette.primary));
        assert_eq!(ind, Some(hsla(t.palette.primary)));
        let (s, fg, ind) = tab_style(&t, false, false);
        assert!(s.fill.is_none() && ind.is_none());
        assert_eq!(fg, hsla(t.palette.roles.on_surface_variant));
        let (s, _, _) = tab_style(&t, false, true);
        assert_eq!(
            s.fill,
            Some(hsla(t.palette.roles.on_surface.with_alpha(0.08)))
        );
    }
}
