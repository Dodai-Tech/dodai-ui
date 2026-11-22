//! Walks the model's [`Node`] tree and draws dodai-ui-egui widgets.
//!
//! egui is immediate mode: every frame builds the page again, and a click or edit pushes an
//! [`Action`] onto `Cx::acts`, which the app applies to the model after the frame.

use dodai_ui_core::{ButtonVariant, ControlSize, Kit, Theme, ToggleKind, Tone};
use dodai_ui_egui::{
    color32, kit_box, Button, Checkbox, Icon, ProgressBar, Radio, Slider, Spinner, Tag, Toggle,
};
use dodai_ui_icons::IconId;
use dodai_ui_showcase_model::node::{Align as Al, Axis, Font, Ink, Place, TextRole as Role};
use dodai_ui_showcase_model::{Action, Node, Value, Values};
use egui::{pos2, vec2, Align, Color32, Layout, RichText, Sense, Stroke, Ui};

/// What the walk reads and writes.
pub struct Cx {
    pub theme: Theme,
    pub vals: Values,
    /// Events of this frame, applied to the model afterwards.
    pub acts: Vec<Action>,
    /// Seconds since start (spinners, shimmer).
    pub time: f64,
}

impl Cx {
    fn set(&mut self, key: &str, v: Value) {
        self.acts.push(Action::Set(key.into(), v));
    }
}

/// Counter range of the number stepper demo.
const STEPPER_RANGE: (i32, i32) = (0, 20);

/// Text of a role: size, family, colour and casing resolved by the model.
pub fn text(theme: &Theme, role: Role, s: &str) -> RichText {
    let st = role.resolve(theme);
    let shown = st.cased(theme, s);
    let mut t = RichText::new(shown)
        .size(st.size)
        .color(color32(st.ink.color(theme)));
    match st.font {
        Font::Mono => t = t.monospace(),
        Font::Display => t = t.strong(),
        Font::Body => {}
    }
    t
}

fn styled(theme: &Theme, size: f32, font: Font, ink: Ink, s: &str) -> RichText {
    text(
        theme,
        Role::Styled(dodai_ui_showcase_model::node::TextStyle {
            size,
            font,
            ink,
            caps: false,
        }),
        s,
    )
}

fn alert_icon(tone: Tone) -> IconId {
    match tone {
        Tone::Info => IconId::Info,
        Tone::Success => IconId::CheckCircle,
        Tone::Warning | Tone::Danger => IconId::Warning,
    }
}

/// Snap `v` to multiples of `step` within `min..=max`.
fn snap(v: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let v = v.clamp(min, max);
    match step {
        Some(s) if s > 0.0 => (min + ((v - min) / s).round() * s).clamp(min, max),
        _ => v,
    }
}

/// A glyph of any icon reference, rasterised once per theme, size and fill.
fn glyph(
    ui: &mut Ui,
    theme: &Theme,
    icon: &dodai_ui_showcase_model::icons::IconRef,
    size: f32,
    filled: bool,
    ink: Color32,
) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let px = (size * ui.ctx().pixels_per_point()).round().max(8.0) as u32;
    let key = ui.make_persistent_id(("glyph", icon.name(), theme.id(), filled, px));
    let tex: Option<egui::TextureHandle> = ui.ctx().data(|d| d.get_temp(key));
    let tex = tex.or_else(|| {
        let g = icon.glyph(theme, filled)?;
        let rgba: Vec<u8> = g
            .coverage(px)
            .iter()
            .flat_map(|&a| [255, 255, 255, a])
            .collect();
        let img = egui::ColorImage::from_rgba_unmultiplied([px as usize, px as usize], &rgba);
        let h = ui
            .ctx()
            .load_texture(format!("{key:?}"), img, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|d| d.insert_temp(key, h.clone()));
        Some(h)
    });
    if let Some(t) = tex {
        ui.painter().image(
            t.id(),
            rect,
            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            ink,
        );
    }
}

/// Draw `node`.
pub fn view(ui: &mut Ui, node: &Node, cx: &mut Cx) {
    let theme = cx.theme.clone();
    let t = &theme;
    let p = &t.palette;
    match node {
        Node::Stack(s) => {
            let gap = s.gap.px(t);
            let body = |ui: &mut Ui, cx: &mut Cx| {
                ui.spacing_mut().item_spacing = vec2(gap, gap);
                for n in &s.items {
                    view(ui, n, cx);
                }
            };
            let run = |ui: &mut Ui, cx: &mut Cx| match (s.axis, s.wrap, s.centre) {
                (Axis::Row, true, centre) => wrapped(ui, &s.items, gap, centre, cx),
                (Axis::Row, false, true) => {
                    ui.horizontal(|ui| body(ui, cx));
                }
                (Axis::Row, false, false) => {
                    ui.horizontal_top(|ui| body(ui, cx));
                }
                (Axis::Column, _, true) => {
                    ui.vertical_centered(|ui| body(ui, cx));
                }
                (Axis::Column, _, false) => {
                    ui.vertical(|ui| body(ui, cx));
                }
            };
            if let Some(w) = s.width {
                ui.scope_builder(
                    egui::UiBuilder::new().layout(Layout::top_down(Align::Min)),
                    |ui| {
                        ui.set_width(w);
                        run(ui, cx)
                    },
                );
            } else {
                run(ui, cx);
            }
        }
        Node::Text(s, role) => {
            ui.label(text(t, *role, s));
        }
        Node::Space(px) => {
            ui.add_space(*px);
        }
        Node::Grow => {}
        Node::Swatch {
            name,
            color,
            detail,
        } => {
            ui.allocate_ui(vec2(118.0, 0.0), |ui| {
                ui.set_width(118.0);
                ui.spacing_mut().item_spacing.y = 4.0;
                let (r, _) = ui.allocate_exact_size(vec2(118.0, 44.0), Sense::hover());
                ui.painter().rect_filled(r, 0.0, color32(*color));
                ui.painter().rect_stroke(
                    r,
                    0.0,
                    Stroke::new(1.0, color32(p.border)),
                    egui::StrokeKind::Inside,
                );
                ui.label(styled(t, t.typography.size_xs, Font::Body, Ink::Text, name));
                ui.label(styled(
                    t,
                    t.typography.size_xs,
                    Font::Mono,
                    Ink::Muted,
                    detail,
                ));
            });
        }
        Node::Sample { w, h, label } => {
            let (r, _) = ui.allocate_exact_size(vec2(*w, *h), Sense::hover());
            let shapes = dodai_ui_egui::paint::box_shapes_for(
                t,
                r,
                Some(p.surface),
                Some(p.border),
                false,
                false,
                Some(dodai_ui_core::SurfaceClass::Panel),
            );
            ui.painter().add(egui::Shape::Vec(shapes));
            ui.painter().text(
                r.center(),
                egui::Align2::CENTER_CENTER,
                label,
                egui::FontId::proportional(t.typography.size_sm),
                color32(p.text),
            );
        }
        Node::Logo(size) => {
            let img = egui::Image::new(egui::include_image!(
                "../../../../assets/icons/dodai-ui-128.png"
            ))
            .fit_to_exact_size(vec2(*size, *size));
            ui.add(img);
        }
        Node::Image {
            src,
            w,
            h,
            caption,
            contain,
            video,
            on_image,
        } => {
            let _ = (contain, on_image);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = t.spacing.xs;
                let (r, _) = ui.allocate_exact_size(vec2(*w, *h), Sense::hover());
                ui.painter()
                    .rect_filled(r, t.shape.radius, color32(p.surface_raised));
                ui.painter().rect_stroke(
                    r,
                    t.shape.radius,
                    Stroke::new(t.shape.border_width, color32(p.border)),
                    egui::StrokeKind::Inside,
                );
                let shown = match src
                    .as_deref()
                    .and_then(dodai_ui_showcase_model::media::picture)
                {
                    Some(pic) => {
                        let img = egui::ColorImage::from_rgba_unmultiplied(
                            [pic.width as usize, pic.height as usize],
                            &pic.rgba,
                        );
                        let h = ui.ctx().load_texture(
                            format!("img-{}", src.as_deref().unwrap_or("")),
                            img,
                            egui::TextureOptions::LINEAR,
                        );
                        ui.painter().image(
                            h.id(),
                            r.shrink(1.0),
                            egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                            Color32::WHITE,
                        );
                        true
                    }
                    None => false,
                };
                if !shown {
                    ui.painter().text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        "image unavailable",
                        egui::FontId::proportional(t.typography.size_md),
                        color32(p.text_muted),
                    );
                }
                if *video {
                    ui.painter().circle_filled(
                        r.center(),
                        (h * 0.14).clamp(14.0, 28.0),
                        color32(p.background).gamma_multiply(0.75),
                    );
                    ui.painter().text(
                        r.center(),
                        egui::Align2::CENTER_CENTER,
                        "▶",
                        egui::FontId::proportional(h * 0.16),
                        color32(p.primary),
                    );
                }
                if let Some(c) = caption {
                    ui.label(text(t, Role::Muted, c));
                }
            });
        }
        Node::Block {
            w,
            h,
            fill,
            border,
            radius,
            label,
        } => {
            let size = vec2(
                if *w > 0.0 { *w } else { ui.available_width() },
                if *h > 0.0 { *h } else { 24.0 },
            );
            let (r, _) = ui.allocate_exact_size(size, Sense::hover());
            ui.painter().rect_filled(r, *radius, color32(*fill));
            if let Some(b) = border {
                ui.painter().rect_stroke(
                    r,
                    *radius,
                    Stroke::new(1.0, color32(*b)),
                    egui::StrokeKind::Inside,
                );
            }
            if let Some(l) = label {
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    l,
                    egui::FontId::proportional(t.typography.size_xs),
                    color32(p.text),
                );
            }
        }
        Node::Button(b) => {
            let r = if b.elevated {
                kit_box(ui, Kit::Elevated, true, |ui, _| {
                    ui.label(b.label.clone());
                })
                .response
                .interact(Sense::click())
            } else {
                let mut btn = Button::new(b.label.clone())
                    .variant(b.variant)
                    .size(b.size)
                    .enabled(!b.disabled && !b.loading);
                if let Some(i) = b.icon {
                    btn = btn.icon(i);
                }
                ui.add(btn)
            };
            if r.clicked() {
                if let Some(k) = &b.counter {
                    cx.acts.push(Action::Press(k.clone()));
                }
            }
        }
        Node::Split {
            label,
            variant,
            open,
            ..
        } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 1.0;
                ui.add(Button::new(label.clone()).variant(*variant));
                ui.add(
                    Icon::new(if *open {
                        IconId::ChevronUp
                    } else {
                        IconId::ChevronDown
                    })
                    .size(16.0),
                );
            });
        }
        Node::Fab { label, .. } => {
            kit_box(
                ui,
                Kit::Fab {
                    extended: label.is_some(),
                },
                true,
                |ui, _| {
                    ui.horizontal(|ui| {
                        ui.add(Icon::new(IconId::Plus).size(24.0));
                        if let Some(l) = label {
                            ui.label(l.clone());
                        }
                    });
                },
            );
        }
        Node::Toolbar {
            floating,
            vibrant,
            icons,
        } => {
            kit_box(
                ui,
                Kit::Toolbar {
                    floating: *floating,
                    vibrant: *vibrant,
                },
                false,
                |ui, fg| {
                    ui.horizontal(|ui| {
                        for i in icons {
                            ui.add(Icon::new(*i).size(18.0).color(fg));
                        }
                    });
                },
            );
        }
        Node::Choice(ch) => {
            let at = cx.vals.index(&ch.bind);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = if ch.connected { 0.0 } else { t.spacing.sm };
                for (i, l) in ch.labels.iter().enumerate() {
                    let v = if i == at {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    };
                    let on = !ch.disabled.contains(&i);
                    if ui
                        .add(
                            Button::new(l.clone())
                                .variant(v)
                                .size(ControlSize::Small)
                                .enabled(on),
                        )
                        .clicked()
                    {
                        cx.set(&ch.bind.key, Value::Index(i));
                    }
                }
            });
        }
        Node::Tabs(ch) => {
            let mut at = cx.vals.index(&ch.bind);
            let before = at;
            ui.add(dodai_ui_egui::Tabs::new(&mut at, ch.labels.clone()));
            if at != before {
                cx.set(&ch.bind.key, Value::Index(at));
            }
        }
        Node::Tag { label, tone, solid } => {
            ui.add(Tag::new(label.clone()).tone(*tone).solid(*solid));
        }
        Node::Chip(ch) => {
            let on = cx.vals.flag(&ch.bind);
            let r = kit_box(
                ui,
                if ch.elevated {
                    Kit::Elevated
                } else {
                    Kit::Chip { selected: on }
                },
                true,
                |ui, fg| {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 6.0;
                        let lead = if ch.filter && on {
                            Some(IconId::Check)
                        } else {
                            ch.icon
                        };
                        if let Some(i) = lead {
                            ui.add(Icon::new(i).size(16.0).color(fg));
                        }
                        ui.label(RichText::new(&ch.label).size(t.typography.size_sm));
                        if ch.removable {
                            ui.add(Icon::new(IconId::Close).size(14.0).color(fg));
                        }
                    });
                },
            );
            if r.response.interact(Sense::click()).clicked() {
                cx.set(&ch.bind.key, Value::Bool(!on));
            }
        }
        Node::Toggle(tg) => match tg.kind {
            ToggleKind::Switch => {
                let mut on = cx.vals.flag(&tg.bind);
                ui.horizontal(|ui| {
                    ui.add(Toggle::new(&mut on).enabled(!tg.disabled));
                    if !tg.label.is_empty() {
                        ui.label(text(t, Role::Body, &tg.label));
                    }
                });
                if on != cx.vals.flag(&tg.bind) {
                    cx.set(&tg.bind.key, Value::Bool(on));
                }
            }
            ToggleKind::Checkbox => {
                let mut on = cx.vals.flag(&tg.bind);
                ui.add(Checkbox::new(&mut on, tg.label.clone()).enabled(!tg.disabled));
                if on != cx.vals.flag(&tg.bind) {
                    cx.set(&tg.bind.key, Value::Bool(on));
                }
            }
            ToggleKind::Radio => {
                let (group, idx) = tg.radio.clone().unwrap_or((tg.bind.key.clone(), 0));
                let picked = cx.vals.index_of(&group, usize::MAX);
                let mut cur = if picked == usize::MAX {
                    if tg.bind.default {
                        idx
                    } else {
                        usize::MAX - 1
                    }
                } else {
                    picked
                };
                ui.add(Radio::new(&mut cur, idx, tg.label.clone()));
                if cur != picked && cur == idx {
                    cx.set(&group, Value::Index(idx));
                }
            }
        },
        Node::Slider(s) => {
            let mut v = cx.vals.num(&s.bind);
            let shown = s.format.text(v);
            ui.horizontal(|ui| {
                if let Some(ic) = s.icon {
                    ui.add(Icon::new(ic).size(20.0));
                }
                let w = if s.vertical {
                    24.0
                } else {
                    (ui.available_width() - 70.0).clamp(120.0, 360.0)
                };
                if ui
                    .add(Slider::new(&mut v, s.min..=s.max).width(w))
                    .changed()
                {
                    cx.set(&s.bind.key, Value::Num(snap(v, s.min, s.max, s.step)));
                }
                if s.readout {
                    ui.label(text(t, Role::Mono, &shown));
                }
            });
        }
        Node::Range(r) => {
            let (mut lo, mut hi) = (cx.vals.num(&r.lo), cx.vals.num(&r.hi));
            ui.horizontal(|ui| {
                let a = ui
                    .add(Slider::new(&mut lo, r.min..=r.max).width(140.0))
                    .changed();
                let b = ui
                    .add(Slider::new(&mut hi, r.min..=r.max).width(140.0))
                    .changed();
                if a || b {
                    let (l, h) = (lo.min(hi), lo.max(hi));
                    cx.set(&r.lo.key, Value::Num(snap(l, r.min, r.max, r.step)));
                    cx.set(&r.hi.key, Value::Num(snap(h, r.min, r.max, r.step)));
                }
            });
        }
        Node::Knob {
            bind,
            min,
            max,
            size,
            ..
        } => {
            let v = cx.vals.num(bind);
            let (rect, resp) = ui.allocate_exact_size(vec2(*size, *size), Sense::click_and_drag());
            let f = ((v - min) / (max - min)).clamp(0.0, 1.0);
            let c = rect.center();
            let r = size / 2.0 - 3.0;
            ui.painter().circle(
                c,
                r,
                color32(p.surface_raised),
                Stroke::new(2.0, color32(p.border)),
            );
            let a = std::f32::consts::PI * (0.75 + 1.5 * f);
            ui.painter().line_segment(
                [c, c + vec2(a.cos(), a.sin()) * r],
                Stroke::new(3.0, color32(p.primary)),
            );
            if resp.dragged() {
                let nv = (v - resp.drag_delta().y * (max - min) / 100.0).clamp(*min, *max);
                cx.set(&bind.key, Value::Num(nv));
            }
        }
        Node::Scroll { height, body } => {
            egui::ScrollArea::vertical()
                .id_salt(("scroll", *height as i32))
                .max_height(*height)
                .show(ui, |ui| view(ui, body, cx));
        }
        Node::Plot(pl) => {
            let (rect, _) = ui.allocate_exact_size(vec2(pl.w, pl.h), Sense::hover());
            let at = |q: &(f32, f32)| pos2(rect.min.x + q.0, rect.min.y + q.1);
            for f in &pl.fills {
                let pts = f.points.iter().map(at).collect::<Vec<_>>();
                fill_polygon(ui, &pts, color32(f.color));
            }
            for l in &pl.lines {
                let pts = l.points.iter().map(at).collect::<Vec<_>>();
                ui.painter().add(egui::Shape::line(
                    pts,
                    Stroke::new(l.width, color32(l.color)),
                ));
            }
        }
        Node::Select {
            key,
            labels,
            selected,
            placeholder,
        } => {
            let shown = selected.and_then(|i| labels.get(i)).unwrap_or(placeholder);
            egui::ComboBox::from_id_salt(key)
                .width(320.0)
                .selected_text(shown.clone())
                .show_ui(ui, |ui| {
                    for (i, l) in labels.iter().enumerate() {
                        if ui.selectable_label(*selected == Some(i), l).clicked() {
                            cx.set(key, Value::Index(i));
                        }
                    }
                });
        }
        Node::Shader(spec) => crate::shader::view(ui, spec, t),
        Node::Progress { value, tone } => {
            let mut b = ProgressBar::new(*value).width(ui.available_width().min(520.0));
            if let Some(tn) = tone {
                b = b.tone(*tn);
            }
            ui.add(b);
        }
        Node::Input(i) => {
            let mut s = cx.vals.text(&i.bind).to_string();
            ui.vertical(|ui| {
                if let Some(l) = &i.label {
                    ui.label(text(t, Role::Label, l));
                }
                ui.horizontal(|ui| {
                    if let Some(ic) = i.icon {
                        ui.add(Icon::new(ic).size(18.0));
                    }
                    let w = ui.available_width().min(420.0);
                    let r = if i.lines > 1 {
                        ui.add(
                            egui::TextEdit::multiline(&mut s)
                                .desired_rows(i.lines as usize)
                                .desired_width(w)
                                .hint_text(&i.placeholder),
                        )
                    } else {
                        ui.add(
                            egui::TextEdit::singleline(&mut s)
                                .password(i.password)
                                .desired_width(w)
                                .hint_text(&i.placeholder),
                        )
                    };
                    if r.changed() {
                        cx.set(&i.bind.key, Value::Text(s.clone()));
                    }
                });
            });
        }
        Node::List(rows) => {
            for (n, r) in rows.iter().enumerate() {
                if n > 0 {
                    ui.separator();
                }
                kit_box(
                    ui,
                    Kit::Row {
                        selected: r.selected,
                    },
                    false,
                    |ui, _| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            if let Some(a) = &r.avatar {
                                avatar(ui, t, a, 32.0);
                            } else if let Some(i) = r.icon {
                                ui.add(Icon::new(i).size(20.0));
                            }
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing.y = 0.0;
                                if let Some(o) = &r.overline {
                                    ui.label(text(t, Role::Caption, o));
                                }
                                ui.label(&r.title);
                                if let Some(d) = &r.detail {
                                    ui.label(text(t, Role::Muted, d));
                                }
                            });
                            if let Some(tr) = &r.trailing {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(text(t, Role::Caption, tr));
                                });
                            }
                        });
                    },
                );
            }
        }
        Node::Pick { bind, items } => {
            let at = cx.vals.index(bind);
            for (i, title) in items.iter().enumerate() {
                let r = kit_box(ui, Kit::Row { selected: i == at }, true, |ui, _| {
                    ui.set_min_width(ui.available_width());
                    ui.label(title);
                });
                if r.response.interact(Sense::click()).clicked() {
                    cx.set(&bind.key, Value::Index(i));
                }
            }
        }
        Node::Divider => {
            ui.separator();
        }
        Node::Framed { body, .. } => {
            dodai_ui_egui::Panel::new().show(ui, |ui| view(ui, body, cx));
        }
        Node::Card {
            title,
            body,
            variant,
            ..
        } => {
            kit_box(ui, Kit::Card(*variant), false, |ui, _| {
                if !title.is_empty() {
                    ui.label(text(t, Role::Title, title));
                }
                view(ui, body, cx);
            });
        }
        Node::Alert {
            tone,
            text: msg,
            banner,
            action,
            dismissible,
        } => {
            kit_box(
                ui,
                Kit::Alert {
                    tone: *tone,
                    banner: *banner,
                },
                false,
                |ui, fg| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        if !*banner {
                            ui.add(Icon::new(alert_icon(*tone)).size(18.0).color(fg));
                        }
                        ui.label(msg);
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if *dismissible {
                                ui.add(Icon::new(IconId::Close).size(18.0).color(fg));
                            }
                            if let Some(a) = action {
                                ui.label(RichText::new(a).underline());
                            }
                        });
                    });
                },
            );
        }
        Node::Snackbar { text: msg, action } => {
            kit_box(ui, Kit::Snackbar, false, |ui, _| {
                ui.horizontal(|ui| {
                    ui.label(msg);
                    ui.label(RichText::new(action).color(color32(p.primary)));
                });
            });
        }
        Node::Popup(inner) => {
            kit_box(ui, Kit::Popup, false, |ui, _| {
                ui.set_min_width(200.0);
                view(ui, inner, cx);
            });
        }
        Node::Glyph {
            icon,
            size,
            filled,
            ink,
        } => glyph(ui, t, icon, *size, *filled, color32(*ink)),
        Node::IconTile {
            icon,
            filled,
            pick,
            selected,
        } => {
            let border = if *selected { p.primary } else { p.border };
            let r = egui::Frame::new()
                .stroke(Stroke::new(1.0, color32(border)))
                .inner_margin(egui::Margin::symmetric(4, t.spacing.sm as i8))
                .show(ui, |ui| {
                    ui.set_width(84.0);
                    ui.vertical_centered(|ui| {
                        glyph(ui, t, icon, 28.0, *filled, color32(p.primary));
                        let name = dodai_ui_showcase_model::icons::shortened(
                            icon.name(),
                            dodai_ui_showcase_model::icons::TILE_NAME_CHARS,
                        );
                        ui.label(styled(
                            t,
                            t.typography.size_xs,
                            Font::Body,
                            Ink::Muted,
                            &name,
                        ));
                    });
                });
            if r.response.interact(Sense::click()).clicked() {
                if let Some(k) = pick {
                    cx.acts.push(Action::Press(k.clone()));
                }
            }
        }
        Node::Frosted { w, h, panels } => {
            let rects: Vec<[f32; 4]> = panels.iter().map(|q| q.rect).collect();
            let scene = dodai_ui_showcase_model::media::frosted(t, *w as u32, *h as u32, &rects);
            let (rect, _) = ui.allocate_exact_size(vec2(*w, *h), Sense::hover());
            let tex = |ui: &Ui, pic: &dodai_ui_showcase_model::media::Picture, name: String| {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [pic.width as usize, pic.height as usize],
                    &pic.rgba,
                );
                ui.ctx()
                    .load_texture(name, img, egui::TextureOptions::LINEAR)
            };
            let uv = egui::Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
            let sharp = tex(ui, &scene.sharp, format!("frost-{}-{w}", t.id()));
            ui.painter().image(sharp.id(), rect, uv, Color32::WHITE);
            for (i, (q, pane)) in panels.iter().zip(&scene.panes).enumerate() {
                let [x, y, pw, ph] = q.rect;
                let r = egui::Rect::from_min_size(rect.min + vec2(x, y), vec2(pw, ph));
                let h = tex(ui, pane, format!("frost-{}-{w}-{i}", t.id()));
                ui.painter().image(h.id(), r, uv, Color32::WHITE);
                let mut inner =
                    ui.new_child(egui::UiBuilder::new().max_rect(r.shrink(t.spacing.md)));
                view(&mut inner, &q.body, cx);
            }
        }
        Node::Gizmo { yaw, pitch, size } => {
            let (rect, _) = ui.allocate_exact_size(vec2(*size, *size), Sense::hover());
            let c = rect.center();
            let (y, pt) = (cx.vals.num(yaw), cx.vals.num(pitch));
            for (ang, col) in [(y, p.danger), (y + 2.1, p.success), (y + 4.2, p.info)] {
                let tip = c + vec2(ang.cos(), -pt.sin() * ang.sin()) * *size * 0.4;
                ui.painter()
                    .line_segment([c, tip], Stroke::new(2.0, color32(col)));
                ui.painter().circle_filled(tip, 4.0, color32(col));
            }
        }
        Node::Frame {
            h,
            body,
            scrim,
            layer,
            place,
        } => {
            let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), *h), Sense::hover());
            ui.painter().rect_stroke(
                rect,
                0.0,
                Stroke::new(1.0, color32(p.border)),
                egui::StrokeKind::Inside,
            );
            let mut inner =
                ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink(t.spacing.md)));
            view(&mut inner, body, cx);
            if let Some(c) = scrim {
                ui.painter().rect_filled(rect, 0.0, color32(*c));
            }
            let (align, rect2) = match place {
                Place::Centre => (egui::Align2::CENTER_CENTER, rect),
                Place::Start => (egui::Align2::LEFT_TOP, rect),
                Place::End => (egui::Align2::RIGHT_TOP, rect),
                Place::Bottom => (egui::Align2::CENTER_BOTTOM, rect),
            };
            let layout = Layout::top_down(Align::Min);
            let mut lay = ui.new_child(egui::UiBuilder::new().max_rect(rect2).layout(layout));
            let _ = align;
            lay.add_space(0.0);
            let sz = vec2(rect2.width().min(280.0), 0.0);
            let anchor_x = match place {
                Place::Centre | Place::Bottom => rect2.center().x - sz.x / 2.0,
                Place::Start => rect2.min.x,
                Place::End => rect2.max.x - sz.x,
            };
            let top = if matches!(place, Place::Bottom) {
                rect2.max.y - 80.0
            } else if matches!(place, Place::Centre) {
                rect2.center().y - 40.0
            } else {
                rect2.min.y
            };
            let mut over = ui.new_child(egui::UiBuilder::new().max_rect(
                egui::Rect::from_min_size(pos2(anchor_x, top), vec2(sz.x, rect2.height())),
            ));
            view(&mut over, layer, cx);
        }
        Node::Anchor {
            open,
            trigger,
            panel,
        } => {
            let shown = cx.vals.flag(open);
            ui.vertical(|ui| {
                let before = cx.acts.len();
                view(ui, trigger, cx);
                let pressed = cx.acts.len() > before;
                let _ = pressed;
                if !matches!(**trigger, Node::Button(_)) {
                    let r = ui.interact(
                        ui.min_rect(),
                        ui.make_persistent_id(&open.key),
                        Sense::click(),
                    );
                    if r.clicked() {
                        cx.set(&open.key, Value::Bool(!shown));
                    }
                }
                if shown {
                    view(ui, panel, cx);
                }
            });
        }
        Node::Icon(i, size) => {
            ui.add(Icon::new(*i).size(*size));
        }
        Node::Rating { value, max } => {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for i in 0..*max {
                    let on = (i as f32) < value.round();
                    ui.label(
                        RichText::new("★")
                            .size(t.typography.size_lg)
                            .color(color32(if on { p.warning } else { p.border })),
                    );
                }
            });
        }
        Node::Stepper(b) => {
            let v = cx.vals.int(b);
            ui.horizontal(|ui| {
                if ui
                    .add(
                        Button::new("−")
                            .variant(ButtonVariant::Outline)
                            .enabled(v > STEPPER_RANGE.0),
                    )
                    .clicked()
                {
                    cx.set(&b.key, Value::Int(v - 1));
                }
                ui.label(text(t, Role::Mono, &v.to_string()));
                if ui
                    .add(
                        Button::new("+")
                            .variant(ButtonVariant::Outline)
                            .enabled(v < STEPPER_RANGE.1),
                    )
                    .clicked()
                {
                    cx.set(&b.key, Value::Int(v + 1));
                }
            });
        }
        Node::Steps { steps, current } => {
            ui.vertical(|ui| {
                for (i, s) in steps.iter().enumerate() {
                    let state = if i < *current {
                        dodai_ui_core::StepState::Done
                    } else if i == *current {
                        dodai_ui_core::StepState::Current
                    } else {
                        dodai_ui_core::StepState::Todo
                    };
                    ui.horizontal(|ui| {
                        kit_box(ui, Kit::StepDot(state), false, |ui, _| {
                            ui.label(if i < *current {
                                "✓".to_string()
                            } else {
                                (i + 1).to_string()
                            });
                        });
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            ui.label(&s.title);
                            if let Some(d) = &s.detail {
                                ui.label(text(t, Role::Muted, d));
                            }
                        });
                    });
                }
            });
        }
        Node::Breadcrumb(items) => {
            ui.horizontal(|ui| {
                for (i, it) in items.iter().enumerate() {
                    if i > 0 {
                        ui.add(Icon::new(IconId::ChevronRight).size(14.0));
                    }
                    let last = i + 1 == items.len();
                    ui.label(styled(
                        t,
                        t.typography.size_sm,
                        Font::Body,
                        if last { Ink::Text } else { Ink::Link },
                        it,
                    ));
                }
            });
        }
        Node::Pagination { pages, bind } => {
            let at = cx.vals.index(bind);
            ui.horizontal(|ui| {
                for i in 0..*pages {
                    let v = if i == at {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Ghost
                    };
                    if ui
                        .add(Button::new((i + 1).to_string()).variant(v).min_width(36.0))
                        .clicked()
                    {
                        cx.set(&bind.key, Value::Index(i));
                    }
                }
            });
        }
        Node::Tree(nodes) => {
            for n in nodes {
                tree(ui, n, 0);
            }
        }
        Node::Table(tb) => table(ui, tb, t),
        Node::Avatar(s) => avatar(ui, t, s, 40.0),
        Node::Tooltip {
            text: msg,
            rich,
            title,
            action,
        } => {
            kit_box(ui, Kit::Tooltip { rich: *rich }, false, |ui, _| {
                if let Some(h) = title {
                    ui.label(RichText::new(h).size(t.typography.size_md));
                }
                ui.label(RichText::new(msg).size(t.typography.size_sm));
                if let Some(a) = action {
                    ui.label(RichText::new(a).color(color32(p.primary)));
                }
            });
        }
        Node::Code { text: code, lang } => code_block(ui, code, lang, t),
        Node::Rich { spans, role } => {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for s in spans {
                    let mut r = text(t, *role, &s.text);
                    if s.bold {
                        r = r.strong();
                    }
                    if s.italic {
                        r = r.italics();
                    }
                    if s.code {
                        r = r
                            .monospace()
                            .color(color32(p.primary))
                            .background_color(color32(p.surface_raised));
                    }
                    if s.link {
                        r = r.color(color32(p.link)).underline();
                    }
                    ui.label(r);
                }
            });
        }
        Node::Quote(body) => {
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(vec2(2.0, 24.0), Sense::hover());
                ui.painter().rect_filled(r, 0.0, color32(p.primary));
                ui.vertical(|ui| view(ui, body, cx));
            });
        }
        Node::AvatarGroup { names, max, size } => {
            let shown = max.unwrap_or(names.len()).min(names.len());
            let extra = names.len() - shown;
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = -size * 0.25;
                for n in &names[..shown] {
                    avatar(ui, t, n, *size);
                }
                if extra > 0 {
                    avatar(ui, t, &format!("+{extra}"), *size);
                }
            });
        }
        Node::Overlay(items) => {
            let start = ui.cursor().min;
            let mut size = vec2(0.0, 0.0);
            for n in items {
                let r = ui.scope_builder(
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        start,
                        vec2(ui.available_width(), 0.0),
                    )),
                    |ui| view(ui, n, cx),
                );
                size = size.max(r.response.rect.size());
            }
            ui.allocate_space(size);
        }
        Node::Timeline(events) => {
            for e in events {
                ui.horizontal(|ui| {
                    kit_box(ui, Kit::Dot(e.tone), false, |ui, _| {
                        ui.add_space(4.0);
                    });
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(&e.title);
                        ui.label(text(t, Role::Caption, &e.time));
                        if let Some(d) = &e.detail {
                            ui.label(text(t, Role::Muted, d));
                        }
                    });
                });
            }
        }
        Node::Badged { icon, count, dot } => {
            ui.horizontal(|ui| {
                ui.add(Icon::new(*icon).size(24.0));
                if let Some((n, tone)) = count {
                    kit_box(ui, Kit::Count(*tone), false, |ui, _| {
                        ui.label(RichText::new(n.to_string()).size(t.typography.size_xs));
                    });
                } else if let Some(tone) = dot {
                    kit_box(ui, Kit::Dot(Some(*tone)), false, |ui, _| ui.add_space(4.0));
                }
            });
        }
        Node::Spinner(size) => {
            ui.add(Spinner::new().size(*size));
        }
        Node::SpinnerOf { size, style } => {
            ui.add(Spinner::new().size(*size).style(*style));
        }
        Node::Indeterminate => {
            let w = ui.available_width().min(520.0);
            let (r, _) = ui.allocate_exact_size(vec2(w, 6.0), Sense::hover());
            ui.painter().rect_filled(r, 3.0, color32(p.border));
            let f = (cx.time * 0.8).fract() as f32;
            let seg = w * 0.3;
            let x = r.min.x + (w + seg) * f - seg;
            let bar = egui::Rect::from_min_max(
                pos2(x.max(r.min.x), r.min.y),
                pos2((x + seg).min(r.max.x), r.max.y),
            );
            if bar.width() > 0.0 {
                ui.painter().rect_filled(bar, 3.0, color32(p.primary));
            }
        }
        Node::WavyProgress { value } => progress_wave(ui, *value, cx),
        Node::CircularProgress { value, .. } => {
            let size = 40.0;
            let (r, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
            let c = r.center();
            let rad = size / 2.0 - 3.0;
            ui.painter()
                .circle_stroke(c, rad, Stroke::new(4.0, color32(p.border)));
            let f = value.unwrap_or(0.25);
            let start = if value.is_some() {
                -std::f32::consts::FRAC_PI_2
            } else {
                (cx.time * 4.0) as f32
            };
            let pts: Vec<_> = (0..=32)
                .map(|i| {
                    let a = start + std::f32::consts::TAU * f * i as f32 / 32.0;
                    c + vec2(a.cos(), a.sin()) * rad
                })
                .collect();
            ui.painter()
                .add(egui::Shape::line(pts, Stroke::new(4.0, color32(p.primary))));
        }
        Node::StyledProgress { value, .. } => progress_wave(ui, *value, cx),
        Node::Skeleton { w, h } => {
            let (r, _) = ui.allocate_exact_size(vec2(*w, *h), Sense::hover());
            let pulse = 0.55 + 0.25 * ((cx.time * 3.0).sin() as f32);
            ui.painter().rect_filled(
                r,
                t.shape.radius.min(8.0),
                color32(t.kit(Kit::Skeleton, false).fill.unwrap_or(p.border))
                    .gamma_multiply(pulse + 0.4),
            );
        }
        Node::Empty {
            icon,
            title,
            detail,
        } => {
            ui.vertical_centered(|ui| {
                ui.label(styled(
                    t,
                    t.typography.size_xxl,
                    Font::Body,
                    Ink::Muted,
                    icon,
                ));
                ui.label(styled(
                    t,
                    t.typography.size_lg,
                    Font::Body,
                    Ink::Text,
                    title,
                ));
                ui.label(styled(
                    t,
                    t.typography.size_sm,
                    Font::Body,
                    Ink::Muted,
                    detail,
                ));
            });
        }
    }
}

fn progress_wave(ui: &mut Ui, value: Option<f32>, cx: &Cx) {
    let p = &cx.theme.palette;
    let w = ui.available_width().min(520.0);
    let (r, _) = ui.allocate_exact_size(vec2(w, 12.0), Sense::hover());
    let f = value.unwrap_or(((cx.time * 0.5).fract()) as f32);
    let mid = r.center().y;
    let wave = |x: f32| mid + ((x * 0.35) + cx.time as f32 * 4.0).sin() * 3.0;
    let end = r.min.x + w * f;
    let pts = |a: f32, b: f32| {
        (0..=60)
            .map(|i| a + (b - a) * i as f32 / 60.0)
            .map(|x| pos2(x, wave(x)))
            .collect::<Vec<_>>()
    };
    ui.painter().add(egui::Shape::line(
        pts(end, r.max.x),
        Stroke::new(3.0, color32(p.border)),
    ));
    ui.painter().add(egui::Shape::line(
        pts(r.min.x, end),
        Stroke::new(3.0, color32(p.primary)),
    ));
}

fn avatar(ui: &mut Ui, t: &Theme, initials: &str, size: f32) {
    let s = t.kit(Kit::Avatar, false);
    let (r, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    ui.painter().circle_filled(
        r.center(),
        size / 2.0,
        color32(s.fill.unwrap_or(t.palette.primary)),
    );
    ui.painter().text(
        r.center(),
        egui::Align2::CENTER_CENTER,
        initials,
        egui::FontId::proportional(size * 0.38),
        color32(s.fg),
    );
}

fn tree(ui: &mut Ui, n: &dodai_ui_showcase_model::node::TreeNode, depth: usize) {
    let leaf = n.children.is_empty();
    let r = kit_box(
        ui,
        Kit::Row {
            selected: n.selected,
        },
        false,
        |ui, _| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.add_space(depth as f32 * 16.0);
                let mark = if leaf {
                    IconId::Dot
                } else if n.open {
                    IconId::ChevronDown
                } else {
                    IconId::ChevronRight
                };
                ui.add(Icon::new(mark).size(14.0));
                if let Some(i) = n.icon {
                    ui.add(Icon::new(i).size(16.0));
                }
                ui.label(&n.label);
            });
        },
    );
    let _ = r;
    if n.open {
        for c in &n.children {
            tree(ui, c, depth + 1);
        }
    }
}

fn table(ui: &mut Ui, tb: &dodai_ui_showcase_model::node::Table, t: &Theme) {
    let total = ui.available_width();
    let fixed: f32 = tb.columns.iter().filter_map(|c| c.width).sum();
    let flex = tb
        .columns
        .iter()
        .filter(|c| c.width.is_none())
        .count()
        .max(1) as f32;
    let widths: Vec<f32> = tb
        .columns
        .iter()
        .map(|c| c.width.unwrap_or(((total - fixed) / flex).max(60.0)))
        .collect();
    let line = |ui: &mut Ui, cells: Vec<String>, kit: Kit, head: bool| {
        kit_box(ui, kit, false, |ui, _| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                for (i, c) in cells.iter().enumerate() {
                    let w = widths.get(i).copied().unwrap_or(80.0);
                    let al = tb.columns.get(i).map_or(Al::Left, |c| c.align);
                    let lay = match al {
                        Al::Left => Layout::left_to_right(Align::Center),
                        Al::Center => Layout::centered_and_justified(egui::Direction::LeftToRight),
                        Al::Right => Layout::right_to_left(Align::Center),
                    };
                    ui.allocate_ui_with_layout(vec2(w, 20.0), lay, |ui| {
                        ui.set_width(w);
                        ui.label(if head {
                            text(t, Role::Label, c)
                        } else {
                            RichText::new(c)
                        });
                    });
                }
            });
        });
    };
    line(
        ui,
        tb.columns.iter().map(|c| c.title.clone()).collect(),
        Kit::TableHead,
        true,
    );
    let body = |ui: &mut Ui| {
        for (i, r) in tb.rows.iter().enumerate() {
            line(ui, r.clone(), Kit::TableRow(i % 2 == 1), false);
        }
    };
    match tb.height {
        Some(h) => {
            egui::ScrollArea::vertical()
                .max_height(h)
                .id_salt("table")
                .show(ui, body);
        }
        None => body(ui),
    }
}

fn code_block(ui: &mut Ui, code: &str, lang: &str, t: &Theme) {
    use dodai_ui_core::syntax::{highlight_lines, CodeToken as K};
    let p = &t.palette;
    let colour = |k: K| match k {
        K::Keyword => p.primary,
        K::Str => p.success,
        K::Comment => p.text_muted,
        K::Number => p.warning,
        K::Type => p.info,
    };
    kit_box(ui, Kit::Code, false, |ui, _| {
        ui.set_min_width(ui.available_width());
        if !lang.is_empty() {
            ui.label(text(t, Role::Caption, &lang.to_uppercase()));
        }
        ui.spacing_mut().item_spacing.y = 0.0;
        for (l, spans) in code.split('\n').zip(highlight_lines(code, lang)) {
            let mut job = egui::text::LayoutJob::default();
            let font = egui::FontId::monospace(t.typography.size_sm);
            let mut at = 0;
            let mut push = |s: &str, c: Color32| {
                job.append(s, 0.0, egui::TextFormat::simple(font.clone(), c));
            };
            for (r, k) in spans {
                if r.start > at {
                    push(&l[at..r.start], color32(p.text));
                }
                push(&l[r.clone()], color32(colour(k)));
                at = r.end;
            }
            push(&l[at..], color32(p.text));
            ui.label(job);
        }
    });
}

/// Fill a simple polygon (not nec. convex) by ear clipping.
fn fill_polygon(ui: &Ui, pts: &[egui::Pos2], color: Color32) {
    let n = pts.len();
    if n < 3 {
        return;
    }
    let area: f32 = (0..n)
        .map(|i| {
            let (a, b) = (pts[i], pts[(i + 1) % n]);
            a.x * b.y - b.x * a.y
        })
        .sum();
    let mut idx: Vec<usize> = if area > 0.0 {
        (0..n).collect()
    } else {
        (0..n).rev().collect()
    };
    let cross = |a: egui::Pos2, b: egui::Pos2, c: egui::Pos2| {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    };
    let inside = |p: egui::Pos2, a: egui::Pos2, b: egui::Pos2, c: egui::Pos2| {
        cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
    };
    let mut mesh = egui::Mesh::default();
    for p in pts {
        mesh.colored_vertex(*p, color);
    }
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut clipped = false;
        for k in 0..m {
            let (i0, i1, i2) = (idx[(k + m - 1) % m], idx[k], idx[(k + 1) % m]);
            let (a, b, c) = (pts[i0], pts[i1], pts[i2]);
            if cross(a, b, c) <= 0.0 {
                continue;
            }
            if idx
                .iter()
                .any(|&j| j != i0 && j != i1 && j != i2 && inside(pts[j], a, b, c))
            {
                continue;
            }
            mesh.add_triangle(i0 as u32, i1 as u32, i2 as u32);
            idx.remove(k);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        mesh.add_triangle(idx[0] as u32, idx[1] as u32, idx[2] as u32);
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}

/// The natural width of `node` (a throwaway layout pass; its events are dropped).
fn measure(ui: &mut Ui, node: &Node, cx: &Cx) -> f32 {
    let mut scratch = Cx {
        theme: cx.theme.clone(),
        vals: cx.vals.clone(),
        acts: Vec::new(),
        time: cx.time,
    };
    let rect = egui::Rect::from_min_size(ui.cursor().min, vec2(100_000.0, 100_000.0));
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .sizing_pass()
            .invisible(),
    );
    child.horizontal_top(|ui| view(ui, node, &mut scratch));
    child.min_rect().width()
}

/// A row that breaks into lines: items keep their natural width and every line is top
/// aligned (egui's own wrapping lays tall items out staggered).
fn wrapped(ui: &mut Ui, items: &[Node], gap: f32, centre: bool, cx: &mut Cx) {
    let avail = ui.available_width();
    let widths: Vec<f32> = items.iter().map(|n| measure(ui, n, cx)).collect();
    let mut lines: Vec<Vec<usize>> = vec![Vec::new()];
    let mut used = 0.0;
    for (i, w) in widths.iter().enumerate() {
        let need = if lines.last().is_some_and(|l| l.is_empty()) {
            *w
        } else {
            used + gap + w
        };
        if need > avail && !lines.last().is_some_and(|l| l.is_empty()) {
            lines.push(Vec::new());
            used = *w;
        } else {
            used = need;
        }
        lines.last_mut().expect("a line").push(i);
    }
    ui.spacing_mut().item_spacing.y = gap;
    for line in lines {
        let row = |ui: &mut Ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for i in line {
                view(ui, &items[i], cx);
            }
        };
        if centre {
            ui.horizontal(row);
        } else {
            ui.horizontal_top(row);
        }
    }
}
