//! Walks the model's [`Node`] tree and builds iced widgets.

use std::collections::HashSet;

use dodai_core::{ButtonVariant as V, ControlSize, Kit, StepState, Tone};
use dodai_iced::IcedTheme;
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Font, Gap, Ink, TextRole as Role, TextStyle, TreeNode};
use dodai_showcase_model::{Action, Node as Demo, Showcase, Value};
use iced::widget::{
    checkbox, column, container, progress_bar, radio, rich_text, row, scrollable, slider, span,
    stack, text, text_input, toggler, vertical_slider, Space,
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
    g.px(lt.theme())
}

/// The iced font of a model font.
fn font(lt: &IcedTheme, f: Font) -> iced::Font {
    match f {
        Font::Mono => lt.font_mono(),
        Font::Display => lt.font_display(),
        Font::Body => lt.font(),
    }
}

fn txt<'a>(cx: &Cx, s: &str, r: Role) -> Element<'a, Msg> {
    let th = cx.lt.theme();
    let st = r.resolve(th);
    let c = dodai_iced::color(st.ink.color(th));
    text(st.cased(th, s))
        .size(st.size)
        .font(font(cx.lt, st.font))
        .style(move |_| text::Style { color: Some(c) })
        .into()
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
            let across = if s.centre {
                alignment::Vertical::Center
            } else {
                alignment::Vertical::Top
            };
            // A row holding a growing gap takes the whole width for it to grow into.
            let grows = s.items.iter().any(|n| matches!(n, Demo::Grow));
            let w = s.width.map_or(
                if grows { Length::Fill } else { Length::Shrink },
                Length::Fixed,
            );
            match (s.axis, s.wrap) {
                (Axis::Row, true) => row(items)
                    .spacing(g)
                    .align_y(across)
                    .width(s.width.map_or(Length::Fill, Length::Fixed))
                    .wrap()
                    .vertical_spacing(g)
                    .into(),
                (Axis::Row, false) => row(items).spacing(g).align_y(across).width(w).into(),
                (Axis::Column, _) if s.centre => column(items)
                    .spacing(g)
                    .width(w)
                    .align_x(alignment::Horizontal::Center)
                    .into(),
                (Axis::Column, _) => column(items)
                    .spacing(g)
                    .width(s.width.map_or(Length::Fill, Length::Fixed))
                    .into(),
            }
        }
        Demo::Text(t, r) => txt(cx, t, *r),
        Demo::Space(px) => Space::new().width(*px).height(*px).into(),
        Demo::Grow => Space::new().width(Length::Fill).into(),
        Demo::Swatch {
            name,
            color: c,
            detail,
        } => {
            let fill = dodai_iced::color(*c);
            let border = dodai_iced::color(th.palette.border);
            let ty = &th.typography;
            column![
                container("")
                    .width(Length::Fill)
                    .height(44)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(fill)),
                        border: iced::Border {
                            color: border,
                            width: 1.0,
                            radius: 0.0.into(),
                        },
                        ..container::Style::default()
                    }),
                txt(cx, name, styled(ty.size_xs, Font::Body, Ink::Text)),
                txt(cx, detail, styled(ty.size_xs, Font::Mono, Ink::Muted)),
            ]
            .spacing(4)
            .width(118)
            .into()
        }
        Demo::Sample { w, h, label } => {
            let mut s = lt.panel_box(false);
            s.padding = iced::Padding::ZERO;
            let ty = &th.typography;
            let inner = container(txt(cx, label, styled(ty.size_sm, Font::Body, Ink::Text)))
                .width(*w)
                .height(*h)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center);
            lt.shaped_box(inner, s)
        }
        Demo::Image {
            src: Some(src),
            video,
            contain,
            w,
            h,
            caption,
            ..
        } => picture(cx, src, (*w, *h), (*video, *contain), caption.as_deref()),
        Demo::Image {
            w,
            h,
            caption,
            on_image,
            ..
        } => image_placeholder(cx, *w, *h, caption.as_deref(), *on_image),
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
                Some(l) => txt(cx, l, styled(th.typography.size_xs, Font::Body, Ink::Text)),
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
        Demo::Button(b) if b.elevated => {
            let msg = b
                .counter
                .clone()
                .map_or(Msg::Noop, |k| Msg::Act(Action::Press(k)));
            let size = th.control_metrics(ControlSize::Medium).text_size;
            lt.kit_button(Kit::Elevated, text(lt.label(&b.label)).size(size), msg)
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
            // The Shape page's crop (cut-corner designs crop the left end by default).
            let default =
                dodai_showcase_model::tune_state::default_crop(&cx.model.values, lt.theme());
            let crop = (!b.plain).then(|| b.crop.or(default)).flatten();
            if b.surface.is_set() {
                let glass = lt.button_glass(&b.surface);
                let own = lt.with_glass(&b.surface);
                let el = own.button_cropped(&b.label, icon_el, b.variant, b.size, crop, msg);
                return lt.frosted(el, glass);
            }
            lt.button_cropped(&b.label, icon_el, b.variant, b.size, crop, msg)
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
            row![
                lt.button_cropped(
                    label,
                    None,
                    *variant,
                    ControlSize::Medium,
                    *crop,
                    Some(Msg::Noop)
                ),
                lt.button_sized(
                    "",
                    Some(icon(cx, arrow, 16.0)),
                    *variant,
                    ControlSize::Medium,
                    Some(Msg::Noop)
                ),
            ]
            .spacing(2)
            .into()
        }
        Demo::Fab { label, .. } => {
            let mut r = row![icon(cx, IconId::Plus, 24.0)]
                .spacing(sp.sm)
                .align_y(alignment::Vertical::Center);
            if let Some(l) = label {
                r = r.push(text(lt.label(l)).size(th.typography.size_md));
            }
            lt.kit_button(
                Kit::Fab {
                    extended: label.is_some(),
                },
                r,
                Msg::Noop,
            )
        }
        Demo::Toolbar {
            floating,
            vibrant,
            icons,
        } => {
            let items = row(icons.iter().map(|&i| {
                lt.button_sized(
                    "",
                    Some(icon(cx, i, 18.0)),
                    V::Ghost,
                    ControlSize::Medium,
                    Some(Msg::Noop),
                )
            }))
            .spacing(sp.xs)
            .align_y(alignment::Vertical::Center);
            let bar = lt.kit_box(
                Kit::Toolbar {
                    floating: *floating,
                    vibrant: *vibrant,
                },
                items,
            );
            if *floating {
                bar
            } else {
                container(bar).width(Length::Fill).into()
            }
        }
        Demo::Choice(ch) if ch.connected => {
            // One bordered strip; the picked segment filled (with a check on tonal designs).
            let at = vals.index(&ch.bind);
            let tonal = th.look().tonal;
            let h = if tonal {
                th.sizes().segmented_height
            } else {
                th.shape.control_height - 6.0
            };
            let segs = row(ch.labels.iter().enumerate().map(|(i, l)| {
                let on = i == at;
                let label = if on && tonal {
                    format!("\u{2713} {}", lt.label(l))
                } else {
                    lt.label(l)
                };
                lt.kit_button(
                    Kit::Segment { selected: on },
                    container(text(label).size(th.typography.size_md))
                        .height(h)
                        .align_y(alignment::Vertical::Center),
                    Msg::Act(Action::Set(ch.bind.key.clone(), Value::Index(i))),
                )
            }))
            .spacing(if tonal { 0.0 } else { sp.half() });
            lt.kit_box(Kit::Segmented, segs)
        }
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
            let look = dodai_iced::TabsLook {
                secondary: ch.secondary,
                disabled: ch.disabled.clone(),
                icons: ch.icons.clone(),
            };
            lt.tabs_with(&labels, at, &look, move |i| {
                Msg::Act(Action::Set(key.clone(), Value::Index(i)))
            })
        }
        Demo::Tag { label, tone, solid } => lt.tag(label, *tone, *solid),
        Demo::Chip(ch) => {
            let on = vals.flag(&ch.bind);
            let mut r = row![].spacing(6).align_y(alignment::Vertical::Center);
            if ch.filter && on {
                r = r.push(icon(cx, IconId::Check, 16.0));
            } else if let Some(i) = ch.icon {
                r = r.push(icon(cx, i, 16.0));
            }
            r = r.push(text(ch.label.clone()).size(th.typography.size_sm));
            if ch.removable {
                r = r.push(icon(cx, IconId::Close, 14.0));
            }
            let kit = if ch.elevated {
                Kit::Elevated
            } else {
                Kit::Chip { selected: on }
            };
            lt.kit_button(
                kit,
                r,
                Msg::Act(Action::Set(ch.bind.key.clone(), Value::Bool(!on))),
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
                    let mut w = checkbox(vals.flag(&t.bind) || t.mixed)
                        .label(t.label.clone())
                        .style(lt.checkbox());
                    if t.mixed {
                        // Half-way: the box drawn on with a dash instead of the check.
                        w = w.icon(iced::widget::checkbox::Icon {
                            font: iced::Font::DEFAULT,
                            code_point: '\u{2212}',
                            size: None,
                            line_height: iced::widget::text::LineHeight::default(),
                            shaping: iced::widget::text::Shaping::Advanced,
                        });
                    }
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
        Demo::Select {
            key,
            labels,
            selected,
            placeholder,
        } => {
            let (list, key) = (labels.clone(), key.clone());
            let chosen = selected.and_then(|i| labels.get(i).cloned());
            iced::widget::pick_list(labels.clone(), chosen, move |s: String| {
                let at = list.iter().position(|x| *x == s).unwrap_or(0);
                Msg::Set(key.clone(), Value::Index(at))
            })
            .placeholder(placeholder.clone())
            .padding(sp.sm)
            .width(Length::Fixed(380.0))
            .style(lt.pick_list())
            .menu_style(lt.menu())
            .into()
        }
        Demo::Shader(spec) => crate::shaders::view(spec, lt.theme()),
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
            let disabled = s.disabled;
            let mut sl = vertical_slider(min..=max, v, move |x| {
                if disabled {
                    return Msg::Noop;
                }
                Msg::Set(key.clone(), Value::Num(snap(x, min, max, step)))
            })
            .height(s.length.unwrap_or(120.0))
            .style(lt.slider_with(s.disabled));
            if let Some(st) = s.step {
                sl = sl.step(st);
            }
            sl.into()
        }
        Demo::Slider(s) => {
            let v = vals.num(&s.bind);
            let key = s.bind.key.clone();
            let (min, max, step) = (s.min, s.max, s.step);
            let disabled = s.disabled;
            let mut sl = slider(min..=max, v, move |x| {
                if disabled {
                    return Msg::Noop;
                }
                Msg::Set(key.clone(), Value::Num(snap(x, min, max, step)))
            })
            .style(lt.slider_with(s.disabled));
            if let Some(st) = s.step {
                sl = sl.step(st);
            }
            let value_text = match s.format {
                dodai_showcase_model::node::SliderFormat::Plain => num_text(v, s.step),
                f => f.text(v),
            };
            // Ticks, a centred fill and the bubble need the canvas slider.
            let track: Element<'_, Msg> = if s.ticks || s.centered || s.bubble.is_some() {
                let key = s.bind.key.clone();
                let spec = dodai_iced::RangeSpec {
                    min,
                    max,
                    step,
                    ticks: s.ticks,
                    vertical: None,
                };
                let look = dodai_iced::SliderLook {
                    centered: s.centered,
                    bubble: s.bubble.map(|above| (above, value_text.clone())),
                    disabled,
                };
                lt.slider_canvas(v, spec, look, move |x| {
                    Msg::Set(key.clone(), Value::Num(snap(x, min, max, step)))
                })
            } else {
                sl.into()
            };
            let mut r = row![].spacing(sp.md).align_y(alignment::Vertical::Center);
            if let Some(i) = s.icon {
                r = r.push(icon(cx, i, 20.0));
            }
            r = r.push(container(track).width(Length::Fill));
            if s.readout {
                r = r.push(txt(cx, &value_text, Role::Mono));
            }
            r.into()
        }
        Demo::Range(r) => {
            let (lo_key, hi_key) = (r.lo.key.clone(), r.hi.key.clone());
            lt.range_slider(
                (vals.num(&r.lo), vals.num(&r.hi)),
                dodai_iced::RangeSpec {
                    min: r.min,
                    max: r.max,
                    step: r.step,
                    ticks: r.ticks,
                    vertical: r.vertical,
                },
                move |a, b| {
                    Msg::SetAll(vec![
                        (lo_key.clone(), Value::Num(a)),
                        (hi_key.clone(), Value::Num(b)),
                    ])
                },
            )
        }
        Demo::Progress { value, tone } => container(
            progress_bar(0.0..=1.0, *value)
                .style(lt.progress(*tone))
                .girth(10),
        )
        .width(Length::Fill)
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
            let mut col = column![].spacing(sp.xs).width(Length::Fill);
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
                col = col.push(lt.kit_box(
                    Kit::Row {
                        selected: r.selected,
                    },
                    line,
                ));
            }
            col.into()
        }
        Demo::Pick { bind, items } => {
            let at = vals.index(bind);
            column(items.iter().enumerate().map(|(i, title)| {
                lt.kit_button(
                    Kit::Row { selected: i == at },
                    container(txt(cx, title, Role::Body)).width(Length::Fill),
                    Msg::Act(Action::Set(bind.key.clone(), Value::Index(i))),
                )
            }))
            .width(Length::Fill)
            .into()
        }
        Demo::Divider => lt.divider().into(),
        Demo::Card {
            title,
            body,
            variant,
            surface,
        } => {
            let mut col = column![].spacing(sp.sm);
            if !title.is_empty() {
                col = col.push(txt(cx, title, Role::Label));
            }
            col = col.push(view(body, cx));
            // As wide as its body: a body that fills makes the card fill.
            container(lt.kit_box_with(Kit::Card(*variant), col, surface)).into()
        }
        Demo::Alert {
            tone,
            text: t,
            banner,
            action,
            dismissible,
        } => {
            let kit = Kit::Alert {
                tone: *tone,
                banner: *banner,
            };
            let ink = th.kit(kit, false).fg;
            let mut line = row![].spacing(sp.sm).align_y(alignment::Vertical::Center);
            if !*banner {
                let ic = match tone {
                    Tone::Danger | Tone::Warning => IconId::Warning,
                    Tone::Success => IconId::CheckCircle,
                    Tone::Info => IconId::Info,
                };
                line = line.push(lt.icon(ic, 18.0, Some(ink)));
            }
            line = line
                .push(container(text(t.clone()).size(th.typography.size_md)).width(Length::Fill));
            if let Some(a) = action {
                line = line.push(
                    rich_text([span(a.clone()).underline(true).size(th.typography.size_md)])
                        .on_link_click(|()| Msg::Noop),
                );
            }
            if *dismissible {
                line = line.push(lt.icon(IconId::Close, 18.0, Some(ink)));
            }
            container(lt.kit_box(kit, line)).width(Length::Fill).into()
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
        Demo::Frame {
            h,
            body,
            scrim,
            layer,
            place,
        } => {
            use dodai_showcase_model::node::Place;
            let edge = dodai_iced::color(th.palette.border);
            let base = container(view(body, cx))
                .padding(sp.md)
                .width(Length::Fill)
                .height(*h)
                .style(move |_| container::Style {
                    border: iced::Border {
                        color: edge,
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    ..container::Style::default()
                });
            let (ax, ay) = match place {
                Place::Centre => (alignment::Horizontal::Center, alignment::Vertical::Center),
                Place::Start => (alignment::Horizontal::Left, alignment::Vertical::Top),
                Place::End => (alignment::Horizontal::Right, alignment::Vertical::Top),
                Place::Bottom => (alignment::Horizontal::Left, alignment::Vertical::Bottom),
            };
            let side = matches!(place, Place::Start | Place::End);
            let layer = container(view(layer, cx))
                .width(if *place == Place::Bottom {
                    Length::Fill
                } else {
                    Length::Shrink
                })
                .height(if side { Length::Fill } else { Length::Shrink });
            let mut st = iced::widget::stack![base];
            if let Some(c) = scrim {
                let c = dodai_iced::color(*c);
                st = st.push(
                    container(Space::new())
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(c)),
                            ..container::Style::default()
                        }),
                );
            }
            st.push(
                container(layer)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(ax)
                    .align_y(ay),
            )
            .into()
        }
        Demo::Gizmo { yaw, pitch, size } => {
            let (yk, pk) = (yaw.key.clone(), pitch.key.clone());
            lt.axis_gizmo((vals.num(yaw), vals.num(pitch)), *size, move |y, p| {
                Msg::SetAll(vec![
                    (yk.clone(), Value::Num(y)),
                    (pk.clone(), Value::Num(p)),
                ])
            })
        }
        Demo::Anchor {
            open,
            trigger,
            panel,
        } => {
            let shown = vals.flag(open);
            let flip = Msg::Set(open.key.clone(), Value::Bool(!shown));
            let trigger = iced::widget::mouse_area(view(trigger, cx)).on_press(flip);
            dodai_iced::anchored(trigger, shown.then(|| view(panel, cx)), sp.xs)
        }
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
            for slot in dodai_core::paging::page_window(at + 1, *pages, 7) {
                let dodai_core::paging::PageItem::Page(p) = slot else {
                    r = r.push(txt(cx, "…", Role::Muted));
                    continue;
                };
                let i = p - 1;
                r = r.push(
                    lt.kit_button(
                        Kit::Page { selected: i == at },
                        container(text(p.to_string()).size(th.typography.size_sm))
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
            let mut col = column![].width(Length::Fill);
            // Keys start with the first root's label: two trees in a section fold apart.
            let id = nodes.first().map_or("", |n| n.label.as_str());
            for (i, n) in nodes.iter().enumerate() {
                col = tree(col, n, 0, &format!("{id}:{i}"), cx);
            }
            col.into()
        }
        Demo::Table(tb) => table(cx, tb),
        Demo::Avatar(s) => lt.avatar(s, 40.0),
        Demo::Tooltip {
            text: t,
            rich,
            title,
            action,
        } => {
            let ty = &th.typography;
            let mut body = column![].spacing(sp.sm);
            if let Some(h) = title {
                body = body.push(text(h.clone()).size(ty.size_md));
            }
            body = body.push(text(t.clone()).size(ty.size_sm));
            if let Some(a) = action {
                let c = dodai_iced::color(th.palette.primary);
                body = body.push(text(lt.label(a)).size(ty.size_sm).color(c));
            }
            container(lt.kit_box(Kit::Tooltip { rich: *rich }, body))
                .width(Length::Shrink)
                .into()
        }
        Demo::Code { text: code, lang } => code_block(cx, code, lang),
        Demo::Rich { spans, role } => rich(cx, spans, *role),
        Demo::Quote(body) => {
            let bar = dodai_iced::color(th.palette.primary);
            row![
                container("")
                    .width(2)
                    .height(Length::Fill)
                    .style(move |_| container::Style {
                        background: Some(iced::Background::Color(bar)),
                        ..container::Style::default()
                    }),
                view(body, cx),
            ]
            .spacing(sp.md)
            .height(Length::Shrink)
            .into()
        }
        Demo::AvatarGroup { names, max, size } => {
            let shown = max.unwrap_or(names.len()).min(names.len());
            let mut r = row![].spacing(-size * 0.25);
            for n in &names[..shown] {
                r = r.push(lt.avatar(n, *size));
            }
            if names.len() > shown {
                r = r.push(lt.avatar(&format!("+{}", names.len() - shown), *size));
            }
            r.into()
        }
        Demo::Overlay(items) => {
            let mut layers = items.iter();
            let base = layers
                .next()
                .map_or_else(|| Space::new().into(), |n| view(n, cx));
            let mut st = stack![base];
            for n in layers {
                st = st.push(
                    container(view(n, cx))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(alignment::Horizontal::Center)
                        .align_y(alignment::Vertical::Center),
                );
            }
            st.into()
        }
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
        Demo::Empty {
            icon: glyph,
            title,
            detail,
        } => {
            let ty = &th.typography;
            container(
                column![
                    txt(cx, glyph, styled(ty.size_xxl, Font::Body, Ink::Muted)),
                    txt(cx, title, styled(ty.size_lg, Font::Body, Ink::Text)),
                    txt(cx, detail, styled(ty.size_sm, Font::Body, Ink::Muted)),
                ]
                .spacing(sp.sm)
                .align_x(alignment::Horizontal::Center),
            )
            .padding(sp.lg)
            .align_x(alignment::Horizontal::Center)
            .into()
        }
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
        Kit::Row {
            selected: n.selected,
        },
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

/// A text role at `size` in `font` and `ink`.
fn styled(size: f32, font: Font, ink: Ink) -> Role {
    Role::Styled(TextStyle {
        size,
        font,
        ink,
        caps: false,
    })
}

/// A framed picture slot (no picture loads: the frame shows "image unavailable", as the gpui
/// showcase's blank image does) with its caption below or on the design's fade.
fn image_placeholder<'a>(
    cx: &Cx,
    w: f32,
    h: f32,
    caption: Option<&str>,
    on_image: bool,
) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    let ty = &th.typography;
    let frame_style = dodai_iced::BoxStyle {
        fill: Some(th.palette.surface_raised),
        stroke: Some(th.palette.border),
        glow: false,
        hard_shadow: false,
        padding: iced::Padding::ZERO,
    };
    let fallback = container(txt(
        cx,
        "image unavailable",
        styled(ty.size_md, Font::Body, Ink::Muted),
    ))
    .width(w)
    .height(h)
    .align_x(alignment::Horizontal::Center)
    .align_y(alignment::Vertical::Center);
    match (on_image, caption) {
        (true, Some(c)) => {
            let fade_h = (h * 0.5).max(th.sizes().touch_target);
            let shadow = th.palette.roles.shadow;
            let ink = dodai_iced::color(th.on(shadow.with_alpha(1.0)));
            // The design's fade as stacked bands, strongest at the bottom edge.
            let stops = th.fade_stops();
            let mut bands = column![Space::new().height(Length::Fill)]
                .width(w)
                .height(h);
            let steps = 24;
            for k in 0..steps {
                let at = 1.0 - (k as f32 + 0.5) / steps as f32;
                let a = fade_alpha(&stops, at);
                let c = dodai_iced::color(shadow.with_alpha(a));
                bands = bands.push(
                    container("")
                        .width(Length::Fill)
                        .height(fade_h / steps as f32)
                        .style(move |_| container::Style {
                            background: Some(iced::Background::Color(c)),
                            ..container::Style::default()
                        }),
                );
            }
            let label = container(
                text(c.to_string())
                    .size(ty.size_sm)
                    .font(lt.font())
                    .style(move |_| text::Style { color: Some(ink) }),
            )
            .width(w)
            .height(h)
            .padding(iced::Padding {
                left: th.spacing.md,
                right: th.spacing.md,
                bottom: th.spacing.sm,
                top: 0.0,
            })
            .align_y(alignment::Vertical::Bottom);
            lt.shaped_box(stack![fallback, bands, label], frame_style)
        }
        (_, c) => {
            let mut col = column![lt.shaped_box(fallback, frame_style)]
                .spacing(th.spacing.xs)
                .width(w);
            if let Some(c) = c {
                col = col.push(txt(cx, c, Role::Muted));
            }
            col.into()
        }
    }
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

/// A data table: a header row, then rows in alternating bands; fixed-width columns keep their
/// width, the rest share the row; scrolls inside `height` when set.
fn table<'a>(cx: &Cx, tb: &dodai_showcase_model::node::Table) -> Element<'a, Msg> {
    use dodai_showcase_model::node::Align;
    let lt = cx.lt;
    let cells = |items: &[String], r: Role| -> Vec<Element<'a, Msg>> {
        items
            .iter()
            .zip(&tb.columns)
            .map(|(c, col)| {
                let x = match col.align {
                    Align::Left => alignment::Horizontal::Left,
                    Align::Center => alignment::Horizontal::Center,
                    Align::Right => alignment::Horizontal::Right,
                };
                container(txt(cx, c, r))
                    .width(col.width.map_or(Length::FillPortion(1), Length::Fixed))
                    .align_x(x)
                    .into()
            })
            .collect()
    };
    let heads: Vec<String> = tb.columns.iter().map(|c| c.title.clone()).collect();
    let head = lt.kit_box(Kit::TableHead, row(cells(&heads, Role::Label)));
    let mut body = column![].width(Length::Fill);
    for (i, r) in tb.rows.iter().enumerate() {
        body = body.push(lt.kit_box(Kit::TableRow(i % 2 == 1), row(cells(r, Role::Body))));
    }
    match tb.height {
        Some(h) => column![head, scrollable(body).height(h)].into(),
        None => column![head, body].into(),
    }
}

/// The palette colour of a syntax token.
fn token_color(t: &dodai_core::Theme, k: dodai_core::syntax::CodeToken) -> dodai_core::Color {
    use dodai_core::syntax::CodeToken as K;
    let p = &t.palette;
    match k {
        K::Keyword => p.primary,
        K::Str => p.success,
        K::Comment => p.text_muted,
        K::Number => p.warning,
        K::Type => p.info,
    }
}

/// A code listing: the language over syntax-coloured lines in the code box.
fn code_block<'a>(cx: &Cx, code: &str, lang: &str) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    let size = th.typography.size_sm;
    let plain = dodai_iced::color(th.palette.text);
    let mut lines = column![];
    for (l, spans) in code
        .split('\n')
        .zip(dodai_core::syntax::highlight_lines(code, lang))
    {
        let mut at = 0;
        let mut runs: Vec<iced::widget::text::Span<'a, (), iced::Font>> = Vec::new();
        for (r, k) in spans {
            if r.start > at {
                runs.push(span(l[at..r.start].to_string()).color(plain));
            }
            runs.push(span(l[r.clone()].to_string()).color(dodai_iced::color(token_color(th, k))));
            at = r.end;
        }
        if at < l.len() || runs.is_empty() {
            runs.push(span(l[at..].to_string()).color(plain));
        }
        lines = lines.push(
            rich_text(runs)
                .size(size)
                .font(lt.font_mono())
                .on_link_click(|()| Msg::Noop),
        );
    }
    let mut col = column![].spacing(th.spacing.xs);
    if !lang.is_empty() {
        col = col.push(txt(cx, &lang.to_uppercase(), Role::Caption));
    }
    container(lt.kit_box(Kit::Code, col.push(lines).width(Length::Fill)))
        .width(Length::Fill)
        .into()
}

/// Inline-styled runs as one wrapped paragraph.
fn rich<'a>(cx: &Cx, spans: &[dodai_showcase_model::node::Span], role: Role) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    let st = role.resolve(th);
    let base = font(lt, st.font);
    let ink = dodai_iced::color(st.ink.color(th));
    let runs: Vec<iced::widget::text::Span<'a, (), iced::Font>> = spans
        .iter()
        .map(|s| {
            let mut f = if s.code { lt.font_mono() } else { base };
            if s.bold {
                f.weight = iced::font::Weight::Bold;
            }
            if s.italic {
                f.style = iced::font::Style::Italic;
            }
            let mut r = span(s.text.clone()).font(f).color(ink);
            if s.code {
                r = r
                    .color(dodai_iced::color(th.palette.primary))
                    .background(dodai_iced::color(th.palette.surface_raised));
            }
            if s.link {
                r = r.color(dodai_iced::color(th.palette.link)).underline(true);
            }
            r
        })
        .collect();
    rich_text(runs)
        .size(st.size)
        .on_link_click(|()| Msg::Noop)
        .into()
}

/// A picture file in the design's frame: cover or contain, a play badge for a video poster, the
/// caption under it.
fn picture<'a>(
    cx: &Cx,
    src: &str,
    (w, h): (f32, f32),
    (video, contain): (bool, bool),
    caption: Option<&str>,
) -> Element<'a, Msg> {
    let lt = cx.lt;
    let th = lt.theme();
    let Some(p) = dodai_showcase_model::media::picture(src) else {
        return image_placeholder(cx, w, h, caption, false);
    };
    let handle = iced::widget::image::Handle::from_rgba(p.width, p.height, p.rgba.to_vec());
    let fit = if contain {
        iced::ContentFit::Contain
    } else {
        iced::ContentFit::Cover
    };
    let pic = iced::widget::image(handle)
        .width(w)
        .height(h)
        .content_fit(fit);
    let frame = dodai_iced::BoxStyle {
        fill: Some(th.palette.surface_raised),
        stroke: Some(th.palette.border),
        glow: false,
        hard_shadow: false,
        padding: iced::Padding::ZERO,
    };
    let mut layers = stack![pic];
    if video {
        let d = (h * 0.28).clamp(28.0, 56.0);
        let (bg, ring) = (
            dodai_iced::color(th.palette.background.with_alpha(0.75)),
            dodai_iced::color(th.palette.primary),
        );
        let badge = container(text("▶").size(d * 0.45).color(ring))
            .width(d)
            .height(d)
            .align_x(alignment::Horizontal::Center)
            .align_y(alignment::Vertical::Center)
            .style(move |_| container::Style {
                background: Some(iced::Background::Color(bg)),
                border: iced::Border {
                    color: ring,
                    width: 1.0,
                    radius: (d / 2.0).into(),
                },
                ..container::Style::default()
            });
        layers = layers.push(
            container(badge)
                .width(w)
                .height(h)
                .align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Center),
        );
    }
    let mut col = column![lt.shaped_box(layers, frame)]
        .spacing(th.spacing.xs)
        .width(w);
    if let Some(c) = caption {
        col = col.push(txt(cx, c, Role::Muted));
    }
    col.into()
}
