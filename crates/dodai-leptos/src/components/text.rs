use leptos::prelude::*;

/// Heading (`level` 1–4). Case follows the theme's `uppercase_labels` via CSS.
#[component]
pub fn Heading(#[prop(default = 2)] level: u8, children: Children) -> impl IntoView {
    match level {
        1 => view! { <h1>{children()}</h1> }.into_any(),
        3 => view! { <h3>{children()}</h3> }.into_any(),
        4 => view! { <h4>{children()}</h4> }.into_any(),
        _ => view! { <h2>{children()}</h2> }.into_any(),
    }
}

/// Small caption label.
#[component]
pub fn Label(children: Children) -> impl IntoView {
    view! { <span class="lg-label">{children()}</span> }
}

/// Muted paragraph text.
#[component]
pub fn Muted(children: Children) -> impl IntoView {
    view! { <p class="lg-muted">{children()}</p> }
}

/// Monospace inline text.
#[component]
pub fn Mono(children: Children) -> impl IntoView {
    view! { <code class="lg-mono">{children()}</code> }
}
