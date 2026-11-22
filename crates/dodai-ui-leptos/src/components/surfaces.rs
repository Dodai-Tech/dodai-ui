//! Bars, buttons, overlays, navigation and sheets on the [`Kit`] looks. Leptos here renders on
//! the server, so these are state-in components: `open`, `selected` and `value` come from the
//! caller and the `on_*` callbacks are for a client build. Looks read the stylesheet variables.

use super::kit::{alert_icon, Icon, KitBox};
use dodai_ui_core::{Kit, Tone};
use dodai_ui_icons::IconId;
use leptos::prelude::*;

const SCRIM: &str = "position:fixed;inset:0;z-index:50;background:color-mix(in srgb, var(--dodai-background) 55%, transparent);";

fn click(cb: Option<Callback<()>>) {
    if let Some(cb) = cb {
        cb.run(());
    }
}

/// Floating action button; with a label, the extended variant.
#[component]
pub fn Fab(
    #[prop(default = IconId::Plus)] icon: IconId,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let kit = Kit::Fab {
        extended: label.is_some(),
    };
    view! {
        <KitBox kit=kit pressable=true style="flex:none;font-size:var(--dodai-size-md);">
            <span on:click=move |_| click(on_click) style="display:contents">
                <Icon icon=icon size=24 />
                {label.map(|l| view! { <span>{l}</span> })}
            </span>
        </KitBox>
    }
}

/// A ghost button holding one icon.
#[component]
pub fn IconButton(
    icon: IconId,
    #[prop(into)] label: String,
    #[prop(optional)] filled: bool,
    #[prop(optional)] disabled: bool,
    #[prop(optional)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <button class="dodai-btn dodai-btn--ghost dodai-btn--md dodai-btn--icon" aria-label=label disabled=disabled
            on:click=move |_| click(on_click)>
            <Icon icon=icon size=18 filled=filled />
        </button>
    }
}

/// A row of tools: docked full width, or floating.
#[component]
pub fn Toolbar(
    #[prop(optional)] floating: bool,
    #[prop(optional)] vibrant: bool,
    children: Children,
) -> impl IntoView {
    let w = if floating { "fit-content" } else { "100%" };
    view! {
        <KitBox kit=Kit::Toolbar { floating, vibrant } style=format!("width:{w};gap:var(--dodai-space-xs);flex:none;")>
            {children()}
        </KitBox>
    }
}

/// A key cap.
#[component]
pub fn Kbd(children: Children) -> impl IntoView {
    view! {
        <kbd class="dodai-kbd" style="display:inline-flex;align-items:center;padding:0 var(--dodai-space-sm);min-height:1.6em;font:var(--dodai-size-xs) var(--dodai-font-mono);color:var(--dodai-text);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border-strong);border-radius:calc(var(--dodai-radius) / 2)">
            {children()}
        </kbd>
    }
}

/// A full-width message bar with an optional action.
#[component]
pub fn Banner(
    tone: Tone,
    #[prop(optional, into)] action: Option<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Alert { tone, banner: true } style="width:100%;">
            <Icon icon=alert_icon(tone) size=18 />
            <span style="flex:1">{children()}</span>
            {action.map(|a| view! { <button class="dodai-btn dodai-btn--ghost dodai-btn--sm">{a}</button> })}
        </KitBox>
    }
}

/// A transient message pinned to the bottom of the nearest positioned ancestor.
#[component]
pub fn Toast(
    open: bool,
    #[prop(into)] text: String,
    #[prop(optional, into)] action: Option<String>,
) -> impl IntoView {
    open.then(|| {
        view! {
            <div role="status" style="position:absolute;left:0;right:0;bottom:var(--dodai-space-lg);display:flex;justify-content:center;pointer-events:none">
                <div style="pointer-events:auto">
                    <KitBox kit=Kit::Snackbar style="gap:var(--dodai-space-lg);width:fit-content;">
                        <span>{text}</span>
                        {action.map(|a| view! { <button class="dodai-btn dodai-btn--ghost">{a}</button> })}
                        <button class="dodai-btn dodai-btn--ghost dodai-btn--icon" aria-label="Dismiss"><Icon icon=IconId::Close size=16 /></button>
                    </KitBox>
                </div>
            </div>
        }
    })
}

/// The dialog's card without the scrim: title, body, action row.
#[component]
pub fn DialogCard(
    #[prop(into)] title: String,
    #[prop(optional, into)] actions: ViewFn,
    children: Children,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Popup style="min-width:280px;max-width:min(92vw,480px);flex-direction:column;align-items:stretch;gap:var(--dodai-space-md);padding:var(--dodai-space-lg);">
            <span class="dodai-label" style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)">{title}</span>
            <div style="color:var(--dodai-text-muted)">{children()}</div>
            <div style="display:flex;justify-content:flex-end;gap:var(--dodai-space-sm)">{actions.run()}</div>
        </KitBox>
    }
}

/// A modal dialog over a scrim.
#[component]
pub fn Dialog(
    open: bool,
    #[prop(into)] title: String,
    #[prop(optional, into)] actions: ViewFn,
    children: Children,
) -> impl IntoView {
    let label = title.clone();
    open.then(|| {
        view! {
            <div role="presentation" style=format!("{SCRIM}display:grid;place-items:center")>
                <div role="dialog" aria-modal="true" aria-label=label>
                    <DialogCard title=title actions=actions>{children()}</DialogCard>
                </div>
            </div>
        }
    })
}

/// A panel under its anchor.
#[component]
pub fn Popover(open: bool, #[prop(into)] anchor: ViewFn, children: Children) -> impl IntoView {
    let body = open.then(|| {
        view! {
            <div style="position:absolute;left:0;top:calc(100% + var(--dodai-space-xs));z-index:41">
                <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;gap:var(--dodai-space-sm);padding:var(--dodai-space-md);">
                    {children()}
                </KitBox>
            </div>
        }
    });
    view! { <span style="position:relative;display:inline-block">{anchor.run()}{body}</span> }
}

/// A header that shows or hides its body.
#[component]
pub fn Accordion(
    #[prop(into)] title: String,
    #[prop(optional)] open: bool,
    children: Children,
) -> impl IntoView {
    let chev = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    view! {
        <div style="width:100%;border-bottom:var(--dodai-border-width) solid var(--dodai-border)">
            <button class="dodai-kit--press" aria-expanded=open.to_string()
                style="display:flex;width:100%;align-items:center;justify-content:space-between;padding:var(--dodai-space-md) 0;background:none;border:0;color:var(--dodai-text);font:inherit;cursor:pointer">
                <span>{title}</span>
                <Icon icon=chev size=18 />
            </button>
            {open.then(|| view! { <div style="padding-bottom:var(--dodai-space-md);color:var(--dodai-text-muted)">{children()}</div> })}
        </div>
    }
}

/// The bar across the top of a screen: leading action, title, trailing actions.
#[component]
pub fn TopAppBar(
    #[prop(into)] title: String,
    #[prop(optional, into)] leading: ViewFn,
    #[prop(optional, into)] trailing: ViewFn,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Toolbar { floating: false, vibrant: false } style="width:100%;gap:var(--dodai-space-sm);flex:none;">
            {leading.run()}
            <span style="flex:1;font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)">{title}</span>
            {trailing.run()}
        </KitBox>
    }
}

/// The bar across the bottom of a screen: actions on the left, an optional [`Fab`] on the right.
#[component]
pub fn BottomAppBar(#[prop(optional, into)] fab: ViewFn, children: Children) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Toolbar { floating: false, vibrant: false } style="width:100%;gap:var(--dodai-space-xs);flex:none;">
            {children()}
            <div style="flex:1"></div>
            {fab.run()}
        </KitBox>
    }
}

/// A search field with a magnifier and a clear button.
#[component]
pub fn SearchBar(
    #[prop(into)] value: String,
    #[prop(default = "Search".to_string(), into)] placeholder: String,
) -> impl IntoView {
    let has = !value.is_empty();
    view! {
        <KitBox kit=Kit::Chip { selected: false } style="width:100%;gap:var(--dodai-space-sm);min-height:var(--dodai-control-height);">
            <Icon icon=IconId::Search size=18 />
            <input type="search" value=value placeholder=placeholder
                style="flex:1;min-width:0;background:none;border:0;outline:0;color:var(--dodai-text);font:inherit" />
            {has.then(|| view! {
                <button aria-label="Clear" style="background:none;border:0;color:inherit;cursor:pointer;display:inline-flex"><Icon icon=IconId::Close size=16 /></button>
            })}
        </KitBox>
    }
}

/// One destination of a navigation bar, rail or drawer; the selected one takes the row look.
#[component]
pub fn NavigationItem(
    icon: IconId,
    #[prop(into)] label: String,
    #[prop(optional)] selected: bool,
    #[prop(default = true)] stacked: bool,
) -> impl IntoView {
    let dir = if stacked { "column" } else { "row" };
    view! {
        <KitBox kit=Kit::Row { selected } pressable=true
            style=format!("flex-direction:{dir};align-items:center;justify-content:center;gap:var(--dodai-space-xs);min-width:var(--dodai-control-height);font-size:var(--dodai-size-xs);")>
            <Icon icon=icon size=22 filled=selected />
            <span>{label}</span>
        </KitBox>
    }
}

/// Destinations along the bottom edge.
#[component]
pub fn NavigationBar(children: Children) -> impl IntoView {
    view! {
        <nav role="navigation" style="display:flex;justify-content:space-around;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs) var(--dodai-space-sm);background:var(--dodai-surface-raised);border-top:var(--dodai-border-width) solid var(--dodai-border)">
            {children()}
        </nav>
    }
}

/// Destinations in a narrow column.
#[component]
pub fn NavigationRail(children: Children) -> impl IntoView {
    view! {
        <nav role="navigation" style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-md) var(--dodai-space-xs);background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)">
            {children()}
        </nav>
    }
}

/// Destinations in a wide column.
#[component]
pub fn NavigationDrawer(children: Children) -> impl IntoView {
    view! {
        <nav role="navigation" style="display:flex;flex-direction:column;gap:var(--dodai-space-xs);padding:var(--dodai-space-md);min-width:280px;background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)">
            {children()}
        </nav>
    }
}

fn sheet(
    open: bool,
    label: String,
    align: &'static str,
    shape: &'static str,
    inner: impl IntoView + 'static,
) -> impl IntoView {
    open.then(|| {
        view! {
            <div role="presentation" style=format!("{SCRIM}display:flex;{align}")>
                <div role="dialog" aria-modal="true" aria-label=label
                    style=format!("{shape};overflow:auto;padding:var(--dodai-space-md);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border);box-shadow:var(--dodai-shadow, none)")>
                    {inner}
                </div>
            </div>
        }
    })
}

/// A drawer sliding over the page from the left.
#[component]
pub fn ModalDrawer(open: bool, #[prop(into)] label: String, children: Children) -> impl IntoView {
    sheet(
        open,
        label,
        "justify-content:flex-start",
        "height:100%;max-width:min(360px,90vw)",
        children(),
    )
}

/// A sheet over the page from the right.
#[component]
pub fn SideSheet(open: bool, #[prop(into)] label: String, children: Children) -> impl IntoView {
    sheet(
        open,
        label,
        "justify-content:flex-end",
        "height:100%;max-width:min(400px,90vw)",
        children(),
    )
}

/// A sheet over the page from the bottom, with a grab handle.
#[component]
pub fn BottomSheet(open: bool, #[prop(into)] label: String, children: Children) -> impl IntoView {
    let inner = view! {
        <div style="width:var(--dodai-space-xl);height:4px;margin:0 auto var(--dodai-space-md);border-radius:9999px;background:var(--dodai-border-strong)"></div>
        {children()}
    };
    sheet(
        open,
        label,
        "flex-direction:column;justify-content:flex-end",
        "width:100%;max-height:70vh",
        inner,
    )
}

/// A dialog that fills the window, with a close button and title.
#[component]
pub fn FullScreenDialog(
    open: bool,
    #[prop(into)] title: String,
    children: Children,
) -> impl IntoView {
    let label = title.clone();
    open.then(|| {
        view! {
            <div role="dialog" aria-modal="true" aria-label=label
                style="position:fixed;inset:0;z-index:60;display:flex;flex-direction:column;background:var(--dodai-background);color:var(--dodai-text)">
                <div style="display:flex;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-sm) var(--dodai-space-md);border-bottom:var(--dodai-border-width) solid var(--dodai-border)">
                    <IconButton icon=IconId::Close label="Close" />
                    <span style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-lg)">{title}</span>
                </div>
                <div style="flex:1;overflow:auto;padding:var(--dodai-space-md)">{children()}</div>
            </div>
        }
    })
}
