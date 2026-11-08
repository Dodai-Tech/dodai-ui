//! The window around the sections: sidebar, control bar, scrolling page.

use dioxus::prelude::*;
use dodai_core::{ButtonVariant, ControlSize, Kit};
use dodai_dioxus::*;
use dodai_showcase_model::{Action, Ctx, ModePref, Showcase, CATEGORIES, SPEEDS};

use crate::render::{self, Cx};

/// A one-row picker of mutually exclusive choices.
#[component]
fn Segmented(labels: Vec<String>, selected: usize, onselect: EventHandler<usize>) -> Element {
    rsx! {
        div { class: "dodai-seg",
            for (i, l) in labels.into_iter().enumerate() {
                Button { key: "{i}", size: Some(ControlSize::Small),
                    variant: if i == selected { ButtonVariant::Primary } else { ButtonVariant::Ghost },
                    onclick: move |_| onselect.call(i), "{l}" }
            }
        }
    }
}

/// A native dropdown over `items`.
#[component]
fn Dropdown(items: Vec<String>, selected: usize, onselect: EventHandler<usize>) -> Element {
    rsx! {
        select { class: "dodai-select",
            onchange: move |e| if let Ok(i) = e.value().parse::<usize>() { onselect.call(i) },
            for (i, it) in items.into_iter().enumerate() {
                option { key: "{i}", value: "{i}", selected: i == selected, "{it}" }
            }
        }
    }
}

#[component]
fn Field(name: String, children: Element) -> Element {
    rsx! { div { class: "dodai-topbar__field", span { class: "dodai-caption", "{name}" } {children} } }
}

fn owned(v: Vec<&str>) -> Vec<String> {
    v.into_iter().map(String::from).collect()
}

/// The labels of an Off/On picker.
pub fn off_on() -> Vec<String> {
    vec!["Off".into(), "On".into()]
}

#[component]
fn TopBar(model: Signal<Showcase>) -> Element {
    let m = model.read();
    let colours = m.colours();
    let colour_at = colours.iter().position(|x| *x == m.colour()).unwrap_or(0);
    let colour_labels: Vec<String> = colours.iter().map(|x| x.label().to_string()).collect();
    let modes = ModePref::ALL;
    let mode_labels: Vec<String> = modes.iter().map(|x| x.label().to_string()).collect();
    let mode_at = modes.iter().position(|x| *x == m.mode()).unwrap_or(0);
    let speeds: Vec<String> = SPEEDS.iter().map(|s| format!("{s}×")).collect();
    let (animate, speed, rtl, patterns) = (m.animate, m.speed_index(), m.rtl, m.patterns);
    let (designs, variants) = (owned(m.design_names()), owned(m.variants()));
    let (design_at, variant_at) = (m.design_index(), m.variant_index());
    drop(m);
    let mut set = move |a: Action| model.write().update(a);
    rsx! {
        div { class: "dodai-topbar",
            Field { name: "Design", Dropdown { items: designs, selected: design_at, onselect: move |i| set(Action::Design(i)) } }
            Field { name: "Theme", Dropdown { items: variants, selected: variant_at, onselect: move |i| set(Action::Variant(i)) } }
            Field { name: "Colour", Segmented { labels: colour_labels, selected: colour_at, onselect: { let c = colours.clone(); move |i| set(Action::Colour(c[i])) } } }
            Field { name: "Animate", Segmented { labels: off_on(), selected: usize::from(animate), onselect: move |i| set(Action::Animate(i == 1)) } }
            Field { name: "Speed", Segmented { labels: speeds, selected: speed, onselect: move |i| set(Action::Speed(i)) } }
            Field { name: "Mode", Segmented { labels: mode_labels, selected: mode_at, onselect: move |i| set(Action::Mode(modes[i])) } }
            Field { name: "Direction", Segmented { labels: owned(vec!["LTR", "RTL"]), selected: usize::from(rtl), onselect: move |i| set(Action::Rtl(i == 1)) } }
            Field { name: "Patterns", Segmented { labels: off_on(), selected: usize::from(patterns), onselect: move |i| set(Action::Patterns(i == 1)) } }
        }
    }
}

#[component]
fn NavItem(label: String, count: usize, on: bool, onclick: EventHandler<()>) -> Element {
    rsx! {
        KitBox { kit: Kit::Row { selected: on }, style: "width:100%;justify-content:space-between;".to_string(),
            onclick: move |_| onclick.call(()),
            span { class: "dodai-label", style: "color:inherit", "{label}" }
            span { class: "dodai-caption", "{count}" }
        }
    }
}

#[component]
fn Sidebar(model: Signal<Showcase>) -> Element {
    let m = model.read();
    let mut set = move |a: Action| model.write().update(a);
    let cat = m.category();
    let titles: Vec<&'static str> = m.visible().iter().map(|s| s.title).collect();
    let counts: Vec<usize> = std::iter::once(m.count(None))
        .chain((0..CATEGORIES.len()).map(|c| m.count(Some(c))))
        .collect();
    let filter = m.filter().to_string();
    drop(m);
    rsx! {
        aside { class: "dodai-side",
            Heading { level: 2, "Dodai UI" }
            TextInput { value: filter, placeholder: "Filter components…".to_string(),
                oninput: move |s| set(Action::Filter(s)) }
            nav { class: "dodai-side__nav",
                NavItem { label: "All".to_string(), count: counts[0], on: cat.is_none(), onclick: move |_| set(Action::Category(None)) }
                for (i, c) in CATEGORIES.iter().enumerate() {
                    NavItem { key: "{c}", label: c.to_string(), count: counts[i + 1], on: cat == Some(i),
                        onclick: move |_| set(Action::Category(Some(i))) }
                    if cat == Some(i) {
                        for t in titles.iter() {
                            div { key: "{t}", class: "dodai-navsub dodai-caption", "{t}" }
                        }
                    }
                }
            }
            span { class: "dodai-caption", "click a page" }
        }
    }
}

#[component]
fn Page(model: Signal<Showcase>) -> Element {
    let m = model.read();
    let theme = m.theme();
    let title = m.category().map_or("All", |c| CATEGORIES[c]);
    let ctx = Ctx {
        theme: &theme,
        values: &m.values,
    };
    let sections: Vec<(&'static str, dodai_showcase_model::Node)> = m
        .visible()
        .iter()
        .map(|s| (s.title, (s.build)(&ctx)))
        .collect();
    drop(m);
    let cx = Cx { model };
    rsx! {
        main { class: "dodai-page",
            Heading { level: 1, "{title}" }
            if sections.is_empty() { Muted { "No components match" } }
            for (t, node) in sections {
                Panel { key: "{t}", title: t.to_string(), {render::view(&node, cx)} }
            }
        }
    }
}

/// The showcase.
#[component]
pub fn App() -> Element {
    let model = use_signal(Showcase::from_env);
    let (theme, rev) = {
        let m = model.read();
        (m.theme(), m.revision())
    };
    let id = theme.id().to_string();
    rsx! {
        ThemeProvider { key: "{rev}", themes: Some(vec![theme]), initial: Some(id),
            div { class: "dodai-app",
                Sidebar { model }
                div { class: "dodai-main", TopBar { model } Page { model } }
            }
        }
    }
}
