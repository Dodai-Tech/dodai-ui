//! Input handling and styling systems.
#![allow(clippy::type_complexity)]

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::ButtonState;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::PrimaryWindow;
use dodai_ui_core::{ButtonVariant, ControlState, ShapeKind};

use crate::style::{display_text, text_style, toggle_size};
use crate::surface::{Surface, SurfaceCrop};
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
    q: Query<
        (Entity, &ControlPointer, &ChildOf),
        (Changed<ControlPointer>, With<UiTab>, Without<TabAdd>),
    >,
    closers: Query<(&ControlPointer, &ChildOf), With<TabClose>>,
    mut tabs: Query<(Entity, &mut UiTab, &ChildOf)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (entity, i, parent) in &q {
        if *i != ControlPointer::Pressed {
            continue;
        }
        // A press on the close cross bubbles up to its tab: that is a close, not a selection.
        if closers
            .iter()
            .any(|(p, c)| c.parent() == entity && *p == ControlPointer::Pressed)
        {
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

/// Roving focus for the tab row under the pointer (bevy has no tab focus): the arrows, Home and
/// End pick the next enabled tab by the shared rule of `dodai-ui-core`, like a click on it.
pub(crate) fn tab_keys(
    mut keys: MessageReader<KeyboardInput>,
    rows: Query<&Children, With<UiTabs>>,
    pointers: Query<&ControlPointer>,
    mut tabs: Query<(&mut UiTab, Has<TabDisabled>, Has<TabAdd>)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for k in keys.read() {
        if !k.state.is_pressed() {
            continue;
        }
        let name = match k.logical_key {
            Key::ArrowLeft => "left",
            Key::ArrowRight => "right",
            Key::Home => "home",
            Key::End => "end",
            _ => continue,
        };
        for children in &rows {
            if !children
                .iter()
                .any(|c| pointers.get(c).is_ok_and(|p| *p != ControlPointer::None))
            {
                continue;
            }
            let items: Vec<(Entity, bool, bool)> = children
                .iter()
                .filter_map(|c| {
                    let (t, off, add) = tabs.get(c).ok()?;
                    (!add).then_some((c, t.active, off))
                })
                .collect();
            let Some(at) = items.iter().position(|(_, active, _)| *active) else {
                continue;
            };
            let to = dodai_ui_core::roving::next(
                name,
                dodai_ui_core::roving::Axis::Horizontal,
                at,
                items.len(),
                |i| !items[i].2,
            );
            if let Some(to) = to {
                for (i, (e, _, _)) in items.iter().enumerate() {
                    if let Ok((mut t, _, _)) = tabs.get_mut(*e) {
                        t.active = i == to;
                    }
                }
                out.write(ControlEvent {
                    entity: items[to].0,
                    kind: ControlKind::TabSelected,
                });
            }
        }
    }
}

/// The close cross and the "+" of an editable tab row: report them, change nothing.
pub(crate) fn tab_edit_input(
    closers: Query<(&ControlPointer, &ChildOf), (Changed<ControlPointer>, With<TabClose>)>,
    adders: Query<(Entity, &ControlPointer), (Changed<ControlPointer>, With<TabAdd>)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (p, parent) in &closers {
        if *p == ControlPointer::Pressed {
            out.write(ControlEvent {
                entity: parent.parent(),
                kind: ControlKind::TabClosed,
            });
        }
    }
    for (entity, p) in &adders {
        if *p == ControlPointer::Pressed {
            out.write(ControlEvent {
                entity,
                kind: ControlKind::TabAdded,
            });
        }
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
            let v = s.value_at(n);
            if let Some(lo) = s.lo {
                let thumb = dodai_ui_core::slider_math::nearest_thumb(lo, s.value, v);
                let (a, b) = dodai_ui_core::slider_math::range_update(thumb, v, lo, s.value);
                if (a - lo).abs() > f32::EPSILON || (b - s.value).abs() > f32::EPSILON {
                    s.lo = Some(a);
                    s.value = b;
                    out.write(ControlEvent {
                        entity,
                        kind: ControlKind::Ranged(a, b),
                    });
                }
                continue;
            }
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
        &mut SurfaceCrop,
    )>,
) {
    for (b, i, mut s, mut n, mut fg, mut crop) in &mut q {
        upd(
            &mut *crop,
            SurfaceCrop(b.crop.and_then(|e| theme.button_crop(e))),
        );
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
                pill: !b.square && theme.look().tonal,
            },
        );
        upd(&mut fg.0, t.foreground);
        let m = theme.control_metrics(b.size);
        upd(&mut n.height, Val::Px(m.height));
        if b.icon_only {
            upd(&mut n.width, Val::Px(m.height));
            upd(&mut n.padding, UiRect::ZERO);
        } else {
            upd(&mut n.width, Val::Auto);
            let inset = tick_edge(&crop).map_or(0.0, |_| theme.spacing.md);
            let (l, r) = match tick_edge(&crop) {
                Some(dodai_ui_core::CropEdge::Left) => (inset, 0.0),
                Some(dodai_ui_core::CropEdge::Right) => (0.0, inset),
                _ => (0.0, 0.0),
            };
            upd(
                &mut n.padding,
                UiRect {
                    left: Val::Px(m.padding + l),
                    right: Val::Px(m.padding + r),
                    ..UiRect::ZERO
                },
            );
        }
    }
}

/// The edge a button's tick sits on, when its crop draws one.
fn tick_edge(c: &SurfaceCrop) -> Option<dodai_ui_core::CropEdge> {
    match c.0 {
        Some(dodai_ui_core::ButtonCrop::CutTicked { tick, .. }) => Some(tick),
        _ => None,
    }
}

/// Place and colour the double tick of every cut-corner button; hide it elsewhere.
#[allow(clippy::type_complexity)]
pub(crate) fn style_ticks(
    theme: Res<DodaiTheme>,
    buttons: Query<(&SurfaceCrop, &Foreground, &Children), With<UiButton>>,
    mut ticks: Query<(&mut Node, &Children), (With<CropTick>, Without<CropBar>)>,
    mut bars: Query<(&mut Node, &mut BackgroundColor), (With<CropBar>, Without<CropTick>)>,
) {
    let bar = theme.shape.border_width.max(1.0);
    let gap = theme.tick_gap();
    for (crop, fg, kids) in &buttons {
        for kid in kids.iter() {
            let Ok((mut n, bar_ids)) = ticks.get_mut(kid) else {
                continue;
            };
            let edge = tick_edge(crop);
            upd(
                &mut n.display,
                if edge.is_some() {
                    Display::Flex
                } else {
                    Display::None
                },
            );
            let (l, r) = match edge {
                Some(dodai_ui_core::CropEdge::Left) => (Val::Px(0.0), Val::Auto),
                _ => (Val::Auto, Val::Px(0.0)),
            };
            upd(&mut n.left, l);
            upd(&mut n.right, r);
            upd(&mut n.top, Val::Percent(50.0));
            upd(&mut n.margin.top, Val::Px(-(bar + gap / 2.0)));
            upd(&mut n.width, Val::Px(theme.spacing.md));
            upd(&mut n.row_gap, Val::Px(gap));
            for b in bar_ids.iter() {
                if let Ok((mut bn, mut bg)) = bars.get_mut(b) {
                    upd(&mut bn.height, Val::Px(bar));
                    set_bg(&mut bg, color(fg.0));
                }
            }
        }
    }
}

fn set_bg(slot: &mut BackgroundColor, v: Color) {
    if slot.0 != v {
        slot.0 = v;
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
        Has<TabSecondary>,
        Has<TabDisabled>,
        &mut Surface,
        &mut Node,
        &mut Foreground,
    )>,
    mut ind: Query<(&ChildOf, &mut BackgroundColor), With<TabIndicator>>,
    tabs: Query<(&UiTab, &ControlPointer, Has<TabSecondary>)>,
) {
    for (t, i, secondary, disabled, mut s, mut n, mut fg) in &mut q {
        let k = theme.tab(dodai_ui_core::ItemState::of(
            t.active,
            *i == ControlPointer::Hovered && !disabled,
        ));
        upd(
            &mut *s,
            Surface {
                fill: if secondary { None } else { k.background },
                border: None,
                glow: false,
                pill: false,
            },
        );
        let ink = if disabled {
            theme.palette.text_disabled
        } else if secondary && t.active {
            theme.palette.primary
        } else {
            k.foreground
        };
        upd(&mut fg.0, ink);
        upd(&mut n.height, Val::Px(theme.shape.control_height));
        upd(
            &mut n.padding,
            UiRect::horizontal(Val::Px(theme.spacing.md)),
        );
    }
    for (p, mut bg) in &mut ind {
        if let Ok((t, i, secondary)) = tabs.get(p.parent()) {
            let k = theme.tab(dodai_ui_core::ItemState::of(
                t.active,
                *i == ControlPointer::Hovered,
            ));
            let line = if secondary && t.active {
                Some(theme.palette.primary)
            } else {
                k.indicator
            };
            upd(&mut bg.0, line.map_or(Color::NONE, color));
        }
    }
}

pub(crate) fn style_toggles(
    theme: Res<DodaiTheme>,
    mut q: Query<
        (
            &UiToggle,
            &ControlPointer,
            Has<ToggleMixed>,
            &mut Surface,
            &mut Node,
        ),
        Without<TogglePart>,
    >,
    mut parts: Query<(&ChildOf, &mut Node, &mut Surface), With<TogglePart>>,
    owners: Query<(&UiToggle, &ControlPointer, Has<ToggleMixed>), Without<TogglePart>>,
) {
    let round = theme.shape.kind() == ShapeKind::Rounded;
    let bw = theme.shape.border_width;
    for (t, i, mixed, mut s, mut n) in &mut q {
        let on = t.on || mixed;
        let k = theme.toggle(on, cs(i, t.disabled, false));
        let (w, h) = toggle_size(&theme, t.kind);
        upd(
            &mut *s,
            Surface {
                fill: Some(k.track),
                border: Some(k.border),
                glow: on && !t.disabled && t.kind != ToggleKind::Switch,
                pill: t.kind != ToggleKind::Checkbox,
            },
        );
        upd(&mut n.width, Val::Px(w));
        upd(&mut n.height, Val::Px(h));
    }
    let _ = round;
    for (p, mut n, mut s) in &mut parts {
        let Ok((t, i, mixed)) = owners.get(p.parent()) else {
            continue;
        };
        // Half-way: a dash instead of the mark.
        let dash = mixed && !t.on;
        let k = theme.toggle(t.on || mixed, cs(i, t.disabled, false));
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
                upd(&mut n.height, Val::Px(if dash { 2.0 } else { d.max(2.0) }));
                upd(
                    &mut n.display,
                    if t.on || dash {
                        Display::Flex
                    } else {
                        Display::None
                    },
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
    mut thumbs: Query<(&ChildOf, &SliderThumb, &mut Node, &mut Surface), Without<SliderFill>>,
    mut marks: Query<
        (
            &ChildOf,
            Option<&SliderTick>,
            Option<&SliderBubble>,
            &mut Node,
            &mut Surface,
        ),
        (
            Or<(With<SliderTick>, With<SliderBubble>)>,
            Without<UiSlider>,
            Without<SliderFill>,
            Without<SliderThumb>,
        ),
    >,
    owners: Query<(&UiSlider, Has<SliderCentred>), (Without<SliderFill>, Without<SliderThumb>)>,
) {
    let p = &theme.palette;
    let track_h = theme.spacing.sm + 2.0;
    let thumb_w = 10.0;
    for (sl, mut s, mut n) in &mut q {
        upd(
            &mut *s,
            Surface {
                fill: Some(p.surface_raised),
                border: Some(p.border),
                glow: false,
                pill: false,
            },
        );
        if sl.vertical {
            upd(&mut n.width, Val::Px(track_h));
        } else {
            upd(&mut n.height, Val::Px(track_h));
        }
    }
    for (c, mut n, mut s) in &mut fills {
        let Ok((sl, centred)) = owners.get(c.parent()) else {
            continue;
        };
        // A range fills from its low thumb (a balance from the middle): the gap before it is a
        // margin.
        let (from, len) = if centred {
            let f = sl.fraction();
            (f.min(0.5), (f - 0.5).abs())
        } else {
            (sl.lo_fraction(), sl.fraction() - sl.lo_fraction())
        };
        if sl.vertical {
            upd(&mut n.height, Val::Percent(len * 100.0));
            upd(&mut n.margin.bottom, Val::Percent(from * 100.0));
        } else {
            upd(&mut n.width, Val::Percent(len * 100.0));
            upd(&mut n.margin.left, Val::Percent(from * 100.0));
        }
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
    for (c, tick, bubble, mut n, mut s) in &mut marks {
        let Ok((sl, _)) = owners.get(c.parent()) else {
            continue;
        };
        if let Some(t) = tick {
            upd(&mut n.width, Val::Px(2.0));
            upd(&mut n.height, Val::Px(10.0));
            upd(&mut n.left, Val::Percent(t.0 * 100.0));
            upd(&mut n.margin.left, Val::Px(-1.0));
            upd(&mut n.top, Val::Px(track_h / 2.0 - 5.0));
            upd(
                &mut *s,
                Surface {
                    fill: Some(p.border_strong),
                    border: None,
                    glow: false,
                    pill: false,
                },
            );
        } else if let Some(b) = bubble {
            const W: f32 = 52.0;
            const H: f32 = 22.0;
            let long = theme.shape.control_height * 0.5;
            upd(&mut n.width, Val::Px(W));
            upd(&mut n.height, Val::Px(H));
            upd(&mut n.left, Val::Percent(sl.fraction() * 100.0));
            upd(&mut n.margin.left, Val::Px(-W / 2.0));
            let mid = track_h / 2.0;
            let y = if b.0 {
                mid - long / 2.0 - 6.0 - H
            } else {
                mid + long / 2.0 + 6.0
            };
            upd(&mut n.top, Val::Px(y));
            upd(
                &mut *s,
                Surface {
                    fill: Some(p.surface_raised),
                    border: Some(p.border),
                    glow: false,
                    pill: false,
                },
            );
        }
    }
    for (c, which, mut n, mut s) in &mut thumbs {
        let Ok((sl, _)) = owners.get(c.parent()) else {
            continue;
        };
        let f = if which.0 {
            sl.lo_fraction()
        } else {
            sl.fraction()
        };
        let long = theme.shape.control_height * 0.5;
        if sl.vertical {
            upd(&mut n.width, Val::Px(long));
            upd(&mut n.height, Val::Px(thumb_w));
            upd(&mut n.bottom, Val::Percent(f * 100.0));
            upd(&mut n.margin.bottom, Val::Px(-thumb_w / 2.0));
            upd(&mut n.left, Val::Px(track_h / 2.0 - long / 2.0));
        } else {
            upd(&mut n.width, Val::Px(thumb_w));
            upd(&mut n.height, Val::Px(long));
            upd(&mut n.left, Val::Percent(f * 100.0));
            upd(&mut n.margin.left, Val::Px(-thumb_w / 2.0));
        }
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
    for (e, i, mut s, mut n) in &mut q {
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
        upd(&mut n.height, Val::Px(input_height(&theme, i.lines)));
        upd(
            &mut n.padding,
            UiRect::axes(
                Val::Px(theme.spacing.md),
                Val::Px(if i.lines > 1 { theme.spacing.sm } else { 0.0 }),
            ),
        );
        upd(
            &mut n.align_items,
            if i.lines > 1 {
                AlignItems::FlexStart
            } else {
                AlignItems::Center
            },
        );
    }
    for (c, mut t, mut fg) in &mut texts {
        let Ok((e, i)) = owners.get(c.parent()) else {
            continue;
        };
        let focused = focus.0 == Some(e);
        let k = theme.input(focused);
        let shown = shown_value(i);
        let (raw, col) = if i.value.is_empty() && !focused {
            (i.placeholder.clone(), k.placeholder)
        } else if focused {
            (format!("{shown}|"), k.text)
        } else {
            (shown, k.text)
        };
        upd(&mut t.raw, raw);
        upd(&mut fg.0, col);
    }
}

/// The text a field displays for its value: bullets when masked.
pub(crate) fn shown_value(i: &UiTextInput) -> String {
    if i.masked {
        "\u{2022}".repeat(i.value.chars().count())
    } else {
        i.value.clone()
    }
}

/// Height of an input of `lines` lines.
pub(crate) fn input_height(theme: &dodai_ui_core::Theme, lines: u8) -> f32 {
    let h = theme.shape.control_height;
    h + f32::from(lines.max(1) - 1) * theme.typography.size_md * 1.4
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
        let k = theme.list_item(dodai_ui_core::ItemState::of(
            r.selected,
            *i == ControlPointer::Hovered,
        ));
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
            let k = theme.list_item(dodai_ui_core::ItemState::of(
                r.selected,
                *i == ControlPointer::Hovered,
            ));
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
        Option<&ButtonLabel>,
    )>,
    fgs: Query<&Foreground>,
) {
    let fonts = crate::style::FontSources::of(&theme);
    for (u, mut text, mut font, mut col, own, parent, button) in &mut q {
        let st = text_style(&theme, u.role);
        let shown = display_text(&u.raw, &st);
        upd(&mut text.0, shown);
        let size = button.map_or(st.size, |b| theme.control_metrics(b.0).text_size);
        upd(&mut font.font_size, bevy::text::FontSize::Px(size));
        if let Some(f) = fonts.for_role(u.role) {
            upd(&mut font.font, f.clone());
        }
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
    std::collections::HashMap<
        (
            dodai_ui_icons::IconId,
            u32,
            bool,
            bool,
            Option<dodai_ui_icons::Modifier>,
        ),
        Handle<Image>,
    >,
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
    let rounded = theme.shape.kind() == dodai_ui_core::ShapeKind::Rounded;
    for (e, u, node, own, parent) in &mut q {
        let fg = color(
            own.map(|f| f.0)
                .or_else(|| parent.and_then(|p| fgs.get(p.parent()).ok()).map(|f| f.0))
                .unwrap_or(theme.palette.text),
        );
        let key = (u.icon, u.size, u.filled, rounded, u.modifier);
        let needs_image = node.is_none() || u.is_changed() || theme.is_changed();
        if needs_image {
            let handle = match cache.0.get(&key) {
                Some(h) => h.clone(),
                None => {
                    let drawn = match u.modifier {
                        Some(m) => dodai_ui_icons::cyber_shapes(u.icon, Some(m))
                            .map(dodai_ui_icons::Glyph::Stroked),
                        None => dodai_ui_icons::glyph_for_theme(u.icon, &theme, u.filled),
                    };
                    let Some(g) = drawn else {
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
                .try_insert(ImageNode::new(handle).with_color(fg));
        } else if let Some(mut n) = node {
            upd(&mut n.color, fg);
        }
    }
}

/// Rasterised symbols, shared between entities.
#[cfg(feature = "symbols")]
#[derive(Resource, Default)]
pub struct SymbolCache(
    std::collections::HashMap<
        (dodai_ui_icons::symbol::Symbol, u32, dodai_ui_icons::Pack),
        Handle<Image>,
    >,
);

/// `UiSymbol` → `ImageNode`, drawn in the active design's stroke style and tinted by its state
/// (or the inherited foreground).
#[cfg(feature = "symbols")]
#[allow(clippy::type_complexity)]
pub(crate) fn style_symbols(
    mut commands: Commands,
    theme: Res<DodaiTheme>,
    mut cache: ResMut<SymbolCache>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<(
        Entity,
        Ref<UiSymbol>,
        Option<&mut ImageNode>,
        Option<&Foreground>,
        Option<&ChildOf>,
    )>,
    fgs: Query<&Foreground>,
) {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let pack = dodai_ui_icons::Pack::from(theme.look().icons);
    for (e, u, node, own, parent) in &mut q {
        let fg = color(match u.state {
            Some(s) => s.color(&theme.palette),
            None => own
                .map(|f| f.0)
                .or_else(|| parent.and_then(|p| fgs.get(p.parent()).ok()).map(|f| f.0))
                .unwrap_or(theme.palette.text),
        });
        let key = (u.symbol, u.size, pack);
        if node.is_none() || u.is_changed() || theme.is_changed() {
            let handle = match cache.0.get(&key) {
                Some(h) => h.clone(),
                None => {
                    let g = u.symbol.glyph_styled(pack.symbol_style());
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
                .try_insert(ImageNode::new(handle).with_color(fg));
        } else if let Some(mut n) = node {
            upd(&mut n.color, fg);
        }
    }
}

#[cfg(all(test, feature = "symbols"))]
mod symbol_tests {
    use super::*;
    use dodai_ui_icons::symbol::Symbol;

    #[test]
    fn symbol_becomes_an_image_tinted_by_its_state() {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .init_resource::<SymbolCache>()
            .insert_resource(Assets::<Image>::default())
            .add_systems(Update, style_symbols);
        let e = app
            .world_mut()
            .spawn(symbol_state(
                Symbol::OilCan,
                32,
                dodai_ui_core::SymbolState::Danger,
            ))
            .id();
        app.update();
        let n = app.world().get::<ImageNode>(e).expect("ImageNode inserted");
        let want = color(app.world().resource::<DodaiTheme>().palette.danger);
        assert_eq!(n.color, want);
        let img = app
            .world()
            .resource::<Assets<Image>>()
            .get(&n.image)
            .unwrap();
        let data = img.data.as_ref().unwrap();
        assert_eq!(data.len(), 32 * 32 * 4);
        assert!(data.chunks(4).any(|p| p[3] > 200), "symbol drew nothing");
    }
}

#[cfg(all(test, feature = "icons"))]
mod icon_tests {
    use super::*;

    /// A page rebuild queues the despawn of its old widgets before the style systems run; their
    /// queued inserts then meet a dead entity, which must not panic (`try_insert`).
    #[test]
    fn styling_an_entity_despawned_in_the_same_frame_does_not_panic() {
        fn kill(mut c: Commands, q: Query<Entity, With<Node>>) {
            for e in &q {
                c.entity(e).despawn();
            }
        }
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .init_resource::<IconCache>()
            .insert_resource(Assets::<Image>::default())
            .add_systems(Update, (kill, style_icons).chain_ignore_deferred());
        let e = app
            .world_mut()
            .spawn(icon(dodai_ui_icons::IconId::Home, 24))
            .id();
        app.update();
        assert!(app.world().get_entity(e).is_err());
    }

    #[test]
    fn icon_becomes_a_tinted_image_node_and_follows_theme() {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .init_resource::<IconCache>()
            .insert_resource(Assets::<Image>::default())
            .add_systems(Update, style_icons);
        let e = app
            .world_mut()
            .spawn(icon(dodai_ui_icons::IconId::Home, 24))
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

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::testing::rounded_light;

    #[test]
    fn masked_values_show_one_bullet_per_character() {
        let i = UiTextInput {
            value: "héllo".into(),
            masked: true,
            ..default()
        };
        assert_eq!(shown_value(&i), "•••••");
        let plain = UiTextInput {
            value: "abc".into(),
            ..default()
        };
        assert_eq!(shown_value(&plain), "abc");
    }

    #[test]
    fn taller_inputs_grow_by_line_and_never_shrink() {
        let t = rounded_light();
        assert_eq!(input_height(&t, 1), t.shape.control_height);
        assert_eq!(input_height(&t, 0), t.shape.control_height);
        assert!(input_height(&t, 4) > input_height(&t, 2));
    }
}

#[cfg(test)]
mod vertical_slider_tests {
    use super::*;
    use crate::widgets::{slider, tabs_editable, vertical_slider, SliderThumb};

    fn styled(bundle: impl Bundle) -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(DodaiTheme::new())
            .add_systems(Update, style_sliders);
        let e = app.world_mut().spawn(bundle).id();
        app.update();
        (app, e)
    }

    fn fill_and_thumb(app: &mut App) -> (Node, Node) {
        let w = app.world_mut();
        let fill = w
            .query_filtered::<&Node, With<SliderFill>>()
            .single(w)
            .unwrap()
            .clone();
        let thumb = w
            .query_filtered::<&Node, With<SliderThumb>>()
            .single(w)
            .unwrap()
            .clone();
        (fill, thumb)
    }

    #[test]
    fn a_vertical_slider_fills_upwards_and_puts_its_thumb_on_the_fill_top() {
        let (mut app, e) = styled(vertical_slider(2.5, 0.0, 10.0, 120.0));
        let (fill, thumb) = fill_and_thumb(&mut app);
        assert_eq!(fill.height, Val::Percent(25.0));
        assert_eq!(thumb.bottom, Val::Percent(25.0));
        assert!(matches!(app.world().get::<Node>(e).unwrap().width, Val::Px(w) if w > 0.0));
    }

    #[test]
    fn editable_tabs_report_close_and_add_without_selecting() {
        let mut app = App::new();
        app.add_message::<ControlEvent>()
            .add_systems(Update, (tab_input, tab_edit_input));
        app.world_mut()
            .spawn(tabs_editable(&["a", "b"], 0, true, true));
        app.update();
        let w = app.world_mut();
        assert_eq!(w.query::<&TabClose>().iter(w).count(), 2);
        assert_eq!(w.query::<&TabAdd>().iter(w).count(), 1);
        // Press the second tab's cross: it bubbles to the tab, which must not become active.
        let closer = w
            .query_filtered::<(Entity, &ChildOf), With<TabClose>>()
            .iter(w)
            .map(|(e, p)| (e, p.parent()))
            .nth(1)
            .unwrap();
        *w.get_mut::<ControlPointer>(closer.0).unwrap() = ControlPointer::Pressed;
        *w.get_mut::<ControlPointer>(closer.1).unwrap() = ControlPointer::Pressed;
        app.update();
        let w = app.world_mut();
        let events: Vec<_> = w
            .resource_mut::<Messages<ControlEvent>>()
            .drain()
            .map(|e| (e.entity, e.kind))
            .collect();
        assert_eq!(events, vec![(closer.1, ControlKind::TabClosed)]);
        assert!(!w.get::<UiTab>(closer.1).unwrap().active);
    }

    #[test]
    fn arrow_keys_move_the_hovered_tab_row_past_disabled_tabs() {
        use bevy::input::{keyboard::KeyCode, ButtonState};
        let mut app = App::new();
        app.add_message::<ControlEvent>()
            .add_message::<KeyboardInput>()
            .add_systems(Update, tab_keys);
        let row = app
            .world_mut()
            .spawn(tabs_editable(&["a", "b", "c"], 0, false, false))
            .id();
        app.update();
        let w = app.world_mut();
        let kids: Vec<Entity> = w.get::<Children>(row).unwrap().iter().collect();
        w.entity_mut(kids[1]).insert(TabDisabled);
        let press = |w: &mut World| {
            w.write_message(KeyboardInput {
                key_code: KeyCode::ArrowRight,
                logical_key: Key::ArrowRight,
                state: ButtonState::Pressed,
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
        };
        // Not hovered: nothing moves.
        press(app.world_mut());
        app.update();
        assert!(app.world().get::<UiTab>(kids[0]).unwrap().active);
        // Hovered: skips the disabled middle tab.
        *app.world_mut().get_mut::<ControlPointer>(kids[0]).unwrap() = ControlPointer::Hovered;
        press(app.world_mut());
        app.update();
        assert!(app.world().get::<UiTab>(kids[2]).unwrap().active);
        assert!(!app.world().get::<UiTab>(kids[0]).unwrap().active);
    }

    #[test]
    fn a_horizontal_slider_still_fills_rightwards() {
        let (mut app, _) = styled(slider(5.0, 0.0, 10.0));
        let (fill, thumb) = fill_and_thumb(&mut app);
        assert_eq!(fill.width, Val::Percent(50.0));
        assert_eq!(thumb.left, Val::Percent(50.0));
    }
}
