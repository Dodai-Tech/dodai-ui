//! Resolved styles of the small token-styled boxes every toolkit draws the same way: cards,
//! alerts, chips, avatars, code, tooltips, rows, steps, skeletons, popups and the closed dropdown.

use crate::{CardVariant, Color, ItemState, StepState, SurfaceTokens, Theme, Tone};

/// What a token-styled box is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kit {
    /// A card.
    Card(CardVariant),
    /// A message bar.
    Alert {
        /// Status tone.
        tone: Tone,
        /// Solid full-width banner.
        banner: bool,
    },
    /// A snackbar.
    Snackbar,
    /// A filter chip.
    Chip {
        /// Selected.
        selected: bool,
    },
    /// A round avatar.
    Avatar,
    /// A code listing.
    Code,
    /// A tooltip bubble.
    Tooltip {
        /// Title-and-body variant.
        rich: bool,
    },
    /// A list, menu or tree row.
    Row {
        /// Selected.
        selected: bool,
    },
    /// A number bubble of a stepper.
    StepDot(StepState),
    /// A small status dot (timeline marker, badge dot).
    Dot(Option<Tone>),
    /// A count badge.
    Count(Tone),
    /// A pagination item.
    Page {
        /// The current page.
        selected: bool,
    },
    /// A loading placeholder.
    Skeleton,
    /// A floating menu or dropdown list.
    Popup,
    /// The closed dropdown button.
    Select,
    /// Header row of a table.
    TableHead,
    /// Body row of a table (`true` = banded).
    TableRow(bool),
    /// A bar of actions: docked, or a floating (lifted, pill on tonal designs) bar; `vibrant`
    /// takes the primary container colours on tonal designs.
    Toolbar {
        /// Lifted and hugging its content.
        floating: bool,
        /// Primary container colours.
        vibrant: bool,
    },
    /// A floating action button (`extended` = with a label).
    Fab {
        /// Carries a label next to the icon.
        extended: bool,
    },
    /// A raised button: the design's elevated button colours, lifted.
    Elevated,
    /// The strip of a segmented control: one bordered box around its segments (a pill
    /// outlined in `outline` on tonal designs).
    Segmented,
    /// One option of a segmented control; the selected one is filled.
    Segment {
        /// The picked option.
        selected: bool,
    },
}

impl Kit {
    /// Which kind of surface the box is for the surface and glass (`None`: it only takes a
    /// component's own [`Surface`](crate::Surface)).
    pub fn surface_class(self) -> Option<crate::SurfaceClass> {
        use crate::SurfaceClass as C;
        match self {
            Self::Card(_) | Self::Code => Some(C::Card),
            Self::Popup | Self::Tooltip { .. } => Some(C::Panel),
            Self::Alert { .. } => Some(C::Card),
            Self::Toolbar { .. } | Self::Snackbar => Some(C::Bar),
            Self::Chip { .. } => Some(C::Tag),
            Self::Fab { .. } | Self::Elevated => Some(C::Button),
            Self::Select => Some(C::Input),
            _ => None,
        }
    }

    /// Whether the box floats over other components (popups, tooltips, toasts), so its glass
    /// is [`floating`](crate::Glass::floating).
    pub fn floats(self) -> bool {
        matches!(self, Self::Popup | Self::Tooltip { .. } | Self::Snackbar)
    }

    /// A stable kebab-case name, unique per variant (`card-elevated`, `alert-danger-banner`,
    /// `chip-on`): what stylesheets key on.
    pub fn name(self) -> String {
        let tone = |t: Tone| match t {
            Tone::Info => "info",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Danger => "danger",
        };
        let on = |b: bool| if b { "on" } else { "off" };
        match self {
            Kit::Card(CardVariant::Elevated) => "card-elevated".into(),
            Kit::Card(CardVariant::Filled) => "card-filled".into(),
            Kit::Card(CardVariant::Outlined) => "card-outlined".into(),
            Kit::Alert { tone: t, banner } => {
                format!("alert-{}{}", tone(t), if banner { "-banner" } else { "" })
            }
            Kit::Snackbar => "snackbar".into(),
            Kit::Chip { selected } => format!("chip-{}", on(selected)),
            Kit::Avatar => "avatar".into(),
            Kit::Code => "code".into(),
            Kit::Tooltip { rich } => if rich { "tooltip-rich" } else { "tooltip" }.into(),
            Kit::Row { selected } => format!("row-{}", on(selected)),
            Kit::StepDot(StepState::Done) => "step-done".into(),
            Kit::StepDot(StepState::Current) => "step-current".into(),
            Kit::StepDot(StepState::Todo) => "step-todo".into(),
            Kit::Dot(None) => "dot".into(),
            Kit::Dot(Some(t)) => format!("dot-{}", tone(t)),
            Kit::Count(t) => format!("count-{}", tone(t)),
            Kit::Page { selected } => format!("page-{}", on(selected)),
            Kit::Skeleton => "skeleton".into(),
            Kit::Popup => "popup".into(),
            Kit::Select => "select".into(),
            Kit::TableHead => "table-head".into(),
            Kit::TableRow(banded) => if banded {
                "table-row-band"
            } else {
                "table-row"
            }
            .into(),
            Kit::Toolbar { floating, vibrant } => format!(
                "toolbar{}{}",
                if floating { "-floating" } else { "" },
                if vibrant { "-vibrant" } else { "" }
            ),
            Kit::Fab { extended } => if extended { "fab-extended" } else { "fab" }.into(),
            Kit::Elevated => "elevated".into(),
            Kit::Segmented => "segmented".into(),
            Kit::Segment { selected } => format!("segment-{}", on(selected)),
        }
    }

    /// Every variant, for stylesheets that need them all.
    pub fn all() -> Vec<Kit> {
        let tones = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];
        let mut v = vec![
            Kit::Card(CardVariant::Elevated),
            Kit::Card(CardVariant::Filled),
            Kit::Card(CardVariant::Outlined),
            Kit::Snackbar,
            Kit::Avatar,
            Kit::Code,
            Kit::Tooltip { rich: false },
            Kit::Tooltip { rich: true },
            Kit::StepDot(StepState::Done),
            Kit::StepDot(StepState::Current),
            Kit::StepDot(StepState::Todo),
            Kit::Dot(None),
            Kit::Skeleton,
            Kit::Popup,
            Kit::Select,
            Kit::TableHead,
            Kit::TableRow(false),
            Kit::TableRow(true),
            Kit::Fab { extended: false },
            Kit::Fab { extended: true },
            Kit::Elevated,
            Kit::Segmented,
        ];
        for floating in [false, true] {
            for vibrant in [false, true] {
                v.push(Kit::Toolbar { floating, vibrant });
            }
        }
        for selected in [false, true] {
            v.extend([
                Kit::Chip { selected },
                Kit::Row { selected },
                Kit::Page { selected },
                Kit::Segment { selected },
            ]);
        }
        for t in tones {
            v.extend([
                Kit::Alert {
                    tone: t,
                    banner: false,
                },
                Kit::Alert {
                    tone: t,
                    banner: true,
                },
                Kit::Dot(Some(t)),
                Kit::Count(t),
            ]);
        }
        v
    }
}

/// How a [`Kit`] looks under a theme: pure data, no ECS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitStyle {
    /// Fill (`None` = transparent).
    pub fill: Option<Color>,
    /// Outline.
    pub border: Option<Color>,
    /// Text colour inside.
    pub fg: Color,
    /// Glow halo.
    pub glow: bool,
    /// Fully round on rounded designs.
    pub pill: bool,
    /// Horizontal and vertical padding px.
    pub padding: (f32, f32),
    /// Lifted off the page (the design's elevation shadow or hard shadow).
    pub lift: bool,
}

impl Theme {
    /// The style of `kit` on this theme; `hovered` is the pointer state of clickable kits.
    pub fn kit(&self, kit: Kit, hovered: bool) -> KitStyle {
        let theme = self;
        let p = &theme.palette;
        let sp = &theme.spacing;
        let from = |t: SurfaceTokens, padding: (f32, f32), pill: bool| KitStyle {
            fill: t.background,
            border: t.border,
            fg: t.foreground,
            glow: t.glow,
            pill,
            padding,
            lift: false,
        };
        let state = if hovered {
            crate::ControlState::Hovered
        } else {
            crate::ControlState::Idle
        };
        let button = |b: crate::ButtonTokens, padding: (f32, f32)| KitStyle {
            fill: b.background,
            border: b.border,
            fg: b.foreground,
            glow: false,
            pill: true,
            padding,
            lift: true,
        };
        match kit {
            Kit::Card(v) => KitStyle {
                lift: v == CardVariant::Elevated,
                ..from(theme.card(v), (sp.md, sp.md), false)
            },
            Kit::Alert { tone, banner } => from(theme.alert(tone, banner), (sp.md, sp.sm), false),
            Kit::Snackbar => from(theme.snackbar().0, (sp.md, sp.sm), false),
            Kit::Chip { selected } => from(
                theme.chip(ItemState::of(selected, hovered)),
                (sp.md, sp.xs),
                true,
            ),
            Kit::Avatar => KitStyle {
                fill: Some(p.series[0]),
                border: None,
                fg: theme.on(p.series[0]),
                glow: false,
                pill: true,
                padding: (0.0, 0.0),
                lift: false,
            },
            Kit::Code => KitStyle {
                fill: Some(p.roles.surface_container_highest),
                border: Some(p.border),
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.md),
                lift: false,
            },
            Kit::Tooltip { rich } => {
                let (bg, fg) = theme.tooltip(rich);
                KitStyle {
                    fill: Some(bg),
                    border: Some(p.border),
                    fg,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.xs + 2.0),
                    lift: false,
                }
            }
            Kit::Row { selected } => {
                let t = theme.list_item(ItemState::of(selected, hovered));
                KitStyle {
                    fill: t.background,
                    border: None,
                    fg: t.foreground,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.sm),
                    lift: false,
                }
            }
            Kit::StepDot(state) => {
                let t = theme.step(state);
                KitStyle {
                    fill: t.fill,
                    border: Some(t.stroke),
                    fg: t.mark,
                    glow: t.glow,
                    pill: true,
                    padding: (0.0, 0.0),
                    lift: false,
                }
            }
            Kit::Dot(tone) => {
                let c = tone.map_or(p.text_muted, |t| p.tone(t));
                KitStyle {
                    fill: Some(c),
                    border: None,
                    fg: theme.on(c),
                    glow: false,
                    pill: true,
                    padding: (0.0, 0.0),
                    lift: false,
                }
            }
            Kit::Count(tone) => from(theme.badge(tone), (sp.xs + 2.0, 0.0), true),
            Kit::Page { selected } => {
                let t = theme.chip(ItemState::of(selected, hovered));
                KitStyle {
                    fill: t.background,
                    border: if selected { t.border } else { None },
                    fg: t.foreground,
                    glow: false,
                    pill: true,
                    padding: (sp.sm, 0.0),
                    lift: false,
                }
            }
            Kit::Popup => KitStyle {
                lift: true,
                ..from(theme.popup(), (sp.xs, sp.xs), false)
            },
            Kit::Toolbar { floating, vibrant } => {
                let tonal = theme.look().tonal;
                let panel = theme.panel(false);
                let (fill, border, fg) = match (tonal, vibrant) {
                    (true, true) => (
                        p.roles.primary_container,
                        None,
                        p.roles.on_primary_container,
                    ),
                    (true, false) => (p.roles.surface_container, None, p.text),
                    (false, _) => (panel.background, Some(panel.border), p.text),
                };
                KitStyle {
                    fill: Some(fill),
                    border,
                    fg,
                    glow: false,
                    pill: tonal && floating,
                    padding: (sp.sm, sp.xs),
                    lift: tonal && floating,
                }
            }
            Kit::Fab { extended } => button(
                theme.fab(crate::ButtonVariant::Primary, state),
                (if extended { sp.lg } else { sp.md }, sp.md),
            ),
            Kit::Elevated => {
                // As tall as a medium button: the label line plus even padding.
                let m = theme.control_metrics(crate::ControlSize::Medium);
                let py = ((m.height - m.text_size * 1.3) / 2.0).max(0.0);
                button(theme.elevated_button(state), (m.padding, py))
            }
            Kit::Select => {
                let i = theme.input(hovered);
                KitStyle {
                    fill: Some(i.background),
                    border: Some(i.border),
                    fg: i.text,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.sm),
                    lift: false,
                }
            }
            Kit::Segmented => {
                let tonal = theme.look().tonal;
                KitStyle {
                    fill: (!tonal).then_some(p.background),
                    border: Some(if tonal {
                        p.roles.outline
                    } else {
                        p.border_strong
                    }),
                    fg: p.text,
                    glow: false,
                    pill: tonal,
                    padding: if tonal {
                        (0.0, 0.0)
                    } else {
                        (sp.half(), sp.half())
                    },
                    lift: false,
                }
            }
            Kit::Segment { selected } => {
                let tonal = theme.look().tonal;
                let (fill, fg) = match (tonal, selected) {
                    (true, true) => (
                        Some(p.roles.secondary_container),
                        p.roles.on_secondary_container,
                    ),
                    (true, false) => (
                        hovered.then(|| p.roles.on_surface.with_alpha(0.08)),
                        p.roles.on_surface,
                    ),
                    (false, true) => (Some(p.primary), p.on_primary),
                    (false, false) => (
                        hovered.then_some(p.surface_raised),
                        if hovered { p.text } else { p.text_muted },
                    ),
                };
                KitStyle {
                    fill,
                    border: None,
                    fg,
                    glow: selected && !tonal,
                    pill: tonal,
                    padding: (if tonal { sp.sm * 1.5 } else { sp.md }, 0.0),
                    lift: false,
                }
            }
            Kit::Skeleton => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (0.0, 0.0),
                lift: false,
            },
            Kit::TableHead => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
                lift: false,
            },
            Kit::TableRow(banded) => KitStyle {
                fill: banded.then_some(p.roles.surface_container_low),
                border: None,
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
                lift: false,
            },
        }
    }
}

/// Motion of the busy indicators, shared by every toolkit so they all turn at the same rate.
pub mod busy {
    use crate::SpinnerStyle;

    /// Revolutions per second of the ring.
    pub const SPIN_RATE: f32 = 0.9;
    /// Width of the sliding segment as a fraction of the track.
    pub const SEGMENT: f32 = 0.3;

    /// Angle (radians) of the ring after `t` seconds.
    pub fn spin_angle(t: f32) -> f32 {
        (t * SPIN_RATE).fract() * std::f32::consts::TAU
    }

    /// Bars or dots in a [`SpinnerStyle::Bars`] / [`SpinnerStyle::Orbit`] spinner.
    pub const TRAIL: usize = 8;
    /// Opacity of a cell the head has not reached for a while.
    pub const DIM: f32 = 0.15;

    /// One bar, dot or square of a spinner, in a `size`×`size` box.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Cell {
        /// Left px.
        pub x: f32,
        /// Top px.
        pub y: f32,
        /// Width px.
        pub w: f32,
        /// Height px (a bar's full height).
        pub h: f32,
        /// A dot instead of a rectangle.
        pub round: bool,
        /// Corner radius px of a rectangle (zero for hard corners).
        pub radius: f32,
        /// Fraction of the cycle at which the bright head reaches this cell.
        pub lag: f32,
    }

    /// The cells of a spinner `size` px across; none for the ring, which is an arc.
    pub fn cells(style: SpinnerStyle, size: f32) -> Vec<Cell> {
        match style {
            SpinnerStyle::Ring => vec![],
            SpinnerStyle::Bars => {
                let pitch = size / TRAIL as f32;
                let w = pitch * 0.58;
                (0..TRAIL)
                    .map(|i| Cell {
                        x: i as f32 * pitch + (pitch - w) / 2.0,
                        y: 0.0,
                        w,
                        h: size,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 / TRAIL as f32,
                    })
                    .collect()
            }
            SpinnerStyle::Orbit => {
                let d = size * 0.18;
                (0..TRAIL)
                    .map(|i| {
                        let (cx, cy) = orbit_xy(i, TRAIL, size / 2.0, size / 2.0 - d / 2.0);
                        Cell {
                            x: cx - d / 2.0,
                            y: cy - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: i as f32 / TRAIL as f32,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Steps => {
                let side = size * 0.42;
                let far = size - side;
                [(0.0, 0.0), (far, 0.0), (far, far), (0.0, far)]
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y))| Cell {
                        x,
                        y,
                        w: side,
                        h: side,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 / 4.0,
                    })
                    .collect()
            }
            SpinnerStyle::Scanner => {
                let n = 7;
                let pitch = size / n as f32;
                let w = pitch * 0.5;
                (0..n)
                    .map(|i| Cell {
                        x: i as f32 * pitch + (pitch - w) / 2.0,
                        y: size * 0.1,
                        w,
                        h: size * 0.8,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 / n as f32 * 0.5,
                    })
                    .collect()
            }
            SpinnerStyle::Hop => {
                let side = size * 0.26;
                (0..3)
                    .map(|i| Cell {
                        x: i as f32 * (size - side) / 2.0,
                        y: size - side,
                        w: side,
                        h: side,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 * 0.12,
                    })
                    .collect()
            }
            SpinnerStyle::Petals => {
                let d = size * 0.34;
                (0..6)
                    .map(|i| {
                        let (cx, cy) = orbit_xy(i, 6, size / 2.0, size / 2.0 - d / 2.0);
                        Cell {
                            x: cx - d / 2.0,
                            y: cy - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: i as f32 / 6.0,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Quad => (0..4)
                .map(|i| {
                    let side = size * (0.32 - 0.02 * i as f32);
                    Cell {
                        x: 0.0,
                        y: 0.0,
                        w: side,
                        h: side,
                        round: false,
                        radius: side * 0.26,
                        lag: i as f32 / 4.0,
                    }
                })
                .collect(),
            SpinnerStyle::Glitch => {
                let h = size * 0.14;
                (0..4)
                    .map(|i| Cell {
                        x: 0.0,
                        y: i as f32 * (size - h) / 3.0,
                        w: size,
                        h,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 * 0.19,
                    })
                    .collect()
            }
            SpinnerStyle::Shuffle => {
                let side = size * 0.44;
                let far = size - side;
                [(0.0, 0.0), (far, 0.0), (far, far)]
                    .into_iter()
                    .enumerate()
                    .map(|(i, (x, y))| Cell {
                        x,
                        y,
                        w: side,
                        h: side,
                        round: false,
                        radius: 0.0,
                        // The piece number, not a phase: the pieces follow their own tracks.
                        lag: i as f32 / 3.0,
                    })
                    .collect()
            }
            SpinnerStyle::Sweep => {
                // Dots along the rounded square the arc follows (where a real stroke is not
                // available, e.g. bevy's UI).
                let d = size * 0.075;
                (0..SWEEP_DOTS)
                    .map(|i| {
                        let f = i as f32 / SWEEP_DOTS as f32;
                        let (px, py) = ring_point(SpinnerStyle::Sweep, f, size, d);
                        Cell {
                            x: px - d / 2.0,
                            y: py - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: f,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Classic => {
                let d = size * 0.1;
                let r = (size - d) / 2.0;
                (0..CLASSIC_DOTS)
                    .map(|i| {
                        let f = i as f32 / CLASSIC_DOTS as f32;
                        let a = f * std::f32::consts::TAU;
                        Cell {
                            x: size / 2.0 + r * a.sin() - d / 2.0,
                            y: size / 2.0 - r * a.cos() - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: f,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Arc => {
                let d = size * 0.13;
                let r = (size - d) / 2.0;
                (0..ARC_DOTS)
                    .map(|i| {
                        let a = i as f32 / ARC_DOTS as f32 * std::f32::consts::TAU;
                        Cell {
                            x: size / 2.0 + r * a.sin() - d / 2.0,
                            y: size / 2.0 - r * a.cos() - d / 2.0,
                            w: d,
                            h: d,
                            round: true,
                            radius: d / 2.0,
                            lag: i as f32 / ARC_DOTS as f32,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Reticle => {
                let (len, th) = (size * 0.4, size * 0.13);
                let far = size - len;
                let farh = size - th;
                (0..8)
                    .map(|i| {
                        let corner = i / 2;
                        let (cx, cy) = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)][corner];
                        let horizontal = i % 2 == 0;
                        let (w, h) = if horizontal { (len, th) } else { (th, len) };
                        Cell {
                            x: if horizontal { cx * far } else { cx * farh },
                            y: if horizontal { cy * farh } else { cy * far },
                            w,
                            h,
                            round: false,
                            radius: 0.0,
                            lag: corner as f32 / 4.0,
                        }
                    })
                    .collect()
            }
            SpinnerStyle::Data => {
                let w = size * 0.13;
                let h = size * 0.3;
                [0.0, 0.37, 0.71, 0.2, 0.55]
                    .into_iter()
                    .enumerate()
                    .map(|(i, lag)| Cell {
                        x: i as f32 * (size - w) / 4.0,
                        y: 0.0,
                        w,
                        h,
                        round: false,
                        radius: 0.0,
                        lag,
                    })
                    .collect()
            }
            SpinnerStyle::Pillars => {
                let w = size * 0.27;
                (0..3)
                    .map(|i| Cell {
                        x: i as f32 * (size - w) / 2.0,
                        y: 0.0,
                        w,
                        h: size,
                        round: false,
                        radius: 0.0,
                        lag: i as f32 * 0.25,
                    })
                    .collect()
            }
            SpinnerStyle::Courses => {
                let h = size * 0.2;
                (0..3)
                    .map(|i| Cell {
                        x: 0.0,
                        y: i as f32 * (size - h) / 2.0,
                        w: size,
                        h,
                        round: false,
                        radius: h * 0.35,
                        lag: i as f32 * 0.14,
                    })
                    .collect()
            }
            SpinnerStyle::Bounce => {
                let d = size * 0.26;
                (0..3)
                    .map(|i| Cell {
                        x: i as f32 * (size - d) / 2.0,
                        y: size - d,
                        w: d,
                        h: d,
                        round: true,
                        radius: d / 2.0,
                        lag: i as f32 * 0.12,
                    })
                    .collect()
            }
            SpinnerStyle::Grid => {
                let side = size * 0.28;
                let gap = (size - 3.0 * side) / 2.0;
                (0..9)
                    .map(|i| {
                        let (col, row) = ((i % 3) as f32, (i / 3) as f32);
                        Cell {
                            x: col * (side + gap),
                            y: row * (side + gap),
                            w: side,
                            h: side,
                            round: false,
                            radius: side * 0.3,
                            lag: (col + row) / 5.0,
                        }
                    })
                    .collect()
            }
        }
    }

    /// Opacity of a cell with `lag` after `t` seconds: a bright head chasing a fading tail, or
    /// for [`SpinnerStyle::Steps`] a hard on/off with no fade.
    pub fn cell_alpha(style: SpinnerStyle, lag: f32, t: f32) -> f32 {
        let since = ((t * SPIN_RATE).fract() - lag).rem_euclid(1.0);
        match style {
            SpinnerStyle::Grid | SpinnerStyle::Petals => 1.0 - (since * 2.0).min(1.0) * (1.0 - DIM),
            SpinnerStyle::Scanner => {
                let lead = (1.0 - (1.0 - since) / 0.12).max(0.0);
                let trail = 1.0 - (since / 0.55).min(1.0) * (1.0 - DIM);
                if since > 0.88 {
                    lead.max(DIM)
                } else {
                    trail
                }
            }
            SpinnerStyle::Hop => 0.8 + 0.2 * hop(since),
            SpinnerStyle::Steps => {
                if since < 0.25 {
                    1.0
                } else {
                    DIM
                }
            }
            _ => 1.0 - since * (1.0 - DIM),
        }
    }

    /// Height of a hop (0..1) of a [`SpinnerStyle::Hop`] square `since` the head reached it.
    pub fn hop(since: f32) -> f32 {
        if since < 0.4 {
            (since / 0.4 * std::f32::consts::PI).sin()
        } else {
            0.0
        }
    }

    /// Where a cell is drawn at opacity `alpha`: a bar of [`SpinnerStyle::Bars`] shrinks towards
    /// its foot, a square of [`SpinnerStyle::Grid`] towards its centre, the rest stay put.
    /// Returns `(x, y, w, h)` px.
    pub fn cell_rect(style: SpinnerStyle, c: &Cell, alpha: f32) -> (f32, f32, f32, f32) {
        match style {
            SpinnerStyle::Bars => {
                let h = c.h * bar_scale(alpha);
                (c.x, c.y + c.h - h, c.w, h)
            }
            SpinnerStyle::Scanner => {
                let h = c.h * (0.4 + 0.6 * alpha);
                (c.x, c.y + (c.h - h) / 2.0, c.w, h)
            }
            SpinnerStyle::Hop => {
                let up = ((alpha - 0.8) / 0.2).clamp(0.0, 1.0) * c.h * 2.0;
                (c.x, c.y - up, c.w, c.h)
            }
            SpinnerStyle::Grid | SpinnerStyle::Petals => {
                let k = 0.35 + 0.65 * alpha;
                let (w, h) = (c.w * k, c.h * k);
                (c.x + (c.w - w) / 2.0, c.y + (c.h - h) / 2.0, w, h)
            }
            _ => (c.x, c.y, c.w, c.h),
        }
    }

    /// Sideways jump (fraction of the size) and opacity of a [`SpinnerStyle::Glitch`] slab in
    /// each of the eight steps of its cycle.
    pub const GLITCH: [(f32, f32, f32); 8] = [
        (0.0, 1.0, 1.0),
        (0.30, 0.45, 0.7),
        (0.0, 1.0, 1.0),
        (-0.25, 1.0, 0.6),
        (0.0, 0.4, 1.0),
        (0.40, 1.0, 0.8),
        (-0.15, 0.6, 1.0),
        (0.0, 1.0, 1.0),
    ];

    /// How much of an arc running `len` round from `from` covers a point `at` (0..1) round
    /// from it, with soft ends `soft` wide so the arc grows and shrinks smoothly.
    pub fn arc_cover(at: f32, len: f32, soft: f32) -> f32 {
        let start = if at > 1.0 - soft { at - 1.0 } else { at };
        let rise = ((start + soft) / soft).clamp(0.0, 1.0);
        let fall = ((len - at) / soft).clamp(0.0, 1.0);
        rise.min(fall)
    }

    /// Dots on a [`SpinnerStyle::Sweep`] ring.
    pub const SWEEP_DOTS: usize = 72;

    /// Dots on a [`SpinnerStyle::Classic`] circle: close enough to read as one line.
    pub const CLASSIC_DOTS: usize = 80;

    /// Dots on a [`SpinnerStyle::Arc`] circle.
    pub const ARC_DOTS: usize = 20;

    /// Height of a [`SpinnerStyle::Pillars`] bar (fraction of full) in each quarter of its cycle.
    pub const PILLARS: [f32; 4] = [0.3, 0.65, 1.0, 0.65];

    /// Slot (0 top-left, 1 top-right, 2 bottom-right, 3 bottom-left) of the piece `piece` of a
    /// [`SpinnerStyle::Shuffle`] spinner, as a position along the ring, after a cycle fraction
    /// `u` (0..1). Every quarter one piece slides into the empty slot; at the end the same slots
    /// are filled again, so the loop shows no jump.
    pub fn shuffle_slot(piece: usize, u: f32) -> f32 {
        // (quarter in which it moves, from, to)
        let moves: &[(usize, f32, f32)] = match piece {
            0 => &[(2, 0.0, 1.0)],
            1 => &[(1, 1.0, 2.0)],
            _ => &[(0, 2.0, 3.0), (3, 3.0, 4.0)],
        };
        let q = ((u * 4.0) as usize).min(3);
        let f = (u * 4.0 - q as f32) / 0.6;
        let f = 1.0 - (1.0 - f.clamp(0.0, 1.0)).powi(3);
        let mut slot = match piece {
            0 => 0.0,
            1 => 1.0,
            _ => 2.0,
        };
        for &(mq, from, to) in moves {
            if q > mq {
                slot = to;
            } else if q == mq {
                slot = from + (to - from) * f;
            }
        }
        slot
    }

    /// Point on the square ring of a [`SpinnerStyle::Quad`] / [`SpinnerStyle::Shuffle`] cell
    /// (`far` px between the corners) at ring position `s` (0 top-left, clockwise, wraps at 4).
    pub fn ring_xy(s: f32, far: f32) -> (f32, f32) {
        let s = s.rem_euclid(4.0);
        let k = s as usize % 4;
        let f = s - k as f32;
        let corner = |n: usize| match n % 4 {
            0 => (0.0, 0.0),
            1 => (far, 0.0),
            2 => (far, far),
            _ => (0.0, far),
        };
        let (a, b) = (corner(k), corner(k + 1));
        (a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f)
    }

    /// Where and how a cell is drawn after `t` seconds: `(x, y, w, h, alpha)`. The one call a
    /// toolkit needs, whatever the style.
    /// A wavy line `w` wide centred on `y = 0`: `(x, y)` points `step` px apart.
    pub fn wave_line(w: f32, amp: f32, wavelength: f32, step: f32) -> Vec<(f32, f32)> {
        let n = (w / step.max(0.5)).ceil().max(1.0) as usize;
        (0..=n)
            .map(|i| {
                let x = (i as f32 * step).min(w);
                (x, amp * (x / wavelength * std::f32::consts::TAU).sin())
            })
            .collect()
    }

    /// A wavy ring in a `size` square, clockwise from the top: radius `r + amp * sin(lobes * a)`.
    pub fn wave_ring(size: f32, amp: f32, lobes: u32, segments: usize) -> Vec<(f32, f32)> {
        let r = size / 2.0 - amp - 1.0;
        (0..=segments)
            .map(|i| {
                let a = i as f32 / segments as f32 * std::f32::consts::TAU;
                let rr = r + amp * (a * lobes as f32).sin();
                (size / 2.0 + rr * a.sin(), size / 2.0 - rr * a.cos())
            })
            .collect()
    }

    /// The part of a progress path to draw, as `(start, end)` fractions of its length (at most two
    /// ranges): `0..frac` when determinate; a travelling window when `frac` is `None`
    /// (`wrap`: it wraps round a ring, otherwise it runs off the end).
    pub fn progress_window(frac: Option<f32>, t: f32, wrap: bool) -> [(f32, f32); 2] {
        const NONE: (f32, f32) = (0.0, 0.0);
        match frac {
            Some(f) => [(0.0, f.clamp(0.0, 1.0)), NONE],
            None => {
                let len = 0.3;
                if wrap {
                    let s = (t * 0.7).fract();
                    if s + len <= 1.0 {
                        [(s, s + len), NONE]
                    } else {
                        [(s, 1.0), (0.0, s + len - 1.0)]
                    }
                } else {
                    let head = (t * 0.7).fract() * (1.0 + len);
                    [((head - len).max(0.0), head.min(1.0)), NONE]
                }
            }
        }
    }

    /// The part of a polyline between fractions `a` and `b` of its point count.
    pub fn slice_points(pts: &[(f32, f32)], a: f32, b: f32) -> &[(f32, f32)] {
        let n = pts.len();
        let i0 = ((a * (n - 1) as f32).floor() as usize).min(n - 1);
        let i1 = ((b * (n - 1) as f32).ceil() as usize).min(n - 1);
        if i1 <= i0 {
            &[]
        } else {
            &pts[i0..=i1]
        }
    }

    /// Which token colour a [`BarPiece`] takes.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum BarRole {
        /// The track.
        Track,
        /// The fill.
        Fill,
        /// The border line.
        Line,
    }

    /// One filled shape of a styled progress bar.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct BarPiece {
        /// Four corners (clockwise); a rectangle is given as its four corners too.
        pub pts: [(f32, f32); 4],
        /// Corner radius (rectangles only; `0` for polygons and square rectangles).
        pub radius: f32,
        /// The colour it takes.
        pub role: BarRole,
        /// Opacity 0..=1.
        pub alpha: f32,
    }

    /// Height of a styled progress bar.
    pub fn bar_height(style: crate::ProgressStyle) -> f32 {
        match style {
            crate::ProgressStyle::Hud => 16.0,
            crate::ProgressStyle::Stripes => 22.0,
            crate::ProgressStyle::Stones => 18.0,
        }
    }

    /// The pieces of a styled progress bar `w` wide at time `t` (seconds): determinate at
    /// `frac`, or running when `None`. Back to front.
    pub fn bar_pieces(
        style: crate::ProgressStyle,
        frac: Option<f32>,
        t: f32,
        w: f32,
    ) -> Vec<BarPiece> {
        use crate::ProgressStyle as P;
        let h = bar_height(style);
        let rect =
            |x: f32, y: f32, rw: f32, rh: f32, radius: f32, role: BarRole, alpha: f32| BarPiece {
                pts: [(x, y), (x + rw, y), (x + rw, y + rh), (x, y + rh)],
                radius,
                role,
                alpha,
            };
        let mut v = Vec::new();
        match style {
            P::Hud => {
                let n = 26usize;
                let slant = h * 0.4;
                let slot = (w - slant) / n as f32;
                let lit = frac.map(|f| f.clamp(0.0, 1.0) * n as f32);
                let comet = (t * 0.9).fract() * (n as f32 + 8.0) - 4.0;
                for i in 0..n {
                    let x0 = i as f32 * slot;
                    let x1 = x0 + slot - 3.0;
                    let cell = |g: f32, role, alpha| BarPiece {
                        pts: [
                            (x0 + slant - g, -g),
                            (x1 + slant + g, -g),
                            (x1 + g, h + g),
                            (x0 - g, h + g),
                        ],
                        radius: 0.0,
                        role,
                        alpha,
                    };
                    let a = match lit {
                        Some(l) => (l - i as f32).clamp(0.0, 1.0),
                        None => {
                            let d = comet - i as f32;
                            if (0.0..8.0).contains(&d) {
                                1.0 - d / 8.0
                            } else {
                                0.0
                            }
                        }
                    };
                    v.push(cell(0.0, BarRole::Track, 0.55));
                    if a > 0.0 {
                        v.push(cell(0.0, BarRole::Fill, a));
                    }
                    let head = match lit {
                        Some(l) => {
                            (l - 0.001).floor() as i64 == i as i64 && l > 0.0 && l < n as f32
                        }
                        None => (0.0..1.0).contains(&(comet - i as f32)),
                    };
                    if head {
                        v.push(cell(
                            3.0,
                            BarRole::Fill,
                            0.28 * (0.55 + 0.45 * (t * 18.0).sin()),
                        ));
                    }
                }
            }
            P::Stripes => {
                let bw = 2.5;
                v.push(rect(0.0, 0.0, w, h, 0.0, BarRole::Line, 1.0));
                v.push(rect(
                    bw,
                    bw,
                    w - 2.0 * bw,
                    h - 2.0 * bw,
                    0.0,
                    BarRole::Track,
                    1.0,
                ));
                let (ix0, ix1, iy, ih) = (bw, w - bw, bw, h - 2.0 * bw);
                let active = frac.map_or(ix1, |f| ix0 + (ix1 - ix0) * f.clamp(0.0, 1.0));
                if active > ix0 {
                    v.push(rect(ix0, iy, active - ix0, ih, 0.0, BarRole::Fill, 0.45));
                    let sw = ih * 0.9;
                    let period = sw * 2.0;
                    let off = (t * 36.0) % period;
                    let mut x = ix0 - ih - period;
                    while x < active + period {
                        let sx = x + off;
                        let c = |u: f32| u.clamp(ix0, active);
                        v.push(BarPiece {
                            pts: [
                                (c(sx + ih), iy),
                                (c(sx + ih + sw), iy),
                                (c(sx + sw), iy + ih),
                                (c(sx), iy + ih),
                            ],
                            radius: 0.0,
                            role: BarRole::Fill,
                            alpha: 1.0,
                        });
                        x += period;
                    }
                    if frac.is_some() {
                        v.push(rect(active - 2.0, iy, 2.0, ih, 0.0, BarRole::Line, 1.0));
                    }
                }
            }
            P::Stones => {
                let n = 12usize;
                let slot = w / n as f32;
                let side = (slot - 5.0).min(h);
                let lit = frac.map(|f| f.clamp(0.0, 1.0) * n as f32);
                for i in 0..n {
                    let cx = slot * (i as f32 + 0.5);
                    let (mut s, a) = match lit {
                        Some(l) => (side, (l - i as f32).clamp(0.0, 1.0)),
                        None => (side, 0.2 + 0.8 * (t * 4.0 - i as f32 * 0.6).sin().max(0.0)),
                    };
                    if let Some(l) = lit {
                        if (l - 0.001).floor() as i64 == i as i64 && l > 0.0 && l < n as f32 {
                            s *= 0.86 + 0.14 * (t * 5.0).sin();
                        }
                    }
                    v.push(rect(
                        cx - side / 2.0,
                        (h - side) / 2.0,
                        side,
                        side,
                        5.0,
                        BarRole::Track,
                        1.0,
                    ));
                    if a > 0.0 {
                        v.push(rect(
                            cx - s / 2.0,
                            (h - s) / 2.0,
                            s,
                            s,
                            5.0,
                            BarRole::Fill,
                            a,
                        ));
                    }
                }
            }
        }
        v
    }

    /// For the arc spinners drawn as a real stroke: `(start, length, track alpha)` with the angles
    /// in turns clockwise from the top. `None` for every other style.
    pub fn arc_span(style: SpinnerStyle, t: f32) -> Option<(f32, f32, f32)> {
        let phase = (t * SPIN_RATE).fract();
        let wave = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
        match style {
            SpinnerStyle::Classic => Some((phase * 3.0, 0.06 + 0.69 * wave, 0.0)),
            SpinnerStyle::Sweep => {
                let len = 0.2 + 0.4 * wave;
                Some((((phase * 2.0).fract() - len).rem_euclid(1.0), len, 0.22))
            }
            _ => None,
        }
    }

    /// Stroke width of an arc spinner as a fraction of its size, and whether its ends are round.
    /// Sweep is a thin hairline ring with square ends; Classic a thicker one with round ends.
    pub fn arc_stroke(style: SpinnerStyle) -> (f32, bool) {
        match style {
            SpinnerStyle::Sweep => (0.06, false),
            _ => (0.1, true),
        }
    }

    /// A point `turn` (0..1, clockwise from the top) along the path an arc spinner follows, as
    /// x, y in a box of `size` with a stroke `w` wide: a circle, except Sweep, which runs round
    /// a rounded square, the dodai's own shape.
    pub fn ring_point(style: SpinnerStyle, turn: f32, size: f32, w: f32) -> (f32, f32) {
        use std::f32::consts::{FRAC_PI_2, TAU};
        let half = size / 2.0;
        if style != SpinnerStyle::Sweep {
            let (r, a) = ((size - w) / 2.0, turn * TAU);
            return (half + r * a.sin(), half - r * a.cos());
        }
        let (lo, hi) = (w / 2.0, size - w / 2.0);
        let r = (hi - lo) * 0.3;
        let l = hi - lo - 2.0 * r;
        let arc = FRAC_PI_2 * r;
        // Legs from the middle of the top edge, clockwise: straight (x0,y0 -> x1,y1) or a
        // quarter corner (centre, start angle).
        enum Leg {
            Line((f32, f32), (f32, f32)),
            Corner((f32, f32), f32),
        }
        let legs = [
            (l / 2.0, Leg::Line((half, lo), (hi - r, lo))),
            (arc, Leg::Corner((hi - r, lo + r), -FRAC_PI_2)),
            (l, Leg::Line((hi, lo + r), (hi, hi - r))),
            (arc, Leg::Corner((hi - r, hi - r), 0.0)),
            (l, Leg::Line((hi - r, hi), (lo + r, hi))),
            (arc, Leg::Corner((lo + r, hi - r), FRAC_PI_2)),
            (l, Leg::Line((lo, hi - r), (lo, lo + r))),
            (arc, Leg::Corner((lo + r, lo + r), 2.0 * FRAC_PI_2)),
            (l / 2.0, Leg::Line((lo + r, lo), (half, lo))),
        ];
        let total: f32 = legs.iter().map(|(n, _)| n).sum();
        let mut d = turn.rem_euclid(1.0) * total;
        for (n, leg) in &legs {
            if d <= *n {
                let f = if *n > 0.0 { d / n } else { 0.0 };
                return match leg {
                    Leg::Line((x0, y0), (x1, y1)) => (x0 + (x1 - x0) * f, y0 + (y1 - y0) * f),
                    Leg::Corner((cx, cy), a0) => {
                        let a = a0 + f * FRAC_PI_2;
                        (cx + r * a.cos(), cy + r * a.sin())
                    }
                };
            }
            d -= n;
        }
        (half, lo)
    }

    pub fn pose(style: SpinnerStyle, c: &Cell, size: f32, t: f32) -> (f32, f32, f32, f32, f32) {
        let phase = (t * SPIN_RATE).fract();
        match style {
            SpinnerStyle::Quad => {
                let since = (phase - c.lag).rem_euclid(1.0);
                let u = since * 4.0;
                let k = u.floor();
                let f = u - k;
                let f = f * f * (3.0 - 2.0 * f);
                let (x, y) = ring_xy(k + f, size - c.w);
                (x, y, c.w, c.h, 1.0 - c.lag * 0.55)
            }
            SpinnerStyle::Glitch => {
                let since = (phase - c.lag).rem_euclid(1.0);
                let (off, wf, a) = GLITCH[((since * 8.0) as usize).min(7)];
                (c.x + off * size, c.y, c.w * wf, c.h, a)
            }
            SpinnerStyle::Sweep => {
                // A faint track all round; the lit arc runs ahead, its tail lagging and catching
                // up. The dot at the head swells a little.
                let head = (phase * 2.0).fract();
                let len = 0.2 + 0.4 * (0.5 - 0.5 * (phase * std::f32::consts::TAU).cos());
                let behind = (head - c.lag).rem_euclid(1.0);
                let cov = arc_cover(behind, len, 0.05);
                let k = 1.0 - (behind / len).min(1.0);
                let swell = 1.0 + 0.5 * (1.0 - behind / 0.05).clamp(0.0, 1.0) * cov;
                let (w, h) = (c.w * swell, c.h * swell);
                (
                    c.x - (w - c.w) / 2.0,
                    c.y - (h - c.h) / 2.0,
                    w,
                    h,
                    0.16 + cov * (0.24 + 0.6 * k),
                )
            }
            SpinnerStyle::Classic => {
                // The arc turns steadily while its length swings between a sliver and 3/4.
                let len = 0.06 + 0.69 * (0.5 - 0.5 * (phase * std::f32::consts::TAU).cos());
                let tail = phase * 3.0;
                let a = (c.lag - tail).rem_euclid(1.0);
                (c.x, c.y, c.w, c.h, arc_cover(a, len, 0.035))
            }
            SpinnerStyle::Arc => {
                // Head runs ahead steadily; the tail catches up and falls back: the arc breathes.
                let head = phase * 2.0;
                let len = 0.45 + 0.3 * (phase * std::f32::consts::TAU).sin();
                let tail = head - len;
                let a = (c.lag - tail).rem_euclid(1.0);
                let fade = arc_cover(a, len, 0.06) * (0.35 + 0.65 * (a / len).min(1.0));
                (c.x, c.y, c.w, c.h, fade)
            }
            SpinnerStyle::Data => {
                let since = (phase - c.lag).rem_euclid(1.0);
                (c.x, since * (size - c.h), c.w, c.h, 1.0 - 0.6 * since)
            }
            SpinnerStyle::Pillars => {
                let since = (phase - c.lag).rem_euclid(1.0);
                let h = c.h * PILLARS[((since * 4.0) as usize).min(3)];
                (c.x, c.y + c.h - h, c.w, h, 1.0)
            }
            SpinnerStyle::Courses => {
                // A course is laid from the left edge, rests, then is lifted off to the right;
                // it never leaves the box.
                let since = (phase - c.lag).rem_euclid(1.0);
                let (x, w) = if since < 0.4 {
                    (0.0, c.w * (1.0 - (1.0 - since / 0.4).powi(3)))
                } else if since < 0.7 {
                    (0.0, c.w)
                } else {
                    let v = ((since - 0.7) / 0.3).powi(3);
                    (c.w * v, c.w * (1.0 - v))
                };
                (c.x + x, c.y, w.max(0.0), c.h, 1.0)
            }
            SpinnerStyle::Bounce => {
                let since = (phase - c.lag).rem_euclid(1.0);
                let up = if since < 0.55 {
                    (since / 0.55 * std::f32::consts::PI).sin()
                } else {
                    0.0
                };
                (c.x, c.y - up * (size - c.h), c.w, c.h, 1.0)
            }
            SpinnerStyle::Shuffle => {
                let piece = (c.lag * 3.0).round() as usize;
                let (x, y) = ring_xy(shuffle_slot(piece, phase), size - c.w);
                (x, y, c.w, c.h, 1.0)
            }
            _ => {
                let a = cell_alpha(style, c.lag, t);
                let (x, y, w, h) = cell_rect(style, c, a);
                (x, y, w, h, a)
            }
        }
    }

    /// Whether the cells of a style move or change size, not only their opacity.
    pub fn moves(style: SpinnerStyle) -> bool {
        !matches!(
            style,
            SpinnerStyle::Ring | SpinnerStyle::Orbit | SpinnerStyle::Steps
        )
    }

    /// Height of a bar of a [`SpinnerStyle::Bars`] spinner as a fraction of its full height.
    pub fn bar_scale(alpha: f32) -> f32 {
        0.4 + 0.6 * alpha
    }

    /// Centre of dot `i` of `n` on a circle of radius `r` around `(c, c)`, starting at the top
    /// and going clockwise.
    pub fn orbit_xy(i: usize, n: usize, c: f32, r: f32) -> (f32, f32) {
        let a = i as f32 / n.max(1) as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        (c + r * a.cos(), c + r * a.sin())
    }

    /// Left edge of the sliding segment as a fraction of the track after `t` seconds. It sweeps from
    /// just off the left to just off the right, so the loop never jumps.
    pub fn slide_left(t: f32) -> f32 {
        let p = (t * 0.7).fract();
        -SEGMENT + p * (1.0 + SEGMENT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::themes;
    use crate::SpinnerStyle;

    #[test]
    fn quad_cells_keep_a_square_ring_and_return_after_a_cycle() {
        use busy::{cells, pose};
        let s = 24.0;
        let cs = cells(SpinnerStyle::Quad, s);
        assert_eq!(cs.len(), 4);
        for c in &cs {
            let a = pose(SpinnerStyle::Quad, c, s, 0.0);
            let b = pose(SpinnerStyle::Quad, c, s, 1.0 / busy::SPIN_RATE - 1e-4);
            assert!(
                (a.0 - b.0).abs() < 0.1 && (a.1 - b.1).abs() < 0.1,
                "{a:?} {b:?}"
            );
            assert!(a.0 >= -0.01 && a.0 + c.w <= s + 0.01);
        }
    }

    #[test]
    fn shuffle_fills_the_same_slots_at_both_ends_and_never_overlaps() {
        use busy::shuffle_slot;
        let set = |u: f32| {
            let mut v: Vec<i32> = (0..3)
                .map(|p| shuffle_slot(p, u).round() as i32 % 4)
                .collect();
            v.sort();
            v
        };
        assert_eq!(set(0.0), set(0.9999));
        for i in 0..100 {
            let u = i as f32 / 100.0;
            let mut s: Vec<f32> = (0..3).map(|p| shuffle_slot(p, u).rem_euclid(4.0)).collect();
            s.sort_by(f32::total_cmp);
            assert!(s.windows(2).all(|w| w[1] - w[0] > 0.05), "{u} {s:?}");
        }
    }

    fn all_kits() -> Vec<Kit> {
        let mut v = vec![
            Kit::Avatar,
            Kit::Code,
            Kit::Snackbar,
            Kit::Skeleton,
            Kit::Popup,
            Kit::Select,
            Kit::TableHead,
            Kit::TableRow(true),
            Kit::TableRow(false),
            Kit::Dot(None),
        ];
        for sel in [false, true] {
            v.extend([
                Kit::Chip { selected: sel },
                Kit::Row { selected: sel },
                Kit::Page { selected: sel },
            ]);
        }
        for rich in [false, true] {
            v.push(Kit::Tooltip { rich });
        }
        for c in [
            CardVariant::Elevated,
            CardVariant::Filled,
            CardVariant::Outlined,
        ] {
            v.push(Kit::Card(c));
        }
        for tone in [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger] {
            v.extend([
                Kit::Alert { tone, banner: true },
                Kit::Alert {
                    tone,
                    banner: false,
                },
                Kit::Dot(Some(tone)),
                Kit::Count(tone),
            ]);
        }
        for s in [StepState::Done, StepState::Current, StepState::Todo] {
            v.push(Kit::StepDot(s));
        }
        v
    }

    #[test]
    fn every_style_but_the_ring_has_cells_inside_its_box() {
        use super::busy::*;
        use crate::SpinnerStyle::*;
        assert!(cells(Ring, 24.0).is_empty());
        for (st, n) in [
            (Bars, TRAIL),
            (Orbit, TRAIL),
            (Steps, 4),
            (Grid, 9),
            (Scanner, 7),
            (Hop, 3),
            (Petals, 6),
        ] {
            let cs = cells(st, 40.0);
            assert_eq!(cs.len(), n);
            for c in cs {
                assert!(c.x >= -1e-3 && c.y >= -1e-3, "{st:?}");
                assert!(c.x + c.w <= 40.001 && c.y + c.h <= 40.001, "{st:?}");
                assert!((0.0..1.0).contains(&c.lag));
            }
        }
    }

    #[test]
    fn the_head_is_brightest_and_the_tail_fades_behind_it() {
        use super::busy::*;
        use crate::SpinnerStyle::Bars;
        let t = 0.4;
        let a: Vec<f32> = cells(Bars, 40.0)
            .iter()
            .map(|c| cell_alpha(Bars, c.lag, t))
            .collect();
        assert!(a.iter().all(|x| *x >= DIM - 1e-4 && *x <= 1.0));
        let (hi, lo) = (
            a.iter().cloned().fold(0.0, f32::max),
            a.iter().cloned().fold(1.0, f32::min),
        );
        assert!(hi > 0.9 && lo < 0.3, "{a:?}");
        assert!(bar_scale(1.0) == 1.0 && bar_scale(0.0) < bar_scale(1.0));
    }

    #[test]
    fn steps_light_one_square_at_a_time_clockwise() {
        use super::busy::*;
        use crate::SpinnerStyle::Steps;
        let cs = cells(Steps, 40.0);
        let mut seen = vec![];
        for k in 0..60 {
            let t = k as f32 * 0.02;
            let lit: Vec<usize> = (0..4)
                .filter(|i| cell_alpha(Steps, cs[*i].lag, t) == 1.0)
                .collect();
            assert_eq!(lit.len(), 1, "t={t}");
            if seen.last() != Some(&lit[0]) {
                seen.push(lit[0]);
            }
        }
        assert_eq!(&seen[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn orbit_dots_lie_on_the_circle_starting_at_the_top() {
        use super::busy::*;
        let (x, y) = orbit_xy(0, 8, 10.0, 6.0);
        assert!((x - 10.0).abs() < 1e-4 && (y - 4.0).abs() < 1e-4);
        for i in 0..8 {
            let (x, y) = orbit_xy(i, 8, 10.0, 6.0);
            assert!((((x - 10.0).powi(2) + (y - 10.0).powi(2)).sqrt() - 6.0).abs() < 1e-4);
        }
    }

    #[test]
    fn each_design_names_its_own_spinner() {
        use crate::SpinnerStyle::*;
        assert_eq!(crate::material::light().look().spinner, Arc);
        assert_eq!(crate::cyberpunk::variants()[0].look().spinner, Glitch);
        assert_eq!(crate::dodai::light().look().spinner, Quad);
        assert_eq!(crate::neobrutalism::variants()[0].look().spinner, Shuffle);
    }

    #[test]
    fn every_kit_resolves_on_every_theme_with_finite_padding() {
        for t in themes::all() {
            for k in all_kits() {
                for hover in [false, true] {
                    let s = t.kit(k, hover);
                    assert!(s.padding.0 >= 0.0 && s.padding.1 >= 0.0, "{} {k:?}", t.id());
                }
            }
        }
    }

    #[test]
    fn filled_kits_stay_legible() {
        for t in themes::all() {
            for k in all_kits() {
                let s = t.kit(k, false);
                if let Some(fill) = s.fill {
                    if fill.a == 255 && !matches!(k, Kit::StepDot(_) | Kit::Skeleton) {
                        assert!(
                            fill.contrast(s.fg) >= 2.5,
                            "{} {k:?}: {:?} on {:?}",
                            t.id(),
                            s.fg,
                            fill
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn names_are_unique_kebab_case_and_all_covers_every_variant() {
        let all = Kit::all();
        let mut names: Vec<String> = all.iter().map(|k| k.name()).collect();
        assert!(names
            .iter()
            .all(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_lowercase() || c == '-')));
        let n = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), n, "duplicate kit names");
        // 3 cards + snackbar, avatar, code, 2 tooltips, 3 steps, dot, skeleton, popup, select,
        // head, 2 rows of the table, 2 fabs, elevated, segmented strip, 4 toolbars,
        // 4 selectable kinds x 2, 4 tones x 4.
        assert_eq!(
            n,
            3 + 3 + 2 + 3 + 1 + 1 + 1 + 1 + 1 + 2 + 2 + 1 + 1 + 4 + 8 + 16
        );
    }

    #[test]
    fn floating_toolbars_and_buttons_lift_and_docked_bars_do_not() {
        for t in themes::all() {
            assert!(t.kit(Kit::Fab { extended: false }, false).lift);
            assert!(t.kit(Kit::Elevated, false).lift);
            let docked = t.kit(
                Kit::Toolbar {
                    floating: false,
                    vibrant: false,
                },
                false,
            );
            assert!(!docked.lift && !docked.pill);
            let floating = t.kit(
                Kit::Toolbar {
                    floating: true,
                    vibrant: false,
                },
                false,
            );
            assert_eq!(floating.lift, t.look().tonal);
        }
    }

    #[test]
    fn selection_changes_chip_and_hover_changes_row() {
        let t = themes::all().remove(0);
        assert_ne!(
            t.kit(Kit::Chip { selected: true }, false),
            t.kit(Kit::Chip { selected: false }, false)
        );
        assert_ne!(
            t.kit(Kit::Row { selected: false }, true),
            t.kit(Kit::Row { selected: false }, false)
        );
    }

    #[test]
    fn elevated_card_differs_from_outlined() {
        for t in themes::all() {
            assert_ne!(
                t.kit(Kit::Card(CardVariant::Elevated), false),
                t.kit(Kit::Card(CardVariant::Outlined), false),
                "{}",
                t.id()
            );
        }
    }
}

#[cfg(test)]
mod busy_tests {
    use super::busy::{progress_window, slice_points, wave_line, wave_ring};

    #[test]
    fn progress_windows_stay_inside_the_path() {
        assert_eq!(progress_window(Some(0.4), 0.0, false)[0], (0.0, 0.4));
        for i in 0..200 {
            for wrap in [false, true] {
                for (a, b) in progress_window(None, i as f32 * 0.037, wrap) {
                    assert!((0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b) && a <= b);
                }
            }
        }
    }

    #[test]
    fn wave_paths_have_points_and_slice_cleanly() {
        let line = wave_line(100.0, 3.0, 20.0, 2.0);
        assert_eq!(line.first().unwrap().0, 0.0);
        assert_eq!(line.last().unwrap().0, 100.0);
        let ring = wave_ring(48.0, 2.0, 10, 100);
        assert_eq!(ring.len(), 101);
        assert!(slice_points(&ring, 0.5, 0.5).is_empty());
        assert!(slice_points(&ring, 0.0, 1.0).len() == 101);
    }
    use super::busy::*;

    #[test]
    fn the_ring_turns_once_per_period_and_stays_in_range() {
        for i in 0..200 {
            let a = spin_angle(i as f32 * 0.037);
            assert!((0.0..std::f32::consts::TAU).contains(&a));
        }
        assert!(spin_angle(1.0 / SPIN_RATE - 1e-3) > 6.0);
    }

    #[test]
    fn the_segment_enters_from_the_left_and_leaves_on_the_right() {
        assert!((slide_left(0.0) + SEGMENT).abs() < 1e-6);
        let end = slide_left(1.0 / 0.7 - 1e-3);
        assert!(end > 0.99, "{end}");
        for i in 0..300 {
            let l = slide_left(i as f32 * 0.013);
            assert!((-SEGMENT..=1.0).contains(&l));
        }
    }
}

#[cfg(test)]
mod surface_class_tests {
    use super::Kit;
    use crate::SurfaceClass;

    #[test]
    fn every_container_kit_takes_part_in_the_glass() {
        assert_eq!(Kit::Popup.surface_class(), Some(SurfaceClass::Panel));
        assert_eq!(
            Kit::Alert {
                tone: crate::Tone::Info,
                banner: false
            }
            .surface_class(),
            Some(SurfaceClass::Card)
        );
        assert!(Kit::Popup.floats() && !Kit::Code.floats());
        assert_eq!(Kit::Code.surface_class(), Some(SurfaceClass::Card));
        assert_eq!(Kit::Snackbar.surface_class(), Some(SurfaceClass::Bar));
        assert_eq!(
            Kit::Toolbar {
                floating: false,
                vibrant: false
            }
            .surface_class(),
            Some(SurfaceClass::Bar)
        );
    }
}
