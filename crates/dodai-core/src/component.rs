//! Resolved, framework-agnostic component styles derived from a [`Theme`].

use crate::{Color, Theme, Tone};

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

/// Tab tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TabTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Underline / indicator colour (active tab).
    pub indicator: Option<Color>,
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
}

/// List / menu row tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListItemTokens {
    pub background: Option<Color>,
    pub foreground: Color,
    /// Accent bar at the row's left edge when selected.
    pub marker: Option<Color>,
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

    fn tonal(&self) -> bool {
        self.shape.kind() == crate::ShapeKind::Rounded
    }

    /// Card. Rounded (Material) themes use the tonal container roles.
    pub fn card(&self, v: CardVariant) -> SurfaceTokens {
        let p = &self.palette;
        let r = &p.roles;
        let tonal = self.tonal();
        let line = if tonal { r.outline_variant } else { p.border };
        match v {
            CardVariant::Elevated => Self::surface(
                Some(if tonal {
                    r.surface_container_low
                } else {
                    p.background
                }),
                (!tonal).then_some(line),
                p.text,
                false,
                true,
            ),
            CardVariant::Filled => Self::surface(
                Some(if tonal {
                    r.surface_container_highest
                } else {
                    p.surface_raised
                }),
                None,
                p.text,
                false,
                false,
            ),
            CardVariant::Outlined => Self::surface(None, Some(line), p.text, false, false),
        }
    }

    /// Filter chip.
    pub fn chip(&self, selected: bool, hovered: bool) -> SurfaceTokens {
        let p = &self.palette;
        if selected && self.tonal() {
            let r = &p.roles;
            Self::surface(
                Some(r.secondary_container),
                None,
                r.on_secondary_container,
                false,
                false,
            )
        } else if selected {
            Self::surface(Some(p.selection), Some(p.primary), p.text, false, false)
        } else if self.tonal() {
            // M3 filter chip, unselected: transparent with an `outline` border; hover is a
            // state layer in `on-surface-variant`.
            let r = &p.roles;
            Self::surface(
                hovered.then(|| r.on_surface_variant.with_alpha(0.08)),
                Some(r.outline),
                r.on_surface_variant,
                false,
                false,
            )
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
        let c = self.palette.tone(tone);
        Self::surface(Some(c), None, self.on(c), false, false)
    }

    /// Alert: tinted and outlined, or a solid banner.
    pub fn alert(&self, tone: Tone, banner: bool) -> SurfaceTokens {
        let c = self.palette.tone(tone);
        if banner && self.shape.kind() == crate::ShapeKind::Rounded {
            // Rounded (Material): a tonal container, not a saturated slab.
            let fill = match tone {
                Tone::Danger => self.palette.roles.error_container,
                _ => self.palette.roles.surface_container_high.mix(c, 0.25),
            };
            Self::surface(Some(fill), None, self.on(fill), false, false)
        } else if banner {
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

    /// Snackbar: `(surface, action colour)`. Rounded (Material) uses the inverse surface.
    pub fn snackbar(&self) -> (SurfaceTokens, Color) {
        let p = &self.palette;
        if self.shape.kind() == crate::ShapeKind::Rounded {
            (
                Self::surface(
                    Some(p.roles.inverse_surface),
                    None,
                    p.roles.inverse_on_surface,
                    false,
                    true,
                ),
                p.roles.inverse_primary,
            )
        } else {
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
    }

    /// Dialog box.
    pub fn dialog(&self) -> SurfaceTokens {
        let p = &self.palette;
        if self.tonal() {
            let r = &p.roles;
            return Self::surface(
                Some(r.surface_container_high),
                None,
                r.on_surface,
                false,
                true,
            );
        }
        Self::surface(Some(p.surface), Some(p.primary), p.text, true, true)
    }

    /// Side / bottom sheet.
    pub fn sheet(&self) -> SurfaceTokens {
        let p = &self.palette;
        if self.tonal() {
            let r = &p.roles;
            return Self::surface(
                Some(r.surface_container_low),
                None,
                r.on_surface,
                false,
                true,
            );
        }
        Self::surface(Some(p.surface), Some(p.border_strong), p.text, false, true)
    }

    /// Floating popup (menu, dropdown list).
    pub fn popup(&self) -> SurfaceTokens {
        let p = &self.palette;
        if self.tonal() {
            let r = &p.roles;
            return Self::surface(Some(r.surface_container), None, r.on_surface, false, true);
        }
        Self::surface(
            Some(p.surface_raised),
            Some(p.border_strong),
            p.text,
            true,
            true,
        )
    }

    /// Modal scrim: the page colour, never fully opaque.
    pub fn scrim(&self) -> Color {
        if self.tonal() {
            return self.palette.roles.scrim.with_alpha(0.32);
        }
        self.palette.background.with_alpha(0.7)
    }

    /// Stepper node.
    pub fn step(&self, state: StepState) -> StepTokens {
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
        let p = &self.palette;
        if self.tonal() {
            // M3 primary tab: transparent, `primary` label + indicator when active.
            let r = &p.roles;
            return TabTokens {
                background: hovered
                    .then(|| if active { p.primary } else { r.on_surface }.with_alpha(0.08)),
                foreground: if active {
                    p.primary
                } else {
                    r.on_surface_variant
                },
                indicator: active.then_some(p.primary),
            };
        }
        if active {
            TabTokens {
                background: Some(p.primary),
                foreground: p.on_primary,
                indicator: Some(p.primary),
            }
        } else if hovered {
            TabTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                indicator: None,
            }
        } else {
            TabTokens {
                background: None,
                foreground: p.text_muted,
                indicator: None,
            }
        }
    }

    /// Toggle / checkbox / radio.
    pub fn toggle(&self, on: bool, state: ControlState) -> ToggleTokens {
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
                // Rounded (Material): the unselected outline is the `outline` role.
                border: if self.shape.kind() == crate::ShapeKind::Rounded {
                    if lit {
                        p.roles.on_surface
                    } else {
                        p.roles.outline
                    }
                } else if lit {
                    p.border_strong
                } else {
                    p.border
                },
            }
        }
    }

    /// Material 3 switch / checkbox / radio colours (`track` = container, `thumb` = handle, mark
    /// or dot, `border` = outline). Non-rounded themes get [`Theme::toggle`].
    pub fn m3_toggle(&self, kind: ToggleKind, on: bool, state: ControlState) -> ToggleTokens {
        if !self.tonal() {
            return self.toggle(on, state);
        }
        let r = &self.palette.roles;
        let p = &self.palette;
        let clear = r.on_surface.with_alpha(0.0);
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        if state == ControlState::Disabled {
            let d = r.on_surface;
            return match (kind, on) {
                (ToggleKind::Switch, true) => ToggleTokens {
                    track: d.with_alpha(0.12),
                    thumb: p.surface,
                    border: d.with_alpha(0.12),
                },
                (ToggleKind::Switch, false) => ToggleTokens {
                    track: r.surface_container_highest.with_alpha(0.12),
                    thumb: d.with_alpha(0.38),
                    border: d.with_alpha(0.12),
                },
                (ToggleKind::Checkbox, true) => ToggleTokens {
                    track: d.with_alpha(0.38),
                    thumb: p.surface,
                    border: d.with_alpha(0.38),
                },
                (_, _) => ToggleTokens {
                    track: clear,
                    thumb: if on { d.with_alpha(0.38) } else { clear },
                    border: d.with_alpha(0.38),
                },
            };
        }
        match (kind, on) {
            (ToggleKind::Switch, true) => ToggleTokens {
                track: p.primary,
                thumb: if lit {
                    r.primary_container
                } else {
                    p.on_primary
                },
                border: p.primary,
            },
            (ToggleKind::Switch, false) => ToggleTokens {
                track: r.surface_container_highest,
                thumb: if lit { r.on_surface_variant } else { r.outline },
                border: r.outline,
            },
            (ToggleKind::Checkbox, true) => ToggleTokens {
                track: p.primary,
                thumb: p.on_primary,
                border: p.primary,
            },
            (ToggleKind::Checkbox, false) => ToggleTokens {
                track: clear,
                thumb: clear,
                border: if lit {
                    r.on_surface
                } else {
                    r.on_surface_variant
                },
            },
            (ToggleKind::Radio, true) => ToggleTokens {
                track: clear,
                thumb: p.primary,
                border: p.primary,
            },
            (ToggleKind::Radio, false) => ToggleTokens {
                track: clear,
                thumb: clear,
                border: if lit {
                    r.on_surface
                } else {
                    r.on_surface_variant
                },
            },
        }
    }

    /// Progress bar in a status tone (`None` = brand colour).
    pub fn progress(&self, tone: Option<Tone>) -> ProgressTokens {
        let p = &self.palette;
        if self.tonal() {
            let track = p.roles.surface_container_highest;
            return ProgressTokens {
                track,
                fill: tone.map_or(p.primary, |t| p.tone(t)),
                border: track,
            };
        }
        ProgressTokens {
            track: p.surface_raised,
            fill: tone.map_or(p.primary, |t| p.tone(t)),
            border: p.border,
        }
    }

    /// List / menu row.
    pub fn list_item(&self, selected: bool, hovered: bool) -> ListItemTokens {
        let p = &self.palette;
        let rounded = self.shape.kind() == crate::ShapeKind::Rounded;
        if rounded && !selected {
            // M3 list: `on-surface` text, an 8 % state layer on hover.
            let r = &p.roles;
            return ListItemTokens {
                background: hovered.then(|| r.on_surface.with_alpha(0.08)),
                foreground: r.on_surface,
                marker: None,
            };
        }
        if selected {
            ListItemTokens {
                // Rounded (Material): the secondary-container pill.
                background: Some(if rounded {
                    p.roles.secondary_container
                } else {
                    p.selection
                }),
                foreground: if rounded {
                    p.roles.on_secondary_container
                } else {
                    p.text
                },
                // Chamfered themes mark the selection with an accent bar;
                // rounded (Material) ones rely on the filled pill alone.
                marker: (self.shape.kind() != crate::ShapeKind::Rounded).then_some(p.primary),
            }
        } else if hovered {
            ListItemTokens {
                background: Some(p.surface_raised),
                foreground: p.text,
                marker: None,
            }
        } else {
            ListItemTokens {
                background: None,
                foreground: p.text_muted,
                marker: None,
            }
        }
    }

    /// Divider / separator colour.
    pub fn divider(&self) -> Color {
        if self.shape.kind() == crate::ShapeKind::Rounded {
            self.palette.roles.outline_variant
        } else {
            self.palette.border
        }
    }

    /// Resolve a button style.
    pub fn button(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        let p = &self.palette;
        use ButtonVariant::*;
        use ControlState::*;
        if self.tonal() {
            return self.m3_button(variant, state);
        }
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
                    // Cyberpunk "fill on hover": outline inverts to solid.
                    background: if lit { Some(c) } else { None },
                    foreground: if lit { self.on(c) } else { self.legible(c) },
                    border: Some(c),
                }
            }
            Ghost => ButtonTokens {
                background: if lit { Some(p.surface_raised) } else { None },
                foreground: if lit { self.legible(p.primary) } else { p.text },
                border: None,
            },
        }
    }

    /// Floating action button colours (M3 container roles on rounded themes; the plain
    /// button tokens elsewhere). Primary/Secondary/other map to the primary / secondary /
    /// tertiary container, Outline and Ghost to the surface FAB.
    /// M3 elevated button: `surface-container-low` container, `primary` label, lifted by a
    /// shadow (the caller adds the elevation). Themes without tonal surfaces get the plain
    /// `Secondary` button, so nothing needs a theme id.
    pub fn elevated_button(&self, state: ControlState) -> ButtonTokens {
        if !self.tonal() {
            return self.button(ButtonVariant::Secondary, state);
        }
        let r = &self.palette.roles;
        if state == ControlState::Disabled {
            return ButtonTokens {
                background: Some(r.on_surface.with_alpha(0.12)),
                foreground: r.on_surface.with_alpha(0.38),
                border: None,
            };
        }
        let layer = match state {
            ControlState::Hovered => 0.08,
            ControlState::Pressed | ControlState::Focused => 0.10,
            _ => 0.0,
        };
        ButtonTokens {
            background: Some(r.surface_container_low.mix(self.palette.primary, layer)),
            foreground: self.palette.primary,
            border: None,
        }
    }

    pub fn fab(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        if !self.tonal() {
            return self.button(variant, state);
        }
        let r = &self.palette.roles;
        use ButtonVariant::*;
        let (bg, fg) = match variant {
            Primary => (r.primary_container, r.on_primary_container),
            Secondary => (r.secondary_container, r.on_secondary_container),
            Danger => (r.error_container, r.on_error_container),
            Outline | OutlineSecondary => (r.tertiary_container, r.on_tertiary_container),
            Ghost => (r.surface_container_high, self.palette.primary),
        };
        let layer = match state {
            ControlState::Hovered => 0.08,
            ControlState::Pressed | ControlState::Focused => 0.10,
            _ => 0.0,
        };
        ButtonTokens {
            background: Some(bg.mix(fg, layer)),
            foreground: fg,
            border: None,
        }
    }

    /// Icon button colours. M3: standard (`Ghost`) icons are `on-surface-variant`, `primary`
    /// when `selected`; outlined icons are `on-surface-variant` on an `outline` border; filled
    /// and tonal follow the buttons. Other themes use the button tokens.
    pub fn icon_button(
        &self,
        variant: ButtonVariant,
        state: ControlState,
        selected: bool,
    ) -> ButtonTokens {
        let mut b = self.button(variant, state);
        if !self.tonal() || state == ControlState::Disabled {
            return b;
        }
        let r = &self.palette.roles;
        let layer = match state {
            ControlState::Hovered => 0.08,
            ControlState::Pressed | ControlState::Focused => 0.10,
            _ => 0.0,
        };
        match variant {
            ButtonVariant::Ghost | ButtonVariant::Outline | ButtonVariant::OutlineSecondary => {
                let fg = if selected {
                    self.palette.primary
                } else {
                    r.on_surface_variant
                };
                b.foreground = fg;
                b.background = (layer > 0.0).then(|| fg.with_alpha(layer));
            }
            _ => {}
        }
        b
    }

    /// Material 3 buttons: filled, tonal, outlined, text; state layers 8% hover / 10% focus and
    /// press; disabled is `on-surface` at 12% (container) and 38% (label).
    fn m3_button(&self, variant: ButtonVariant, state: ControlState) -> ButtonTokens {
        use ButtonVariant::*;
        use ControlState::*;
        let r = &self.palette.roles;
        let p = &self.palette;
        // (container, label, outlined)
        let (container, label, outlined) = match variant {
            Primary => (Some(p.primary), p.on_primary, false),
            Secondary => (Some(r.secondary_container), r.on_secondary_container, false),
            Danger => (Some(p.danger), r.on_error, false),
            Outline => (None, p.primary, true),
            OutlineSecondary => (None, p.secondary, true),
            Ghost => (None, p.primary, false),
        };
        if state == Disabled {
            let disabled = r.on_surface;
            return ButtonTokens {
                background: container.map(|_| disabled.with_alpha(0.12)),
                foreground: disabled.with_alpha(0.38),
                border: outlined.then(|| disabled.with_alpha(0.12)),
            };
        }
        let layer = match state {
            Hovered => 0.08,
            Pressed | Focused => 0.10,
            _ => 0.0,
        };
        let background = match container {
            Some(c) => Some(c.mix(label, layer)),
            None if layer > 0.0 => Some(label.with_alpha(layer)),
            None => None,
        };
        ButtonTokens {
            background,
            foreground: label,
            border: outlined.then_some(if state == Focused { label } else { r.outline }),
        }
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
        let p = &self.palette;
        if self.tonal() {
            // M3 outlined text field: `outline` border (primary when focused), `on-surface` text.
            let r = &p.roles;
            return InputTokens {
                background: p.background,
                border: if focused { p.primary } else { r.outline },
                text: r.on_surface,
                placeholder: r.on_surface_variant,
                selection: p.selection,
            };
        }
        InputTokens {
            background: p.background,
            border: if focused {
                p.secondary
            } else {
                p.border_strong
            },
            text: p.text,
            placeholder: p.text_disabled,
            selection: p.selection,
        }
    }
}
