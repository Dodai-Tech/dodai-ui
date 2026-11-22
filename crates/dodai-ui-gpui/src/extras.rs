//! Beyond-M3 building blocks: [`Toolbar`], [`BottomAppBar`], [`EmptyState`],
//! [`Rating`], [`NumberStepper`], [`Timeline`], [`ColorPicker`],
//! [`FileDropZone`], [`TagField`].

use crate::data::Chip;
use crate::interact::{track, use_interact};
use crate::paint::{round_shape, shaped, shaped_id, ShapeStyle};
use crate::surfaces::IconButton;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_ui_core::{Color, Theme, Tone};
use gpui::{
    div, px, AnyElement, App, ClickEvent, ElementId, InteractiveElement, IntoElement,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window,
};
use std::rc::Rc;

fn bar_style(t: &Theme) -> ShapeStyle {
    if t.look().tonal {
        return ShapeStyle::from_tokens(
            t,
            Some(t.palette.roles.surface_container),
            None,
            false,
            false,
        )
        .with_surface(t, dodai_ui_core::SurfaceClass::Bar);
    }
    let p = t.panel(false);
    ShapeStyle::from_tokens(t, Some(p.background), Some(p.border), false, false)
        .with_surface(t, dodai_ui_core::SurfaceClass::Bar)
}

// ──────────────────────── Toolbar / BottomAppBar ────────────────────────

/// Compact bar of actions (children), optionally with a trailing floating action.
#[derive(IntoElement)]
pub struct Toolbar {
    items: Vec<AnyElement>,
    trailing: Option<AnyElement>,
    corners: Option<dodai_ui_core::Corners>,
    surface: dodai_ui_core::Surface,
    floating: bool,
    vibrant: bool,
}

crate::corners_setter!(Toolbar);
crate::surface_setter!(Toolbar);

impl Toolbar {
    /// Create a new [`Toolbar`].
    pub fn new() -> Self {
        Self {
            corners: None,
            surface: dodai_ui_core::Surface::INHERIT,
            items: Vec::new(),
            trailing: None,
            floating: false,
            vibrant: false,
        }
    }
    /// Material 3 Expressive floating toolbar: a lifted pill that hugs its content, instead of
    /// the docked full-width bar.
    pub fn floating(mut self, on: bool) -> Self {
        self.floating = on;
        self
    }
    /// Vibrant colour scheme (primary container) instead of the standard surface one.
    pub fn vibrant(mut self, on: bool) -> Self {
        self.vibrant = on;
        self
    }
    /// Element pinned to the right end (e.g. a [`crate::Fab`]).
    pub fn trailing(mut self, e: impl IntoElement) -> Self {
        self.trailing = Some(e.into_any_element());
        self
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Toolbar {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.items.extend(e);
    }
}

impl Toolbar {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let mut style =
            bar_style(&t).with_own_surface(&t, dodai_ui_core::SurfaceClass::Bar, &self.surface);
        let mut shape = t.role_shape(dodai_ui_core::ShapeRole::Toolbar);
        let tonal = t.look().tonal;
        let mut fg = hsla(t.palette.text);
        if tonal && self.vibrant {
            style.fill = Some(hsla(t.palette.roles.primary_container));
            fg = hsla(t.palette.roles.on_primary_container);
        }
        if tonal && self.floating {
            style = style.with_lift(&t, 3);
            shape.radius = 999.0;
        }
        let mut bar = shaped(&shape, style)
            .flex()
            .items_center()
            .gap(px(t.spacing.xs))
            .px(px(t.spacing.sm))
            .py(px(t.spacing.xs))
            .text_color(fg)
            .children(self.items);
        if tonal {
            // Docked: 64 px tall; floating: 64 px pill.
            bar = bar.h(px(t.scale(dodai_ui_core::Scale::Wide)));
            if self.floating {
                bar = bar.flex_none().w_auto();
            }
        }
        if let Some(e) = self.trailing {
            bar = bar.child(div().flex_1()).child(e);
        }
        bar
    }
}

crate::entry_render!(Toolbar);

/// Bottom app bar: actions (children) at the start, a floating action at the end.
#[derive(IntoElement)]
pub struct BottomAppBar {
    items: Vec<AnyElement>,
    trailing: Option<AnyElement>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(BottomAppBar);

impl BottomAppBar {
    /// Create a new [`BottomAppBar`].
    pub fn new() -> Self {
        Self {
            corners: None,
            items: Vec::new(),
            trailing: None,
        }
    }
    /// Element pinned to the end (e.g. a [`crate::Fab`]).
    pub fn trailing(mut self, e: impl IntoElement) -> Self {
        self.trailing = Some(e.into_any_element());
        self
    }
}

impl Default for BottomAppBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for BottomAppBar {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.items.extend(e);
    }
}

impl BottomAppBar {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let tok = t.app_bar(dodai_ui_core::AppBarKind::Bottom, false);
        let style = ShapeStyle::from_tokens(&t, tok.background, tok.border, tok.glow, false);
        let mut bar = shaped(&t.role_shape(dodai_ui_core::ShapeRole::AppBar), style)
            .h(px(tok.height))
            .flex()
            .items_center()
            .gap(px(t.spacing.xs))
            .px(px(t.spacing.md))
            .text_color(hsla(tok.leading))
            .children(self.items);
        if let Some(e) = self.trailing {
            bar = bar.child(div().flex_1()).child(e);
        }
        bar
    }
}

crate::entry_render!(BottomAppBar);

// ───────────────────────────── EmptyState ─────────────────────────────

/// Placeholder for empty / error screens.
#[derive(IntoElement)]
pub struct EmptyState {
    glyph: SharedString,
    title: SharedString,
    message: Option<SharedString>,
    tone: Option<Tone>,
    actions: Vec<AnyElement>,
}

impl EmptyState {
    /// Create a new [`EmptyState`].
    pub fn new(glyph: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            glyph: glyph.into(),
            title: title.into(),
            message: None,
            tone: None,
            actions: Vec::new(),
        }
    }
    /// Message text.
    pub fn message(mut self, m: impl Into<SharedString>) -> Self {
        self.message = Some(m.into());
        self
    }
    /// Tint the glyph (e.g. `Tone::Danger` for errors).
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }
}

impl ParentElement for EmptyState {
    fn extend(&mut self, e: impl IntoIterator<Item = AnyElement>) {
        self.actions.extend(e);
    }
}

impl EmptyState {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let tint = self
            .tone
            .map(|k| t.palette.tone(k))
            .unwrap_or(t.palette.text_muted);
        let mut el = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .p(px(t.spacing.xl))
            .font_family(first_family(t.typography.body))
            .text_color(hsla(t.palette.text))
            .child(
                div()
                    .text_size(px(t.typography.size_xl * 2.0))
                    .text_color(hsla(tint))
                    .child(self.glyph),
            )
            .child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_lg))
                    .child(format_label(&t.typography, &self.title)),
            );
        if let Some(m) = self.message {
            el = el.child(div().text_color(hsla(t.palette.text_muted)).child(m));
        }
        if !self.actions.is_empty() {
            el = el.child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .mt(px(t.spacing.sm))
                    .children(self.actions),
            );
        }
        el
    }
}

crate::entry_render!(EmptyState);

// ─────────────────────────────── Rating ───────────────────────────────

type ClickFn = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

type StarFn = Rc<dyn Fn(f32, &mut Window, &mut App)>;

/// Star rating (`value` of `max`). With [`Rating::half`] it also takes half stars: the left
/// half of a star is `n - 0.5`, the right half is `n`.
#[derive(IntoElement)]
pub struct Rating {
    id: ElementId,
    value: f32,
    max: u32,
    half: bool,
    on_change: Option<StarFn>,
}

impl Rating {
    /// Create a new [`Rating`]. Without [`Rating::half`] the value is rounded to whole stars.
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value,
            max: 5,
            half: false,
            on_change: None,
        }
    }
    /// Maximum value.
    pub fn max(mut self, m: u32) -> Self {
        self.max = m.max(1);
        self
    }
    /// Allow half stars.
    pub fn half(mut self, on: bool) -> Self {
        self.half = on;
        self
    }
    crate::callback!(
        /// Called when the value changes, with the new value.
        on_change,
        f32
    );
}

impl Rating {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.value);
        self.value = own.get();
        let (state, inner) = (own.setter(), self.on_change.take());
        self.on_change = Some(std::rc::Rc::new(move |v, w, cx| {
            crate::own::choose(&state, cx, v);
            if let Some(f) = &inner {
                f(v, w, cx);
            }
        }));
        let t = theme_of(cx);
        let value = if self.half {
            (self.value * 2.0).round() / 2.0
        } else {
            self.value.round()
        };
        let size = px(t.typography.size_xl);
        let (lit, dim) = (hsla(t.palette.primary), hsla(t.palette.border_strong));
        let stars = (1..=self.max).map(|i| {
            let n = i as f32;
            // How much of this star is filled: 0, 0.5 or 1.
            let fill = (value - (n - 1.0)).clamp(0.0, 1.0);
            let glyph = |c, g: &'static str| div().text_size(size).text_color(c).child(g);
            let mut s = div()
                .id((self.id.clone(), i.to_string()))
                .relative()
                .child(glyph(
                    if fill >= 1.0 { lit } else { dim },
                    if fill >= 1.0 { "★" } else { "☆" },
                ));
            if fill > 0.0 && fill < 1.0 {
                // The left half of a lit star over the outline.
                s = s.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w_1_2()
                        .overflow_hidden()
                        .child(glyph(lit, "★")),
                );
            }
            if let Some(f) = self.on_change.clone() {
                s = s.cursor_pointer();
                // Clicking the current value clears one step.
                let pick = move |to: f32| {
                    if (to - value).abs() < 0.01 {
                        to - if self.half { 0.5 } else { 1.0 }
                    } else {
                        to
                    }
                };
                if self.half {
                    let (f1, f2) = (f.clone(), f.clone());
                    let (lo, hi) = (pick(n - 0.5), pick(n));
                    s = s
                        .child(
                            div()
                                .id((self.id.clone(), format!("{i}l")))
                                .absolute()
                                .top_0()
                                .left_0()
                                .h_full()
                                .w_1_2()
                                .on_click(move |_, w, cx| f1(lo, w, cx)),
                        )
                        .child(
                            div()
                                .id((self.id.clone(), format!("{i}r")))
                                .absolute()
                                .top_0()
                                .right_0()
                                .h_full()
                                .w_1_2()
                                .on_click(move |_, w, cx| f2(hi, w, cx)),
                        );
                } else {
                    let to = pick(n);
                    s = s.on_click(move |_, w, cx| f(to, w, cx));
                }
            }
            s
        });
        div().flex().gap(px(t.spacing.half())).children(stars)
    }
}

crate::entry_render!(Rating);

// ───────────────────────────── NumberStepper ─────────────────────────────

/// `value + delta` clamped to `min..=max`.
pub fn step_value(value: i32, delta: i32, min: i32, max: i32) -> i32 {
    (value + delta).clamp(min.min(max), max.max(min))
}

type IntFn = Rc<dyn Fn(i32, &mut Window, &mut App)>;

/// − value + control.
#[derive(IntoElement)]
pub struct NumberStepper {
    id: ElementId,
    value: i32,
    min: i32,
    max: i32,
    step: i32,
    on_change: Option<IntFn>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(NumberStepper);

impl NumberStepper {
    /// Create a new [`NumberStepper`].
    pub fn new(id: impl Into<ElementId>, value: i32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            value,
            min: i32::MIN,
            max: i32::MAX,
            step: 1,
            on_change: None,
        }
    }
    /// Value range `min..max`.
    pub fn range(mut self, min: i32, max: i32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// Step between values.
    pub fn step(mut self, s: i32) -> Self {
        self.step = s.max(1);
        self
    }
    crate::callback!(
        /// Called when the value changes, with the new value.
        on_change,
        i32
    );
}

impl NumberStepper {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = t.panel(false);
        let (v, lo, hi) = (self.value, self.min, self.max);
        let mk = |suffix: &'static str, glyph: &'static str, d: i32| {
            let next = step_value(v, d, lo, hi);
            let mut b = IconButton::new((self.id.clone(), suffix), glyph)
                .variant(dodai_ui_core::ButtonVariant::Outline)
                .disabled(next == v);
            if let (true, Some(f)) = (next != v, self.on_change.clone()) {
                b = b.on_click(move |_, w, cx| f(next, w, cx));
            }
            b
        };
        let display = shaped(
            &t.shape,
            ShapeStyle::from_tokens(&t, None, Some(p.border), false, false),
        )
        .w(px(t.scale(dodai_ui_core::Scale::Wide)))
        .h(px(t.shape.control_height))
        .flex()
        .items_center()
        .justify_center()
        .font_family(first_family(t.typography.mono))
        .text_color(hsla(t.palette.text))
        .child(v.to_string());
        div()
            .flex()
            .items_center()
            .gap(px(t.spacing.xs))
            .child(mk("dec", "−", -self.step))
            .child(display)
            .child(mk("inc", "+", self.step))
    }
}

crate::entry_render!(NumberStepper);

// ───────────────────────────── Timeline ─────────────────────────────

/// One timeline entry.
pub struct TimelineEvent {
    /// Event title.
    pub title: SharedString,
    /// Optional detail line.
    pub detail: Option<SharedString>,
    /// Optional time label.
    pub time: Option<SharedString>,
    /// Marker colour.
    pub tone: Tone,
}

impl TimelineEvent {
    /// Create a new [`TimelineEvent`].
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            time: None,
            tone: Tone::Info,
        }
    }
    /// Detail text.
    pub fn detail(mut self, d: impl Into<SharedString>) -> Self {
        self.detail = Some(d.into());
        self
    }
    /// Time label.
    pub fn time(mut self, d: impl Into<SharedString>) -> Self {
        self.time = Some(d.into());
        self
    }
    /// Semantic tone (colour).
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = t;
        self
    }
}

/// Vertical sequence of events joined by a line.
#[derive(IntoElement)]
pub struct Timeline {
    events: Vec<TimelineEvent>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(Timeline);

impl Timeline {
    /// Create a new [`Timeline`].
    pub fn new(events: impl IntoIterator<Item = TimelineEvent>) -> Self {
        Self {
            corners: None,
            events: events.into_iter().collect(),
        }
    }
}

impl Timeline {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let n = self.events.len();
        let dot_shape = round_shape(&t.shape, 12.0);
        let rows = self.events.into_iter().enumerate().map(|(i, e)| {
            let c = t.palette.tone(e.tone);
            let dot = shaped(
                &dot_shape,
                ShapeStyle::from_tokens(&t, Some(c), None, true, false),
            )
            .w(px(12.0))
            .h(px(12.0))
            .flex_none();
            let rail = div()
                .flex()
                .flex_col()
                .items_center()
                .w(px(12.0))
                .child(dot)
                .child(div().flex_1().w(px(2.0)).bg(hsla(if i + 1 < n {
                    t.palette.border_strong
                } else {
                    t.palette.background
                })));
            let mut body = div().flex().flex_col().pb(px(t.spacing.md)).child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .items_baseline()
                    .child(
                        div()
                            .text_color(hsla(t.palette.text))
                            .child(format_label(&t.typography, &e.title)),
                    )
                    .children(e.time.map(|x| {
                        div()
                            .text_size(px(t.typography.size_sm))
                            .text_color(hsla(t.palette.text_muted))
                            .child(x)
                    })),
            );
            if let Some(d) = e.detail {
                body = body.child(
                    div()
                        .text_size(px(t.typography.size_sm))
                        .text_color(hsla(t.palette.text_muted))
                        .child(d),
                );
            }
            div().flex().gap(px(t.spacing.md)).child(rail).child(body)
        });
        div()
            .flex()
            .flex_col()
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .children(rows)
    }
}

crate::entry_render!(Timeline);

// ───────────────────────────── ColorPicker ─────────────────────────────

type ColorFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Swatch grid; shows the selected colour's hex code.
#[derive(IntoElement)]
pub struct ColorPicker {
    id: ElementId,
    colors: Vec<Color>,
    selected: Option<usize>,
    columns: usize,
    on_select: Option<ColorFn>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(ColorPicker);

impl ColorPicker {
    /// Create a new [`ColorPicker`].
    pub fn new(id: impl Into<ElementId>, colors: impl IntoIterator<Item = Color>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            colors: colors.into_iter().collect(),
            selected: None,
            columns: 8,
            on_select: None,
        }
    }
    /// The selected item (controlled).
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    /// Number of columns.
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the index.
        on_select,
        usize
    );
}

impl ColorPicker {
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.selected);
        self.selected = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        self.on_select = Some(std::rc::Rc::new(move |i, w, cx| {
            crate::own::choose(&state, cx, Some(i));
            if let Some(f) = &inner {
                f(i, w, cx);
            }
        }));
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let side = t.scale(dodai_ui_core::Scale::Compact);
        let mut grid = div().flex().flex_col().gap(px(t.spacing.xs));
        for (ri, chunk) in self.colors.chunks(self.columns).enumerate() {
            let mut row = div().flex().gap(px(t.spacing.xs));
            for (ci, c) in chunk.iter().enumerate() {
                let i = ri * self.columns + ci;
                let id: ElementId = (self.id.clone(), i.to_string()).into();
                let st = use_interact(&id, window, cx);
                let on = self.selected == Some(i);
                let hov = st.read(cx).hovered;
                let stroke = if on {
                    Some(t.palette.text)
                } else if hov {
                    Some(t.palette.border_strong)
                } else {
                    Some(t.palette.border)
                };
                let style = ShapeStyle::from_tokens(&t, Some(*c), stroke, on, false);
                let mut sw = track(shaped_id(id, &t.shape, style), &st, cx)
                    .w(px(side))
                    .h(px(side))
                    .cursor_pointer();
                if let Some(f) = self.on_select.clone() {
                    sw = sw.on_click(move |_, w, cx| f(i, w, cx));
                }
                row = row.child(sw);
            }
            grid = grid.child(row);
        }
        let hex = self
            .selected
            .and_then(|i| self.colors.get(i))
            .map(|c| c.to_hex())
            .unwrap_or_else(|| "—".into());
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .child(grid)
            .child(
                div()
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(hex),
            )
    }
}

crate::entry_render!(ColorPicker);

// ───────────────────────────── FileDropZone ─────────────────────────────

/// Drop target look (click to browse). Wire `on_click` to your file dialog.
#[derive(IntoElement)]
pub struct FileDropZone {
    id: ElementId,
    title: SharedString,
    hint: SharedString,
    on_click: Option<ClickFn>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(FileDropZone);

impl FileDropZone {
    /// Create a new [`FileDropZone`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            corners: None,
            id: id.into(),
            title: "Drop files here".into(),
            hint: "or click to browse".into(),
            on_click: None,
        }
    }
    /// Title text.
    pub fn title(mut self, t: impl Into<SharedString>) -> Self {
        self.title = t.into();
        self
    }
    /// Secondary hint text.
    pub fn hint(mut self, t: impl Into<SharedString>) -> Self {
        self.hint = t.into();
        self
    }
    crate::callback!(
        /// Called when it is clicked, with the click event.
        on_click,
        &ClickEvent
    );
}

impl FileDropZone {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let st = use_interact(&self.id, window, cx);
        let hov = st.read(cx).hovered;
        let stroke = if hov {
            t.palette.primary
        } else {
            t.palette.border_strong
        };
        let style = ShapeStyle::from_tokens(&t, None, Some(stroke), hov, false);
        let mut el = track(shaped_id(self.id, &t.shape, style), &st, cx)
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.xs))
            .p(px(t.spacing.xl))
            .cursor_pointer()
            .font_family(first_family(t.typography.body))
            .text_color(hsla(t.palette.text))
            .child(
                div()
                    .text_size(px(t.typography.size_xl * 1.6))
                    .text_color(hsla(if hov {
                        t.palette.primary
                    } else {
                        t.palette.text_muted
                    }))
                    .child("⇪"),
            )
            .child(format_label(&t.typography, &self.title))
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(self.hint),
            );
        if let Some(f) = self.on_click {
            el = el.on_click(move |e, w, cx| f(e, w, cx));
        }
        el
    }
}

crate::entry_render!(FileDropZone);

// ───────────────────────────── TagField ─────────────────────────────

type TagFn = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// Removable chips inside an input-style box (pair with a text input to add).
#[derive(IntoElement)]
pub struct TagField {
    id: ElementId,
    tags: Vec<SharedString>,
    on_remove: Option<TagFn>,
    trailing: Option<AnyElement>,
    corners: Option<dodai_ui_core::Corners>,
}

crate::corners_setter!(TagField);

impl TagField {
    /// Create a new [`TagField`].
    pub fn new<S: Into<SharedString>>(
        id: impl Into<ElementId>,
        tags: impl IntoIterator<Item = S>,
    ) -> Self {
        Self {
            corners: None,
            id: id.into(),
            tags: tags.into_iter().map(Into::into).collect(),
            on_remove: None,
            trailing: None,
        }
    }
    crate::callback!(
        /// Called when it is removed, with the index.
        on_remove,
        usize
    );
    /// Element after the chips (typically a `TextInput`).
    pub fn trailing(mut self, e: impl IntoElement) -> Self {
        self.trailing = Some(e.into_any_element());
        self
    }
}

impl TagField {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let i = t.input(false);
        let style = ShapeStyle::from_tokens(&t, Some(i.background), Some(i.border), false, false);
        let chips = self.tags.into_iter().enumerate().map(|(n, label)| {
            let mut c = Chip::new((self.id.clone(), n.to_string()), label);
            if let Some(f) = self.on_remove.clone() {
                c = c.on_remove(move |_, w, cx| f(n, w, cx));
            }
            c
        });
        let mut el = shaped(&t.shape, style)
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(t.spacing.xs))
            .p(px(t.spacing.sm))
            .min_h(px(t.shape.control_height))
            .children(chips);
        if let Some(e) = self.trailing {
            el = el.child(e);
        }
        el
    }
}

crate::entry_render!(TagField);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stepper_clamps() {
        assert_eq!(step_value(5, 1, 0, 5), 5);
        assert_eq!(step_value(0, -1, 0, 5), 0);
        assert_eq!(step_value(3, 2, 0, 10), 5);
        assert_eq!(step_value(3, -10, -2, 10), -2);
    }
}
