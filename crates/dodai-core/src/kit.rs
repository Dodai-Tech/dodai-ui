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
        };
        match kit {
            Kit::Card(v) => from(theme.card(v), (sp.md, sp.md), false),
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
            },
            Kit::Code => KitStyle {
                fill: Some(p.roles.surface_container_highest),
                border: Some(p.border),
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.md),
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
                }
            }
            Kit::Popup => from(theme.popup(), (sp.xs, sp.xs), false),
            Kit::Select => {
                let i = theme.input(hovered);
                KitStyle {
                    fill: Some(i.background),
                    border: Some(i.border),
                    fg: i.text,
                    glow: false,
                    pill: false,
                    padding: (sp.md, sp.sm),
                }
            }
            Kit::Skeleton => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (0.0, 0.0),
            },
            Kit::TableHead => KitStyle {
                fill: Some(p.surface_raised),
                border: None,
                fg: p.text_muted,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
            },
            Kit::TableRow(banded) => KitStyle {
                fill: banded.then_some(p.roles.surface_container_low),
                border: None,
                fg: p.text,
                glow: false,
                pill: false,
                padding: (sp.md, sp.sm),
            },
        }
    }
}

/// Motion of the busy indicators, shared by every toolkit so they all turn at the same rate.
pub mod busy {
    /// Revolutions per second of the ring.
    pub const SPIN_RATE: f32 = 0.9;
    /// Width of the sliding segment as a fraction of the track.
    pub const SEGMENT: f32 = 0.3;

    /// Angle (radians) of the ring after `t` seconds.
    pub fn spin_angle(t: f32) -> f32 {
        (t * SPIN_RATE).fract() * std::f32::consts::TAU
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
