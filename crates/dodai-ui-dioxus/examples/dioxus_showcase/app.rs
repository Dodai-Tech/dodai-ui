//! The window around the sections: sidebar, control bar, scrolling page.

use dioxus::prelude::*;
use dodai_ui_core::{ButtonVariant, ControlSize, Kit};
use dodai_ui_dioxus::*;
use dodai_ui_showcase_model::{Action, Ctx, Showcase, CATEGORIES, SPEEDS};

use crate::render::{self, Cx};

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
            Field { name: "Direction", Segmented { labels: owned(vec!["LTR", "RTL"]), selected: usize::from(rtl), onselect: move |i| set(Action::Rtl(i == 1)) } }
            Field { name: "Patterns", Segmented { labels: off_on(), selected: usize::from(patterns), onselect: move |i| set(Action::Patterns(i == 1)) } }
        }
    }
}

#[component]
fn NavItem(
    label: String,
    count: usize,
    on: bool,
    page: bool,
    listed: bool,
    onclick: EventHandler<()>,
) -> Element {
    let icon = dodai_ui_showcase_model::catalog::category_icon(&label);
    // A page shows whether its sections are listed under it.
    let chevron = if listed {
        dodai_ui_icons::IconId::ChevronDown
    } else {
        dodai_ui_icons::IconId::ChevronRight
    };
    rsx! {
        KitBox { kit: Kit::Row { selected: on }, style: "width:100%;gap:8px;align-items:center;".to_string(),
            onclick: move |_| onclick.call(()),
            Icon { icon, size: 16 }
            span { class: "dodai-label", style: "color:inherit;flex:1;text-align:start", "{label}" }
            span { class: "dodai-caption", "{count}" }
            if page {
                span { class: "dodai-caption", Icon { icon: chevron, size: 14 } }
            } else {
                span { style: "width:14px" }
            }
        }
    }
}

/// A section entry under the open page: picking it scrolls the page to it.
#[component]
fn SubItem(label: String, on: bool, onclick: EventHandler<()>) -> Element {
    rsx! {
        div { style: "padding-left:16px;width:100%",
            KitBox { kit: Kit::Row { selected: on }, style: "width:100%;".to_string(),
                onclick: move |_| onclick.call(()),
                span { class: "dodai-label", style: "color:inherit;font-size:var(--dodai-size-xs)", "{label}" }
            }
        }
    }
}

/// Scroll the section named `title` to the top of the page.
fn scroll_to(title: &str) {
    let id = format!("sec-{title}");
    let _ = document::eval(&format!(
        "document.getElementById({id:?})?.scrollIntoView({{behavior:'smooth',block:'start'}})"
    ));
}

/// The logo beside the sidebar title: 1.6 × the theme's xl text size.
const LOGO_STYLE: &str =
    "width:calc(var(--dodai-size-xl) * 1.6);height:calc(var(--dodai-size-xl) * 1.6);flex:none";

#[component]
fn Sidebar(model: Signal<Showcase>) -> Element {
    let m = model.read();
    let mut set = move |a: Action| model.write().update(a);
    let cat = m.current_category();
    let listed_titles: Vec<Vec<&'static str>> = (0..CATEGORIES.len())
        .map(|i| m.listed(i).iter().map(|s| s.title).collect())
        .collect();
    let active = m.active_section();
    let counts: Vec<usize> = std::iter::once(m.count(None))
        .chain((0..CATEGORIES.len()).map(|c| m.count(Some(c))))
        .collect();
    let filter = m.filter().to_string();
    let rail = m.rail();
    let listed: Vec<bool> = (0..CATEGORIES.len()).map(|i| m.lists(i)).collect();
    drop(m);
    let fold = rsx! {
        Button { variant: ButtonVariant::Ghost, size: Some(ControlSize::Small),
            onclick: move |_| set(Action::Press(Showcase::rail_toggle())),
            if rail { "»" } else { "«" }
        }
    };
    if rail {
        return rsx! {
            aside { class: "dodai-side", style: "width:72px;min-width:72px;padding:var(--dodai-space-xs);align-items:center",
                {fold}
                for (i, name) in std::iter::once("All").chain(CATEGORIES.iter().copied()).enumerate() {
                    KitBox { key: "{name}", kit: Kit::Row { selected: if i == 0 { cat.is_none() } else { cat == Some(i - 1) } },
                        style: "width:100%;justify-content:center".to_string(),
                        onclick: move |_| set(Action::Category(i.checked_sub(1))),
                        Icon { icon: dodai_ui_showcase_model::catalog::category_icon(name), size: 20 }
                    }
                }
            }
        };
    }
    rsx! {
        aside { class: "dodai-side",
            div { style: "display:flex;align-items:center;justify-content:space-between",
                div { style: "display:flex;align-items:center;gap:var(--dodai-space-sm)",
                    img { src: dodai_ui_showcase_model::media::logo_data_url(), alt: "", style: "{LOGO_STYLE}" }
                    Heading { level: 2, "Dodai UI" }
                }
                {fold}
            }
            TextInput { value: filter, placeholder: "Filter components…".to_string(),
                oninput: move |s| set(Action::Filter(s)) }
            nav { class: "dodai-side__nav",
                NavItem { label: "All".to_string(), count: counts[0], on: cat.is_none(), page: false, listed: false, onclick: move |_| set(Action::Category(None)) }
                for (i, c) in CATEGORIES.iter().enumerate() {
                    NavItem { key: "{c}", label: c.to_string(), count: counts[i + 1], on: cat == Some(i), page: true, listed: listed[i],
                        onclick: move |_| set(Action::Category(Some(i))) }
                    if listed[i] {
                        for t in listed_titles[i].iter().copied() {
                            SubItem { key: "{t}", label: t.to_string(), on: active == Some(t),
                                onclick: move |_| { set(Action::Section(t)); scroll_to(t) } }
                        }
                    }
                }
            }
            span { class: "dodai-caption", "{dodai_ui_showcase_model::catalog::KEYS_HINT}" }
        }
    }
}

#[component]
fn Page(model: Signal<Showcase>) -> Element {
    // Live widgets (clock, timer, stopwatch, beat lights) redraw the page on a timer.
    let mut tick = use_signal(|| 0u64);
    let _ = tick();
    use_future(move || async move {
        loop {
            let every = model.peek().redraw_every();
            tokio::time::sleep(every.unwrap_or(std::time::Duration::from_millis(250))).await;
            if every.is_some() {
                tick += 1;
            }
        }
    });
    let m = model.read();
    let theme = m.theme();
    let title = m.category().map_or("All", |c| CATEGORIES[c]);
    let ctx = Ctx {
        theme: &theme,
        values: &m.values,
    };
    let sections: Vec<(&'static str, &'static str, dodai_ui_showcase_model::Node)> = m
        .visible()
        .iter()
        .map(|s| (s.title, s.heading, (s.build)(&ctx)))
        .collect();
    // The app-wide surface of the Surface effects page paints every panel (drawn for a
    // nominal panel size and stretched over the real one).
    let fx = m.resolved_surface();
    let bg = dodai_ui_dioxus::surface_background(
        &theme,
        fx,
        dodai_ui_core::SurfaceClass::Panel,
        (900.0, 420.0),
    )
    .map_or(String::new(), |b| {
        format!("background-image:{b};background-size:100% 100%;border-radius:var(--dodai-radius)")
    });
    let overview = m.overview("dioxus");
    drop(m);
    let cx = Cx { model };
    rsx! {
        main { class: "dodai-page",
            Heading { level: 1, "{title}" }
            if let Some(o) = overview { {render::view(&o, cx)} }
            if sections.is_empty() { Muted { "No components match" } }
            for (t, heading, node) in sections {
                div { key: "{t}", id: "sec-{t}", "data-sec": "{t}", style: "position:relative",
                    Panel { title: heading.to_string(), {render::view(&node, cx)} }
                    if !bg.is_empty() { div { style: "position:absolute;inset:0;pointer-events:none;{bg}" } }
                }
            }
        }
    }
}

/// The live preview pinned above the Shape and Surface-effects pages.
#[component]
fn Pinned(model: Signal<Showcase>) -> Element {
    let Some(node) = model.read().pinned() else {
        return rsx! {};
    };
    rsx! {
        div { style: "padding:var(--dodai-space-md) var(--dodai-space-lg);border-bottom:1px solid var(--dodai-border);flex:none",
            {render::view(&node, Cx { model })}
        }
    }
}

/// Hands the Surface effects page's mix to the provider, so the components draw it.
#[component]
fn SurfaceEffects(model: Signal<Showcase>) -> Element {
    let surface = model.read().surface();
    let mut h = use_theme();
    if h.surface() != surface {
        h.set_surface(surface);
    }
    rsx! {}
}

/// The showcase.
#[component]
pub fn App() -> Element {
    let model = use_signal(Showcase::from_env);
    let theme = model.read().theme();
    let id = theme.id().to_string();
    // Follow the page's scroll with the shared rule (`Showcase::follow_scroll`): the last section
    // that reached the top, the last one at the end of the page, a picked one until the user
    // scrolls.
    use_future(move || async move {
        let mut spy = document::eval(
            r#"const page = () => document.querySelector('.dodai-page');
            const user = () => dioxus.send(['user', false, []]);
            for (const ev of ['wheel', 'touchmove', 'keydown']) addEventListener(ev, user, {capture: true, passive: true});
            addEventListener('scroll', () => {
                const p = page();
                if (!p) return;
                const top = p.getBoundingClientRect().top;
                const tops = [...document.querySelectorAll('[data-sec]')]
                    .map(e => [e.dataset.sec, e.getBoundingClientRect().top - top]);
                const end = p.scrollHeight - p.clientHeight > 1 && p.scrollTop >= p.scrollHeight - p.clientHeight - 1;
                dioxus.send([p.scrollTop < 1 ? 'top' : 'scroll', end, tops]);
            }, true);"#,
        );
        let mut model = model;
        while let Ok((kind, end, tops)) = spy.recv::<(String, bool, Vec<(String, f64)>)>().await {
            if kind == "user" {
                // Only a held pick needs releasing: skip the write (and the redraw) otherwise.
                if model.peek().held_section().is_some() {
                    model.write().update(Action::Unpin);
                }
                continue;
            }
            let at_top = kind == "top";
            if model.peek().at_top() != at_top {
                model.write().set_at_top(at_top);
            }
            let titles: Vec<&'static str> =
                model.peek().visible().iter().map(|s| s.title).collect();
            let tops: Vec<(&'static str, f32)> = tops
                .iter()
                .filter_map(|(t, y)| Some((*titles.iter().find(|x| **x == t.as_str())?, *y as f32)))
                .collect();
            let now = {
                let m = model.peek();
                dodai_ui_core::spy::pick(&tops, m.held_section(), 0.0, end, 60.0)
                    .filter(|t| m.active_section() != Some(*t))
            };
            if let Some(t) = now {
                model.write().update(Action::Visible(t));
            }
        }
    });
    // The frost demo's picture goes behind everything, fixed to the window.
    let wallpaper = if model.read().wallpaper() {
        format!(
            "background:url({}) center/cover fixed",
            dodai_ui_showcase_model::media::wallpaper_url(&theme)
        )
    } else {
        String::new()
    };
    rsx! {
        ThemeProvider { themes: Some(vec![theme]), initial: Some(id),
            SurfaceEffects { model }
            div { class: "dodai-app", style: wallpaper,
                Sidebar { model }
                div { class: "dodai-main", TopBar { model } Pinned { model } Page { model } }
            }
        }
    }
}
