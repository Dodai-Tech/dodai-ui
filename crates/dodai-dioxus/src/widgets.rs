//! Small building blocks: progress and loading, form fields, steppers, rows, tooltips and menus.
//! Built on the [`Kit`] looks and the stylesheet variables, so a design restyles all of them.

use crate::{Icon, KitBox};
use dioxus::prelude::*;
use dodai_core::{Kit, Tone};
use dodai_icons::IconId;

/// Properties of [`ProgressBar`].
#[derive(Props, Clone, PartialEq)]
pub struct ProgressBarProps {
    /// 0.0..=1.0
    pub value: f32,
    #[props(default)]
    pub tone: Option<Tone>,
    /// Shown above the bar with the percentage.
    #[props(default)]
    pub label: Option<String>,
}

/// A determinate bar, optionally labelled with its percentage.
#[component]
pub fn ProgressBar(props: ProgressBarProps) -> Element {
    let pct = (props.value.clamp(0.0, 1.0) * 100.0).round();
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-xs)",
            if let Some(l) = props.label {
                div { style: "display:flex;justify-content:space-between;font-size:var(--dodai-size-sm);color:var(--dodai-text-muted)",
                    span { "{l}" }
                    span { "{pct}%" }
                }
            }
            crate::Progress { value: props.value, tone: props.tone }
        }
    }
}

/// A bar that slides while the length is unknown.
#[component]
pub fn IndeterminateBar() -> Element {
    rsx! { crate::Indeterminate {} }
}

/// Properties of [`CircularProgress`].
#[derive(Props, Clone, PartialEq)]
pub struct CircularProgressProps {
    /// 0.0..=1.0; `None` spins.
    #[props(default)]
    pub value: Option<f32>,
    #[props(default = 40)]
    pub size: u32,
}

/// A ring; with a value it fills, without it spins.
#[component]
pub fn CircularProgress(props: CircularProgressProps) -> Element {
    let s = props.size;
    let Some(v) = props.value else {
        return rsx! { span { class: "dodai-spinner", role: "progressbar", style: "--s:{s}px" } };
    };
    let pct = v.clamp(0.0, 1.0) * 100.0;
    rsx! {
        span { role: "progressbar", aria_valuenow: "{pct.round()}", style: "display:inline-block;flex:none;width:{s}px;height:{s}px;border-radius:50%;background:conic-gradient(var(--dodai-primary) {pct}%, var(--dodai-surface-raised) 0);-webkit-mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px));mask:radial-gradient(farthest-side, transparent calc(100% - 4px), #000 calc(100% - 3px))" }
    }
}

/// The design's own spinner.
#[component]
pub fn LoadingIndicator(#[props(default = 24)] size: u32) -> Element {
    rsx! { crate::Spinner { size } }
}

/// A sheen that sweeps across its children while they load.
#[component]
pub fn Shimmer(children: Element) -> Element {
    rsx! { div { class: "dodai-skeleton", style: "height:auto;display:block", {children} } }
}

/// Properties of [`IconBadge`].
#[derive(Props, Clone, PartialEq)]
pub struct IconBadgeProps {
    pub icon: IconId,
    #[props(default)]
    pub tone: Option<Tone>,
}

/// An icon on a round tinted disc.
#[component]
pub fn IconBadge(props: IconBadgeProps) -> Element {
    let colour = match props.tone {
        Some(Tone::Success) => "var(--dodai-success)",
        Some(Tone::Warning) => "var(--dodai-warning)",
        Some(Tone::Danger) => "var(--dodai-danger)",
        Some(Tone::Info) => "var(--dodai-info)",
        None => "var(--dodai-primary)",
    };
    rsx! {
        span { style: "display:inline-grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border-radius:50%;color:{colour};background:color-mix(in srgb, {colour} 16%, transparent)",
            Icon { icon: props.icon, size: 20 }
        }
    }
}

/// Body text in the design's font.
#[component]
pub fn Text(children: Element) -> Element {
    rsx! { span { style: "font-family:var(--dodai-font-body);font-size:var(--dodai-size-md);color:var(--dodai-text)", {children} } }
}

/// Properties of [`Column`].
#[derive(Props, Clone, PartialEq)]
pub struct ColumnProps {
    /// A spacing token name: `xs`, `sm`, `md`, `lg` or `xl`.
    #[props(default = "md".to_string())]
    pub gap: String,
    pub children: Element,
}

/// Children stacked top to bottom with a token gap.
#[component]
pub fn Column(props: ColumnProps) -> Element {
    rsx! { div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-{props.gap})", {props.children} } }
}

/// An image that takes the design's corner radius.
#[component]
pub fn Image(src: String, alt: String) -> Element {
    rsx! { img { src: "{src}", alt: "{alt}", style: "display:block;max-width:100%;border-radius:var(--dodai-radius)" } }
}

/// Properties of [`FormField`].
#[derive(Props, Clone, PartialEq)]
pub struct FormFieldProps {
    pub label: String,
    #[props(default)]
    pub hint: Option<String>,
    /// An error replaces the hint and tints the text.
    #[props(default)]
    pub error: Option<String>,
    pub children: Element,
}

/// A label, a control and a hint or error line.
#[component]
pub fn FormField(props: FormFieldProps) -> Element {
    let note = props.error.clone().or(props.hint.clone());
    let colour = if props.error.is_some() {
        "var(--dodai-danger)"
    } else {
        "var(--dodai-text-muted)"
    };
    rsx! {
        label { style: "display:flex;flex-direction:column;gap:var(--dodai-space-xs)",
            span { style: "font-size:var(--dodai-size-sm);color:var(--dodai-text)", "{props.label}" }
            {props.children}
            if let Some(n) = note { span { style: "font-size:var(--dodai-size-xs);color:{colour}", "{n}" } }
        }
    }
}

/// Properties of [`InputBox`].
#[derive(Props, Clone, PartialEq)]
pub struct InputBoxProps {
    pub value: String,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub leading: Option<IconId>,
    #[props(default)]
    pub oninput: EventHandler<String>,
}

/// A text field with an optional leading icon.
#[component]
pub fn InputBox(props: InputBoxProps) -> Element {
    rsx! {
        div { style: "position:relative;display:flex;align-items:center",
            if let Some(i) = props.leading {
                span { style: "position:absolute;left:var(--dodai-space-sm);display:flex;color:var(--dodai-text-muted)", Icon { icon: i, size: 18 } }
            }
            input { class: "dodai-input", style: if props.leading.is_some() { "padding-left:calc(var(--dodai-space-sm) * 3)" } else { "" },
                value: props.value, placeholder: props.placeholder, oninput: move |e| props.oninput.call(e.value()) }
        }
    }
}

/// Properties of [`NumberStepper`].
#[derive(Props, Clone, PartialEq)]
pub struct NumberStepperProps {
    pub value: i32,
    #[props(default = i32::MIN)]
    pub min: i32,
    #[props(default = i32::MAX)]
    pub max: i32,
    #[props(default = 1)]
    pub step: i32,
    #[props(default)]
    pub onchange: EventHandler<i32>,
}

/// Minus, the value and plus. Controlled; clamps to `min..=max`.
#[component]
pub fn NumberStepper(props: NumberStepperProps) -> Element {
    let (v, lo, hi, by, on) = (
        props.value,
        props.min,
        props.max,
        props.step,
        props.onchange,
    );
    rsx! {
        div { style: "display:inline-flex;align-items:center;gap:var(--dodai-space-sm)",
            crate::IconButton { icon: IconId::Minus, label: "Decrease", disabled: v <= lo, onclick: move |_| on.call(crate::stepped(v, -by, lo, hi)) }
            span { style: "min-width:3ch;text-align:center;font-family:var(--dodai-font-mono)", "{v}" }
            crate::IconButton { icon: IconId::Plus, label: "Increase", disabled: v >= hi, onclick: move |_| on.call(crate::stepped(v, by, lo, hi)) }
        }
    }
}

/// Properties of [`SplitButton`].
#[derive(Props, Clone, PartialEq)]
pub struct SplitButtonProps {
    pub label: String,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
    /// The chevron half, which usually opens a menu.
    #[props(default)]
    pub onmenu: EventHandler<MouseEvent>,
}

/// A button with a second, attached chevron button.
#[component]
pub fn SplitButton(props: SplitButtonProps) -> Element {
    let (a, b) = (props.onclick, props.onmenu);
    rsx! {
        div { style: "display:inline-flex;gap:2px",
            crate::Button { onclick: move |e| a.call(e), "{props.label}" }
            crate::Button { icon_only: true, onclick: move |e| b.call(e),
                span { aria_label: "More", Icon { icon: IconId::ChevronDown, size: 18 } }
            }
        }
    }
}

/// Properties of [`OtpInput`].
#[derive(Props, Clone, PartialEq)]
pub struct OtpInputProps {
    pub value: String,
    #[props(default = 6)]
    pub len: usize,
    #[props(default)]
    pub onchange: EventHandler<String>,
}

/// A one-time code: one cell per digit over a single hidden field. Controlled.
#[component]
pub fn OtpInput(props: OtpInputProps) -> Element {
    let digits: Vec<char> = props.value.chars().collect();
    let len = props.len;
    rsx! {
        label { style: "position:relative;display:inline-flex;gap:var(--dodai-space-xs)",
            for i in 0..len {
                {let edge = if i == digits.len() { "var(--dodai-primary)" } else { "var(--dodai-border)" };
                rsx! { span { key: "{i}", style: "display:grid;place-items:center;width:var(--dodai-control-height);height:var(--dodai-control-height);border:var(--dodai-border-width) solid {edge};border-radius:var(--dodai-radius);background:var(--dodai-surface-raised);font-family:var(--dodai-font-mono);font-size:var(--dodai-size-lg)",
                    "{digits.get(i).map(|c| c.to_string()).unwrap_or_default()}"
                } }}
            }
            input { inputmode: "numeric", maxlength: "{len}", value: props.value, aria_label: "One-time code",
                style: "position:absolute;inset:0;opacity:0;cursor:text",
                oninput: move |e| props.onchange.call(e.value().chars().filter(char::is_ascii_digit).take(len).collect()) }
        }
    }
}

/// Properties of [`FileDropZone`].
#[derive(Props, Clone, PartialEq)]
pub struct FileDropZoneProps {
    #[props(default = "Drop files here or click to choose".to_string())]
    pub label: String,
    /// File names picked or dropped.
    #[props(default)]
    pub onfiles: EventHandler<Vec<String>>,
}

/// A dashed target that takes dropped or chosen files.
#[component]
pub fn FileDropZone(props: FileDropZoneProps) -> Element {
    let on = props.onfiles;
    rsx! {
        label { style: "display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm);padding:var(--dodai-space-lg);border:2px dashed var(--dodai-border-strong);border-radius:var(--dodai-radius);color:var(--dodai-text-muted);cursor:pointer",
            Icon { icon: IconId::Upload, size: 28 }
            span { "{props.label}" }
            input { r#type: "file", multiple: true, style: "display:none",
                onchange: move |e| on.call(e.files().iter().map(|f| f.name()).collect()) }
        }
    }
}

/// Properties of [`Table`].
#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// A header and banded rows.
#[component]
pub fn Table(props: TableProps) -> Element {
    rsx! { crate::DataTable { columns: props.columns, rows: props.rows } }
}

/// Properties of [`TreeRow`].
#[derive(Props, Clone, PartialEq)]
pub struct TreeRowProps {
    pub label: String,
    #[props(default)]
    pub depth: u32,
    /// `None` is a leaf; `Some(open)` shows a chevron.
    #[props(default)]
    pub open: Option<bool>,
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// One indented row of a tree.
#[component]
pub fn TreeRow(props: TreeRowProps) -> Element {
    let on = props.onclick;
    let chevron = match props.open {
        Some(true) => Some(IconId::ChevronDown),
        Some(false) => Some(IconId::ChevronRight),
        None => None,
    };
    rsx! {
        KitBox { kit: Kit::Row { selected: props.selected }, style: format!("padding-left:calc(var(--dodai-space-sm) + {} * var(--dodai-space-md));gap:var(--dodai-space-xs);width:100%", props.depth),
            onclick: move |e| on.call(e),
            span { style: "display:inline-flex;width:18px",
                if let Some(c) = chevron { Icon { icon: c, size: 16 } }
            }
            span { "{props.label}" }
        }
    }
}

/// Properties of [`TimelineEvent`].
#[derive(Props, Clone, PartialEq)]
pub struct TimelineEventProps {
    pub title: String,
    pub time: String,
    #[props(default)]
    pub tone: Option<Tone>,
    #[props(default = rsx! {})]
    pub children: Element,
}

/// One entry of a timeline: a dot, a title, a time and a body.
#[component]
pub fn TimelineEvent(props: TimelineEventProps) -> Element {
    rsx! {
        div { style: "display:flex;gap:var(--dodai-space-sm)",
            div { style: "display:flex;flex-direction:column;align-items:center",
                KitBox { kit: Kit::Dot(props.tone), style: "width:10px;height:10px;flex:none".to_string(), span {} }
                div { style: "flex:1;width:2px;background:var(--dodai-border)" }
            }
            div { style: "display:flex;flex-direction:column;gap:2px;padding-bottom:var(--dodai-space-md)",
                span { style: "color:var(--dodai-text)", "{props.title}" }
                span { style: "font-size:var(--dodai-size-xs);color:var(--dodai-text-muted)", "{props.time}" }
                {props.children}
            }
        }
    }
}

/// Properties of [`MenuItem`].
#[derive(Props, Clone, PartialEq)]
pub struct MenuItemProps {
    pub label: String,
    #[props(default)]
    pub icon: Option<IconId>,
    /// A shortcut hint on the right.
    #[props(default)]
    pub shortcut: Option<String>,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// One row of a menu.
#[component]
pub fn MenuItem(props: MenuItemProps) -> Element {
    let on = props.onclick;
    rsx! {
        KitBox { kit: Kit::Row { selected: false }, style: "gap:var(--dodai-space-sm);width:100%;justify-content:flex-start".to_string(), onclick: move |e| on.call(e),
            if let Some(i) = props.icon { Icon { icon: i, size: 18 } }
            span { style: "flex:1;text-align:left", "{props.label}" }
            if let Some(s) = props.shortcut { crate::Kbd { "{s}" } }
        }
    }
}

/// Properties of [`DropdownMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuProps {
    pub open: bool,
    /// The element the menu hangs from.
    pub trigger: Element,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A popup list under its trigger. Controlled; pressing outside dismisses.
#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    let dismiss = props.ondismiss;
    rsx! {
        div { style: "position:relative;display:inline-block",
            {props.trigger}
            if props.open {
                div { style: "position:fixed;inset:0;z-index:40", onclick: move |e| dismiss.call(e) }
                div { role: "menu", style: "position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)",
                    KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch;min-width:200px".to_string(), {props.children} }
                }
            }
        }
    }
}

/// Properties of [`ContextMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct ContextMenuProps {
    pub open: bool,
    /// Viewport position of the pointer.
    pub x: f64,
    pub y: f64,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
    pub children: Element,
}

/// A popup list at the pointer. Controlled.
#[component]
pub fn ContextMenu(props: ContextMenuProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let dismiss = props.ondismiss;
    rsx! {
        div { style: "position:fixed;inset:0;z-index:40", onclick: move |e| dismiss.call(e) }
        div { role: "menu", style: "position:fixed;left:{props.x}px;top:{props.y}px;z-index:41",
            KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch;min-width:200px".to_string(), {props.children} }
        }
    }
}

/// Properties of [`RichTooltip`].
#[derive(Props, Clone, PartialEq)]
pub struct RichTooltipProps {
    pub title: String,
    pub body: String,
}

/// A tooltip with a title and a body, shown in place.
#[component]
pub fn RichTooltip(props: RichTooltipProps) -> Element {
    rsx! {
        KitBox { kit: Kit::Tooltip { rich: true }, style: "flex-direction:column;align-items:flex-start;gap:var(--dodai-space-xs);max-width:280px".to_string(),
            span { style: "font-family:var(--dodai-font-display)", "{props.title}" }
            span { style: "font-size:var(--dodai-size-sm)", "{props.body}" }
        }
    }
}

/// Properties of [`HoverCard`].
#[derive(Props, Clone, PartialEq)]
pub struct HoverCardProps {
    pub trigger: Element,
    pub children: Element,
}

/// A card that appears under its trigger on hover or focus, with no script.
#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    let mut shown = use_signal(|| false);
    rsx! {
        div { style: "position:relative;display:inline-block",
            onmouseenter: move |_| shown.set(true), onmouseleave: move |_| shown.set(false),
            onfocusin: move |_| shown.set(true), onfocusout: move |_| shown.set(false),
            {props.trigger}
            if shown() {
                div { style: "position:absolute;top:100%;left:0;z-index:41;margin-top:var(--dodai-space-xs)",
                    KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:flex-start;min-width:240px".to_string(), {props.children} }
                }
            }
        }
    }
}
