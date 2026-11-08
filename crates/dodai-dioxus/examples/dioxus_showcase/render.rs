//! Walks the model's [`Node`] tree and builds dodai-dioxus components.

use dioxus::prelude::*;
use dodai_core::{ControlSize, ToggleKind};
use dodai_dioxus::*;
use dodai_icons::IconId;
use dodai_showcase_model::node::{Axis, Gap, TextRole as Role};
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

/// CSS class of a stack gap.
pub fn gap_class(g: Gap) -> &'static str {
    match g {
        Gap::Xs => "dodai-gap-xs",
        Gap::Sm => "dodai-gap-sm",
        Gap::Md => "dodai-gap-md",
        Gap::Lg => "dodai-gap-lg",
    }
}

/// The `class` of a text role.
pub fn text_class(r: Role) -> &'static str {
    match r {
        Role::Title => "dodai-heading",
        Role::Label => "dodai-label",
        Role::Body => "",
        Role::Muted => "dodai-muted",
        Role::Caption => "dodai-caption",
        Role::Mono => "dodai-mono",
        Role::Type(_) => "",
    }
}

/// The inline style of a text role (the type scale sets size and weight).
pub fn text_style(r: Role, theme: &dodai_core::Theme) -> String {
    match r {
        Role::Type(t) => {
            let s = theme.type_style(t);
            format!(
                "font-size:{}px;line-height:{}px;font-weight:{};{}",
                s.size,
                s.line_height,
                s.weight,
                if t.is_heading() {
                    "font-family:var(--dodai-font-display)"
                } else {
                    ""
                }
            )
        }
        _ => String::new(),
    }
}

/// Everything the builder reads.
#[derive(Clone, Copy)]
pub struct Cx {
    pub model: Signal<Showcase>,
}

fn act(mut model: Signal<Showcase>, a: Action) {
    model.write().update(a);
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
            let align = if s.axis == Axis::Row && !s.centre {
                "align-items:flex-start"
            } else if s.axis == Axis::Column && s.centre {
                "align-items:center;width:auto"
            } else {
                ""
            };
            rsx! {
                div { class: "dodai-stack {axis} {wrap} {gap_class(s.gap)}", style: align,
                    for (i, n) in s.items.iter().enumerate() { {rsx! { Fragment { key: "{i}", {view(n, cx)} } }} }
                }
            }
        }
        Demo::Text(t, r) => {
            rsx! { span { class: text_class(*r), style: text_style(*r, &theme), "{t}" } }
        }
        Demo::Space(px) => rsx! { div { style: "width:{px}px;height:{px}px" } },
        Demo::Swatch { name, color } => rsx! {
            div { class: "dodai-swatch",
                div { class: "dodai-swatch__chip", style: "background:{color.to_css()}" }
                span { class: "dodai-caption", "{name}" }
                code { class: "dodai-mono dodai-caption", "#{color.r:02x}{color.g:02x}{color.b:02x}" }
            }
        },
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
                    if let Some(l) = label { span { class: "dodai-caption", "{l}" } }
                }
            }
        }
        Demo::Button(b) => {
            let (key, variant) = (b.counter.clone(), b.variant);
            rsx! {
                Button { variant, size: Some(b.size), square: b.square, icon_only: b.label.is_empty(),
                    disabled: b.disabled, loading: b.loading,
                    onclick: move |_| if let Some(k) = key.clone() { act(model, Action::Press(k)) },
                    if let Some(i) = b.icon { Icon { icon: i, size: 16, theme: theme.clone() } }
                    if !b.label.is_empty() { "{b.label}" }
                }
            }
        }
        Demo::Split { label, variant } => rsx! {
            span { class: "dodai-split",
                Button { variant: *variant, size: Some(ControlSize::Medium), "{label}" }
                Button { variant: *variant, size: Some(ControlSize::Medium), icon_only: true,
                    Icon { icon: IconId::ChevronDown, size: 16, theme: theme.clone() } }
            }
        },
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
                    onselect: move |i| act(model, Action::Set(key.clone(), Value::Index(i))) }
            }
        }
        Demo::Tag { label, tone, solid } => {
            rsx! { Tag { kind: TagKind::from(*tone), soft: !*solid, "{label}" } }
        }
        Demo::Chip { label, bind, icon } => {
            let on = vals.flag(bind);
            let key = bind.key.clone();
            rsx! {
                Chip { label: label.clone(), selected: on, icon: *icon,
                    onclick: move |_| act(model, Action::Set(key.clone(), Value::Bool(!on))) }
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
                    Checkbox { checked: vals.flag(&t.bind), disabled: t.disabled, label: t.label.clone(),
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
            rsx! {
                Slider { value: v, min, max, step: step.unwrap_or((max - min) / 100.0), vertical: true,
                    onchange: move |x| act(model, Action::Set(key.clone(), Value::Num(snap(x, min, max, step)))) }
            }
        }
        Demo::Slider(s) => {
            let v = vals.num(&s.bind);
            let (key, min, max, step) = (s.bind.key.clone(), s.min, s.max, s.step);
            rsx! {
                div { class: "dodai-stack dodai-stack--row", style: "max-width:420px;width:100%",
                    div { style: "flex:1", Slider { value: v, min, max, step: step.unwrap_or((max - min) / 100.0),
                        onchange: move |x| act(model, Action::Set(key.clone(), Value::Num(snap(x, min, max, step)))) } }
                    if s.readout { code { class: "dodai-mono", "{num_text(v, step)}" } }
                }
            }
        }
        Demo::Progress { value, tone } => rsx! {
            div { style: "max-width:420px;width:100%", Progress { value: *value, tone: *tone } }
        },
        Demo::Input(i) => {
            let key = i.bind.key.clone();
            rsx! {
                div { class: "dodai-field", style: "max-width:420px;width:100%",
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
                    KitBox { kit: dodai_core::Kit::Row { selected: false }, style: "gap:var(--dodai-space-md)".to_string(),
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
        Demo::Card {
            title,
            body,
            variant,
        } => rsx! {
            Card { variant: *variant, title: title.clone(), {view(body, cx)} }
        },
        Demo::Alert { tone, text, banner } => {
            rsx! { Alert { tone: *tone, banner: *banner, "{text}" } }
        }
        Demo::Snackbar { text, action } => {
            rsx! { Snackbar { text: text.clone(), action: action.clone() } }
        }
        Demo::Popup(inner) => rsx! {
            KitBox { kit: dodai_core::Kit::Popup, style: "flex-direction:column;align-items:stretch;min-width:200px;width:fit-content;".to_string(), {view(inner, cx)} }
        },
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
        Demo::Table { columns, rows } => {
            rsx! { DataTable { columns: columns.clone(), rows: rows.clone() } }
        }
        Demo::Avatar(s) => rsx! { Avatar { initials: s.clone() } },
        Demo::Tooltip { text, rich } => rsx! { TooltipBubble { text: text.clone(), rich: *rich } },
        Demo::Code(s) => rsx! { CodeBlock { code: s.clone() } },
        Demo::Timeline(events) => rsx! {
            Timeline { events: events.iter().map(|e| TimelineItem { title: e.title.clone(), time: e.time.clone(), detail: e.detail.clone(), tone: e.tone }).collect::<Vec<_>>() }
        },
        Demo::Badged { icon, count, dot } => {
            rsx! { Badged { icon: *icon, count: *count, dot: *dot } }
        }
        Demo::Spinner(size) => rsx! { Spinner { size: *size as u32 } },
        Demo::Indeterminate => rsx! { Indeterminate {} },
        Demo::Skeleton { w, h } => rsx! { Skeleton { width: *w as u32, height: *h as u32 } },
        Demo::Empty { title, detail } => {
            rsx! { EmptyState { title: title.clone(), detail: detail.clone() } }
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
    fn gap_and_text_classes_are_distinct() {
        let gaps = [Gap::Xs, Gap::Sm, Gap::Md, Gap::Lg].map(gap_class);
        for (i, a) in gaps.iter().enumerate() {
            assert!(gaps.iter().skip(i + 1).all(|b| a != b));
        }
        assert_eq!(text_class(Role::Muted), "dodai-muted");
    }

    #[test]
    fn the_type_scale_sets_size_weight_and_the_display_font() {
        let t = dodai_core::themes::all().remove(0);
        let big = text_style(Role::Type(dodai_core::TypeRole::DisplayLarge), &t);
        assert!(
            big.contains("font-size:") && big.contains("font-family:var(--dodai-font-display)")
        );
        let small = text_style(Role::Type(dodai_core::TypeRole::LabelSmall), &t);
        assert!(!small.contains("font-family"));
        assert_eq!(text_style(Role::Body, &t), "");
    }

    #[test]
    fn tree_items_keep_labels_nesting_and_open_state() {
        let n = TreeNode {
            label: "src".into(),
            icon: None,
            open: true,
            children: vec![TreeNode {
                label: "a".into(),
                icon: None,
                children: vec![],
                open: false,
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
