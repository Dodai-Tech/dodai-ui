//! Material-3-style structural components, themed purely from tokens:
//! [`Card`], [`IconButton`], [`Fab`], [`Navigation`] (bar / rail / drawer),
//! [`TopAppBar`] and [`SearchBar`].

use crate::button::button_style;
use crate::interact::{track, use_interact, ClickHandler};
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::tabs::tab_style;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, ControlState, Theme};
use gpui::{
    div, px, AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── Card ─────────────────────────────

/// Card emphasis.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// Raised surface with hard shadow / glow.
    #[default]
    Elevated,
    /// Flat raised surface, no border.
    Filled,
    /// Transparent with a border.
    Outlined,
}

/// Paint style of a card (pure).
pub fn card_style(theme: &Theme, v: CardVariant) -> ShapeStyle {
    let p = theme.panel(false);
    let raised = theme.palette.surface_raised;
    match v {
        CardVariant::Elevated => {
            ShapeStyle::from_tokens(theme, Some(p.background), Some(p.border), false, true)
        }
        CardVariant::Filled => ShapeStyle::from_tokens(theme, Some(raised), None, false, false),
        CardVariant::Outlined => ShapeStyle::from_tokens(theme, None, Some(p.border), false, false),
    }
}

/// Container with optional headline, supporting text and children.
#[derive(IntoElement)]
pub struct Card {
    variant: CardVariant,
    headline: Option<SharedString>,
    supporting: Option<SharedString>,
    width: Option<f32>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            headline: None,
            supporting: None,
            width: None,
            children: Vec::new(),
        }
    }
    pub fn headline(mut self, t: impl Into<SharedString>) -> Self {
        self.headline = Some(t.into());
        self
    }
    /// Fixed width in px (default: fill the parent).
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn supporting(mut self, t: impl Into<SharedString>) -> Self {
        self.supporting = Some(t.into());
        self
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = t.panel(false);
        let mut el = shaped(&t.shape, card_style(&t, self.variant))
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .p(px(t.spacing.lg))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(t.palette.text));
        if let Some(w) = self.width {
            el = el.w(px(w));
        }
        if let Some(h) = self.headline {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_lg))
                    .text_color(hsla(p.title))
                    .child(format_label(&t.typography, &h)),
            );
        }
        if let Some(s) = self.supporting {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(s),
            );
        }
        el.children(self.children)
    }
}

// ─────────────────────────── IconButton ───────────────────────────

/// Square icon button. Pass any glyph string (emoji, symbol, icon-font char).
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    glyph: SharedString,
    variant: ButtonVariant,
    toggled: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, glyph: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            glyph: glyph.into(),
            variant: ButtonVariant::Ghost,
            toggled: None,
            disabled: false,
            on_click: None,
        }
    }
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Make it a toggle button; `true` renders as the filled (primary) state.
    pub fn toggled(mut self, on: bool) -> Self {
        self.toggled = Some(on);
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = use_interact(&self.id, window, cx);
        let state = st.read(cx).control_state(self.disabled);
        let variant = match self.toggled {
            Some(true) => ButtonVariant::Primary,
            Some(false) => ButtonVariant::Outline,
            None => self.variant,
        };
        let (style, fg) = button_style(&t, variant, state);
        let side = t.shape.control_height;
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .w(px(side))
            .h(px(side))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(t.typography.size_lg))
            .text_color(fg)
            .child(self.glyph);
        if !self.disabled {
            el = el.cursor_pointer();
            if let Some(h) = self.on_click {
                el = el.on_click(move |e, w, cx| h(e, w, cx));
            }
        }
        el
    }
}

// ────────────────────────────── FAB ──────────────────────────────

/// Floating action button size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FabSize {
    Small,
    #[default]
    Regular,
    Large,
}

impl FabSize {
    /// Side length in px relative to the theme control height.
    pub fn side(self, theme: &Theme) -> f32 {
        let h = theme.shape.control_height;
        match self {
            FabSize::Small => h,
            FabSize::Regular => h * 1.2,
            FabSize::Large => h * 1.8,
        }
    }
}

/// Floating action button; `extended` adds a text label.
#[derive(IntoElement)]
pub struct Fab {
    id: ElementId,
    glyph: SharedString,
    label: Option<SharedString>,
    size: FabSize,
    variant: ButtonVariant,
    on_click: Option<ClickHandler>,
}

impl Fab {
    pub fn new(id: impl Into<ElementId>, glyph: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            glyph: glyph.into(),
            label: None,
            size: FabSize::Regular,
            variant: ButtonVariant::Primary,
            on_click: None,
        }
    }
    pub fn extended(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    pub fn size(mut self, s: FabSize) -> Self {
        self.size = s;
        self
    }
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for Fab {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = use_interact(&self.id, window, cx);
        let state = st.read(cx).control_state(false);
        let (mut style, fg) = button_style(&t, self.variant, state);
        // FABs always float: shadow + glow even at rest.
        let fx = &t.effects;
        if fx.glow_enabled() {
            style.glow = Some((hsla(fx.glow_color()), fx.glow_radius()));
        }
        let side = self.size.side(&t);
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .h(px(side))
            .flex()
            .items_center()
            .justify_center()
            .gap(px(t.spacing.sm))
            .text_color(fg)
            .text_size(px(t.typography.size_lg))
            .cursor_pointer();
        el = match &self.label {
            Some(_) => el.px(px(t.spacing.lg)),
            None => el.w(px(side)),
        };
        el = el.child(self.glyph);
        if let Some(l) = self.label {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_md))
                    .child(format_label(&t.typography, &l)),
            );
        }
        if let Some(h) = self.on_click {
            el = el.on_click(move |e, w, cx| h(e, w, cx));
        }
        el
    }
}

// ──────────────────────────── Navigation ────────────────────────────

/// One destination.
#[derive(Clone)]
pub struct NavItem {
    pub glyph: SharedString,
    pub label: SharedString,
    /// Optional count badge.
    pub badge: Option<u32>,
}

impl NavItem {
    pub fn new(glyph: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            glyph: glyph.into(),
            label: label.into(),
            badge: None,
        }
    }
    pub fn badge(mut self, n: u32) -> Self {
        self.badge = Some(n);
        self
    }
}

/// Where the navigation lives.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NavLayout {
    /// Horizontal strip (bottom of compact screens).
    #[default]
    Bar,
    /// Narrow vertical strip: glyph over label.
    Rail,
    /// Wide vertical list: glyph beside label.
    Drawer,
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Navigation bar / rail / drawer.
#[derive(IntoElement)]
pub struct Navigation {
    id: ElementId,
    layout: NavLayout,
    items: Vec<NavItem>,
    active: usize,
    on_select: Option<IndexHandler>,
}

impl Navigation {
    pub fn new(id: impl Into<ElementId>, layout: NavLayout) -> Self {
        Self {
            id: id.into(),
            layout,
            items: Vec::new(),
            active: 0,
            on_select: None,
        }
    }
    pub fn items(mut self, items: impl IntoIterator<Item = NavItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    pub fn active(mut self, i: usize) -> Self {
        self.active = i;
        self
    }
    pub fn on_select(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Navigation {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let layout = self.layout;
        let p = t.panel(false);
        let strip = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, false);
        let mut root = shaped(&t.shape, strip)
            .flex()
            .p(px(t.spacing.sm))
            .gap(px(t.spacing.xs));
        root = match layout {
            NavLayout::Bar => root.flex_row().justify_around(),
            NavLayout::Rail => root.flex_col().w(px(112.0)).items_center(),
            NavLayout::Drawer => root.flex_col().w(px(260.0)),
        };
        for (i, item) in self.items.iter().enumerate() {
            let id: ElementId = (self.id.clone(), i.to_string()).into();
            let st = use_interact(&id, window, cx);
            let hovered = st.read(cx).hovered;
            let active = i == self.active;
            let (style, fg, ind) = tab_style(&t, active, hovered);
            let mut el = track(shaped_id(id, &t.shape, style), &st)
                .relative()
                .flex()
                .items_center()
                .justify_center()
                .gap(px(t.spacing.xs))
                .px(px(t.spacing.md))
                .py(px(t.spacing.sm))
                .cursor_pointer()
                .whitespace_nowrap()
                .text_color(fg)
                .font_family(first_family(t.typography.body))
                .text_size(px(t.typography.size_sm));
            el = match layout {
                NavLayout::Bar => el.flex_col().flex_1(),
                NavLayout::Rail => el.flex_col().w_full(),
                NavLayout::Drawer => el.flex_row().justify_start().w_full(),
            };
            let mut glyph = div()
                .relative()
                .text_size(px(t.typography.size_lg))
                .child(item.glyph.clone());
            if let Some(n) = item.badge {
                glyph = glyph.child(
                    div()
                        .absolute()
                        .top(px(-4.0))
                        .right(px(-14.0))
                        .px(px(4.0))
                        .text_size(px(10.0))
                        .bg(hsla(t.palette.danger))
                        .text_color(hsla(t.palette.text))
                        .child(n.to_string()),
                );
            }
            el = el
                .child(glyph)
                .child(format_label(&t.typography, &item.label));
            if let Some(c) = ind {
                el = el.child(crate::paint::bottom_band_canvas(
                    &t.shape,
                    t.shape.border_width,
                    c,
                ));
            }
            if let Some(f) = self.on_select.clone() {
                el = el.on_click(move |_, w, cx| f(i, w, cx));
            }
            root = root.child(el);
        }
        root
    }
}

// ──────────────────────────── TopAppBar ────────────────────────────

/// App bar height class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppBarSize {
    #[default]
    Small,
    Large,
}

/// Top app bar: leading glyph, title, trailing actions (children).
#[derive(IntoElement)]
pub struct TopAppBar {
    title: SharedString,
    leading: Option<SharedString>,
    size: AppBarSize,
    actions: Vec<AnyElement>,
}

impl TopAppBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            leading: None,
            size: AppBarSize::Small,
            actions: Vec::new(),
        }
    }
    pub fn leading(mut self, glyph: impl Into<SharedString>) -> Self {
        self.leading = Some(glyph.into());
        self
    }
    pub fn size(mut self, s: AppBarSize) -> Self {
        self.size = s;
        self
    }
}

impl ParentElement for TopAppBar {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.actions.extend(elements);
    }
}

impl RenderOnce for TopAppBar {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = t.panel(false);
        let style = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, false);
        let h = t.shape.control_height
            * if self.size == AppBarSize::Large {
                2.4
            } else {
                1.3
            };
        let large = self.size == AppBarSize::Large;
        let mut bar = shaped(&t.shape, style)
            .h(px(h))
            .flex()
            .items_center()
            .gap(px(t.spacing.md))
            .px(px(t.spacing.md))
            .text_color(hsla(t.palette.text));
        if let Some(g) = self.leading {
            bar = bar.child(div().text_size(px(t.typography.size_lg)).child(g));
        }
        bar.child(
            div()
                .flex_1()
                .font_family(first_family(t.typography.display))
                .text_size(px(if large {
                    t.typography.size_xl
                } else {
                    t.typography.size_lg
                }))
                .text_color(hsla(p.title))
                .child(format_label(&t.typography, &self.title)),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.xs))
                .children(self.actions),
        )
    }
}

// ───────────────────────────── SearchBar ─────────────────────────────

/// Search field shell: leading glyph + placeholder/value text; click to focus
/// your own input. Use [`crate::TextInput`] inside for real editing.
#[derive(IntoElement)]
pub struct SearchBar {
    id: ElementId,
    glyph: SharedString,
    text: SharedString,
    placeholder: bool,
    on_click: Option<ClickHandler>,
}

impl SearchBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            glyph: "⌕".into(),
            text: "Search".into(),
            placeholder: true,
            on_click: None,
        }
    }
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.text = p.into();
        self.placeholder = true;
        self
    }
    pub fn value(mut self, v: impl Into<SharedString>) -> Self {
        self.text = v.into();
        self.placeholder = false;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = use_interact(&self.id, window, cx);
        let hovered = st.read(cx).control_state(false) != ControlState::Idle;
        let i = t.input(hovered);
        let style = ShapeStyle::from_tokens(&t, Some(i.background), Some(i.border), hovered, false);
        let fg = if self.placeholder {
            i.placeholder
        } else {
            i.text
        };
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .h(px(t.shape.control_height))
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .px(px(t.spacing.md))
            .text_size(px(t.typography.size_md))
            .text_color(hsla(fg))
            .cursor_text()
            .child(self.glyph)
            .child(self.text);
        if let Some(h) = self.on_click {
            el = el.on_click(move |e, w, cx| h(e, w, cx));
        }
        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};

    #[test]
    fn card_variants_differ_by_tokens() {
        for t in [light_rounded(), square()] {
            let e = card_style(&t, CardVariant::Elevated);
            let f = card_style(&t, CardVariant::Filled);
            let o = card_style(&t, CardVariant::Outlined);
            assert!(e.fill.is_some() && e.stroke.is_some());
            assert!(f.fill.is_some() && f.stroke.is_none());
            assert!(o.fill.is_none() && o.stroke.is_some());
        }
    }

    #[test]
    fn fab_sizes_scale_with_theme() {
        let t = square();
        assert!(FabSize::Small.side(&t) < FabSize::Regular.side(&t));
        assert!(FabSize::Regular.side(&t) < FabSize::Large.side(&t));
    }
}
