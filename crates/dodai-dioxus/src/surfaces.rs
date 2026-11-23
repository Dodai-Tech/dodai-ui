//! Bars, buttons and overlays built on the [`Kit`] looks: FAB, icon button, toolbar, key cap,
//! banner, toast, dialog, popover, accordion, app bars and search bar. Each takes its looks from
//! the design through the stylesheet variables, so a design restyles all of them.

use crate::{Icon, KitBox};
use dioxus::prelude::*;
use dodai_core::{Kit, Tone};
use dodai_icons::IconId;

/// Properties of [`Fab`].
#[derive(Props, Clone, PartialEq)]
pub struct FabProps {
    #[props(default = IconId::Plus)]
    pub icon: IconId,
    /// A label makes it the extended variant.
    #[props(default)]
    pub label: Option<String>,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// Floating action button; with a label, the extended variant.
#[component]
pub fn Fab(props: FabProps) -> Element {
    let kit = Kit::Fab {
        extended: props.label.is_some(),
    };
    let on = props.onclick;
    rsx! {
        KitBox { kit, style: "flex:none;font-size:var(--dodai-size-md)".to_string(), onclick: move |e| on.call(e),
            Icon { icon: props.icon, size: 24 }
            if let Some(l) = props.label { span { "{l}" } }
        }
    }
}

/// Properties of [`IconButton`].
#[derive(Props, Clone, PartialEq)]
pub struct IconButtonProps {
    pub icon: IconId,
    /// The accessible name.
    pub label: String,
    #[props(default)]
    pub filled: bool,
    #[props(default)]
    pub disabled: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// A ghost button holding one icon.
#[component]
pub fn IconButton(props: IconButtonProps) -> Element {
    rsx! {
        crate::Button {
            variant: dodai_core::ButtonVariant::Ghost,
            icon_only: true,
            disabled: props.disabled,
            onclick: props.onclick,
            span { aria_label: "{props.label}", Icon { icon: props.icon, size: 18, filled: props.filled } }
        }
    }
}

/// Properties of [`Toolbar`].
#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    #[props(default)]
    pub floating: bool,
    #[props(default)]
    pub vibrant: bool,
    pub children: Element,
}

/// A row of tools: docked full width, or floating.
#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let kit = Kit::Toolbar {
        floating: props.floating,
        vibrant: props.vibrant,
    };
    let width = if props.floating {
        "fit-content"
    } else {
        "100%"
    };
    rsx! {
        KitBox { kit, style: format!("width:{width};gap:var(--dodai-space-xs);flex:none"), {props.children} }
    }
}

/// A key cap.
#[component]
pub fn Kbd(children: Element) -> Element {
    rsx! {
        kbd {
            class: "dodai-kbd",
            style: "display:inline-flex;align-items:center;padding:0 var(--dodai-space-sm);min-height:1.6em;font:var(--dodai-size-xs) var(--dodai-font-mono);color:var(--dodai-text);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border-strong);border-radius:calc(var(--dodai-radius) / 2)",
            {children}
        }
    }
}

/// Properties of [`Banner`].
#[derive(Props, Clone, PartialEq)]
pub struct BannerProps {
    pub tone: Tone,
    /// An optional action button.
    #[props(default)]
    pub action: Option<String>,
    #[props(default)]
    pub onaction: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A full-width message bar with an optional action.
#[component]
pub fn Banner(props: BannerProps) -> Element {
    rsx! {
        crate::Alert { tone: props.tone, banner: true,
            {props.children}
            if let Some(a) = props.action {
                crate::Button { variant: dodai_core::ButtonVariant::Ghost, size: Some(dodai_core::ControlSize::Small), onclick: props.onaction, "{a}" }
            }
        }
    }
}

/// Properties of [`Toast`].
#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    pub open: bool,
    pub text: String,
    #[props(default)]
    pub action: Option<String>,
    #[props(default)]
    pub onaction: EventHandler<MouseEvent>,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
}

/// A transient message pinned to the bottom of the nearest positioned ancestor. Controlled:
/// the owner decides when it is `open` and hides it from `ondismiss` or a timer.
#[component]
pub fn Toast(props: ToastProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    rsx! {
        div { role: "status", style: "position:absolute;left:0;right:0;bottom:var(--dodai-space-lg);display:flex;justify-content:center;pointer-events:none",
            div { style: "pointer-events:auto",
                KitBox { kit: Kit::Snackbar, style: "gap:var(--dodai-space-lg);width:fit-content;".to_string(),
                    span { "{props.text}" }
                    if let Some(a) = props.action {
                        crate::Button { variant: dodai_core::ButtonVariant::Ghost, onclick: props.onaction, "{a}" }
                    }
                    crate::Button { variant: dodai_core::ButtonVariant::Ghost, icon_only: true, onclick: props.ondismiss,
                        Icon { icon: IconId::Close, size: 16 }
                    }
                }
            }
        }
    }
}

/// Properties of [`Dialog`].
#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    pub title: String,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
    /// Buttons along the bottom edge.
    #[props(default = rsx! {})]
    pub actions: Element,
    pub children: Element,
}

/// A modal dialog over a scrim; pressing the scrim dismisses. Controlled.
#[component]
pub fn Dialog(props: DialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let dismiss = props.ondismiss;
    rsx! {
        div {
            role: "presentation",
            style: "position:fixed;inset:0;z-index:50;display:grid;place-items:center;background:color-mix(in srgb, var(--dodai-background) 55%, transparent)",
            onclick: move |e| dismiss.call(e),
            div { role: "dialog", aria_modal: "true", aria_label: "{props.title}", onclick: move |e| e.stop_propagation(),
                DialogCard { title: props.title.clone(), actions: props.actions, {props.children} }
            }
        }
    }
}

/// Properties of [`DialogCard`].
#[derive(Props, Clone, PartialEq)]
pub struct DialogCardProps {
    pub title: String,
    #[props(default = rsx! {})]
    pub actions: Element,
    pub children: Element,
}

/// The dialog's card without the scrim: title, body, action row.
#[component]
pub fn DialogCard(props: DialogCardProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Popup, style: "min-width:280px;max-width:min(92vw,480px);flex-direction:column;align-items:stretch;gap:var(--dodai-space-md);padding:var(--dodai-space-lg)".to_string(),
            span { class: "dodai-label", style: "font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)", "{props.title}" }
            div { style: "color:var(--dodai-text-muted)", {props.children} }
            div { style: "display:flex;justify-content:flex-end;gap:var(--dodai-space-sm)", {props.actions} }
        }
    }
}

/// Properties of [`Popover`].
#[derive(Props, Clone, PartialEq)]
pub struct PopoverProps {
    pub open: bool,
    /// What the popover hangs from.
    pub anchor: Element,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A panel under its anchor; pressing outside dismisses. Controlled.
#[component]
pub fn Popover(props: PopoverProps) -> Element {
    let dismiss = props.ondismiss;
    rsx! {
        span { style: "position:relative;display:inline-block",
            {props.anchor}
            if props.open {
                div { style: "position:fixed;inset:0;z-index:40", onclick: move |e| dismiss.call(e) }
                div { style: "position:absolute;left:0;top:calc(100% + var(--dodai-space-xs));z-index:41",
                    KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch;gap:var(--dodai-space-sm);padding:var(--dodai-space-md)".to_string(),
                        {props.children}
                    }
                }
            }
        }
    }
}

/// Properties of [`Accordion`].
#[derive(Props, Clone, PartialEq)]
pub struct AccordionProps {
    pub title: String,
    #[props(default)]
    pub open: bool,
    /// Gets the state the header asks for.
    #[props(default)]
    pub ontoggle: EventHandler<bool>,
    pub children: Element,
}

/// A header that shows or hides its body. Controlled.
#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    let (open, toggle) = (props.open, props.ontoggle);
    let chev = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    rsx! {
        div { style: "width:100%;border-bottom:var(--dodai-border-width) solid var(--dodai-border)",
            button {
                class: "dodai-kit--press",
                aria_expanded: "{open}",
                style: "display:flex;width:100%;align-items:center;justify-content:space-between;padding:var(--dodai-space-md) 0;background:none;border:0;color:var(--dodai-text);font:inherit;cursor:pointer",
                onclick: move |_| toggle.call(!open),
                span { "{props.title}" }
                Icon { icon: chev, size: 18 }
            }
            if open {
                div { style: "padding-bottom:var(--dodai-space-md);color:var(--dodai-text-muted)", {props.children} }
            }
        }
    }
}

/// Properties of [`TopAppBar`].
#[derive(Props, Clone, PartialEq)]
pub struct TopAppBarProps {
    pub title: String,
    #[props(default = rsx! {})]
    pub leading: Element,
    #[props(default = rsx! {})]
    pub trailing: Element,
}

/// The bar across the top of a screen: leading action, title, trailing actions.
#[component]
pub fn TopAppBar(props: TopAppBarProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Toolbar { floating: false, vibrant: false }, style: "width:100%;gap:var(--dodai-space-sm);flex:none".to_string(),
            {props.leading}
            span { style: "flex:1;font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)", "{props.title}" }
            {props.trailing}
        }
    }
}

/// The bar across the bottom of a screen: actions on the left, an optional [`Fab`] on the right.
#[component]
pub fn BottomAppBar(#[props(default = rsx! {})] fab: Element, children: Element) -> Element {
    rsx! {
        KitBox { kit: Kit::Toolbar { floating: false, vibrant: false }, style: "width:100%;gap:var(--dodai-space-xs);flex:none".to_string(),
            {children}
            div { style: "flex:1" }
            {fab}
        }
    }
}

/// Properties of [`SearchBar`].
#[derive(Props, Clone, PartialEq)]
pub struct SearchBarProps {
    pub value: String,
    #[props(default = "Search".to_string())]
    pub placeholder: String,
    #[props(default)]
    pub oninput: EventHandler<String>,
}

/// A search field with a magnifier and a clear button. Controlled.
#[component]
pub fn SearchBar(props: SearchBarProps) -> Element {
    let on = props.oninput;
    let has = !props.value.is_empty();
    rsx! {
        KitBox { kit: Kit::Chip { selected: false }, style: "width:100%;gap:var(--dodai-space-sm);min-height:var(--dodai-control-height)".to_string(),
            Icon { icon: IconId::Search, size: 18 }
            input {
                r#type: "search",
                value: "{props.value}",
                placeholder: "{props.placeholder}",
                style: "flex:1;min-width:0;background:none;border:0;outline:0;color:var(--dodai-text);font:inherit",
                oninput: move |e| on.call(e.value()),
            }
            if has {
                button { aria_label: "Clear", style: "background:none;border:0;color:inherit;cursor:pointer;display:inline-flex", onclick: move |_| on.call(String::new()),
                    Icon { icon: IconId::Close, size: 16 }
                }
            }
        }
    }
}
