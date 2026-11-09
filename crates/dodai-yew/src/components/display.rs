//! Display components: panel, tag, badge, tabs, progress, list, divider, tooltip.
use crate::classes::{clamp_pct, join, progress_tone, tone};
use dodai_core::Tone;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct PanelProps {
    #[prop_or_default]
    pub title: Option<AttrValue>,
    /// Primary border with glow.
    #[prop_or_default]
    pub focused: bool,
    #[prop_or_default]
    pub raised: bool,
    #[prop_or_default]
    pub children: Html,
}

/// Surface panel / card.
#[function_component]
pub fn Panel(props: &PanelProps) -> Html {
    let class = join(&[
        "dodai-panel",
        if props.focused {
            "dodai-panel--focus"
        } else {
            ""
        },
        if props.raised {
            "dodai-panel--raised"
        } else {
            ""
        },
    ]);
    html! {
        <div {class}>
            <span class="dodai-panel__shade" aria-hidden="true"></span>
            if let Some(t) = &props.title { <div class="dodai-panel__title">{ t.clone() }</div> }
            { props.children.clone() }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TagProps {
    #[prop_or(Tone::Info)]
    pub tone: Tone,
    /// Use the brand colour instead of a status tone.
    #[prop_or_default]
    pub primary: bool,
    #[prop_or_default]
    pub secondary: bool,
    /// Tinted (soft) instead of solid.
    #[prop_or_default]
    pub soft: bool,
    /// Round count badge.
    #[prop_or_default]
    pub badge: bool,
    #[prop_or_default]
    pub children: Html,
}

/// Tag / badge.
#[function_component]
pub fn Tag(props: &TagProps) -> Html {
    let class = join(&[
        if props.badge {
            "dodai-badge"
        } else {
            "dodai-tag"
        },
        if props.soft { "dodai-tag--soft" } else { "" },
        if props.primary {
            "dodai-tag--primary"
        } else if props.secondary {
            "dodai-tag--secondary"
        } else {
            tone(props.tone)
        },
    ]);
    html! { <span {class}>{ props.children.clone() }</span> }
}

#[derive(Properties, PartialEq)]
pub struct TabsProps {
    pub tabs: Vec<AttrValue>,
    pub active: usize,
    #[prop_or_default]
    pub onselect: Callback<usize>,
}

/// Tab strip; `active` is the selected index.
#[function_component]
pub fn Tabs(props: &TabsProps) -> Html {
    html! {
        <nav class="dodai-tabs" role="tablist">
        { for props.tabs.iter().enumerate().map(|(i, t)| {
            let on = i == props.active;
            let cb = props.onselect.clone();
            html! {
                <a key={i} role="tab" aria-selected={on.to_string()}
                   class={join(&["dodai-tab", if on { "dodai-tab--active" } else { "" }])}
                   href="#" onclick={move |e: MouseEvent| { e.prevent_default(); cb.emit(i) }}>
                    { t.clone() }
                </a>
            }
        }) }
        </nav>
    }
}

#[derive(Properties, PartialEq)]
pub struct ProgressProps {
    /// 0–100.
    pub value: f32,
    #[prop_or(Tone::Info)]
    pub tone: Tone,
}

/// Progress bar.
#[function_component]
pub fn Progress(props: &ProgressProps) -> Html {
    let v = clamp_pct(props.value);
    html! {
        <div class={join(&["dodai-progress", progress_tone(props.tone)])} role="progressbar"
             aria-valuemin="0" aria-valuemax="100" aria-valuenow={v.to_string()}>
            <span style={format!("--v:{v}%")}></span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ListProps {
    #[prop_or_default]
    pub children: Html,
}

/// List container for [`ListItem`]s.
#[function_component]
pub fn List(props: &ListProps) -> Html {
    html! { <ul class="dodai-list">{ props.children.clone() }</ul> }
}

#[derive(Properties, PartialEq)]
pub struct ListItemProps {
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
    #[prop_or_default]
    pub children: Html,
}

/// Selectable row.
#[function_component]
pub fn ListItem(props: &ListItemProps) -> Html {
    html! {
        <li class="dodai-list__item" aria-selected={props.selected.to_string()}
            onclick={props.onclick.clone()}>{ props.children.clone() }</li>
    }
}

/// Horizontal rule.
#[function_component]
pub fn Divider() -> Html {
    html! { <hr class="dodai-divider" /> }
}

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    /// Text shown on hover/focus (CSS-only).
    pub text: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// Wraps children with a hover tooltip.
#[function_component]
pub fn Tooltip(props: &TooltipProps) -> Html {
    html! { <span class="dodai-tip" data-tip={props.text.clone()}>{ props.children.clone() }</span> }
}

/// Static tooltip bubble.
#[function_component]
pub fn TooltipBubble(props: &ListProps) -> Html {
    html! { <span class="dodai-tooltip">{ props.children.clone() }</span> }
}
