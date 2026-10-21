//! [`NavigationDrawer`] and [`NavigationRail`]: app-level navigation following the Material 3
//! specs (drawer: 56 px pill rows, section headlines, dividers, `surface-container-low`; rail:
//! 80 px wide, 56×32 indicator pill, label below the icon, badge). Other designs keep their own
//! shape and colours through the same theme tokens.

use crate::interact::{track, use_interact};
use crate::paint::{round_shape, shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label, typeset};
use crate::{hsla, theme_of};
use dodai_core::{Theme, TypeRole};
use gpui::{
    div, px, App, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};

type Click = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

/// One destination of a drawer or rail.
pub struct NavigationItem {
    id: ElementId,
    label: SharedString,
    icon: Option<crate::icons::IconName>,
    badge: Option<SharedString>,
    selected: bool,
    depth: u8,
    on_click: Option<Click>,
}

impl NavigationItem {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            selected: false,
            depth: 0,
            on_click: None,
        }
    }
    /// Nest the row `levels` deep (drawer only): indented, shorter, no icon.
    pub fn depth(mut self, levels: u8) -> Self {
        self.depth = levels;
        self
    }
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Count or short text at the end of a drawer row / on a rail icon.
    pub fn badge(mut self, b: impl Into<SharedString>) -> Self {
        self.badge = Some(b.into());
        self
    }
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

/// Colours of a destination: (indicator fill, foreground, label foreground) (pure).
pub fn nav_colors(t: &Theme, selected: bool, hovered: bool) -> (Option<Hsla>, Hsla) {
    let r = &t.palette.roles;
    if t.look().tonal {
        if selected {
            (
                Some(hsla(r.secondary_container)),
                hsla(r.on_secondary_container),
            )
        } else {
            let hover = hovered.then(|| hsla(r.on_surface_variant).opacity(0.08));
            (hover, hsla(r.on_surface_variant))
        }
    } else {
        let k = t.list_item(selected, hovered);
        (k.background.map(hsla), hsla(k.foreground))
    }
}

fn icon_el(i: crate::icons::IconName, fg: Hsla) -> impl IntoElement {
    crate::icons::Icon::new(i).size(24.0).color(fg)
}

enum Entry {
    Item(NavigationItem),
    Headline(SharedString),
    Divider,
}

/// Vertical drawer: items, section headlines and dividers.
#[derive(IntoElement)]
pub struct NavigationDrawer {
    entries: Vec<Entry>,
    max_width: f32,
}

impl NavigationDrawer {
    pub fn new(_id: impl Into<ElementId>) -> Self {
        Self {
            entries: Vec::new(),
            max_width: 360.0,
        }
    }
    pub fn item(mut self, i: NavigationItem) -> Self {
        self.entries.push(Entry::Item(i));
        self
    }
    /// Section headline above the following items.
    pub fn headline(mut self, t: impl Into<SharedString>) -> Self {
        self.entries.push(Entry::Headline(t.into()));
        self
    }
    pub fn divider(mut self) -> Self {
        self.entries.push(Entry::Divider);
        self
    }
    /// Spec width cap (360 px by default); the drawer fills its parent up to it.
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = px;
        self
    }
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let tonal = t.look().tonal;
        let muted = hsla(t.palette.roles.on_surface_variant);
        let mut col = div()
            .flex()
            .flex_col()
            .w_full()
            .max_w(px(self.max_width))
            .gap(px(if tonal { 0.0 } else { t.spacing.xs }))
            .px(px(if tonal { 12.0 } else { t.spacing.sm }))
            .py(px(if tonal { 12.0 } else { t.spacing.sm }));
        if tonal {
            col = col.bg(hsla(t.palette.roles.surface_container_low));
        }
        for e in self.entries {
            match e {
                Entry::Headline(h) => {
                    col = col.child(
                        typeset(div(), &t, TypeRole::TitleSmall)
                            .h(px(56.0))
                            .px(px(16.0))
                            .flex()
                            .items_center()
                            .text_color(muted)
                            .child(h),
                    );
                }
                Entry::Divider => {
                    col = col.child(
                        div()
                            .my(px(8.0))
                            .mx(px(16.0))
                            .h(px(1.0))
                            .bg(hsla(t.palette.roles.outline_variant)),
                    );
                }
                Entry::Item(it) => {
                    col = col.child(drawer_item(it, &t, window, cx));
                }
            }
        }
        col
    }
}

fn drawer_item(
    it: NavigationItem,
    t: &Theme,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let tonal = t.look().tonal;
    let st = use_interact(&it.id, window, cx);
    let hovered = st.read(cx).hovered;
    let (fill, fg) = nav_colors(t, it.selected, hovered);
    let nested = it.depth > 0;
    let h = match (tonal, nested) {
        (true, false) => 56.0,
        (true, true) => 40.0,
        (false, n) => t.list_row_height(n),
    };
    let shape = round_shape(&t.shape, h);
    let style = ShapeStyle::from_tokens(t, None, None, false, false);
    let style = ShapeStyle { fill, ..style };
    let mut el = track(shaped_id(it.id, &shape, style), &st)
        .h(px(h))
        .ml(px(it.depth as f32 * 16.0))
        .px(px(if tonal { 16.0 } else { t.spacing.md }))
        .flex()
        .items_center()
        .gap(px(12.0))
        .cursor_pointer()
        .text_color(fg)
        .font_family(first_family(t.typography.body));
    if let Some(i) = it.icon.filter(|_| !nested) {
        el = el.child(icon_el(i, fg));
    }
    el = el.child(
        typeset(
            div(),
            t,
            if nested {
                TypeRole::LabelMedium
            } else {
                TypeRole::LabelLarge
            },
        )
        .flex_1()
        .min_w_0()
        .child(format_label(&t.typography, &it.label)),
    );
    if let Some(b) = it.badge {
        el = el.child(typeset(div(), t, TypeRole::LabelLarge).child(b));
    }
    if let Some(h) = it.on_click {
        el = el.on_click(move |e, w, cx| h(e, w, cx));
    }
    el
}

/// Narrow icon rail: 80 px wide, label under each icon, indicator pill behind the icon.
#[derive(IntoElement)]
pub struct NavigationRail {
    items: Vec<NavigationItem>,
    header: Option<gpui::AnyElement>,
}

impl NavigationRail {
    pub fn new(_id: impl Into<ElementId>) -> Self {
        Self {
            items: Vec::new(),
            header: None,
        }
    }
    pub fn item(mut self, i: NavigationItem) -> Self {
        self.items.push(i);
        self
    }
    /// Element above the destinations (menu button, FAB).
    pub fn header(mut self, e: impl IntoElement) -> Self {
        self.header = Some(e.into_any_element());
        self
    }
}

/// Spec width of the rail (px).
pub const RAIL_WIDTH: f32 = 80.0;

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let tonal = t.look().tonal;
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .w(px(RAIL_WIDTH))
            .flex_none()
            .gap(px(12.0))
            .py(px(12.0));
        if tonal {
            col = col.bg(hsla(t.palette.roles.surface_container_low));
        }
        if let Some(h) = self.header {
            col = col.child(h);
        }
        for it in self.items {
            let st = use_interact(&it.id, window, cx);
            let hovered = st.read(cx).hovered;
            let (fill, fg) = nav_colors(&t, it.selected, hovered);
            let label_fg = if tonal && it.selected {
                hsla(t.palette.roles.on_surface)
            } else if tonal {
                hsla(t.palette.roles.on_surface_variant)
            } else {
                fg
            };
            let pill_shape = round_shape(&t.shape, 32.0);
            let pill = shaped(
                &pill_shape,
                ShapeStyle {
                    fill,
                    ..ShapeStyle::from_tokens(&t, None, None, false, false)
                },
            )
            .w(px(56.0))
            .h(px(32.0))
            .flex()
            .items_center()
            .justify_center()
            .relative()
            .children(it.icon.map(|i| icon_el(i, fg)))
            .children(it.badge.map(|b| {
                div()
                    .absolute()
                    .top(px(-2.0))
                    .right(px(6.0))
                    .child(crate::Badge::new(b, dodai_core::Tone::Danger))
            }));
            let mut el = track(div().id(it.id.clone()), &st)
                .flex()
                .flex_col()
                .items_center()
                .gap(px(4.0))
                .w_full()
                .cursor_pointer()
                .child(pill)
                .child(
                    typeset(div(), &t, TypeRole::LabelMedium)
                        .text_color(label_fg)
                        .child(format_label(&t.typography, &it.label)),
                );
            if let Some(h) = it.on_click {
                el = el.on_click(move |e, w, cx| h(e, w, cx));
            }
            col = col.child(el);
        }
        col
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn tonal_destinations_use_the_secondary_container_pill() {
        let t = light_rounded();
        let (fill, fg) = nav_colors(&t, true, false);
        assert_eq!(fill, Some(hsla(t.palette.roles.secondary_container)));
        assert_eq!(fg, hsla(t.palette.roles.on_secondary_container));
        let (fill, fg) = nav_colors(&t, false, false);
        assert!(fill.is_none());
        assert_eq!(fg, hsla(t.palette.roles.on_surface_variant));
    }

    #[test]
    fn other_designs_keep_their_list_colours() {
        let t = square();
        let (fill, fg) = nav_colors(&t, true, false);
        assert_eq!(fill, Some(hsla(t.palette.selection)));
        assert_eq!(fg, hsla(t.palette.text));
    }
}
