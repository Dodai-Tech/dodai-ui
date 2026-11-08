//! Boxes that are only a token-styled surface: cards, alerts, chips, avatars, code, tooltips,
//! rows, steps, skeletons. One component ([`UiKit`]) and one system restyle all of them from
//! the active theme, so they re-skin like every other widget.

use bevy::prelude::*;
use dodai_core::{CardVariant, Color as Rgba, ItemState, StepState, Theme, Tone};

use crate::surface::Surface;
use crate::theme::DodaiTheme;
use crate::widgets::{ControlEvent, ControlKind, ControlPointer, Foreground};

/// What a [`UiKit`] box is.
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
    /// Header row of a table.
    TableHead,
    /// Body row of a table (`true` = banded).
    TableRow(bool),
}

/// A token-styled box. Add [`UiPress`] to make it clickable.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface, Foreground, ControlPointer)]
pub struct UiKit(pub Kit);

/// Reports [`ControlKind::Pressed`] when the pointer goes down on the entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiPress;

/// How a [`Kit`] looks under a theme: pure data, no ECS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KitStyle {
    /// Fill (`None` = transparent).
    pub fill: Option<Rgba>,
    /// Outline.
    pub border: Option<Rgba>,
    /// Text colour inside.
    pub fg: Rgba,
    /// Glow halo.
    pub glow: bool,
    /// Fully round on rounded designs.
    pub pill: bool,
    /// Horizontal and vertical padding px.
    pub padding: (f32, f32),
}

/// Resolve `kit` against `theme`; `hovered` is the pointer state of clickable kits.
pub fn kit_style(theme: &Theme, kit: Kit, hovered: bool) -> KitStyle {
    let p = &theme.palette;
    let sp = &theme.spacing;
    let from = |t: dodai_core::SurfaceTokens, padding: (f32, f32), pill: bool| KitStyle {
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

pub(crate) fn style_kits(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiKit,
        &ControlPointer,
        &mut Surface,
        &mut Node,
        &mut Foreground,
    )>,
) {
    for (k, pointer, mut s, mut n, mut fg) in &mut q {
        let st = kit_style(&theme, k.0, *pointer != ControlPointer::None);
        let surface = Surface {
            fill: st.fill,
            border: st.border,
            glow: st.glow,
            pill: st.pill,
        };
        if *s != surface {
            *s = surface;
        }
        if fg.0 != st.fg {
            fg.0 = st.fg;
        }
        let pad = UiRect::axes(Val::Px(st.padding.0), Val::Px(st.padding.1));
        if n.padding != pad {
            n.padding = pad;
        }
    }
}

pub(crate) fn press_input(
    q: Query<(Entity, &ControlPointer), (Changed<ControlPointer>, With<UiPress>)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, p) in &q {
        if *p == ControlPointer::Pressed {
            out.write(ControlEvent {
                entity,
                kind: ControlKind::Pressed,
            });
        }
    }
}

/// Box of `kit` with the given content, sized by its content.
pub fn kit(kit: Kit) -> impl Bundle {
    (
        UiKit(kit),
        Node {
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            ..default()
        },
    )
}

/// A clickable box of `kit`.
pub fn kit_pressable(k: Kit) -> impl Bundle {
    (kit(k), UiPress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::themes;

    fn all_kits() -> Vec<Kit> {
        let mut v = vec![
            Kit::Avatar,
            Kit::Code,
            Kit::Snackbar,
            Kit::Skeleton,
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
                    let s = kit_style(&t, k, hover);
                    assert!(s.padding.0 >= 0.0 && s.padding.1 >= 0.0, "{} {k:?}", t.id());
                }
            }
        }
    }

    #[test]
    fn filled_kits_stay_legible() {
        for t in themes::all() {
            for k in all_kits() {
                let s = kit_style(&t, k, false);
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
            kit_style(&t, Kit::Chip { selected: true }, false),
            kit_style(&t, Kit::Chip { selected: false }, false)
        );
        assert_ne!(
            kit_style(&t, Kit::Row { selected: false }, true),
            kit_style(&t, Kit::Row { selected: false }, false)
        );
    }

    #[test]
    fn elevated_card_differs_from_outlined() {
        for t in themes::all() {
            assert_ne!(
                kit_style(&t, Kit::Card(CardVariant::Elevated), false),
                kit_style(&t, Kit::Card(CardVariant::Outlined), false),
                "{}",
                t.id()
            );
        }
    }
}
