//! Display components: panel, tag, badge, tabs, progress, list, divider, tooltip.
use crate::classes::{clamp_pct, join, progress_tone, tone};
use crate::use_glass;
use dodai_core::{SurfaceClass, Tone};
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
    /// The panel's own surface override (frost, fill opacity).
    #[prop_or_default]
    pub surface: dodai_core::Surface,
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
    let (glass, style) = use_glass(SurfaceClass::Panel, &props.surface);
    let class = format!("{class}{glass}");
    html! {
        <div {class} {style}>
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
    /// A close cross on every tab; the app removes the tab in `onclose`.
    #[prop_or_default]
    pub closable: bool,
    #[prop_or_default]
    pub onclose: Callback<usize>,
    /// A "+" after the last tab; the app adds the tab in `onadd`.
    #[prop_or_default]
    pub addable: bool,
    #[prop_or_default]
    pub onadd: Callback<()>,
}

/// Tab strip; `active` is the selected index.
#[function_component]
pub fn Tabs(props: &TabsProps) -> Html {
    // Roving focus (shared rule in dodai-core): arrows, Home and End pick the next tab.
    let (n, active, onselect) = (props.tabs.len(), props.active, props.onselect.clone());
    let onkeydown = move |e: KeyboardEvent| {
        if let Some(to) = dodai_core::roving::key_name(&e.key()).and_then(|k| {
            dodai_core::roving::next(k, dodai_core::roving::Axis::Horizontal, active, n, |_| true)
        }) {
            e.prevent_default();
            onselect.emit(to);
        }
    };
    html! {
        <nav class="dodai-tabs" role="tablist" {onkeydown}>
        { for props.tabs.iter().enumerate().map(|(i, t)| {
            let on = i == props.active;
            let cb = props.onselect.clone();
            let close = props.onclose.clone();
            html! {
                <a key={i} role="tab" aria-selected={on.to_string()} tabindex={if on { "0" } else { "-1" }}
                   class={join(&["dodai-tab", if on { "dodai-tab--active" } else { "" }])}
                   href="#" onclick={move |e: MouseEvent| { e.prevent_default(); cb.emit(i) }}>
                    <span>{ t.clone() }</span>
                    if props.closable {
                        <span class="dodai-tab__close" role="button" tabindex="0" aria-label="Close tab"
                              onclick={move |e: MouseEvent| { e.prevent_default(); e.stop_propagation(); close.emit(i) }}>
                            { "\u{00d7}" }
                        </span>
                    }
                </a>
            }
        }) }
        if props.addable {
            <button type="button" class="dodai-tab dodai-tab--add" aria-label="Add tab"
                    onclick={{ let add = props.onadd.clone(); move |_| add.emit(()) }}>{ "+" }</button>
        }
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
    let (glass, style) = use_glass(SurfaceClass::Card, &dodai_core::Surface::INHERIT);
    let class = format!("dodai-list{glass}");
    html! { <ul {class} {style}>{ props.children.clone() }</ul> }
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

#[derive(Properties, PartialEq)]
pub struct SpinnerProps {
    /// Width and height, px.
    #[prop_or(24)]
    pub size: u32,
    /// The spinner's shape; the plain ring when unset.
    #[prop_or(dodai_core::SpinnerStyle::Ring)]
    pub style: dodai_core::SpinnerStyle,
}

/// A busy spinner in any [`dodai_core::SpinnerStyle`].
#[function_component]
pub fn Spinner(props: &SpinnerProps) -> Html {
    Html::from_html_unchecked(AttrValue::from(dodai_topcoat::html::spinner_of(
        props.style,
        props.size,
    )))
}

#[derive(Properties, PartialEq)]
pub struct StyledProgressProps {
    /// A design's own bar style.
    pub style: dodai_core::ProgressStyle,
    /// Fraction 0..=1, or unset for a running bar.
    #[prop_or_default]
    pub value: Option<f32>,
}

/// A progress bar in a design's own style (hud, stripes, stones).
#[function_component]
pub fn StyledProgress(props: &StyledProgressProps) -> Html {
    Html::from_html_unchecked(AttrValue::from(dodai_topcoat::html::styled_progress(
        props.style,
        props.value,
    )))
}

#[derive(Properties, PartialEq)]
pub struct MenuProps {
    pub rows: Vec<dodai_core::menu::MenuRow>,
    /// The highlighted row (controlled).
    #[prop_or_default]
    pub active: usize,
    /// The highlight moved (arrow, Home, End or hover); the app updates `active`.
    #[prop_or_default]
    pub onactive: Callback<usize>,
    /// A row was picked (click, Enter or Space).
    #[prop_or_default]
    pub onpick: Callback<usize>,
}

/// Popup action list. Keys follow the shared rule of `dodai_core::menu`.
#[function_component]
pub fn Menu(props: &MenuProps) -> Html {
    let (rows, active) = (props.rows.clone(), props.active);
    let (onactive, onpick) = (props.onactive.clone(), props.onpick.clone());
    let onkeydown = {
        let (onactive, onpick) = (onactive.clone(), onpick.clone());
        move |e: KeyboardEvent| match dodai_core::menu::step(&e.key(), &rows, active) {
            Some(dodai_core::menu::Step::Move(i)) => {
                e.prevent_default();
                onactive.emit(i)
            }
            Some(dodai_core::menu::Step::Pick(i)) => {
                e.prevent_default();
                onpick.emit(i)
            }
            None => {}
        }
    };
    html! {
        <div class="dodai-menu" role="menu" tabindex="0" {onkeydown}>
        { for props.rows.iter().enumerate().map(|(i, r)| {
            if r.separator {
                return html! { <div key={i} class="dodai-menu__sep" role="separator"></div> };
            }
            let pickable = r.pickable();
            let class = join(&[
                "dodai-menu__item",
                if r.disabled { "dodai-menu__item--disabled" } else { "" },
                if r.danger { "dodai-menu__item--danger" } else { "" },
                if i == props.active { "dodai-menu__item--active" } else { "" },
            ]);
            let (hover, pick) = (onactive.clone(), onpick.clone());
            html! {
                <div key={i} {class} role="menuitem" aria-disabled={r.disabled.to_string()}
                     onmouseenter={move |_| if pickable { hover.emit(i) }}
                     onclick={move |_| if pickable { pick.emit(i) }}>
                    <span class="dodai-menu__check">{ if r.checked { "\u{2713}" } else { "" } }</span>
                    <span class="dodai-menu__label">{ r.label.clone() }</span>
                    if let Some(k) = &r.shortcut { <span class="dodai-menu__key">{ k.clone() }</span> }
                </div>
            }
        }) }
        </div>
    }
}
