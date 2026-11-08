//! Widget components and spawn helpers (`impl Bundle` functions).
//!
//! Spawn with `commands.spawn(button("Go", ButtonVariant::Primary))` or nest
//! with `children![...]`. Colours, sizes and shapes are filled in each frame by
//! the plugin's styling systems from the active theme, so the same entity
//! re-skins when the theme switches.

use bevy::prelude::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

/// What the pointer is doing to a control. Kept up to date by [`crate::DodaiUiPlugin`]
/// from picking events; every widget requires it.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlPointer {
    /// Not over the control.
    #[default]
    None,
    /// Over the control.
    Hovered,
    /// Held down on the control.
    Pressed,
}
use dodai_core::{ButtonVariant, Color as Rgba, Tone};

use crate::surface::Surface;

/// Colour a widget's text children inherit (written by the style systems).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Foreground(pub Rgba);

impl Default for Foreground {
    fn default() -> Self {
        Self(Rgba::hex(0xffffff))
    }
}

/// Typographic role of a [`UiText`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    /// Large display heading (`size_xl`), uppercase per `uppercase_labels`.
    Heading,
    /// Sub-heading (`size_lg`), uppercase per `uppercase_labels`.
    Subheading,
    /// Body text (`size_md`).
    Body,
    /// Small label (`size_sm`), uppercase per `uppercase_labels`.
    Label,
    /// Muted secondary text (`size_sm`).
    Muted,
    /// Monospace-styled text (`size_sm`; the bundled font is used).
    Mono,
    /// Smallest muted text (`size_xs`).
    Caption,
    /// Control label (`size_sm`), colour inherited, uppercase per tokens.
    Control,
    /// Small control label (`size_xs`), colour inherited, uppercase per tokens.
    ControlSmall,
    /// Raw content (`size_md`), colour inherited, never re-cased.
    Content,
    /// A role of the Material type scale (size from the theme's scale; colour inherited).
    Type(dodai_core::TypeRole),
}

/// Marks a button's label so its size follows the button's [`dodai_core::ControlSize`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonLabel(pub dodai_core::ControlSize);

/// Themed text: `raw` is stored un-cased; the system derives the displayed
/// string, size and colour from tokens (so theme changes re-case it).
#[derive(Component, Debug, Clone, PartialEq)]
pub struct UiText {
    /// Source string.
    pub raw: String,
    /// Role.
    pub role: TextRole,
}

/// Themed button.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(ControlPointer, FocusPolicy::Block, Surface, Foreground)]
pub struct UiButton {
    /// Emphasis.
    pub variant: ButtonVariant,
    /// Ignore input and show the disabled style.
    pub disabled: bool,
    /// Size step (height, padding and label size).
    pub size: dodai_core::ControlSize,
    /// Square corners instead of the design's own button shape.
    pub square: bool,
    /// No label: the button is as wide as it is high.
    pub icon_only: bool,
}

/// Panel / card.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
#[require(Surface, Foreground)]
pub struct UiPanel {
    /// Highlight the border (and title).
    pub focused: bool,
}

/// Tag / badge.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface, Foreground)]
pub struct UiTag {
    /// Status tone.
    pub tone: Tone,
    /// Solid fill vs outlined.
    pub solid: bool,
}

/// Row of tabs.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiTabs;

/// One tab.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[require(Surface, ControlPointer, Foreground)]
pub struct UiTab {
    /// Active tab.
    pub active: bool,
}

/// Which toggle control.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleKind {
    /// Track + thumb switch.
    Switch,
    /// Box with a mark.
    Checkbox,
    /// Round (in rounded themes) single-choice mark; exclusive among siblings.
    Radio,
}

/// Switch / checkbox / radio.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface, ControlPointer)]
pub struct UiToggle {
    /// Control kind.
    pub kind: ToggleKind,
    /// Checked.
    pub on: bool,
    /// Ignore input.
    pub disabled: bool,
}

/// Progress bar (`value` in `0..=1`).
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface)]
pub struct UiProgress {
    /// Fraction `0..=1`.
    pub value: f32,
    /// Status tone (`None` = brand colour).
    pub tone: Option<Tone>,
}

/// Draggable slider.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface, ControlPointer, RelativeCursorPosition)]
pub struct UiSlider {
    /// Current value.
    pub value: f32,
    /// Minimum.
    pub min: f32,
    /// Maximum.
    pub max: f32,
}

impl UiSlider {
    /// Value as a fraction of the range, `0..=1`.
    pub fn fraction(&self) -> f32 {
        if self.max > self.min {
            ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Single-line text input (keyboard focus via click).
#[derive(Component, Debug, Clone, PartialEq, Default)]
#[require(Surface, ControlPointer)]
pub struct UiTextInput {
    /// Current text.
    pub value: String,
    /// Shown while empty.
    pub placeholder: String,
    /// Show bullets instead of the characters.
    pub masked: bool,
    /// Visible lines (`1` = single line).
    pub lines: u8,
}

/// Selectable list row.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[require(Surface, ControlPointer, Foreground)]
pub struct UiListItem {
    /// Selected row.
    pub selected: bool,
}

/// 1-px separator.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiDivider {
    /// Vertical instead of horizontal.
    pub vertical: bool,
}

/// Tooltip text shown near the cursor while this entity is hovered.
#[derive(Component, Debug, Clone, PartialEq)]
#[require(ControlPointer)]
pub struct UiTooltip(pub String);

// Internal part markers (children styled from their owner).
#[derive(Component)]
pub(crate) struct TogglePart;
#[derive(Component)]
pub(crate) struct ProgressFill;
#[derive(Component)]
pub(crate) struct SliderFill;
#[derive(Component)]
pub(crate) struct SliderThumb;
#[derive(Component)]
pub(crate) struct TabIndicator;
#[derive(Component)]
pub(crate) struct ListMarker;
#[derive(Component)]
pub(crate) struct InputText;
#[derive(Component)]
pub(crate) struct TooltipPopup;
#[derive(Component)]
pub(crate) struct TooltipText;

/// Something a control reported.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct ControlEvent {
    /// Source entity.
    pub entity: Entity,
    /// What happened.
    pub kind: ControlKind,
}

/// Kind of [`ControlEvent`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlKind {
    /// Button pressed.
    Pressed,
    /// Switch/checkbox/radio now has this state.
    Toggled(bool),
    /// Tab became active.
    TabSelected,
    /// List row became selected.
    Selected,
    /// Slider value changed.
    Slid(f32),
    /// Text input content changed.
    TextChanged,
}

fn abs_fill() -> Node {
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
}

/// Themed text.
pub fn text(raw: impl Into<String>, role: TextRole) -> impl Bundle {
    let raw = raw.into();
    (
        Text::new(raw.clone()),
        TextFont::default(),
        TextColor::default(),
        UiText { raw, role },
    )
}

/// Heading text.
pub fn heading(raw: impl Into<String>) -> impl Bundle {
    text(raw, TextRole::Heading)
}
/// Label text.
pub fn label(raw: impl Into<String>) -> impl Bundle {
    text(raw, TextRole::Label)
}
/// Muted text.
pub fn muted(raw: impl Into<String>) -> impl Bundle {
    text(raw, TextRole::Muted)
}
/// Monospace-role text.
pub fn mono(raw: impl Into<String>) -> impl Bundle {
    text(raw, TextRole::Mono)
}
/// Body text.
pub fn body(raw: impl Into<String>) -> impl Bundle {
    text(raw, TextRole::Body)
}

/// A themed icon: Material Symbols on rounded themes, the Cyberpunk pack (or
/// faceted Material) elsewhere. Rasterised to an `ImageNode` by the plugin and
/// tinted with the inherited [`Foreground`] (or the theme text colour).
#[cfg(feature = "icons")]
#[derive(Component, Debug, Clone, PartialEq)]
pub struct UiIcon {
    pub icon: dodai_icons::IconId,
    pub size: u32,
    pub filled: bool,
}

/// Icon `icon` at `size` px.
#[cfg(feature = "icons")]
pub fn icon(icon: dodai_icons::IconId, size: u32) -> impl Bundle {
    (
        UiIcon {
            icon,
            size,
            filled: false,
        },
        Node {
            width: Val::Px(size as f32),
            height: Val::Px(size as f32),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// A dashboard symbol (vehicle, aviation, medical, military, industrial), drawn in the active
/// design's own style and coloured by what it reports (or the inherited [`Foreground`]).
#[cfg(feature = "symbols")]
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSymbol {
    pub symbol: dodai_icons::symbol::Symbol,
    pub size: u32,
    pub state: Option<dodai_core::SymbolState>,
}

/// Symbol `symbol` at `size` px.
#[cfg(feature = "symbols")]
pub fn symbol(symbol: dodai_icons::symbol::Symbol, size: u32) -> impl Bundle {
    (
        UiSymbol {
            symbol,
            size,
            state: None,
        },
        Node {
            width: Val::Px(size as f32),
            height: Val::Px(size as f32),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// Symbol `symbol` at `size` px, lit in the colour of `state`.
#[cfg(feature = "symbols")]
pub fn symbol_state(
    symbol: dodai_icons::symbol::Symbol,
    size: u32,
    state: dodai_core::SymbolState,
) -> impl Bundle {
    (
        UiSymbol {
            symbol,
            size,
            state: Some(state),
        },
        Node {
            width: Val::Px(size as f32),
            height: Val::Px(size as f32),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// Everything a button can be, built up with chained setters and spawned with
/// [`ButtonSpec::bundle`].
#[derive(Debug, Clone)]
pub struct ButtonSpec {
    label: String,
    variant: ButtonVariant,
    size: dodai_core::ControlSize,
    square: bool,
    disabled: bool,
    #[cfg(feature = "icons")]
    icon: Option<dodai_icons::IconId>,
}

impl ButtonSpec {
    /// A medium button.
    pub fn new(label: impl Into<String>, variant: ButtonVariant) -> Self {
        Self {
            label: label.into(),
            variant,
            size: dodai_core::ControlSize::Medium,
            square: false,
            disabled: false,
            #[cfg(feature = "icons")]
            icon: None,
        }
    }

    /// Size step.
    pub fn size(mut self, size: dodai_core::ControlSize) -> Self {
        self.size = size;
        self
    }

    /// Square corners.
    pub fn square(mut self, square: bool) -> Self {
        self.square = square;
        self
    }

    /// Ignore input.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A leading icon (the whole button when the label is empty).
    #[cfg(feature = "icons")]
    pub fn icon(mut self, icon: dodai_icons::IconId) -> Self {
        self.icon = Some(icon);
        self
    }

    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        let icon_only = self.label.is_empty();
        let label = self.label.clone();
        let size = self.size;
        #[cfg(feature = "icons")]
        let icon = self.icon;
        (
            UiButton {
                variant: self.variant,
                disabled: self.disabled,
                size: self.size,
                square: self.square,
                icon_only,
            },
            Node {
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                column_gap: Val::Px(6.0),
                flex_shrink: 0.0,
                ..default()
            },
            Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
                #[cfg(feature = "icons")]
                if let Some(i) = icon {
                    parent.spawn(crate::widgets::icon(i, 18));
                }
                if !label.is_empty() {
                    parent.spawn((text(label, TextRole::Control), ButtonLabel(size)));
                }
            })),
        )
    }
}

/// Button.
pub fn button(label: impl Into<String>, variant: ButtonVariant) -> impl Bundle {
    ButtonSpec::new(label, variant).bundle()
}

/// Disabled button.
pub fn button_disabled(label: impl Into<String>, variant: ButtonVariant) -> impl Bundle {
    ButtonSpec::new(label, variant).disabled(true).bundle()
}

/// Panel (card) with a title; add content with `.with_children`. Add your own
/// `Node` (width etc.) next to it: `(panel("T"), Node { width: px(300), ..default() })`.
pub fn panel(title: &str) -> impl Bundle + use<> {
    (
        UiPanel::default(),
        children![text(title.to_string(), TextRole::Control)],
    )
}

/// Panel (card) without a title.
pub fn panel_untitled() -> impl Bundle {
    UiPanel::default()
}

/// Tag / badge.
pub fn tag(label: impl Into<String>, tone: Tone, solid: bool) -> impl Bundle {
    (
        UiTag { tone, solid },
        Node {
            align_items: AlignItems::Center,
            align_self: AlignSelf::FlexStart,
            ..default()
        },
        children![text(label, TextRole::ControlSmall)],
    )
}

/// One tab.
pub fn tab(label: impl Into<String>, active: bool) -> impl Bundle {
    (
        UiTab { active },
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![
            text(label, TextRole::Control),
            (
                TabIndicator,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    height: Val::Px(2.0),
                    ..default()
                },
                BackgroundColor::default(),
            )
        ],
    )
}

/// Tab row; the tab at `active` starts selected.
pub fn tabs(labels: &[&str], active: usize) -> impl Bundle + use<> {
    let items: Vec<_> = labels
        .iter()
        .enumerate()
        .map(|(i, l)| tab(l.to_string(), i == active))
        .collect();
    (
        UiTabs,
        Node {
            flex_direction: FlexDirection::Row,
            ..default()
        },
        Children::spawn(SpawnIter(items.into_iter())),
    )
}

fn toggle(kind: ToggleKind, on: bool, disabled: bool) -> impl Bundle {
    (
        UiToggle { kind, on, disabled },
        Node {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_shrink: 0.0,
            ..default()
        },
        children![(TogglePart, abs_fill(), Surface::default())],
    )
}

/// Switch.
pub fn switch(on: bool) -> impl Bundle {
    toggle(ToggleKind::Switch, on, false)
}
/// Checkbox.
pub fn checkbox(on: bool) -> impl Bundle {
    toggle(ToggleKind::Checkbox, on, false)
}
/// Radio button (exclusive among its siblings).
pub fn radio(on: bool) -> impl Bundle {
    toggle(ToggleKind::Radio, on, false)
}
/// Disabled variant of any toggle.
pub fn toggle_disabled(kind: ToggleKind, on: bool) -> impl Bundle {
    toggle(kind, on, true)
}

/// Progress bar, full width.
pub fn progress(value: f32, tone: Option<Tone>) -> impl Bundle {
    (
        UiProgress {
            value: value.clamp(0.0, 1.0),
            tone,
        },
        Node {
            width: Val::Percent(100.0),
            ..default()
        },
        children![(
            ProgressFill,
            Node {
                height: Val::Percent(100.0),
                ..default()
            },
            Surface::default(),
        )],
    )
}

/// Slider, full width.
pub fn slider(value: f32, min: f32, max: f32) -> impl Bundle {
    (
        UiSlider { value, min, max },
        Node {
            width: Val::Percent(100.0),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                SliderFill,
                Node {
                    height: Val::Percent(100.0),
                    ..default()
                },
                Surface::default(),
            ),
            (SliderThumb, abs_fill(), Surface::default()),
        ],
    )
}

/// Text input showing bullets instead of its characters.
pub fn password_input(placeholder: impl Into<String>) -> impl Bundle {
    text_field(placeholder, true, 1)
}

/// Multi-line text input showing `lines` lines.
pub fn text_area(placeholder: impl Into<String>, lines: u8) -> impl Bundle {
    text_field(placeholder, false, lines)
}

/// Text input.
pub fn text_input(placeholder: impl Into<String>) -> impl Bundle {
    text_field(placeholder, false, 1)
}

fn text_field(placeholder: impl Into<String>, masked: bool, lines: u8) -> impl Bundle {
    (
        UiTextInput {
            value: String::new(),
            placeholder: placeholder.into(),
            masked,
            lines,
        },
        Node {
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        children![(
            InputText,
            text("", TextRole::Content),
            Foreground::default(),
        )],
    )
}

/// Selectable list row.
pub fn list_item(label: impl Into<String>, selected: bool) -> impl Bundle {
    (
        UiListItem { selected },
        Node {
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            text(label, TextRole::Content),
            (
                ListMarker,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    width: Val::Px(3.0),
                    ..default()
                },
                BackgroundColor::default(),
            )
        ],
    )
}

/// Horizontal divider.
pub fn divider() -> impl Bundle {
    (
        UiDivider::default(),
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(1.0),
            ..default()
        },
        BackgroundColor::default(),
    )
}

/// Vertical divider.
pub fn divider_vertical() -> impl Bundle {
    (
        UiDivider { vertical: true },
        Node {
            width: Val::Px(1.0),
            align_self: AlignSelf::Stretch,
            ..default()
        },
        BackgroundColor::default(),
    )
}
