use crate::class;
use dodai_core::{ButtonVariant, Tone};
use leptos::prelude::*;

/// Button with any [`ButtonVariant`]; `disabled` renders the disabled style.
#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(optional)] disabled: bool,
    /// The button's own surface override (frost, fill opacity).
    #[prop(optional)]
    surface: dodai_core::Surface,
    children: Children,
) -> impl IntoView {
    let (glass, style) = crate::glass(dodai_core::SurfaceClass::Button, &surface);
    let class = format!("{}{glass}", class::button(variant));
    view! { <button class=class style=style disabled=disabled>{children()}</button> }
}

/// Panel / card with optional title.
#[component]
pub fn Panel(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] focused: bool,
    #[prop(optional)] raised: bool,
    /// The panel's own surface override (frost, fill opacity).
    #[prop(optional)]
    surface: dodai_core::Surface,
    children: Children,
) -> impl IntoView {
    let (glass, style) = crate::glass(dodai_core::SurfaceClass::Panel, &surface);
    let class = format!("{}{glass}", class::panel(focused, raised));
    view! {
        <div class=class style=style>
            <span class="dodai-panel__shade" aria-hidden="true"></span>
            {title.map(|t| view! { <div class="dodai-panel__title">{t}</div> })}
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
        <nav class="dodai-tabs">
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
    view! { <li class="dodai-list__item" aria-selected=selected.to_string()>{children()}</li> }
}

/// List container for [`ListItem`]s.
#[component]
pub fn List(children: Children) -> impl IntoView {
    let (glass, style) = crate::glass(
        dodai_core::SurfaceClass::Card,
        &dodai_core::Surface::INHERIT,
    );
    view! { <ul class=format!("dodai-list{glass}") style=style>{children()}</ul> }
}

/// Horizontal divider.
#[component]
pub fn Divider() -> impl IntoView {
    view! { <hr class="dodai-divider"/> }
}

/// CSS-only tooltip: shows `text` on hover/focus of the children.
#[component]
pub fn Tooltip(#[prop(into)] text: String, children: Children) -> impl IntoView {
    view! { <span class="dodai-tip" data-tip=text>{children()}</span> }
}

/// A busy spinner in any [`dodai_core::SpinnerStyle`] (the plain ring by default).
#[component]
pub fn Spinner(
    #[prop(default = 24)] size: u32,
    #[prop(default = dodai_core::SpinnerStyle::Ring)] style: dodai_core::SpinnerStyle,
) -> impl IntoView {
    view! { <span inner_html=dodai_topcoat::html::spinner_of(style, size)></span> }
}

/// A progress bar in a design's own style (hud, stripes, stones); `value` 0–1, or none for a running bar.
#[component]
pub fn StyledProgress(
    style: dodai_core::ProgressStyle,
    #[prop(optional)] value: Option<f32>,
) -> impl IntoView {
    view! { <div inner_html=dodai_topcoat::html::styled_progress(style, value)></div> }
}
