use crate::class;
use dodai_core::{ButtonVariant, Tone};
use leptos::prelude::*;

/// Button with any [`ButtonVariant`]; `disabled` renders the disabled style.
#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    view! { <button class=class::button(variant) disabled=disabled>{children()}</button> }
}

/// Panel / card with optional title.
#[component]
pub fn Panel(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] focused: bool,
    #[prop(optional)] raised: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=class::panel(focused, raised)>
            {title.map(|t| view! { <div class="lg-panel__title">{t}</div> })}
            {children()}
        </div>
    }
}

/// Tag (or compact badge when `badge`).
#[component]
pub fn Tag(
    #[prop(optional)] tone: class::TagTone,
    #[prop(optional)] soft: bool,
    #[prop(optional)] badge: bool,
    children: Children,
) -> impl IntoView {
    view! { <span class=class::tag(tone, soft, badge)>{children()}</span> }
}

/// Tab strip: `tabs` are labels, `active` the selected index.
#[component]
pub fn Tabs(tabs: Vec<String>, #[prop(optional)] active: usize) -> impl IntoView {
    view! {
        <nav class="lg-tabs">
            {tabs
                .into_iter()
                .enumerate()
                .map(|(i, t)| view! { <a class=class::tab(i == active) href="#">{t}</a> })
                .collect_view()}
        </nav>
    }
}

/// Progress bar, `value` 0–100.
#[component]
pub fn Progress(value: f32, #[prop(optional)] tone: Option<Tone>) -> impl IntoView {
    let v = format!("--v:{}", class::percent(value));
    view! { <div class=class::progress(tone)><span style=v></span></div> }
}

/// Selectable list row.
#[component]
pub fn ListItem(#[prop(optional)] selected: bool, children: Children) -> impl IntoView {
    view! { <li class="lg-list__item" aria-selected=selected.to_string()>{children()}</li> }
}

/// List container for [`ListItem`]s.
#[component]
pub fn List(children: Children) -> impl IntoView {
    view! { <ul class="lg-list">{children()}</ul> }
}

/// Horizontal divider.
#[component]
pub fn Divider() -> impl IntoView {
    view! { <hr class="lg-divider"/> }
}

/// CSS-only tooltip: shows `text` on hover/focus of the children.
#[component]
pub fn Tooltip(#[prop(into)] text: String, children: Children) -> impl IntoView {
    view! { <span class="lg-tip" data-tip=text>{children()}</span> }
}
