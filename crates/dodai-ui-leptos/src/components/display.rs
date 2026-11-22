use crate::class;
use dodai_ui_core::{ButtonVariant, Tone};
use leptos::prelude::*;

/// Button with any [`ButtonVariant`]; `disabled` renders the disabled style.
#[component]
pub fn Button(
    #[prop(default = ButtonVariant::Primary)] variant: ButtonVariant,
    #[prop(optional)] disabled: bool,
    /// The button's own surface override (frost, fill opacity).
    #[prop(optional)]
    surface: dodai_ui_core::Surface,
    children: Children,
) -> impl IntoView {
    let (glass, style) = crate::glass(dodai_ui_core::SurfaceClass::Button, &surface);
    let class = format!("{}{glass}", class::button(variant));
    view! { <button class=class style=style disabled=disabled>{children()}</button> }
}

/// Panel / card with optional title.
#[component]
pub fn Panel(
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional)] focused: bool,
    #[prop(optional)] raised: bool,
    /// The panel's own surface override (frost, fill opacity).
    #[prop(optional)]
    surface: dodai_ui_core::Surface,
    children: Children,
) -> impl IntoView {
    let (glass, style) = crate::glass(dodai_ui_core::SurfaceClass::Panel, &surface);
    let class = format!("{}{glass}", class::panel(focused, raised));
    view! {
        <div class=class style=style>
            <span class="dodai-panel__shade" aria-hidden="true"></span>
            {title.map(|t| view! { <div class="dodai-panel__title">{t}</div> })}
            {children()}
        </div>
    }
}

/// Tag (or compact badge when `badge`).
#[component]
pub fn Tag(
    #[prop(optional)] tone: class::TagTone,
    #[prop(optional)] soft: bool,
    #[prop(optional)] badge: bool,
    children: Children,
) -> impl IntoView {
    view! { <span class=class::tag(tone, soft, badge)>{children()}</span> }
}

/// Tab strip: `tabs` are labels, `active` the selected index.
#[component]
pub fn Tabs(
    tabs: Vec<String>,
    #[prop(optional)] active: usize,
    /// Called with the index of the tab to select when the arrow, Home or End keys move through
    /// the strip (the shared roving rule of `dodai-ui-core`); the app updates `active`.
    #[prop(optional, into)]
    on_select: Option<Callback<usize>>,
    /// Called with the index of the tab whose close cross was clicked; its presence makes every
    /// tab closable. The app removes the tab.
    #[prop(optional, into)]
    on_close: Option<Callback<usize>>,
    /// Called when the "+" after the last tab is clicked; its presence shows the button.
    #[prop(optional, into)]
    on_add: Option<Callback<()>>,
) -> impl IntoView {
    let n = tabs.len();
    let onkeydown = move |e: leptos::ev::KeyboardEvent| {
        if let (Some(cb), Some(to)) = (
            on_select,
            dodai_ui_core::roving::key_name(&e.key()).and_then(|k| {
                dodai_ui_core::roving::next(
                    k,
                    dodai_ui_core::roving::Axis::Horizontal,
                    active,
                    n,
                    |_| true,
                )
            }),
        ) {
            e.prevent_default();
            cb.run(to);
        }
    };
    view! {
        <nav class="dodai-tabs" role="tablist" on:keydown=onkeydown>
            {tabs
                .into_iter()
                .enumerate()
                .map(|(i, t)| {
                    let close = on_close.map(|cb| {
                        view! {
                            <span
                                class="dodai-tab__close"
                                role="button"
                                tabindex="0"
                                aria-label="Close tab"
                                on:click=move |e| {
                                    e.prevent_default();
                                    e.stop_propagation();
                                    cb.run(i)
                                }
                            >
                                "\u{00d7}"
                            </span>
                        }
                    });
                    view! { <a class=class::tab(i == active) href="#"><span>{t}</span>{close}</a> }
                })
                .collect_view()}
            {on_add
                .map(|cb| {
                    view! {
                        <button
                            type="button"
                            class="dodai-tab dodai-tab--add"
                            aria-label="Add tab"
                            on:click=move |_| cb.run(())
                        >
                            "+"
                        </button>
                    }
                })}
        </nav>
    }
}

/// Progress bar, `value` 0–100.
#[component]
pub fn Progress(value: f32, #[prop(optional)] tone: Option<Tone>) -> impl IntoView {
    let v = format!("--v:{}", class::percent(value));
    view! { <div class=class::progress(tone)><span style=v></span></div> }
}

/// Selectable list row.
#[component]
pub fn ListItem(#[prop(optional)] selected: bool, children: Children) -> impl IntoView {
    view! { <li class="dodai-list__item" aria-selected=selected.to_string()>{children()}</li> }
}

/// List container for [`ListItem`]s.
#[component]
pub fn List(children: Children) -> impl IntoView {
    let (glass, style) = crate::glass(
        dodai_ui_core::SurfaceClass::Card,
        &dodai_ui_core::Surface::INHERIT,
    );
    view! { <ul class=format!("dodai-list{glass}") style=style>{children()}</ul> }
}

/// Horizontal divider.
#[component]
pub fn Divider() -> impl IntoView {
    view! { <hr class="dodai-divider"/> }
}

/// CSS-only tooltip: shows `text` on hover/focus of the children.
#[component]
pub fn Tooltip(#[prop(into)] text: String, children: Children) -> impl IntoView {
    view! { <span class="dodai-tip" data-tip=text>{children()}</span> }
}

/// A busy spinner in any [`dodai_ui_core::SpinnerStyle`] (the plain ring by default).
#[component]
pub fn Spinner(
    #[prop(default = 24)] size: u32,
    #[prop(default = dodai_ui_core::SpinnerStyle::Ring)] style: dodai_ui_core::SpinnerStyle,
) -> impl IntoView {
    view! { <span inner_html=dodai_ui_topcoat::html::spinner_of(style, size)></span> }
}

/// A progress bar in a design's own style (hud, stripes, stones); `value` 0–1, or none for a running bar.
#[component]
pub fn StyledProgress(
    style: dodai_ui_core::ProgressStyle,
    #[prop(optional)] value: Option<f32>,
) -> impl IntoView {
    view! { <div inner_html=dodai_ui_topcoat::html::styled_progress(style, value)></div> }
}

/// Popup action list. `active` is the highlighted row; `on_active` is called when an arrow,
/// Home, End or hover moves it and `on_pick` when a row is picked (click, Enter or Space),
/// following the shared rule of `dodai_ui_core::menu`. The app updates `active`.
#[component]
pub fn Menu(
    rows: Vec<dodai_ui_core::menu::MenuRow>,
    #[prop(optional)] active: usize,
    #[prop(optional, into)] on_active: Option<Callback<usize>>,
    #[prop(optional, into)] on_pick: Option<Callback<usize>>,
) -> impl IntoView {
    let keys = rows.clone();
    let onkeydown = move |e: leptos::ev::KeyboardEvent| match dodai_ui_core::menu::step(
        &e.key(),
        &keys,
        active,
    ) {
        Some(dodai_ui_core::menu::Step::Move(i)) => {
            e.prevent_default();
            if let Some(cb) = on_active {
                cb.run(i)
            }
        }
        Some(dodai_ui_core::menu::Step::Pick(i)) => {
            e.prevent_default();
            if let Some(cb) = on_pick {
                cb.run(i)
            }
        }
        None => {}
    };
    view! {
        <div class="dodai-menu" role="menu" tabindex="0" on:keydown=onkeydown>
            {rows
                .into_iter()
                .enumerate()
                .map(|(i, r)| {
                    if r.separator {
                        return view! { <div class="dodai-menu__sep" role="separator"></div> }
                            .into_any();
                    }
                    let pickable = r.pickable();
                    let class = class::menu_item(&r, i == active);
                    view! {
                        <div
                            class=class
                            role="menuitem"
                            aria-disabled=r.disabled.to_string()
                            on:mouseenter=move |_| {
                                if let (true, Some(cb)) = (pickable, on_active) {
                                    cb.run(i)
                                }
                            }
                            on:click=move |_| {
                                if let (true, Some(cb)) = (pickable, on_pick) {
                                    cb.run(i)
                                }
                            }
                        >
                            <span class="dodai-menu__check">{if r.checked { "\u{2713}" } else { "" }}</span>
                            <span class="dodai-menu__label">{r.label}</span>
                            {r.shortcut.map(|k| view! { <span class="dodai-menu__key">{k}</span> })}
                        </div>
                    }
                        .into_any()
                })
                .collect_view()}
        </div>
    }
}
