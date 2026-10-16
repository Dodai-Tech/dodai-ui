//! Interactive controls: button, toggle, checkbox, radio, slider, text input.
use crate::classes::{button, clamp_pct};
use crate::dom::{input_checked, input_value};
use dodai_core::ButtonVariant;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    #[prop_or(ButtonVariant::Primary)]
    pub variant: ButtonVariant,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// Button (all [`ButtonVariant`]s, chamfered/rounded/square from tokens).
#[function_component]
pub fn Button(props: &ButtonProps) -> Html {
    html! {
        <button class={button(props.variant)} disabled={props.disabled}
                onclick={props.onclick.clone()}>{ props.children.clone() }</button>
    }
}

#[derive(Properties, PartialEq)]
pub struct CheckProps {
    pub checked: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onchange: Callback<bool>,
    #[prop_or_default]
    pub children: Html,
}

fn check_like(
    class: &'static str,
    box_class: &'static str,
    kind: &'static str,
    p: &CheckProps,
    name: Option<AttrValue>,
) -> Html {
    let cb = p.onchange.clone();
    let onchange = Callback::from(move |e: Event| cb.emit(input_checked(&e)));
    html! {
        <label class={class}>
            <input type={kind} name={name} checked={p.checked} disabled={p.disabled} {onchange} />
            <span class={box_class}></span>{ p.children.clone() }
        </label>
    }
}

/// On/off switch.
#[function_component]
pub fn Toggle(props: &CheckProps) -> Html {
    check_like("lg-switch", "lg-switch__box", "checkbox", props, None)
}

/// Checkbox.
#[function_component]
pub fn Checkbox(props: &CheckProps) -> Html {
    check_like("lg-check", "lg-check__box", "checkbox", props, None)
}

#[derive(Properties, PartialEq)]
pub struct RadioProps {
    pub checked: bool,
    /// Radio group name.
    pub name: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onchange: Callback<bool>,
    #[prop_or_default]
    pub children: Html,
}

/// Radio button.
#[function_component]
pub fn Radio(props: &RadioProps) -> Html {
    let p = CheckProps {
        checked: props.checked,
        disabled: props.disabled,
        onchange: props.onchange.clone(),
        children: props.children.clone(),
    };
    check_like(
        "lg-radio",
        "lg-radio__box",
        "radio",
        &p,
        Some(props.name.clone()),
    )
}

#[derive(Properties, PartialEq)]
pub struct SliderProps {
    /// 0–100.
    pub value: f32,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onchange: Callback<f32>,
}

/// Range slider (0–100).
#[function_component]
pub fn Slider(props: &SliderProps) -> Html {
    let v = clamp_pct(props.value);
    let cb = props.onchange.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        if let Ok(f) = input_value(&e).parse::<f32>() {
            cb.emit(clamp_pct(f));
        }
    });
    html! {
        <input class="lg-slider" type="range" min="0" max="100" value={v.to_string()}
               style={format!("--v:{v}%")} disabled={props.disabled} {oninput} />
    }
}

#[derive(Properties, PartialEq)]
pub struct TextInputProps {
    pub value: AttrValue,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub oninput: Callback<String>,
}

/// Single-line text input.
#[function_component]
pub fn TextInput(props: &TextInputProps) -> Html {
    let cb = props.oninput.clone();
    let oninput = Callback::from(move |e: InputEvent| cb.emit(input_value(&e)));
    html! {
        <input class="lg-input" value={props.value.clone()} placeholder={props.placeholder.clone()}
               disabled={props.disabled} {oninput} />
    }
}
