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
    div, px, AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── Card ─────────────────────────────

pub use dodai_core::CardVariant;

/// Paint style of a card (pure).
pub fn card_style(theme: &Theme, v: CardVariant) -> ShapeStyle {
    let mut st = ShapeStyle::from_surface(theme, theme.card(v)).0;
    // No hard offset shadow in this theme: lift an elevated card with a soft one.
    if v == CardVariant::Elevated && st.shadow.is_none() && st.glow.is_none() {
        st.glow = Some((crate::hsla(theme.effects.shadow), 10.0));
    }
    st
}

/// Container with optional headline, supporting text and children.
#[derive(IntoElement)]
pub struct Card {
    variant: CardVariant,
    headline: Option<SharedString>,
    supporting: Option<SharedString>,
    width: Option<f32>,
    corners: Option<dodai_core::Corners>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            headline: None,
            supporting: None,
            width: None,
            corners: None,
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
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
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

impl Card {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let p = t.panel(false);
        let mut shape = t.shape.clone();
        if let Some(c) = self.corners {
            shape.corners = c;
        }
        let mut el = shaped(&shape, card_style(&t, self.variant))
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

crate::entry_render!(Card);

// ─────────────────────────── IconButton ───────────────────────────

/// Vector icon for a well-known text glyph, so call sites that pass `"✕"`,
/// `"⌕"`, `"‹"`... get crisp, theme-coloured drawn icons for free.
pub fn glyph_icon(g: &str) -> Option<crate::icons::IconName> {
    use crate::icons::IconName as I;
    Some(match g {
        "✕" | "×" => I::Close,
        "⌕" => I::Search,
        "‹" => I::ChevronLeft,
        "›" => I::ChevronRight,
        "▲" => I::ChevronUp,
        "▼" => I::ChevronDown,
        "⋮" => I::DotsVertical,
        "☰" | "≡" => I::Menu,
        "♥" => I::Heart,
        "✎" => I::Edit,
        "✓" => I::Check,
        "+" => I::Plus,
        "⎘" => I::Copy,
        "✉" => I::Mail,
        "🔔" => I::Bell,
        "★" => I::Star,
        _ => return None,
    })
}

/// Render `glyph` as a drawn icon when it has one, else as text.
fn glyph_el(glyph: SharedString, size: f32, color: gpui::Hsla) -> gpui::AnyElement {
    match glyph_icon(&glyph) {
        Some(i) => crate::icons::Icon::new(i)
            .size(size)
            .color(color)
            .into_any_element(),
        None => div().child(glyph).into_any_element(),
    }
}

/// Square icon button. Pass any glyph string (emoji, symbol, icon-font char).
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    glyph: SharedString,
    icon: Option<crate::icons::IconName>,
    variant: ButtonVariant,
    toggled: Option<bool>,
    disabled: bool,
    on_click: Option<ClickHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(IconButton);

impl IconButton {
    pub fn new(id: impl Into<ElementId>, glyph: impl Into<SharedString>) -> Self {
        let glyph: SharedString = glyph.into();
        Self {
            corners: None,
            id: id.into(),
            icon: glyph_icon(&glyph),
            glyph,
            variant: ButtonVariant::Ghost,
            toggled: None,
            disabled: false,
            on_click: None,
        }
    }
    /// Draw a vector icon instead of the text glyph (crisp at any size/theme).
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
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

impl IconButton {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let state = st.read(cx).control_state(self.disabled);
        let variant = match self.toggled {
            Some(true) => ButtonVariant::Primary,
            Some(false) => ButtonVariant::Outline,
            None => self.variant,
        };
        let (style, fg) = button_style(&t, variant, state);
        let side = t.shape.control_height;
        let lift = if self.disabled {
            0.0
        } else {
            crate::motion::hover_lift(
                window,
                cx,
                (self.id.clone(), "lift").into(),
                state != ControlState::Idle,
            )
        };
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .mt(px(lift))
            .mb(px(-lift))
            .w(px(side))
            .h(px(side))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(t.typography.size_lg))
            .text_color(fg);
        el = match self.icon {
            Some(i) => el.child(crate::icons::Icon::new(i).size(side * 0.5).color(fg)),
            None => el.child(self.glyph),
        };
        if !self.disabled {
            el = el.cursor_pointer();
            if let Some(h) = self.on_click {
                el = el.on_click(move |e, w, cx| h(e, w, cx));
            }
        }
        el
    }
}

crate::entry_render!(IconButton);

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
    icon: Option<crate::icons::IconName>,
    label: Option<SharedString>,
    size: FabSize,
    variant: ButtonVariant,
    on_click: Option<ClickHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Fab);

impl Fab {
    pub fn new(id: impl Into<ElementId>, glyph: impl Into<SharedString>) -> Self {
        let glyph: SharedString = glyph.into();
        Self {
            corners: None,
            id: id.into(),
            icon: glyph_icon(&glyph),
            glyph,
            label: None,
            size: FabSize::Regular,
            variant: ButtonVariant::Primary,
            on_click: None,
        }
    }
    /// Draw a vector icon instead of the text glyph.
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
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

impl Fab {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let state = st.read(cx).control_state(false);
        let (mut style, fg) = button_style(&t, self.variant, state);
        // FABs always float: shadow + glow even at rest.
        let fx = &t.effects;
        if fx.glow_enabled() {
            style.glow = Some((hsla(fx.glow_color()), fx.glow_radius()));
        }
        let side = self.size.side(&t);
        let lift = crate::motion::hover_lift(
            window,
            cx,
            (self.id.clone(), "lift").into(),
            state != ControlState::Idle,
        );
        let mut el = track(shaped_id(self.id, &t.shape, style), &st)
            .mt(px(lift))
            .mb(px(-lift))
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
        el = match self.icon {
            Some(i) => el.child(crate::icons::Icon::new(i).size(side * 0.42).color(fg)),
            None => el.child(self.glyph),
        };
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

crate::entry_render!(Fab);

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
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(Navigation);

impl Navigation {
    pub fn new(id: impl Into<ElementId>, layout: NavLayout) -> Self {
        Self {
            corners: None,
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
    crate::callback!(on_select, usize);
}

impl Navigation {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
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
                let mut pill = div()
                    .absolute()
                    .top(px(-4.0))
                    .right(px(-14.0))
                    .px(px(4.0))
                    .text_size(px(10.0))
                    .bg(hsla(t.palette.danger))
                    .text_color(hsla(t.on(t.palette.danger)))
                    .child(n.to_string());
                if t.shape.kind() == dodai_core::ShapeKind::Rounded {
                    pill = pill.rounded_full().min_w(px(16.0)).flex().justify_center();
                }
                glyph = glyph.child(pill);
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

crate::entry_render!(Navigation);

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
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(TopAppBar);

impl TopAppBar {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
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

impl TopAppBar {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
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
            bar = bar.child(glyph_el(
                g,
                t.typography.size_lg * 1.2,
                hsla(t.palette.text),
            ));
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

crate::entry_render!(TopAppBar);

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
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(SearchBar);

impl SearchBar {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
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

impl SearchBar {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let mut t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let hovered = st.read(cx).control_state(false) != ControlState::Idle;
        let i = t.input(hovered);
        let pill = t.shape.kind() == dodai_core::ShapeKind::Rounded && self.corners.is_none();
        let style = if pill {
            // Rounded (Material): a full pill on a tonal container, no outline.
            t.shape.radius = t.shape.control_height / 2.0;
            ShapeStyle::from_tokens(
                &t,
                Some(t.palette.roles.surface_container_high),
                None,
                false,
                false,
            )
        } else {
            ShapeStyle::from_tokens(&t, Some(i.background), Some(i.border), hovered, false)
        };
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
            .child(glyph_el(self.glyph, t.typography.size_md * 1.1, hsla(fg)))
            .child(self.text);
        if let Some(h) = self.on_click {
            el = el.on_click(move |e, w, cx| h(e, w, cx));
        }
        el
    }
}

crate::entry_render!(SearchBar);

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
