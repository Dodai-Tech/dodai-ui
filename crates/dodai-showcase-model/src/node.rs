//! The demo widget tree: what a section shows, as plain data.
//!
//! A section builder returns a [`Node`]; each toolkit walks the tree and draws every variant
//! with its own components. Interactive nodes carry a [`Bind`] (a key into the showcase's
//! [`Values`](crate::Values) plus a default), so a click in any toolkit goes through the same
//! state and the same tests.

use dodai_core::{ButtonVariant, Color, ControlSize, ToggleKind, Tone, TypeRole};
use dodai_icons::IconId;

/// Where an interactive node keeps its value: a key into [`Values`](crate::Values) and the
/// value shown until the user changes it.
#[derive(Debug, Clone, PartialEq)]
pub struct Bind<T> {
    /// State key, unique across the showcase.
    pub key: String,
    /// Value while the key is unset.
    pub default: T,
}

impl<T> Bind<T> {
    /// A binding to `key` starting at `default`.
    pub fn new(key: impl Into<String>, default: T) -> Self {
        Self {
            key: key.into(),
            default,
        }
    }
}

/// Layout direction of a [`Stack`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Children side by side.
    Row,
    /// Children one below the other.
    Column,
}

/// Gap between the children of a [`Stack`], in theme spacing steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gap {
    /// `spacing.xs`.
    Xs,
    /// `spacing.sm`.
    Sm,
    /// `spacing.md`.
    Md,
    /// `spacing.lg`.
    Lg,
}

/// Children laid out in a row or a column.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// Direction.
    pub axis: Axis,
    /// A row wraps onto the next line when it runs out of width.
    pub wrap: bool,
    /// Space between children.
    pub gap: Gap,
    /// Centre children across the axis.
    pub centre: bool,
    /// The children.
    pub items: Vec<Node>,
}

/// Text roles: the showcase's own plus the Material type scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextRole {
    /// Page title.
    Title,
    /// Small label (design chips, field captions).
    Label,
    /// Body copy.
    Body,
    /// Secondary text.
    Muted,
    /// Smallest secondary text.
    Caption,
    /// Monospaced code.
    Mono,
    /// A role of the type scale.
    Type(TypeRole),
}

/// A button.
#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    /// Label (empty for an icon-only button).
    pub label: String,
    /// Emphasis.
    pub variant: ButtonVariant,
    /// Size step.
    pub size: ControlSize,
    /// Square corners instead of the pill.
    pub square: bool,
    /// Leading icon.
    pub icon: Option<IconId>,
    /// Shows its busy state.
    pub loading: bool,
    /// Ignores input.
    pub disabled: bool,
    /// Counts presses under this key when set.
    pub counter: Option<String>,
}

/// A row of options of which one is selected (button group, segmented control, tabs).
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    /// Selected index.
    pub bind: Bind<usize>,
    /// Option labels.
    pub labels: Vec<String>,
    /// Drawn as one connected strip.
    pub connected: bool,
}

/// Switch, checkbox or radio.
#[derive(Debug, Clone, PartialEq)]
pub struct Toggle {
    /// Which control.
    pub kind: ToggleKind,
    /// Checked state (radios: whether this one is picked).
    pub bind: Bind<bool>,
    /// Caption next to it.
    pub label: String,
    /// Ignores input.
    pub disabled: bool,
    /// Radios of one group share this key; picking one sets it to `index`.
    pub radio: Option<(String, usize)>,
}

/// A value slider.
#[derive(Debug, Clone, PartialEq)]
pub struct Slider {
    /// The value.
    pub bind: Bind<f32>,
    /// Lowest value.
    pub min: f32,
    /// Highest value.
    pub max: f32,
    /// Snap to multiples of this when set.
    pub step: Option<f32>,
    /// Shows the number next to the track.
    pub readout: bool,
    /// Runs bottom to top instead of left to right.
    pub vertical: bool,
}

/// A text field.
#[derive(Debug, Clone, PartialEq)]
pub struct Input {
    /// The text.
    pub bind: Bind<String>,
    /// Shown while empty.
    pub placeholder: String,
    /// Caption above the field.
    pub label: Option<String>,
    /// Hide the characters.
    pub password: bool,
    /// Visible lines.
    pub lines: u8,
    /// Icon at the start.
    pub icon: Option<IconId>,
}

/// A row of a [`Node::List`].
#[derive(Debug, Clone, PartialEq)]
pub struct ListRow {
    /// Main text.
    pub title: String,
    /// Second line.
    pub detail: Option<String>,
    /// Leading icon.
    pub icon: Option<IconId>,
    /// Trailing text.
    pub trailing: Option<String>,
    /// Small text above the title.
    pub overline: Option<String>,
    /// Initials of a leading avatar.
    pub avatar: Option<String>,
}

/// One event of a timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineEvent {
    /// Title.
    pub title: String,
    /// When.
    pub time: String,
    /// Details.
    pub detail: Option<String>,
    /// Status colour of the marker.
    pub tone: Option<Tone>,
}

/// One step of a stepper or timeline.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Title.
    pub title: String,
    /// Description.
    pub detail: Option<String>,
}

/// A node of a tree view.
#[derive(Debug, Clone, PartialEq)]
pub struct TreeNode {
    /// Label.
    pub label: String,
    /// Icon.
    pub icon: Option<IconId>,
    /// Children (empty for a leaf).
    pub children: Vec<TreeNode>,
    /// Expanded when first shown.
    pub open: bool,
}

/// A demo widget.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Children in a row or column.
    Stack(Stack),
    /// Text.
    Text(String, TextRole),
    /// An empty gap of this many px.
    Space(f32),
    /// A colour chip with its name and value below.
    Swatch {
        /// Token name.
        name: String,
        /// The colour.
        color: Color,
    },
    /// A plain rectangle of `w`×`h` px (spacing, radius and elevation demos).
    Block {
        /// Width px.
        w: f32,
        /// Height px.
        h: f32,
        /// Fill.
        fill: Color,
        /// Outline.
        border: Option<Color>,
        /// Corner radius px.
        radius: f32,
        /// Text inside.
        label: Option<String>,
    },
    /// A button.
    Button(Button),
    /// A split button: the action and its menu arrow.
    Split {
        /// Action label.
        label: String,
        /// Emphasis.
        variant: ButtonVariant,
    },
    /// A choice strip (button group or segmented control).
    Choice(Choice),
    /// Tab bar.
    Tabs(Choice),
    /// A tag / badge.
    Tag {
        /// Text.
        label: String,
        /// Status tone.
        tone: Tone,
        /// Solid fill instead of outline.
        solid: bool,
    },
    /// A filter chip.
    Chip {
        /// Text.
        label: String,
        /// Selected state.
        bind: Bind<bool>,
        /// Leading icon.
        icon: Option<IconId>,
    },
    /// A switch, checkbox or radio.
    Toggle(Toggle),
    /// A slider.
    Slider(Slider),
    /// A rotary knob over `min..=max`.
    Knob {
        /// The value.
        bind: Bind<f32>,
        /// Lowest value.
        min: f32,
        /// Highest value.
        max: f32,
        /// Diameter in px.
        size: f32,
        /// Dimmed and not draggable.
        disabled: bool,
    },
    /// A fixed-height area whose content scrolls.
    Scroll {
        /// Height in px.
        height: f32,
        /// The scrolling content.
        body: Box<Node>,
    },
    /// A progress bar (`0..=1`).
    Progress {
        /// Fraction.
        value: f32,
        /// Status tone.
        tone: Option<Tone>,
    },
    /// A text field.
    Input(Input),
    /// Rows of a list with a divider between them.
    List(Vec<ListRow>),
    /// A separator line.
    Divider,
    /// A titled card.
    Card {
        /// Heading.
        title: String,
        /// Content.
        body: Box<Node>,
        /// Variant: elevated, filled or outlined.
        variant: dodai_core::CardVariant,
    },
    /// A message bar (inline or full-width banner).
    Alert {
        /// Tone.
        tone: Tone,
        /// Message.
        text: String,
        /// Full-width banner.
        banner: bool,
    },
    /// A snackbar with an action.
    Snackbar {
        /// Message.
        text: String,
        /// Action label.
        action: String,
    },
    /// A floating menu or dropdown list around its content.
    Popup(Box<Node>),
    /// An icon.
    Icon(IconId, f32),
    /// Star rating.
    Rating {
        /// Filled stars (halves allowed).
        value: f32,
        /// Number of stars.
        max: u8,
    },
    /// A − / value / + counter.
    Stepper(Bind<i32>),
    /// A numbered step indicator.
    Steps {
        /// The steps.
        steps: Vec<Step>,
        /// Index of the current step.
        current: usize,
    },
    /// Breadcrumb trail (the last item is the current page).
    Breadcrumb(Vec<String>),
    /// Page selector.
    Pagination {
        /// Number of pages.
        pages: usize,
        /// Selected page.
        bind: Bind<usize>,
    },
    /// An expandable tree.
    Tree(Vec<TreeNode>),
    /// A data table.
    Table {
        /// Column headings.
        columns: Vec<String>,
        /// Rows of cells.
        rows: Vec<Vec<String>>,
    },
    /// A round avatar with initials.
    Avatar(String),
    /// A tooltip shown on its own beside the trigger.
    Tooltip {
        /// Tooltip text.
        text: String,
        /// Rich variant (title and body).
        rich: bool,
    },
    /// A code listing.
    Code(String),
    /// Events down a line.
    Timeline(Vec<TimelineEvent>),
    /// An icon button with a count or dot badge.
    Badged {
        /// The icon.
        icon: IconId,
        /// Count badge.
        count: Option<(u32, Tone)>,
        /// Dot badge.
        dot: Option<Tone>,
    },
    /// A spinning ring.
    Spinner(f32),
    /// An indeterminate bar.
    Indeterminate,
    /// A placeholder block shown while content loads.
    Skeleton {
        /// Width px (`0` = fill).
        w: f32,
        /// Height px.
        h: f32,
    },
    /// The empty state of a list.
    Empty {
        /// Title.
        title: String,
        /// Hint.
        detail: String,
    },
}

/// Builders: short names so section code reads like the layout.
pub mod b {
    use super::*;

    fn stack(axis: Axis, wrap: bool, gap: Gap, items: Vec<Node>) -> Node {
        Node::Stack(Stack {
            axis,
            wrap,
            gap,
            centre: axis == Axis::Row,
            items,
        })
    }

    /// Column with medium gaps.
    pub fn col(items: Vec<Node>) -> Node {
        stack(Axis::Column, false, Gap::Md, items)
    }

    /// Column with large gaps (a section's rows).
    pub fn rows(items: Vec<Node>) -> Node {
        stack(Axis::Column, false, Gap::Lg, items)
    }

    /// Wrapping row with medium gaps.
    pub fn row(items: Vec<Node>) -> Node {
        stack(Axis::Row, true, Gap::Md, items)
    }

    /// A column that is only as wide as its widest child, children centred (a dial and its
    /// readout).
    pub fn dial(items: Vec<Node>) -> Node {
        Node::Stack(Stack {
            axis: Axis::Column,
            wrap: false,
            gap: Gap::Xs,
            centre: true,
            items,
        })
    }

    /// Row that never wraps, small gaps.
    pub fn line(items: Vec<Node>) -> Node {
        stack(Axis::Row, false, Gap::Sm, items)
    }

    /// Body text.
    pub fn body(s: impl Into<String>) -> Node {
        Node::Text(s.into(), TextRole::Body)
    }

    /// Secondary text.
    pub fn muted(s: impl Into<String>) -> Node {
        Node::Text(s.into(), TextRole::Muted)
    }

    /// Small label.
    pub fn label(s: impl Into<String>) -> Node {
        Node::Text(s.into(), TextRole::Label)
    }

    /// Text in a role.
    pub fn text(s: impl Into<String>, role: TextRole) -> Node {
        Node::Text(s.into(), role)
    }

    /// A medium primary button.
    pub fn button(label: impl Into<String>, variant: ButtonVariant) -> Button {
        Button {
            label: label.into(),
            variant,
            size: ControlSize::Medium,
            square: false,
            icon: None,
            loading: false,
            disabled: false,
            counter: None,
        }
    }

    impl Button {
        /// Size step.
        pub fn size(mut self, size: ControlSize) -> Self {
            self.size = size;
            self
        }
        /// Square corners.
        pub fn square(mut self) -> Self {
            self.square = true;
            self
        }
        /// Leading icon.
        pub fn icon(mut self, icon: IconId) -> Self {
            self.icon = Some(icon);
            self
        }
        /// Busy state.
        pub fn loading(mut self) -> Self {
            self.loading = true;
            self
        }
        /// Disabled.
        pub fn disabled(mut self) -> Self {
            self.disabled = true;
            self
        }
        /// Count presses under `key`.
        pub fn counter(mut self, key: impl Into<String>) -> Self {
            self.counter = Some(key.into());
            self
        }
        /// As a node.
        pub fn node(self) -> Node {
            Node::Button(self)
        }
    }

    /// A choice strip.
    pub fn choice(key: &str, labels: &[&str], default: usize) -> Choice {
        Choice {
            bind: Bind::new(key, default),
            labels: labels.iter().map(|s| s.to_string()).collect(),
            connected: false,
        }
    }

    impl Choice {
        /// One connected strip.
        pub fn connected(mut self) -> Self {
            self.connected = true;
            self
        }
    }

    /// A tag.
    pub fn tag(label: impl Into<String>, tone: Tone, solid: bool) -> Node {
        Node::Tag {
            label: label.into(),
            tone,
            solid,
        }
    }

    /// A toggle of `kind`.
    pub fn toggle(kind: ToggleKind, key: &str, on: bool, label: impl Into<String>) -> Toggle {
        Toggle {
            kind,
            bind: Bind::new(key, on),
            label: label.into(),
            disabled: false,
            radio: None,
        }
    }

    impl Toggle {
        /// Disabled.
        pub fn disabled(mut self) -> Self {
            self.disabled = true;
            self
        }
        /// Member `index` of the radio group `group`.
        pub fn in_group(mut self, group: &str, index: usize) -> Self {
            self.radio = Some((group.to_string(), index));
            self
        }
        /// As a node.
        pub fn node(self) -> Node {
            Node::Toggle(self)
        }
    }

    /// A slider over `min..=max`.
    pub fn slider(key: &str, value: f32, min: f32, max: f32) -> Slider {
        Slider {
            bind: Bind::new(key, value),
            min,
            max,
            step: None,
            readout: true,
            vertical: false,
        }
    }

    /// A knob over `min..=max`, `size` px across.
    pub fn knob(key: &str, value: f32, min: f32, max: f32, size: f32) -> Node {
        Node::Knob {
            bind: Bind::new(key, value),
            min,
            max,
            size,
            disabled: false,
        }
    }

    /// `body` in a scroll area `height` px tall.
    pub fn scroll(height: f32, body: Node) -> Node {
        Node::Scroll {
            height,
            body: Box::new(body),
        }
    }

    impl Slider {
        /// Snap to multiples of `step`.
        pub fn step(mut self, step: f32) -> Self {
            self.step = Some(step);
            self
        }
        /// Show or hide the number.
        pub fn readout(mut self, on: bool) -> Self {
            self.readout = on;
            self
        }
        /// Run bottom to top.
        pub fn vertical(mut self) -> Self {
            self.vertical = true;
            self
        }
        /// As a node.
        pub fn node(self) -> Node {
            Node::Slider(self)
        }
    }

    /// A text field.
    pub fn input(key: &str, placeholder: impl Into<String>) -> Input {
        Input {
            bind: Bind::new(key, String::new()),
            placeholder: placeholder.into(),
            label: None,
            password: false,
            lines: 1,
            icon: None,
        }
    }

    impl Input {
        /// Caption.
        pub fn label(mut self, label: impl Into<String>) -> Self {
            self.label = Some(label.into());
            self
        }
        /// Start text.
        pub fn value(mut self, v: impl Into<String>) -> Self {
            self.bind.default = v.into();
            self
        }
        /// Hidden characters.
        pub fn password(mut self) -> Self {
            self.password = true;
            self
        }
        /// Visible lines.
        pub fn lines(mut self, n: u8) -> Self {
            self.lines = n;
            self
        }
        /// Leading icon.
        pub fn icon(mut self, icon: IconId) -> Self {
            self.icon = Some(icon);
            self
        }
        /// As a node.
        pub fn node(self) -> Node {
            Node::Input(self)
        }
    }

    /// A list row.
    pub fn list_row(title: impl Into<String>) -> ListRow {
        ListRow {
            title: title.into(),
            detail: None,
            icon: None,
            trailing: None,
            overline: None,
            avatar: None,
        }
    }

    impl ListRow {
        /// Text above the title.
        pub fn overline(mut self, s: impl Into<String>) -> Self {
            self.overline = Some(s.into());
            self
        }
        /// Leading avatar initials.
        pub fn avatar(mut self, s: impl Into<String>) -> Self {
            self.avatar = Some(s.into());
            self
        }
        /// Second line.
        pub fn detail(mut self, s: impl Into<String>) -> Self {
            self.detail = Some(s.into());
            self
        }
        /// Leading icon.
        pub fn icon(mut self, icon: IconId) -> Self {
            self.icon = Some(icon);
            self
        }
        /// Trailing text.
        pub fn trailing(mut self, s: impl Into<String>) -> Self {
            self.trailing = Some(s.into());
            self
        }
    }

    /// A tree node.
    pub fn tree(
        label: impl Into<String>,
        icon: Option<IconId>,
        children: Vec<TreeNode>,
    ) -> TreeNode {
        let open = !children.is_empty();
        TreeNode {
            label: label.into(),
            icon,
            children,
            open,
        }
    }

    /// A step.
    pub fn step(title: impl Into<String>, detail: Option<&str>) -> Step {
        Step {
            title: title.into(),
            detail: detail.map(str::to_string),
        }
    }
}

impl Node {
    /// Visits this node and every descendant, parents first.
    pub fn walk(&self, f: &mut impl FnMut(&Node)) {
        f(self);
        match self {
            Node::Stack(s) => s.items.iter().for_each(|n| n.walk(f)),
            Node::Card { body, .. } | Node::Scroll { body, .. } => body.walk(f),
            Node::Popup(body) => body.walk(f),
            _ => {}
        }
    }
}
