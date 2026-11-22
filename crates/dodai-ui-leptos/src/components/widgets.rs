//! Small building blocks: progress and loading, form fields, steppers, rows, tooltips and menus.
//! Built on the [`Kit`] looks and the stylesheet variables, so a design restyles all of them.
//! State comes in as props; the pages are rendered on the server.

use super::kit::{stepped, Icon, KitBox};
use super::surfaces::{IconButton, Kbd};
use super::{Button, DataTable};
use dodai_ui_core::{Kit, Tone};
use dodai_ui_icons::IconId;
use leptos::prelude::*;

fn tone_colour(tone: Option<Tone>) -> &'static str {
    match tone {
        Some(Tone::Success) => "var(--dodai-success)",
        Some(Tone::Warning) => "var(--dodai-warning)",
        Some(Tone::Danger) => "var(--dodai-danger)",
        Some(Tone::Info) => "var(--dodai-info)",
        None => "var(--dodai-primary)",
    }
}

/// A bar for `value` 0–1 with an optional label and percentage above it.
#[component]
pub fn ProgressBar(
    value: f32,
    #[prop(optional)] tone: Option<Tone>,
    #[prop(optional, into)] label: Option<String>,
) -> impl IntoView {
    let pct = (value.clamp(0.0, 1.0) * 100.0).round();
    view! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            {label.map(|l| view! {
                <div style="display:flex;justify-content:space-between;font-size:var(--dodai-size-sm);color:var(--dodai-text-muted)">
                    <span>{l}</span><span>{format!("{pct}%")}</span>
                </div>
            })}
            <div class=crate::class::progress(tone)><span style=format!("--v:{}", crate::class::percent(pct))></span></div>
        </div>
    }
}

/// A bar that runs while the length of the work is unknown.
#[component]
pub fn IndeterminateBar() -> impl IntoView {
    view! { <div class="dodai-indeterminate" role="progressbar"><span></span></div> }
}

/// A ring: filled to `value` (0–1), or spinning without one.
#[component]
pub fn CircularProgress(
    #[prop(optional)] value: Option<f32>,
    #[prop(default = 40)] size: u32,
) -> impl IntoView {
    match value {
        None => view! { <span class="dodai-spinner" role="progressbar" style=format!("--s:{size}px")></span> }.into_any(),
        Some(v) => {
            let pct = v.clamp(0.0, 1.0) * 100.0;
            view! {
                <span role="progressbar" aria-valuenow=pct.round().to_string()
                    style=format!("display:inline-block;flex:none;width:{size}px;height:{size}px;border-radius:50%;background:conic-gradient(var(--dodai-primary) {pct}%, var(--dodai-surface-raised) 0);-webkit-mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px));mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px))")></span>
            }.into_any()
        }
    }
}

/// The design's busy spinner at `size`.
#[component]
pub fn LoadingIndicator(#[prop(default = 24)] size: u32) -> impl IntoView {
    view! { <super::Spinner size=size /> }
}

/// Content with the skeleton shimmer running over it.
#[component]
pub fn Shimmer(children: Children) -> impl IntoView {
    view! { <div class="dodai-skeleton" style="height:auto;display:block">{children()}</div> }
}

/// An icon in a tinted circle.
#[component]
pub fn IconBadge(icon: IconId, #[prop(optional)] tone: Option<Tone>) -> impl IntoView {
    let c = tone_colour(tone);
    view! {
        <span style=format!("display:inline-grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border-radius:50%;color:{c};background:color-mix(in srgb, {c} 16%, transparent)")>
            <Icon icon=icon size=20 />
        </span>
    }
}

/// Body text in the design's font.
#[component]
pub fn Text(children: Children) -> impl IntoView {
    view! { <span style="font-family:var(--dodai-font-body);font-size:var(--dodai-size-md);color:var(--dodai-text)">{children()}</span> }
}

/// A vertical stack with a spacing token (`xs`, `sm`, `md`, `lg`, `xl`).
#[component]
pub fn Column(
    #[prop(default = "md".to_string(), into)] gap: String,
    children: Children,
) -> impl IntoView {
    view! { <div style=format!("display:flex;flex-direction:column;gap:var(--dodai-space-{gap})")>{children()}</div> }
}

/// An image with the design's corner radius.
#[component]
pub fn Image(#[prop(into)] src: String, #[prop(into)] alt: String) -> impl IntoView {
    view! { <img src=src alt=alt style="display:block;max-width:100%;border-radius:var(--dodai-radius)" /> }
}

/// A label above a control, with a hint or an error (the error wins) below.
#[component]
pub fn FormField(
    #[prop(into)] label: String,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional, into)] error: Option<String>,
    children: Children,
) -> impl IntoView {
    let colour = if error.is_some() {
        "var(--dodai-danger)"
    } else {
        "var(--dodai-text-muted)"
    };
    let note = error.or(hint);
    view! {
        <label style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            <span style="font-size:var(--dodai-size-sm);color:var(--dodai-text)">{label}</span>
            {children()}
            {note.map(|n| view! { <span style=format!("font-size:var(--dodai-size-xs);color:{colour}")>{n}</span> })}
        </label>
    }
}

/// A text input with an optional leading icon.
#[component]
pub fn InputBox(
    #[prop(into)] value: String,
    #[prop(optional, into)] placeholder: String,
    #[prop(optional)] leading: Option<IconId>,
) -> impl IntoView {
    let pad = if leading.is_some() {
        "padding-left:calc(var(--dodai-space-sm) * 3)"
    } else {
        ""
    };
    view! {
        <div style="position:relative;display:flex;align-items:center">
            {leading.map(|i| view! {
                <span style="position:absolute;left:var(--dodai-space-sm);display:flex;color:var(--dodai-text-muted)"><Icon icon=i size=18 /></span>
            })}
            <input class="dodai-input" style=pad value=value placeholder=placeholder />
        </div>
    }
}

/// A value with minus and plus buttons that stop at `min` and `max`.
#[component]
pub fn NumberStepper(
    value: i32,
    #[prop(default = i32::MIN)] min: i32,
    #[prop(default = i32::MAX)] max: i32,
    #[prop(default = 1)] step: i32,
    #[prop(optional)] on_change: Option<Callback<i32>>,
) -> impl IntoView {
    let go = move |by: i32| {
        if let Some(cb) = on_change {
            cb.run(stepped(value, by, min, max));
        }
    };
    view! {
        <div style="display:inline-flex;align-items:center;gap:var(--dodai-space-sm)">
            <IconButton icon=IconId::Minus label="Decrease" disabled={value <= min} on_click=Callback::new(move |()| go(-step)) />
            <span style="min-width:3ch;text-align:center;font-family:var(--dodai-font-mono)">{value}</span>
            <IconButton icon=IconId::Plus label="Increase" disabled={value >= max} on_click=Callback::new(move |()| go(step)) />
        </div>
    }
}

/// A primary action with a chevron button for related actions.
#[component]
pub fn SplitButton(#[prop(into)] label: String) -> impl IntoView {
    view! {
        <div style="display:inline-flex;gap:2px">
            <Button>{label}</Button>
            <button class="dodai-btn dodai-btn--primary dodai-btn--md dodai-btn--icon" aria-label="More"><Icon icon=IconId::ChevronDown size=18 /></button>
        </div>
    }
}

/// A one-time code in `len` boxes; the next empty one is outlined.
#[component]
pub fn OtpInput(#[prop(into)] value: String, #[prop(default = 6)] len: usize) -> impl IntoView {
    let digits: Vec<char> = value.chars().collect();
    let boxes = (0..len)
        .map(|i| {
            let edge = if i == digits.len() { "var(--dodai-primary)" } else { "var(--dodai-border)" };
            let c = digits.get(i).map(char::to_string).unwrap_or_default();
            view! {
                <span style=format!("display:grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border:var(--dodai-border-width) solid {edge};border-radius:var(--dodai-radius);background:var(--dodai-surface-raised);font-family:var(--dodai-font-mono);font-size:var(--dodai-size-lg)")>{c}</span>
            }
        })
        .collect_view();
    view! {
        <label style="position:relative;display:inline-flex;gap:var(--dodai-space-xs)">
            {boxes}
            <input inputmode="numeric" maxlength=len.to_string() value=value aria-label="One-time code"
                style="position:absolute;inset:0;opacity:0;cursor:text" />
        </label>
    }
}

/// A dashed area that takes files by drop or click.
#[component]
pub fn FileDropZone(
    #[prop(default = "Drop files here or click to choose".to_string(), into)] label: String,
) -> impl IntoView {
    view! {
        <label style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-lg);border:2px dashed var(--dodai-border-strong);border-radius:var(--dodai-radius);color:var(--dodai-text-muted);cursor:pointer">
            <Icon icon=IconId::Upload size=28 />
            <span>{label}</span>
            <input type="file" multiple style="display:none" />
        </label>
    }
}

/// A table with a head and banded rows.
#[component]
pub fn Table(columns: Vec<String>, rows: Vec<Vec<String>>) -> impl IntoView {
    view! { <DataTable columns=columns rows=rows /> }
}

/// One row of a tree, indented by `depth`; `open` is `None` for a leaf.
#[component]
pub fn TreeRow(
    #[prop(into)] label: String,
    #[prop(optional)] depth: u32,
    #[prop(optional)] open: Option<bool>,
    #[prop(optional)] selected: bool,
) -> impl IntoView {
    let chevron = open.map(|o| {
        if o {
            IconId::ChevronDown
        } else {
            IconId::ChevronRight
        }
    });
    view! {
        <KitBox kit=Kit::Row { selected } pressable=true
            style=format!("padding-left:calc(var(--dodai-space-sm) + {depth} * var(--dodai-space-md));gap:var(--dodai-space-xs);width:100%;")>
            <span style="display:inline-flex;width:18px">{chevron.map(|c| view! { <Icon icon=c size=16 /> })}</span>
            <span>{label}</span>
        </KitBox>
    }
}

/// One event on a timeline: a dot on a line, a title, a time and optional detail.
#[component]
pub fn TimelineEvent(
    #[prop(into)] title: String,
    #[prop(into)] time: String,
    #[prop(optional)] tone: Option<Tone>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div style="display:flex;gap:var(--dodai-space-sm)">
            <div style="display:flex;flex-direction:column;align-items:center">
                <KitBox kit=Kit::Dot(tone) style="width:10px;height:10px;flex:none;"><span></span></KitBox>
                <div style="flex:1;width:2px;background:var(--dodai-border)"></div>
            </div>
            <div style="display:flex;flex-direction:column;gap:2px;padding-bottom:var(--dodai-space-md)">
                <span style="color:var(--dodai-text)">{title}</span>
                <span style="font-size:var(--dodai-size-xs);color:var(--dodai-text-muted)">{time}</span>
                {children.map(|c| c())}
            </div>
        </div>
    }
}

/// A menu row: icon, label, shortcut.
#[component]
pub fn MenuItem(
    #[prop(into)] label: String,
    #[prop(optional)] icon: Option<IconId>,
    #[prop(optional, into)] shortcut: Option<String>,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Row { selected: false } pressable=true style="gap:var(--dodai-space-sm);width:100%;justify-content:flex-start;">
            {icon.map(|i| view! { <Icon icon=i size=18 /> })}
            <span style="flex:1;text-align:left">{label}</span>
            {shortcut.filter(|s| !s.is_empty()).map(|s| view! { <Kbd>{s}</Kbd> })}
        </KitBox>
    }
}

/// A trigger with a menu hanging under it while `open`.
#[component]
pub fn DropdownMenu(
    open: bool,
    #[prop(into)] trigger: ViewFn,
    children: Children,
) -> impl IntoView {
    let menu = open.then(|| view! {
        <div role="menu" style="position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)">
            <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;min-width:200px;">{children()}</KitBox>
        </div>
    });
    view! { <div style="position:relative;display:inline-block">{trigger.run()}{menu}</div> }
}

/// A menu at a point of the window (a right-click).
#[component]
pub fn ContextMenu(open: bool, x: f64, y: f64, children: Children) -> impl IntoView {
    open.then(|| view! {
        <div role="menu" style=format!("position:fixed;left:{x}px;top:{y}px;z-index:41")>
            <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;min-width:200px;">{children()}</KitBox>
        </div>
    })
}

/// A tooltip with a title and a body.
#[component]
pub fn RichTooltip(#[prop(into)] title: String, #[prop(into)] body: String) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Tooltip { rich: true } style="flex-direction:column;align-items:flex-start;gap:var(--dodai-space-xs);max-width:280px;">
            <span style="font-family:var(--dodai-font-display)">{title}</span>
            <span style="font-size:var(--dodai-size-sm)">{body}</span>
        </KitBox>
    }
}

/// A trigger with a card that shows under it on hover or focus (CSS only).
#[component]
pub fn HoverCard(#[prop(into)] trigger: ViewFn, children: Children) -> impl IntoView {
    view! {
        <div class="dodai-hovercard" style="position:relative;display:inline-block">
            <style>".dodai-hovercard>.dodai-hovercard__card{display:none}.dodai-hovercard:hover>.dodai-hovercard__card,.dodai-hovercard:focus-within>.dodai-hovercard__card{display:block}"</style>
            {trigger.run()}
            <div class="dodai-hovercard__card" style="position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)">
                <KitBox kit=Kit::Popup style="flex-direction:column;align-items:flex-start;min-width:240px;">{children()}</KitBox>
            </div>
        </div>
    }
}
