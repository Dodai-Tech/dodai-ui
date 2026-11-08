//! Walks the model's [`Node`] tree and spawns the matching dodai-bevy widgets.

use bevy::prelude::*;
use dodai_bevy::kit::{kit, kit_pressable, Kit};
use dodai_bevy::prelude::*;
use dodai_bevy::theme::color;
use dodai_core::{ControlSize, StepState, Theme, Tone};
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Font, Gap, Ink, TextRole as Role, TextStyle};
use dodai_showcase_model::{Action, Node as Demo, Showcase, Value};

/// What a spawned widget does when the user touches it. The event system maps a
/// [`ControlEvent`] of the entity back to a model [`Action`].
#[derive(Component, Debug, Clone, PartialEq)]
pub enum Bound {
    /// A press runs this action.
    Act(Action),
    /// A switch or checkbox stores its state under the key.
    Toggle(String),
    /// A radio stores its index under the group key.
    Radio(String, usize),
    /// A slider stores its value under the key (snapped to `step`).
    Slide { key: String, step: Option<f32> },
    /// A text field stores its text under the key.
    Text(String),
    /// A − / + button changes the integer under the key.
    Step { key: String, by: i32, default: i32 },
    /// The sidebar filter.
    Filter,
    /// Opens (or closes) the named dropdown.
    Menu(&'static str),
}

/// A text that shows a stored value and follows it.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum Readout {
    /// A slider's number.
    Num {
        key: String,
        default: f32,
        step: Option<f32>,
    },
    /// A stepper's integer.
    Int { key: String, default: i32 },
}

/// A tree row: pressing it opens or closes the sibling that follows.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct TreeRow {
    pub open: bool,
}

#[derive(Component)]
pub struct TreeChevron;

/// Everything the spawner reads.
pub struct Cx<'a> {
    pub theme: &'a Theme,
    pub model: &'a Showcase,
    /// The dropdown that is open.
    pub open: Option<&'static str>,
}

/// The dropdown's name: `Bound::Menu` wants a `'static` one, so each key's is made once.
fn menu_name(key: &str) -> &'static str {
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    static NAMES: OnceLock<Mutex<HashMap<String, &'static str>>> = OnceLock::new();
    let mut names = NAMES.get_or_init(Default::default).lock().unwrap();
    names
        .entry(key.to_string())
        .or_insert_with(|| Box::leak(format!("sel:{key}").into_boxed_str()))
}

fn gap_px(theme: &Theme, g: Gap) -> f32 {
    g.px(theme)
}

/// The library role that draws a model role: size, family, colour and casing resolved by the
/// model, so text matches the gpui showcase.
fn role(r: Role, theme: &Theme) -> dodai_bevy::widgets::TextRole {
    use dodai_bevy::widgets::{Family, Ink as I, TextRole as T};
    let st = r.resolve(theme);
    T::Styled {
        size: st.size,
        family: match st.font {
            Font::Display => Family::Display,
            Font::Body => Family::Body,
            Font::Mono => Family::Mono,
        },
        ink: match st.ink {
            Ink::Text => I::Text,
            Ink::Muted => I::Muted,
            Ink::Disabled => I::Disabled,
            Ink::Primary => I::Primary,
        },
        upper: st.caps,
    }
}

/// A text role at `size` in `font` and `ink`.
fn styled(size: f32, font: Font, ink: Ink) -> Role {
    Role::Styled(TextStyle {
        size,
        font,
        ink,
        caps: false,
    })
}

/// Snap `v` to multiples of `step` within `min..=max`.
pub fn snap(v: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let v = v.clamp(min, max);
    match step {
        Some(s) if s > 0.0 => (min + ((v - min) / s).round() * s).clamp(min, max),
        _ => v,
    }
}

/// The text a slider readout shows.
pub fn num_text(v: f32, step: Option<f32>) -> String {
    if step.is_some_and(|s| s.fract() == 0.0) || v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        format!("{v:.1}")
    }
}

fn icon_node(i: IconId, size: u32) -> impl Bundle {
    dodai_bevy::widgets::icon(i, size)
}

/// Spawn `node` as children of `p`.
pub fn spawn(p: &mut ChildSpawnerCommands, node: &Demo, cx: &Cx) {
    let theme = cx.theme;
    let vals = &cx.model.values;
    match node {
        Demo::Stack(s) => {
            let gap = Val::Px(gap_px(theme, s.gap));
            let mut n = Node {
                flex_direction: match s.axis {
                    Axis::Row => FlexDirection::Row,
                    Axis::Column => FlexDirection::Column,
                },
                flex_wrap: if s.wrap {
                    FlexWrap::Wrap
                } else {
                    FlexWrap::NoWrap
                },
                row_gap: gap,
                column_gap: gap,
                align_items: if s.centre {
                    AlignItems::Center
                } else if s.axis == Axis::Row {
                    AlignItems::FlexStart
                } else {
                    AlignItems::Stretch
                },
                ..default()
            };
            if let Some(w) = s.width {
                n.width = Val::Px(w);
                n.flex_shrink = 0.0;
            } else if s.axis == Axis::Column && !s.centre {
                n.width = Val::Percent(100.0);
            }
            p.spawn(n).with_children(|c| {
                for i in &s.items {
                    spawn(c, i, cx);
                }
            });
        }
        Demo::Text(t, r) => {
            p.spawn(text(t.clone(), role(*r, theme)));
        }
        Demo::Space(px) => {
            p.spawn(Node {
                width: Val::Px(*px),
                height: Val::Px(*px),
                ..default()
            });
        }
        Demo::Swatch {
            name,
            color: c,
            detail,
        } => {
            let xs = theme.typography.size_xs;
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                width: Val::Px(118.0),
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|c2| {
                c2.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(44.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(color(*c)),
                    BorderColor::all(color(theme.palette.border)),
                ));
                c2.spawn(text(
                    name.clone(),
                    role(styled(xs, Font::Body, Ink::Text), theme),
                ));
                c2.spawn(text(
                    detail.clone(),
                    role(styled(xs, Font::Mono, Ink::Muted), theme),
                ));
            });
        }
        Demo::Sample { w, h, label } => {
            let sm = theme.typography.size_sm;
            p.spawn((
                panel_untitled(),
                Node {
                    width: Val::Px(*w),
                    height: Val::Px(*h),
                    padding: UiRect::ZERO,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(text(
                    label.clone(),
                    role(styled(sm, Font::Body, Ink::Text), theme),
                ));
            });
        }
        Demo::Image {
            w,
            h,
            caption,
            on_image,
        } => image_placeholder(p, theme, (*w, *h), caption.as_deref(), *on_image),
        Demo::Block {
            w,
            h,
            fill,
            border,
            radius,
            label,
        } => {
            let mut n = Node {
                width: if *w > 0.0 { Val::Px(*w) } else { Val::Auto },
                height: if *h > 0.0 { Val::Px(*h) } else { Val::Auto },
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(*radius)),
                flex_shrink: 0.0,
                ..default()
            };
            if border.is_some() {
                n.border = UiRect::all(Val::Px(1.0));
            }
            p.spawn((
                n,
                BackgroundColor(color(*fill)),
                BorderColor::all(border.map_or(Color::NONE, color)),
            ))
            .with_children(|c| {
                if let Some(l) = label {
                    let xs = theme.typography.size_xs;
                    c.spawn(text(
                        l.clone(),
                        role(styled(xs, Font::Body, Ink::Text), theme),
                    ));
                }
            });
        }
        Demo::Button(b) => {
            let mut spec = ButtonSpec::new(b.label.clone(), b.variant)
                .size(b.size)
                .square(b.square)
                .disabled(b.disabled || b.loading);
            if let Some(i) = b.icon {
                spec = spec.icon(i);
            }
            // Cut-corner designs crop every button by default, the others only when asked.
            let default = theme.look().cut_corners.then_some(dodai_core::Crop::Left);
            if let Some(end) = (!b.plain).then(|| b.crop.or(default)).flatten() {
                spec = spec.crop(end);
            }
            let mut e = p.spawn(spec.bundle());
            if let Some(k) = &b.counter {
                e.insert(Bound::Act(Action::Press(k.clone())));
            }
            if b.loading {
                e.with_children(|c| {
                    c.spawn(dodai_bevy::spinner::spinner(14.0));
                });
            }
        }
        Demo::Split { label, variant } => {
            p.spawn(Node {
                column_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn(ButtonSpec::new(label.clone(), *variant).bundle());
                c.spawn(
                    ButtonSpec::new("", *variant)
                        .icon(IconId::ChevronDown)
                        .bundle(),
                );
            });
        }
        Demo::Choice(ch) => {
            let at = vals.index(&ch.bind);
            p.spawn(Node {
                column_gap: Val::Px(if ch.connected { 0.0 } else { theme.spacing.xs }),
                ..default()
            })
            .with_children(|c| {
                for (i, l) in ch.labels.iter().enumerate() {
                    let v = if i == at {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    };
                    c.spawn((
                        ButtonSpec::new(l.clone(), v).bundle(),
                        Bound::Act(Action::Set(ch.bind.key.clone(), Value::Index(i))),
                    ));
                }
            });
        }
        Demo::Tabs(ch) => {
            let at = vals.index(&ch.bind);
            p.spawn((
                UiTabs,
                Node {
                    flex_direction: FlexDirection::Row,
                    ..default()
                },
            ))
            .with_children(|c| {
                for (i, l) in ch.labels.iter().enumerate() {
                    c.spawn((
                        tab(l.clone(), i == at),
                        Bound::Act(Action::Set(ch.bind.key.clone(), Value::Index(i))),
                    ));
                }
            });
        }
        Demo::Tag { label, tone, solid } => {
            p.spawn(tag(label.clone(), *tone, *solid));
        }
        Demo::Chip { label, bind, icon } => {
            let on = vals.flag(bind);
            p.spawn((
                kit_pressable(Kit::Chip { selected: on }),
                Bound::Act(Action::Set(bind.key.clone(), Value::Bool(!on))),
            ))
            .with_children(|c| {
                if let Some(i) = icon {
                    c.spawn(icon_node(*i, 16));
                }
                c.spawn(text(label.clone(), TextRole::Control));
            });
        }
        Demo::Toggle(t) => {
            let on = match &t.radio {
                Some((g, i)) => {
                    vals.index_of(g, usize::MAX) == *i
                        || (vals.index_of(g, usize::MAX) == usize::MAX && t.bind.default)
                }
                None => vals.flag(&t.bind),
            };
            let bound = match &t.radio {
                Some((g, i)) => Bound::Radio(g.clone(), *i),
                None => Bound::Toggle(t.bind.key.clone()),
            };
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.sm),
                ..default()
            })
            .with_children(|c| {
                // Radios of a group are exclusive among their siblings: keep them together.
                let kind = match t.kind {
                    dodai_core::ToggleKind::Switch => ToggleKind::Switch,
                    dodai_core::ToggleKind::Checkbox => ToggleKind::Checkbox,
                    dodai_core::ToggleKind::Radio => ToggleKind::Radio,
                };
                let mut e = if t.disabled {
                    c.spawn(toggle_disabled(kind, on))
                } else {
                    match kind {
                        ToggleKind::Switch => c.spawn(switch(on)),
                        ToggleKind::Checkbox => c.spawn(checkbox(on)),
                        ToggleKind::Radio => c.spawn(radio(on)),
                    }
                };
                e.insert(bound);
                c.spawn(text(t.label.clone(), TextRole::Control));
            });
        }
        Demo::Knob {
            bind,
            min,
            max,
            size,
            disabled,
        } => {
            let mut e = p.spawn(knob(vals.num(bind), *min, *max, *size));
            e.insert(Bound::Slide {
                key: bind.key.clone(),
                step: None,
            });
            if *disabled {
                e.entry::<UiKnob>().and_modify(|mut k| k.disabled = true);
            }
        }
        Demo::Select {
            key,
            labels,
            selected,
            placeholder,
        } => {
            let name = menu_name(key);
            let key = key.clone();
            crate::chrome::dropdown(
                p,
                &crate::chrome::Dropdown {
                    name,
                    items: labels,
                    selected: *selected,
                    placeholder,
                    width: 380.0,
                    open: cx.open == Some(name),
                },
                move |i| Action::Set(key.clone(), Value::Index(i)),
            );
        }
        Demo::Shader(spec) => {
            let mut e = p.spawn(Node {
                width: Val::Px(spec.width),
                flex_shrink: 0.0,
                ..default()
            });
            e.insert(dodai_shaders::bevy::ShaderNode::new(spec.to_spec(theme)));
        }
        Demo::Plot(pl) => {
            let lines = pl
                .lines
                .iter()
                .map(|l| PlotLine {
                    color: color(l.color),
                    width: l.width,
                    points: l.points.iter().map(|(x, y)| Vec2::new(*x, *y)).collect(),
                })
                .collect();
            p.spawn(plot(pl.w, pl.h, lines));
        }
        Demo::Slider(s) if s.vertical => {
            p.spawn((
                vertical_slider(vals.num(&s.bind), s.min, s.max, 120.0),
                Bound::Slide {
                    key: s.bind.key.clone(),
                    step: s.step,
                },
            ));
        }
        Demo::Scroll { height, body } => {
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(520.0),
                    height: Val::Px(*height),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                bevy::ui::RelativeCursorPosition::default(),
                crate::events::Scroller::Area,
            ))
            .with_children(|c| spawn(c, body, cx));
        }
        Demo::Slider(s) => {
            let v = vals.num(&s.bind);
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.md),
                width: Val::Percent(100.0),
                max_width: Val::Px(420.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    slider(v, s.min, s.max),
                    Bound::Slide {
                        key: s.bind.key.clone(),
                        step: s.step,
                    },
                ));
                if s.readout {
                    c.spawn((
                        text(num_text(v, s.step), TextRole::Mono),
                        Readout::Num {
                            key: s.bind.key.clone(),
                            default: s.bind.default,
                            step: s.step,
                        },
                    ));
                }
            });
        }
        Demo::Progress { value, tone } => {
            p.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(420.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn(progress(*value, *tone));
            });
        }
        Demo::Input(i) => {
            let init = vals.text(&i.bind).to_string();
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.spacing.xs),
                width: Val::Percent(100.0),
                max_width: Val::Px(420.0),
                ..default()
            })
            .with_children(|c| {
                if let Some(l) = &i.label {
                    c.spawn(text(l.clone(), TextRole::Label));
                }
                let mut e = if i.password {
                    c.spawn(password_input(i.placeholder.clone()))
                } else if i.lines > 1 {
                    c.spawn(text_area(i.placeholder.clone(), i.lines))
                } else {
                    c.spawn(text_input(i.placeholder.clone()))
                };
                e.insert((
                    UiTextInput {
                        value: init,
                        placeholder: i.placeholder.clone(),
                        masked: i.password,
                        lines: i.lines,
                    },
                    Bound::Text(i.bind.key.clone()),
                ));
            });
        }
        Demo::List(rows) => {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                max_width: Val::Px(520.0),
                ..default()
            })
            .with_children(|c| {
                for (n, r) in rows.iter().enumerate() {
                    if n > 0 {
                        c.spawn(divider());
                    }
                    c.spawn((
                        kit(Kit::Row { selected: false }),
                        Node {
                            width: Val::Percent(100.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        if let Some(a) = &r.avatar {
                            row.spawn((
                                kit(Kit::Avatar),
                                Node {
                                    width: Val::Px(40.0),
                                    height: Val::Px(40.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ))
                            .with_children(|a2| {
                                a2.spawn(text(a.clone(), TextRole::ControlSmall));
                            });
                        } else if let Some(i) = r.icon {
                            row.spawn(icon_node(i, 20));
                        }
                        row.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            ..default()
                        })
                        .with_children(|col| {
                            if let Some(o) = &r.overline {
                                col.spawn(text(o.clone(), TextRole::Caption));
                            }
                            col.spawn(text(r.title.clone(), TextRole::Content));
                            if let Some(d) = &r.detail {
                                col.spawn(text(d.clone(), TextRole::Muted));
                            }
                        });
                        if let Some(t) = &r.trailing {
                            row.spawn(text(t.clone(), TextRole::Caption));
                        }
                    });
                }
            });
        }
        Demo::Divider => {
            p.spawn(divider());
        }
        Demo::Card {
            title,
            body,
            variant,
        } => {
            p.spawn((
                kit(Kit::Card(*variant)),
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(theme.spacing.sm),
                    align_items: AlignItems::Stretch,
                    min_width: Val::Px(180.0),
                    ..default()
                },
            ))
            .with_children(|c| {
                if !title.is_empty() {
                    c.spawn(text(title.clone(), TextRole::Control));
                }
                spawn(c, body, cx);
            });
        }
        Demo::Alert {
            tone,
            text: t,
            banner,
        } => {
            p.spawn((
                kit(Kit::Alert {
                    tone: *tone,
                    banner: *banner,
                }),
                Node {
                    width: Val::Percent(100.0),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(icon_node(
                    match tone {
                        Tone::Danger | Tone::Warning => IconId::Warning,
                        Tone::Success => IconId::CheckCircle,
                        Tone::Info => IconId::Info,
                    },
                    18,
                ));
                c.spawn(text(t.clone(), TextRole::Content));
            });
        }
        Demo::Snackbar { text: t, action } => {
            p.spawn((
                kit(Kit::Snackbar),
                Node {
                    column_gap: Val::Px(theme.spacing.lg),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(text(t.clone(), TextRole::Content));
                c.spawn(button(action.clone(), ButtonVariant::Ghost));
            });
        }
        Demo::Popup(inner) => {
            p.spawn((
                kit(Kit::Popup),
                Node {
                    flex_direction: FlexDirection::Column,
                    min_width: Val::Px(200.0),
                    ..default()
                },
            ))
            .with_children(|c| spawn(c, inner, cx));
        }
        Demo::Icon(i, size) => {
            p.spawn(icon_node(*i, *size as u32));
        }
        Demo::Rating { value, max } => {
            p.spawn(Node {
                column_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|c| {
                let full = value.round() as u8;
                for i in 0..*max {
                    c.spawn((
                        UiIcon {
                            icon: IconId::Star,
                            size: 20,
                            filled: i < full,
                        },
                        Node {
                            width: Val::Px(20.0),
                            height: Val::Px(20.0),
                            ..default()
                        },
                    ));
                }
            });
        }
        Demo::Stepper(b) => {
            let v = vals.int(b);
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.sm),
                ..default()
            })
            .with_children(|c| {
                c.spawn((
                    ButtonSpec::new("−", ButtonVariant::Outline)
                        .size(ControlSize::Small)
                        .bundle(),
                    Bound::Step {
                        key: b.key.clone(),
                        by: -1,
                        default: b.default,
                    },
                ));
                c.spawn((
                    text(v.to_string(), TextRole::Content),
                    Readout::Int {
                        key: b.key.clone(),
                        default: b.default,
                    },
                ));
                c.spawn((
                    ButtonSpec::new("+", ButtonVariant::Outline)
                        .size(ControlSize::Small)
                        .bundle(),
                    Bound::Step {
                        key: b.key.clone(),
                        by: 1,
                        default: b.default,
                    },
                ));
            });
        }
        Demo::Steps { steps, current } => {
            p.spawn(Node {
                align_items: AlignItems::FlexStart,
                column_gap: Val::Px(theme.spacing.sm),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|c| {
                for (i, s) in steps.iter().enumerate() {
                    let st = StepState::of(i, *current);
                    c.spawn(Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(theme.spacing.sm),
                        ..default()
                    })
                    .with_children(|r| {
                        r.spawn((
                            kit(Kit::StepDot(st)),
                            Node {
                                width: Val::Px(28.0),
                                height: Val::Px(28.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|d| {
                            if st == StepState::Done {
                                d.spawn(icon_node(IconId::Check, 16));
                            } else {
                                d.spawn(text((i + 1).to_string(), TextRole::ControlSmall));
                            }
                        });
                        r.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|t| {
                            t.spawn(text(s.title.clone(), TextRole::Control));
                            if let Some(d) = &s.detail {
                                t.spawn(text(d.clone(), TextRole::Caption));
                            }
                        });
                        if i + 1 < steps.len() {
                            r.spawn((
                                Node {
                                    width: Val::Px(32.0),
                                    height: Val::Px(1.0),
                                    ..default()
                                },
                                BackgroundColor(color(theme.palette.border_strong)),
                            ));
                        }
                    });
                }
            });
        }
        Demo::Breadcrumb(items) => {
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.sm),
                ..default()
            })
            .with_children(|c| {
                for (i, it) in items.iter().enumerate() {
                    let last = i + 1 == items.len();
                    c.spawn(text(
                        it.clone(),
                        if last {
                            TextRole::Body
                        } else {
                            TextRole::Muted
                        },
                    ));
                    if !last {
                        c.spawn(icon_node(IconId::ChevronRight, 14));
                    }
                }
            });
        }
        Demo::Pagination { pages, bind } => {
            let at = vals.index(bind);
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.xs),
                ..default()
            })
            .with_children(|c| {
                let go = |i: usize| Bound::Act(Action::Set(bind.key.clone(), Value::Index(i)));
                c.spawn((
                    ButtonSpec::new("", ButtonVariant::Ghost)
                        .icon(IconId::ChevronLeft)
                        .size(ControlSize::Small)
                        .disabled(at == 0)
                        .bundle(),
                    go(at.saturating_sub(1)),
                ));
                for i in 0..*pages {
                    c.spawn((
                        kit_pressable(Kit::Page { selected: i == at }),
                        Node {
                            min_width: Val::Px(34.0),
                            height: Val::Px(34.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        go(i),
                    ))
                    .with_children(|b| {
                        b.spawn(text((i + 1).to_string(), TextRole::Control));
                    });
                }
                c.spawn((
                    ButtonSpec::new("", ButtonVariant::Ghost)
                        .icon(IconId::ChevronRight)
                        .size(ControlSize::Small)
                        .disabled(at + 1 >= *pages)
                        .bundle(),
                    go((at + 1).min(pages - 1)),
                ));
            });
        }
        Demo::Tree(nodes) => {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                max_width: Val::Px(420.0),
                ..default()
            })
            .with_children(|c| {
                for n in nodes {
                    tree_node(c, n, 0, cx);
                }
            });
        }
        Demo::Table { columns, rows } => {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|c| {
                let cells = |r: &mut ChildSpawnerCommands, items: &[String], role: TextRole| {
                    for it in items {
                        r.spawn((
                            Node {
                                flex_grow: 1.0,
                                flex_basis: Val::Px(0.0),
                                ..default()
                            },
                            children![text(it.clone(), role)],
                        ));
                    }
                };
                let line = |k: Kit| {
                    (
                        kit(k),
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                    )
                };
                c.spawn(line(Kit::TableHead))
                    .with_children(|r| cells(r, columns, TextRole::ControlSmall));
                for (i, row) in rows.iter().enumerate() {
                    c.spawn(line(Kit::TableRow(i % 2 == 1)))
                        .with_children(|r| cells(r, row, TextRole::Content));
                }
            });
        }
        Demo::Avatar(s) => {
            p.spawn((
                kit(Kit::Avatar),
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(text(s.clone(), TextRole::ControlSmall));
            });
        }
        Demo::Tooltip { text: t, rich } => {
            p.spawn(kit(Kit::Tooltip { rich: *rich }))
                .with_children(|c| {
                    c.spawn(text(t.clone(), TextRole::Content));
                });
        }
        Demo::Code(s) => {
            p.spawn((
                kit(Kit::Code),
                Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(520.0),
                    ..default()
                },
            ))
            .with_children(|c| {
                c.spawn(text(s.clone(), TextRole::Mono));
            });
        }
        Demo::Timeline(events) => {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(theme.spacing.md),
                ..default()
            })
            .with_children(|c| {
                for e in events {
                    c.spawn(Node {
                        column_gap: Val::Px(theme.spacing.md),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|r| {
                        r.spawn((
                            kit(Kit::Dot(e.tone)),
                            Node {
                                width: Val::Px(12.0),
                                height: Val::Px(12.0),
                                margin: UiRect::top(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                        r.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            ..default()
                        })
                        .with_children(|t| {
                            t.spawn(text(e.title.clone(), TextRole::Content));
                            t.spawn(text(e.time.clone(), TextRole::Caption));
                            if let Some(d) = &e.detail {
                                t.spawn(text(d.clone(), TextRole::Muted));
                            }
                        });
                    });
                }
            });
        }
        Demo::Badged { icon, count, dot } => {
            p.spawn(Node {
                width: Val::Px(theme.shape.control_height),
                height: Val::Px(theme.shape.control_height),
                ..default()
            })
            .with_children(|c| {
                c.spawn(
                    ButtonSpec::new("", ButtonVariant::Ghost)
                        .icon(*icon)
                        .bundle(),
                );
                let corner = Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(0.0),
                    right: Val::Px(0.0),
                    ..default()
                };
                if let Some((n, tone)) = count {
                    c.spawn((kit(Kit::Count(*tone)), corner.clone()))
                        .with_children(|b| {
                            b.spawn(text(n.to_string(), TextRole::ControlSmall));
                        });
                }
                if let Some(tone) = dot {
                    c.spawn((
                        kit(Kit::Dot(Some(*tone))),
                        Node {
                            width: Val::Px(10.0),
                            height: Val::Px(10.0),
                            ..corner
                        },
                    ));
                }
            });
        }
        Demo::Spinner(size) => {
            p.spawn(dodai_bevy::spinner::spinner(*size));
        }
        Demo::Indeterminate => {
            p.spawn(Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(420.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn(dodai_bevy::spinner::indeterminate());
            });
        }
        Demo::Skeleton { w, h } => {
            p.spawn((
                kit(Kit::Skeleton),
                Node {
                    width: if *w > 0.0 {
                        Val::Px(*w)
                    } else {
                        Val::Percent(100.0)
                    },
                    height: Val::Px(*h),
                    ..default()
                },
            ));
        }
        Demo::Empty { title, detail } => {
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(theme.spacing.xs),
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(theme.spacing.lg)),
                ..default()
            })
            .with_children(|c| {
                c.spawn(icon_node(IconId::Folder, 32));
                c.spawn(text(title.clone(), TextRole::Control));
                c.spawn(text(detail.clone(), TextRole::Muted));
            });
        }
    }
}

fn tree_node(
    p: &mut ChildSpawnerCommands,
    n: &dodai_showcase_model::node::TreeNode,
    depth: usize,
    cx: &Cx,
) {
    let theme = cx.theme;
    let branch = !n.children.is_empty();
    p.spawn((
        kit_pressable(Kit::Row { selected: false }),
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::left(Val::Px(8.0 + 18.0 * depth as f32)),
            column_gap: Val::Px(theme.spacing.sm),
            ..default()
        },
        TreeRow { open: n.open },
    ))
    .with_children(|r| {
        if branch {
            r.spawn((
                TreeChevron,
                UiIcon {
                    icon: if n.open {
                        IconId::ChevronDown
                    } else {
                        IconId::ChevronRight
                    },
                    size: 14,
                    filled: false,
                },
                Node {
                    width: Val::Px(14.0),
                    height: Val::Px(14.0),
                    ..default()
                },
            ));
        } else {
            r.spawn(Node {
                width: Val::Px(14.0),
                ..default()
            });
        }
        if let Some(i) = n.icon {
            r.spawn(icon_node(i, 16));
        }
        r.spawn(text(n.label.clone(), TextRole::Content));
    });
    if branch {
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            display: if n.open { Display::Flex } else { Display::None },
            ..default()
        })
        .with_children(|c| {
            for k in &n.children {
                tree_node(c, k, depth + 1, cx);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snap_clamps_and_rounds_to_the_step() {
        assert_eq!(snap(7.4, 0.0, 10.0, Some(1.0)), 7.0);
        assert_eq!(snap(7.6, 0.0, 10.0, Some(1.0)), 8.0);
        assert_eq!(snap(-3.0, 0.0, 10.0, Some(1.0)), 0.0);
        assert_eq!(snap(99.0, 0.0, 10.0, Some(2.0)), 10.0);
        assert_eq!(snap(3.3, 0.0, 10.0, None), 3.3);
        // A step that does not divide the range never leaves it.
        assert!(snap(10.0, 0.0, 10.0, Some(3.0)) <= 10.0);
    }

    #[test]
    fn snap_respects_a_nonzero_minimum() {
        assert_eq!(snap(2.4, 1.0, 5.0, Some(1.0)), 2.0);
        assert_eq!(snap(0.0, 1.0, 5.0, Some(1.0)), 1.0);
    }

    #[test]
    fn readouts_drop_the_fraction_of_whole_numbers() {
        assert_eq!(num_text(40.0, None), "40");
        assert_eq!(num_text(3.0, Some(1.0)), "3");
        assert_eq!(num_text(3.25, None), "3.2");
    }

    #[test]
    fn gaps_follow_the_spacing_scale() {
        let t = dodai_core::themes::all().remove(0);
        assert!(gap_px(&t, Gap::Xs) <= gap_px(&t, Gap::Sm));
        assert!(gap_px(&t, Gap::Sm) <= gap_px(&t, Gap::Md));
        assert!(gap_px(&t, Gap::Md) <= gap_px(&t, Gap::Lg));
    }
}

/// A framed picture slot (no picture loads: the frame says "image unavailable", as the gpui
/// showcase's blank image does), its caption below or on the design's fade.
fn image_placeholder(
    p: &mut ChildSpawnerCommands,
    theme: &Theme,
    (w, h): (f32, f32),
    caption: Option<&str>,
    on_image: bool,
) {
    let ty = &theme.typography;
    let over = on_image && caption.is_some();
    p.spawn(Node {
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(theme.spacing.xs),
        width: Val::Px(w),
        flex_shrink: 0.0,
        ..default()
    })
    .with_children(|col| {
        col.spawn((
            Node {
                width: Val::Px(w),
                height: Val::Px(h),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(theme.shape.border_width)),
                border_radius: BorderRadius::all(Val::Px(theme.shape.radius)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(color(theme.palette.surface_raised)),
            BorderColor::all(color(theme.palette.border)),
        ))
        .with_children(|f| {
            f.spawn(text(
                "image unavailable",
                role(styled(ty.size_md, Font::Body, Ink::Muted), theme),
            ));
            if !over {
                return;
            }
            let shadow = theme.palette.roles.shadow;
            let fade_h = (h * 0.5).max(theme.sizes().touch_target);
            let stops = theme.fade_stops();
            let steps = 24;
            for k in 0..steps {
                // `at` 0 = bottom edge, 1 = top of the fade.
                let at = 1.0 - (k as f32 + 0.5) / steps as f32;
                let band = fade_h / steps as f32;
                f.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        right: Val::Px(0.0),
                        top: Val::Px(h - fade_h + k as f32 * band),
                        height: Val::Px(band + 0.5),
                        ..default()
                    },
                    BackgroundColor(color(shadow.with_alpha(fade_alpha(&stops, at)))),
                ));
            }
            let ink = theme.on(shadow.with_alpha(1.0));
            f.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(theme.spacing.md),
                    right: Val::Px(theme.spacing.md),
                    bottom: Val::Px(theme.spacing.sm),
                    ..default()
                },
                Text::new(caption.unwrap_or_default().to_string()),
                TextFont {
                    font_size: bevy::text::FontSize::Px(ty.size_sm),
                    ..default()
                },
                TextColor(color(ink)),
            ));
        });
        if let (false, Some(c)) = (over, caption) {
            col.spawn(text(c.to_string(), role(Role::Muted, theme)));
        }
    });
}

/// Alpha of the fade at `at` (0 = bottom edge, 1 = top), linear between the design's stops.
fn fade_alpha(stops: &[dodai_core::FadeStop], at: f32) -> f32 {
    for w in stops.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        if at >= lo.at && at <= hi.at {
            let f = if hi.at > lo.at {
                (at - lo.at) / (hi.at - lo.at)
            } else {
                0.0
            };
            return lo.alpha + (hi.alpha - lo.alpha) * f;
        }
    }
    stops.last().map_or(0.0, |s| s.alpha)
}
