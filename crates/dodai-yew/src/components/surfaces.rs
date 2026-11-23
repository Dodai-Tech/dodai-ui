//! Bars, buttons, overlays, navigation and sheets on the [`Kit`] looks. Controlled: `open`,
//! `selected` and `value` come from the caller, who handles the callbacks. Looks read the
//! stylesheet variables, so a design restyles all of them.

use super::kit::{alert_icon, Icon, KitBox};
use dodai_core::{Kit, Tone};
use dodai_icons::IconId;
use yew::prelude::*;

const SCRIM: &str = "position:fixed;inset:0;z-index:50;background:color-mix(in srgb, var(--dodai-background) 55%, transparent);";

fn keep(e: MouseEvent) {
    e.stop_propagation();
}

#[derive(Properties, PartialEq)]
pub struct FabProps {
    #[prop_or(IconId::Plus)]
    pub icon: IconId,
    /// A label makes it the extended variant.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// Floating action button; with a label, the extended variant.
#[function_component]
pub fn Fab(props: &FabProps) -> Html {
    html! {
        <KitBox kit={Kit::Fab { extended: props.label.is_some() }} pressable=true style="flex:none;font-size:var(--dodai-size-md);" onclick={props.onclick.clone()}>
            <Icon icon={props.icon} size={24} />
            if let Some(l) = &props.label { <span>{ l.clone() }</span> }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct IconButtonProps {
    pub icon: IconId,
    /// The accessible name.
    pub label: AttrValue,
    #[prop_or_default]
    pub filled: bool,
    #[prop_or_default]
    pub disabled: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// A ghost button holding one icon.
#[function_component]
pub fn IconButton(props: &IconButtonProps) -> Html {
    html! {
        <button class="dodai-btn dodai-btn--ghost dodai-btn--md dodai-btn--icon" aria-label={props.label.clone()}
            disabled={props.disabled} onclick={props.onclick.clone()}>
            <Icon icon={props.icon} size={18} filled={props.filled} />
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToolbarProps {
    #[prop_or_default]
    pub floating: bool,
    #[prop_or_default]
    pub vibrant: bool,
    #[prop_or_default]
    pub children: Html,
}

/// A row of tools: docked full width, or floating.
#[function_component]
pub fn Toolbar(props: &ToolbarProps) -> Html {
    let w = if props.floating {
        "fit-content"
    } else {
        "100%"
    };
    html! {
        <KitBox kit={Kit::Toolbar { floating: props.floating, vibrant: props.vibrant }} style={format!("width:{w};gap:var(--dodai-space-xs);flex:none;")}>
            { props.children.clone() }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct KbdProps {
    #[prop_or_default]
    pub children: Html,
}

/// A key cap.
#[function_component]
pub fn Kbd(props: &KbdProps) -> Html {
    html! {
        <kbd class="dodai-kbd" style="display:inline-flex;align-items:center;padding:0 var(--dodai-space-sm);min-height:1.6em;font:var(--dodai-size-xs) var(--dodai-font-mono);color:var(--dodai-text);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border-strong);border-radius:calc(var(--dodai-radius) / 2)">
            { props.children.clone() }
        </kbd>
    }
}

#[derive(Properties, PartialEq)]
pub struct BannerProps {
    pub tone: Tone,
    #[prop_or_default]
    pub action: Option<AttrValue>,
    #[prop_or_default]
    pub onaction: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A full-width message bar with an optional action.
#[function_component]
pub fn Banner(props: &BannerProps) -> Html {
    html! {
        <KitBox kit={Kit::Alert { tone: props.tone, banner: true }} style="width:100%;">
            <Icon icon={alert_icon(props.tone)} size={18} />
            <span style="flex:1">{ props.children.clone() }</span>
            if let Some(a) = &props.action {
                <button class="dodai-btn dodai-btn--ghost dodai-btn--sm" onclick={props.onaction.clone()}>{ a.clone() }</button>
            }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct ToastProps {
    pub open: bool,
    pub text: AttrValue,
    #[prop_or_default]
    pub action: Option<AttrValue>,
    #[prop_or_default]
    pub onaction: Callback<MouseEvent>,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
}

/// A transient message pinned to the bottom of the nearest positioned ancestor. Controlled.
#[function_component]
pub fn Toast(props: &ToastProps) -> Html {
    if !props.open {
        return html! {};
    }
    html! {
        <div role="status" style="position:absolute;left:0;right:0;bottom:var(--dodai-space-lg);display:flex;justify-content:center;pointer-events:none">
            <div style="pointer-events:auto">
                <KitBox kit={Kit::Snackbar} style="gap:var(--dodai-space-lg);width:fit-content;">
                    <span>{ props.text.clone() }</span>
                    if let Some(a) = &props.action {
                        <button class="dodai-btn dodai-btn--ghost" onclick={props.onaction.clone()}>{ a.clone() }</button>
                    }
                    <button class="dodai-btn dodai-btn--ghost dodai-btn--icon" aria-label="Dismiss" onclick={props.ondismiss.clone()}>
                        <Icon icon={IconId::Close} size={16} />
                    </button>
                </KitBox>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogCardProps {
    pub title: AttrValue,
    /// Buttons along the bottom edge.
    #[prop_or_default]
    pub actions: Html,
    #[prop_or_default]
    pub children: Html,
}

/// The dialog's card without the scrim: title, body, action row.
#[function_component]
pub fn DialogCard(props: &DialogCardProps) -> Html {
    html! {
        <KitBox kit={Kit::Popup} style="min-width:280px;max-width:min(92vw,480px);flex-direction:column;align-items:stretch;gap:var(--dodai-space-md);padding:var(--dodai-space-lg);">
            <span class="dodai-label" style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)">{ props.title.clone() }</span>
            <div style="color:var(--dodai-text-muted)">{ props.children.clone() }</div>
            <div style="display:flex;justify-content:flex-end;gap:var(--dodai-space-sm)">{ props.actions.clone() }</div>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct DialogProps {
    pub open: bool,
    pub title: AttrValue,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub actions: Html,
    #[prop_or_default]
    pub children: Html,
}

/// A modal dialog over a scrim; pressing the scrim dismisses. Controlled.
#[function_component]
pub fn Dialog(props: &DialogProps) -> Html {
    if !props.open {
        return html! {};
    }
    html! {
        <div role="presentation" style={format!("{SCRIM}display:grid;place-items:center")} onclick={props.ondismiss.clone()}>
            <div role="dialog" aria-modal="true" aria-label={props.title.clone()} onclick={Callback::from(keep)}>
                <DialogCard title={props.title.clone()} actions={props.actions.clone()}>{ props.children.clone() }</DialogCard>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct PopoverProps {
    pub open: bool,
    /// What the popover hangs from.
    pub anchor: Html,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A panel under its anchor; pressing outside dismisses. Controlled.
#[function_component]
pub fn Popover(props: &PopoverProps) -> Html {
    html! {
        <span style="position:relative;display:inline-block">
            { props.anchor.clone() }
            if props.open {
                <div style="position:fixed;inset:0;z-index:40" onclick={props.ondismiss.clone()}></div>
                <div style="position:absolute;left:0;top:calc(100% + var(--dodai-space-xs));z-index:41">
                    <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;gap:var(--dodai-space-sm);padding:var(--dodai-space-md);">
                        { props.children.clone() }
                    </KitBox>
                </div>
            }
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct AccordionProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub open: bool,
    /// Gets the state the header asks for.
    #[prop_or_default]
    pub ontoggle: Callback<bool>,
    #[prop_or_default]
    pub children: Html,
}

/// A header that shows or hides its body. Controlled.
#[function_component]
pub fn Accordion(props: &AccordionProps) -> Html {
    let open = props.open;
    let chev = if open {
        IconId::ChevronUp
    } else {
        IconId::ChevronDown
    };
    let toggle = props.ontoggle.clone();
    html! {
        <div style="width:100%;border-bottom:var(--dodai-border-width) solid var(--dodai-border)">
            <button class="dodai-kit--press" aria-expanded={open.to_string()}
                style="display:flex;width:100%;align-items:center;justify-content:space-between;padding:var(--dodai-space-md) 0;background:none;border:0;color:var(--dodai-text);font:inherit;cursor:pointer"
                onclick={Callback::from(move |_| toggle.emit(!open))}>
                <span>{ props.title.clone() }</span>
                <Icon icon={chev} size={18} />
            </button>
            if open { <div style="padding-bottom:var(--dodai-space-md);color:var(--dodai-text-muted)">{ props.children.clone() }</div> }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TopAppBarProps {
    pub title: AttrValue,
    #[prop_or_default]
    pub leading: Html,
    #[prop_or_default]
    pub trailing: Html,
}

/// The bar across the top of a screen: leading action, title, trailing actions.
#[function_component]
pub fn TopAppBar(props: &TopAppBarProps) -> Html {
    html! {
        <KitBox kit={Kit::Toolbar { floating: false, vibrant: false }} style="width:100%;gap:var(--dodai-space-sm);flex:none;">
            { props.leading.clone() }
            <span style="flex:1;font-family:var(--dodai-font-display);font-size:var(--dodai-size-md)">{ props.title.clone() }</span>
            { props.trailing.clone() }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct BottomAppBarProps {
    #[prop_or_default]
    pub fab: Html,
    #[prop_or_default]
    pub children: Html,
}

/// The bar across the bottom of a screen: actions on the left, an optional [`Fab`] on the right.
#[function_component]
pub fn BottomAppBar(props: &BottomAppBarProps) -> Html {
    html! {
        <KitBox kit={Kit::Toolbar { floating: false, vibrant: false }} style="width:100%;gap:var(--dodai-space-xs);flex:none;">
            { props.children.clone() }
            <div style="flex:1"></div>
            { props.fab.clone() }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchBarProps {
    pub value: AttrValue,
    #[prop_or(AttrValue::Static("Search"))]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub oninput: Callback<String>,
}

/// A search field with a magnifier and a clear button. Controlled.
#[function_component]
pub fn SearchBar(props: &SearchBarProps) -> Html {
    let on = props.oninput.clone();
    let clear = props.oninput.clone();
    html! {
        <KitBox kit={Kit::Chip { selected: false }} style="width:100%;gap:var(--dodai-space-sm);min-height:var(--dodai-control-height);">
            <Icon icon={IconId::Search} size={18} />
            <input type="search" value={props.value.clone()} placeholder={props.placeholder.clone()}
                style="flex:1;min-width:0;background:none;border:0;outline:0;color:var(--dodai-text);font:inherit"
                oninput={Callback::from(move |e: InputEvent| {
                    let t: web_sys::HtmlInputElement = e.target_unchecked_into();
                    on.emit(t.value());
                })} />
            if !props.value.is_empty() {
                <button aria-label="Clear" style="background:none;border:0;color:inherit;cursor:pointer;display:inline-flex"
                    onclick={Callback::from(move |_| clear.emit(String::new()))}>
                    <Icon icon={IconId::Close} size={16} />
                </button>
            }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavigationItemProps {
    pub icon: IconId,
    pub label: AttrValue,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or(true)]
    pub stacked: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// One destination of a navigation bar, rail or drawer; the selected one takes the row look.
#[function_component]
pub fn NavigationItem(props: &NavigationItemProps) -> Html {
    let dir = if props.stacked { "column" } else { "row" };
    html! {
        <KitBox kit={Kit::Row { selected: props.selected }} pressable=true onclick={props.onclick.clone()}
            style={format!("flex-direction:{dir};align-items:center;justify-content:center;gap:var(--dodai-space-xs);min-width:var(--dodai-control-height);font-size:var(--dodai-size-xs);")}>
            <Icon icon={props.icon} size={22} filled={props.selected} />
            <span>{ props.label.clone() }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct NavigationProps {
    #[prop_or_default]
    pub children: Html,
}

/// Destinations along the bottom edge.
#[function_component]
pub fn NavigationBar(props: &NavigationProps) -> Html {
    html! {
        <nav role="navigation" style="display:flex;justify-content:space-around;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs) var(--dodai-space-sm);background:var(--dodai-surface-raised);border-top:var(--dodai-border-width) solid var(--dodai-border)">
            { props.children.clone() }
        </nav>
    }
}

/// Destinations in a narrow column.
#[function_component]
pub fn NavigationRail(props: &NavigationProps) -> Html {
    html! {
        <nav role="navigation" style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-md) var(--dodai-space-xs);background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)">
            { props.children.clone() }
        </nav>
    }
}

/// Destinations in a wide column.
#[function_component]
pub fn NavigationDrawer(props: &NavigationProps) -> Html {
    html! {
        <nav role="navigation" style="display:flex;flex-direction:column;gap:var(--dodai-space-xs);padding:var(--dodai-space-md);min-width:280px;background:var(--dodai-surface-raised);border-right:var(--dodai-border-width) solid var(--dodai-border)">
            { props.children.clone() }
        </nav>
    }
}

#[derive(Properties, PartialEq)]
pub struct SheetProps {
    pub open: bool,
    pub label: AttrValue,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

fn sheet(p: &SheetProps, align: &str, shape: &str, children: Html) -> Html {
    if !p.open {
        return html! {};
    }
    html! {
        <div role="presentation" style={format!("{SCRIM}display:flex;{align}")} onclick={p.ondismiss.clone()}>
            <div role="dialog" aria-modal="true" aria-label={p.label.clone()} onclick={Callback::from(keep)}
                style={format!("{shape};overflow:auto;padding:var(--dodai-space-md);background:var(--dodai-surface-raised);border:var(--dodai-border-width) solid var(--dodai-border);box-shadow:var(--dodai-shadow, none)")}>
                { children }
            </div>
        </div>
    }
}

/// A drawer sliding over the page from the left.
#[function_component]
pub fn ModalDrawer(props: &SheetProps) -> Html {
    sheet(
        props,
        "justify-content:flex-start",
        "height:100%;max-width:min(360px,90vw)",
        props.children.clone(),
    )
}

/// A sheet over the page from the right.
#[function_component]
pub fn SideSheet(props: &SheetProps) -> Html {
    sheet(
        props,
        "justify-content:flex-end",
        "height:100%;max-width:min(400px,90vw)",
        props.children.clone(),
    )
}

/// A sheet over the page from the bottom, with a grab handle.
#[function_component]
pub fn BottomSheet(props: &SheetProps) -> Html {
    let inner = html! {
        <>
            <div style="width:var(--dodai-space-xl);height:4px;margin:0 auto var(--dodai-space-md);border-radius:9999px;background:var(--dodai-border-strong)"></div>
            { props.children.clone() }
        </>
    };
    sheet(
        props,
        "flex-direction:column;justify-content:flex-end",
        "width:100%;max-height:70vh",
        inner,
    )
}

#[derive(Properties, PartialEq)]
pub struct FullScreenDialogProps {
    pub open: bool,
    pub title: AttrValue,
    #[prop_or_default]
    pub onclose: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A dialog that fills the window, with a close button and title.
#[function_component]
pub fn FullScreenDialog(props: &FullScreenDialogProps) -> Html {
    if !props.open {
        return html! {};
    }
    html! {
        <div role="dialog" aria-modal="true" aria-label={props.title.clone()}
            style="position:fixed;inset:0;z-index:60;display:flex;flex-direction:column;background:var(--dodai-background);color:var(--dodai-text)">
            <div style="display:flex;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-sm) var(--dodai-space-md);border-bottom:var(--dodai-border-width) solid var(--dodai-border)">
                <IconButton icon={IconId::Close} label="Close" onclick={props.onclose.clone()} />
                <span style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-lg)">{ props.title.clone() }</span>
            </div>
            <div style="flex:1;overflow:auto;padding:var(--dodai-space-md)">{ props.children.clone() }</div>
        </div>
    }
}
