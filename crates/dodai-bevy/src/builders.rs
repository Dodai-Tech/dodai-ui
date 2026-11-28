//! Builders named after the spec (`dist/design/spec`): the same component, property and event
//! names as every other toolkit. Each builder turns into a bundle; the free functions
//! (`badge`, `avatar`, ...) stay for existing callers.

use bevy::prelude::*;
use dodai_core::Tone;

use crate::components::{alert, avatar, badge, breadcrumb, chip};
use crate::overlay::accordion;
use crate::widgets::{self, ButtonSpec, TextRole};
use dodai_core::{ButtonVariant, ControlSize, Crop};

/// A count or short label in a tone.
pub struct Badge {
    text: String,
    tone: Tone,
}

impl Badge {
    /// A badge showing `text` in `tone`.
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
        }
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        badge(self.text, self.tone)
    }
}

/// A round avatar with initials.
pub struct Avatar {
    initials: String,
    size: f32,
}

impl Avatar {
    /// An avatar showing `initials`.
    pub fn new(initials: impl Into<String>) -> Self {
        Self {
            initials: initials.into(),
            size: 32.0,
        }
    }
    /// Size in px.
    #[must_use]
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        avatar(self.initials, self.size)
    }
}

/// A trail of places, the last one current.
pub struct Breadcrumb {
    items: Vec<String>,
}

impl Breadcrumb {
    /// A breadcrumb over `items`.
    pub fn new<S: Into<String>>(items: impl IntoIterator<Item = S>) -> Self {
        Self {
            items: items.into_iter().map(Into::into).collect(),
        }
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        let items: Vec<&str> = self.items.iter().map(String::as_str).collect();
        breadcrumb(&items)
    }
}

/// Sections that fold: a header over a body.
#[derive(Default)]
pub struct Accordion {
    items: Vec<(String, String)>,
    expanded: Option<usize>,
}

impl Accordion {
    /// An empty accordion.
    pub fn new() -> Self {
        Self::default()
    }
    /// Add an item.
    #[must_use]
    pub fn item(mut self, title: impl Into<String>, body: impl Into<String>) -> Self {
        self.items.push((title.into(), body.into()));
        self
    }
    /// The item shown open at the start.
    #[must_use]
    pub fn expanded(mut self, open: usize) -> Self {
        self.expanded = Some(open);
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        accordion(self.items, self.expanded)
    }
}

/// A message in a tone.
pub struct Alert {
    tone: Tone,
    message: String,
}

impl Alert {
    /// An alert of `tone`.
    pub fn new(tone: Tone) -> Self {
        Self {
            tone,
            message: String::new(),
        }
    }
    /// Message text.
    #[must_use]
    pub fn message(mut self, m: impl Into<String>) -> Self {
        self.message = m.into();
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        alert(self.tone, self.message)
    }
}

/// A button. Not in the prelude: bevy has its own `Button`; use `builders::Button`.
pub struct Button {
    spec: ButtonSpec,
}

impl Button {
    /// A button labelled `label` (empty for an icon-only button).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            spec: ButtonSpec::new(label, ButtonVariant::Primary),
        }
    }
    /// The emphasis variant (default `Primary`).
    #[must_use]
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.spec = ButtonSpec::from_parts(self.spec, v);
        self
    }
    /// Ignore input and use the disabled tokens.
    #[must_use]
    pub fn disabled(mut self, d: bool) -> Self {
        self.spec = self.spec.disabled(d);
        self
    }
    /// Size step.
    #[must_use]
    pub fn size(mut self, s: ControlSize) -> Self {
        self.spec = self.spec.size(s);
        self
    }
    /// Square corners instead of the pill, on rounded designs.
    #[must_use]
    pub fn square(mut self, on: bool) -> Self {
        self.spec = self.spec.square(on);
        self
    }
    /// Crop one end the way the design crops buttons.
    #[must_use]
    pub fn crop(mut self, c: Crop) -> Self {
        self.spec = self.spec.crop(c);
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        self.spec.bundle()
    }
}

/// A rule between content.
#[derive(Default)]
pub struct Divider {
    vertical: bool,
}

impl Divider {
    /// A horizontal divider.
    pub fn new() -> Self {
        Self::default()
    }
    /// Vertical (needs a parent with a height).
    #[must_use]
    pub fn vertical(mut self, on: bool) -> Self {
        self.vertical = on;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        let node = if self.vertical {
            Node {
                width: Val::Px(1.0),
                align_self: AlignSelf::Stretch,
                ..default()
            }
        } else {
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(1.0),
                ..default()
            }
        };
        (
            widgets::UiDivider {
                vertical: self.vertical,
            },
            node,
            BackgroundColor::default(),
        )
    }
}

/// Section title text.
pub struct Heading(String);

impl Heading {
    /// A heading reading `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self(text.into())
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        widgets::heading(self.0)
    }
}

/// Themed text in a role. Not in the prelude: bevy has its own `Text`; use `builders::Text`.
pub struct Text {
    role: TextRole,
    text: String,
    muted: bool,
}

impl Text {
    /// `text` in `role`.
    pub fn new(role: TextRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
            muted: false,
        }
    }
    /// The secondary text colour.
    #[must_use]
    pub fn muted(mut self, on: bool) -> Self {
        self.muted = on;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        widgets::text(
            self.text,
            if self.muted {
                TextRole::Muted
            } else {
                self.role
            },
        )
    }
}

/// A small tinted label.
pub struct Tag {
    text: String,
    tone: Tone,
    solid: bool,
}

impl Tag {
    /// A tag reading `text` in `tone`.
    pub fn new(text: impl Into<String>, tone: Tone) -> Self {
        Self {
            text: text.into(),
            tone,
            solid: false,
        }
    }
    /// Solid fill (default is a tinted background).
    #[must_use]
    pub fn solid(mut self, on: bool) -> Self {
        self.solid = on;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        widgets::tag(self.text, self.tone, self.solid)
    }
}

/// A spinning ring.
pub struct Spinner {
    size: f32,
    style: Option<dodai_core::SpinnerStyle>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            size: 24.0,
            style: None,
        }
    }
}

impl Spinner {
    /// A 24 px spinner in the design's own style.
    pub fn new() -> Self {
        Self::default()
    }
    /// Side in px.
    #[must_use]
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
    /// Draw this style instead of the design's own.
    #[must_use]
    pub fn style(mut self, s: dodai_core::SpinnerStyle) -> Self {
        self.style = Some(s);
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        crate::spinner::spinner_with(self.size, self.style)
    }
}

/// A selectable pill.
pub struct Chip {
    label: String,
    selected: bool,
}

impl Chip {
    /// A chip labelled `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            selected: false,
        }
    }
    /// Shown selected.
    #[must_use]
    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        chip(self.label, self.selected)
    }
}

macro_rules! choice {
    ($(#[$m:meta])* $name:ident, $kind:expr) => {
        $(#[$m])*
        pub struct $name {
            on: bool,
            disabled: bool,
        }
        impl $name {
            /// `on` is the current value.
            pub fn new(on: bool) -> Self {
                Self { on, disabled: false }
            }
            /// Disable interaction.
            #[must_use]
            pub fn disabled(mut self, d: bool) -> Self {
                self.disabled = d;
                self
            }
            /// The bundle to spawn.
            pub fn bundle(self) -> impl Bundle {
                widgets::toggle($kind, self.on, self.disabled)
            }
        }
    };
}

choice!(
    /// On/off switch.
    Toggle,
    widgets::ToggleKind::Switch
);
choice!(
    /// Checkbox with a tick.
    Checkbox,
    widgets::ToggleKind::Checkbox
);
choice!(
    /// Radio button, exclusive among its siblings.
    Radio,
    widgets::ToggleKind::Radio
);

/// A draggable value in a range.
pub struct Slider {
    value: f32,
    min: f32,
    max: f32,
}

impl Slider {
    /// A slider at `value`, over 0..=1 until `range` says otherwise.
    pub fn new(value: f32) -> Self {
        Self {
            value,
            min: 0.0,
            max: 1.0,
        }
    }
    /// Value range.
    #[must_use]
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        widgets::slider(self.value, self.min, self.max)
    }
}

/// A bar filled to a fraction.
pub struct ProgressBar {
    value: f32,
    tone: Option<Tone>,
}

impl ProgressBar {
    /// A bar filled to `value` (0..=1).
    pub fn new(value: f32) -> Self {
        Self { value, tone: None }
    }
    /// Colour by status tone (default: the brand colour).
    #[must_use]
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }
    /// The bundle to spawn.
    pub fn bundle(self) -> impl Bundle {
        widgets::progress(self.value, self.tone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builders_make_bundles() {
        fn is_bundle(_: impl Bundle) {}
        is_bundle(Badge::new("3", Tone::Info).bundle());
        is_bundle(Avatar::new("AB").size(40.0).bundle());
        is_bundle(Breadcrumb::new(["Home", "Docs"]).bundle());
        is_bundle(Accordion::new().item("a", "b").expanded(0).bundle());
        is_bundle(Alert::new(Tone::Warning).message("Low").bundle());
        is_bundle(
            Button::new("Go")
                .variant(ButtonVariant::Ghost)
                .disabled(true)
                .bundle(),
        );
        is_bundle(Divider::new().vertical(true).bundle());
        is_bundle(Heading::new("T").bundle());
        is_bundle(Text::new(TextRole::Content, "t").muted(true).bundle());
        is_bundle(Tag::new("x", Tone::Info).solid(true).bundle());
        is_bundle(Spinner::new().size(16.0).bundle());
        is_bundle(Chip::new("c").selected(true).bundle());
        is_bundle(Toggle::new(true).disabled(true).bundle());
        is_bundle(Checkbox::new(false).bundle());
        is_bundle(Radio::new(true).bundle());
        is_bundle(Slider::new(0.5).range(0.0, 2.0).bundle());
        is_bundle(ProgressBar::new(0.3).tone(Tone::Success).bundle());
    }
}
