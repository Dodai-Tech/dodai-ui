//! Walks the model's [`Node`] tree and builds dodai-dioxus components.

use dioxus::prelude::*;
use dodai_core::{ControlSize, ToggleKind};
use dodai_dioxus::*;
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Font, Ink, TextRole as Role, TextStyle};
use dodai_showcase_model::{Action, Node as Demo, Showcase, Value};

/// Counter range of the number stepper demo.
pub const STEPPER_RANGE: (i32, i32) = (0, 20);

/// An SVG `points` attribute.
fn svg_points(points: &[(f32, f32)]) -> String {
    points
        .iter()
        .map(|(x, y)| format!("{x:.1},{y:.1}"))
        .collect::<Vec<_>>()
        .join(" ")
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

/// The inline style of a text role: size, family, colour and casing resolved by the model, so
/// text matches the gpui showcase.
pub fn text_style(r: Role, theme: &dodai_core::Theme) -> String {
    let st = r.resolve(theme);
    let family = match st.font {
        Font::Display => "var(--dodai-font-display)",
        Font::Body => "var(--dodai-font-body)",
        Font::Mono => "var(--dodai-font-mono)",
    };
    let caps = if st.caps && theme.typography.uppercase_labels {
        "text-transform:uppercase;"
    } else {
        ""
    };
    let lh = match r {
        Role::Type(t) => {
            let s = theme.type_style(t);
            format!("line-height:{}px;font-weight:{};", s.line_height, s.weight)
        }
        _ => String::new(),
    };
    format!(
        "font-size:{}px;font-family:{family};color:{};{caps}{lh}",
        st.size,
        st.ink.color(theme).to_css()
    )
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

/// Everything the builder reads.
#[derive(Clone, Copy)]
pub struct Cx {
    pub model: Signal<Showcase>,
}

fn act(mut model: Signal<Showcase>, a: Action) {
    let effect = {
        let mut m = model.write();
        m.update(a);
        m.take_effect()
    };
    if let Some(dodai_showcase_model::icon_actions::Effect::Copy(text)) = effect {
        // The browser owns the clipboard.
        let write = document::eval("navigator.clipboard.writeText(await dioxus.recv())");
        let _ = write.send(text);
    }
}

/// Build `node`.
pub fn view(node: &Demo, cx: Cx) -> Element {
    let model = cx.model;
    let theme = model.read().theme();
    let vals = model.read().values.clone();
    match node {
        Demo::Stack(s) => {
            let axis = if s.axis == Axis::Row {
                "dodai-stack--row"
            } else {
                "dodai-stack--col"
            };
            let wrap = if s.wrap { "dodai-stack--wrap" } else { "" };
            let gap = s.gap.px(&theme);
            let align = if let Some(w) = s.width {
                format!(
                    "width:{w}px;flex:none;align-items:{}",
                    if s.centre && s.axis == Axis::Row {
                        "center"
                    } else {
                        "flex-start"
                    }
                )
            } else if s.axis == Axis::Row && !s.centre {
                "align-items:flex-start".to_string()
            } else if s.axis == Axis::Column && s.centre {
                "align-items:center;width:auto".to_string()
            } else {
                String::new()
            };
            rsx! {
                div { class: "dodai-stack {axis} {wrap}", style: "gap:{gap}px;{align}",
                    for (i, n) in s.items.iter().enumerate() { {rsx! { Fragment { key: "{i}", {view(n, cx)} } }} }
                }
            }
        }
        Demo::Text(t, r) => {
            rsx! { span { style: text_style(*r, &theme), "{t}" } }
        }
        Demo::Space(px) => rsx! { div { style: "width:{px}px;height:{px}px" } },
        Demo::Grow => rsx! { div { style: "flex:1 1 0" } },
        Demo::Swatch {
            name,
            color,
            detail,
        } => {
            let xs = theme.typography.size_xs;
            rsx! {
                div { style: "display:flex;flex-direction:column;gap:4px;width:118px;flex:none",
                    div { style: "height:44px;background:{color.to_css()};border:1px solid {theme.palette.border.to_css()}" }
                    span { style: text_style(styled(xs, Font::Body, Ink::Text), &theme), "{name}" }
                    span { style: text_style(styled(xs, Font::Mono, Ink::Muted), &theme), "{detail}" }
                }
            }
        }
        Demo::Sample { w, h, label } => {
            let sm = theme.typography.size_sm;
            rsx! {
                div { class: "dodai-panel", style: "width:{w}px;height:{h}px;padding:0;display:grid;place-items:center;flex:none",
                    span { class: "dodai-panel__shade", aria_hidden: "true" }
                    span { style: text_style(styled(sm, Font::Body, Ink::Text), &theme), "{label}" }
                }
            }
        }
        Demo::Logo(size) => {
            let url = dodai_showcase_model::media::logo_data_url();
            rsx! { img { src: url, alt: "Dodai UI", style: "width:{size}px;height:{size}px;flex:none" } }
        }
        Demo::Image {
            src: Some(src),
            video,
            contain,
            w,
            h,
            caption,
            ..
        } => {
            let url = dodai_showcase_model::media::data_url(src).unwrap_or_default();
            let fit = if *contain { "contain" } else { "cover" };
            let d = (h * 0.28).clamp(28.0, 56.0);
            let frame = format!(
                "position:relative;overflow:hidden;width:{w}px;height:{h}px;background:{};border:{}px solid {};border-radius:{}px;display:grid;place-items:center",
                theme.palette.surface_raised.to_css(),
                theme.shape.border_width,
                theme.palette.border.to_css(),
                theme.shape.radius,
            );
            let badge = format!(
                "position:absolute;width:{d}px;height:{d}px;border-radius:50%;display:grid;place-items:center;background:{};border:1px solid {};color:{};font-size:{}px",
                theme.palette.background.with_alpha(0.75).to_css(),
                theme.palette.primary.to_css(),
                theme.palette.primary.to_css(),
                d * 0.45
            );
            rsx! {
                div { style: "display:flex;flex-direction:column;gap:{theme.spacing.xs}px;width:{w}px;flex:none",
                    div { style: frame,
                        img { src: url, style: "width:100%;height:100%;object-fit:{fit}" }
                        if *video { div { style: badge, "▶" } }
                    }
                    if let Some(c) = caption { span { style: text_style(Role::Muted, &theme), "{c}" } }
                }
            }
        }
        Demo::Image {
            w,
            h,
            caption,
            on_image,
            ..
        } => {
            let (sm, md) = (theme.typography.size_sm, theme.typography.size_md);
            let shadow = theme.palette.roles.shadow;
            let ink = theme.on(shadow.with_alpha(1.0)).to_css();
            let fade_h = (h * 0.5).max(theme.sizes().touch_target);
            // The design's fade as a CSS gradient, strongest at the bottom edge.
            let stops = theme
                .fade_stops()
                .iter()
                .rev()
                .map(|s| {
                    format!(
                        "{} {:.1}%",
                        shadow.with_alpha(s.alpha).to_css(),
                        100.0 - s.at * 100.0
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            let frame = format!(
                "position:relative;overflow:hidden;width:{w}px;height:{h}px;background:{};border:{}px solid {};border-radius:{}px;display:grid;place-items:center",
                theme.palette.surface_raised.to_css(),
                theme.shape.border_width,
                theme.palette.border.to_css(),
                theme.shape.radius,
            );
            let over = *on_image && caption.is_some();
            rsx! {
                div { style: "display:flex;flex-direction:column;gap:{theme.spacing.xs}px;width:{w}px;flex:none",
                    div { style: frame,
                        span { style: text_style(styled(md, Font::Body, Ink::Muted), &theme), "image unavailable" }
                        if over {
                            div { style: "position:absolute;left:0;right:0;bottom:0;height:{fade_h}px;background:linear-gradient(to bottom,{stops})" }
                            span { style: "position:absolute;left:{theme.spacing.md}px;right:{theme.spacing.md}px;bottom:{theme.spacing.sm}px;font-size:{sm}px;color:{ink};font-family:var(--dodai-font-body)",
                                "{caption.clone().unwrap_or_default()}"
                            }
                        }
                    }
                    if !over {
                        if let Some(c) = caption { span { style: text_style(Role::Muted, &theme), "{c}" } }
                    }
                }
            }
        }
        Demo::Block {
            w,
            h,
            fill,
            border,
            radius,
            label,
        } => {
            let size = format!(
                "{}{}",
                if *w > 0.0 {
                    format!("width:{w}px;")
                } else {
                    String::new()
                },
                if *h > 0.0 {
                    format!("height:{h}px;")
                } else {
                    String::new()
                }
            );
            let bd = border.map_or("none".to_string(), |b| format!("1px solid {}", b.to_css()));
            rsx! {
                div { style: "{size}background:{fill.to_css()};border:{bd};border-radius:{radius}px;display:grid;place-items:center;flex:none",
                    if let Some(l) = label { span { style: text_style(styled(theme.typography.size_xs, Font::Body, Ink::Text), &theme), "{l}" } }
                }
            }
        }
        Demo::Button(b) if b.elevated => {
            let key = b.counter.clone();
            let size = theme.control_metrics(ControlSize::Medium).text_size;
            rsx! {
                KitBox { kit: dodai_core::Kit::Elevated, style: format!("flex:none;font-size:{size}px"),
                    onclick: move |_| if let Some(k) = key.clone() { act(model, Action::Press(k)) },
                    "{b.label}"
                }
            }
        }
        Demo::Button(b) => {
            let (key, variant) = (b.counter.clone(), b.variant);
            // The Shape page's crop (cut-corner designs crop the left end by default).
            let crop = (!b.plain)
                .then(|| {
                    b.crop.or(dodai_showcase_model::tune_state::default_crop(
                        &vals, &theme,
                    ))
                })
                .flatten();
            rsx! {
                Button { variant, size: Some(b.size), square: b.square, icon_only: b.label.is_empty(), crop, surface: b.surface,
                    disabled: b.disabled, loading: b.loading, shimmer: b.shimmer,
                    onclick: move |_| if let Some(k) = key.clone() { act(model, Action::Press(k)) },
                    if let Some(i) = b.icon { Icon { icon: i, size: 16, theme: theme.clone() } }
                    if !b.label.is_empty() { "{b.label}" }
                }
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
            rsx! {
                span { class: "dodai-split",
                    Button { variant: *variant, size: Some(ControlSize::Medium), crop: *crop, "{label}" }
                    Button { variant: *variant, size: Some(ControlSize::Medium), icon_only: true,
                        Icon { icon: arrow, size: 16, theme: theme.clone() } }
                }
            }
        }
        Demo::Fab { label, .. } => {
            let kit = dodai_core::Kit::Fab {
                extended: label.is_some(),
            };
            rsx! {
                KitBox { kit, onclick: move |_| {}, style: "flex:none;font-size:var(--dodai-size-md)".to_string(),
                    Icon { icon: IconId::Plus, size: 24, theme: theme.clone() }
                    if let Some(l) = label { span { "{l}" } }
                }
            }
        }
        Demo::Toolbar {
            floating,
            vibrant,
            icons,
        } => {
            let kit = dodai_core::Kit::Toolbar {
                floating: *floating,
                vibrant: *vibrant,
            };
            let width = if *floating { "fit-content" } else { "100%" };
            rsx! {
                KitBox { kit, style: format!("width:{width};gap:var(--dodai-space-xs);flex:none"),
                    for (i, ic) in icons.iter().enumerate() {
                        Button { key: "{i}", variant: dodai_core::ButtonVariant::Ghost, icon_only: true,
                            Icon { icon: *ic, size: 18, theme: theme.clone() } }
                    }
                }
            }
        }
        Demo::Choice(ch) if ch.connected => {
            let key = ch.bind.key.clone();
            rsx! {
                Segmented { labels: ch.labels.clone(), selected: vals.index(&ch.bind),
                    onselect: move |i| act(model, Action::Set(key.clone(), Value::Index(i))) }
            }
        }
        Demo::Choice(ch) => {
            let at = vals.index(&ch.bind);
            let key = ch.bind.key.clone();
            rsx! {
                div { class: "dodai-seg", style: if ch.connected { "gap:0" } else { "" },
                    for (i, l) in ch.labels.iter().enumerate() {
                        Button { key: "{i}", size: Some(ControlSize::Medium),
                            variant: if i == at { dodai_core::ButtonVariant::Primary } else { dodai_core::ButtonVariant::Outline },
                            onclick: { let key = key.clone(); move |_| act(model, Action::Set(key.clone(), Value::Index(i))) },
                            "{l}" }
                    }
                }
            }
        }
        Demo::Tabs(ch) => {
            let key = ch.bind.key.clone();
            rsx! {
                Tabs { tabs: ch.labels.clone(), active: vals.index(&ch.bind),
                    secondary: ch.secondary, disabled: ch.disabled.clone(), icons: ch.icons.clone(),
                    onselect: move |i| act(model, Action::Set(key.clone(), Value::Index(i))) }
            }
        }
        Demo::Tag { label, tone, solid } => {
            rsx! { Tag { kind: TagKind::from(*tone), soft: !*solid, "{label}" } }
        }
        Demo::Chip(ch) => {
            let on = vals.flag(&ch.bind);
            let key = ch.bind.key.clone();
            let kit = if ch.elevated {
                dodai_core::Kit::Elevated
            } else {
                dodai_core::Kit::Chip { selected: on }
            };
            let lead = if ch.filter && on {
                Some(IconId::Check)
            } else {
                ch.icon
            };
            let sm = theme.typography.size_sm;
            rsx! {
                KitBox { kit, style: format!("flex:none;font-size:{sm}px;gap:6px"),
                    onclick: move |_| act(model, Action::Set(key.clone(), Value::Bool(!on))),
                    if let Some(i) = lead { Icon { icon: i, size: 16, theme: theme.clone() } }
                    span { "{ch.label}" }
                    if ch.removable { Icon { icon: IconId::Close, size: 14, theme: theme.clone() } }
                }
            }
        }
        Demo::Toggle(t) => {
            let key = t.bind.key.clone();
            match t.kind {
                ToggleKind::Switch => rsx! {
                    Toggle { checked: vals.flag(&t.bind), disabled: t.disabled, label: t.label.clone(),
                        onchange: move |b| act(model, Action::Set(key.clone(), Value::Bool(b))) }
                },
                ToggleKind::Checkbox => rsx! {
                    Checkbox { checked: vals.flag(&t.bind), disabled: t.disabled, label: t.label.clone(), mixed: t.mixed,
                        onchange: move |b| act(model, Action::Set(key.clone(), Value::Bool(b))) }
                },
                ToggleKind::Radio => {
                    let (group, idx) = t.radio.clone().unwrap_or((key, 0));
                    let picked = vals.index_of(&group, usize::MAX);
                    let on = if picked == usize::MAX {
                        t.bind.default
                    } else {
                        picked == idx
                    };
                    rsx! {
                        Radio { name: group.clone(), checked: on, disabled: t.disabled, label: t.label.clone(),
                            onselect: move |_| act(model, Action::Set(group.clone(), Value::Index(idx))) }
                    }
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
            let v = vals.num(bind);
            let (key, min, max) = (bind.key.clone(), *min, *max);
            rsx! {
                Knob { value: v, min, max, size: *size, disabled: *disabled, theme: theme.clone(),
                    onchange: move |x| act(model, Action::Set(key.clone(), Value::Num(x))) }
            }
        }
        Demo::Select {
            key,
            labels,
            selected,
            placeholder,
        } => {
            let key = key.clone();
            rsx! {
                select { class: "dodai-select", style: "min-width:320px",
                    onchange: move |e| if let Ok(i) = e.value().parse::<usize>() {
                        act(model, Action::Set(key.clone(), Value::Index(i)))
                    },
                    if selected.is_none() {
                        option { value: "", selected: true, disabled: true, "{placeholder}" }
                    }
                    for (i, it) in labels.iter().enumerate() {
                        option { key: "{i}", value: "{i}", selected: *selected == Some(i), "{it}" }
                    }
                }
            }
        }
        Demo::Shader(spec) => {
            let id = format!("{spec:?}{}", theme.name());
            rsx! {
                Fragment { key: "{id}",
                    dodai_shaders::dioxus::ShaderView { spec: spec.to_spec(&theme), width: spec.width }
                }
            }
        }
        Demo::Plot(p) => rsx! {
            svg { width: "{p.w}", height: "{p.h}", view_box: "0 0 {p.w} {p.h}", style: "flex:none",
                for l in p.lines.iter() {
                    polyline {
                        fill: "none",
                        stroke: "{l.color.to_css()}",
                        stroke_width: "{l.width}",
                        stroke_linejoin: "round",
                        points: "{svg_points(&l.points)}",
                    }
                }
            }
        },
        Demo::Scroll { height, body } => rsx! {
            div { class: "dodai-scroll", style: "height:{height}px;overflow:auto;width:100%;max-width:520px", {view(body, cx)} }
        },
        Demo::Slider(s) if s.vertical => {
            let v = vals.num(&s.bind);
            let (key, min, max, step) = (s.bind.key.clone(), s.min, s.max, s.step);
            let length = s.length.unwrap_or(120.0) as u32;
            rsx! {
                Slider { value: v, min, max, step: step.unwrap_or((max - min) / 100.0), vertical: true, length,
                    disabled: s.disabled,
                    onchange: move |x| act(model, Action::Set(key.clone(), Value::Num(snap(x, min, max, step)))) }
            }
        }
        Demo::Range(r) => {
            let (lo_key, hi_key, min, max, step) =
                (r.lo.key.clone(), r.hi.key.clone(), r.min, r.max, r.step);
            rsx! {
                RangeSlider { lo: vals.num(&r.lo), hi: vals.num(&r.hi), min, max,
                    step: step.unwrap_or((max - min) / 100.0),
                    vertical: r.vertical.is_some(), length: r.vertical.unwrap_or(120.0) as u32,
                    onchange: move |(a, b)| {
                        act(model, Action::Set(lo_key.clone(), Value::Num(snap(a, min, max, step))));
                        act(model, Action::Set(hi_key.clone(), Value::Num(snap(b, min, max, step))));
                    } }
            }
        }
        Demo::Slider(s) => {
            let v = vals.num(&s.bind);
            let (key, min, max, step) = (s.bind.key.clone(), s.min, s.max, s.step);
            let shown = match s.format {
                dodai_showcase_model::node::SliderFormat::Plain => num_text(v, step),
                f => f.text(v),
            };
            let bubble = s.bubble.map(|_| shown.clone());
            rsx! {
                div { class: "dodai-stack dodai-stack--row", style: "width:100%",
                    if let Some(ic) = s.icon { Icon { icon: ic, size: 20, theme: theme.clone() } }
                    div { style: "flex:1", Slider { value: v, min, max, step: step.unwrap_or((max - min) / 100.0),
                        disabled: s.disabled, ticks: s.ticks, centered: s.centered,
                        bubble, bubble_below: s.bubble == Some(false),
                        onchange: move |x| act(model, Action::Set(key.clone(), Value::Num(snap(x, min, max, step)))) } }
                    if s.readout { code { class: "dodai-mono", "{shown}" } }
                }
            }
        }
        Demo::Progress { value, tone } => rsx! {
            div { style: "width:100%", Progress { value: *value, tone: *tone } }
        },
        Demo::Input(i) => {
            let key = i.bind.key.clone();
            rsx! {
                div { class: "dodai-field", style: "width:100%",
                    if let Some(l) = &i.label { span { class: "dodai-label", "{l}" } }
                    div { class: "dodai-stack dodai-stack--row",
                        if let Some(ic) = i.icon { Icon { icon: ic, size: 18, theme: theme.clone() } }
                        div { style: "flex:1",
                            TextInput { value: vals.text(&i.bind).to_string(), placeholder: i.placeholder.clone(),
                                password: i.password, lines: i.lines,
                                oninput: move |s| act(model, Action::Set(key.clone(), Value::Text(s))) }
                        }
                    }
                }
            }
        }
        Demo::List(rows) => rsx! {
            div { class: "dodai-list dodai-list--plain",
                for (n, r) in rows.iter().enumerate() {
                    Fragment { key: "{n}",
                    if n > 0 { hr { class: "dodai-divider", style: "margin:0" } }
                    KitBox { kit: dodai_core::Kit::Row { selected: r.selected }, style: "gap:var(--dodai-space-md)".to_string(),
                        if let Some(a) = &r.avatar { Avatar { initials: a.clone() } }
                        else if let Some(i) = r.icon { Icon { icon: i, size: 20, theme: theme.clone() } }
                        div { class: "dodai-grow dodai-col-text",
                            if let Some(o) = &r.overline { span { class: "dodai-caption", "{o}" } }
                            span { "{r.title}" }
                            if let Some(d) = &r.detail { span { class: "dodai-muted dodai-sm", "{d}" } }
                        }
                        if let Some(t) = &r.trailing { span { class: "dodai-caption", "{t}" } }
                    }
                    }
                }
            }
        },
        Demo::Divider => rsx! { Divider {} },
        Demo::Framed { border, body } => {
            let t = &theme;
            let frame = t.framed(t.panel(false).border, *border);
            let vars: String = dodai_topcoat::panel_frame_vars(t, &frame)
                .iter()
                .map(|(k, v)| format!("--dodai-{k}:{v};"))
                .collect();
            rsx! { div { style: "{vars}", Panel { {view(body, cx)} } } }
        }
        Demo::Card {
            title,
            body,
            variant,
            surface,
        } => rsx! {
            Card { variant: *variant, title: title.clone(), surface: *surface, {view(body, cx)} }
        },
        Demo::Alert {
            tone,
            text,
            banner,
            action,
            dismissible,
        } => {
            let kit = dodai_core::Kit::Alert {
                tone: *tone,
                banner: *banner,
            };
            rsx! {
                KitBox { kit, style: "width:100%;".to_string(),
                    if !*banner { Icon { icon: alert_icon(*tone), size: 18, theme: theme.clone() } }
                    span { style: "flex:1", "{text}" }
                    if let Some(a) = action { span { style: "text-decoration:underline;cursor:pointer", "{a}" } }
                    if *dismissible { Icon { icon: IconId::Close, size: 18, theme: theme.clone() } }
                }
            }
        }
        Demo::Snackbar { text, action } => {
            rsx! { Snackbar { text: text.clone(), action: action.clone() } }
        }
        Demo::Popup(inner) => rsx! {
            KitBox { kit: dodai_core::Kit::Popup, style: "flex-direction:column;align-items:stretch;min-width:200px;width:fit-content;".to_string(), {view(inner, cx)} }
        },
        Demo::Frame {
            h,
            body,
            scrim,
            layer,
            place,
        } => {
            use dodai_showcase_model::node::Place;
            let at = match place {
                Place::Centre => "top:50%;left:50%;transform:translate(-50%,-50%)",
                Place::Start => "top:0;bottom:0;left:0;display:flex",
                Place::End => "top:0;bottom:0;right:0;display:flex",
                Place::Bottom => "left:0;right:0;bottom:0",
            };
            let scrim = scrim.map(|c| c.to_css());
            rsx! {
                div { style: "position:relative;height:{h}px;width:100%;overflow:hidden;border:1px solid var(--dodai-border);padding:var(--dodai-space-md);box-sizing:border-box",
                    {view(body, cx)}
                    if let Some(c) = scrim { div { style: "position:absolute;inset:0;background:{c}" } }
                    div { style: "position:absolute;{at}", {view(layer, cx)} }
                }
            }
        }
        Demo::Glyph {
            icon,
            size,
            filled,
            ink,
        } => {
            let svg = icon
                .glyph(&theme, *filled)
                .map(|g| g.svg(*size as u32, &ink.to_css()))
                .unwrap_or_default();
            rsx! { span { style: "display:inline-flex;flex:none;width:{size}px;height:{size}px", dangerous_inner_html: "{svg}" } }
        }
        Demo::IconTile {
            icon,
            filled,
            pick,
            selected,
        } => {
            let key = pick.clone();
            let svg = icon
                .glyph(&theme, *filled)
                .map(|g| g.svg(28, &theme.palette.primary.to_css()))
                .unwrap_or_default();
            let name = dodai_showcase_model::icons::shortened(
                icon.name(),
                dodai_showcase_model::icons::TILE_NAME_CHARS,
            );
            let (sp, bd, muted, xs) = (
                theme.spacing.sm,
                if *selected {
                    theme.palette.primary
                } else {
                    theme.palette.border
                }
                .to_css(),
                theme.palette.text_muted.to_css(),
                theme.typography.size_xs,
            );
            rsx! {
                div { style: "width:92px;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;gap:4px;padding:{sp}px 4px;border:1px solid {bd};cursor:pointer",
                    onclick: move |_| if let Some(k) = key.clone() { act(model, Action::Press(k)) },
                    span { style: "display:inline-flex;width:28px;height:28px", dangerous_inner_html: "{svg}" }
                    span { style: "font-size:{xs}px;color:{muted};white-space:nowrap", "{name}" }
                }
            }
        }
        Demo::Frosted { w, h, panels } => {
            // The sharp picture, and over it each panel's blurred, tinted copy of what it covers
            // (the same pixels as the other toolkits, not the browser's backdrop-filter).
            let rects: Vec<[f32; 4]> = panels.iter().map(|p| p.rect).collect();
            let scene = dodai_showcase_model::media::frosted(&theme, *w as u32, *h as u32, &rects);
            let sharp = dodai_showcase_model::media::picture_url(&scene.sharp);
            let r = theme.spacing.md;
            let pad = theme.spacing.md;
            let edge = theme.frost().edge.map_or("none".to_string(), |(c, wd)| {
                format!("{wd}px solid {}", c.to_css())
            });
            let panes: Vec<(String, [f32; 4], &Demo)> = panels
                .iter()
                .zip(&scene.panes)
                .map(|(p, pane)| {
                    (
                        dodai_showcase_model::media::picture_url(pane),
                        p.rect,
                        &p.body,
                    )
                })
                .collect();
            rsx! {
                div { style: "position:relative;flex:none;overflow:hidden;width:{w}px;height:{h}px;background:url({sharp}) 0 0/100% 100%",
                    for (i, (url, [x, y, pw, ph], body)) in panes.into_iter().enumerate() {
                        div { key: "{i}",
                            style: "position:absolute;box-sizing:border-box;left:{x}px;top:{y}px;width:{pw}px;height:{ph}px;border-radius:{r}px;border:{edge};padding:{pad}px;background:url({url}) 0 0/100% 100%;display:flex;flex-direction:column",
                            {view(body, cx)}
                        }
                    }
                }
            }
        }
        Demo::Gizmo { yaw, pitch, size } => {
            let (yk, pk) = (yaw.key.clone(), pitch.key.clone());
            rsx! {
                AxisGizmo { yaw: vals.num(yaw), pitch: vals.num(pitch), size: *size, theme: theme.clone(),
                    onchange: move |(y, p)| {
                        act(model, Action::Set(yk.clone(), Value::Num(y)));
                        act(model, Action::Set(pk.clone(), Value::Num(p)));
                    } }
            }
        }
        Demo::Anchor {
            open,
            trigger,
            panel,
        } => {
            let shown = vals.flag(open);
            let key = open.key.clone();
            // A button trigger flips it through its own command.
            let wrap_click = !matches!(**trigger, Demo::Button(_));
            rsx! {
                div { style: "position:relative;display:inline-flex",
                    onclick: move |_| if wrap_click { act(model, Action::Set(key.clone(), Value::Bool(!shown))) },
                    {view(trigger, cx)}
                    if shown {
                        div { style: "position:absolute;top:calc(100% + 4px);left:0;z-index:50",
                            onclick: move |e| e.stop_propagation(),
                            {view(panel, cx)}
                        }
                    }
                }
            }
        }
        Demo::Icon(i, size) => rsx! { Icon { icon: *i, size: *size as u32, theme: theme.clone() } },
        Demo::Rating { value, max } => rsx! { Rating { value: *value, max: *max } },
        Demo::Stepper(b) => {
            let key = b.key.clone();
            rsx! {
                Stepper { value: vals.int(b), min: STEPPER_RANGE.0, max: STEPPER_RANGE.1,
                    onchange: move |v| act(model, Action::Set(key.clone(), Value::Int(v))) }
            }
        }
        Demo::Steps { steps, current } => rsx! {
            Steps { current: *current, steps: steps.iter().map(|s| StepItem { title: s.title.clone(), detail: s.detail.clone() }).collect::<Vec<_>>() }
        },
        Demo::Breadcrumb(items) => rsx! { Breadcrumb { items: items.clone() } },
        Demo::Pagination { pages, bind } => {
            let key = bind.key.clone();
            rsx! {
                Pagination { pages: *pages, page: vals.index(bind),
                    onselect: move |i| act(model, Action::Set(key.clone(), Value::Index(i))) }
            }
        }
        Demo::Tree(nodes) => {
            rsx! { TreeView { items: nodes.iter().map(tree_item).collect::<Vec<_>>() } }
        }
        Demo::Table(tb) => table(tb, &theme),
        Demo::Pick { bind, items } => {
            let at = vals.index(bind);
            rsx! {
                div { style: "display:flex;flex-direction:column;width:100%",
                    for (i, title) in items.iter().enumerate() {
                        {
                            let key = bind.key.clone();
                            rsx! {
                                KitBox { key: "{i}", kit: dodai_core::Kit::Row { selected: i == at },
                                    onclick: move |_| act(model, Action::Set(key.clone(), Value::Index(i))),
                                    span { "{title}" }
                                }
                            }
                        }
                    }
                }
            }
        }
        Demo::Rich { spans, role } => rich(spans, *role, &theme),
        Demo::Quote(body) => rsx! {
            div { style: "padding-left:var(--dodai-space-md);border-left:2px solid {theme.palette.primary.to_css()}",
                {view(body, cx)}
            }
        },
        Demo::AvatarGroup { names, max, size } => {
            let shown = max.unwrap_or(names.len()).min(names.len());
            let extra = names.len() - shown;
            let overlap = size * 0.25;
            let px = *size as u32;
            rsx! {
                div { style: "display:flex;flex:none",
                    for (i, n) in names[..shown].iter().enumerate() {
                        div { key: "{i}", style: if i > 0 { format!("margin-left:-{overlap}px") } else { String::new() },
                            Avatar { initials: n.clone(), size: px }
                        }
                    }
                    if extra > 0 {
                        div { style: "margin-left:-{overlap}px", Avatar { initials: format!("+{extra}"), size: px } }
                    }
                }
            }
        }
        Demo::Overlay(items) => rsx! {
            div { style: "display:grid;flex:none;place-items:center",
                for (i, n) in items.iter().enumerate() {
                    div { key: "{i}", style: "grid-area:1/1", {view(n, cx)} }
                }
            }
        },
        Demo::Avatar(s) => rsx! { Avatar { initials: s.clone() } },
        Demo::Tooltip {
            text,
            rich,
            title,
            action,
        } => {
            let ty = &theme.typography;
            let primary = theme.palette.primary.to_css();
            rsx! {
                KitBox { kit: dodai_core::Kit::Tooltip { rich: *rich }, style: format!("flex-direction:column;align-items:flex-start;width:fit-content;font-size:{}px", ty.size_sm),
                    if let Some(h) = title { span { style: "font-size:{ty.size_md}px", "{h}" } }
                    span { "{text}" }
                    if let Some(a) = action { span { style: "color:{primary}", "{a}" } }
                }
            }
        }
        Demo::Code { text, lang } => code_block(text, lang, &theme),
        Demo::Timeline(events) => rsx! {
            Timeline { events: events.iter().map(|e| TimelineItem { title: e.title.clone(), time: e.time.clone(), detail: e.detail.clone(), tone: e.tone }).collect::<Vec<_>>() }
        },
        Demo::Badged { icon, count, dot } => {
            rsx! { Badged { icon: *icon, count: *count, dot: *dot } }
        }
        Demo::Spinner(size) => rsx! { Spinner { size: *size as u32 } },
        Demo::SpinnerOf { size, style } => {
            rsx! { Spinner { size: *size as u32, style: Some(*style) } }
        }
        Demo::StyledProgress { style, value } => {
            rsx! { StyledProgress { style: *style, value: *value } }
        }
        Demo::Indeterminate => rsx! { Indeterminate {} },
        Demo::Skeleton { w, h } => rsx! { Skeleton { width: *w as u32, height: *h as u32 } },
        Demo::Empty {
            icon: glyph,
            title,
            detail,
        } => {
            let ty = &theme.typography;
            rsx! {
                div { style: "display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-lg)",
                    span { style: text_style(styled(ty.size_xxl, Font::Body, Ink::Muted), &theme), "{glyph}" }
                    span { style: text_style(styled(ty.size_lg, Font::Body, Ink::Text), &theme), "{title}" }
                    span { style: text_style(styled(ty.size_sm, Font::Body, Ink::Muted), &theme), "{detail}" }
                }
            }
        }
    }
}

/// The model's tree node as the component's.
pub fn tree_item(n: &dodai_showcase_model::node::TreeNode) -> TreeItem {
    TreeItem {
        label: n.label.clone(),
        icon: n.icon,
        children: n.children.iter().map(tree_item).collect(),
        open: n.open,
        selected: n.selected,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_showcase_model::node::TreeNode;

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
    fn text_styles_carry_size_family_and_colour() {
        let t = dodai_core::themes::all().remove(0);
        let big = text_style(Role::Type(dodai_core::TypeRole::DisplayLarge), &t);
        assert!(big.contains("font-family:var(--dodai-font-display)"));
        assert!(big.contains("font-weight:"));
        let small = text_style(Role::Type(dodai_core::TypeRole::LabelSmall), &t);
        assert!(small.contains("font-family:var(--dodai-font-body)"));
        let mono = text_style(Role::Mono, &t);
        assert!(mono.contains("var(--dodai-font-mono)"));
        assert!(mono.contains(&format!("color:{}", t.palette.text.to_css())));
    }

    #[test]
    fn tree_items_keep_labels_nesting_and_open_state() {
        let n = TreeNode {
            label: "src".into(),
            icon: None,
            open: true,
            selected: false,
            children: vec![TreeNode {
                label: "a".into(),
                icon: None,
                children: vec![],
                open: false,
                selected: true,
            }],
        };
        let i = tree_item(&n);
        assert_eq!(
            (i.label.as_str(), i.open, i.children.len()),
            ("src", true, 1)
        );
        assert_eq!(i.children[0].label, "a");
    }
}

/// A data table: header row, banded rows; fixed-width columns keep their width, the others
/// share the rest; scrolls inside `height` when set.
fn table(tb: &dodai_showcase_model::node::Table, theme: &dodai_core::Theme) -> Element {
    use dodai_showcase_model::node::Align;
    let tracks = tb
        .columns
        .iter()
        .map(|c| c.width.map_or("1fr".to_string(), |w| format!("{w}px")))
        .collect::<Vec<_>>()
        .join(" ");
    let align = |a: Align| match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    };
    let aligns: Vec<&str> = tb.columns.iter().map(|c| align(c.align)).collect();
    let scroll = tb.height.map_or(String::new(), |h| {
        format!("max-height:{h}px;overflow:auto;")
    });
    let label = text_style(Role::Label, theme);
    rsx! {
        div { class: "dodai-datatable", role: "table", style: "width:100%",
            KitBox { kit: dodai_core::Kit::TableHead, style: format!("display:grid;grid-template-columns:{tracks};"),
                for (i, c) in tb.columns.iter().enumerate() { span { key: "{i}", style: "{label}text-align:{aligns[i]}", "{c.title}" } }
            }
            div { style: scroll,
                for (i, r) in tb.rows.iter().enumerate() {
                    KitBox { key: "{i}", kit: dodai_core::Kit::TableRow(i % 2 == 1), style: format!("display:grid;grid-template-columns:{tracks};"),
                        for (j, c) in r.iter().enumerate() { span { key: "{j}", style: "text-align:{aligns.get(j).copied().unwrap_or(\"left\")}", "{c}" } }
                    }
                }
            }
        }
    }
}

/// A code listing: the language over syntax-coloured lines.
fn code_block(code: &str, lang: &str, theme: &dodai_core::Theme) -> Element {
    use dodai_core::syntax::{highlight_lines, CodeToken as K};
    let p = &theme.palette;
    let colour = |k: K| match k {
        K::Keyword => p.primary,
        K::Str => p.success,
        K::Comment => p.text_muted,
        K::Number => p.warning,
        K::Type => p.info,
    };
    let lines: Vec<Vec<(String, Option<String>)>> = code
        .split('\n')
        .zip(highlight_lines(code, lang))
        .map(|(l, spans)| {
            let mut out = Vec::new();
            let mut at = 0;
            for (r, k) in spans {
                if r.start > at {
                    out.push((l[at..r.start].to_string(), None));
                }
                out.push((l[r.clone()].to_string(), Some(colour(k).to_css())));
                at = r.end;
            }
            out.push((l[at..].to_string(), None));
            out
        })
        .collect();
    let cap = text_style(Role::Caption, theme);
    rsx! {
        KitBox { kit: dodai_core::Kit::Code, style: "display:block;width:100%".to_string(),
            if !lang.is_empty() { div { style: cap, "{lang.to_uppercase()}" } }
            pre { class: "dodai-code", style: "margin:0",
                for (i, l) in lines.into_iter().enumerate() {
                    div { key: "{i}", style: "min-height:1.3em",
                        for (j, (t, c)) in l.into_iter().enumerate() {
                            span { key: "{j}", style: c.map_or(String::new(), |c| format!("color:{c}")), "{t}" }
                        }
                    }
                }
            }
        }
    }
}

/// Inline-styled runs as one paragraph.
fn rich(
    spans: &[dodai_showcase_model::node::Span],
    role: Role,
    theme: &dodai_core::Theme,
) -> Element {
    let base = text_style(role, theme);
    let p = &theme.palette;
    let (code_fg, code_bg, link) = (
        p.primary.to_css(),
        p.surface_raised.to_css(),
        p.link.to_css(),
    );
    rsx! {
        span { style: base,
            for (i, s) in spans.iter().enumerate() {
                span { key: "{i}",
                    style: format!(
                        "{}{}{}{}",
                        if s.bold { "font-weight:700;" } else { "" },
                        if s.italic { "font-style:italic;" } else { "" },
                        if s.code { format!("font-family:var(--dodai-font-mono);color:{code_fg};background:{code_bg};") } else { String::new() },
                        if s.link { format!("color:{link};text-decoration:underline;") } else { String::new() },
                    ),
                    "{s.text}"
                }
            }
        }
    }
}
