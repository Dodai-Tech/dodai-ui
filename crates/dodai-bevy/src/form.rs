//! Inputs and form parts: fields, steppers, ratings, OTP and tag inputs, colour swatches, file
//! drops, progress and loading indicators.

#![allow(clippy::type_complexity)]

use bevy::ecs::spawn::SpawnWith;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::ButtonState;
use bevy::prelude::*;
#[cfg(feature = "icons")]
use dodai_core::ButtonVariant;
use dodai_core::{CardVariant, Tone};

use crate::interact::{
    RatingStar, StepperButton, StepperReadout, UiRating, UiSelectGroup, UiSelectable, UiStepper,
};
use crate::kit::{kit, kit_pressable, Kit};
use crate::parts::{caption, column, glyph, row, wide, Glyph, StarFill};
use crate::systems::FocusedInput;
use crate::widgets::{
    text, text_input_with, ControlEvent, ControlKind, ControlPointer, TextRole, UiTextInput,
};

type Spawner<'a> = ChildSpawner<'a>;

/// An icon-only button.
#[cfg(feature = "icons")]
pub fn icon_button(icon: dodai_icons::IconId, variant: ButtonVariant) -> impl Bundle {
    crate::widgets::ButtonSpec::new("", variant)
        .icon(icon)
        .bundle()
}

/// An icon with a count badge pinned to its top-right corner.
#[cfg(feature = "icons")]
pub fn icon_badge(icon: dodai_icons::IconId, count: impl Into<String>, tone: Tone) -> impl Bundle {
    let count = count.into();
    (
        Node {
            width: Val::Px(32.0),
            height: Val::Px(32.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(crate::widgets::icon(icon, 24));
            c.spawn((
                crate::components::badge(count, tone),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(-4.0),
                    right: Val::Px(-6.0),
                    ..default()
                },
            ));
        })),
    )
}

/// A progress bar (`value` in `0..=1`).
pub fn progress_bar(value: f32) -> impl Bundle {
    crate::widgets::progress(value, None)
}

/// A bar that runs back and forth while something loads.
pub fn indeterminate_bar() -> impl Bundle {
    crate::spinner::indeterminate()
}

/// A loading spinner of `size` px.
pub fn loading_indicator(size: f32) -> impl Bundle {
    crate::spinner::spinner(size)
}

/// State of a [`form_field`]'s helper line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldState {
    /// A neutral hint.
    Hint(String),
    /// An error message.
    Error(String),
}

/// A label above `control` and a hint or error line below it.
pub fn form_field(
    label: impl Into<String>,
    control: impl Bundle,
    state: Option<FieldState>,
) -> impl Bundle {
    let label = label.into();
    (
        Node {
            row_gap: Val::Px(4.0),
            ..wide(column())
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(label, TextRole::Label));
            c.spawn(control);
            match state {
                Some(FieldState::Hint(h)) => {
                    c.spawn(text(h, TextRole::Muted));
                }
                Some(FieldState::Error(e)) => {
                    c.spawn((
                        kit(Kit::Dot(Some(Tone::Danger))),
                        Node {
                            width: Val::Px(0.0),
                            ..default()
                        },
                    ));
                    c.spawn(caption(e));
                }
                None => {}
            }
        })),
    )
}

/// A text field with an optional prefix (a currency, `https://`) and suffix (a unit).
pub fn input_box(
    placeholder: impl Into<String>,
    prefix: Option<&str>,
    suffix: Option<&str>,
) -> impl Bundle {
    let placeholder = placeholder.into();
    let (prefix, suffix) = (prefix.map(str::to_string), suffix.map(str::to_string));
    (
        kit(Kit::Select),
        wide(row(8.0)),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            if let Some(p) = prefix {
                c.spawn(text(p, TextRole::Muted));
            }
            c.spawn(text_input_with(placeholder, |n| n.flex_grow = 1.0));
            if let Some(s) = suffix {
                c.spawn(text(s, TextRole::Muted));
            }
        })),
    )
}

/// A number between minus and plus buttons.
pub fn number_stepper(value: f32, min: f32, max: f32, step: f32) -> impl Bundle {
    let s = UiStepper {
        value: value.clamp(min, max),
        min,
        max,
        step,
    };
    let shown = crate::interact::format_value(s.value, step);
    (
        s,
        row(8.0),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                StepperButton(-1.0),
                children![glyph(Glyph::Minus, 16)],
            ));
            c.spawn((
                StepperReadout,
                Node {
                    min_width: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                text(shown, TextRole::Content),
            ));
            c.spawn((
                kit_pressable(Kit::Chip { selected: false }),
                StepperButton(1.0),
                children![glyph(Glyph::Plus, 16)],
            ));
        })),
    )
}

/// A row of `max` stars with `value` lit; pressing a star sets the value (pressing the current
/// one clears a step).
pub fn rating(value: u8, max: u8) -> impl Bundle {
    let value = value.min(max);
    (
        UiRating { value, max },
        Node {
            column_gap: Val::Px(2.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for i in 1..=max {
                let mut e = c.spawn((
                    glyph(Glyph::Star, 20),
                    RatingStar(i),
                    StarFill(i <= value),
                    ControlPointer::default(),
                ));
                #[cfg(feature = "icons")]
                if let Some(mut u) = e.get_mut::<crate::widgets::UiIcon>() {
                    u.filled = i <= value;
                }
                let _ = &mut e;
            }
        })),
    )
}

/// One box of an [`otp_input`].
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OtpBox(pub usize);

/// A one-time-code input: `len` single-character boxes; typing fills one and moves to the next.
pub fn otp_input(len: usize) -> impl Bundle {
    (
        row(8.0),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for i in 0..len {
                c.spawn((
                    text_input_with("", |n| {
                        n.width = Val::Px(40.0);
                        n.height = Val::Px(48.0);
                        n.justify_content = JustifyContent::Center;
                    }),
                    OtpBox(i),
                ));
            }
        })),
    )
}

pub(crate) fn otp_advance(
    mut boxes: Query<(Entity, &OtpBox, &mut UiTextInput, &ChildOf), Changed<UiTextInput>>,
    siblings: Query<(Entity, &OtpBox, &ChildOf)>,
    mut focus: ResMut<FocusedInput>,
) {
    for (e, b, mut input, parent) in &mut boxes {
        if input.value.chars().count() > 1 {
            // Keep the last typed character.
            let last = input.value.chars().last().unwrap_or(' ');
            input.value = last.to_string();
        }
        if input.value.chars().count() == 1 && focus.0 == Some(e) {
            if let Some((next, _, _)) = siblings
                .iter()
                .find(|(_, s, p)| p.parent() == parent.parent() && s.0 == b.0 + 1)
            {
                focus.0 = Some(next);
            }
        }
    }
}

/// The code typed into the boxes of the [`otp_input`] `root`.
pub fn otp_value(inputs: impl IntoIterator<Item = (usize, String)>) -> String {
    let mut v: Vec<(usize, String)> = inputs.into_iter().collect();
    v.sort_by_key(|(i, _)| *i);
    v.into_iter().map(|(_, s)| s).collect()
}

/// A removable tag of a [`tag_field`].
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct TagChip(pub String);

/// A field of tags with an input at the end: Enter adds the typed text, pressing a tag removes it.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct UiTagField;

/// The input of a [`tag_field`].
#[derive(Component, Debug, Clone, Copy, Default)]
pub(crate) struct TagInput;

fn tag_chip(label: String) -> impl Bundle {
    (
        TagChip(label.clone()),
        kit_pressable(Kit::Chip { selected: true }),
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(text(label, TextRole::ControlSmall));
            c.spawn(glyph(Glyph::Close, 12));
        })),
    )
}

/// A tag field starting with `tags`.
pub fn tag_field(tags: &[&str]) -> impl Bundle {
    let tags: Vec<String> = tags.iter().map(|s| (*s).to_string()).collect();
    (
        UiTagField,
        kit(Kit::Select),
        Node {
            flex_wrap: FlexWrap::Wrap,
            row_gap: Val::Px(4.0),
            ..wide(row(6.0))
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for t in tags {
                c.spawn(tag_chip(t));
            }
            c.spawn((
                TagInput,
                text_input_with("Add…", |n| {
                    n.flex_grow = 1.0;
                    n.min_width = Val::Px(80.0);
                }),
            ));
        })),
    )
}

pub(crate) fn tag_field_keys(
    mut keys: MessageReader<KeyboardInput>,
    focus: Res<FocusedInput>,
    mut inputs: Query<(&mut UiTextInput, &ChildOf), With<TagInput>>,
    mut commands: Commands,
    mut out: MessageWriter<ControlEvent>,
) {
    let Some(f) = focus.0 else { return };
    for k in keys.read() {
        if k.state != ButtonState::Pressed
            || !matches!(k.logical_key, bevy::input::keyboard::Key::Enter)
        {
            continue;
        }
        let Ok((mut input, _)) = inputs.get_mut(f) else {
            continue;
        };
        let t = input.value.trim().to_string();
        if t.is_empty() {
            continue;
        }
        input.value.clear();
        let chip = commands.spawn(tag_chip(t)).id();
        commands.queue(move |world: &mut World| {
            let root = world.get::<ChildOf>(f).map(ChildOf::parent);
            if let Some(root) = root {
                let idx = world
                    .get::<Children>(root)
                    .map_or(0, |c| c.len().saturating_sub(1));
                world.entity_mut(root).insert_children(idx, &[chip]);
            }
        });
        out.write(ControlEvent {
            entity: f,
            kind: ControlKind::Selected,
        });
    }
}

pub(crate) fn tag_chips_remove(
    pressed: Query<(Entity, &ControlPointer), (Changed<ControlPointer>, With<TagChip>)>,
    mut commands: Commands,
    mut out: MessageWriter<ControlEvent>,
) {
    for (e, p) in &pressed {
        if *p == ControlPointer::Pressed {
            out.write(ControlEvent {
                entity: e,
                kind: ControlKind::TabClosed,
            });
            commands.entity(e).despawn();
        }
    }
}

/// The colour of a swatch of a [`color_picker`].
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiSwatch(pub Color);

/// A grid of colour swatches; pressing one selects it (a ring marks it).
pub fn color_picker(colors: &[Color], selected: usize) -> impl Bundle {
    let colors = colors.to_vec();
    (
        UiSelectGroup {
            selected: Some(selected),
        },
        Node {
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            for (i, col) in colors.into_iter().enumerate() {
                c.spawn((
                    UiSwatch(col),
                    UiSelectable(i),
                    ControlPointer::default(),
                    Node {
                        width: Val::Px(28.0),
                        height: Val::Px(28.0),
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    },
                    BackgroundColor(col),
                    Outline::new(
                        Val::Px(if i == selected { 2.0 } else { 0.0 }),
                        Val::Px(2.0),
                        Color::WHITE,
                    ),
                ));
            }
        })),
    )
}

pub(crate) fn swatch_rings(
    groups: Query<(&UiSelectGroup, &Children), Changed<UiSelectGroup>>,
    mut swatches: Query<(&UiSelectable, &mut Outline), With<UiSwatch>>,
    theme: Res<crate::theme::DodaiTheme>,
) {
    for (g, kids) in &groups {
        for k in kids.iter() {
            if let Ok((s, mut o)) = swatches.get_mut(k) {
                o.width = Val::Px(if g.selected == Some(s.0) { 2.0 } else { 0.0 });
                o.color = crate::theme::color(theme.palette.text);
            }
        }
    }
}

/// A drop target for files dragged from the desktop.
#[derive(Component, Debug, Clone, Copy, Default)]
#[require(bevy::ui::RelativeCursorPosition)]
pub struct UiFileDrop;

/// A file was dropped on a [`file_drop_zone`].
#[derive(Message, Debug, Clone, PartialEq)]
pub struct FileDropped {
    /// The zone.
    pub entity: Entity,
    /// The file.
    pub path: std::path::PathBuf,
}

/// A bordered zone with an upload glyph and `label`.
pub fn file_drop_zone(label: impl Into<String>) -> impl Bundle {
    let label = label.into();
    (
        UiFileDrop,
        kit(Kit::Card(CardVariant::Outlined)),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            min_height: Val::Px(96.0),
            ..wide(column())
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(glyph(Glyph::Upload, 24));
            c.spawn(text(label, TextRole::Muted));
        })),
    )
}

pub(crate) fn file_drops(
    mut events: MessageReader<bevy::window::FileDragAndDrop>,
    zones: Query<(Entity, &bevy::ui::RelativeCursorPosition), With<UiFileDrop>>,
    mut out: MessageWriter<FileDropped>,
) {
    for e in events.read() {
        if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = e {
            for (zone, rel) in &zones {
                if rel.cursor_over() {
                    out.write(FileDropped {
                        entity: zone,
                        path: path_buf.clone(),
                    });
                }
            }
        }
    }
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.init_resource::<FocusedInput>()
        .add_message::<FileDropped>()
        .add_message::<bevy::window::FileDragAndDrop>()
        .add_message::<KeyboardInput>()
        .add_systems(
            Update,
            (
                otp_advance,
                tag_field_keys.before(crate::systems::input_keys),
                tag_chips_remove,
                swatch_rings,
                file_drops,
            )
                .after(crate::kit::press_input),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interact::UiRating;
    use crate::widgets::UiText;

    fn app() -> App {
        crate::testing::component_app()
    }

    #[test]
    fn stepper_clamps_and_updates_its_readout() {
        let mut app = app();
        let s = app
            .world_mut()
            .spawn(number_stepper(1.0, 0.0, 2.0, 1.0))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(s).unwrap().iter().collect();
        let (minus, readout, plus) = (kids[0], kids[1], kids[2]);
        for want in [2.0, 2.0] {
            *app.world_mut().get_mut::<ControlPointer>(plus).unwrap() = ControlPointer::Hovered;
            *app.world_mut().get_mut::<ControlPointer>(plus).unwrap() = ControlPointer::Pressed;
            app.update();
            assert_eq!(app.world().get::<UiStepper>(s).unwrap().value, want);
        }
        assert_eq!(app.world().get::<UiText>(readout).unwrap().raw, "2");
        *app.world_mut().get_mut::<ControlPointer>(minus).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(app.world().get::<UiText>(readout).unwrap().raw, "1");
    }

    #[test]
    fn stepped_respects_bounds() {
        let s = UiStepper {
            value: 0.5,
            min: 0.0,
            max: 1.0,
            step: 0.25,
        };
        assert_eq!(s.stepped(-5.0), 0.0);
        assert_eq!(s.stepped(5.0), 1.0);
        assert_eq!(crate::interact::format_value(0.25, 0.25), "0.25");
    }

    #[test]
    fn rating_sets_and_clears_a_step() {
        let mut app = app();
        let r = app.world_mut().spawn(rating(2, 5)).id();
        app.update();
        let stars: Vec<Entity> = app.world().get::<Children>(r).unwrap().iter().collect();
        assert_eq!(stars.len(), 5);
        *app.world_mut().get_mut::<ControlPointer>(stars[3]).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(app.world().get::<UiRating>(r).unwrap().value, 4);
        let lit: Vec<bool> = stars
            .iter()
            .map(|s| app.world().get::<StarFill>(*s).unwrap().0)
            .collect();
        assert_eq!(lit, vec![true, true, true, true, false]);
        *app.world_mut().get_mut::<ControlPointer>(stars[3]).unwrap() = ControlPointer::Hovered;
        *app.world_mut().get_mut::<ControlPointer>(stars[3]).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(app.world().get::<UiRating>(r).unwrap().value, 3);
    }

    #[test]
    fn otp_keeps_one_character_and_moves_on() {
        let mut app = app();
        let o = app.world_mut().spawn(otp_input(3)).id();
        app.update();
        let boxes: Vec<Entity> = app.world().get::<Children>(o).unwrap().iter().collect();
        app.world_mut().resource_mut::<FocusedInput>().0 = Some(boxes[0]);
        app.world_mut()
            .get_mut::<UiTextInput>(boxes[0])
            .unwrap()
            .value = "12".into();
        app.update();
        assert_eq!(app.world().get::<UiTextInput>(boxes[0]).unwrap().value, "2");
        assert_eq!(app.world().resource::<FocusedInput>().0, Some(boxes[1]));
        assert_eq!(otp_value([(1, "b".into()), (0, "a".into())]), "ab");
    }

    #[test]
    fn pressing_a_tag_removes_it() {
        let mut app = app();
        let f = app.world_mut().spawn(tag_field(&["a", "b"])).id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(f).unwrap().iter().collect();
        assert_eq!(kids.len(), 3, "two tags and the input");
        *app.world_mut().get_mut::<ControlPointer>(kids[0]).unwrap() = ControlPointer::Pressed;
        app.update();
        assert_eq!(app.world().get::<Children>(f).unwrap().len(), 2);
    }

    #[test]
    fn color_picker_marks_the_picked_swatch() {
        let mut app = app();
        let p = app
            .world_mut()
            .spawn(color_picker(
                &[Color::WHITE, Color::BLACK, Color::srgb(1.0, 0.0, 0.0)],
                0,
            ))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(p).unwrap().iter().collect();
        *app.world_mut().get_mut::<ControlPointer>(kids[2]).unwrap() = ControlPointer::Pressed;
        app.update();
        app.update();
        let ring = |app: &App, e: Entity| app.world().get::<Outline>(e).unwrap().width;
        assert_eq!(ring(&app, kids[2]), Val::Px(2.0));
        assert_eq!(ring(&app, kids[0]), Val::Px(0.0));
    }

    #[test]
    fn form_field_and_input_box_compose() {
        let mut app = app();
        let f = app
            .world_mut()
            .spawn(form_field(
                "Name",
                input_box("x", Some("$"), Some("USD")),
                Some(FieldState::Error("bad".into())),
            ))
            .id();
        app.update();
        // label, control, dot, message
        assert_eq!(app.world().get::<Children>(f).unwrap().len(), 4);
    }

    #[test]
    fn enter_adds_the_typed_tag_before_the_input() {
        let mut app = app();
        let f = app.world_mut().spawn(tag_field(&["a"])).id();
        app.update();
        let input = *app.world().get::<Children>(f).unwrap().last().unwrap();
        app.world_mut().get_mut::<UiTextInput>(input).unwrap().value = " new ".into();
        app.world_mut().resource_mut::<FocusedInput>().0 = Some(input);
        app.world_mut().write_message(KeyboardInput {
            key_code: KeyCode::Enter,
            logical_key: bevy::input::keyboard::Key::Enter,
            state: ButtonState::Pressed,
            text: None,
            repeat: false,
            window: Entity::PLACEHOLDER,
        });
        app.update();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(f).unwrap().iter().collect();
        assert_eq!(kids.len(), 3, "a, new, input");
        assert_eq!(app.world().get::<TagChip>(kids[1]).unwrap().0, "new");
        assert_eq!(kids[2], input, "the input stays last");
        assert!(app
            .world()
            .get::<UiTextInput>(input)
            .unwrap()
            .value
            .is_empty());
    }
}
