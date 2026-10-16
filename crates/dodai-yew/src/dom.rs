//! Tiny helpers reading values off input events (browser only at runtime).
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub fn input_value(e: &InputEvent) -> String {
    e.target_unchecked_into::<HtmlInputElement>().value()
}

pub fn input_checked(e: &Event) -> bool {
    e.target_unchecked_into::<HtmlInputElement>().checked()
}
