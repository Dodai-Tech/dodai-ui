//! Text components. `uppercase_labels` is honoured by CSS (`--lg-label-case`).
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TextProps {
    #[prop_or_default]
    pub children: Html,
}

/// Heading level 1–4 (tag `h1`..`h4`), themed display font.
#[derive(Properties, PartialEq)]
pub struct HeadingProps {
    #[prop_or(2)]
    pub level: u8,
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn Heading(props: &HeadingProps) -> Html {
    let c = props.children.clone();
    match props.level {
        1 => html! { <h1 class="lg-heading">{c}</h1> },
        3 => html! { <h3 class="lg-heading">{c}</h3> },
        4 => html! { <h4 class="lg-heading">{c}</h4> },
        _ => html! { <h2 class="lg-heading">{c}</h2> },
    }
}

/// Small caps-style label (uppercase when the theme says so).
#[function_component]
pub fn Label(props: &TextProps) -> Html {
    html! { <span class="lg-label">{ props.children.clone() }</span> }
}

/// Muted body text.
#[function_component]
pub fn Muted(props: &TextProps) -> Html {
    html! { <p class="lg-muted">{ props.children.clone() }</p> }
}

/// Monospace inline text.
#[function_component]
pub fn Mono(props: &TextProps) -> Html {
    html! { <code>{ props.children.clone() }</code> }
}
