//! Renders every component to static HTML on stdout (`ServerRenderer`).
//! Theme/glow switching in the page is done by a small inline script over
//! `data-theme` / `data-glow` (pre-generated CSS), since SSR output is static.
//! Usage: `yew_showcase [theme-id] > out.html`
use dodai_core::{themes, ButtonVariant, Tone};
use dodai_topcoat::{multi_theme_stylesheet, theme_vars, GOOGLE_FONTS_URL};
use dodai_yew::*;
use yew::prelude::*;

#[function_component]
fn Showcase() -> Html {
    let tabs: Vec<AttrValue> = ["Overview", "Network", "Systems"]
        .map(AttrValue::from)
        .to_vec();
    html! {
    <ThemeProvider inject_css={false}>
      <main>
        <div class="dodai-row"><Heading level={1}>{"Dodai UI · Yew"}</Heading>
          <span class="dodai-label" id="tname"></span></div>
        <div class="dodai-row" id="themes">
          { for themes::all().iter().map(|t| html!{
            <button class="dodai-btn dodai-btn--ghost" data-set-theme={t.id()}>{ t.name() }</button> }) }
          <button class="dodai-btn dodai-btn--outline" data-glow="-1">{"Glow −"}</button>
          <span class="dodai-label" id="gval"></span>
          <button class="dodai-btn dodai-btn--outline" data-glow="1">{"Glow +"}</button>
          <span class="dodai-muted">{"keys: t / T"}</span>
        </div>
        <Tabs {tabs} active={0} />
        <section><Heading level={2}>{"Typography"}</Heading>
          <Heading level={1}>{"Heading 1"}</Heading><Heading level={3}>{"Heading 3"}</Heading>
          <Label>{"Label"}</Label><Muted>{"Muted text "}<Mono>{"mono"}</Mono></Muted></section>
        <section><Heading level={2}>{"Buttons"}</Heading><div class="dodai-row">
          <Button variant={ButtonVariant::Primary}>{"Primary"}</Button>
          <Button variant={ButtonVariant::Secondary}>{"Secondary"}</Button>
          <Button variant={ButtonVariant::Outline}>{"Outline"}</Button>
          <Button variant={ButtonVariant::OutlineSecondary}>{"Outline 2"}</Button>
          <Button variant={ButtonVariant::Danger}>{"Danger"}</Button>
          <Button variant={ButtonVariant::Ghost}>{"Ghost"}</Button>
          <Button disabled={true}>{"Disabled"}</Button></div></section>
        <section><Heading level={2}>{"Panels"}</Heading><div class="dodai-grid2">
          <Panel title="Panel"><Muted>{"Plain surface."}</Muted></Panel>
          <Panel title="Focused" focused={true}><Muted>{"Glow border."}</Muted></Panel></div></section>
        <section><Heading level={2}>{"Tags"}</Heading><div class="dodai-row">
          <Tag>{"info"}</Tag><Tag tone={Tone::Success}>{"online"}</Tag>
          <Tag tone={Tone::Warning}>{"warn"}</Tag><Tag tone={Tone::Danger}>{"breach"}</Tag>
          <Tag primary={true}>{"primary"}</Tag><Tag secondary={true}>{"secondary"}</Tag>
          <Tag soft={true}>{"soft"}</Tag><Tag soft={true} tone={Tone::Danger}>{"soft bad"}</Tag>
          <Tag badge={true} primary={true}>{"7"}</Tag></div></section>
        <section><Heading level={2}>{"Controls"}</Heading>
          <div class="dodai-grid2"><div><Label>{"Input"}</Label>
            <TextInput value="" placeholder="search the net…" />
            <Label>{"Disabled"}</Label><TextInput value="locked" disabled={true} /></div>
          <div><Label>{"Slider"}</Label><Slider value={65.0} />
            <Label>{"Progress"}</Label><Progress value={68.0} />
            <Progress value={40.0} tone={Tone::Success} /><Progress value={90.0} tone={Tone::Danger} /></div></div>
          <div class="dodai-row">
            <Toggle checked={true}>{"Switch on"}</Toggle><Toggle checked={false}>{"Switch off"}</Toggle>
            <Checkbox checked={true}>{"Checked"}</Checkbox><Checkbox checked={false}>{"Unchecked"}</Checkbox>
            <Radio name="r" checked={true}>{"Radio A"}</Radio><Radio name="r" checked={false}>{"Radio B"}</Radio>
            <Checkbox checked={false} disabled={true}>{"Disabled"}</Checkbox></div></section>
        <section><Heading level={2}>{"List, divider, tooltip"}</Heading><div class="dodai-grid2">
          <List><ListItem>{"Dashboard"}</ListItem><ListItem selected={true}>{"Selected item"}</ListItem>
            <ListItem>{"Settings"}</ListItem></List>
          <div><TooltipBubble>{"Static tooltip"}</TooltipBubble><Divider />
            <Tooltip text="CSS-only tooltip"><Button variant={ButtonVariant::Outline}>{"Hover me"}</Button></Tooltip></div>
        </div></section>
      </main>
    </ThemeProvider>
    }
}

const LAYOUT: &str = "main{max-width:960px;margin:0 auto;padding:var(--dodai-space-xl) var(--dodai-space-md);display:grid;gap:var(--dodai-space-lg)}\
.dodai-row{display:flex;gap:var(--dodai-space-md);flex-wrap:wrap;align-items:center}\
.dodai-grid2{display:grid;align-items:start;grid-template-columns:1fr 1fr;gap:var(--dodai-space-md)}";

const SCRIPT: &str = r#"(function(){var ids=IDS,g=2,i=0,d=document.documentElement;
function set(){d.setAttribute('data-theme',ids[i]);d.setAttribute('data-glow',g);
document.getElementById('tname').textContent=ids[i];document.getElementById('gval').textContent=(g/2).toFixed(1)}
var p=INIT;if(ids.indexOf(p)>=0)i=ids.indexOf(p);set();
document.addEventListener('click',function(e){var t=e.target.closest('button');if(!t)return;
if(t.dataset.setTheme){i=ids.indexOf(t.dataset.setTheme)}
if(t.dataset.glow){g=Math.max(0,Math.min(GMAX,g+ +t.dataset.glow))}set()});
document.addEventListener('keydown',function(e){if(e.key==='t')i=(i+1)%ids.length;else if(e.key==='T')i=(i+ids.length-1)%ids.length;else return;set()})})();"#;

const GMAX: u32 = 6; // glow steps of 0.5 → up to 3.0

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let all = themes::all();
    let init = std::env::args().nth(1).unwrap_or_default();
    let body = yew::ServerRenderer::<Showcase>::new().render().await;
    // Glow variants: `:root[data-theme=id][data-glow=n]` (n half-steps).
    let mut glow_css = String::new();
    for t in &all {
        for n in 0..=GMAX {
            let tv = t.clone().with_glow_intensity(n as f32 / 2.0);
            glow_css.push_str(&format!(
                ":root[data-theme=\"{}\"][data-glow=\"{n}\"]{{{}}}\n",
                t.id(),
                theme_vars(&tv)
            ));
        }
    }
    let ids = format!(
        "[{}]",
        all.iter()
            .map(|t| format!("\"{}\"", t.id()))
            .collect::<Vec<_>>()
            .join(",")
    );
    let script = SCRIPT
        .replace("IDS", &ids)
        .replace("INIT", &format!("\"{init}\""))
        .replace("GMAX", &GMAX.to_string());
    println!(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>Dodai UI · Yew</title>\
<link rel=\"stylesheet\" href=\"{GOOGLE_FONTS_URL}\"><style>{}{glow_css}{LAYOUT}</style></head><body>{body}<script>{script}</script></body></html>",
        multi_theme_stylesheet(&all)
    );
}
