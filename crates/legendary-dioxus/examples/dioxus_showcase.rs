//! Showcase of every legendary-dioxus component, server-rendered to static HTML.
//!
//! `cargo run -p legendary-dioxus --example dioxus_showcase -- [out.html] [theme-id] [glow]`
//! Defaults: `.shots/dioxus_showcase.html`, first theme, glow 1.0. The page has a button
//! per theme and `t`/`T` keys (tiny inline script switching `data-theme`); glow is fixed
//! at render time (pass e.g. `1.5`), as SSR has no runtime state.
//! The same `App` works unchanged in any Dioxus renderer (desktop/web) where signals are live.

use dioxus::prelude::*;
use legendary_core::{themes, ButtonVariant, Tone};
use legendary_dioxus::*;

#[derive(Props, Clone, PartialEq)]
struct AppProps {
    initial: String,
    glow: f32,
}

#[component]
fn App(props: AppProps) -> Element {
    rsx! {
        ThemeProvider { initial: props.initial, glow: props.glow,
            Showcase {}
        }
    }
}

#[component]
fn Showcase() -> Element {
    let mut theme = use_theme();
    let mut tab = use_signal(|| 0usize);
    let mut on = use_signal(|| true);
    let mut sel = use_signal(|| 1usize);
    let mut vol = use_signal(|| 0.6f32);
    let mut name = use_signal(|| String::from("V"));
    rsx! {
        div { style: "padding:24px;display:grid;gap:16px;max-width:980px;margin:auto",
            Heading { level: 1, "Legendary UI · Dioxus" }
            Muted { "Active theme: {theme.id()} · glow {theme.glow():.1}" }
            Panel { title: "Theme",
                div { style: "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
                    for (id, n) in theme.list() {
                        button { key: "{id}", class: "lg-btn lg-btn--outline", "data-theme-btn": id,
                            onclick: move |_| { theme.select(id); }, "{n}" }
                    }
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| theme.prev(), "◀ prev (T)" }
                    Button { variant: ButtonVariant::Ghost, onclick: move |_| theme.next(), "next (t) ▶" }
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| { let g = theme.glow() - 0.25; theme.set_glow(g) }, "Glow −" }
                    Button { variant: ButtonVariant::Secondary, onclick: move |_| { let g = theme.glow() + 0.25; theme.set_glow(g) }, "Glow +" }
                }
            }
            Panel { title: "Buttons", focus: true,
                div { style: "display:flex;gap:8px;flex-wrap:wrap",
                    Button { "Primary" }
                    Button { variant: ButtonVariant::Secondary, "Secondary" }
                    Button { variant: ButtonVariant::Outline, "Outline" }
                    Button { variant: ButtonVariant::OutlineSecondary, "Outline 2" }
                    Button { variant: ButtonVariant::Danger, "Danger" }
                    Button { variant: ButtonVariant::Ghost, "Ghost" }
                    Button { disabled: true, "Disabled" }
                }
            }
            Panel { title: "Tags & badges",
                div { style: "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
                    Tag { kind: TagKind::Primary, "primary" }
                    Tag { kind: TagKind::Secondary, "secondary" }
                    Tag { kind: Tone::Info.into(), "info" }
                    Tag { kind: Tone::Success.into(), "success" }
                    Tag { kind: Tone::Warning.into(), soft: true, "warning" }
                    Tag { kind: Tone::Danger.into(), soft: true, "danger" }
                    Badge { "7" }
                }
            }
            Panel { title: "Tabs",
                Tabs { tabs: vec!["Overview".into(), "Stats".into(), "Logs".into()], active: tab(), onselect: move |i| tab.set(i) }
                p { "Tab {tab()} content" }
            }
            Panel { title: "Controls", raised: true,
                div { style: "display:grid;gap:12px",
                    Toggle { checked: on(), label: "Toggle", onchange: move |v| on.set(v) }
                    Checkbox { checked: true, label: "Checkbox" }
                    Checkbox { checked: false, disabled: true, label: "Disabled" }
                    div { style: "display:flex;gap:16px",
                        for i in 0..3usize {
                            Radio { key: "{i}", name: "r", checked: sel() == i, label: format!("Option {i}"), onselect: move |_| sel.set(i) }
                        }
                    }
                    Slider { value: vol(), onchange: move |v| vol.set(v) }
                    Progress { value: vol() }
                    Progress { value: 0.4, tone: Tone::Success }
                    Progress { value: 0.7, tone: Tone::Warning }
                    Progress { value: 0.9, tone: Tone::Danger }
                    TextInput { value: name(), placeholder: "Handle", oninput: move |v| name.set(v) }
                    TextInput { value: String::new(), placeholder: "Disabled", disabled: true }
                }
            }
            Panel { title: "List",
                List {
                    for i in 0..3usize {
                        ListItem { key: "{i}", selected: sel() == i, onclick: move |_| sel.set(i), "Row {i}" }
                    }
                }
            }
            Panel { title: "Text",
                Heading { level: 2, "Heading two" }
                Heading { level: 3, "Heading three" }
                div { Label { "label" } }
                div { Muted { "muted text" } }
                div { Mono { "mono_text()" } }
                Divider {}
                Tooltip { tip: "Tooltip bubble", Button { variant: ButtonVariant::Outline, "Hover me" } }
            }
        }
    }
}

fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let out = a
        .first()
        .cloned()
        .unwrap_or(".shots/dioxus_showcase.html".into());
    let initial = a
        .get(1)
        .cloned()
        .unwrap_or_else(|| themes::all()[0].id().into());
    let glow = a.get(2).and_then(|g| g.parse().ok()).unwrap_or(1.0);
    let mut vdom = VirtualDom::new_with_props(App, AppProps { initial, glow });
    vdom.rebuild_in_place();
    let body = dioxus_ssr::render(&vdom);
    let ids: Vec<String> = themes::all()
        .iter()
        .map(|t| format!("\"{}\"", t.id()))
        .collect();
    let script = format!(
        "var ids=[{}],r=document.querySelector('.lg-root'),i=Math.max(0,ids.indexOf(r.dataset.theme));\
function s(n){{i=(n+ids.length)%ids.length;r.dataset.theme=ids[i]}}\
document.querySelectorAll('[data-theme-btn]').forEach(function(b){{b.onclick=function(){{s(ids.indexOf(b.dataset.themeBtn))}}}});\
addEventListener('keydown',function(e){{if(e.key=='t')s(i+1);if(e.key=='T')s(i-1)}});",
        ids.join(",")
    );
    let html = format!(
        "<!doctype html><html><head><meta charset=utf-8><title>legendary-dioxus</title></head><body style=\"margin:0\">{body}<script>{script}</script></body></html>"
    );
    if let Some(p) = std::path::Path::new(&out).parent() {
        let _ = std::fs::create_dir_all(p);
    }
    std::fs::write(&out, html).expect("write html");
    println!("wrote {out}");
}
