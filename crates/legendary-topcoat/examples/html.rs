//! `cargo run -p legendary-topcoat --example html > showcase.html` then open it.
//! Every component plus a pure-CSS-attribute theme switcher (`data-theme`).
use legendary_core::themes;
use legendary_topcoat::{multi_theme_stylesheet, switcher_script, GOOGLE_FONTS_URL};

const BODY: &str = r##"<main>
<div class="row"><h1 style="flex:1">Legendary UI</h1>
<button class="lg-btn lg-btn--outline" data-lg-theme-prev>◀ Prev</button>
<span class="lg-label" id="tname"></span>
<button class="lg-btn lg-btn--primary" data-lg-theme-next>Next ▶</button></div>
<div class="row">THEME_BUTTONS</div>
<nav class="lg-tabs"><a class="lg-tab lg-tab--active" href="#">Overview</a><a class="lg-tab" href="#">Network</a><a class="lg-tab" href="#">Systems</a></nav>

<section><h2>Typography</h2><h1>Heading 1</h1><h2>Heading 2</h2><h3>Heading 3</h3><h4>Heading 4</h4>
<span class="lg-label">Label</span> <p class="lg-muted">Muted body text with <a href="#">a link</a> and <code>mono</code>.</p></section>

<section><h2>Buttons</h2><div class="row"><button class="lg-btn lg-btn--primary">Primary</button><button class="lg-btn lg-btn--secondary">Secondary</button><button class="lg-btn lg-btn--outline">Outline</button><button class="lg-btn lg-btn--outline-secondary">Outline 2</button><button class="lg-btn lg-btn--danger">Danger</button><button class="lg-btn lg-btn--ghost">Ghost</button><button class="lg-btn" disabled>Disabled</button></div></section>

<section><h2>Panels</h2><div class="grid2"><div class="lg-panel"><div class="lg-panel__title">Panel</div><p class="lg-muted">Plain surface panel.</p></div>
<div class="lg-panel lg-panel--focus"><div class="lg-panel__title">Focused</div><p class="lg-muted">Primary border with glow.</p></div></div></section>

<section><h2>Tags &amp; badges</h2><div class="row"><span class="lg-tag">info</span><span class="lg-tag lg-tag--success">online</span><span class="lg-tag lg-tag--warning">warn</span><span class="lg-tag lg-tag--danger">breach</span><span class="lg-tag lg-tag--primary">primary</span><span class="lg-tag lg-tag--secondary">secondary</span>
<span class="lg-tag lg-tag--soft">soft</span><span class="lg-tag lg-tag--soft lg-tag--success">soft ok</span><span class="lg-tag lg-tag--soft lg-tag--danger">soft bad</span><span class="lg-badge lg-tag--primary">7</span></div></section>

<section><h2>Form controls</h2><div class="grid2">
<div class="lg-field"><span class="lg-label">Input</span><input class="lg-input" placeholder="search the net…"></div>
<div class="lg-field"><span class="lg-label">Select</span><select class="lg-select"><option>Night City</option><option>Pacifica</option></select></div>
<div class="lg-field"><span class="lg-label">Textarea</span><textarea class="lg-textarea" placeholder="notes…"></textarea></div>
<div class="lg-field"><span class="lg-label">Disabled</span><input class="lg-input" disabled value="locked"></div></div>
<div class="row" style="margin-top:var(--lg-space-md)">
<label class="lg-switch"><input type="checkbox" checked><span class="lg-switch__box"></span>Switch on</label>
<label class="lg-switch"><input type="checkbox"><span class="lg-switch__box"></span>Switch off</label>
<label class="lg-check"><input type="checkbox" checked><span class="lg-check__box"></span>Checked</label>
<label class="lg-check"><input type="checkbox"><span class="lg-check__box"></span>Unchecked</label>
<label class="lg-radio"><input type="radio" name="r" checked><span class="lg-radio__box"></span>Radio A</label>
<label class="lg-radio"><input type="radio" name="r"><span class="lg-radio__box"></span>Radio B</label>
<label class="lg-check"><input type="checkbox" disabled><span class="lg-check__box"></span>Disabled</label></div>
<div class="grid2" style="margin-top:var(--lg-space-md)"><div class="lg-field"><span class="lg-label">Slider</span><input class="lg-slider" type="range" value="65" style="--v:65%"></div>
<div class="lg-field"><span class="lg-label">Progress</span><div class="lg-progress"><span style="--v:68%"></span></div><div class="lg-progress lg-progress--success"><span style="--v:40%"></span></div><div class="lg-progress lg-progress--danger"><span style="--v:90%"></span></div></div></div></section>

<section><h2>List, divider, tooltip</h2><div class="grid2"><ul class="lg-list"><li class="lg-list__item">Dashboard</li><li class="lg-list__item" aria-selected="true">Selected item</li><li class="lg-list__item">Settings</li></ul>
<div><span class="lg-tooltip">Static tooltip</span><hr class="lg-divider"><span class="lg-tip" data-tip="CSS-only tooltip"><button class="lg-btn lg-btn--outline">Hover me</button></span></div></div></section>

<section><h2>Table</h2><table class="lg-table"><thead><tr><th>Corp</th><th>HQ</th><th>Threat</th></tr></thead><tbody>
<tr><td>Arasaka</td><td>Japantown</td><td><span class="lg-tag lg-tag--danger">high</span></td></tr>
<tr><td>Militech</td><td>Santo Domingo</td><td><span class="lg-tag lg-tag--warning">med</span></td></tr></tbody></table></section>
</main>"##;

fn main() {
    let all = themes::all();
    // Optional: `html <theme-id>` pre-selects a theme (used for screenshots).
    let pre = std::env::args().nth(1).unwrap_or_default();
    let buttons: String = all
        .iter()
        .map(|t| {
            format!(
                r#"<button class="lg-btn lg-btn--ghost" onclick="legendaryTheme.set('{}')">{}</button>"#,
                t.id(),
                t.name()
            )
        })
        .collect();
    let body = BODY.replace("THEME_BUTTONS", &buttons);
    let init = if pre.is_empty() {
        String::new()
    } else {
        format!("legendaryTheme.set('{pre}');")
    };
    println!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Legendary UI · Topcoat</title>
<link rel="stylesheet" href="{GOOGLE_FONTS_URL}">
<style>{css}
main{{max-width:960px;margin:0 auto;padding:var(--lg-space-xl) var(--lg-space-md);display:grid;gap:var(--lg-space-lg)}}
.row{{display:flex;gap:var(--lg-space-md);flex-wrap:wrap;align-items:center}}
.grid2{{display:grid;align-items:start;grid-template-columns:1fr 1fr;gap:var(--lg-space-md)}}
section{{display:block}}</style></head><body>{body}
<script>{js}{init}
(function(){{var n=document.getElementById('tname');function u(){{n.textContent=legendaryTheme.current()}}u();document.addEventListener('click',function(){{setTimeout(u,0)}})}})();</script>
</body></html>"##,
        js = switcher_script(&all),
        css = multi_theme_stylesheet(&all)
    );
}
