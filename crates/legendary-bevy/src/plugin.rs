//! [`LegendaryUiPlugin`]: wires resources, materials and systems.

use bevy::prelude::*;
use bevy::ui_render::prelude::{MaterialNode, UiMaterialPlugin};
use legendary_core::ThemeSwitcher;

use crate::material::{load_shaders, ChamferMaterial, ScanlineMaterial};
use crate::surface::apply_surfaces;
use crate::systems::*;
use crate::theme::{color, LegendaryTheme};
use crate::widgets::ControlEvent;

/// Add after `DefaultPlugins`/`UiPlugin` + render. Inserts [`LegendaryTheme`]
/// (unless you inserted one), enables hot theme switching, spawns an optional
/// scanline overlay and a tooltip popup.
#[derive(Debug, Clone)]
pub struct LegendaryUiPlugin {
    /// Spawn the scanline overlay (shown only while the theme's
    /// `Effects::scanlines` is on).
    pub scanlines: bool,
    /// Custom theme list; `None` = every built-in theme.
    pub switcher: Option<ThemeSwitcher>,
}

impl Default for LegendaryUiPlugin {
    fn default() -> Self {
        Self {
            scanlines: true,
            switcher: None,
        }
    }
}

impl LegendaryUiPlugin {
    /// Use a custom theme list.
    pub fn with_switcher(mut self, switcher: ThemeSwitcher) -> Self {
        self.switcher = Some(switcher);
        self
    }

    /// Enable/disable the scanline overlay.
    pub fn with_scanlines(mut self, on: bool) -> Self {
        self.scanlines = on;
        self
    }
}

/// Marker of the full-screen scanline overlay.
#[derive(Component)]
pub struct ScanlineOverlay;

/// System set containing all input and styling systems.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct LegendarySet;

impl Plugin for LegendaryUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<LegendaryTheme>() {
            let t = match &self.switcher {
                Some(s) => LegendaryTheme::with_switcher(s.clone()),
                None => LegendaryTheme::new(),
            };
            app.insert_resource(t);
        }
        load_shaders(app);
        app.add_plugins((
            UiMaterialPlugin::<ChamferMaterial>::default(),
            UiMaterialPlugin::<ScanlineMaterial>::default(),
        ))
        .init_resource::<FocusedInput>()
        .add_message::<ControlEvent>()
        .add_systems(Startup, spawn_tooltip)
        .add_systems(
            Update,
            (
                (
                    button_input,
                    toggle_input,
                    radio_exclusive,
                    tab_input,
                    list_input,
                    slider_input,
                    input_focus,
                    input_keys,
                ),
                (
                    style_buttons,
                    style_panels,
                    style_tags,
                    style_tabs,
                    style_toggles,
                    style_progress,
                    style_sliders,
                    style_inputs,
                    style_list_items,
                    style_dividers,
                    update_tooltip,
                ),
                (style_text, apply_surfaces, sync_clear_color, sync_scanlines).chain(),
            )
                .chain()
                .in_set(LegendarySet),
        );
        if self.scanlines {
            app.add_systems(Startup, spawn_scanlines);
        }
    }
}

fn sync_clear_color(theme: Res<LegendaryTheme>, mut clear: ResMut<ClearColor>) {
    if theme.is_changed() {
        clear.0 = color(theme.palette.background);
    }
}

fn spawn_scanlines(mut commands: Commands, mut mats: ResMut<Assets<ScanlineMaterial>>) {
    commands.spawn((
        ScanlineOverlay,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            display: Display::None,
            ..default()
        },
        MaterialNode(mats.add(ScanlineMaterial {
            color: LinearRgba::BLACK,
            params: Vec4::new(3.0, 0.34, 0.0, 0.0),
        })),
        GlobalZIndex(1000),
    ));
}

fn sync_scanlines(
    theme: Res<LegendaryTheme>,
    mut q: Query<(&mut Node, &MaterialNode<ScanlineMaterial>), With<ScanlineOverlay>>,
    mut mats: ResMut<Assets<ScanlineMaterial>>,
) {
    if !theme.is_changed() {
        return;
    }
    for (mut n, m) in &mut q {
        let on = theme.effects.scanlines;
        n.display = if on { Display::Flex } else { Display::None };
        if let Some(mat) = mats.get_mut(&m.0) {
            mat.color = color(theme.palette.background).to_linear().with_alpha(0.28);
        }
    }
}

/// Optional keyboard shortcuts: `t`/`T` next/previous theme, `+`/`-` glow.
/// Ignored while a text input has focus.
pub fn theme_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<FocusedInput>,
    mut theme: ResMut<LegendaryTheme>,
) {
    if focus.0.is_some() {
        return;
    }
    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    if keys.just_pressed(KeyCode::KeyT) {
        if shift {
            theme.cycle_prev();
        } else {
            theme.cycle_next();
        }
    }
    let g = theme.glow_intensity();
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        theme.set_glow_intensity(g + 0.25);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        theme.set_glow_intensity((g - 0.25).max(0.0));
    }
}
