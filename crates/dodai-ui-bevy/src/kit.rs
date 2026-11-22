//! Boxes that are only a token-styled surface: cards, alerts, chips, avatars, code, tooltips,
//! rows, steps, skeletons. One component ([`UiKit`]) and one system restyle all of them from
//! the active theme, so they re-skin like every other widget.

use bevy::prelude::*;

use crate::surface::Surface;
use crate::theme::DodaiTheme;
use crate::widgets::{ControlEvent, ControlKind, ControlPointer, Foreground};

pub use dodai_ui_core::{Kit, KitStyle};

/// A token-styled box. Add [`UiPress`] to make it clickable.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[require(Surface, Foreground, ControlPointer)]
pub struct UiKit(pub Kit);

/// Reports [`ControlKind::Pressed`] when the pointer goes down on the entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiPress;

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
        let st = theme.kit(k.0, *pointer != ControlPointer::None);
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
        let align = if matches!(k.0, Kit::Card(_)) {
            AlignItems::Stretch
        } else {
            AlignItems::Center
        };
        if n.align_items != align {
            n.align_items = align;
        }
        let gap = Val::Px(theme.spacing.sm);
        if n.column_gap != gap {
            n.column_gap = gap;
        }
    }
}

type PointerChanged = (Changed<ControlPointer>, With<UiPress>);

pub(crate) fn press_input(
    q: Query<(Entity, &ControlPointer), PointerChanged>,
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

/// A box of `kit`. Give it its own [`Node`] for size and direction; alignment (centred, cards
/// stretch) and the gap between children come from the theme.
pub fn kit(kit: Kit) -> impl Bundle {
    (UiKit(kit), glass_of(kit))
}

/// A clickable box of `kit`.
pub fn kit_pressable(k: Kit) -> impl Bundle {
    (UiKit(k), UiPress, glass_of(k))
}

/// The surface kind of a `kit` box for the glass.
fn glass_of(kit: Kit) -> crate::glass::UiGlass {
    crate::glass::UiGlass {
        class: kit.surface_class(),
        over: dodai_ui_core::Surface::INHERIT,
        floating: kit.floats(),
    }
}
