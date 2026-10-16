//! Renders every component to a static HTML file via SSR.
//! `cargo run -p dodai-leptos --example leptos_showcase [out.html] [theme-id]`
//! Theme buttons, `t`/`T` keys and glow +/- are wired with a tiny inline script.
use dodai_core::{themes, ButtonVariant, Tone};
use dodai_leptos::class::TagTone;
use dodai_leptos::*;
use dodai_topcoat::{switcher_script, GOOGLE_FONTS_URL};
use leptos::prelude::*;

#[component]
fn Showcase(names: Vec<(String, String)>) -> impl IntoView {
    let tabs = vec!["Overview".to_string(), "Network".into(), "Systems".into()];
    view! {
        <main>
            <div class="row">
                <h1 style="flex:1">"Dodai UI · Leptos"</h1>
                <Label>"Glow"</Label>
                <button class="dodai-btn dodai-btn--outline" data-glow-dec="">"−"</button>
                <span class="dodai-label" id="gval"></span>
                <button class="dodai-btn dodai-btn--outline" data-glow-inc="">"+"</button>
            </div>
            <div class="row">
                {names
                    .into_iter()
                    .map(|(id, n)| view! { <button class="dodai-btn dodai-btn--ghost" data-set-theme=id>{n}</button> })
                    .collect_view()}
                <Label>"(t / T cycle)"</Label>
            </div>
            <Tabs tabs=tabs active=0/>
            <section>
                <Heading level=2>"Typography"</Heading>
                <Heading level=1>"Heading 1"</Heading>
                <Heading level=3>"Heading 3"</Heading>
                <Heading level=4>"Heading 4"</Heading>
                <Label>"Label"</Label>
                <Muted>"Muted text"</Muted>
                <Mono>"mono text"</Mono>
            </section>
            <section>
                <Heading>"Buttons"</Heading>
                <div class="row">
                    <Button variant=ButtonVariant::Primary>"Primary"</Button>
                    <Button variant=ButtonVariant::Secondary>"Secondary"</Button>
                    <Button variant=ButtonVariant::Outline>"Outline"</Button>
                    <Button variant=ButtonVariant::OutlineSecondary>"Outline 2"</Button>
                    <Button variant=ButtonVariant::Danger>"Danger"</Button>
                    <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                    <Button disabled=true>"Disabled"</Button>
                </div>
            </section>
            <section>
                <Heading>"Panels"</Heading>
                <div class="grid2">
                    <Panel title="Panel">"Plain surface."</Panel>
                    <Panel title="Focused" focused=true>"Primary border + glow."</Panel>
                </div>
            </section>
            <section>
                <Heading>"Tags & badges"</Heading>
                <div class="row">
                    <Tag>"info"</Tag>
                    <Tag tone=TagTone::Success>"online"</Tag>
                    <Tag tone=Tone::Warning.into()>"warn"</Tag>
                    <Tag tone=TagTone::Danger>"breach"</Tag>
                    <Tag tone=TagTone::Primary soft=true>"soft"</Tag>
                    <Tag tone=TagTone::Secondary badge=true>"7"</Tag>
                </div>
            </section>
            <section>
                <Heading>"Form controls"</Heading>
                <div class="grid2">
                    <TextInput placeholder="search the net…"/>
                    <TextInput value="locked" disabled=true/>
                </div>
                <div class="row">
                    <Toggle checked=true>"On"</Toggle>
                    <Toggle>"Off"</Toggle>
                    <Checkbox checked=true>"Checked"</Checkbox>
                    <Checkbox>"Unchecked"</Checkbox>
                    <Checkbox disabled=true>"Disabled"</Checkbox>
                    <Radio name="r" checked=true>"A"</Radio>
                    <Radio name="r">"B"</Radio>
                </div>
                <div class="grid2">
                    <Slider value=65.0/>
                    <div>
                        <Progress value=68.0/>
                        <Progress value=40.0 tone=Tone::Success/>
                        <Progress value=90.0 tone=Tone::Danger/>
                    </div>
                </div>
            </section>
            <section>
                <Heading>"List, divider, tooltip"</Heading>
                <div class="grid2">
                    <List>
                        <ListItem>"Dashboard"</ListItem>
                        <ListItem selected=true>"Selected"</ListItem>
                        <ListItem>"Settings"</ListItem>
                    </List>
                    <div>
                        <Divider/>
                        <Tooltip text="CSS-only tooltip"><Button variant=ButtonVariant::Outline>"Hover me"</Button></Tooltip>
                    </div>
                </div>
            </section>
        </main>
    }
}

const GLOW_JS: &str = "(function(){var L=[0,0.5,1,1.5,2],g=2,d=document.documentElement;\
function sg(n){g=Math.max(0,Math.min(L.length-1,n));d.setAttribute('data-glow',String(L[g]));document.getElementById('gval').textContent=L[g]}\
sg(2);document.addEventListener('click',function(e){var t=e.target.closest('[data-set-theme],[data-glow-inc],[data-glow-dec]');if(!t)return;\
if(t.hasAttribute('data-glow-inc'))sg(g+1);else if(t.hasAttribute('data-glow-dec'))sg(g-1);else dodaiTheme.set(t.getAttribute('data-set-theme'))});\
document.addEventListener('keydown',function(e){if(e.key==='t')dodaiTheme.next();else if(e.key==='T')dodaiTheme.prev()})})();";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = args
        .get(1)
        .cloned()
        .unwrap_or("leptos_showcase.html".into());
    let pre = args.get(2).cloned().unwrap_or_default();
    let all = themes::all();
    let names = all
        .iter()
        .map(|t| (t.id().to_string(), t.name().to_string()))
        .collect();
    let css = install_css_with(&all, &GLOW_LEVELS);
    let body = view! { <Showcase names=names/> }.to_html();
    let init = if pre.is_empty() {
        String::new()
    } else {
        format!("dodaiTheme.set('{pre}');")
    };
    let html = format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Dodai UI · Leptos</title>
<link rel="stylesheet" href="{GOOGLE_FONTS_URL}"><style>{css}
main{{max-width:960px;margin:0 auto;padding:var(--dodai-space-xl) var(--dodai-space-md);display:grid;gap:var(--dodai-space-lg)}}
.row{{display:flex;gap:var(--dodai-space-md);flex-wrap:wrap;align-items:center}}
.grid2{{display:grid;align-items:start;grid-template-columns:1fr 1fr;gap:var(--dodai-space-md)}}</style></head>
<body>{body}<script>{js}{init}{GLOW_JS}</script></body></html>"#,
        js = switcher_script(&all)
    );
    std::fs::write(&out, html).expect("write html");
    println!("wrote {out}");
}
