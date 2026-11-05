//! [`NavigationDrawer`], [`NavigationRail`] and [`NavigationBar`]: app-level navigation following the Material 3
//! specs (drawer: 56 px pill rows, section headlines, dividers, `surface-container-low`; rail:
//! 80 px wide, 56×32 indicator pill, label below the icon, badge). Other designs keep their own
//! shape and colours through the same theme tokens.

use crate::interact::{track, use_interact};
use crate::paint::{round_shape, shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label, typeset};
use crate::{hsla, theme_of};
use dodai_core::{NavigationTokens, Shape, Theme};
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
    expanded: Option<bool>,
    on_click: Option<Click>,
}

impl NavigationItem {
    /// Create a new [`NavigationItem`].
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            selected: false,
            depth: 0,
            expanded: None,
            on_click: None,
        }
    }
    /// Nest the row `levels` deep (drawer only): indented, shorter, no icon.
    pub fn depth(mut self, levels: u8) -> Self {
        self.depth = levels;
        self
    }
    /// Make the row a branch of a tree (drawer only): a chevron at the end points down while it
    /// is `open` and right while it is closed. The app owns the state and flips it in `on_click`;
    /// show or hide the child rows (which are nested with [`depth`](Self::depth)) to match.
    pub fn expanded(mut self, open: bool) -> Self {
        self.expanded = Some(open);
        self
    }
    /// Leading icon.
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Count or short text at the end of a drawer row / on a rail icon.
    pub fn badge(mut self, b: impl Into<SharedString>) -> Self {
        self.badge = Some(b.into());
        self
    }
    /// The selected item (controlled).
    pub fn selected(mut self, s: bool) -> Self {
        self.selected = s;
        self
    }
    /// Called when clicked.
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
    /// Run `extra` after the item's own click handler.
    pub(crate) fn also_on_click(mut self, extra: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        let own = self.on_click.take();
        self.on_click = Some(Box::new(move |e, w, cx| {
            if let Some(f) = &own {
                f(e, w, cx);
            }
            extra(w, cx);
        }));
        self
    }
}

/// Colours of a destination: (indicator fill, foreground) (pure; from the design's tokens).
pub fn nav_colors(t: &Theme, selected: bool, hovered: bool) -> (Option<Hsla>, Hsla) {
    let (fill, fg, _) = nav_paint(t, selected, hovered);
    (fill, fg)
}

/// [`nav_colors`] plus whether the destination is drawn outlined with a hard shadow.
pub fn nav_paint(t: &Theme, selected: bool, hovered: bool) -> (Option<Hsla>, Hsla, bool) {
    let k = t.navigation_item(dodai_core::ItemState::of(selected, hovered));
    (k.background.map(hsla), hsla(k.foreground), k.outline)
}

fn indicator_shape(t: &Theme, k: &NavigationTokens, side: f32) -> Shape {
    if k.pill {
        round_shape(&t.shape, side)
    } else {
        t.shape.clone()
    }
}

fn icon_el(i: crate::icons::IconName, size: f32, fg: Hsla) -> impl IntoElement {
    crate::icons::Icon::new(i).size(size).color(fg)
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
    max_width: Option<f32>,
    width: Option<f32>,
}

impl NavigationDrawer {
    /// Create a new [`NavigationDrawer`].
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_width: None,
            width: None,
        }
    }
    /// Add an item.
    pub fn item(mut self, i: NavigationItem) -> Self {
        self.entries.push(Entry::Item(i));
        self
    }
    /// Section headline above the following items.
    pub fn headline(mut self, t: impl Into<SharedString>) -> Self {
        self.entries.push(Entry::Headline(t.into()));
        self
    }
    /// Add a divider.
    pub fn divider(mut self) -> Self {
        self.entries.push(Entry::Divider);
        self
    }
    /// Known pixel width of the drawer; long labels are cut with an ellipsis to fit.
    /// (gpui only truncates text whose width is definite at measure time.)
    pub fn width(mut self, px: f32) -> Self {
        self.width = Some(px);
        self
    }
    /// Width cap; defaults to the design's `drawer_max_width`. The drawer fills its parent up to it.
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = Some(px);
        self
    }
}

impl RenderOnce for NavigationDrawer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let k = t.navigation();
        let muted = hsla(t.palette.roles.on_surface_variant);
        let mut col = div()
            .flex()
            .flex_col()
            .w_full()
            .min_w_0()
            .max_w(px(self.max_width.unwrap_or(k.drawer_max_width)))
            .gap(px(k.item_gap))
            .p(px(k.container_padding));
        if let Some(c) = k.container {
            col = col.bg(hsla(c));
        }
        for e in self.entries {
            match e {
                Entry::Headline(h) => {
                    col = col.child(
                        typeset(div(), &t, k.roles.3)
                            .min_h(px(k.headline_height))
                            .px(px(k.item_padding))
                            .flex()
                            .items_center()
                            .text_color(muted)
                            .child(h),
                    );
                }
                Entry::Divider => {
                    col = col.child(
                        div()
                            .my(px(k.item_content_gap * 0.67))
                            .mx(px(k.item_padding))
                            .h(crate::HAIRLINE)
                            .bg(hsla(t.palette.roles.outline_variant)),
                    );
                }
                Entry::Item(it) => {
                    col = col.child(drawer_item(it, &t, self.width, window, cx));
                }
            }
        }
        col
    }
}

fn drawer_item(
    it: NavigationItem,
    t: &Theme,
    drawer_w: Option<f32>,
    window: &mut Window,
    cx: &mut App,
) -> impl IntoElement {
    let k = t.navigation();
    let st = use_interact(&it.id, window, cx);
    let hovered = st.read(cx).hovered;
    let (fill, fg, outline) = nav_paint(t, it.selected, hovered);
    let nested = it.depth > 0;
    let h = if nested {
        k.nested_height
    } else {
        k.item_height
    };
    let shape = indicator_shape(t, &k, h);
    let style = ShapeStyle::from_tokens(
        t,
        None,
        outline.then_some(t.palette.border_strong),
        false,
        outline,
    );
    let style = ShapeStyle { fill, ..style };
    let mut el = track(shaped_id(it.id, &shape, style), &st, cx)
        .min_h(px(h))
        .ml(px(it.depth as f32 * k.indent))
        // Nested rows never touch the row above, even when the design stacks items tight.
        .mt(px(if nested {
            (t.spacing.xs - k.item_gap).max(0.0)
        } else {
            0.0
        }))
        .px(px(k.item_padding))
        .flex()
        .items_center()
        .gap(px(k.item_content_gap))
        .cursor_pointer()
        .text_color(fg)
        .font_family(first_family(t.typography.body));
    if let Some(i) = it.icon.filter(|_| !nested) {
        el = el.child(icon_el(i, k.icon_size, fg));
    }
    let role = if nested { k.roles.1 } else { k.roles.0 };
    let label = format_label(&t.typography, &it.label);
    let label: SharedString = match drawer_w {
        Some(w) => {
            let icon = if it.icon.is_some() && !nested {
                k.icon_size + k.item_content_gap
            } else {
                0.0
            };
            let badge = it.badge.as_ref().map_or(0.0, |b| {
                b.len() as f32 * t.type_style(k.roles.0).size * 0.6 + k.item_content_gap
            }) + it
                .expanded
                .map_or(0.0, |_| k.icon_size * 0.75 + k.item_content_gap);
            let room = w
                - 2.0 * k.container_padding
                - 2.0 * k.item_padding
                - it.depth as f32 * k.indent
                - icon
                - badge;
            ellipsize(cx, t, role, label, room)
        }
        None => label,
    };
    el = el.child(
        typeset(div(), t, role)
            .flex_1()
            .min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
            .child(label),
    );
    if let Some(b) = it.badge {
        el = el.child(typeset(div(), t, k.roles.0).child(b));
    }
    if let Some(open) = it.expanded {
        let chevron = if open {
            crate::icons::Icon::ChevronDown
        } else {
            crate::icons::Icon::ChevronRight
        };
        el = el.child(chevron.size(k.icon_size * 0.75).color(fg));
    }
    if let Some(h) = it.on_click {
        el = el.on_click(move |e, w, cx| h(e, w, cx));
    }
    el
}

/// Narrow icon rail (width from the design's tokens): label under each icon, indicator pill behind the icon.
#[derive(IntoElement)]
pub struct NavigationRail {
    items: Vec<NavigationItem>,
    header: Option<gpui::AnyElement>,
}

impl NavigationRail {
    /// Create a new [`NavigationRail`].
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            header: None,
        }
    }
    /// Add an item.
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

/// A stacked destination (indicator over its label), shared by the rail and the bar.
fn destination(
    it: NavigationItem,
    t: &Theme,
    k: &NavigationTokens,
    ind_w: f32,
    window: &mut Window,
    cx: &mut App,
) -> gpui::Stateful<gpui::Div> {
    let st = use_interact(&it.id, window, cx);
    let hovered = st.read(cx).hovered;
    let (fill, fg, outline) = nav_paint(t, it.selected, hovered);
    // The label sits outside the indicator, so it reads on the surface.
    let label_fg = if it.selected {
        hsla(t.palette.text)
    } else {
        fg
    };
    let pill_shape = indicator_shape(t, k, k.indicator.1);
    let pill = shaped(
        &pill_shape,
        ShapeStyle {
            fill,
            ..ShapeStyle::from_tokens(
                t,
                None,
                outline.then_some(t.palette.border_strong),
                false,
                outline,
            )
        },
    )
    .w(px(ind_w))
    .h(px(k.indicator.1))
    .flex()
    .items_center()
    .justify_center()
    .relative()
    .children(it.icon.map(|i| icon_el(i, k.icon_size, fg)))
    .children(it.badge.map(|b| {
        div()
            .absolute()
            .top(px(-2.0))
            .right(px(6.0))
            .child(crate::Badge::new(b, dodai_core::Tone::Danger))
    }));
    let mut el = track(div().id(it.id.clone()), &st, cx)
        .flex()
        .flex_col()
        .items_center()
        .gap(px(k.rail_label_gap))
        .cursor_pointer()
        .child(pill)
        .child(
            typeset(div(), t, k.roles.2)
                .text_color(label_fg)
                .child(format_label(&t.typography, &it.label)),
        );
    if let Some(h) = it.on_click {
        el = el.on_click(move |e, w, cx| h(e, w, cx));
    }
    el
}

/// Navigation bar: the destinations side by side along the bottom of a compact screen.
#[derive(IntoElement)]
pub struct NavigationBar {
    items: Vec<NavigationItem>,
}

impl Default for NavigationBar {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigationBar {
    /// Create a new [`NavigationBar`].
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    /// Add an item.
    pub fn item(mut self, i: NavigationItem) -> Self {
        self.items.push(i);
        self
    }
}

impl RenderOnce for NavigationBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let k = t.navigation();
        let mut row = div()
            .flex()
            .w_full()
            .h(px(k.bar_height))
            .px(px(k.container_padding))
            .pt(px(k.container_padding))
            .pb(px(k.container_padding + k.rail_label_gap));
        if let Some(c) = k.container {
            row = row.bg(hsla(c));
        }
        for it in self.items {
            row = row.child(destination(it, &t, &k, k.bar_indicator_width, window, cx).flex_1());
        }
        row
    }
}

impl RenderOnce for NavigationRail {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let k = t.navigation();
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .w(px(k.rail_width))
            .flex_none()
            .gap(px(k.container_padding))
            .py(px(k.container_padding));
        if let Some(c) = k.container {
            col = col.bg(hsla(c));
        }
        if let Some(h) = self.header {
            col = col.child(h);
        }
        for it in self.items {
            let el = destination(it, &t, &k, k.indicator.0, window, cx).w_full();
            col = col.child(el);
        }
        col
    }
}

/// Cut `text` with an ellipsis so it fits `max_w` pixels in the given type role.
fn ellipsize(
    cx: &App,
    t: &Theme,
    role: dodai_core::TypeRole,
    text: SharedString,
    max_w: f32,
) -> SharedString {
    let s = t.type_style(role);
    let fam = if role.is_heading() {
        t.typography.display
    } else {
        t.typography.body
    };
    let mut f = gpui::font(first_family(fam));
    f.weight = gpui::FontWeight(s.weight as f32);
    let mut runs = vec![gpui::TextRun {
        len: text.len(),
        font: f.clone(),
        color: gpui::Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }];
    let mut wrapper = cx.text_system().line_wrapper(f, px(s.size));
    wrapper.truncate_line(text, px(max_w.max(0.0)), "…", &mut runs)
}

impl Default for NavigationDrawer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NavigationRail {
    fn default() -> Self {
        Self::new()
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
