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

/// What a shader node shows; a toolkit turns it into `dodai_shaders::player::Spec`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShaderSpec {
    /// Catalogue ids: a generator, then post effects.
    pub chain: Vec<&'static str>,
    /// A preset id: its chain and its own palette win over `chain`.
    pub preset: Option<&'static str>,
    /// Colours from the active design instead of the shader's own.
    pub themed: bool,
    /// Width in px; the height follows `aspect`.
    pub width: f32,
    /// Width ÷ height.
    pub aspect: f32,
    /// Post-effect strength `0..=1`.
    pub intensity: f32,
    /// Device pixels per shader pixel.
    pub pixel: f32,
    /// Moves, or one still frame.
    pub animated: bool,
}

impl ShaderSpec {
    /// A chain at the default size, moving.
    pub fn chain(ids: &[&'static str]) -> Self {
        Self {
            chain: ids.to_vec(),
            preset: None,
            themed: false,
            width: 220.0,
            aspect: 16.0 / 9.0,
            intensity: 1.0,
            pixel: 2.0,
            animated: true,
        }
    }

    /// A ready-made preset.
    pub fn preset(id: &'static str) -> Self {
        Self {
            preset: Some(id),
            ..Self::chain(&[])
        }
    }

    /// One still frame.
    pub fn still(mut self) -> Self {
        self.animated = false;
        self
    }

    /// Width in px.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }
}

impl ShaderSpec {
    /// The player description a toolkit view takes.
    pub fn to_spec(&self, theme: &dodai_core::Theme) -> dodai_shaders::player::Spec {
        use dodai_shaders::player::Spec;
        let mut spec = match self.preset {
            Some(id) => dodai_shaders::presets::all()
                .find(|p| p.id == id)
                .map_or_else(|| Spec::new(""), Spec::preset),
            None => self.chain.iter().skip(1).fold(
                Spec::new(self.chain.first().copied().unwrap_or("")),
                |s, id| s.fx(id),
            ),
        };
        if self.themed {
            spec = spec.themed(theme);
        }
        spec.aspect(self.aspect)
            .intensity(self.intensity)
            .pixel(self.pixel)
            .animated(self.animated)
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gap {
    /// `spacing.xs`.
    Xs,
    /// `spacing.sm`.
    Sm,
    /// `spacing.md`.
    Md,
    /// `spacing.lg`.
    Lg,
    /// `spacing.xl`.
    Xl,
    /// Exactly this many px.
    Px(f32),
}

impl Gap {
    /// The gap in px under `theme`.
    pub fn px(self, theme: &dodai_core::Theme) -> f32 {
        let s = &theme.spacing;
        match self {
            Gap::Xs => s.xs,
            Gap::Sm => s.sm,
            Gap::Md => s.md,
            Gap::Lg => s.lg,
            Gap::Xl => s.xl,
            Gap::Px(v) => v,
        }
    }
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
    /// A fixed width in px (a gallery tile), or as wide as the layout gives.
    pub width: Option<f32>,
    /// The children.
    pub items: Vec<Node>,
}

/// A font family of the theme's typography.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Font {
    /// `typography.display`.
    Display,
    /// `typography.body`.
    Body,
    /// `typography.mono`.
    Mono,
}

/// A text colour of the palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ink {
    /// `palette.text`.
    Text,
    /// `palette.text_muted`.
    Muted,
    /// `palette.text_disabled`.
    Disabled,
    /// `palette.primary`.
    Primary,
    /// `palette.link`.
    Link,
    /// The panel title colour.
    Title,
}

impl Ink {
    /// The colour under `theme`.
    pub fn color(self, theme: &dodai_core::Theme) -> Color {
        let p = &theme.palette;
        match self {
            Ink::Text => p.text,
            Ink::Muted => p.text_muted,
            Ink::Disabled => p.text_disabled,
            Ink::Primary => p.primary,
            Ink::Link => p.link,
            Ink::Title => theme.panel(false).title,
        }
    }
}

/// How a run of text is drawn, resolved from a [`TextRole`] and the theme: every toolkit
/// draws text from this, so sizes and colours match the gpui showcase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// Size in px.
    pub size: f32,
    /// Family.
    pub font: Font,
    /// Colour.
    pub ink: Ink,
    /// Label casing (uppercased when the theme's labels are).
    pub caps: bool,
}

impl TextStyle {
    /// `s` as this style shows it (label casing applied).
    pub fn cased(&self, theme: &dodai_core::Theme, s: &str) -> String {
        if self.caps && theme.typography.uppercase_labels {
            s.to_uppercase()
        } else {
            s.to_string()
        }
    }
}

/// Heading size for a level (1 = largest … 4 = smallest), as the gpui `Heading`.
pub fn heading_size(t: &dodai_core::Typography, level: u8) -> f32 {
    match level {
        0 | 1 => t.size_xxl,
        2 => t.size_xl,
        3 => t.size_lg,
        _ => t.size_md,
    }
}

/// Text roles: the showcase's own plus the Material type scale.
#[derive(Debug, Clone, Copy, PartialEq)]
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
    /// A display-font heading of level 1..=4.
    Heading(u8),
    /// An explicit size, family and colour.
    Styled(TextStyle),
}

impl TextRole {
    /// Size, family, colour and casing under `theme` (the gpui showcase's text components).
    pub fn resolve(self, theme: &dodai_core::Theme) -> TextStyle {
        let ty = &theme.typography;
        let st = |size, font, ink, caps| TextStyle {
            size,
            font,
            ink,
            caps,
        };
        match self {
            TextRole::Title => st(ty.size_xl, Font::Display, Ink::Text, true),
            TextRole::Heading(l) => st(heading_size(ty, l), Font::Display, Ink::Text, true),
            TextRole::Label => st(ty.size_sm, Font::Body, Ink::Text, true),
            TextRole::Body => st(ty.size_md, Font::Body, Ink::Text, false),
            TextRole::Muted => st(ty.size_sm, Font::Body, Ink::Muted, false),
            TextRole::Caption => st(ty.size_xs, Font::Body, Ink::Muted, false),
            TextRole::Mono => st(ty.size_sm, Font::Mono, Ink::Text, false),
            TextRole::Type(r) => {
                let font = if r.is_heading() {
                    Font::Display
                } else {
                    Font::Body
                };
                st(theme.type_style(r).size, font, Ink::Text, false)
            }
            TextRole::Styled(s) => s,
        }
    }
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
    /// Which end the design crops (`None` = the design's own default).
    pub crop: Option<dodai_core::Crop>,
    /// Never crops, whatever the design's default.
    pub plain: bool,
    /// The design's raised button (elevated colours, lifted) instead of `variant`.
    pub elevated: bool,
    /// A light band sweeps across it (busy without a spinner).
    pub shimmer: bool,
    /// Which corners a cropped button cuts (`None` = the design's own).
    pub corners: Option<dodai_core::Corners>,
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

/// A filter / assist / input chip.
#[derive(Debug, Clone, PartialEq)]
pub struct Chip {
    /// Text.
    pub label: String,
    /// Selected state.
    pub bind: Bind<bool>,
    /// Leading icon.
    pub icon: Option<IconId>,
    /// Raised (the design's elevated chip).
    pub elevated: bool,
    /// A filter chip: a check mark leads it while selected.
    pub filter: bool,
    /// An input chip: a remove mark trails it.
    pub removable: bool,
}

/// How a table column lines its cells up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    /// Start.
    #[default]
    Left,
    /// Centre.
    Center,
    /// End (numbers).
    Right,
}

/// A table column.
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    /// Heading.
    pub title: String,
    /// Fixed width px (`None` shares the row).
    pub width: Option<f32>,
    /// Cell alignment.
    pub align: Align,
}

/// A data table.
#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// Columns.
    pub columns: Vec<Column>,
    /// Rows of cells (already filtered).
    pub rows: Vec<Vec<String>>,
    /// Scrolls inside this height px when set.
    pub height: Option<f32>,
}

/// A run of text inside [`Node::Rich`].
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    /// The text.
    pub text: String,
    /// Bold.
    pub bold: bool,
    /// Italic.
    pub italic: bool,
    /// Inline code (primary on the raised surface).
    pub code: bool,
    /// A link (link colour, underlined).
    pub link: bool,
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
        /// The line under the name (hex and contrast).
        detail: String,
    },
    /// A box in the design's panel style (shape, fill, border) with a centred label.
    Sample {
        /// Width px.
        w: f32,
        /// Height px.
        h: f32,
        /// Text inside.
        label: String,
    },
    /// A framed picture (or a placeholder) with an optional caption.
    Image {
        /// Picture file (`None` = the "image unavailable" placeholder).
        src: Option<String>,
        /// A play badge over it (a video poster).
        video: bool,
        /// Fit inside the frame instead of covering it.
        contain: bool,
        /// Width px.
        w: f32,
        /// Height px.
        h: f32,
        /// Caption.
        caption: Option<String>,
        /// Caption laid over the picture on the design's fade instead of below it.
        on_image: bool,
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
        /// The menu arrow shows its open state.
        open: bool,
        /// Which end the design crops.
        crop: Option<dodai_core::Crop>,
    },
    /// A floating action button with the add icon; `label` makes it extended.
    Fab {
        /// Label of the extended form.
        label: Option<String>,
        /// Which end the design crops.
        crop: Option<dodai_core::Crop>,
    },
    /// A bar of icon actions.
    Toolbar {
        /// Lifted and hugging its content.
        floating: bool,
        /// Primary container colours.
        vibrant: bool,
        /// The actions.
        icons: Vec<IconId>,
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
    /// A chip.
    Chip(Chip),
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
    /// Coloured polylines in a fixed box (easing curves, physics lanes).
    Plot(crate::plot::Plot),
    /// A dropdown: a closed button showing the pick, a list when open. A pick arrives as
    /// `Action::Set(key, Value::Index(i))`.
    Select {
        /// Value key.
        key: String,
        /// Options.
        labels: Vec<String>,
        /// Current pick (`None` shows the placeholder).
        selected: Option<usize>,
        /// Shown while nothing is picked.
        placeholder: String,
    },
    /// A live shader picture (catalogue ids, so the model needs no GPU code).
    Shader(ShaderSpec),
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
    /// A list whose rows are picked one at a time.
    Pick {
        /// The picked row.
        bind: Bind<usize>,
        /// Row titles.
        items: Vec<String>,
    },
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
        /// A text action at the end.
        action: Option<String>,
        /// A close mark at the end.
        dismissible: bool,
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
    Table(Table),
    /// A round avatar with initials.
    Avatar(String),
    /// A tooltip shown on its own beside the trigger.
    Tooltip {
        /// Tooltip text (the body of a rich one).
        text: String,
        /// Rich variant (title, body, action).
        rich: bool,
        /// Title of a rich tooltip.
        title: Option<String>,
        /// Action of a rich tooltip.
        action: Option<String>,
    },
    /// A syntax-highlighted code listing.
    Code {
        /// The code.
        text: String,
        /// Language tag (`rust`, `js`, …; empty = plain).
        lang: String,
    },
    /// Text runs with inline styles, wrapped as one paragraph.
    Rich {
        /// The runs.
        spans: Vec<Span>,
        /// Base style.
        role: TextRole,
    },
    /// A block quote: a primary bar at the start, muted text.
    Quote(Box<Node>),
    /// Overlapping round avatars, the overflow as a count.
    AvatarGroup {
        /// Initials.
        names: Vec<String>,
        /// Avatars shown before the `+n` (`None` = all).
        max: Option<usize>,
        /// Diameter px.
        size: f32,
    },
    /// Children laid over each other, centred on the first (a ring and its label).
    Overlay(Vec<Node>),
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
        /// Glyph above the title.
        icon: String,
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
            width: None,
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
            width: None,
            items,
        })
    }

    /// A dropdown over `labels`.
    pub fn select(
        key: &str,
        labels: Vec<String>,
        selected: Option<usize>,
        placeholder: &str,
    ) -> Node {
        Node::Select {
            key: key.into(),
            labels,
            selected,
            placeholder: placeholder.into(),
        }
    }

    /// A column of fixed width, small gaps: one tile of a gallery.
    pub fn tile(width: f32, items: Vec<Node>) -> Node {
        Node::Stack(Stack {
            axis: Axis::Column,
            wrap: false,
            gap: Gap::Xs,
            centre: false,
            width: Some(width),
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

    /// A display-font heading of `level` (1 = largest).
    pub fn heading(s: impl Into<String>, level: u8) -> Node {
        Node::Text(s.into(), TextRole::Heading(level))
    }

    /// Text at `size` px in `font` and `ink`.
    pub fn styled(s: impl Into<String>, size: f32, font: Font, ink: Ink) -> Node {
        Node::Text(
            s.into(),
            TextRole::Styled(TextStyle {
                size,
                font,
                ink,
                caps: false,
            }),
        )
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
            crop: None,
            plain: false,
            elevated: false,
            shimmer: false,
            corners: None,
        }
    }

    impl Button {
        /// Size step.
        pub fn size(mut self, size: ControlSize) -> Self {
            self.size = size;
            self
        }
        /// Crop this end the way the design crops buttons.
        pub fn crop(mut self, end: dodai_core::Crop) -> Self {
            self.crop = Some(end);
            self
        }
        /// Uncropped even on a design that crops its buttons by default.
        pub fn plain(mut self) -> Self {
            self.plain = true;
            self
        }
        /// The design's raised button.
        pub fn elevated(mut self) -> Self {
            self.elevated = true;
            self
        }
        /// A light band sweeps across it.
        pub fn shimmer(mut self) -> Self {
            self.shimmer = true;
            self
        }
        /// Cut these corners when cropped.
        pub fn corners(mut self, c: dodai_core::Corners) -> Self {
            self.corners = Some(c);
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

    /// A chip under `key`, selected when `on`.
    pub fn chip(label: impl Into<String>, key: &str, on: bool) -> Chip {
        Chip {
            label: label.into(),
            bind: Bind::new(key, on),
            icon: None,
            elevated: false,
            filter: false,
            removable: false,
        }
    }

    impl Chip {
        /// Leading icon.
        pub fn icon(mut self, i: IconId) -> Self {
            self.icon = Some(i);
            self
        }
        /// Raised.
        pub fn elevated(mut self) -> Self {
            self.elevated = true;
            self
        }
        /// A filter chip.
        pub fn filter(mut self) -> Self {
            self.filter = true;
            self
        }
        /// An input chip with a remove mark.
        pub fn removable(mut self) -> Self {
            self.removable = true;
            self
        }
        /// As a node.
        pub fn node(self) -> Node {
            Node::Chip(self)
        }
    }

    /// A message bar.
    pub fn alert(tone: Tone, text: impl Into<String>, banner: bool) -> Node {
        Node::Alert {
            tone,
            text: text.into(),
            banner,
            action: None,
            dismissible: false,
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
    /// A stack with `gap` between its children (other nodes are returned as they are).
    pub fn gap(mut self, gap: Gap) -> Self {
        if let Node::Stack(s) = &mut self {
            s.gap = gap;
        }
        self
    }

    /// A stack `w` px wide.
    pub fn width(mut self, w: f32) -> Self {
        if let Node::Stack(s) = &mut self {
            s.width = Some(w);
        }
        self
    }

    /// A stack whose children start at the top / left instead of centring across the axis.
    pub fn start(mut self) -> Self {
        if let Node::Stack(s) = &mut self {
            s.centre = false;
        }
        self
    }

    /// Visits this node and every descendant, parents first.
    pub fn walk(&self, f: &mut impl FnMut(&Node)) {
        f(self);
        match self {
            Node::Stack(s) => s.items.iter().for_each(|n| n.walk(f)),
            Node::Card { body, .. } | Node::Scroll { body, .. } => body.walk(f),
            Node::Popup(body) | Node::Quote(body) => body.walk(f),
            Node::Overlay(items) => items.iter().for_each(|n| n.walk(f)),
            _ => {}
        }
    }
}
