//! Material-3-style structural components, themed purely from tokens:
//! [`Card`], [`IconButton`], [`Fab`], `Navigation` (bar / rail / drawer),
//! [`TopAppBar`] and [`SearchBar`].

use crate::button::button_style;
use crate::interact::{track, use_interact, ClickHandler};
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, ControlState, Theme};
use gpui::{
    div, px, AnyElement, App, ClickEvent, ElementId, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── Card ─────────────────────────────

pub use dodai_core::CardVariant;

/// Paint style of a card (pure).
pub fn card_style(theme: &Theme, v: CardVariant) -> ShapeStyle {
    let mut st = ShapeStyle::from_surface(theme, theme.card(v))
        .0
        .with_surface(theme, dodai_core::SurfaceClass::Card);
    // No hard offset shadow in this theme: lift an elevated card with a soft one.
    if v == CardVariant::Elevated && st.shadow.is_none() && st.glow.is_none() {
        if theme.look().soft_elevation {
            st = st.with_lift(theme, 1);
        } else {
            st.glow = Some((crate::hsla(theme.effects.shadow), 10.0));
        }
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
    surface: dodai_core::Surface,
    accent: bool,
    children: Vec<AnyElement>,
}

crate::surface_setter!(Card);

impl Card {
    /// Create a new [`Card`].
    pub fn new(variant: CardVariant) -> Self {
        Self {
            variant,
            accent: false,
            headline: None,
            supporting: None,
            width: None,
            corners: None,
            surface: dodai_core::Surface::INHERIT,
            children: Vec::new(),
        }
    }
    /// Fills the card with the accent colour (primary) and switches text to the on-accent colour.
    pub fn accent(mut self, on: bool) -> Self {
        self.accent = on;
        self
    }
    /// Headline text.
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
    /// Supporting text.
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
        let (ink, head_ink, sub_ink) = if self.accent {
            let a = t.palette.on_primary;
            (a, a, a)
        } else {
            (t.palette.text, p.title, t.palette.text_muted)
        };
        let base = if self.accent {
            let a = t.palette.primary;
            ShapeStyle::from_tokens(&t, Some(a), Some(a), false, true)
        } else {
            card_style(&t, self.variant)
        };
        let mut el = shaped(
            &shape,
            base.with_own_surface(&t, dodai_core::SurfaceClass::Card, &self.surface),
        )
        .flex()
        .flex_col()
        .gap(px(t.spacing.sm))
        .p(px(t.spacing.lg))
        .font_family(first_family(t.typography.body))
        .text_size(px(t.typography.size_md))
        .text_color(hsla(ink));
        if let Some(w) = self.width {
            el = el.w(px(w));
        }
        if let Some(h) = self.headline {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_lg))
                    .text_color(hsla(head_ink))
                    .child(format_label(&t.typography, &h)),
            );
        }
        if let Some(s) = self.supporting {
            el = el.child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(sub_ink))
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

/// Width of an [`IconButton`] relative to its height (Material 3: narrow, default, wide).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum IconWidth {
    /// Narrow icon button.
    Narrow,
    /// Default width.
    #[default]
    Default,
    /// Wide icon button.
    Wide,
}

impl IconWidth {
    /// Width as a multiple of the height.
    pub fn factor(self) -> f32 {
        match self {
            Self::Narrow => 0.8,
            Self::Default => 1.0,
            Self::Wide => 1.3,
        }
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
    surface: dodai_core::Surface,
    size: Option<dodai_core::ControlSize>,
    square: bool,
    width: IconWidth,
    crop: Option<crate::Crop>,
    cut_gap: Option<f32>,
}

crate::corners_setter!(IconButton);
crate::surface_setter!(IconButton);
crate::crop_setters!(IconButton);

impl IconButton {
    /// Create a new [`IconButton`].
    pub fn new(id: impl Into<ElementId>, glyph: impl Into<SharedString>) -> Self {
        let glyph: SharedString = glyph.into();
        Self {
            corners: None,
            surface: dodai_core::Surface::INHERIT,
            id: id.into(),
            icon: glyph_icon(&glyph),
            glyph,
            variant: ButtonVariant::Ghost,
            toggled: None,
            disabled: false,
            on_click: None,
            size: None,
            square: false,
            width: IconWidth::Default,
            crop: None,
            cut_gap: None,
        }
    }
    /// Narrow, default or wide (0.8 / 1 / 1.3 × the height).
    pub fn icon_width(mut self, w: IconWidth) -> Self {
        self.width = w;
        self
    }
    /// One of the five sizes (XS 32 … XL 136 under Material).
    pub fn size(mut self, s: dodai_core::ControlSize) -> Self {
        self.size = Some(s);
        self
    }
    /// Square shape instead of the circle, on rounded designs.
    pub fn square(mut self, on: bool) -> Self {
        self.square = on;
        self
    }
    /// Draw a vector icon instead of the text glyph (crisp at any size/theme).
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Visual variant.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Make it a toggle button; `true` renders as the filled (primary) state.
    pub fn selected(mut self, on: bool) -> Self {
        self.toggled = Some(on);
        self
    }
    /// Disable the control.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Called when clicked.
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
        let selected = self.toggled == Some(true);
        let (mut style, fg) = if t.look().tonal && self.toggled.is_some() {
            // M3 toggle icon button: selected = filled, unselected = outlined.
            let b = t.icon_button(variant, state, false);
            (
                ShapeStyle::from_tokens(&t, b.background, b.border, false, false),
                hsla(b.foreground),
            )
        } else if t.look().tonal {
            let b = t.icon_button(variant, state, selected);
            (
                ShapeStyle::from_tokens(&t, b.background, b.border, false, false),
                hsla(b.foreground),
            )
        } else {
            button_style(&t, variant, state)
        };
        let m = self.size.map(|s| t.control_metrics(s));
        let side = m.map_or(t.shape.control_height, |m| m.height);
        let mut shape = t.role_shape(dodai_core::ShapeRole::IconButton);
        if let (true, Some(m), true) = (self.square, m, t.look().tonal) {
            shape.radius = m.square_radius;
        }
        style = style.with_own_surface(&t, dodai_core::SurfaceClass::Button, &self.surface);
        let crop = crate::button::resolve_crop(&t, cx, self.crop, self.cut_gap);
        if let Some(c) = &crop {
            c.apply(&t, &mut shape, &mut style);
        }
        let style_stroke = style.stroke.unwrap_or(fg);
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
        let mut el = track(shaped_id(self.id, &shape, style), &st)
            .mt(px(lift))
            .mb(px(-lift))
            .w(px(
                side * self.width.factor() + crop.map_or(0.0, |c| c.inset(&t))
            ))
            .h(px(side))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(t.typography.size_lg))
            .text_color(fg);
        if let Some(c) = &crop {
            el = el.relative().children(c.tick(&t, style_stroke));
        }
        el = match self.icon {
            Some(i) => el.child(
                crate::icons::Icon::new(i)
                    .size(m.map_or(side * 0.5, |m| m.icon_size))
                    .color(fg),
            ),
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
    /// Small.
    Small,
    /// Regular.
    #[default]
    Regular,
    /// Large.
    Large,
}

impl FabSize {
    /// Side length in px relative to the theme control height.
    pub fn side(self, theme: &Theme) -> f32 {
        let h = theme.shape.control_height;
        if theme.look().tonal {
            // M3: 40 / 56 / 96.
            return match self {
                FabSize::Small => 40.0,
                FabSize::Regular => 56.0,
                FabSize::Large => 96.0,
            };
        }
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
    crop: Option<crate::Crop>,
    cut_gap: Option<f32>,
}

crate::corners_setter!(Fab);
crate::crop_setters!(Fab);

impl Fab {
    /// Create a new [`Fab`].
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
            crop: None,
            cut_gap: None,
        }
    }
    /// Draw a vector icon instead of the text glyph.
    pub fn icon(mut self, i: crate::icons::IconName) -> Self {
        self.icon = Some(i);
        self
    }
    /// Show a label next to the glyph (extended FAB).
    pub fn extended(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Size step.
    pub fn size(mut self, s: FabSize) -> Self {
        self.size = s;
        self
    }
    /// Visual variant.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Called when clicked.
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
        let b = t.fab(self.variant, state);
        let (mut style, fg) = (
            ShapeStyle::from_tokens(&t, b.background, b.border, false, false),
            hsla(b.foreground),
        );
        if t.look().soft_elevation {
            let level = match state {
                ControlState::Hovered => 4,
                _ => 3,
            };
            style = style.with_lift(&t, level);
        }
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
        let role = match (self.label.is_some(), self.size) {
            (false, FabSize::Small) => dodai_core::ShapeRole::FabSmall,
            (false, FabSize::Large) => dodai_core::ShapeRole::FabLarge,
            _ => dodai_core::ShapeRole::Fab,
        };
        let mut shape = t.role_shape(role);
        let crop = crate::button::resolve_crop(&t, cx, self.crop, self.cut_gap);
        if let Some(c) = &crop {
            c.apply(&t, &mut shape, &mut style);
        }
        let inset = crop.map_or(0.0, |c| c.inset(&t));
        let style_stroke = style.stroke.unwrap_or(fg);
        let mut el = track(shaped_id(self.id, &shape, style), &st)
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
            Some(_) => {
                let lead = crop.is_some_and(|c| c.tick_leads());
                let (pl, pr) = (t.spacing.md, t.spacing.lg);
                if lead {
                    el.pl(px(pl + inset)).pr(px(pr))
                } else {
                    el.pl(px(pl)).pr(px(pr + inset))
                }
            }
            None => el.w(px(side + inset)),
        };
        if let Some(c) = &crop {
            el = el.relative().children(c.tick(&t, style_stroke));
        }
        el = match self.icon {
            Some(i) => {
                let sz = if t.look().tonal {
                    if self.size == FabSize::Large && self.label.is_none() {
                        36.0
                    } else {
                        24.0
                    }
                } else {
                    side * 0.42
                };
                el.child(crate::icons::Icon::new(i).size(sz).color(fg))
            }
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

// ────────────────────────────── FAB menu ──────────────────────────────

/// A FAB that opens a stack of labelled actions above it (Material 3 FAB menu). The main
/// button turns into a close button while open.
#[derive(IntoElement)]
pub struct FabMenu {
    id: ElementId,
    icon: crate::icons::IconName,
    items: Vec<(crate::icons::IconName, SharedString)>,
    on_select: Option<IndexHandler>,
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

impl FabMenu {
    /// Create a new [`FabMenu`].
    pub fn new(id: impl Into<ElementId>, icon: crate::icons::IconName) -> Self {
        Self {
            id: id.into(),
            icon,
            items: Vec::new(),
            on_select: None,
        }
    }
    /// The actions, nearest to the button last.
    pub fn items<S: Into<SharedString>>(
        mut self,
        items: impl IntoIterator<Item = (crate::icons::IconName, S)>,
    ) -> Self {
        self.items = items.into_iter().map(|(i, s)| (i, s.into())).collect();
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select, usize
    );
}

impl RenderOnce for FabMenu {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let open = window.use_keyed_state((self.id.clone(), "open"), cx, |_, _| false);
        let is_open = *open.read(cx);
        let mut col = div()
            .flex()
            .flex_col()
            .items_end()
            .gap(px(t.spacing.sm + 2.0));
        if is_open {
            for (i, (icon, label)) in self.items.iter().enumerate() {
                let (open, f) = (open.clone(), self.on_select.clone());
                col = col.child(
                    Fab::new((self.id.clone(), format!("item{i}")), "+")
                        .icon(*icon)
                        .extended(label.clone())
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, w, cx| {
                            open.update(cx, |o, cx| {
                                *o = false;
                                cx.notify();
                            });
                            if let Some(f) = &f {
                                f(i, w, cx);
                            }
                        }),
                );
            }
        }
        let toggle = open.clone();
        col.child(
            Fab::new((self.id.clone(), "main"), "+")
                .icon(if is_open {
                    crate::icons::IconName::Close
                } else {
                    self.icon
                })
                .on_click(move |_, _, cx| {
                    toggle.update(cx, |o, cx| {
                        *o = !*o;
                        cx.notify();
                    })
                }),
        )
    }
}

// ──────────────────────────── TopAppBar ────────────────────────────

/// Top app bar kind (M3: centre-aligned, small, medium, large).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AppBarSize {
    /// Title centred between the icons.
    CenterAligned,
    /// Small.
    #[default]
    Small,
    /// Icons on top, title below (headline-small).
    Medium,
    /// Icons on top, a large title below (headline-medium).
    Large,
}

impl AppBarSize {
    fn kind(self) -> dodai_core::AppBarKind {
        match self {
            Self::CenterAligned => dodai_core::AppBarKind::CenterAligned,
            Self::Small => dodai_core::AppBarKind::Small,
            Self::Medium => dodai_core::AppBarKind::Medium,
            Self::Large => dodai_core::AppBarKind::Large,
        }
    }
}

/// Top app bar: leading glyph, title, trailing actions (children).
///
/// `scrolled(true)` switches to the tonal "content is scrolling beneath" container;
/// `collapse(0..1)` shrinks a medium or large bar into the single-row layout as the page scrolls.
#[derive(IntoElement)]
pub struct TopAppBar {
    title: SharedString,
    leading: Option<SharedString>,
    size: AppBarSize,
    actions: Vec<AnyElement>,
    corners: Option<dodai_core::Corners>,
    scrolled: bool,
    collapse: f32,
}

crate::corners_setter!(TopAppBar);

impl TopAppBar {
    /// Create a new [`TopAppBar`].
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            corners: None,
            title: title.into(),
            leading: None,
            size: AppBarSize::Small,
            actions: Vec::new(),
            scrolled: false,
            collapse: 0.0,
        }
    }
    /// Leading content.
    pub fn leading(mut self, glyph: impl Into<SharedString>) -> Self {
        self.leading = Some(glyph.into());
        self
    }
    /// Size step.
    pub fn size(mut self, s: AppBarSize) -> Self {
        self.size = s;
        self
    }
    /// Content is scrolled beneath the bar (tonal container in Material).
    pub fn scrolled(mut self, scrolled: bool) -> Self {
        self.scrolled = scrolled;
        self
    }
    /// Collapse progress of a medium / large bar, `0.0` expanded to `1.0` one row.
    pub fn collapse(mut self, progress: f32) -> Self {
        self.collapse = progress.clamp(0.0, 1.0);
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
        let tok = t.app_bar(self.size.kind(), self.scrolled);
        let style = ShapeStyle::from_tokens(&t, tok.background, tok.border, tok.glow, false);
        let tall = matches!(self.size, AppBarSize::Medium | AppBarSize::Large);
        let two_row = tall && self.collapse < 0.999;
        let height = if tall {
            tok.height + (tok.row_height - tok.height) * self.collapse
        } else {
            tok.row_height
        };
        let title_size = if tall {
            tok.title_size + (tok.collapsed_title_size - tok.title_size) * self.collapse
        } else {
            tok.title_size
        };
        let title = || {
            div()
                .font_family(first_family(t.typography.display))
                .text_size(px(title_size))
                .text_color(hsla(tok.title))
                .child(format_label(&t.typography, &self.title))
        };
        let actions = div()
            .flex()
            .items_center()
            .justify_end()
            .gap(px(t.spacing.xs))
            .text_color(hsla(tok.trailing))
            .children(self.actions);
        let leading = |g: SharedString| {
            div()
                .w(px(t.sizes().touch_target))
                .flex()
                .justify_center()
                .child(glyph_el(g, t.typography.size_lg * 1.2, hsla(tok.leading)))
        };
        let mut bar = shaped(&t.role_shape(dodai_core::ShapeRole::AppBar), style)
            .h(px(height))
            .flex()
            .flex_col();
        let mut row = div()
            .h(px(tok.row_height))
            .flex_none()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .px(px(t.spacing.sm + 4.0));
        if let Some(g) = self.leading {
            row = row.child(leading(g));
        }
        row = if two_row {
            row.child(div().flex_1()).child(actions)
        } else if self.size == AppBarSize::CenterAligned {
            row.child(div().flex_1().flex().justify_center().child(title()))
                .child(div().min_w(px(t.sizes().touch_target)).child(actions))
        } else {
            row.child(div().flex_1().child(title())).child(actions)
        };
        bar = bar.child(row);
        if two_row {
            bar = bar.child(
                div()
                    .flex_1()
                    .flex()
                    .items_end()
                    .px(px(t.spacing.md))
                    .pb(px(if self.size == AppBarSize::Large {
                        28.0
                    } else {
                        24.0
                    }))
                    .child(title()),
            );
        }
        bar
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
    /// Create a new [`SearchBar`].
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
    /// Placeholder text.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.text = p.into();
        self.placeholder = true;
        self
    }
    /// Current value.
    pub fn value(mut self, v: impl Into<SharedString>) -> Self {
        self.text = v.into();
        self.placeholder = false;
        self
    }
    /// Called when clicked.
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
        let pill = t.look().tonal && self.corners.is_none();
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
            assert!(e.fill.is_some());
            // M3 elevated cards are lifted by a shadow, not an outline.
            assert_eq!(e.stroke.is_some(), !t.look().tonal);
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
