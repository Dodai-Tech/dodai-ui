//! Themed Freya components. All read the active theme via
//! [`use_dodai_theme`]; shape, glow, hard shadow and label casing come from
//! tokens only.
//!
//! Boxes with a chamfered theme need explicit pixel width/height (the SVG
//! outline is generated for that size); components estimate a width from the
//! label when none is given.

pub use crate::paint::shaped;
use crate::paint::{color, text};
use crate::style::*;
use crate::theme::{use_dodai, use_dodai_theme};
use dodai_core::{ButtonVariant, Color as Rgba, ControlState, ShapeKind, Theme, Tone};
use freya::prelude::*;

/// A no-argument callback prop.
type Action = EventHandler<()>;

fn action(f: impl FnMut() + 'static) -> Action {
    let mut f = f;
    EventHandler::new(move |()| f())
}

fn fire(handler: &Option<Action>) {
    if let Some(h) = handler {
        h.call(());
    }
}

// ── text ────────────────────────────────────────────────────────────────────

/// Themed text in one of the [`TextRole`]s (uppercased when the theme says so,
/// never for mono).
#[derive(PartialEq)]
pub struct DodaiText {
    role: TextRole,
    text: String,
    size: Option<f32>,
}

impl DodaiText {
    fn of(role: TextRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
            size: None,
        }
    }
    /// Display heading.
    pub fn heading(text: impl Into<String>) -> Self {
        Self::of(TextRole::Heading, text)
    }
    /// Body label.
    pub fn label(text: impl Into<String>) -> Self {
        Self::of(TextRole::Label, text)
    }
    /// Muted secondary text.
    pub fn muted(text: impl Into<String>) -> Self {
        Self::of(TextRole::Muted, text)
    }
    /// Monospace text.
    pub fn mono(text: impl Into<String>) -> Self {
        Self::of(TextRole::Mono, text)
    }
    /// Override the role's default size (px).
    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }
}

impl Component for DodaiText {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        text(&text_style(&t, self.role, self.size), None, &self.text)
    }
}

// ── interaction helper ──────────────────────────────────────────────────────

fn state_of(hover: bool, down: bool, disabled: bool) -> ControlState {
    if disabled {
        ControlState::Disabled
    } else if down {
        ControlState::Pressed
    } else if hover {
        ControlState::Hovered
    } else {
        ControlState::Idle
    }
}

// ── button ──────────────────────────────────────────────────────────────────

/// Themed button, every [`ButtonVariant`] plus disabled.
#[derive(PartialEq)]
pub struct DodaiButton {
    label: String,
    variant: ButtonVariant,
    disabled: bool,
    width: Option<f32>,
    on_press: Option<Action>,
    key: DiffKey,
}

impl KeyExt for DodaiButton {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl DodaiButton {
    /// Primary button with `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::Primary,
            disabled: false,
            width: None,
            on_press: None,
            key: DiffKey::None,
        }
    }
    /// Emphasis.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }
    /// Ignore input and show the disabled style.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Fixed width in px; estimated from the label when unset.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }
    /// Called on press (never while disabled).
    pub fn on_press(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_press = Some(action(f));
        self
    }
}

impl Component for DodaiButton {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let mut down = use_state(|| false);
        let state = state_of(*hover.read(), *down.read(), self.disabled);
        let (style, fg) = button_style(&t, self.variant, state);
        let ts = text_style(&t, TextRole::Label, None);
        let w = self.width.unwrap_or_else(|| {
            estimate_text_width(&ts.apply(&self.label), ts.size, ts.letter_spacing)
                + 2.0 * t.spacing.lg
        });
        let (on_press, disabled) = (self.on_press.clone(), self.disabled);
        rect()
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| {
                hover.set(false);
                down.set(false);
            })
            .on_pointer_down(move |_| down.set(true))
            .on_mouse_up(move |_| down.set(false))
            .on_press(move |_| {
                if !disabled {
                    fire(&on_press);
                }
            })
            .child(shaped(
                &t,
                &style,
                w,
                t.shape.control_height,
                0.0,
                false,
                text(&ts, Some(fg), &self.label),
            ))
    }
}

// ── panel ───────────────────────────────────────────────────────────────────

/// Bordered container with optional title.
#[derive(PartialEq)]
pub struct DodaiPanel {
    title: Option<String>,
    width: f32,
    height: f32,
    focused: bool,
    children: Vec<Element>,
}

impl DodaiPanel {
    /// Empty 320×160 panel.
    pub fn new() -> Self {
        Self {
            title: None,
            width: 320.0,
            height: 160.0,
            focused: false,
            children: Vec::new(),
        }
    }
    /// Title shown at the top.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
    /// Size in px.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
    /// Glows when focused (if the theme has glow).
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }
}

impl Default for DodaiPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl ChildrenExt for DodaiPanel {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for DodaiPanel {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let style = panel_style(&t, self.focused);
        let title = self.title.as_ref().map(|s| {
            let mut ts = text_style(&t, TextRole::Label, Some(t.typography.size_lg));
            ts.color = t.panel(self.focused).title;
            ts.weight = 700;
            text(&ts, None, s)
        });
        let inner = rect()
            .direction(Direction::Vertical)
            .spacing(t.spacing.sm)
            .margin(t.spacing.md)
            .maybe_child(title)
            .children(self.children.clone());
        shaped(&t, &style, self.width, self.height, 0.0, true, inner)
    }
}

// ── tag ─────────────────────────────────────────────────────────────────────

/// Status tag.
#[derive(PartialEq)]
pub struct DodaiTag {
    label: String,
    tone: Tone,
    solid: bool,
}

impl DodaiTag {
    /// Solid info tag.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            tone: Tone::Info,
            solid: true,
        }
    }
    /// Status colour.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }
    /// Filled (default) or outline.
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
}

impl Component for DodaiTag {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let (style, fg) = tag_style(&t, self.tone, self.solid);
        let ts = text_style(&t, TextRole::Label, Some(t.typography.size_xs));
        let w = estimate_text_width(&ts.apply(&self.label), ts.size, ts.letter_spacing)
            + 2.0 * t.spacing.sm;
        let h = t.typography.size_xs + 2.0 * t.spacing.xs + 4.0;
        shaped(
            &t,
            &style,
            w,
            h,
            0.0,
            false,
            text(&ts, Some(fg), &self.label),
        )
    }
}

// ── tabs ────────────────────────────────────────────────────────────────────

#[derive(PartialEq)]
struct DodaiTab {
    label: String,
    active: bool,
    on_select: Option<Action>,
    key: DiffKey,
}

impl KeyExt for DodaiTab {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for DodaiTab {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let (style, fg, ind) = tab_style(&t, self.active, *hover.read());
        let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
        let w = estimate_text_width(&ts.apply(&self.label), ts.size, ts.letter_spacing)
            + 2.0 * t.spacing.md;
        let line = ind.map(|i| {
            rect()
                .position(Position::new_absolute().bottom(0.0))
                .width(Size::px(w))
                .height(Size::px(2.0))
                .background(color(i))
        });
        let body = rect()
            .child(text(&ts, Some(fg), &self.label))
            .maybe_child(line);
        let on_select = self.on_select.clone();
        rect()
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| hover.set(false))
            .on_press(move |_| fire(&on_select))
            .child(shaped(
                &t,
                &style,
                w,
                t.shape.control_height,
                0.0,
                false,
                body,
            ))
    }
}

/// Tab strip.
#[derive(PartialEq)]
pub struct DodaiTabs {
    labels: Vec<String>,
    selected: usize,
    on_select: Option<EventHandler<usize>>,
}

impl DodaiTabs {
    /// Strip with one tab per label.
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            selected: 0,
            on_select: None,
        }
    }
    /// Index of the active tab.
    pub fn selected(mut self, selected: usize) -> Self {
        self.selected = selected;
        self
    }
    /// Called with the index of the pressed tab.
    pub fn on_select(mut self, f: impl FnMut(usize) + 'static) -> Self {
        self.on_select = Some(EventHandler::new(f));
        self
    }
}

impl Component for DodaiTabs {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        rect()
            .direction(Direction::Horizontal)
            .spacing(t.spacing.xs)
            .children(self.labels.iter().enumerate().map(|(i, l)| {
                let handler = self.on_select.clone();
                DodaiTab {
                    label: l.clone(),
                    active: i == self.selected,
                    on_select: Some(action(move || {
                        if let Some(h) = &handler {
                            h.call(i);
                        }
                    })),
                    key: DiffKey::None,
                }
                .key(i)
            }))
    }
}

// ── toggle / checkbox / radio ───────────────────────────────────────────────

fn thumb(theme: &Theme, col: Rgba, size: f32, round: bool) -> Rect {
    let r = if round && theme.shape.kind() == ShapeKind::Rounded {
        theme.shape.radius.min(size / 2.0)
    } else {
        0.0
    };
    rect()
        .width(Size::px(size))
        .height(Size::px(size))
        .background(color(col))
        .corner_radius(r)
}

/// On/off switch.
#[derive(PartialEq)]
pub struct DodaiToggle {
    on: bool,
    disabled: bool,
    on_change: Option<EventHandler<bool>>,
}

impl DodaiToggle {
    /// Switch in the given state.
    pub fn new(on: bool) -> Self {
        Self {
            on,
            disabled: false,
            on_change: None,
        }
    }
    /// Ignore input and show the disabled style.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Called with the new state on press.
    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(EventHandler::new(f));
        self
    }
}

impl Component for DodaiToggle {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let state = state_of(*hover.read(), false, self.disabled);
        let (style, thumb_c) = toggle_style(&t, self.on, state);
        let (w, h, pad) = (44.0, 24.0, 4.0);
        let inner = rect()
            .width(Size::px(w - 2.0 * pad))
            .height(Size::px(h - 2.0 * pad))
            .direction(Direction::Horizontal)
            .main_align(if self.on {
                Alignment::End
            } else {
                Alignment::Start
            })
            .cross_align(Alignment::Center)
            .child(thumb(&t, thumb_c, h - 2.0 * pad, true));
        let (on, disabled, handler) = (self.on, self.disabled, self.on_change.clone());
        rect()
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| hover.set(false))
            .on_press(move |_| {
                if let (false, Some(h)) = (disabled, &handler) {
                    h.call(!on);
                }
            })
            .child(shaped(&t, &style, w, h, pad, false, inner))
    }
}

fn mark_box(t: &Theme, on: bool, state: ControlState, round: bool) -> Rect {
    let (mut style, mark) = toggle_style(t, on, state);
    if round && style.kind == ShapeKind::Rounded {
        style.radius = 9.0;
    }
    let inner = on.then(|| thumb(t, mark, 8.0, round));
    shaped(t, &style, 18.0, 18.0, 0.0, false, rect().maybe_child(inner))
}

/// Mark + optional label row shared by checkbox and radio.
fn mark_row(t: &Theme, mark: Rect, label: &Option<String>, disabled: bool) -> Rect {
    let ts = text_style(t, TextRole::Label, Some(t.typography.size_sm));
    let col = disabled.then_some(t.palette.text_disabled);
    rect()
        .direction(Direction::Horizontal)
        .cross_align(Alignment::Center)
        .spacing(t.spacing.sm)
        .child(mark)
        .maybe_child(label.as_ref().map(|l| text(&ts, col, l)))
}

/// Checkbox with optional label.
#[derive(PartialEq)]
pub struct DodaiCheckbox {
    checked: bool,
    label: Option<String>,
    disabled: bool,
    on_change: Option<EventHandler<bool>>,
}

impl DodaiCheckbox {
    /// Checkbox in the given state.
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            label: None,
            disabled: false,
            on_change: None,
        }
    }
    /// Text next to the box.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Ignore input and show the disabled style.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Called with the new state on press.
    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(EventHandler::new(f));
        self
    }
}

impl Component for DodaiCheckbox {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let state = state_of(*hover.read(), false, self.disabled);
        let (on, disabled, handler) = (self.checked, self.disabled, self.on_change.clone());
        mark_row(&t, mark_box(&t, on, state, false), &self.label, disabled)
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| hover.set(false))
            .on_press(move |_| {
                if let (false, Some(h)) = (disabled, &handler) {
                    h.call(!on);
                }
            })
    }
}

/// Radio button (round only when the theme's shape is `Rounded`).
#[derive(PartialEq)]
pub struct DodaiRadio {
    selected: bool,
    label: Option<String>,
    disabled: bool,
    on_select: Option<Action>,
}

impl DodaiRadio {
    /// Radio in the given state.
    pub fn new(selected: bool) -> Self {
        Self {
            selected,
            label: None,
            disabled: false,
            on_select: None,
        }
    }
    /// Text next to the mark.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
    /// Ignore input and show the disabled style.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Called on press.
    pub fn on_select(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_select = Some(action(f));
        self
    }
}

impl Component for DodaiRadio {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let state = state_of(*hover.read(), false, self.disabled);
        let (disabled, handler) = (self.disabled, self.on_select.clone());
        mark_row(
            &t,
            mark_box(&t, self.selected, state, true),
            &self.label,
            disabled,
        )
        .on_pointer_enter(move |_| hover.set(true))
        .on_pointer_leave(move |_| hover.set(false))
        .on_press(move |_| {
            if !disabled {
                fire(&handler);
            }
        })
    }
}

// ── progress / slider ───────────────────────────────────────────────────────

/// Progress bar.
#[derive(PartialEq)]
pub struct DodaiProgress {
    value: f32,
    tone: Option<Tone>,
    width: f32,
    height: f32,
}

impl DodaiProgress {
    /// Bar at `value` (0..=100), 240×12.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            tone: None,
            width: 240.0,
            height: 12.0,
        }
    }
    /// Fill colour from a status tone (default: primary).
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = Some(tone);
        self
    }
    /// Size in px.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

impl Component for DodaiProgress {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let (style, fill) = progress_style(&t, self.tone);
        let bw = t.shape.border_width;
        let inner_w = (self.width - 2.0 * bw) * self.value.clamp(0.0, 100.0) / 100.0;
        let bar = rect()
            .width(Size::px(inner_w))
            .height(Size::px(self.height - 2.0 * bw))
            .background(color(fill))
            .corner_radius(style.corner_radius());
        shaped(&t, &style, self.width, self.height, bw, false, bar)
    }
}

/// Busy spinner: a ring of dots with a fading tail, driven by an endless animation.
#[derive(PartialEq)]
pub struct DodaiSpinner {
    size: f32,
}

impl DodaiSpinner {
    /// A spinner 24 px wide.
    pub fn new() -> Self {
        Self { size: 24.0 }
    }
    /// Width and height, px.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Default for DodaiSpinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for DodaiSpinner {
    fn render(&self) -> impl IntoElement {
        use freya::animation::{use_animation, AnimNum, Function, OnCreation, OnFinish};
        let t = use_dodai_theme();
        let phase = use_animation(|conf| {
            conf.on_creation(OnCreation::Run);
            conf.on_finish(OnFinish::restart());
            AnimNum::new(0.0, 1.0).time(1100).function(Function::Linear)
        });
        let phase: f32 = phase.get().value();
        const N: usize = 12;
        let dot = self.size * 0.16;
        let r = (self.size - dot) / 2.0;
        let mut ring = rect()
            .width(Size::px(self.size))
            .height(Size::px(self.size));
        for i in 0..N {
            let f = i as f32 / N as f32;
            let age = (phase - f).rem_euclid(1.0);
            let a = 1.0 - age * 0.85;
            let (x, y) = (
                self.size / 2.0 + r * (f * std::f32::consts::TAU).sin(),
                self.size / 2.0 - r * (f * std::f32::consts::TAU).cos(),
            );
            let mut c = color(t.progress(None).fill);
            c = c.with_a((255.0 * a) as u8);
            ring = ring.child(
                rect()
                    .position(
                        Position::new_absolute()
                            .left(x - dot / 2.0)
                            .top(y - dot / 2.0),
                    )
                    .width(Size::px(dot))
                    .height(Size::px(dot))
                    .background(c)
                    .corner_radius(dot / 2.0),
            );
        }
        ring
    }
}

/// Slider (wraps Freya's `Slider`, themed from tokens).
#[derive(PartialEq)]
pub struct DodaiSlider {
    value: f64,
    width: f32,
    on_moved: Option<EventHandler<f64>>,
}

impl DodaiSlider {
    /// Slider at `value` (0..=100), 220 px wide.
    pub fn new(value: f64) -> Self {
        Self {
            value,
            width: 220.0,
            on_moved: None,
        }
    }
    /// Width in px.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Called with the dragged value.
    pub fn on_moved(mut self, f: impl FnMut(f64) + 'static) -> Self {
        self.on_moved = Some(EventHandler::new(f));
        self
    }
}

impl Component for DodaiSlider {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let p = &t.palette;
        let theme = SliderThemePartial {
            background: Some(color(p.surface_raised).into()),
            thumb_background: Some(color(p.primary).into()),
            thumb_inner_background: Some(color(p.primary).into()),
            border_fill: Some(color(p.border).into()),
        };
        let handler = self.on_moved.clone();
        rect().width(Size::px(self.width)).child(
            Slider::new(move |v| {
                if let Some(h) = &handler {
                    h.call(v);
                }
            })
            .theme(theme)
            .size(Size::fill())
            .value(self.value),
        )
    }
}

// ── text input ──────────────────────────────────────────────────────────────

/// Text input (wraps Freya's `Input`; chamfered themes render it square since
/// Freya's `Input` cannot be clipped to a polygon).
#[derive(PartialEq)]
pub struct DodaiTextInput {
    value: State<String>,
    placeholder: Option<String>,
    width: f32,
}

impl DodaiTextInput {
    /// Input editing `value`, 220 px wide.
    pub fn new(value: State<String>) -> Self {
        Self {
            value,
            placeholder: None,
            width: 220.0,
        }
    }
    /// Hint shown while empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    /// Width in px.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Component for DodaiTextInput {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let tok = t.input(false);
        let focus = t.input(true);
        // On the glass the surface mix gives input fields.
        let field = input_style(&t, false);
        let fill = field.fill.unwrap_or(tok.background);
        let colors = InputColorsThemePartial {
            background: Some(color(fill).into()),
            focus_background: Some(color(fill).into()),
            border_fill: Some(color(tok.border).into()),
            focus_border_fill: Some(color(focus.border).into()),
            color: Some(color(tok.text).into()),
            placeholder_color: Some(color(tok.placeholder).into()),
        };
        let layout = InputLayoutThemePartial {
            corner_radius: Some(
                CornerRadius::new_all(input_style(&t, false).corner_radius()).into(),
            ),
            padding: None,
        };
        let input = Input::new(self.value)
            .width(Size::px(self.width))
            .theme_colors(colors)
            .theme_layout(layout);
        let input = match &self.placeholder {
            Some(p) => input.placeholder(p.clone()),
            None => input,
        };
        match field.blur {
            Some(b) => rect()
                .backdrop_blur(b)
                .corner_radius(field.corner_radius())
                .child(input)
                .into_element(),
            None => input.into_element(),
        }
    }
}

// ── list / divider / tooltip ────────────────────────────────────────────────

/// Selectable list row with marker.
#[derive(PartialEq)]
pub struct DodaiListItem {
    label: String,
    selected: bool,
    width: f32,
    on_press: Option<Action>,
    key: DiffKey,
}

impl DodaiListItem {
    /// Row with `label`, 260 px wide.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: false,
            width: 260.0,
            on_press: None,
            key: DiffKey::None,
        }
    }
    /// Highlight as selected.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// Width in px.
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    /// Called on press.
    pub fn on_press(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_press = Some(action(f));
        self
    }
}

impl KeyExt for DodaiListItem {
    fn write_key(&mut self) -> &mut DiffKey {
        &mut self.key
    }
}

impl Component for DodaiListItem {
    fn render_key(&self) -> DiffKey {
        self.key.clone().or(self.default_key())
    }

    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let mut hover = use_state(|| false);
        let (bg, fg, marker) = list_item_style(&t, self.selected, *hover.read());
        let ts = text_style(&t, TextRole::Label, Some(t.typography.size_sm));
        let handler = self.on_press.clone();
        rect()
            .width(Size::px(self.width))
            .height(Size::px(t.shape.control_height))
            .direction(Direction::Horizontal)
            .cross_align(Alignment::Center)
            .background(bg.map_or(Color::TRANSPARENT, color))
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| hover.set(false))
            .on_press(move |_| fire(&handler))
            .child(
                rect()
                    .width(Size::px(3.0))
                    .height(Size::fill())
                    .background(marker.map_or(Color::TRANSPARENT, color)),
            )
            .child(
                rect()
                    .padding(Gaps::new(0.0, t.spacing.md, 0.0, t.spacing.md))
                    .child(text(&ts, Some(fg), &self.label)),
            )
    }
}

/// Horizontal rule.
#[derive(PartialEq, Default)]
pub struct DodaiDivider;

impl Component for DodaiDivider {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        rect()
            .width(Size::fill())
            .height(Size::px(t.shape.border_width))
            .background(color(t.divider()))
    }
}

/// Tooltip bubble (static). Use [`DodaiTooltipHost`] for hover behaviour.
#[derive(PartialEq)]
pub struct DodaiTooltip {
    text: String,
}

impl DodaiTooltip {
    /// Bubble showing `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl Component for DodaiTooltip {
    fn render(&self) -> impl IntoElement {
        let t = use_dodai_theme();
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.overlay),
            Some(t.palette.border_strong),
            false,
            false,
        );
        let ts = text_style(&t, TextRole::Muted, Some(t.typography.size_xs));
        let w = estimate_text_width(&self.text, ts.size, 0.0) + 2.0 * t.spacing.sm;
        let h = ts.size + 2.0 * t.spacing.sm;
        shaped(
            &t,
            &style,
            w,
            h,
            0.0,
            false,
            text(&ts, Some(t.palette.text), &self.text),
        )
    }
}

/// Shows [`DodaiTooltip`] under its child while hovered.
#[derive(PartialEq)]
pub struct DodaiTooltipHost {
    tip: String,
    offset: f32,
    children: Vec<Element>,
}

impl DodaiTooltipHost {
    /// Host that shows `tip`.
    pub fn new(tip: impl Into<String>) -> Self {
        Self {
            tip: tip.into(),
            offset: 44.0,
            children: Vec::new(),
        }
    }
    /// Distance from the top of the host to the tooltip.
    pub fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }
}

impl ChildrenExt for DodaiTooltipHost {
    fn get_children(&mut self) -> &mut Vec<Element> {
        &mut self.children
    }
}

impl Component for DodaiTooltipHost {
    fn render(&self) -> impl IntoElement {
        let mut hover = use_state(|| false);
        let tip = (*hover.read()).then(|| {
            rect()
                .position(Position::new_absolute().top(self.offset).left(0.0))
                .layer(Layer::Overlay)
                .child(DodaiTooltip::new(self.tip.clone()))
        });
        rect()
            .on_pointer_enter(move |_| hover.set(true))
            .on_pointer_leave(move |_| hover.set(false))
            .children(self.children.clone())
            .maybe_child(tip)
    }
}

// ── theme switcher bar ──────────────────────────────────────────────────────

/// One button per registered theme; the active one is `Primary`, others `Outline`.
#[derive(PartialEq, Default)]
pub struct DodaiThemeSwitcher;

impl Component for DodaiThemeSwitcher {
    fn render(&self) -> impl IntoElement {
        let dodai = use_dodai();
        let t = dodai.theme();
        let cur = dodai.current_index();
        rect()
            .direction(Direction::Horizontal)
            .spacing(t.spacing.sm)
            .children(dodai.list().into_iter().enumerate().map(|(i, (id, name))| {
                DodaiButton::new(name)
                    .variant(if i == cur {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_press(move || dodai.select_index(i))
                    .key(id)
            }))
    }
}
