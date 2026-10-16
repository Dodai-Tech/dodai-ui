//! Typed components emitting the `.lg-*` classes styled by `dodai-topcoat` CSS.

use dioxus::prelude::*;
use dodai_core::{ButtonVariant, Tone};

/// CSS modifier class for a [`ButtonVariant`].
pub fn button_class(v: ButtonVariant) -> &'static str {
    match v {
        ButtonVariant::Primary => "lg-btn lg-btn--primary",
        ButtonVariant::Secondary => "lg-btn lg-btn--secondary",
        ButtonVariant::Outline => "lg-btn lg-btn--outline",
        ButtonVariant::OutlineSecondary => "lg-btn lg-btn--outline-secondary",
        ButtonVariant::Danger => "lg-btn lg-btn--danger",
        ButtonVariant::Ghost => "lg-btn lg-btn--ghost",
    }
}

/// Tag colour: a status [`Tone`] or a brand colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TagKind {
    Primary,
    Secondary,
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl From<Tone> for TagKind {
    fn from(t: Tone) -> Self {
        match t {
            Tone::Info => Self::Info,
            Tone::Success => Self::Success,
            Tone::Warning => Self::Warning,
            Tone::Danger => Self::Danger,
        }
    }
}

impl TagKind {
    fn class(self) -> &'static str {
        match self {
            Self::Primary => "lg-tag--primary",
            Self::Secondary => "lg-tag--secondary",
            Self::Info => "",
            Self::Success => "lg-tag--success",
            Self::Warning => "lg-tag--warning",
            Self::Danger => "lg-tag--danger",
        }
    }
}

/// Button properties.
#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default = ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

/// Themed button (all [`ButtonVariant`]s, disabled state).
#[component]
pub fn Button(props: ButtonProps) -> Element {
    let class = button_class(props.variant);
    rsx! {
        button { class, disabled: props.disabled, onclick: move |e| props.onclick.call(e), {props.children} }
    }
}

/// Panel properties.
#[derive(Props, Clone, PartialEq)]
pub struct PanelProps {
    #[props(default)]
    pub title: Option<String>,
    /// Highlight with the primary border + glow.
    #[props(default)]
    pub focus: bool,
    #[props(default)]
    pub raised: bool,
    pub children: Element,
}

/// Card / panel with optional title; chamfered per theme shape.
#[component]
pub fn Panel(props: PanelProps) -> Element {
    let mut class = String::from("lg-panel");
    if props.focus {
        class.push_str(" lg-panel--focus");
    }
    if props.raised {
        class.push_str(" lg-panel--raised");
    }
    rsx! {
        div { class,
            if let Some(t) = props.title { div { class: "lg-panel__title", "{t}" } }
            {props.children}
        }
    }
}

/// Tag properties.
#[derive(Props, Clone, PartialEq)]
pub struct TagProps {
    #[props(default)]
    pub kind: TagKind,
    #[props(default)]
    pub soft: bool,
    pub children: Element,
}

/// Status tag.
#[component]
pub fn Tag(props: TagProps) -> Element {
    let soft = if props.soft { "lg-tag--soft" } else { "" };
    let class = format!("lg-tag {} {soft}", props.kind.class());
    rsx! { span { class, {props.children} } }
}

/// Count badge.
#[component]
pub fn Badge(children: Element) -> Element {
    rsx! { span { class: "lg-badge", {children} } }
}

/// Tabs properties.
#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    pub tabs: Vec<String>,
    pub active: usize,
    #[props(default)]
    pub onselect: EventHandler<usize>,
}

/// Tab strip; active tab is controlled by `active`.
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    rsx! {
        div { class: "lg-tabs", role: "tablist",
            for (i, t) in props.tabs.iter().enumerate() {
                a {
                    key: "{i}",
                    class: if i == props.active { "lg-tab lg-tab--active" } else { "lg-tab" },
                    role: "tab",
                    "aria-selected": "{i == props.active}",
                    href: "#",
                    onclick: move |e| { e.prevent_default(); props.onselect.call(i) },
                    "{t}"
                }
            }
        }
    }
}

/// Toggle properties.
#[derive(Props, Clone, PartialEq)]
pub struct CheckProps {
    pub checked: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub label: String,
    #[props(default)]
    pub onchange: EventHandler<bool>,
}

/// Switch.
#[component]
pub fn Toggle(props: CheckProps) -> Element {
    rsx! {
        label { class: "lg-switch",
            input { r#type: "checkbox", role: "switch", checked: props.checked, disabled: props.disabled,
                onchange: move |e| props.onchange.call(e.checked()) }
            span { class: "lg-switch__box" }
            "{props.label}"
        }
    }
}

/// Checkbox.
#[component]
pub fn Checkbox(props: CheckProps) -> Element {
    rsx! {
        label { class: "lg-check",
            input { r#type: "checkbox", checked: props.checked, disabled: props.disabled,
                onchange: move |e| props.onchange.call(e.checked()) }
            span { class: "lg-check__box" }
            "{props.label}"
        }
    }
}

/// Radio properties.
#[derive(Props, Clone, PartialEq)]
pub struct RadioProps {
    pub checked: bool,
    #[props(default)]
    pub name: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub label: String,
    #[props(default)]
    pub onselect: EventHandler<()>,
}

/// Radio button.
#[component]
pub fn Radio(props: RadioProps) -> Element {
    rsx! {
        label { class: "lg-radio",
            input { r#type: "radio", name: props.name, checked: props.checked, disabled: props.disabled,
                onchange: move |_| props.onselect.call(()) }
            span { class: "lg-radio__box" }
            "{props.label}"
        }
    }
}

/// Progress properties.
#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    /// 0.0..=1.0
    pub value: f32,
    #[props(default)]
    pub tone: Option<Tone>,
}

/// Progress bar.
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let pct = props.value.clamp(0.0, 1.0) * 100.0;
    let tone = match props.tone {
        Some(Tone::Success) => "lg-progress--success",
        Some(Tone::Warning) => "lg-progress--warning",
        Some(Tone::Danger) => "lg-progress--danger",
        _ => "",
    };
    rsx! {
        div { class: "lg-progress {tone}", role: "progressbar", "aria-valuenow": "{pct:.0}",
            span { style: "--v:{pct}%" }
        }
    }
}

/// Slider properties.
#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    pub value: f32,
    #[props(default = 0.0)]
    pub min: f32,
    #[props(default = 1.0)]
    pub max: f32,
    #[props(default = 0.01)]
    pub step: f32,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onchange: EventHandler<f32>,
}

/// Range slider (filled track via `--v`).
#[component]
pub fn Slider(props: SliderProps) -> Element {
    let span = (props.max - props.min).max(f32::EPSILON);
    let pct = ((props.value - props.min) / span).clamp(0.0, 1.0) * 100.0;
    rsx! {
        input { class: "lg-slider", r#type: "range", min: "{props.min}", max: "{props.max}", step: "{props.step}",
            value: "{props.value}", disabled: props.disabled, style: "--v:{pct}%",
            oninput: move |e| if let Ok(v) = e.value().parse::<f32>() { props.onchange.call(v) } }
    }
}

/// Text input properties.
#[derive(Props, Clone, PartialEq)]
pub struct TextInputProps {
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub oninput: EventHandler<String>,
}

/// Single-line text input.
#[component]
pub fn TextInput(props: TextInputProps) -> Element {
    rsx! {
        input { class: "lg-input", r#type: "text", value: props.value, placeholder: props.placeholder,
            disabled: props.disabled, oninput: move |e| props.oninput.call(e.value()) }
    }
}

/// Wrapper `<ul class="lg-list">` for [`ListItem`]s.
#[component]
pub fn List(children: Element) -> Element {
    rsx! { ul { class: "lg-list", role: "listbox", {children} } }
}

/// List item properties.
#[derive(Props, Clone, PartialEq)]
pub struct ListItemProps {
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    pub children: Element,
}

/// Selectable row.
#[component]
pub fn ListItem(props: ListItemProps) -> Element {
    rsx! {
        li { class: "lg-list__item", role: "option", "aria-selected": "{props.selected}",
            onclick: move |e| props.onclick.call(e), {props.children} }
    }
}

/// Horizontal rule.
#[component]
pub fn Divider() -> Element {
    rsx! { hr { class: "lg-divider" } }
}

/// Tooltip properties.
#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    pub tip: String,
    pub children: Element,
}

/// Hover tooltip around its children.
#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    rsx! { span { class: "lg-tip", "data-tip": props.tip, tabindex: "0", {props.children} } }
}

/// Heading (`h1`..`h4`, honours `uppercase_labels` via CSS).
#[component]
pub fn Heading(#[props(default = 2)] level: u8, children: Element) -> Element {
    match level {
        1 => rsx! { h1 { {children} } },
        2 => rsx! { h2 { {children} } },
        3 => rsx! { h3 { {children} } },
        _ => rsx! { h4 { {children} } },
    }
}

/// Small label text (case follows the theme).
#[component]
pub fn Label(children: Element) -> Element {
    rsx! { span { class: "lg-label", {children} } }
}

/// Muted body text.
#[component]
pub fn Muted(children: Element) -> Element {
    rsx! { span { class: "lg-muted", {children} } }
}

/// Monospace text.
#[component]
pub fn Mono(children: Element) -> Element {
    rsx! { code { class: "lg-mono", {children} } }
}
