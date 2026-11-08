//! Walks the model's [`Node`] tree and builds iced widgets.

use std::collections::HashSet;

use dodai_core::{ButtonVariant as V, ControlSize, Kit, StepState, Tone};
use dodai_iced::IcedTheme;
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Gap, TextRole as Role, TreeNode};
use dodai_showcase_model::{Action, Node as Demo, Showcase, Value};
use iced::widget::{
    checkbox, column, container, progress_bar, radio, row, scrollable, slider, stack, text,
    text_input, toggler, vertical_slider, Space,
};
use iced::{alignment, Element, Length};

use crate::Msg;

/// Everything the builder reads.
pub struct Cx<'a> {
    pub lt: &'a IcedTheme,
    pub model: &'a Showcase,
    /// Seconds since start (spinners).
    pub t: f32,
    /// Tree rows whose open state the user flipped (by path).
    pub tree_toggled: &'a HashSet<String>,
}

/// A model plot drawn as stroked polylines.
struct PlotView(dodai_showcase_model::plot::Plot);

impl<M> iced::widget::canvas::Program<M> for PlotView {
    type State = ();

    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        for l in &self.0.lines {
            let mut pts = l.points.iter().map(|(x, y)| iced::Point::new(*x, *y));
            let Some(first) = pts.next() else { continue };
            let path = Path::new(|b| {
                b.move_to(first);
                pts.for_each(|p| b.line_to(p));
            });
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(dodai_iced::color(l.color))
                    .with_width(l.width),
            );
        }
        vec![frame.into_geometry()]
    }
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

/// Counter range of the number stepper demo.
pub const STEPPER_RANGE: (i32, i32) = (0, 20);

/// The message of a stepper button: the new value, clamped to the range.
pub fn stepped(cur: i32, by: i32) -> i32 {
    (cur + by).clamp(STEPPER_RANGE.0, STEPPER_RANGE.1)
}

fn gap(lt: &IcedTheme, g: Gap) -> f32 {
    let s = &lt.theme().spacing;
    match g {
        Gap::Xs => s.xs,
        Gap::Sm => s.sm,
        Gap::Md => s.md,
        Gap::Lg => s.lg,
    }
}

fn txt<'a>(cx: &Cx, s: &str, r: Role) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    match r {
        Role::Title => lt.heading_text(s).into(),
        Role::Label => lt.label_text(s).into(),
        Role::Body => lt.body_text(s).into(),
        Role::Muted => lt.muted_text(s).into(),
        Role::Caption => text(s.to_string())
            .size(th.typography.size_xs)
            .style(lt.muted())
            .into(),
        Role::Mono => lt.mono_text(s).into(),
        Role::Type(t) => {
            let st = th.type_style(t);
            let c = dodai_iced::color(th.palette.text);
            text(s.to_string())
                .size(st.size)
                .font(lt.font())
                .style(move |_| text::Style { color: Some(c) })
                .into()
        }
    }
}

fn icon<'a>(cx: &Cx, i: IconId, size: f32) -> Element<'a, Msg> {
    cx.lt.icon(i, size, None)
}

/// Build `node`.
pub fn view<'a>(node: &Demo, cx: &Cx) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    let sp = &th.spacing;
    let vals = &cx.model.values;
    match node {
        Demo::Stack(s) => {
            let g = gap(lt, s.gap);
            let items: Vec<Element<Msg>> = s.items.iter().map(|n| view(n, cx)).collect();
            match (s.axis, s.wrap) {
                (Axis::Row, true) => row(items)
                    .spacing(g)
                    .align_y(alignment::Vertical::Center)
                    .wrap()
                    .vertical_spacing(g)
                    .into(),
                (Axis::Row, false) => row(items)
                    .spacing(g)
                    .align_y(alignment::Vertical::Center)
                    .into(),
                (Axis::Column, _) if s.centre => column(items)
                    .spacing(g)
                    .align_x(alignment::Horizontal::Center)
                    .into(),
                (Axis::Column, _) => column(items).spacing(g).width(Length::Fill).into(),
            }
        }
        Demo::Text(t, r) => txt(cx, t, *r),
        Demo::Space(px) => Space::new().width(*px).height(*px).into(),
        Demo::Swatch { name, color: c } => {
            let fill = dodai_iced::color(*c);
            let border = dodai_iced::color(th.palette.border);
            let radius = th.shape.radius.min(10.0);
            column![
                container("")
                    .width(104)
                    .height(44)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(fill)),
                        border: iced::Border {
                            color: border,
                            width: 1.0,
                            radius: radius.into(),
                        },
                        ..container::Style::default()
                    }),
                txt(cx, name, Role::Caption),
                txt(
                    cx,
                    &format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b),
                    Role::Mono
                ),
            ]
            .spacing(4)
            .width(104)
            .into()
        }
        Demo::Block {
            w,
            h,
            fill,
            border,
            radius,
            label,
        } => {
            let fill = dodai_iced::color(*fill);
            let border = border.map(dodai_iced::color);
            let radius = *radius;
            let inner: Element<Msg> = match label {
                Some(l) => txt(cx, l, Role::Caption),
                None => Space::new().into(),
            };
            let mut c = container(inner)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center)
                .style(move |_| container::Style {
                    background: Some(iced::Background::Color(fill)),
                    border: iced::Border {
                        color: border.unwrap_or(iced::Color::TRANSPARENT),
                        width: if border.is_some() { 1.0 } else { 0.0 },
                        radius: radius.into(),
                    },
                    ..container::Style::default()
                });
            if *w > 0.0 {
                c = c.width(*w);
            }
            if *h > 0.0 {
                c = c.height(*h);
            }
            c.into()
        }
        Demo::Button(b) => {
            let icon_el = if b.loading {
                Some(lt.spinner(14.0, cx.t))
            } else {
                b.icon.map(|i| icon(cx, i, 16.0))
            };
            let msg = if b.disabled || b.loading {
                None
            } else {
                Some(match &b.counter {
                    Some(k) => Msg::Act(Action::Press(k.clone())),
                    None => Msg::Noop,
                })
            };
            lt.button_sized(&b.label, icon_el, b.variant, b.size, msg)
        }
        Demo::Split { label, variant } => row![
            lt.button_sized(label, None, *variant, ControlSize::Medium, Some(Msg::Noop)),
            lt.button_sized(
                "",
                Some(icon(cx, IconId::ChevronDown, 16.0)),
                *variant,
                ControlSize::Medium,
                Some(Msg::Noop)
            ),
        ]
        .spacing(2)
        .into(),
        Demo::Choice(ch) => {
            let at = vals.index(&ch.bind);
            row(ch.labels.iter().enumerate().map(|(i, l)| {
                let v = if i == at { V::Primary } else { V::Outline };
                lt.button_sized(
                    l,
                    None,
                    v,
                    ControlSize::Medium,
                    Some(Msg::Act(Action::Set(ch.bind.key.clone(), Value::Index(i)))),
                )
            }))
            .spacing(if ch.connected { 0.0 } else { sp.xs })
            .into()
        }
        Demo::Tabs(ch) => {
            let at = vals.index(&ch.bind);
            let key = ch.bind.key.clone();
            let labels: Vec<&str> = ch.labels.iter().map(String::as_str).collect();
            lt.tabs(&labels, at, move |i| {
                Msg::Act(Action::Set(key.clone(), Value::Index(i)))
            })
        }
        Demo::Tag { label, tone, solid } => lt.tag(label, *tone, *solid),
        Demo::Chip {
            label,
            bind,
            icon: ic,
        } => {
            let on = vals.flag(bind);
            let mut r = row![].spacing(6).align_y(alignment::Vertical::Center);
            if let Some(i) = ic {
                r = r.push(icon(cx, *i, 16.0));
            }
            r = r.push(text(label.clone()).size(th.typography.size_sm));
            lt.kit_button(
                Kit::Chip { selected: on },
                r,
                Msg::Act(Action::Set(bind.key.clone(), Value::Bool(!on))),
            )
        }
        Demo::Toggle(t) => {
            let key = t.bind.key.clone();
            match t.kind {
                dodai_core::ToggleKind::Switch => {
                    let mut w = toggler(vals.flag(&t.bind))
                        .label(t.label.clone())
                        .style(lt.toggler());
                    if !t.disabled {
                        w = w.on_toggle(move |b| Msg::Set(key.clone(), Value::Bool(b)));
                    }
                    w.into()
                }
                dodai_core::ToggleKind::Checkbox => {
                    let mut w = checkbox(vals.flag(&t.bind))
                        .label(t.label.clone())
                        .style(lt.checkbox());
                    if !t.disabled {
                        w = w.on_toggle(move |b| Msg::Set(key.clone(), Value::Bool(b)));
                    }
                    w.into()
                }
                dodai_core::ToggleKind::Radio => {
                    let (group, idx) = t.radio.clone().unwrap_or((key, 0));
                    let picked = vals.index_of(&group, usize::MAX);
                    let selected = if picked == usize::MAX {
                        t.bind.default.then_some(idx)
                    } else {
                        Some(picked)
                    };
                    radio(t.label.clone(), idx, selected, move |i| {
                        Msg::Set(group.clone(), Value::Index(i))
                    })
                    .style(lt.radio())
                    .into()
                }
            }
        }
        Demo::Knob {
            bind,
            min,
            max,
            size,
            disabled,
        } => {
            let key = bind.key.clone();
            let range = dodai_iced::KnobRange {
                min: *min,
                max: *max,
                step: None,
            };
            lt.knob(vals.num(bind), range, *size, *disabled, move |x| {
                Msg::Set(key.clone(), Value::Num(x))
            })
        }
        Demo::Plot(p) => iced::widget::canvas(PlotView(p.clone()))
            .width(p.w)
            .height(p.h)
            .into(),
        Demo::Scroll { height, body } => {
            scrollable(container(view(body, cx)).padding(sp.sm).width(Length::Fill))
                .height(Length::Fixed(*height))
                .width(Length::Fixed(520.0))
                .style(lt.scrollable())
                .into()
        }
        Demo::Slider(s) if s.vertical => {
            let v = vals.num(&s.bind);
            let key = s.bind.key.clone();
            let (min, max, step) = (s.min, s.max, s.step);
            let mut sl = vertical_slider(min..=max, v, move |x| {
                Msg::Set(key.clone(), Value::Num(snap(x, min, max, step)))
            })
            .height(120)
            .style(lt.slider());
            if let Some(st) = s.step {
                sl = sl.step(st);
            }
            sl.into()
        }
        Demo::Slider(s) => {
            let v = vals.num(&s.bind);
            let key = s.bind.key.clone();
            let (min, max, step) = (s.min, s.max, s.step);
            let mut sl = slider(min..=max, v, move |x| {
                Msg::Set(key.clone(), Value::Num(snap(x, min, max, step)))
            })
            .style(lt.slider());
            if let Some(st) = s.step {
                sl = sl.step(st);
            }
            let mut r = row![container(sl).width(Length::Fixed(320.0))]
                .spacing(sp.md)
                .align_y(alignment::Vertical::Center);
            if s.readout {
                r = r.push(txt(cx, &num_text(v, s.step), Role::Mono));
            }
            r.into()
        }
        Demo::Progress { value, tone } => container(
            progress_bar(0.0..=1.0, *value)
                .style(lt.progress(*tone))
                .girth(10),
        )
        .width(Length::Fixed(420.0))
        .into(),
        Demo::Input(i) => {
            let key = i.bind.key.clone();
            let mut inp = text_input(&i.placeholder, vals.text(&i.bind))
                .on_input(move |s| Msg::Set(key.clone(), Value::Text(s)))
                .padding(sp.md)
                .style(lt.text_input())
                .secure(i.password);
            if i.lines > 1 {
                inp = inp.size(th.typography.size_md);
            }
            let field: Element<Msg> = match i.icon {
                Some(ic) => row![icon(cx, ic, 18.0), inp]
                    .spacing(sp.sm)
                    .align_y(alignment::Vertical::Center)
                    .into(),
                None => inp.into(),
            };
            let mut col = column![].spacing(sp.xs).width(Length::Fixed(420.0));
            if let Some(l) = &i.label {
                col = col.push(txt(cx, l, Role::Label));
            }
            let field: Element<Msg> = if i.lines > 1 {
                container(field)
                    .height(f32::from(i.lines) * th.typography.size_md * 1.4 + 2.0 * sp.md)
                    .into()
            } else {
                field
            };
            col.push(field).into()
        }
        Demo::List(rows) => {
            let mut col = column![].width(Length::Fixed(520.0));
            for (n, r) in rows.iter().enumerate() {
                if n > 0 {
                    col = col.push(lt.divider());
                }
                let mut line = row![].spacing(sp.md).align_y(alignment::Vertical::Center);
                if let Some(a) = &r.avatar {
                    line = line.push(lt.avatar(a, 40.0));
                } else if let Some(i) = r.icon {
                    line = line.push(icon(cx, i, 20.0));
                }
                let mut body = column![];
                if let Some(o) = &r.overline {
                    body = body.push(txt(cx, o, Role::Caption));
                }
                body = body.push(txt(cx, &r.title, Role::Body));
                if let Some(d) = &r.detail {
                    body = body.push(txt(cx, d, Role::Muted));
                }
                line = line.push(container(body).width(Length::Fill));
                if let Some(t) = &r.trailing {
                    line = line.push(txt(cx, t, Role::Caption));
                }
                col = col.push(lt.kit_box(Kit::Row { selected: false }, line));
            }
            col.into()
        }
        Demo::Divider => lt.divider().into(),
        Demo::Card {
            title,
            body,
            variant,
        } => {
            let mut col = column![].spacing(sp.sm);
            if !title.is_empty() {
                col = col.push(txt(cx, title, Role::Label));
            }
            col = col.push(view(body, cx));
            container(lt.kit_box(Kit::Card(*variant), col))
                .width(Length::Shrink)
                .into()
        }
        Demo::Alert {
            tone,
            text: t,
            banner,
        } => {
            let ic = match tone {
                Tone::Danger | Tone::Warning => IconId::Warning,
                Tone::Success => IconId::CheckCircle,
                Tone::Info => IconId::Info,
            };
            let ink = th
                .kit(
                    Kit::Alert {
                        tone: *tone,
                        banner: *banner,
                    },
                    false,
                )
                .fg;
            let line = row![
                lt.icon(ic, 18.0, Some(ink)),
                text(t.clone()).size(th.typography.size_md)
            ]
            .spacing(sp.sm)
            .align_y(alignment::Vertical::Center);
            container(lt.kit_box(
                Kit::Alert {
                    tone: *tone,
                    banner: *banner,
                },
                line,
            ))
            .width(Length::Fill)
            .into()
        }
        Demo::Snackbar { text: t, action } => {
            let (_, action_colour) = th.snackbar();
            let ac = dodai_iced::color(action_colour);
            let line = row![
                text(t.clone()).size(th.typography.size_md),
                text(lt.label(action)).size(th.typography.size_sm).color(ac)
            ]
            .spacing(sp.lg)
            .align_y(alignment::Vertical::Center);
            lt.kit_box(Kit::Snackbar, line)
        }
        Demo::Popup(inner) => container(lt.kit_box(Kit::Popup, view(inner, cx)))
            .width(Length::Shrink)
            .into(),
        Demo::Icon(i, size) => icon(cx, *i, *size),
        Demo::Rating { value, max } => {
            let full = value.round() as u8;
            row((0..*max)
                .map(|i| lt.icon_styled(IconId::Star, 20.0, Some(th.palette.text), i < full)))
            .spacing(2)
            .into()
        }
        Demo::Stepper(b) => {
            let v = vals.int(b);
            let key = b.key.clone();
            let step = |label: &str, by: i32| {
                lt.button_sized(
                    label,
                    None,
                    V::Outline,
                    ControlSize::Small,
                    Some(Msg::Set(key.clone(), Value::Int(stepped(v, by)))),
                )
            };
            row![
                step("−", -1),
                txt(cx, &v.to_string(), Role::Body),
                step("+", 1)
            ]
            .spacing(sp.sm)
            .align_y(alignment::Vertical::Center)
            .into()
        }
        Demo::Steps { steps, current } => {
            let mut r = row![].spacing(sp.sm).align_y(alignment::Vertical::Center);
            for (i, s) in steps.iter().enumerate() {
                let st = StepState::of(i, *current);
                let tok = th.step(st);
                let mark: Element<Msg> = if st == StepState::Done {
                    lt.icon(IconId::Check, 16.0, Some(tok.mark))
                } else {
                    text((i + 1).to_string())
                        .size(th.typography.size_xs)
                        .color(dodai_iced::color(tok.mark))
                        .into()
                };
                let dot = container(mark)
                    .width(28)
                    .height(28)
                    .align_x(alignment::Horizontal::Center)
                    .align_y(alignment::Vertical::Center)
                    .style(move |_| container::Style {
                        background: tok
                            .fill
                            .map(|c| iced::Background::Color(dodai_iced::color(c))),
                        border: iced::Border {
                            color: dodai_iced::color(tok.stroke),
                            width: 1.5,
                            radius: 14.0.into(),
                        },
                        ..container::Style::default()
                    });
                let mut label = column![txt(cx, &s.title, Role::Label)];
                if let Some(d) = &s.detail {
                    label = label.push(txt(cx, d, Role::Caption));
                }
                r = r.push(dot).push(label);
                if i + 1 < steps.len() {
                    r = r.push(container("").width(32).height(1).style(move |_| {
                        container::Style {
                            background: Some(iced::Background::Color(dodai_iced::color(
                                tok.stroke,
                            ))),
                            ..container::Style::default()
                        }
                    }));
                }
            }
            r.into()
        }
        Demo::Breadcrumb(items) => {
            let mut r = row![].spacing(sp.sm).align_y(alignment::Vertical::Center);
            for (i, it) in items.iter().enumerate() {
                let last = i + 1 == items.len();
                r = r.push(txt(cx, it, if last { Role::Body } else { Role::Muted }));
                if !last {
                    r = r.push(icon(cx, IconId::ChevronRight, 14.0));
                }
            }
            r.into()
        }
        Demo::Pagination { pages, bind } => {
            let at = vals.index(bind);
            let go = |i: usize| Msg::Act(Action::Set(bind.key.clone(), Value::Index(i)));
            let mut r = row![lt.button_sized(
                "",
                Some(icon(cx, IconId::ChevronLeft, 16.0)),
                V::Ghost,
                ControlSize::Small,
                (at > 0).then(|| go(at - 1)),
            )]
            .spacing(sp.xs)
            .align_y(alignment::Vertical::Center);
            for i in 0..*pages {
                r = r.push(
                    lt.kit_button(
                        Kit::Page { selected: i == at },
                        container(text((i + 1).to_string()).size(th.typography.size_sm))
                            .width(24)
                            .align_x(alignment::Horizontal::Center),
                        go(i),
                    ),
                );
            }
            r = r.push(lt.button_sized(
                "",
                Some(icon(cx, IconId::ChevronRight, 16.0)),
                V::Ghost,
                ControlSize::Small,
                (at + 1 < *pages).then(|| go(at + 1)),
            ));
            r.into()
        }
        Demo::Tree(nodes) => {
            let mut col = column![].width(Length::Fixed(420.0));
            for (i, n) in nodes.iter().enumerate() {
                col = tree(col, n, 0, &i.to_string(), cx);
            }
            col.into()
        }
        Demo::Table { columns, rows } => {
            let cells = |items: &[String], r: Role| -> Vec<Element<'a, Msg>> {
                items
                    .iter()
                    .map(|c| {
                        container(txt(cx, c, r))
                            .width(Length::FillPortion(1))
                            .into()
                    })
                    .collect()
            };
            let mut col = column![lt.kit_box(Kit::TableHead, row(cells(columns, Role::Label)))]
                .width(Length::Fill);
            for (i, r) in rows.iter().enumerate() {
                col = col.push(lt.kit_box(Kit::TableRow(i % 2 == 1), row(cells(r, Role::Body))));
            }
            col.into()
        }
        Demo::Avatar(s) => lt.avatar(s, 40.0),
        Demo::Tooltip { text: t, rich } => lt.kit_box(
            Kit::Tooltip { rich: *rich },
            text(t.clone()).size(th.typography.size_md),
        ),
        Demo::Code(s) => container(lt.kit_box(Kit::Code, lt.mono_text(s)))
            .width(Length::Fixed(520.0))
            .into(),
        Demo::Timeline(events) => {
            let mut col = column![].spacing(sp.md);
            for e in events {
                let c =
                    dodai_iced::color(e.tone.map_or(th.palette.text_muted, |t| th.palette.tone(t)));
                let dot = container("")
                    .width(12)
                    .height(12)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(c)),
                        border: iced::Border {
                            radius: 6.0.into(),
                            ..iced::Border::default()
                        },
                        ..container::Style::default()
                    });
                let mut body = column![
                    txt(cx, &e.title, Role::Body),
                    txt(cx, &e.time, Role::Caption)
                ];
                if let Some(d) = &e.detail {
                    body = body.push(txt(cx, d, Role::Muted));
                }
                col = col.push(
                    row![
                        container(dot).padding(iced::Padding::new(0.0).top(4.0)),
                        body
                    ]
                    .spacing(sp.md),
                );
            }
            col.into()
        }
        Demo::Badged {
            icon: ic,
            count,
            dot,
        } => {
            let base = lt.button_sized(
                "",
                Some(icon(cx, *ic, 18.0)),
                V::Ghost,
                ControlSize::Medium,
                Some(Msg::Noop),
            );
            let badge: Element<Msg> = match (count, dot) {
                (Some((n, tone)), _) => lt.kit_box(
                    Kit::Count(*tone),
                    text(n.to_string()).size(th.typography.size_xs),
                ),
                (None, Some(tone)) => {
                    let c = dodai_iced::color(th.palette.tone(*tone));
                    container("")
                        .width(10)
                        .height(10)
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(c)),
                            border: iced::Border {
                                radius: 5.0.into(),
                                ..iced::Border::default()
                            },
                            ..container::Style::default()
                        })
                        .into()
                }
                _ => Space::new().into(),
            };
            stack![
                base,
                container(badge)
                    .width(Length::Fill)
                    .align_x(alignment::Horizontal::Right)
            ]
            .into()
        }
        Demo::Spinner(size) => lt.spinner(*size, cx.t),
        Demo::Indeterminate => container(lt.indeterminate(cx.t))
            .width(Length::Fixed(420.0))
            .into(),
        Demo::Skeleton { w, h } => lt.skeleton(*w, *h),
        Demo::Empty { title, detail } => container(
            column![
                icon(cx, IconId::Folder, 32.0),
                txt(cx, title, Role::Label),
                txt(cx, detail, Role::Muted),
            ]
            .spacing(sp.xs)
            .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .padding(sp.lg)
        .align_x(alignment::Horizontal::Center)
        .into(),
    }
}

/// Path key of a tree row: its position among its ancestors (`"0.2.1"`).
pub fn path_key(parent: &str, index: usize) -> String {
    if parent.is_empty() {
        index.to_string()
    } else {
        format!("{parent}.{index}")
    }
}

/// Whether the row at `key` is open: the node's own default, flipped once for every toggle the
/// user made (`toggled` holds the keys whose state differs from the default).
pub fn is_open(toggled: &HashSet<String>, key: &str, default: bool) -> bool {
    default ^ toggled.contains(key)
}

fn tree<'a>(
    mut col: iced::widget::Column<'a, Msg>,
    n: &TreeNode,
    depth: usize,
    key: &str,
    cx: &Cx,
) -> iced::widget::Column<'a, Msg> {
    let lt = cx.lt;
    let sp = &lt.theme().spacing;
    let branch = !n.children.is_empty();
    let open = is_open(cx.tree_toggled, key, n.open);
    let mut line = row![Space::new().width(8.0 + 18.0 * depth as f32)]
        .spacing(sp.sm)
        .align_y(alignment::Vertical::Center);
    line = line.push(if branch {
        icon(
            cx,
            if open {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            },
            14.0,
        )
    } else {
        Space::new().width(14).into()
    });
    if let Some(i) = n.icon {
        line = line.push(icon(cx, i, 16.0));
    }
    line = line.push(text(n.label.clone()).size(lt.theme().typography.size_md));
    let row_el = lt.kit_button(
        Kit::Row { selected: false },
        line,
        if branch {
            Msg::TreeToggle(key.to_string())
        } else {
            Msg::Noop
        },
    );
    col = col.push(container(row_el).width(Length::Fill));
    if branch && open {
        for (i, c) in n.children.iter().enumerate() {
            col = tree(col, c, depth + 1, &path_key(key, i), cx);
        }
    }
    col
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
        assert_eq!(snap(2.4, 1.0, 5.0, Some(1.0)), 2.0);
    }

    #[test]
    fn readouts_drop_the_fraction_of_whole_numbers() {
        assert_eq!(num_text(40.0, None), "40");
        assert_eq!(num_text(3.0, Some(1.0)), "3");
        assert_eq!(num_text(3.25, None), "3.2");
    }

    #[test]
    fn the_stepper_never_leaves_its_range() {
        assert_eq!(stepped(0, -1), STEPPER_RANGE.0);
        assert_eq!(stepped(STEPPER_RANGE.1, 1), STEPPER_RANGE.1);
        assert_eq!(stepped(5, 1), 6);
    }

    #[test]
    fn tree_rows_flip_their_default_when_toggled() {
        let mut t = HashSet::new();
        assert!(is_open(&t, "0", true) && !is_open(&t, "0", false));
        t.insert("0".to_string());
        assert!(!is_open(&t, "0", true) && is_open(&t, "0", false));
        assert!(is_open(&t, "1", true));
    }

    #[test]
    fn path_keys_nest() {
        assert_eq!(path_key("", 2), "2");
        assert_eq!(path_key("2", 0), "2.0");
        assert_ne!(path_key("1", 12), path_key("11", 2));
    }
}
