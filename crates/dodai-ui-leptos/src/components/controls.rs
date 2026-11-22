use crate::class;
use leptos::prelude::*;

/// Toggle switch.
#[component]
pub fn Toggle(
    #[prop(optional)] checked: bool,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <label class="dodai-switch">
            <input type="checkbox" checked=checked disabled=disabled/>
            <span class="dodai-switch__box"></span>{children()}
        </label>
    }
}

/// Checkbox.
#[component]
pub fn Checkbox(
    #[prop(optional)] checked: bool,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <label class="dodai-check">
            <input type="checkbox" checked=checked disabled=disabled/>
            <span class="dodai-check__box"></span>{children()}
        </label>
    }
}

/// Radio button in group `name`.
#[component]
pub fn Radio(
    #[prop(into)] name: String,
    #[prop(optional)] checked: bool,
    #[prop(optional)] disabled: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <label class="dodai-radio">
            <input type="radio" name=name checked=checked disabled=disabled/>
            <span class="dodai-radio__box"></span>{children()}
        </label>
    }
}

/// Range slider, `value` 0–100.
#[component]
pub fn Slider(value: f32, #[prop(optional)] disabled: bool) -> impl IntoView {
    let style = format!("--v:{}", class::percent(value));
    view! { <input class="dodai-slider" type="range" min="0" max="100" value=value.to_string() style=style disabled=disabled/> }
}

/// Single-line text input.
#[component]
pub fn TextInput(
    #[prop(optional, into)] placeholder: String,
    #[prop(optional, into)] value: String,
    #[prop(optional)] disabled: bool,
) -> impl IntoView {
    let (glass, style) = crate::glass(
        dodai_ui_core::SurfaceClass::Input,
        &dodai_ui_core::Surface::INHERIT,
    );
    view! { <input class=format!("dodai-input{glass}") style=style placeholder=placeholder value=value disabled=disabled/> }
}
