//! Resolved, framework-agnostic component styles derived from a [`Theme`].

use crate::{Color, Sizes, Theme, Tone};

/// Button emphasis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Solid brand fill.
    Primary,
    /// Solid secondary (cyan) fill.
    Secondary,
    /// Outlined brand colour on transparent.
    Outline,
    /// Outlined secondary colour on transparent.
    OutlineSecondary,
    /// Solid danger fill.
    Danger,
    /// Brand outline over a translucent dark fill, for use on imagery; fills solid on hover.
    Scrim,
    /// Text only.
    Ghost,
}

/// Interaction state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ControlState {
    #[default]
    Idle,
    Hovered,
    Pressed,
    Focused,
    Disabled,
}

/// Fully resolved colours for one button in one state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonTokens {
    /// `None` = transparent.
    pub background: Option<Color>,
    pub foreground: Color,
    /// `None` = no border.
    pub border: Option<Color>,
}

/// Panel (card) tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelTokens {
    pub background: Color,
    pub border: Color,
    pub title: Color,
}

/// Text-input tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputTokens {
    pub background: Color,
    pub border: Color,
    pub text: Color,
    pub placeholder: Color,
    pub selection: Color,
}

/// Tag / badge tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TagTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    pub border: Option<Color>,
}

/// Geometry of the app-level navigation components (drawer and rail), in px.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NavigationTokens {
    /// Drawer row height.
    pub item_height: f32,
    /// Height of a nested drawer row.
    pub nested_height: f32,
    /// Section headline height.
    pub headline_height: f32,
    /// Horizontal padding inside a row.
    pub item_padding: f32,
    /// Padding around the drawer's rows.
    pub container_padding: f32,
    /// Gap between a row's icon, label and badge.
    pub item_content_gap: f32,
    /// Gap between rows.
    pub item_gap: f32,
    /// Extra left margin per nesting level.
    pub indent: f32,
    /// Icon size.
    pub icon_size: f32,
    /// Widest the drawer grows.
    pub drawer_max_width: f32,
    /// Rail width.
    pub rail_width: f32,
    /// Rail indicator size (width, height).
    pub indicator: (f32, f32),
    /// Rail gap between the indicator and its label.
    pub rail_label_gap: f32,
    /// Navigation bar height.
    pub bar_height: f32,
    /// Width of the navigation bar's indicator.
    pub bar_indicator_width: f32,
    /// Rows and indicators are fully rounded pills.
    pub pill: bool,
    /// Surface behind the drawer and rail.
    pub container: Option<Color>,
    /// Label type roles: (row, nested row, rail label, headline).
    pub roles: (TypeRole, TypeRole, TypeRole, TypeRole),
}

/// Tab tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Underline / indicator colour (active tab).
    pub indicator: Option<Color>,
    /// Drawn as an outlined block with a hard shadow.
    pub outline: bool,
}

/// Toggle / checkbox / radio tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToggleTokens {
    pub track: Color,
    pub thumb: Color,
    pub border: Color,
}

/// Which selection control a [`ToggleTokens`] is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToggleKind {
    Switch,
    Checkbox,
    Radio,
}

/// Progress bar tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressTokens {
    pub track: Color,
    pub fill: Color,
    pub border: Color,
    /// Draw the bar as this many HUD segments (`0` = one continuous bar).
    pub segments: u32,
}

/// List / menu row tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Accent bar at the row's left edge when selected.
    pub marker: Option<Color>,
    /// Drawn as an outlined block with a hard shadow (Neo-Brutalism's selected row).
    pub outline: bool,
}

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

/// Progress of one step in a stepper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Done,
    Current,
    Todo,
}

impl StepState {
    /// State of step `i` when `current` is the active step.
    pub fn of(i: usize, current: usize) -> Self {
        match i.cmp(&current) {
            std::cmp::Ordering::Less => Self::Done,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Todo,
        }
    }
}

/// A generic surface (card, chip, badge, alert, dialog, sheet, popup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceTokens {
    /// `None` = transparent.
    pub background: Option<Color>,
    /// `None` = no border.
    pub border: Option<Color>,
    /// Text colour on the surface.
    pub foreground: Color,
    /// Draw the theme glow around it.
    pub glow: bool,
    /// Draw the theme shadow behind it.
    pub shadow: bool,
}

/// Control size step (buttons, icon buttons): the five Material 3 Expressive sizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ControlSize {
    XSmall,
    #[default]
    Small,
    Medium,
    Large,
    XLarge,
}

impl ControlSize {
    /// Every size, smallest first.
    pub const ALL: [ControlSize; 5] = [
        Self::XSmall,
        Self::Small,
        Self::Medium,
        Self::Large,
        Self::XLarge,
    ];
    /// Short label.
    pub fn name(self) -> &'static str {
        match self {
            Self::XSmall => "XS",
            Self::Small => "S",
            Self::Medium => "M",
            Self::Large => "L",
            Self::XLarge => "XL",
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}

/// Geometry of a [`ControlSize`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeMetrics {
    pub height: f32,
    /// Horizontal padding of a label.
    pub padding: f32,
    pub text_size: f32,
    pub icon_size: f32,
    /// Corner radius of the *square* shape variant (rounded designs).
    pub square_radius: f32,
}

/// One of the 15 Material 3 type-scale roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeRole {
    DisplayLarge,
    DisplayMedium,
    DisplaySmall,
    HeadlineLarge,
    HeadlineMedium,
    HeadlineSmall,
    TitleLarge,
    TitleMedium,
    TitleSmall,
    BodyLarge,
    BodyMedium,
    BodySmall,
    LabelLarge,
    LabelMedium,
    LabelSmall,
}

impl TypeRole {
    /// Every role, largest first.
    pub const ALL: [TypeRole; 15] = [
        Self::DisplayLarge,
        Self::DisplayMedium,
        Self::DisplaySmall,
        Self::HeadlineLarge,
        Self::HeadlineMedium,
        Self::HeadlineSmall,
        Self::TitleLarge,
        Self::TitleMedium,
        Self::TitleSmall,
        Self::BodyLarge,
        Self::BodyMedium,
        Self::BodySmall,
        Self::LabelLarge,
        Self::LabelMedium,
        Self::LabelSmall,
    ];
    /// Readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::DisplayLarge => "Display large",
            Self::DisplayMedium => "Display medium",
            Self::DisplaySmall => "Display small",
            Self::HeadlineLarge => "Headline large",
            Self::HeadlineMedium => "Headline medium",
            Self::HeadlineSmall => "Headline small",
            Self::TitleLarge => "Title large",
            Self::TitleMedium => "Title medium",
            Self::TitleSmall => "Title small",
            Self::BodyLarge => "Body large",
            Self::BodyMedium => "Body medium",
            Self::BodySmall => "Body small",
            Self::LabelLarge => "Label large",
            Self::LabelMedium => "Label medium",
            Self::LabelSmall => "Label small",
        }
    }
    /// Display / headline / title use the heading family; body / label the text family.
    pub fn is_heading(self) -> bool {
        (self as usize) < 9
    }
}

/// Size, line height, weight and tracking of a [`TypeRole`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TypeStyle {
    pub size: f32,
    pub line_height: f32,
    /// CSS weight (400 regular, 500 medium).
    pub weight: u16,
    /// Letter spacing in px.
    pub tracking: f32,
}

/// Which app bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AppBarKind {
    /// Title centred between the leading and trailing icons.
    CenterAligned,
    /// One row, title at the start.
    Small,
    /// Icons on top, title in a second row.
    Medium,
    /// Icons on top, a large title in a tall second row.
    Large,
    /// Actions at the start, a floating action at the end.
    Bottom,
}

/// Resolved app bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AppBarTokens {
    /// Height of the icon row (64 in M3), which is the whole bar for Small / CenterAligned.
    pub row_height: f32,
    /// Total height when fully expanded (Medium 112, Large 152 in M3).
    pub height: f32,
    /// `None` = transparent.
    pub background: Option<Color>,
    pub border: Option<Color>,
    /// Title colour.
    pub title: Color,
    /// Colour of the leading navigation icon.
    pub leading: Color,
    /// Colour of the trailing action icons.
    pub trailing: Color,
    /// Font size of the title when expanded.
    pub title_size: f32,
    /// Font size of the title when collapsed into the icon row.
    pub collapsed_title_size: f32,
    /// Draw the theme glow around the bar.
    pub glow: bool,
}

/// One stepper node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepTokens {
    pub fill: Option<Color>,
    pub stroke: Color,
    /// Colour of the number / check inside the node.
    pub mark: Color,
    /// Colour of the label.
    pub text: Color,
    pub glow: bool,
}

/// Resolve a hook of the theme's design-system spec, falling back to the baseline below.
macro_rules! hook {
    ($self:ident . $m:ident ( $($a:expr),* )) => {
        if let Some(v) = $self.spec.get().and_then(|s| s.$m($self $(, $a)*)) {
            return v;
        }
    };
}

impl Theme {
    fn surface(
        background: Option<Color>,
        border: Option<Color>,
        foreground: Color,
        glow: bool,
        shadow: bool,
    ) -> SurfaceTokens {
        SurfaceTokens {
            background,
            border,
            foreground,
            glow,
            shadow,
        }
    }

    /// Card.
    pub fn card(&self, v: CardVariant) -> SurfaceTokens {
        hook!(self.card(v));
        let p = &self.palette;
        match v {
            CardVariant::Elevated => {
                Self::surface(Some(p.background), Some(p.border), p.text, false, true)
            }
            CardVariant::Filled => {
                Self::surface(Some(p.surface_raised), None, p.text, false, false)
            }
            CardVariant::Outlined => Self::surface(None, Some(p.border), p.text, false, false),
        }
    }

    /// Filter chip.
    pub fn chip(&self, selected: bool, hovered: bool) -> SurfaceTokens {
        hook!(self.chip(selected, hovered));
        let p = &self.palette;
        if selected {
            Self::surface(Some(p.selection), Some(p.primary), p.text, false, false)
        } else {
            Self::surface(
                hovered.then_some(p.surface_raised),
                Some(if hovered { p.border_strong } else { p.border }),
                if hovered { p.text } else { p.text_muted },
                false,
                false,
            )
        }
    }

    /// Solid status badge.
    pub fn badge(&self, tone: Tone) -> SurfaceTokens {
        hook!(self.badge(tone));
        let c = self.palette.tone(tone);
        Self::surface(Some(c), None, self.on(c), false, false)
    }

    /// Alert: tinted and outlined, or a solid banner.
    pub fn alert(&self, tone: Tone, banner: bool) -> SurfaceTokens {
        hook!(self.alert(tone, banner));
        let c = self.palette.tone(tone);
        if banner {
            Self::surface(Some(c), None, self.on(c), false, false)
        } else {
            Self::surface(
                Some(c.with_alpha(0.12)),
                Some(c),
                self.palette.text,
                false,
                false,
            )
        }
    }

    /// Snackbar: `(surface, action colour)`.
    pub fn snackbar(&self) -> (SurfaceTokens, Color) {
        hook!(self.snackbar());
        let p = &self.palette;
        let bg = p.surface_raised;
        (
            Self::surface(Some(bg), None, p.text, false, false),
            if p.primary.contrast(bg) >= 3.0 {
                p.primary
            } else {
                p.text
            },
        )
    }

    /// Dialog box.
    pub fn dialog(&self) -> SurfaceTokens {
        hook!(self.dialog());
        let p = &self.palette;
        Self::surface(Some(p.surface), Some(p.border_strong), p.text, false, true)
    }

    /// Side / bottom sheet.
    pub fn sheet(&self) -> SurfaceTokens {
        hook!(self.sheet());
        let p = &self.palette;
        Self::surface(Some(p.surface), Some(p.border_strong), p.text, false, true)
    }

    /// Floating popup (menu, dropdown list).
    pub fn popup(&self) -> SurfaceTokens {
        hook!(self.popup());
        let p = &self.palette;
        Self::surface(
            Some(p.surface_raised),
            Some(p.border_strong),
            p.text,
            false,
            true,
        )
    }

    /// Modal scrim: a translucent dark veil.
    pub fn scrim(&self) -> Color {
        hook!(self.scrim());
        self.palette.roles.scrim.with_alpha(0.4)
    }

    /// Stepper node.
    pub fn step(&self, state: StepState) -> StepTokens {
        hook!(self.step(state));
        let p = &self.palette;
        match state {
            StepState::Done => StepTokens {
                fill: Some(p.primary),
                stroke: p.primary,
                mark: p.on_primary,
                text: p.text,
                glow: false,
            },
            StepState::Current => StepTokens {
                fill: Some(p.selection),
                stroke: p.primary,
                mark: p.text,
                text: p.text,
                glow: true,
            },
            StepState::Todo => StepTokens {
                fill: None,
                stroke: p.border_strong,
                mark: p.text_muted,
                text: p.text_muted,
                glow: false,
            },
        }
    }

    /// Tag / badge: `solid` fills with the tone, otherwise outlined.
    pub fn tag(&self, tone: Tone, solid: bool) -> TagTokens {
        let c = self.palette.tone(tone);
        if solid {
            TagTokens {
                background: Some(c),
                foreground: self.on(c),
                border: None,
            }
        } else {
            TagTokens {
                background: Some(c.with_alpha(0.12)),
                foreground: c,
                border: Some(c),
            }
        }
    }

    /// Tab: the active tab is highlighted with an indicator.
    pub fn tab(&self, active: bool, hovered: bool) -> TabTokens {
        hook!(self.tab(active, hovered));
        let p = &self.palette;
        if active {
            // Plain: brand-coloured label with an underline indicator.
            TabTokens {
                background: None,
                foreground: self.legible(p.primary),
                indicator: Some(p.primary),
                outline: false,
            }
        } else if hovered {
            TabTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                indicator: None,
                outline: false,
            }
        } else {
            TabTokens {
                background: None,
                foreground: p.text_muted,
                indicator: None,
                outline: false,
            }
        }
    }

    /// Toggle / checkbox / radio.
    pub fn toggle(&self, on: bool, state: ControlState) -> ToggleTokens {
        hook!(self.toggle(on, state));
        let p = &self.palette;
        if state == ControlState::Disabled {
            return ToggleTokens {
                track: p.surface_raised,
                thumb: p.text_disabled,
                border: p.border,
            };
        }
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        if on {
            ToggleTokens {
                track: if lit { p.primary_hover } else { p.primary },
                thumb: p.on_primary,
                border: p.primary,
            }
        } else {
            ToggleTokens {
                track: p.surface_raised,
                thumb: if lit { p.text } else { p.text_muted },
                border: if lit { p.border_strong } else { p.border },
            }
        }
    }

    /// Switch / checkbox / radio colours for a design system with its own selection controls
    /// (`track` = container, `thumb` = handle, mark or dot, `border` = outline). Without a spec
    /// override this is [`Theme::toggle`].
    pub fn choice_toggle(&self, kind: ToggleKind, on: bool, state: ControlState) -> ToggleTokens {
        hook!(self.choice_toggle(kind, on, state));
        self.toggle(on, state)
    }

    /// Progress bar in a status tone (`None` = brand colour).
    pub fn progress(&self, tone: Option<Tone>) -> ProgressTokens {
        hook!(self.progress(tone));
        let p = &self.palette;
        ProgressTokens {
            track: p.surface_raised,
            fill: tone.map_or(p.primary, |t| p.tone(t)),
            border: p.border,
            segments: 0,
        }
    }

    /// List / menu row.
    pub fn list_item(&self, selected: bool, hovered: bool) -> ListItemTokens {
        hook!(self.list_item(selected, hovered));
        let p = &self.palette;
        if selected {
            ListItemTokens {
                background: Some(p.selection),
                foreground: p.text,
                marker: None,
                outline: false,
            }
        } else if hovered {
            ListItemTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                marker: None,
                outline: false,
            }
        } else {
            ListItemTokens {
                background: None,
                foreground: p.text_muted,
                marker: None,
                outline: false,
            }
        }
    }

    /// The size scale; the design's own, or the shared defaults.
    pub fn sizes(&self) -> Sizes {
        hook!(self.sizes());
        Sizes::default()
    }

    /// Drawer and rail geometry; derived from spacing and the control height unless the design
    /// overrides it.
    pub fn navigation(&self) -> NavigationTokens {
        hook!(self.navigation());
        let sp = &self.spacing;
        NavigationTokens {
            item_height: self.list_row_height(false),
            nested_height: self.list_row_height(true),
            headline_height: self.list_row_height(false),
            item_padding: sp.md,
            container_padding: sp.sm,
            item_content_gap: sp.sm,
            item_gap: sp.sm,
            indent: sp.md,
            icon_size: self.sizes().icon_sm,
            drawer_max_width: 360.0,
            rail_width: 80.0,
            indicator: (self.shape.control_height * 1.75, self.shape.control_height),
            rail_label_gap: sp.sm,
            bar_height: 80.0,
            bar_indicator_width: self.shape.control_height * 1.75 + sp.sm,
            pill: false,
            container: None,
            roles: (
                TypeRole::LabelLarge,
                TypeRole::LabelMedium,
                TypeRole::LabelMedium,
                TypeRole::TitleSmall,
            ),
        }
    }

    /// Colours of a drawer row or rail destination (a list row unless the design overrides).
    pub fn navigation_item(&self, selected: bool, hovered: bool) -> ListItemTokens {
        hook!(self.navigation_item(selected, hovered));
        self.list_item(selected, hovered)
    }

    /// Divider / separator colour.
    pub fn divider(&self) -> Color {
        hook!(self.divider());
        self.palette.border
    }

    /// Resolve a button style.
    pub fn button(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        hook!(self.button(variant, state));
        let p = &self.palette;
        use ButtonVariant::*;
        use ControlState::*;
        if state == Disabled {
            return ButtonTokens {
                background: Some(p.surface_raised),
                foreground: p.text_disabled,
                border: None,
            };
        }
        let lit = matches!(state, Hovered | Pressed | Focused);
        let (solid, solid_hi, solid_lo, on) = match variant {
            Primary => (p.primary, p.primary_hover, p.primary_active, p.on_primary),
            Secondary => (
                p.secondary,
                p.secondary_hover,
                p.secondary.darken(0.2),
                p.on_secondary,
            ),
            Danger => (
                p.danger,
                p.danger.lighten(0.15),
                p.danger.darken(0.2),
                self.on(p.danger),
            ),
            _ => (p.primary, p.primary_hover, p.primary_active, p.on_primary),
        };
        match variant {
            Primary | Secondary | Danger => ButtonTokens {
                background: Some(match state {
                    Pressed => solid_lo,
                    Hovered | Focused => solid_hi,
                    _ => solid,
                }),
                foreground: on,
                border: None,
            },
            Outline | OutlineSecondary => {
                let c = if variant == Outline {
                    p.primary
                } else {
                    p.secondary
                };
                ButtonTokens {
                    // Plain: a tint on hover, the outline stays.
                    background: if lit { Some(c.with_alpha(0.12)) } else { None },
                    foreground: self.legible(c),
                    border: Some(c),
                }
            }
            Scrim => ButtonTokens {
                background: Some(if lit {
                    p.primary
                } else {
                    p.background.with_alpha(0.5)
                }),
                foreground: if lit {
                    p.on_primary
                } else {
                    self.legible(p.primary)
                },
                border: Some(p.primary),
            },
            Ghost => ButtonTokens {
                background: if lit { Some(p.surface_raised) } else { None },
                foreground: if lit { self.legible(p.primary) } else { p.text },
                border: None,
            },
        }
    }

    /// App bar tokens; `scrolled` is true once content scrolls beneath it.
    pub fn app_bar(&self, kind: AppBarKind, scrolled: bool) -> AppBarTokens {
        hook!(self.app_bar(kind, scrolled));
        let p = &self.palette;
        let h = self.shape.control_height;
        let (row, height, title) = match kind {
            AppBarKind::CenterAligned | AppBarKind::Small => {
                (h * 1.3, h * 1.3, self.typography.size_lg)
            }
            AppBarKind::Medium => (h * 1.3, h * 2.0, self.typography.size_xl),
            AppBarKind::Large => (h * 1.3, h * 2.4, self.typography.size_xl),
            AppBarKind::Bottom => (h * 1.3, h * 1.3, self.typography.size_lg),
        };
        let panel = self.panel(false);
        AppBarTokens {
            row_height: row,
            height,
            background: Some(panel.background),
            border: Some(panel.border),
            title: panel.title,
            leading: p.text,
            trailing: p.text,
            title_size: title,
            collapsed_title_size: self.typography.size_lg,
            glow: false,
        }
    }

    /// Colour of the hard offset shadow of hard-edged designs.
    pub fn hard_shadow_color(&self) -> Color {
        hook!(self.hard_shadow_color());
        self.palette.primary_active
    }

    /// The design's own shadow layers for an elevation level,
    /// or `None` to use the default two-layer shadow.
    pub fn elevation_layers(&self, level: u8) -> Option<Vec<crate::ShadowLayer>> {
        self.spec
            .get()
            .and_then(|s| s.elevation_layers(self, level))
    }

    /// The page grid at a viewport width.
    pub fn page_grid(&self, viewport: f32) -> crate::GridSpec {
        self.spec
            .get()
            .and_then(|s| s.page_grid(self, viewport))
            .unwrap_or(crate::GridSpec {
                columns: if viewport >= 760.0 { 12 } else { 4 },
                gap: self.spacing.md,
                margin: self.spacing.lg,
                max_width: None,
            })
    }

    /// This theme with its type sizes and spacing resolved for a viewport width. Designs
    /// without fluid scales return an unchanged copy.
    pub fn responsive(&self, viewport: f32) -> Theme {
        let mut t = self.clone();
        if let Some(f) = self.spec.get().and_then(|s| s.fluid(self, viewport)) {
            t.typography = f.typography;
            t.spacing = f.spacing;
        }
        t
    }

    /// The surface mix this design suggests (none when the design has no opinion).
    pub fn surface_preset(&self) -> crate::SurfaceFx {
        self.spec
            .get()
            .and_then(|s| s.surface_preset(self))
            .unwrap_or_default()
    }

    /// The design's frosted glass: tint and backdrop blur.
    pub fn frost(&self) -> crate::Frost {
        self.spec
            .get()
            .and_then(|s| s.frost(self))
            .unwrap_or(crate::Frost {
                tint: self.palette.surface.with_alpha(0.6),
                blur: 24.0,
            })
    }

    /// Stops of the dark fade laid over media.
    pub fn fade_stops(&self) -> Vec<crate::FadeStop> {
        self.spec
            .get()
            .and_then(|s| s.fade_stops(self))
            .unwrap_or_else(|| {
                vec![
                    crate::FadeStop {
                        alpha: 0.7,
                        at: 0.0,
                    },
                    crate::FadeStop {
                        alpha: 0.0,
                        at: 1.0,
                    },
                ]
            })
    }

    /// Keyboard focus indicator `(colour, thickness px)`.
    pub fn focus_ring(&self) -> (Color, f32) {
        hook!(self.focus_ring());
        (self.palette.focus_ring, 2.0)
    }

    /// Row height of a list item (nested rows are denser).
    pub fn list_row_height(&self, nested: bool) -> f32 {
        hook!(self.list_row_height(nested));
        self.shape.control_height * if nested { 0.65 } else { 0.8 }
    }

    /// Row height of a list item with `lines` lines of text (1 to 3).
    pub fn list_lines_height(&self, lines: u8) -> f32 {
        let lines = lines.clamp(1, 3);
        hook!(self.list_lines_height(lines));
        self.list_row_height(false) * (1.0 + 0.4 * (lines - 1) as f32)
    }

    /// Row height of a menu / dropdown item.
    pub fn menu_row_height(&self) -> f32 {
        hook!(self.menu_row_height());
        self.shape.control_height * 0.8
    }

    /// Tooltip `(container, label)` colours; `rich` tooltips carry a title and body.
    pub fn tooltip(&self, rich: bool) -> (Color, Color) {
        hook!(self.tooltip(rich));
        (self.palette.overlay, self.palette.text)
    }

    /// Height, padding, text and icon size of a button at `size`. Baseline: multiples of the
    /// theme's control height, spacing and type scale.
    pub fn control_metrics(&self, size: ControlSize) -> SizeMetrics {
        hook!(self.control_metrics(size));
        let i = size.index();
        let h = self.shape.control_height;
        let ty = &self.typography;
        SizeMetrics {
            height: h * [0.75, 1.0, 1.25, 1.6, 2.0][i],
            padding: self.spacing.md * [0.75, 1.0, 1.25, 1.75, 2.25][i],
            text_size: [ty.size_sm, ty.size_md, ty.size_lg, ty.size_xl, ty.size_xxl][i],
            icon_size: h * [0.45, 0.5, 0.5, 0.55, 0.65][i],
            square_radius: [12.0, 12.0, 16.0, 28.0, 28.0][i],
        }
    }

    /// Type-scale style of `role`. Baseline: derived from the theme's six sizes.
    pub fn type_style(&self, role: TypeRole) -> TypeStyle {
        hook!(self.type_style(role));
        let ty = &self.typography;
        use TypeRole::*;
        let size = match role {
            DisplayLarge => ty.size_xxl * 1.5,
            DisplayMedium => ty.size_xxl * 1.25,
            DisplaySmall => ty.size_xxl,
            HeadlineLarge => ty.size_xl * 1.33,
            HeadlineMedium => ty.size_xl * 1.17,
            HeadlineSmall => ty.size_xl,
            TitleLarge => ty.size_lg * 1.2,
            TitleMedium => ty.size_lg,
            TitleSmall => ty.size_md,
            BodyLarge => ty.size_md,
            BodyMedium => ty.size_sm,
            BodySmall => ty.size_xs * 1.1,
            LabelLarge => ty.size_sm,
            LabelMedium => ty.size_xs * 1.1,
            LabelSmall => ty.size_xs,
        };
        let heading = (role as usize) < 6;
        TypeStyle {
            size,
            line_height: size * if heading { 1.2 } else { 1.4 },
            weight: if matches!(
                role,
                TitleMedium | TitleSmall | LabelLarge | LabelMedium | LabelSmall
            ) {
                500
            } else {
                400
            },
            tracking: 0.0,
        }
    }

    /// Horizontal padding of a button label.
    pub fn button_padding(&self) -> f32 {
        hook!(self.button_padding());
        self.spacing.md
    }

    /// A raised button: the `Secondary` button unless the design system lifts it.
    pub fn elevated_button(&self, state: ControlState) -> ButtonTokens {
        hook!(self.elevated_button(state));
        self.button(ButtonVariant::Secondary, state)
    }

    /// Floating action button colours (the plain button tokens unless overridden).
    pub fn fab(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        hook!(self.fab(variant, state));
        self.button(variant, state)
    }

    /// Icon button colours (the plain button tokens unless overridden).
    pub fn icon_button(
        &self,
        variant: ButtonVariant,
        state: ControlState,
        selected: bool,
    ) -> ButtonTokens {
        let base = self.button(variant, state);
        if let Some(b) = self
            .spec
            .get()
            .and_then(|s| s.icon_button(self, variant, state, selected, base))
        {
            return b;
        }
        base
    }

    /// `c` as text on the page background: itself when it holds 3:1 contrast,
    /// otherwise the theme's text colour (a pale brand colour on a light page).
    pub fn legible(&self, c: Color) -> Color {
        if c.contrast(self.palette.background) >= 3.0 {
            c
        } else {
            self.palette.text
        }
    }

    /// Readable text colour on top of `bg`.
    pub fn on(&self, bg: Color) -> Color {
        let p = &self.palette;
        let best = if bg.contrast(p.background) >= bg.contrast(p.text) {
            p.background
        } else {
            p.text
        };
        if best.contrast(bg) >= 4.5 {
            return best;
        }
        // Neither theme colour reads well enough (a mid-tone fill): use black or white.
        let (black, white) = (Color::hex(0x000000), Color::hex(0xffffff));
        if black.contrast(bg) >= white.contrast(bg) {
            black
        } else {
            white
        }
    }

    /// Resolve a panel style; `focused` highlights the border.
    pub fn panel(&self, focused: bool) -> PanelTokens {
        let p = &self.palette;
        PanelTokens {
            background: p.surface,
            border: if focused { p.primary } else { p.border },
            title: if focused {
                self.legible(p.primary)
            } else {
                p.text_muted
            },
        }
    }

    /// Resolve a text-input style.
    pub fn input(&self, focused: bool) -> InputTokens {
        hook!(self.input(focused));
        let p = &self.palette;
        InputTokens {
            background: p.background,
            border: if focused { p.primary } else { p.border_strong },
            text: p.text,
            placeholder: p.text_disabled,
            selection: p.selection,
        }
    }
}
