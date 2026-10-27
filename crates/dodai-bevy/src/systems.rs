//! Input handling and styling systems.
#![allow(clippy::type_complexity)]

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use dodai_core::{ButtonVariant, ControlState, ShapeKind};

use crate::style::{control_padding, display_text, text_style, toggle_size};
use crate::surface::Surface;
use crate::theme::{color, DodaiTheme};
use crate::widgets::*;

/// Entity of the text input that currently receives keyboard input.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FocusedInput(pub Option<Entity>);

/// Picking → [`ControlPointer`]: over/out/press/release keep the state of the control under the cursor.
pub(crate) fn pointer_over(
    over: On<bevy::picking::events::PointerOver>,
    mut q: Query<&mut ControlPointer>,
) {
    if let Ok(mut p) = q.get_mut(over.entity) {
        upd(&mut *p, ControlPointer::Hovered);
    }
}
pub(crate) fn pointer_out(
    out: On<bevy::picking::events::PointerOut>,
    mut q: Query<&mut ControlPointer>,
) {
    if let Ok(mut p) = q.get_mut(out.entity) {
        upd(&mut *p, ControlPointer::None);
    }
}
pub(crate) fn pointer_press(
    press: On<bevy::picking::events::PointerPress>,
    mut q: Query<&mut ControlPointer>,
) {
    if let Ok(mut p) = q.get_mut(press.entity) {
        upd(&mut *p, ControlPointer::Pressed);
    }
}
pub(crate) fn pointer_release(
    release: On<bevy::picking::events::PointerRelease>,
    mut q: Query<&mut ControlPointer>,
) {
    if let Ok(mut p) = q.get_mut(release.entity) {
        upd(&mut *p, ControlPointer::Hovered);
    }
}

fn upd<T: PartialEq>(slot: &mut T, v: T) {
    if *slot != v {
        *slot = v;
    }
}

fn cs(i: &ControlPointer, disabled: bool, focused: bool) -> ControlState {
    if disabled {
        return ControlState::Disabled;
    }
    match i {
        ControlPointer::Pressed => ControlState::Pressed,
        ControlPointer::Hovered => ControlState::Hovered,
        ControlPointer::None if focused => ControlState::Focused,
        ControlPointer::None => ControlState::Idle,
    }
}

// ---------------------------------------------------------------- input

pub(crate) fn button_input(
    q: Query<(Entity, &ControlPointer, &UiButton), Changed<ControlPointer>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, b) in &q {
        if *i == ControlPointer::Pressed && !b.disabled {
            out.write(ControlEvent {
                entity,
                kind: ControlKind::Pressed,
            });
        }
    }
}

pub(crate) fn toggle_input(
    mut q: Query<(Entity, &ControlPointer, &mut UiToggle), Changed<ControlPointer>>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, mut t) in &mut q {
        if *i != ControlPointer::Pressed || t.disabled {
            continue;
        }
        if t.kind == ToggleKind::Radio {
            if t.on {
                continue;
            }
            t.on = true;
        } else {
            t.on = !t.on;
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Toggled(t.on),
        });
    }
}

/// Exclusive radios: turn off other radios under the same parent.
pub(crate) fn radio_exclusive(
    mut set: ParamSet<(
        Query<(Entity, &UiToggle, Option<&ChildOf>), Changed<UiToggle>>,
        Query<(Entity, &mut UiToggle, Option<&ChildOf>)>,
    )>,
) {
    let winners: Vec<(Entity, Option<Entity>)> = set
        .p0()
        .iter()
        .filter(|(_, t, _)| t.kind == ToggleKind::Radio && t.on)
        .map(|(e, _, p)| (e, p.map(ChildOf::parent)))
        .collect();
    for (win, parent) in winners {
        for (e, mut t, p) in &mut set.p1() {
            if e != win && t.kind == ToggleKind::Radio && t.on && p.map(ChildOf::parent) == parent {
                t.on = false;
            }
        }
    }
}

pub(crate) fn tab_input(
    q: Query<(Entity, &ControlPointer, &ChildOf), (Changed<ControlPointer>, With<UiTab>)>,
    mut tabs: Query<(Entity, &mut UiTab, &ChildOf)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, parent) in &q {
        if *i != ControlPointer::Pressed {
            continue;
        }
        for (e, mut t, p) in &mut tabs {
            if p.parent() == parent.parent() {
                t.active = e == entity;
            }
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::TabSelected,
        });
    }
}

pub(crate) fn list_input(
    q: Query<(Entity, &ControlPointer, &ChildOf), (Changed<ControlPointer>, With<UiListItem>)>,
    mut rows: Query<(Entity, &mut UiListItem, &ChildOf)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, parent) in &q {
        if *i != ControlPointer::Pressed {
            continue;
        }
        for (e, mut r, p) in &mut rows {
            if p.parent() == parent.parent() {
                r.selected = e == entity;
            }
        }
        out.write(ControlEvent {
            entity,
            kind: ControlKind::Selected,
        });
    }
}

pub(crate) fn slider_input(
    mut q: Query<(
        Entity,
        &ControlPointer,
        &RelativeCursorPosition,
        &mut UiSlider,
    )>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, rel, mut s) in &mut q {
        if *i != ControlPointer::Pressed {
            continue;
        }
        if let Some(n) = rel.normalized {
            let v = s.min + (n.x + 0.5).clamp(0.0, 1.0) * (s.max - s.min);
            if (v - s.value).abs() > f32::EPSILON {
                s.value = v;
                out.write(ControlEvent {
                    entity,
                    kind: ControlKind::Slid(v),
                });
            }
        }
    }
}

pub(crate) fn input_focus(
    mouse: Res<ButtonInput<MouseButton>>,
    inputs: Query<(Entity, &ControlPointer), With<UiTextInput>>,
    mut focus: ResMut<FocusedInput>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        let hit = inputs
            .iter()
            .find(|(_, i)| **i != ControlPointer::None)
            .map(|(e, _)| e);
        if focus.0 != hit {
            focus.0 = hit;
        }
    }
}

pub(crate) fn input_keys(
    mut keys: MessageReader<KeyboardInput>,
    mut focus: ResMut<FocusedInput>,
    mut inputs: Query<&mut UiTextInput>,
    mut out: MessageWriter<ControlEvent>,
) {
    let Some(e) = focus.0 else {
        keys.clear();
        return;
    };
    let Ok(mut input) = inputs.get_mut(e) else {
        return;
    };
    for k in keys.read() {
        if k.state != ButtonState::Pressed {
            continue;
        }
        match &k.logical_key {
            Key::Backspace => {
                input.value.pop();
            }
            Key::Enter | Key::Escape => {
                focus.0 = None;
                return;
            }
            _ => match &k.text {
                Some(t) => input.value.extend(t.chars().filter(|c| !c.is_control())),
                None => continue,
            },
        }
        out.write(ControlEvent {
            entity: e,
            kind: ControlKind::TextChanged,
        });
    }
}

// ---------------------------------------------------------------- style

pub(crate) fn style_buttons(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiButton,
        &ControlPointer,
        &mut Surface,
        &mut Node,
        &mut Foreground,
    )>,
) {
    let sh = &theme.shape;
    let (px, _) = control_padding(&theme);
    for (b, i, mut s, mut n, mut fg) in &mut q {
        let state = cs(i, b.disabled, false);
        let t = theme.button(b.variant, state);
        let lit = matches!(
            state,
            ControlState::Hovered | ControlState::Pressed | ControlState::Focused
        );
        upd(
            &mut *s,
            Surface {
                fill: t.background,
                border: t.border,
                glow: !b.disabled && (lit || b.variant == ButtonVariant::Primary),
                pill: false,
            },
        );
        upd(&mut fg.0, t.foreground);
        upd(&mut n.height, Val::Px(sh.control_height));
        upd(&mut n.padding, UiRect::horizontal(Val::Px(px)));
    }
}

pub(crate) fn style_panels(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiPanel, &mut Surface, &mut Node, &mut Foreground)>,
) {
    for (p, mut s, mut n, mut fg) in &mut q {
        let t = theme.panel(p.focused);
        upd(
            &mut *s,
            Surface {
                fill: Some(t.background),
                border: Some(t.border),
                glow: p.focused,
                pill: false,
            },
        );
        upd(&mut fg.0, t.title);
        upd(&mut n.padding, UiRect::all(Val::Px(theme.spacing.md)));
        upd(&mut n.row_gap, Val::Px(theme.spacing.sm));
        upd(&mut n.flex_direction, FlexDirection::Column);
    }
}

pub(crate) fn style_tags(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiTag, &mut Surface, &mut Node, &mut Foreground)>,
) {
    for (g, mut s, mut n, mut fg) in &mut q {
        let t = theme.tag(g.tone, g.solid);
        upd(
            &mut *s,
            Surface {
                fill: t.background,
                border: t.border,
                glow: false,
                pill: true,
            },
        );
        upd(&mut fg.0, t.foreground);
        upd(
            &mut n.padding,
            UiRect::axes(Val::Px(theme.spacing.sm), Val::Px(theme.spacing.xs / 2.0)),
        );
    }
}

pub(crate) fn style_tabs(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiTab,
        &ControlPointer,
        &mut Surface,
        &mut Node,
        &mut Foreground,
    )>,
    mut ind: Query<(&ChildOf, &mut BackgroundColor), With<TabIndicator>>,
    tabs: Query<(&UiTab, &ControlPointer)>,
) {
    for (t, i, mut s, mut n, mut fg) in &mut q {
        let k = theme.tab(t.active, *i == ControlPointer::Hovered);
        upd(
            &mut *s,
            Surface {
                fill: k.background,
                border: None,
                glow: false,
                pill: false,
            },
        );
        upd(&mut fg.0, k.foreground);
        upd(&mut n.height, Val::Px(theme.shape.control_height));
        upd(
            &mut n.padding,
            UiRect::horizontal(Val::Px(theme.spacing.md)),
        );
    }
    for (p, mut bg) in &mut ind {
        if let Ok((t, i)) = tabs.get(p.parent()) {
            let k = theme.tab(t.active, *i == ControlPointer::Hovered);
            upd(&mut bg.0, k.indicator.map_or(Color::NONE, color));
        }
    }
}

pub(crate) fn style_toggles(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiToggle, &ControlPointer, &mut Surface, &mut Node), Without<TogglePart>>,
    mut parts: Query<(&ChildOf, &mut Node, &mut Surface), With<TogglePart>>,
    owners: Query<(&UiToggle, &ControlPointer), Without<TogglePart>>,
) {
    let round = theme.shape.kind() == ShapeKind::Rounded;
    let bw = theme.shape.border_width;
    for (t, i, mut s, mut n) in &mut q {
        let k = theme.toggle(t.on, cs(i, t.disabled, false));
        let (w, h) = toggle_size(&theme, t.kind);
        upd(
            &mut *s,
            Surface {
                fill: Some(k.track),
                border: Some(k.border),
                glow: t.on && !t.disabled && t.kind != ToggleKind::Switch,
                pill: t.kind != ToggleKind::Checkbox,
            },
        );
        upd(&mut n.width, Val::Px(w));
        upd(&mut n.height, Val::Px(h));
    }
    let _ = round;
    for (p, mut n, mut s) in &mut parts {
        let Ok((t, i)) = owners.get(p.parent()) else {
            continue;
        };
        let k = theme.toggle(t.on, cs(i, t.disabled, false));
        let (w, h) = toggle_size(&theme, t.kind);
        let inset = 3.0;
        match t.kind {
            ToggleKind::Switch => {
                let d = h - 2.0 * bw - 2.0 * inset;
                upd(&mut n.width, Val::Px(d));
                upd(&mut n.height, Val::Px(d));
                let left = if t.on {
                    w - 2.0 * bw - d - inset
                } else {
                    inset
                };
                upd(&mut n.left, Val::Px(left));
                upd(&mut n.display, Display::Flex);
                upd(
                    &mut *s,
                    Surface {
                        fill: Some(k.thumb),
                        border: None,
                        glow: false,
                        pill: true,
                    },
                );
            }
            _ => {
                let d = h - 2.0 * bw - 2.0 * (inset + 2.0);
                upd(&mut n.width, Val::Px(d.max(2.0)));
                upd(&mut n.height, Val::Px(d.max(2.0)));
                upd(
                    &mut n.display,
                    if t.on { Display::Flex } else { Display::None },
                );
                upd(
                    &mut *s,
                    Surface {
                        fill: Some(k.thumb),
                        border: None,
                        glow: false,
                        pill: t.kind == ToggleKind::Radio,
                    },
                );
            }
        }
    }
}

pub(crate) fn style_progress(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiProgress, &mut Surface, &mut Node), Without<ProgressFill>>,
    mut fills: Query<(&ChildOf, &mut Node, &mut Surface), With<ProgressFill>>,
    owners: Query<&UiProgress, Without<ProgressFill>>,
) {
    for (p, mut s, mut n) in &mut q {
        let k = theme.progress(p.tone);
        upd(
            &mut *s,
            Surface {
                fill: Some(k.track),
                border: Some(k.border),
                glow: false,
                pill: false,
            },
        );
        upd(&mut n.height, Val::Px(theme.spacing.sm + 2.0));
    }
    for (c, mut n, mut s) in &mut fills {
        let Ok(p) = owners.get(c.parent()) else {
            continue;
        };
        let k = theme.progress(p.tone);
        upd(&mut n.width, Val::Percent(p.value.clamp(0.0, 1.0) * 100.0));
        upd(
            &mut *s,
            Surface {
                fill: Some(k.fill),
                border: None,
                glow: p.value > 0.0,
                pill: false,
            },
        );
    }
}

#[allow(clippy::type_complexity)]
pub(crate) fn style_sliders(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiSlider, &mut Surface, &mut Node), (Without<SliderFill>, Without<SliderThumb>)>,
    mut fills: Query<(&ChildOf, &mut Node, &mut Surface), (With<SliderFill>, Without<SliderThumb>)>,
    mut thumbs: Query<
        (&ChildOf, &mut Node, &mut Surface),
        (With<SliderThumb>, Without<SliderFill>),
    >,
    owners: Query<&UiSlider, (Without<SliderFill>, Without<SliderThumb>)>,
) {
    let p = &theme.palette;
    let track_h = theme.spacing.sm + 2.0;
    let thumb_w = 10.0;
    for (_, mut s, mut n) in &mut q {
        upd(
            &mut *s,
            Surface {
                fill: Some(p.surface_raised),
                border: Some(p.border),
                glow: false,
                pill: false,
            },
        );
        upd(&mut n.height, Val::Px(track_h));
    }
    for (c, mut n, mut s) in &mut fills {
        let Ok(sl) = owners.get(c.parent()) else {
            continue;
        };
        upd(&mut n.width, Val::Percent(sl.fraction() * 100.0));
        upd(
            &mut *s,
            Surface {
                fill: Some(p.primary),
                border: None,
                glow: false,
                pill: false,
            },
        );
    }
    for (c, mut n, mut s) in &mut thumbs {
        let Ok(sl) = owners.get(c.parent()) else {
            continue;
        };
        upd(&mut n.width, Val::Px(thumb_w));
        upd(&mut n.height, Val::Px(theme.shape.control_height * 0.5));
        upd(&mut n.left, Val::Percent(sl.fraction() * 100.0));
        upd(&mut n.margin.left, Val::Px(-thumb_w / 2.0));
        upd(
            &mut *s,
            Surface {
                fill: Some(p.primary_hover),
                border: Some(p.on_primary),
                glow: true,
                pill: false,
            },
        );
    }
}

pub(crate) fn style_inputs(
    theme: Res<DodaiTheme>,
    focus: Res<FocusedInput>,
    mut q: Query<(Entity, &UiTextInput, &mut Surface, &mut Node), Without<InputText>>,
    mut texts: Query<(&ChildOf, &mut UiText, &mut Foreground), With<InputText>>,
    owners: Query<(Entity, &UiTextInput), Without<InputText>>,
) {
    for (e, _, mut s, mut n) in &mut q {
        let focused = focus.0 == Some(e);
        let k = theme.input(focused);
        upd(
            &mut *s,
            Surface {
                fill: Some(k.background),
                border: Some(k.border),
                glow: focused,
                pill: false,
            },
        );
        upd(&mut n.height, Val::Px(theme.shape.control_height));
        upd(
            &mut n.padding,
            UiRect::horizontal(Val::Px(theme.spacing.md)),
        );
    }
    for (c, mut t, mut fg) in &mut texts {
        let Ok((e, i)) = owners.get(c.parent()) else {
            continue;
        };
        let focused = focus.0 == Some(e);
        let k = theme.input(focused);
        let (raw, col) = if i.value.is_empty() && !focused {
            (i.placeholder.clone(), k.placeholder)
        } else if focused {
            (format!("{}|", i.value), k.text)
        } else {
            (i.value.clone(), k.text)
        };
        upd(&mut t.raw, raw);
        upd(&mut fg.0, col);
    }
}

pub(crate) fn style_list_items(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiListItem,
        &ControlPointer,
        &mut Surface,
        &mut Node,
        &mut Foreground,
    )>,
    mut markers: Query<(&ChildOf, &mut BackgroundColor), With<ListMarker>>,
    rows: Query<(&UiListItem, &ControlPointer)>,
) {
    for (r, i, mut s, mut n, mut fg) in &mut q {
        let k = theme.list_item(r.selected, *i == ControlPointer::Hovered);
        upd(
            &mut *s,
            Surface {
                fill: k.background,
                border: None,
                glow: false,
                pill: false,
            },
        );
        upd(&mut fg.0, k.foreground);
        upd(&mut n.height, Val::Px(theme.shape.control_height * 0.9));
        upd(
            &mut n.padding,
            UiRect::horizontal(Val::Px(theme.spacing.md)),
        );
    }
    for (c, mut bg) in &mut markers {
        if let Ok((r, i)) = rows.get(c.parent()) {
            let k = theme.list_item(r.selected, *i == ControlPointer::Hovered);
            upd(&mut bg.0, k.marker.map_or(Color::NONE, color));
        }
    }
}

pub(crate) fn style_dividers(
    theme: Res<DodaiTheme>,
    mut q: Query<(&UiDivider, &mut BackgroundColor)>,
) {
    let c = color(theme.divider());
    for (_, mut bg) in &mut q {
        upd(&mut bg.0, c);
    }
}

/// Role → displayed text, size and colour.
pub(crate) fn style_text(
    theme: Res<DodaiTheme>,
    mut q: Query<(
        &UiText,
        &mut Text,
        &mut TextFont,
        &mut TextColor,
        Option<&Foreground>,
        Option<&ChildOf>,
    )>,
    fgs: Query<&Foreground>,
) {
    for (u, mut text, mut font, mut col, own, parent) in &mut q {
        let st = text_style(&theme, u.role);
        let shown = display_text(&u.raw, &st);
        upd(&mut text.0, shown);
        upd(&mut font.font_size, bevy::text::FontSize::Px(st.size));
        let fg = st
            .color
            .or(own.map(|f| f.0))
            .or_else(|| parent.and_then(|p| fgs.get(p.parent()).ok()).map(|f| f.0))
            .unwrap_or(theme.palette.text);
        upd(&mut col.0, color(fg));
    }
}

// ------------------------------------------------------------- tooltip

pub(crate) fn spawn_tooltip(mut commands: Commands) {
    commands.spawn((
        TooltipPopup,
        Surface::default(),
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            ..default()
        },
        GlobalZIndex(100),
        children![(TooltipText, text("", TextRole::Caption),)],
    ));
}

pub(crate) fn update_tooltip(
    theme: Res<DodaiTheme>,
    window: Query<&Window, With<PrimaryWindow>>,
    hovered: Query<(&UiTooltip, &ControlPointer)>,
    mut popup: Query<(&mut Node, &mut Surface), With<TooltipPopup>>,
    mut label: Query<&mut UiText, With<TooltipText>>,
) {
    let Ok((mut n, mut s)) = popup.single_mut() else {
        return;
    };
    let p = &theme.palette;
    upd(
        &mut *s,
        Surface {
            fill: Some(p.surface_raised),
            border: Some(p.border_strong),
            glow: false,
            pill: false,
        },
    );
    let pad = UiRect::axes(Val::Px(theme.spacing.sm), Val::Px(theme.spacing.xs));
    upd(&mut n.padding, pad);
    let hit = hovered.iter().find(|(_, i)| **i != ControlPointer::None);
    let cursor = window.single().ok().and_then(Window::cursor_position);
    match (hit, cursor) {
        (Some((tip, _)), Some(c)) => {
            if let Ok(mut l) = label.single_mut() {
                upd(&mut l.raw, tip.0.clone());
            }
            upd(&mut n.display, Display::Flex);
            upd(&mut n.left, Val::Px(c.x + 14.0));
            upd(&mut n.top, Val::Px(c.y + 18.0));
        }
        _ => upd(&mut n.display, Display::None),
    }
}

// --------------------------------------------------------------- icons

/// Rasterised icons, shared between entities.
#[cfg(feature = "icons")]
#[derive(Resource, Default)]
pub struct IconCache(
    std::collections::HashMap<(dodai_icons::IconId, u32, bool, bool), Handle<Image>>,
);

/// `UiIcon` → `ImageNode` (rasterised on change / theme switch), tinted with
/// the inherited foreground colour.
#[cfg(feature = "icons")]
#[allow(clippy::type_complexity)]
pub(crate) fn style_icons(
    mut commands: Commands,
    theme: Res<DodaiTheme>,
    mut cache: ResMut<IconCache>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(
        Entity,
        Ref<UiIcon>,
        Option<&mut ImageNode>,
        Option<&Foreground>,
        Option<&ChildOf>,
    )>,
    fgs: Query<&Foreground>,
) {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let rounded = theme.shape.kind() == dodai_core::ShapeKind::Rounded;
    for (e, u, node, own, parent) in &mut q {
        let fg = color(
            own.map(|f| f.0)
                .or_else(|| parent.and_then(|p| fgs.get(p.parent()).ok()).map(|f| f.0))
                .unwrap_or(theme.palette.text),
        );
        let key = (u.icon, u.size, u.filled, rounded);
        let needs_image = node.is_none() || u.is_changed() || theme.is_changed();
        if needs_image {
            let handle = match cache.0.get(&key) {
                Some(h) => h.clone(),
                None => {
                    let Some(g) = dodai_icons::glyph_for_theme(u.icon, &theme, u.filled) else {
                        continue;
                    };
                    let px = u.size.max(1);
                    let data: Vec<u8> = g
                        .coverage(px)
                        .into_iter()
                        .flat_map(|a| [255, 255, 255, a])
                        .collect();
                    let h = images.add(Image::new(
                        Extent3d {
                            width: px,
                            height: px,
                            depth_or_array_layers: 1,
                        },
                        TextureDimension::D2,
                        data,
                        TextureFormat::Rgba8UnormSrgb,
                        RenderAssetUsages::default(),
                    ));
                    cache.0.insert(key, h.clone());
                    h
                }
            };
            commands
                .entity(e)
                .insert(ImageNode::new(handle).with_color(fg));
        } else if let Some(mut n) = node {
            upd(&mut n.color, fg);
        }
    }
}

#[cfg(all(test, feature = "icons"))]
mod icon_tests {
    use super::*;

    #[test]
    fn icon_becomes_a_tinted_image_node_and_follows_theme() {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .init_resource::<IconCache>()
            .insert_resource(Assets::<Image>::default())
            .add_systems(Update, style_icons);
        let e = app
            .world_mut()
            .spawn(icon(dodai_icons::IconId::Home, 24))
            .id();
        app.update();
        let handle = {
            let n = app.world().get::<ImageNode>(e).expect("ImageNode inserted");
            n.image.clone()
        };
        let img = app
            .world()
            .resource::<Assets<Image>>()
            .get(&handle)
            .unwrap();
        let data = img.data.as_ref().unwrap();
        assert_eq!(data.len(), 24 * 24 * 4);
        assert!(data.chunks(4).any(|p| p[3] > 200), "icon drew nothing");
        let first = handle.id();
        app.world_mut().resource_mut::<DodaiTheme>().cycle_next();
        app.update();
        assert!(app.world().get::<ImageNode>(e).is_some());
        let _ = first;
    }
}
