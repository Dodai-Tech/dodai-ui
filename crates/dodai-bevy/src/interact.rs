//! Behaviour shared by the composite components: groups where one item is selected, sections that
//! open and close, steppers and ratings. One plugin system each; the constructors in the
//! component modules only mark the entities.

#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::kit::{Kit, UiKit};
use crate::parts::{Glyph, GlyphSlot};
use crate::widgets::{ControlEvent, ControlKind, ControlPointer, UiText};

/// A parent whose [`UiSelectable`] children behave as one choice: pressing one selects it and
/// clears the others.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiSelectGroup {
    /// The selected index.
    pub selected: Option<usize>,
}

/// One choice of a [`UiSelectGroup`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiSelectable(pub usize);

/// A header that opens and closes the [`DisclosureBody`] next to it (same parent).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiDisclosure {
    /// The body is shown.
    pub open: bool,
}

/// The part a [`UiDisclosure`] header shows or hides.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DisclosureBody;

/// Number with minus and plus buttons.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiStepper {
    /// Current value.
    pub value: f32,
    /// Lowest value.
    pub min: f32,
    /// Highest value.
    pub max: f32,
    /// Change per press.
    pub step: f32,
}

impl UiStepper {
    /// `value` moved by `dir` steps, clamped.
    pub fn stepped(&self, dir: f32) -> f32 {
        (self.value + dir * self.step).clamp(self.min, self.max)
    }
}

/// A button of a [`UiStepper`]: `-1.0` or `1.0`.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct StepperButton(pub f32);

/// The readout of a [`UiStepper`].
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct StepperReadout;

/// Row of stars; pressing one sets the value.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiRating {
    /// Current value, `0..=max`.
    pub value: u8,
    /// Number of stars.
    pub max: u8,
}

/// Star `n` (1-based) of a [`UiRating`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RatingStar(pub u8);

/// The kit with its selected flag set (kinds without one are returned as they are).
pub fn with_selected(k: Kit, on: bool) -> Kit {
    match k {
        Kit::Row { .. } => Kit::Row { selected: on },
        Kit::Chip { .. } => Kit::Chip { selected: on },
        Kit::Page { .. } => Kit::Page { selected: on },
        Kit::Segment { .. } => Kit::Segment { selected: on },
        other => other,
    }
}

pub(crate) fn select_groups(
    pressed: Query<(Entity, &UiSelectable, &ChildOf, &ControlPointer), Changed<ControlPointer>>,
    mut groups: Query<(&mut UiSelectGroup, &Children)>,
    mut items: Query<(&UiSelectable, &mut UiKit)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, sel, parent, pointer) in &pressed {
        if *pointer != ControlPointer::Pressed {
            continue;
        }
        let Ok((mut group, kids)) = groups.get_mut(parent.parent()) else {
            continue;
        };
        group.selected = Some(sel.0);
        for kid in kids.iter() {
            if let Ok((s, mut k)) = items.get_mut(kid) {
                let next = with_selected(k.0, s.0 == sel.0);
                if k.0 != next {
                    k.0 = next;
                }
            }
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Selected,
        });
    }
}

pub(crate) fn disclosures(
    mut headers: Query<
        (Entity, &mut UiDisclosure, &ControlPointer, &ChildOf),
        Changed<ControlPointer>,
    >,
    kids: Query<&Children>,
    mut bodies: Query<&mut Node, With<DisclosureBody>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, mut d, pointer, parent) in &mut headers {
        if *pointer != ControlPointer::Pressed {
            continue;
        }
        d.open = !d.open;
        if let Ok(siblings) = kids.get(parent.parent()) {
            for s in siblings.iter() {
                if let Ok(mut n) = bodies.get_mut(s) {
                    n.display = if d.open { Display::Flex } else { Display::None };
                }
            }
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Toggled(d.open),
        });
    }
}

/// Chevrons follow the open state of the section they sit in.
pub(crate) fn disclosure_chevrons(
    headers: Query<(&UiDisclosure, &Children), Changed<UiDisclosure>>,
    mut slots: Query<&mut GlyphSlot>,
) {
    for (d, kids) in &headers {
        for k in kids.iter() {
            if let Ok(mut s) = slots.get_mut(k) {
                let want = if d.open {
                    Glyph::ChevronDown
                } else {
                    Glyph::ChevronRight
                };
                if s.0 != want {
                    s.0 = want;
                }
            }
        }
    }
}

pub(crate) fn steppers(
    pressed: Query<(&StepperButton, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    mut steppers: Query<(Entity, &mut UiStepper, &Children)>,
    mut readouts: Query<&mut UiText, With<StepperReadout>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (button, pointer, parent) in &pressed {
        if *pointer != ControlPointer::Pressed {
            continue;
        }
        let Ok((entity, mut s, kids)) = steppers.get_mut(parent.parent()) else {
            continue;
        };
        let next = s.stepped(button.0);
        if next == s.value {
            continue;
        }
        s.value = next;
        let shown = format_value(next, s.step);
        for k in kids.iter() {
            if let Ok(mut t) = readouts.get_mut(k) {
                t.raw = shown.clone();
            }
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Slid(next),
        });
    }
}

/// `v` without a fraction when `step` is whole.
pub fn format_value(v: f32, step: f32) -> String {
    if step.fract() == 0.0 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.2}")
    }
}

pub(crate) fn ratings(
    pressed: Query<(&RatingStar, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    mut ratings: Query<(Entity, &mut UiRating)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (star, pointer, parent) in &pressed {
        if *pointer != ControlPointer::Pressed {
            continue;
        }
        let Ok((entity, mut r)) = ratings.get_mut(parent.parent()) else {
            continue;
        };
        // Pressing the current value clears one step, like the other toolkits.
        let next = if r.value == star.0 {
            star.0 - 1
        } else {
            star.0
        };
        r.value = next;
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Slid(f32::from(next)),
        });
    }
}

pub(crate) fn sync_stars(
    ratings: Query<(&UiRating, &Children), Changed<UiRating>>,
    mut stars: Query<(&RatingStar, &mut crate::parts::StarFill)>,
) {
    for (r, kids) in &ratings {
        for k in kids.iter() {
            if let Ok((s, mut f)) = stars.get_mut(k) {
                let on = s.0 <= r.value;
                if f.0 != on {
                    f.0 = on;
                }
            }
        }
    }
}

#[cfg(feature = "icons")]
pub(crate) fn draw_stars(
    mut q: Query<
        (&crate::parts::StarFill, &mut crate::widgets::UiIcon),
        Changed<crate::parts::StarFill>,
    >,
) {
    for (f, mut i) in &mut q {
        if i.filled != f.0 {
            i.filled = f.0;
        }
    }
}

#[cfg(not(feature = "icons"))]
pub(crate) fn draw_stars(
    mut q: Query<(&crate::parts::StarFill, &mut UiText), Changed<crate::parts::StarFill>>,
) {
    for (f, mut t) in &mut q {
        let want = if f.0 { "*" } else { "-" };
        if t.raw != want {
            t.raw = want.to_string();
        }
    }
}

#[cfg(feature = "icons")]
pub(crate) fn sync_glyphs(
    mut q: Query<(&GlyphSlot, &mut crate::widgets::UiIcon), Changed<GlyphSlot>>,
) {
    for (g, mut i) in &mut q {
        if i.icon != g.0.id() {
            i.icon = g.0.id();
        }
    }
}

#[cfg(not(feature = "icons"))]
pub(crate) fn sync_glyphs(_q: Query<&GlyphSlot, Changed<GlyphSlot>>) {}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_systems(
        Update,
        (
            select_groups,
            disclosures,
            disclosure_chevrons,
            steppers,
            ratings,
            sync_stars,
            draw_stars,
            sync_glyphs,
        )
            .chain()
            .after(crate::kit::press_input),
    );
}
