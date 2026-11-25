//! Theme pickers: a dropdown of every theme, a previous/next bar and a row of design systems.
//! They read and write the [`DodaiTheme`] resource.

#![allow(clippy::type_complexity)]

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;

use crate::interact::{UiSelectGroup, UiSelectable};
use crate::kit::{kit, kit_pressable, Kit};
use crate::overlay::dropdown_with;
use crate::parts::{glyph, row, Glyph};
use crate::theme::DodaiTheme;
use crate::widgets::{text, ControlPointer, TextRole, UiText};

type Spawner<'a> = ChildSpawner<'a>;

/// A list that switches the theme to the entry picked (the index into the switcher).
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiThemePick;

/// A dropdown of every theme by name; picking one switches to it.
pub fn theme_selector(theme: &DodaiTheme) -> impl Bundle {
    let names: Vec<String> = theme
        .switcher()
        .themes()
        .iter()
        .map(|t| t.meta.name.to_string())
        .collect();
    dropdown_with(theme.theme().meta.name.to_string(), names, UiThemePick)
}

/// The name of the current theme between previous and next buttons.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiThemeBar;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ThemeStep(pub i32);

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ThemeName;

/// Previous and next buttons around the current theme's name.
pub fn theme_switcher_bar(theme: &DodaiTheme) -> impl Bundle {
    let name = theme.theme().meta.name.to_string();
    (
        UiThemeBar,
        kit(Kit::Toolbar {
            floating: false,
            vibrant: false,
        }),
        Node {
            align_self: AlignSelf::FlexStart,
            column_gap: Val::Px(12.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                ThemeStep(-1),
                children![glyph(Glyph::ChevronLeft, 16)],
            ));
            c.spawn((
                ThemeName,
                Node {
                    min_width: Val::Px(140.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                text(name, TextRole::Control),
            ));
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                ThemeStep(1),
                children![glyph(Glyph::ChevronRight, 16)],
            ));
        })),
    )
}

/// A row of chips, one per design system; picking one switches to its first theme.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiDesignSelector;

/// The design system a chip of a [`design_system_selector`] stands for.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub(crate) struct FamilyChip(pub String);

/// A chip per design system (family) of the switcher.
pub fn design_system_selector(theme: &DodaiTheme) -> impl Bundle {
    let families: Vec<String> = theme
        .switcher()
        .families()
        .into_iter()
        .map(str::to_string)
        .collect();
    let current = theme.switcher().current_family();
    let selected = families.iter().position(|f| f == current);
    (
        UiDesignSelector,
        UiSelectGroup { selected },
        row(8.0),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, f) in families.into_iter().enumerate() {
                c.spawn((
                    kit_pressable(Kit::Chip {
                        selected: Some(i) == selected,
                    }),
                    UiSelectable(i),
                    FamilyChip(f.clone()),
                    children![text(f, TextRole::ControlSmall)],
                ));
            }
        })),
    )
}

pub(crate) fn pick_theme(
    groups: Query<&UiSelectGroup, (With<UiThemePick>, Changed<UiSelectGroup>)>,
    mut theme: ResMut<DodaiTheme>,
) {
    for g in &groups {
        if let Some(i) = g.selected {
            theme.select_index(i);
        }
    }
}

pub(crate) fn step_theme(
    pressed: Query<(&ThemeStep, &ControlPointer), Changed<ControlPointer>>,
    mut theme: ResMut<DodaiTheme>,
) {
    for (s, p) in &pressed {
        if *p != ControlPointer::Pressed {
            continue;
        }
        if s.0 > 0 {
            theme.cycle_next();
        } else {
            theme.cycle_prev();
        }
    }
}

pub(crate) fn pick_family(
    pressed: Query<(&FamilyChip, &ControlPointer), Changed<ControlPointer>>,
    mut theme: ResMut<DodaiTheme>,
) {
    for (f, p) in &pressed {
        if *p == ControlPointer::Pressed && theme.switcher().current_family() != f.0 {
            theme.select_family(&f.0);
        }
    }
}

/// Keeps the names and the design chips in line with the theme (it can change from a hotkey).
pub(crate) fn sync_theme_widgets(
    theme: Res<DodaiTheme>,
    mut names: Query<&mut UiText, With<ThemeName>>,
    mut groups: Query<(&mut UiSelectGroup, &Children), With<UiDesignSelector>>,
    mut chips: Query<(&FamilyChip, &mut crate::kit::UiKit)>,
) {
    if !theme.is_changed() {
        return;
    }
    let name = theme.theme().meta.name;
    for mut n in &mut names {
        if n.raw != name {
            n.raw = name.to_string();
        }
    }
    let family = theme.switcher().current_family();
    for (mut g, kids) in &mut groups {
        let mut idx = None;
        for (i, k) in kids.iter().enumerate() {
            if let Ok((f, mut kit)) = chips.get_mut(k) {
                let on = f.0 == family;
                if on {
                    idx = Some(i);
                }
                let next = Kit::Chip { selected: on };
                if kit.0 != next {
                    kit.0 = next;
                }
            }
        }
        if g.selected != idx {
            g.selected = idx;
        }
    }
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_systems(
        Update,
        (pick_theme, step_theme, pick_family, sync_theme_widgets)
            .chain()
            .after(crate::kit::press_input),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        crate::testing::component_app()
    }

    fn press(app: &mut App, e: Entity) {
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Hovered;
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Pressed;
    }

    #[test]
    fn the_bar_steps_the_theme_and_shows_its_name() {
        let mut app = app();
        let t = app.world().resource::<DodaiTheme>().clone();
        let first = t.theme().meta.id;
        let bar = app.world_mut().spawn(theme_switcher_bar(&t)).id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(bar).unwrap().iter().collect();
        press(&mut app, kids[2]);
        app.update();
        app.update();
        let now = app.world().resource::<DodaiTheme>().theme().meta;
        assert_ne!(now.id, first);
        assert_eq!(app.world().get::<UiText>(kids[1]).unwrap().raw, now.name);
    }

    #[test]
    fn the_selector_lists_every_theme_and_picks_one() {
        let mut app = app();
        let t = app.world().resource::<DodaiTheme>().clone();
        let n = t.switcher().themes().len();
        let sel = app.world_mut().spawn(theme_selector(&t)).id();
        app.update();
        let parts: Vec<Entity> = app.world().get::<Children>(sel).unwrap().iter().collect();
        assert_eq!(app.world().get::<Children>(parts[1]).unwrap().len(), n);
        let item = app.world().get::<Children>(parts[1]).unwrap()[3];
        press(&mut app, item);
        app.update();
        app.update();
        assert_eq!(app.world().resource::<DodaiTheme>().switcher().index(), 3);
    }

    #[test]
    fn design_chips_switch_family_and_follow_hotkey_changes() {
        let mut app = app();
        let t = app.world().resource::<DodaiTheme>().clone();
        let families = t.switcher().families();
        assert!(families.len() > 1, "built-in designs");
        let sel = app.world_mut().spawn(design_system_selector(&t)).id();
        app.update();
        let chips: Vec<Entity> = app.world().get::<Children>(sel).unwrap().iter().collect();
        assert_eq!(chips.len(), families.len());
        press(&mut app, chips[1]);
        app.update();
        app.update();
        assert_eq!(
            app.world()
                .resource::<DodaiTheme>()
                .switcher()
                .current_family(),
            families[1]
        );
        assert_eq!(
            app.world().get::<UiSelectGroup>(sel).unwrap().selected,
            Some(1)
        );
        // a change from elsewhere moves the selection
        app.world_mut()
            .resource_mut::<DodaiTheme>()
            .select_family(families[0]);
        app.update();
        assert_eq!(
            app.world().get::<UiSelectGroup>(sel).unwrap().selected,
            Some(0)
        );
    }
}
