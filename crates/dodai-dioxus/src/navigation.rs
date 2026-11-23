//! Navigation and sheets: items, bar, rail, drawers, side and bottom sheets, full-screen dialog.
//! Selected items take the [`Kit::Row`] look; everything reads the stylesheet variables.

use crate::{Icon, KitBox};
use dioxus::prelude::*;
use dodai_core::Kit;
use dodai_icons::IconId;

/// Properties of [`NavigationItem`].
#[derive(Props, Clone, PartialEq)]
pub struct NavigationItemProps {
    pub icon: IconId,
    pub label: String,
    #[props(default)]
    pub selected: bool,
    /// Icon above the label (bar and rail) instead of beside it (drawer).
    #[props(default = true)]
    pub stacked: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// One destination of a bar, rail or drawer.
#[component]
pub fn NavigationItem(props: NavigationItemProps) -> Element {
    let dir = if props.stacked { "column" } else { "row" };
    let on = props.onclick;
    rsx! {
        KitBox {
            kit: Kit::Row { selected: props.selected },
            style: format!("flex-direction:{dir};align-items:center;justify-content:center;gap:var(--dodai-space-xs);min-width:var(--dodai-control-height);font-size:var(--dodai-size-xs)"),
            onclick: move |e| on.call(e),
            Icon { icon: props.icon, size: 22, filled: props.selected }
            span { "{props.label}" }
        }
    }
}

/// Properties shared by the navigation containers.
#[derive(Props, Clone, PartialEq)]
pub struct NavigationProps {
    pub children: Element,
}

/// A bottom bar of [`NavigationItem`]s.
#[component]
pub fn NavigationBar(props: NavigationProps) -> Element {
    rsx! {
        nav { role: "navigation", style: "display:flex;justify-content:space-around;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs) var(--dodai-space-sm);background:var(--dodai-surface-raised);border-top:var(--dodai-border-width) solid var(--dodai-border)",
            {props.children}
        }
    }
}

/// A side rail of [`NavigationItem`]s.
#[component]
pub fn NavigationRail(props: NavigationProps) -> Element {
    rsx! {
        nav { role: "navigation", style: "display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-md) var(--dodai-space-xs);background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)",
            {props.children}
        }
    }
}

/// A full-height list of destinations (set `stacked: false` on its items).
#[component]
pub fn NavigationDrawer(props: NavigationProps) -> Element {
    rsx! {
        nav { role: "navigation", style: "display:flex;flex-direction:column;gap:var(--dodai-space-xs);padding:var(--dodai-space-md);min-width:280px;background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)",
            {props.children}
        }
    }
}

/// Which edge a [`Sheet`] sits on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Edge {
    Left,
    Right,
    Bottom,
}

#[derive(Props, Clone, PartialEq)]
struct SheetProps {
    open: bool,
    edge: Edge,
    label: String,
    ondismiss: EventHandler<MouseEvent>,
    children: Element,
}

#[component]
fn Sheet(props: SheetProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let (align, shape) = match props.edge {
        Edge::Left => (
            "justify-content:flex-start",
            "height:100%;max-width:min(360px,90vw)",
        ),
        Edge::Right => (
            "justify-content:flex-end",
            "height:100%;max-width:min(400px,90vw)",
        ),
        Edge::Bottom => (
            "flex-direction:column;justify-content:flex-end",
            "width:100%;max-height:70vh",
        ),
    };
    let dismiss = props.ondismiss;
    rsx! {
        div {
            role: "presentation",
            style: "position:fixed;inset:0;z-index:50;display:flex;{align};background:color-mix(in srgb, var(--dodai-background) 55%, transparent)",
            onclick: move |e| dismiss.call(e),
            div {
                role: "dialog",
                aria_modal: "true",
                aria_label: "{props.label}",
                style: "{shape};overflow:auto;padding:var(--dodai-space-md);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border);box-shadow:var(--dodai-shadow, none)",
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

/// Properties of the sheet and drawer overlays.
#[derive(Props, Clone, PartialEq)]
pub struct SheetOverlayProps {
    pub open: bool,
    /// The accessible name.
    pub label: String,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A navigation drawer over a scrim, from the left. Controlled.
#[component]
pub fn ModalDrawer(props: SheetOverlayProps) -> Element {
    rsx! { Sheet { open: props.open, edge: Edge::Left, label: props.label, ondismiss: props.ondismiss, {props.children} } }
}

/// A secondary panel from the right edge. Controlled.
#[component]
pub fn SideSheet(props: SheetOverlayProps) -> Element {
    rsx! { Sheet { open: props.open, edge: Edge::Right, label: props.label, ondismiss: props.ondismiss, {props.children} } }
}

/// A sheet rising from the bottom edge, with a drag handle. Controlled.
#[component]
pub fn BottomSheet(props: SheetOverlayProps) -> Element {
    rsx! {
        Sheet { open: props.open, edge: Edge::Bottom, label: props.label, ondismiss: props.ondismiss,
            div { style: "width:var(--dodai-space-xl);height:4px;margin:0 auto var(--dodai-space-md);border-radius:9999px;background:var(--dodai-border-strong)" }
            {props.children}
        }
    }
}

/// Properties of [`FullScreenDialog`].
#[derive(Props, Clone, PartialEq)]
pub struct FullScreenDialogProps {
    pub open: bool,
    pub title: String,
    #[props(default)]
    pub onclose: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A dialog that fills the window, with a close button and a title. Controlled.
#[component]
pub fn FullScreenDialog(props: FullScreenDialogProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let close = props.onclose;
    rsx! {
        div { role: "dialog", aria_modal: "true", aria_label: "{props.title}",
            style: "position:fixed;inset:0;z-index:60;display:flex;flex-direction:column;background:var(--dodai-background);color:var(--dodai-text)",
            div { style: "display:flex;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-sm) var(--dodai-space-md);border-bottom:var(--dodai-border-width) solid var(--dodai-border)",
                crate::IconButton { icon: IconId::Close, label: "Close", onclick: move |e| close.call(e) }
                span { style: "font-family:var(--dodai-font-display);font-size:var(--dodai-size-lg)", "{props.title}" }
            }
            div { style: "flex:1;overflow:auto;padding:var(--dodai-space-md)", {props.children} }
        }
    }
}
