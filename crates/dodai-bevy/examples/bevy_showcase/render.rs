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
    /// A range slider stores its thumbs under the two keys (snapped to `step`).
    Range {
        lo: String,
        hi: String,
        step: Option<f32>,
        span: (f32, f32),
    },
    /// A text field stores its text under the key.
    Text(String),
    /// A − / + button changes the integer under the key.
    Step { key: String, by: i32, default: i32 },
    /// The sidebar filter.
    Filter,
    /// An orientation gizmo stores its camera under the yaw and pitch keys.
    Gizmo { yaw: String, pitch: String },
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
            Ink::Link => I::Link,
            Ink::Title => I::Title,
            Ink::Tone(t) => I::Tone(t),
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

/// The camera an orientation gizmo starts at and goes home to: yaw and pitch, radians.
pub const GIZMO_HOME: (f32, f32) = (0.6, 0.45);

/// Keeps a readout on one line beside a growing control.
fn no_shrink() -> Node {
    Node {
        flex_shrink: 0.0,
        ..default()
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
        Demo::Grow => {
            p.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
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
            src: Some(src),
            video,
            contain,
            w,
            h,
            caption,
            ..
        } => picture(
            p,
            theme,
            src,
            (*w, *h),
            (*video, *contain),
            caption.as_deref(),
        ),
        Demo::Image {
            w,
            h,
            caption,
            on_image,
            ..
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
        Demo::Button(b) if b.elevated => {
            let mut e = p.spawn((kit_pressable(Kit::Elevated), Node::default()));
            if let Some(sh) = lift(theme, Kit::Elevated) {
                e.insert(sh);
            }
            if let Some(k) = &b.counter {
                e.insert(Bound::Act(Action::Press(k.clone())));
            }
            e.with_children(|c| {
                c.spawn(text(b.label.clone(), TextRole::Control));
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
            // The Shape page's crop (cut-corner designs crop the left end by default).
            let default = dodai_showcase_model::tune_state::default_crop(vals, theme);
            if let Some(end) = (!b.plain).then(|| b.crop.or(default)).flatten() {
                spec = spec.crop(end);
            }
            let mut e = p.spawn(spec.bundle());
            if b.surface.is_set() {
                e.insert(UiGlass::of(dodai_core::SurfaceClass::Button).with(b.surface));
            }
            if let Some(k) = &b.counter {
                e.insert(Bound::Act(Action::Press(k.clone())));
            }
            if b.loading {
                e.with_children(|c| {
                    c.spawn(dodai_bevy::spinner::spinner(14.0));
                });
            }
        }
        Demo::Split {
            label,
            variant,
            open,
            crop,
        } => {
            let arrow = if *open {
                IconId::ChevronUp
            } else {
                IconId::ChevronDown
            };
            let mut main = ButtonSpec::new(label.clone(), *variant);
            if let Some(end) = crop {
                main = main.crop(*end);
            }
            p.spawn(Node {
                column_gap: Val::Px(2.0),
                ..default()
            })
            .with_children(|c| {
                c.spawn(main.bundle());
                c.spawn(ButtonSpec::new("", *variant).icon(arrow).bundle());
            });
        }
        Demo::Fab { label, .. } => {
            let k = Kit::Fab {
                extended: label.is_some(),
            };
            let mut e = p.spawn((kit_pressable(k), Node::default()));
            if let Some(sh) = lift(theme, k) {
                e.insert(sh);
            }
            e.with_children(|c| {
                c.spawn(icon_node(IconId::Plus, 24));
                if let Some(l) = label {
                    c.spawn(text(l.clone(), TextRole::Control));
                }
            });
        }
        Demo::Toolbar {
            floating,
            vibrant,
            icons,
        } => {
            let k = Kit::Toolbar {
                floating: *floating,
                vibrant: *vibrant,
            };
            let mut e = p.spawn((
                kit(k),
                Node {
                    width: if *floating {
                        Val::Auto
                    } else {
                        Val::Percent(100.0)
                    },
                    column_gap: Val::Px(theme.spacing.xs),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
            ));
            if let Some(sh) = lift(theme, k) {
                e.insert(sh);
            }
            e.with_children(|c| {
                for &i in icons {
                    c.spawn(ButtonSpec::new("", ButtonVariant::Ghost).icon(i).bundle());
                }
            });
        }
        Demo::Choice(ch) if ch.connected => {
            segmented(p, theme, &ch.labels, vals.index(&ch.bind), |i| {
                Action::Set(ch.bind.key.clone(), Value::Index(i))
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
                    let mut e = c.spawn(tab(l.clone(), i == at));
                    if ch.disabled.contains(&i) {
                        e.insert(TabDisabled);
                    } else {
                        e.insert(Bound::Act(Action::Set(
                            ch.bind.key.clone(),
                            Value::Index(i),
                        )));
                    }
                    if ch.secondary {
                        e.insert(TabSecondary);
                    }
                    if let Some(&ic) = ch.icons.get(i) {
                        let icon = e.commands().spawn(icon_node(ic, 18)).id();
                        e.insert(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: Val::Px(theme.spacing.xs),
                            ..default()
                        })
                        .insert_children(0, &[icon]);
                    }
                }
            });
        }
        Demo::Tag { label, tone, solid } => {
            p.spawn(tag(label.clone(), *tone, *solid));
        }
        Demo::Chip(ch) => {
            let on = vals.flag(&ch.bind);
            let k = if ch.elevated {
                Kit::Elevated
            } else {
                Kit::Chip { selected: on }
            };
            let mut e = p.spawn((
                kit_pressable(k),
                Bound::Act(Action::Set(ch.bind.key.clone(), Value::Bool(!on))),
            ));
            if let Some(sh) = lift(theme, k) {
                e.insert(sh);
            }
            e.with_children(|c| {
                let lead = if ch.filter && on {
                    Some(IconId::Check)
                } else {
                    ch.icon
                };
                if let Some(i) = lead {
                    c.spawn(icon_node(i, 16));
                }
                c.spawn(text(ch.label.clone(), TextRole::Control));
                if ch.removable {
                    c.spawn(icon_node(IconId::Close, 14));
                }
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
                if t.mixed {
                    e.insert(ToggleMixed);
                }
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
            let mut e = p.spawn(vertical_slider(
                vals.num(&s.bind),
                s.min,
                s.max,
                s.length.unwrap_or(120.0),
            ));
            if !s.disabled {
                e.insert(Bound::Slide {
                    key: s.bind.key.clone(),
                    step: s.step,
                });
            }
        }
        Demo::Range(r) => {
            p.spawn((
                range_slider(vals.num(&r.lo), vals.num(&r.hi), r.min, r.max, r.vertical),
                Bound::Range {
                    lo: r.lo.key.clone(),
                    hi: r.hi.key.clone(),
                    step: r.step,
                    span: (r.min, r.max),
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
            // Room for the bubble over or under the thumb.
            let room = Val::Px(36.0);
            p.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(theme.spacing.md),
                width: Val::Percent(100.0),
                margin: UiRect {
                    top: if s.bubble == Some(true) {
                        room
                    } else {
                        Val::ZERO
                    },
                    bottom: if s.bubble == Some(false) {
                        room
                    } else {
                        Val::ZERO
                    },
                    ..default()
                },
                ..default()
            })
            .with_children(|c| {
                if let Some(i) = s.icon {
                    c.spawn(icon_node(i, 20));
                }
                let mut e = c.spawn(slider(v, s.min, s.max));
                if !s.disabled {
                    e.insert(Bound::Slide {
                        key: s.bind.key.clone(),
                        step: s.step,
                    });
                }
                if s.centered {
                    e.insert(SliderCentred);
                }
                let ticks = if s.ticks {
                    dodai_core::slider_math::tick_fractions(s.min, s.max, s.step)
                } else {
                    Vec::new()
                };
                let shown = match s.format {
                    dodai_showcase_model::node::SliderFormat::Plain => num_text(v, s.step),
                    f => f.text(v),
                };
                e.with_children(|t| {
                    for f in ticks {
                        t.spawn(slider_tick(f));
                    }
                    if let Some(above) = s.bubble {
                        t.spawn(slider_bubble(shown, above));
                    }
                });
                if s.readout && s.format != dodai_showcase_model::node::SliderFormat::Plain {
                    c.spawn((text(s.format.text(v), TextRole::Mono), no_shrink()));
                } else if s.readout {
                    c.spawn((
                        text(num_text(v, s.step), TextRole::Mono),
                        no_shrink(),
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
                        kit(Kit::Row {
                            selected: r.selected,
                        }),
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
        Demo::Framed { border, body } => {
            p.spawn((
                panel_untitled(),
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .insert(dodai_bevy::surface::SurfaceFramed(Some(*border)))
            .with_children(|c| spawn(c, body, cx));
        }
        Demo::Card {
            title,
            body,
            variant,
            surface,
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
            .insert(UiGlass::of(dodai_core::SurfaceClass::Card).with(*surface))
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
            action,
            dismissible,
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
                if !*banner {
                    c.spawn(icon_node(
                        match tone {
                            Tone::Danger | Tone::Warning => IconId::Warning,
                            Tone::Success => IconId::CheckCircle,
                            Tone::Info => IconId::Info,
                        },
                        18,
                    ));
                }
                c.spawn((
                    Node {
                        flex_grow: 1.0,
                        ..default()
                    },
                    children![text(t.clone(), TextRole::Content)],
                ));
                if let Some(a) = action {
                    c.spawn((text(a.clone(), TextRole::Content), Underline));
                }
                if *dismissible {
                    c.spawn(icon_node(IconId::Close, 18));
                }
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
        Demo::Frame {
            h,
            body,
            scrim,
            layer,
            place,
        } => {
            use dodai_showcase_model::node::Place;
            let mut at = Node {
                position_type: PositionType::Absolute,
                ..default()
            };
            match place {
                Place::Centre => {
                    at.width = Val::Percent(100.0);
                    at.height = Val::Percent(100.0);
                    at.justify_content = JustifyContent::Center;
                    at.align_items = AlignItems::Center;
                }
                Place::Start => {
                    at.top = Val::Px(0.0);
                    at.bottom = Val::Px(0.0);
                    at.left = Val::Px(0.0);
                }
                Place::End => {
                    at.top = Val::Px(0.0);
                    at.bottom = Val::Px(0.0);
                    at.right = Val::Px(0.0);
                }
                Place::Bottom => {
                    at.left = Val::Px(0.0);
                    at.right = Val::Px(0.0);
                    at.bottom = Val::Px(0.0);
                    at.flex_direction = FlexDirection::Column;
                }
            }
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(*h),
                    padding: UiRect::all(Val::Px(theme.spacing.md)),
                    border: UiRect::all(Val::Px(1.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::all(color(theme.palette.border)),
            ))
            .with_children(|f| {
                spawn(f, body, cx);
                if let Some(c) = scrim {
                    f.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(0.0),
                            right: Val::Px(0.0),
                            top: Val::Px(0.0),
                            bottom: Val::Px(0.0),
                            ..default()
                        },
                        BackgroundColor(color(*c)),
                    ));
                }
                f.spawn(at).with_children(|l| spawn(l, layer, cx));
            });
        }
        Demo::Glyph {
            icon,
            size,
            filled,
            ink,
        } => {
            glyph_node(p, *icon, *size, *filled, *ink);
        }
        Demo::IconTile { icon, filled } => {
            let name = dodai_showcase_model::icons::shortened(
                icon.name(),
                dodai_showcase_model::icons::TILE_NAME_CHARS,
            );
            p.spawn((
                Node {
                    width: Val::Px(92.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(4.0),
                    padding: UiRect::axes(Val::Px(4.0), Val::Px(theme.spacing.sm)),
                    border: UiRect::all(Val::Px(1.0)),
                    flex_shrink: 0.0,
                    ..default()
                },
                BorderColor::all(color(theme.palette.border)),
            ))
            .with_children(|c| {
                glyph_node(c, *icon, 28.0, *filled, theme.palette.primary);
                c.spawn(text(
                    name,
                    role(
                        styled(theme.typography.size_xs, Font::Body, Ink::Muted),
                        theme,
                    ),
                ));
            });
        }
        Demo::Frosted { w, h, panels } => {
            // The sharp picture, and over it each panel's blurred, tinted copy of what it covers.
            let rects: Vec<[f32; 4]> = panels.iter().map(|p| p.rect).collect();
            let scene = dodai_showcase_model::media::frosted(theme, *w as u32, *h as u32, &rects);
            let r = BorderRadius::all(Val::Px(theme.spacing.md));
            let edge = theme.frost().edge;
            let abs = |x: f32, y: f32, w: f32, h: f32| Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                width: Val::Px(w),
                height: Val::Px(h),
                ..default()
            };
            p.spawn(Node {
                width: Val::Px(*w),
                height: Val::Px(*h),
                flex_shrink: 0.0,
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|s| {
                s.spawn((abs(0.0, 0.0, *w, *h), PicturePixels(scene.sharp.clone())));
                for (panel, pane) in panels.iter().zip(&scene.panes) {
                    let [x, y, pw, ph] = panel.rect;
                    s.spawn((
                        Node {
                            border_radius: r,
                            overflow: Overflow::clip(),
                            ..abs(x, y, pw, ph)
                        },
                        PicturePixels(pane.clone()),
                    ));
                    s.spawn((
                        Node {
                            border_radius: r,
                            border: UiRect::all(Val::Px(edge.map_or(0.0, |e| e.1))),
                            padding: UiRect::all(Val::Px(theme.spacing.md)),
                            flex_direction: FlexDirection::Column,
                            ..abs(x, y, pw, ph)
                        },
                        BorderColor::all(edge.map_or(Color::NONE, |e| color(e.0))),
                    ))
                    .with_children(|g| spawn(g, &panel.body, cx));
                }
            });
        }
        Demo::Gizmo { yaw, pitch, size } => {
            let g = UiAxisGizmo {
                basis: Basis::from_orbit(vals.num(yaw), vals.num(pitch)),
                layout: dodai_core::orientation::GizmoLayout::new(*size),
            };
            p.spawn((
                axis_gizmo(*size),
                Bound::Gizmo {
                    yaw: yaw.key.clone(),
                    pitch: pitch.key.clone(),
                },
            ))
            .insert(g);
        }
        Demo::Anchor {
            open,
            trigger,
            panel,
        } => {
            let shown = vals.flag(open);
            let mut e = p.spawn(Node::default());
            // A button trigger flips it through its own command.
            if !matches!(**trigger, Demo::Button(_)) {
                e.insert((
                    UiPress,
                    ControlPointer::default(),
                    Bound::Act(Action::Set(open.key.clone(), Value::Bool(!shown))),
                ));
            }
            e.with_children(|c| {
                spawn(c, trigger, cx);
                if shown {
                    c.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            top: Val::Percent(100.0),
                            left: Val::Px(0.0),
                            margin: UiRect::top(Val::Px(theme.spacing.xs)),
                            ..default()
                        },
                        GlobalZIndex(50),
                    ))
                    .with_children(|f| spawn(f, panel, cx));
                }
            });
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
                for slot in dodai_core::paging::page_window(at + 1, *pages, 7) {
                    let dodai_core::paging::PageItem::Page(p) = slot else {
                        c.spawn(text("…", TextRole::Muted));
                        continue;
                    };
                    let i = p - 1;
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
                ..default()
            })
            .with_children(|c| {
                for n in nodes {
                    tree_node(c, n, 0, cx);
                }
            });
        }
        Demo::Table(tb) => table(p, tb, theme),
        Demo::Pick { bind, items } => {
            let at = vals.index(bind);
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|c| {
                for (i, title) in items.iter().enumerate() {
                    c.spawn((
                        kit_pressable(Kit::Row { selected: i == at }),
                        Bound::Act(Action::Set(bind.key.clone(), Value::Index(i))),
                        Node {
                            width: Val::Percent(100.0),
                            ..default()
                        },
                    ))
                    .with_children(|r| {
                        r.spawn(text(title.clone(), role(Role::Body, theme)));
                    });
                }
            });
        }
        Demo::Rich { spans, role: r } => {
            rich(p, spans, *r, theme);
        }
        Demo::Quote(body) => {
            p.spawn((
                Node {
                    padding: UiRect::left(Val::Px(theme.spacing.md)),
                    border: UiRect::left(Val::Px(2.0)),
                    ..default()
                },
                BorderColor::all(color(theme.palette.primary)),
            ))
            .with_children(|c| spawn(c, body, cx));
        }
        Demo::AvatarGroup { names, max, size } => {
            let shown = max.unwrap_or(names.len()).min(names.len());
            let mut all: Vec<String> = names[..shown].to_vec();
            if names.len() > shown {
                all.push(format!("+{}", names.len() - shown));
            }
            p.spawn(Node::default()).with_children(|c| {
                for (i, n) in all.into_iter().enumerate() {
                    c.spawn((
                        kit(Kit::Avatar),
                        Node {
                            width: Val::Px(*size),
                            height: Val::Px(*size),
                            justify_content: JustifyContent::Center,
                            margin: UiRect::left(Val::Px(if i == 0 { 0.0 } else { -size * 0.25 })),
                            ..default()
                        },
                    ))
                    .with_children(|a| {
                        a.spawn(text(n, TextRole::ControlSmall));
                    });
                }
            });
        }
        Demo::Overlay(items) => {
            p.spawn(Node {
                display: Display::Grid,
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|c| {
                for n in items {
                    c.spawn(Node {
                        grid_row: GridPlacement::start(1),
                        grid_column: GridPlacement::start(1),
                        ..default()
                    })
                    .with_children(|l| spawn(l, n, cx));
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
        Demo::Tooltip {
            text: t,
            rich: r,
            title,
            action,
        } => {
            let ty = &theme.typography;
            p.spawn((
                kit(Kit::Tooltip { rich: *r }),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    align_self: AlignSelf::FlexStart,
                    row_gap: Val::Px(theme.spacing.sm),
                    ..default()
                },
            ))
            .with_children(|c| {
                if let Some(h) = title {
                    c.spawn(text(
                        h.clone(),
                        role(styled(ty.size_md, Font::Body, Ink::Text), theme),
                    ));
                }
                c.spawn(text(
                    t.clone(),
                    role(styled(ty.size_sm, Font::Body, Ink::Text), theme),
                ));
                if let Some(a) = action {
                    c.spawn(text(
                        a.clone(),
                        role(styled(ty.size_sm, Font::Body, Ink::Primary), theme),
                    ));
                }
            });
        }
        Demo::Code { text: code, lang } => code_block(p, code, lang, theme),
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
        Demo::Empty {
            icon: glyph,
            title,
            detail,
        } => {
            let ty = &theme.typography;
            p.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(theme.spacing.sm),
                padding: UiRect::all(Val::Px(theme.spacing.lg)),
                ..default()
            })
            .with_children(|c| {
                c.spawn(text(
                    glyph.clone(),
                    role(styled(ty.size_xxl, Font::Body, Ink::Muted), theme),
                ));
                c.spawn(text(
                    title.clone(),
                    role(styled(ty.size_lg, Font::Body, Ink::Text), theme),
                ));
                c.spawn(text(
                    detail.clone(),
                    role(styled(ty.size_sm, Font::Body, Ink::Muted), theme),
                ));
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
        kit_pressable(Kit::Row {
            selected: n.selected,
        }),
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

/// The shadow of a lifted kit: the design's hard shadow, or a soft drop shadow.
fn lift(theme: &Theme, k: Kit) -> Option<BoxShadow> {
    if !theme.kit(k, false).lift {
        return None;
    }
    let hard = theme.effects.hard_shadow;
    Some(if hard > 0.0 {
        BoxShadow::new(
            color(theme.palette.border_strong),
            Val::Px(hard),
            Val::Px(hard),
            Val::Px(0.0),
            Val::Px(0.0),
        )
    } else {
        BoxShadow::new(
            Color::srgba(0.0, 0.0, 0.0, 0.28),
            Val::Px(0.0),
            Val::Px(2.0),
            Val::Px(0.0),
            Val::Px(6.0),
        )
    })
}

/// A data table: header row, banded rows; fixed-width columns keep their width, the others
/// share the rest; scrolls inside `height` when set.
fn table(p: &mut ChildSpawnerCommands, tb: &dodai_showcase_model::node::Table, theme: &Theme) {
    use dodai_showcase_model::node::Align;
    let cell = |r: &mut ChildSpawnerCommands,
                s: &str,
                col: &dodai_showcase_model::node::Column,
                rl: Role| {
        let mut n = Node {
            justify_content: match col.align {
                Align::Left => JustifyContent::FlexStart,
                Align::Center => JustifyContent::Center,
                Align::Right => JustifyContent::FlexEnd,
            },
            ..default()
        };
        match col.width {
            Some(w) => {
                n.width = Val::Px(w);
                n.flex_shrink = 0.0;
            }
            None => {
                n.flex_grow = 1.0;
                n.flex_basis = Val::Px(0.0);
            }
        }
        r.spawn(n).with_children(|c| {
            c.spawn(text(s.to_string(), role(rl, theme)));
        });
    };
    let line = |k: Kit| {
        (
            kit(k),
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                ..default()
            },
        )
    };
    p.spawn(Node {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        ..default()
    })
    .with_children(|c| {
        c.spawn(line(Kit::TableHead)).with_children(|r| {
            for col in &tb.columns {
                cell(r, &col.title, col, Role::Label);
            }
        });
        let mut body = Node {
            flex_direction: FlexDirection::Column,
            width: Val::Percent(100.0),
            ..default()
        };
        if let Some(h) = tb.height {
            body.height = Val::Px(h);
            body.overflow = Overflow::scroll_y();
        }
        let mut b = c.spawn(body);
        if tb.height.is_some() {
            b.insert((
                ScrollPosition::default(),
                bevy::ui::RelativeCursorPosition::default(),
                crate::events::Scroller::Area,
            ));
        }
        b.with_children(|rows| {
            for (i, row) in tb.rows.iter().enumerate() {
                rows.spawn(line(Kit::TableRow(i % 2 == 1)))
                    .with_children(|r| {
                        for (s, col) in row.iter().zip(&tb.columns) {
                            cell(r, s, col, Role::Body);
                        }
                    });
            }
        });
    });
}

/// The font source of a model font, when fonts come from the system.
fn font_source(theme: &Theme, f: Font) -> Option<bevy::text::FontSource> {
    use dodai_bevy::widgets::{Family, Ink as I, TextRole as T};
    let family = match f {
        Font::Display => Family::Display,
        Font::Body => Family::Body,
        Font::Mono => Family::Mono,
    };
    dodai_bevy::style::FontSources::of(theme)
        .for_role(T::Styled {
            size: 0.0,
            family,
            ink: I::Text,
            upper: false,
        })
        .cloned()
}

/// Runs of `(text, colour, bold, italic, mono)` as one text block.
fn runs(
    p: &mut ChildSpawnerCommands,
    theme: &Theme,
    size: f32,
    base: Font,
    parts: Vec<(String, dodai_core::Color, bool, bool, bool)>,
) {
    let font = |mono: bool, bold: bool, italic: bool| {
        let mut f = TextFont {
            font_size: bevy::text::FontSize::Px(size),
            ..default()
        };
        if let Some(src) = font_source(theme, if mono { Font::Mono } else { base }) {
            f.font = src;
        }
        if bold {
            f.weight = bevy::text::FontWeight::BOLD;
        }
        if italic {
            f.style = bevy::text::FontStyle::Italic;
        }
        f
    };
    p.spawn((Text::new(""), font(false, false, false)))
        .with_children(|t| {
            for (s, c, bold, italic, mono) in parts {
                t.spawn((
                    TextSpan::new(s),
                    font(mono, bold, italic),
                    TextColor(color(c)),
                ));
            }
        });
}

/// Inline-styled runs as one paragraph.
fn rich(
    p: &mut ChildSpawnerCommands,
    spans: &[dodai_showcase_model::node::Span],
    r: Role,
    theme: &Theme,
) {
    let st = r.resolve(theme);
    let ink = st.ink.color(theme);
    let pl = &theme.palette;
    let parts = spans
        .iter()
        .map(|s| {
            let c = if s.link {
                pl.link
            } else if s.code {
                pl.primary
            } else {
                ink
            };
            (s.text.clone(), c, s.bold, s.italic, s.code)
        })
        .collect();
    runs(p, theme, st.size, st.font, parts);
}

/// A code listing: the language over syntax-coloured lines.
fn code_block(p: &mut ChildSpawnerCommands, code: &str, lang: &str, theme: &Theme) {
    use dodai_core::syntax::{highlight_lines, CodeToken as K};
    let pl = &theme.palette;
    let colour = |k: K| match k {
        K::Keyword => pl.primary,
        K::Str => pl.success,
        K::Comment => pl.text_muted,
        K::Number => pl.warning,
        K::Type => pl.info,
    };
    let size = theme.typography.size_sm;
    p.spawn((
        kit(Kit::Code),
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            ..default()
        },
    ))
    .with_children(|outer| {
        // The kit centres its children; the listing reads from the left.
        let mut inner = outer.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            width: Val::Percent(100.0),
            ..default()
        });
        inner.with_children(|c| {
            if !lang.is_empty() {
                c.spawn(text(lang.to_uppercase(), role(Role::Caption, theme)));
            }
            for (l, spans) in code.split('\n').zip(highlight_lines(code, lang)) {
                let mut parts = Vec::new();
                let mut at = 0;
                for (r, k) in spans {
                    if r.start > at {
                        parts.push((l[at..r.start].to_string(), pl.text, false, false, true));
                    }
                    parts.push((l[r.clone()].to_string(), colour(k), false, false, true));
                    at = r.end;
                }
                // An empty line keeps its height.
                let rest = if l[at..].is_empty() && parts.is_empty() {
                    " "
                } else {
                    &l[at..]
                };
                parts.push((rest.to_string(), pl.text, false, false, true));
                runs(c, theme, size, Font::Mono, parts);
            }
        });
    });
}

/// A picture file to show in this node; [`load_pictures`] turns it into an image.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct PicturePath {
    path: String,
    contain: bool,
}

/// A segmented control: the options in one bordered strip, the picked one filled (with a check
/// on tonal designs); a press runs `act` of its index.
pub fn segmented(
    p: &mut ChildSpawnerCommands,
    theme: &Theme,
    labels: &[String],
    selected: usize,
    act: impl Fn(usize) -> Action,
) {
    let tonal = theme.look().tonal;
    let h = if tonal {
        theme.sizes().segmented_height
    } else {
        theme.shape.control_height - 6.0
    };
    p.spawn((
        kit(Kit::Segmented),
        Node {
            column_gap: Val::Px(if tonal { 0.0 } else { theme.spacing.half() }),
            align_self: AlignSelf::FlexStart,
            flex_shrink: 0.0,
            ..default()
        },
    ))
    .with_children(|c| {
        for (i, l) in labels.iter().enumerate() {
            let on = i == selected;
            let label = if on && tonal {
                format!("\u{2713} {l}")
            } else {
                l.clone()
            };
            c.spawn((
                kit_pressable(Kit::Segment { selected: on }),
                Node {
                    height: Val::Px(h),
                    align_items: AlignItems::Center,
                    ..default()
                },
                Bound::Act(act(i)),
            ))
            .with_children(|b| {
                b.spawn(text(label, TextRole::Control));
            });
        }
    });
}

/// A catalogue icon or symbol at `size` px in `ink`.
fn glyph_node(
    p: &mut ChildSpawnerCommands,
    icon: dodai_showcase_model::icons::IconRef,
    size: f32,
    filled: bool,
    ink: dodai_core::Color,
) {
    use dodai_showcase_model::icons::IconRef;
    match icon {
        IconRef::Named(i) => {
            p.spawn((
                UiIcon {
                    icon: i,
                    size: size as u32,
                    filled,
                },
                Node {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    flex_shrink: 0.0,
                    ..default()
                },
                Foreground(ink),
            ));
        }
        IconRef::Symbol(s) => {
            p.spawn((dodai_bevy::widgets::symbol(s, size as u32), Foreground(ink)));
        }
    }
}

/// Model pixels to show in this node, stretched over it; [`load_pictures`] turns them into an
/// image.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct PicturePixels(dodai_showcase_model::media::Picture);

/// The image asset of model pixels.
fn picture_image(pic: &dodai_showcase_model::media::Picture) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    Image::new(
        Extent3d {
            width: pic.width,
            height: pic.height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pic.rgba.to_vec(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Gives every new [`PicturePath`] and [`PicturePixels`] node its image.
pub fn load_pictures(
    mut commands: Commands,
    q: Query<(Entity, &PicturePath), Added<PicturePath>>,
    pixels: Query<(Entity, &PicturePixels), Added<PicturePixels>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (e, px) in &pixels {
        let mut node = ImageNode::new(images.add(picture_image(&px.0)));
        node.image_mode = bevy::ui::widget::NodeImageMode::Stretch;
        commands.entity(e).insert(node);
    }
    for (e, pp) in &q {
        let Some(pic) = dodai_showcase_model::media::picture(&pp.path) else {
            continue;
        };
        let img = picture_image(&pic);
        let mut node = ImageNode::new(images.add(img));
        if !pp.contain {
            node.image_mode = bevy::ui::widget::NodeImageMode::Stretch;
        }
        commands.entity(e).insert(node);
    }
}

/// A picture file in the design's frame, a play badge for a video poster, the caption under it.
fn picture(
    p: &mut ChildSpawnerCommands,
    theme: &Theme,
    src: &str,
    (w, h): (f32, f32),
    (video, contain): (bool, bool),
    caption: Option<&str>,
) {
    let fit = dodai_showcase_model::media::picture(src)
        .map(|pic| {
            // Cover or contain: the picture's size inside the frame.
            let (pw, ph) = (pic.width as f32, pic.height as f32);
            let s = if contain {
                (w / pw).min(h / ph)
            } else {
                (w / pw).max(h / ph)
            };
            (pw * s, ph * s)
        })
        .unwrap_or((w, h));
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
            f.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(fit.0),
                    height: Val::Px(fit.1),
                    ..default()
                },
                PicturePath {
                    path: src.to_string(),
                    contain,
                },
            ));
            if video {
                let d = (h * 0.28).clamp(28.0, 56.0);
                f.spawn((
                    Node {
                        width: Val::Px(d),
                        height: Val::Px(d),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(color(theme.palette.background.with_alpha(0.75))),
                    BorderColor::all(color(theme.palette.primary)),
                ))
                .with_children(|b| {
                    b.spawn(text(
                        "▶",
                        role(styled(d * 0.45, Font::Body, Ink::Primary), theme),
                    ));
                });
            }
        });
        if let Some(c) = caption {
            col.spawn(text(c.to_string(), role(Role::Muted, theme)));
        }
    });
}
