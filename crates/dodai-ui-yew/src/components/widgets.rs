//! Small building blocks: progress and loading, form fields, steppers, rows, tooltips and menus.
//! Built on the [`Kit`] looks and the stylesheet variables, so a design restyles all of them.

use super::kit::{stepped, DataTable, Icon, KitBox};
use super::surfaces::{IconButton, Kbd};
use super::{Button, Progress, Spinner};
use crate::dom::input_value;
use dodai_ui_core::{Kit, Tone};
use dodai_ui_icons::IconId;
use yew::prelude::*;

fn tone_colour(tone: Option<Tone>) -> &'static str {
    match tone {
        Some(Tone::Success) => "var(--dodai-success)",
        Some(Tone::Warning) => "var(--dodai-warning)",
        Some(Tone::Danger) => "var(--dodai-danger)",
        Some(Tone::Info) => "var(--dodai-info)",
        None => "var(--dodai-primary)",
    }
}

#[derive(Properties, PartialEq)]
pub struct ProgressBarProps {
    /// 0–1.
    pub value: f32,
    #[prop_or(Tone::Info)]
    pub tone: Tone,
    #[prop_or_default]
    pub label: Option<AttrValue>,
}

/// A bar for `value` 0–1 with an optional label and percentage above it.
#[function_component]
pub fn ProgressBar(props: &ProgressBarProps) -> Html {
    let pct = (props.value.clamp(0.0, 1.0) * 100.0).round();
    html! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            if let Some(l) = &props.label {
                <div style="display:flex;justify-content:space-between;font-size:var(--dodai-size-sm);color:var(--dodai-text-muted)">
                    <span>{ l.clone() }</span><span>{ format!("{pct}%") }</span>
                </div>
            }
            <Progress value={pct} tone={props.tone} />
        </div>
    }
}

/// A bar that runs while the length of the work is unknown.
#[function_component]
pub fn IndeterminateBar() -> Html {
    html! { <div class="dodai-indeterminate" role="progressbar"><span></span></div> }
}

#[derive(Properties, PartialEq)]
pub struct CircularProgressProps {
    /// 0–1; none spins.
    #[prop_or_default]
    pub value: Option<f32>,
    #[prop_or(40)]
    pub size: u32,
}

/// A ring: filled to `value` (0–1), or spinning without one.
#[function_component]
pub fn CircularProgress(props: &CircularProgressProps) -> Html {
    let s = props.size;
    match props.value {
        None => {
            html! { <span class="dodai-spinner" role="progressbar" style={format!("--s:{s}px")}></span> }
        }
        Some(v) => {
            let pct = v.clamp(0.0, 1.0) * 100.0;
            html! {
                <span role="progressbar" aria-valuenow={pct.round().to_string()}
                    style={format!("display:inline-block;flex:none;width:{s}px;height:{s}px;border-radius:50%;background:conic-gradient(var(--dodai-primary) {pct}%, var(--dodai-surface-raised) 0);-webkit-mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px));mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px))")}></span>
            }
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct LoadingIndicatorProps {
    #[prop_or(24)]
    pub size: u32,
}

/// The design's busy spinner at `size`.
#[function_component]
pub fn LoadingIndicator(props: &LoadingIndicatorProps) -> Html {
    html! { <Spinner size={props.size} /> }
}

#[derive(Properties, PartialEq)]
pub struct ChildrenProps {
    #[prop_or_default]
    pub children: Html,
}

/// Content with the skeleton shimmer running over it.
#[function_component]
pub fn Shimmer(props: &ChildrenProps) -> Html {
    html! { <div class="dodai-skeleton" style="height:auto;display:block">{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct IconBadgeProps {
    pub icon: IconId,
    #[prop_or_default]
    pub tone: Option<Tone>,
}

/// An icon in a tinted circle.
#[function_component]
pub fn IconBadge(props: &IconBadgeProps) -> Html {
    let c = tone_colour(props.tone);
    html! {
        <span style={format!("display:inline-grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border-radius:50%;color:{c};background:color-mix(in srgb, {c} 16%, transparent)")}>
            <Icon icon={props.icon} size={20} />
        </span>
    }
}

/// Body text in the design's font.
#[function_component]
pub fn Text(props: &ChildrenProps) -> Html {
    html! { <span style="font-family:var(--dodai-font-body);font-size:var(--dodai-size-md);color:var(--dodai-text)">{ props.children.clone() }</span> }
}

#[derive(Properties, PartialEq)]
pub struct ColumnProps {
    /// A spacing token: `xs`, `sm`, `md`, `lg`, `xl`.
    #[prop_or(AttrValue::Static("md"))]
    pub gap: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// A vertical stack with a spacing token.
#[function_component]
pub fn Column(props: &ColumnProps) -> Html {
    html! { <div style={format!("display:flex;flex-direction:column;gap:var(--dodai-space-{})", props.gap)}>{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct ImageProps {
    pub src: AttrValue,
    pub alt: AttrValue,
}

/// An image with the design's corner radius.
#[function_component]
pub fn Image(props: &ImageProps) -> Html {
    html! { <img src={props.src.clone()} alt={props.alt.clone()} style="display:block;max-width:100%;border-radius:var(--dodai-radius)" /> }
}

#[derive(Properties, PartialEq)]
pub struct FormFieldProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub hint: Option<AttrValue>,
    #[prop_or_default]
    pub error: Option<AttrValue>,
    #[prop_or_default]
    pub children: Html,
}

/// A label above a control, with a hint or an error (the error wins) below.
#[function_component]
pub fn FormField(props: &FormFieldProps) -> Html {
    let colour = if props.error.is_some() {
        "var(--dodai-danger)"
    } else {
        "var(--dodai-text-muted)"
    };
    let note = props.error.clone().or(props.hint.clone());
    html! {
        <label style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            <span style="font-size:var(--dodai-size-sm);color:var(--dodai-text)">{ props.label.clone() }</span>
            { props.children.clone() }
            if let Some(n) = note { <span style={format!("font-size:var(--dodai-size-xs);color:{colour}")}>{ n }</span> }
        </label>
    }
}

#[derive(Properties, PartialEq)]
pub struct InputBoxProps {
    pub value: AttrValue,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub leading: Option<IconId>,
    #[prop_or_default]
    pub oninput: Callback<String>,
}

/// A text input with an optional leading icon.
#[function_component]
pub fn InputBox(props: &InputBoxProps) -> Html {
    let cb = props.oninput.clone();
    let pad = if props.leading.is_some() {
        "padding-left:calc(var(--dodai-space-sm) * 3)"
    } else {
        ""
    };
    html! {
        <div style="position:relative;display:flex;align-items:center">
            if let Some(i) = props.leading {
                <span style="position:absolute;left:var(--dodai-space-sm);display:flex;color:var(--dodai-text-muted)"><Icon icon={i} size={18} /></span>
            }
            <input class="dodai-input" style={pad} value={props.value.clone()} placeholder={props.placeholder.clone()}
                oninput={Callback::from(move |e: InputEvent| cb.emit(input_value(&e)))} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct NumberStepperProps {
    pub value: i32,
    #[prop_or(i32::MIN)]
    pub min: i32,
    #[prop_or(i32::MAX)]
    pub max: i32,
    #[prop_or(1)]
    pub step: i32,
    #[prop_or_default]
    pub onchange: Callback<i32>,
}

/// A value with minus and plus buttons that stop at `min` and `max`.
#[function_component]
pub fn NumberStepper(props: &NumberStepperProps) -> Html {
    let (v, lo, hi, by) = (props.value, props.min, props.max, props.step);
    let (a, b) = (props.onchange.clone(), props.onchange.clone());
    html! {
        <div style="display:inline-flex;align-items:center;gap:var(--dodai-space-sm)">
            <IconButton icon={IconId::Minus} label="Decrease" disabled={v <= lo} onclick={Callback::from(move |_| a.emit(stepped(v, -by, lo, hi)))} />
            <span style="min-width:3ch;text-align:center;font-family:var(--dodai-font-mono)">{ v }</span>
            <IconButton icon={IconId::Plus} label="Increase" disabled={v >= hi} onclick={Callback::from(move |_| b.emit(stepped(v, by, lo, hi)))} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SplitButtonProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub onmenu: Callback<MouseEvent>,
}

/// A primary action with a chevron button for related actions.
#[function_component]
pub fn SplitButton(props: &SplitButtonProps) -> Html {
    html! {
        <div style="display:inline-flex;gap:2px">
            <Button onclick={props.onclick.clone()}>{ props.label.clone() }</Button>
            <button class="dodai-btn dodai-btn--primary dodai-btn--md dodai-btn--icon" aria-label="More" onclick={props.onmenu.clone()}>
                <Icon icon={IconId::ChevronDown} size={18} />
            </button>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct OtpInputProps {
    pub value: AttrValue,
    #[prop_or(6)]
    pub len: usize,
    #[prop_or_default]
    pub onchange: Callback<String>,
}

/// A one-time code in `len` boxes; the next empty one is outlined.
#[function_component]
pub fn OtpInput(props: &OtpInputProps) -> Html {
    let digits: Vec<char> = props.value.chars().collect();
    let len = props.len;
    let cb = props.onchange.clone();
    html! {
        <label style="position:relative;display:inline-flex;gap:var(--dodai-space-xs)">
            { for (0..len).map(|i| {
                let edge = if i == digits.len() { "var(--dodai-primary)" } else { "var(--dodai-border)" };
                let c = digits.get(i).map(char::to_string).unwrap_or_default();
                html! { <span style={format!("display:grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border:var(--dodai-border-width) solid {edge};border-radius:var(--dodai-radius);background:var(--dodai-surface-raised);font-family:var(--dodai-font-mono);font-size:var(--dodai-size-lg)")}>{ c }</span> }
            }) }
            <input inputmode="numeric" maxlength={len.to_string()} value={props.value.clone()} aria-label="One-time code"
                style="position:absolute;inset:0;opacity:0;cursor:text"
                oninput={Callback::from(move |e: InputEvent| cb.emit(input_value(&e).chars().filter(char::is_ascii_digit).take(len).collect()))} />
        </label>
    }
}

#[derive(Properties, PartialEq)]
pub struct FileDropZoneProps {
    #[prop_or(AttrValue::Static("Drop files here or click to choose"))]
    pub label: AttrValue,
}

/// A dashed area that takes files by drop or click.
#[function_component]
pub fn FileDropZone(props: &FileDropZoneProps) -> Html {
    html! {
        <label style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-lg);border:2px dashed var(--dodai-border-strong);border-radius:var(--dodai-radius);color:var(--dodai-text-muted);cursor:pointer">
            <Icon icon={IconId::Upload} size={28} />
            <span>{ props.label.clone() }</span>
            <input type="file" multiple=true style="display:none" />
        </label>
    }
}

#[derive(Properties, PartialEq)]
pub struct TableProps {
    pub columns: Vec<AttrValue>,
    pub rows: Vec<Vec<AttrValue>>,
}

/// A table with a head and banded rows.
#[function_component]
pub fn Table(props: &TableProps) -> Html {
    html! { <DataTable columns={props.columns.clone()} rows={props.rows.clone()} /> }
}

#[derive(Properties, PartialEq)]
pub struct TreeRowProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub depth: u32,
    /// `None` for a leaf.
    #[prop_or_default]
    pub open: Option<bool>,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// One row of a tree, indented by `depth`.
#[function_component]
pub fn TreeRow(props: &TreeRowProps) -> Html {
    let chevron = props.open.map(|o| {
        if o {
            IconId::ChevronDown
        } else {
            IconId::ChevronRight
        }
    });
    html! {
        <KitBox kit={Kit::Row { selected: props.selected }} pressable=true onclick={props.onclick.clone()}
            style={format!("padding-left:calc(var(--dodai-space-sm) + {} * var(--dodai-space-md));gap:var(--dodai-space-xs);width:100%;", props.depth)}>
            <span style="display:inline-flex;width:18px">
                if let Some(c) = chevron { <Icon icon={c} size={16} /> }
            </span>
            <span>{ props.label.clone() }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct TimelineEventProps {
    pub title: AttrValue,
    pub time: AttrValue,
    #[prop_or_default]
    pub tone: Option<Tone>,
    #[prop_or_default]
    pub children: Html,
}

/// One event on a timeline: a dot on a line, a title, a time and optional detail.
#[function_component]
pub fn TimelineEvent(props: &TimelineEventProps) -> Html {
    html! {
        <div style="display:flex;gap:var(--dodai-space-sm)">
            <div style="display:flex;flex-direction:column;align-items:center">
                <KitBox kit={Kit::Dot(props.tone)} style="width:10px;height:10px;flex:none;"><span></span></KitBox>
                <div style="flex:1;width:2px;background:var(--dodai-border)"></div>
            </div>
            <div style="display:flex;flex-direction:column;gap:2px;padding-bottom:var(--dodai-space-md)">
                <span style="color:var(--dodai-text)">{ props.title.clone() }</span>
                <span style="font-size:var(--dodai-size-xs);color:var(--dodai-text-muted)">{ props.time.clone() }</span>
                { props.children.clone() }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct MenuItemProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub icon: Option<IconId>,
    #[prop_or_default]
    pub shortcut: Option<AttrValue>,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// A menu row: icon, label, shortcut.
#[function_component]
pub fn MenuItem(props: &MenuItemProps) -> Html {
    html! {
        <KitBox kit={Kit::Row { selected: false }} pressable=true onclick={props.onclick.clone()}
            style="gap:var(--dodai-space-sm);width:100%;justify-content:flex-start;">
            if let Some(i) = props.icon { <Icon icon={i} size={18} /> }
            <span style="flex:1;text-align:left">{ props.label.clone() }</span>
            if let Some(s) = props.shortcut.clone().filter(|s| !s.is_empty()) { <Kbd>{ s }</Kbd> }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct DropdownMenuProps {
    pub open: bool,
    pub trigger: Html,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A trigger with a menu hanging under it while `open`; pressing outside dismisses.
#[function_component]
pub fn DropdownMenu(props: &DropdownMenuProps) -> Html {
    html! {
        <div style="position:relative;display:inline-block">
            { props.trigger.clone() }
            if props.open {
                <div style="position:fixed;inset:0;z-index:40" onclick={props.ondismiss.clone()}></div>
                <div role="menu" style="position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)">
                    <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;min-width:200px;">{ props.children.clone() }</KitBox>
                </div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ContextMenuProps {
    pub open: bool,
    pub x: f64,
    pub y: f64,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// A menu at a point of the window (a right-click).
#[function_component]
pub fn ContextMenu(props: &ContextMenuProps) -> Html {
    if !props.open {
        return html! {};
    }
    html! {
        <>
            <div style="position:fixed;inset:0;z-index:40" onclick={props.ondismiss.clone()}></div>
            <div role="menu" style={format!("position:fixed;left:{}px;top:{}px;z-index:41", props.x, props.y)}>
                <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;min-width:200px;">{ props.children.clone() }</KitBox>
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
pub struct RichTooltipProps {
    pub title: AttrValue,
    pub body: AttrValue,
}

/// A tooltip with a title and a body.
#[function_component]
pub fn RichTooltip(props: &RichTooltipProps) -> Html {
    html! {
        <KitBox kit={Kit::Tooltip { rich: true }} style="flex-direction:column;align-items:flex-start;gap:var(--dodai-space-xs);max-width:280px;">
            <span style="font-family:var(--dodai-font-display)">{ props.title.clone() }</span>
            <span style="font-size:var(--dodai-size-sm)">{ props.body.clone() }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct HoverCardProps {
    pub trigger: Html,
    #[prop_or_default]
    pub children: Html,
}

/// A trigger with a card that shows under it on hover or focus (CSS only).
#[function_component]
pub fn HoverCard(props: &HoverCardProps) -> Html {
    html! {
        <div class="dodai-hovercard" style="position:relative;display:inline-block">
            <style>{ ".dodai-hovercard>.dodai-hovercard__card{display:none}.dodai-hovercard:hover>.dodai-hovercard__card,.dodai-hovercard:focus-within>.dodai-hovercard__card{display:block}" }</style>
            { props.trigger.clone() }
            <div class="dodai-hovercard__card" style="position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)">
                <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:flex-start;min-width:240px;">{ props.children.clone() }</KitBox>
            </div>
        </div>
    }
}
