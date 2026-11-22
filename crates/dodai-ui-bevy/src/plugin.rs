//! [`DodaiUiPlugin`]: wires resources, materials and systems.

use bevy::prelude::*;
use bevy::ui_render::prelude::{MaterialNode, UiMaterialPlugin};
use dodai_ui_core::ThemeSwitcher;

use crate::material::{load_shaders, ChamferMaterial, ScanlineMaterial};
use crate::surface::apply_surfaces;
use crate::systems::*;
use crate::theme::{color, DodaiTheme};
use crate::widgets::ControlEvent;

/// Add after `DefaultPlugins`/`UiPlugin` + render. Inserts [`DodaiTheme`]
/// (unless you inserted one), enables hot theme switching, spawns an optional
/// scanline overlay and a tooltip popup.
#[derive(Debug, Clone)]
pub struct DodaiUiPlugin {
    /// Spawn the scanline overlay (shown only while the theme's
    /// `Effects::scanlines` is on).
    pub scanlines: bool,
    /// Custom theme list; `None` = every built-in theme.
    pub switcher: Option<ThemeSwitcher>,
}

impl Default for DodaiUiPlugin {
    fn default() -> Self {
        Self {
            scanlines: true,
            switcher: None,
        }
    }
}

impl DodaiUiPlugin {
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
pub struct DodaiSet;

impl Plugin for DodaiUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<DodaiTheme>() {
            let t = match &self.switcher {
                Some(s) => DodaiTheme::with_switcher(s.clone()),
                None => DodaiTheme::new(),
            };
            app.insert_resource(t);
        }
        load_shaders(app);
        app.add_plugins((
            UiMaterialPlugin::<ChamferMaterial>::default(),
            UiMaterialPlugin::<ScanlineMaterial>::default(),
        ))
        .add_observer(pointer_over)
        .add_observer(pointer_out)
        .add_observer(pointer_press)
        .add_observer(pointer_release)
        .add_observer(crate::gizmo::gizmo_press)
        .add_observer(crate::gizmo::gizmo_drag)
        .add_observer(crate::gizmo::gizmo_click)
        .add_message::<crate::gizmo::GizmoEvent>()
        .add_systems(
            Update,
            (
                crate::gizmo::style_gizmo_roots,
                crate::gizmo::style_gizmo_parts,
            )
                .in_set(DodaiSet),
        )
        .init_resource::<FocusedInput>()
        .init_resource::<crate::glass::DodaiGlass>()
        .add_systems(
            PostUpdate,
            crate::glass::frost_layers.after(bevy::ui::UiSystems::Layout),
        )
        .add_message::<ControlEvent>()
        .add_message::<crate::menu::MenuEvent>()
        .add_systems(
            Update,
            (crate::menu::build_bars, crate::menu::drive_menus)
                .chain()
                .after(button_input),
        )
        .add_systems(Startup, spawn_tooltip)
        .add_systems(
            Update,
            (
                (
                    button_input,
                    crate::kit::press_input,
                    crate::knob::knob_input,
                    toggle_input,
                    radio_exclusive,
                    tab_input,
                    tab_edit_input,
                    tab_keys,
                    list_input,
                    slider_input,
                    input_focus,
                    input_keys,
                ),
                (
                    style_buttons,
                    style_ticks,
                    style_panels,
                    style_tags,
                    style_tabs,
                    style_toggles,
                    style_progress,
                    style_sliders,
                    style_inputs,
                    style_list_items,
                    style_dividers,
                    crate::kit::style_kits,
                    crate::knob::style_knobs,
                    crate::spinner::build_spinners,
                    crate::spinner::animate_spinners,
                    crate::spinner::animate_bars,
                    (
                        crate::spinner::animate_styled_bars,
                        crate::spinner::animate_wavy,
                    ),
                    update_tooltip,
                    #[cfg(feature = "icons")]
                    style_icons,
                    #[cfg(feature = "symbols")]
                    style_symbols,
                ),
                (
                    style_text,
                    crate::glass::sync_backdrop,
                    apply_surfaces,
                    crate::shimmer::animate_shimmer,
                    sync_clear_color,
                    sync_scanlines,
                )
                    .chain(),
            )
                .chain()
                .in_set(DodaiSet),
        );
        #[cfg(feature = "icons")]
        app.init_resource::<IconCache>();
        #[cfg(feature = "symbols")]
        app.init_resource::<SymbolCache>();
        if self.scanlines {
            app.add_systems(Startup, spawn_scanlines);
        }
    }
}

fn sync_clear_color(theme: Res<DodaiTheme>, mut clear: ResMut<ClearColor>) {
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
    theme: Res<DodaiTheme>,
    mut q: Query<(&mut Node, &MaterialNode<ScanlineMaterial>), With<ScanlineOverlay>>,
    mut mats: ResMut<Assets<ScanlineMaterial>>,
) {
    if !theme.is_changed() {
        return;
    }
    for (mut n, m) in &mut q {
        let on = theme.effects.scanlines;
        n.display = if on { Display::Flex } else { Display::None };
        if let Some(mut mat) = mats.get_mut(&m.0) {
            mat.color = color(theme.palette.background).to_linear().with_alpha(0.28);
        }
    }
}

/// Optional keyboard shortcuts: `t`/`T` next/previous theme, `+`/`-` glow.
/// Ignored while a text input has focus.
pub fn theme_hotkeys(
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<FocusedInput>,
    mut theme: ResMut<DodaiTheme>,
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
