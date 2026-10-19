//! `cargo run -p dodai-topcoat --example html > showcase.html` then open it.
//! Every component plus a pure-CSS-attribute theme switcher (`data-theme`).
use dodai_core::themes;
use dodai_topcoat::{multi_theme_stylesheet, switcher_script, GOOGLE_FONTS_URL};

const BODY: &str = r##"<main>
<div class="row"><h1 style="flex:1">Dodai UI</h1>
<button class="dodai-btn dodai-btn--outline" data-dodai-theme-prev>◀ Prev</button>
<span class="dodai-label" id="tname"></span>
<button class="dodai-btn dodai-btn--primary" data-dodai-theme-next>Next ▶</button></div>
<div class="row">THEME_BUTTONS</div>
<nav class="dodai-tabs"><a class="dodai-tab dodai-tab--active" href="#">Overview</a><a class="dodai-tab" href="#">Network</a><a class="dodai-tab" href="#">Systems</a></nav>

<section><h2>Typography</h2><h1>Heading 1</h1><h2>Heading 2</h2><h3>Heading 3</h3><h4>Heading 4</h4>
<span class="dodai-label">Label</span> <p class="dodai-muted">Muted body text with <a href="#">a link</a> and <code>mono</code>.</p></section>

<section><h2>Buttons</h2><div class="row"><button class="dodai-btn dodai-btn--primary">Primary</button><button class="dodai-btn dodai-btn--secondary">Secondary</button><button class="dodai-btn dodai-btn--outline">Outline</button><button class="dodai-btn dodai-btn--outline-secondary">Outline 2</button><button class="dodai-btn dodai-btn--danger">Danger</button><button class="dodai-btn dodai-btn--ghost">Ghost</button><button class="dodai-btn" disabled>Disabled</button></div></section>

<section><h2>Panels</h2><div class="grid2"><div class="dodai-panel"><div class="dodai-panel__title">Panel</div><p class="dodai-muted">Plain surface panel.</p></div>
<div class="dodai-panel dodai-panel--focus"><div class="dodai-panel__title">Focused</div><p class="dodai-muted">Primary border with glow.</p></div></div></section>

<section><h2>Tags &amp; badges</h2><div class="row"><span class="dodai-tag">info</span><span class="dodai-tag dodai-tag--success">online</span><span class="dodai-tag dodai-tag--warning">warn</span><span class="dodai-tag dodai-tag--danger">error</span><span class="dodai-tag dodai-tag--primary">primary</span><span class="dodai-tag dodai-tag--secondary">secondary</span>
<span class="dodai-tag dodai-tag--soft">soft</span><span class="dodai-tag dodai-tag--soft dodai-tag--success">soft ok</span><span class="dodai-tag dodai-tag--soft dodai-tag--danger">soft bad</span><span class="dodai-badge dodai-tag--primary">7</span></div></section>

<section><h2>Form controls</h2><div class="grid2">
<div class="dodai-field"><span class="dodai-label">Input</span><input class="dodai-input" placeholder="search the net…"></div>
<div class="dodai-field"><span class="dodai-label">Select</span><select class="dodai-select"><option>Yellow</option><option>Pacifica</option></select></div>
<div class="dodai-field"><span class="dodai-label">Textarea</span><textarea class="dodai-textarea" placeholder="notes…"></textarea></div>
<div class="dodai-field"><span class="dodai-label">Disabled</span><input class="dodai-input" disabled value="locked"></div></div>
<div class="row" style="margin-top:var(--dodai-space-md)">
<label class="dodai-switch"><input type="checkbox" checked><span class="dodai-switch__box"></span>Switch on</label>
<label class="dodai-switch"><input type="checkbox"><span class="dodai-switch__box"></span>Switch off</label>
<label class="dodai-check"><input type="checkbox" checked><span class="dodai-check__box"></span>Checked</label>
<label class="dodai-check"><input type="checkbox"><span class="dodai-check__box"></span>Unchecked</label>
<label class="dodai-radio"><input type="radio" name="r" checked><span class="dodai-radio__box"></span>Radio A</label>
<label class="dodai-radio"><input type="radio" name="r"><span class="dodai-radio__box"></span>Radio B</label>
<label class="dodai-check"><input type="checkbox" disabled><span class="dodai-check__box"></span>Disabled</label></div>
<div class="grid2" style="margin-top:var(--dodai-space-md)"><div class="dodai-field"><span class="dodai-label">Slider</span><input class="dodai-slider" type="range" value="65" style="--v:65%"></div>
<div class="dodai-field"><span class="dodai-label">Progress</span><div class="dodai-progress"><span style="--v:68%"></span></div><div class="dodai-progress dodai-progress--success"><span style="--v:40%"></span></div><div class="dodai-progress dodai-progress--danger"><span style="--v:90%"></span></div></div></div></section>

<section><h2>List, divider, tooltip</h2><div class="grid2"><ul class="dodai-list"><li class="dodai-list__item">Dashboard</li><li class="dodai-list__item" aria-selected="true">Selected item</li><li class="dodai-list__item">Settings</li></ul>
<div><span class="dodai-tooltip">Static tooltip</span><hr class="dodai-divider"><span class="dodai-tip" data-tip="CSS-only tooltip"><button class="dodai-btn dodai-btn--outline">Hover me</button></span></div></div></section>

<section><h2>Table</h2><table class="dodai-table"><thead><tr><th>Vendor</th><th>HQ</th><th>Threat</th></tr></thead><tbody>
<tr><td>Acme</td><td>Berlin</td><td><span class="dodai-tag dodai-tag--danger">high</span></td></tr>
<tr><td>Globex</td><td>Lisbon</td><td><span class="dodai-tag dodai-tag--warning">med</span></td></tr></tbody></table></section>
</main>"##;

fn main() {
    let all = themes::all();
    // Optional: `html <theme-id>` pre-selects a theme (used for screenshots).
    let pre = std::env::args().nth(1).unwrap_or_default();
    let buttons: String = all
        .iter()
        .map(|t| {
            format!(
                r#"<button class="dodai-btn dodai-btn--ghost" onclick="dodaiTheme.set('{}')">{}</button>"#,
                t.id(),
                t.name()
            )
        })
        .collect();
    let body = BODY.replace("THEME_BUTTONS", &buttons);
    let init = if pre.is_empty() {
        String::new()
    } else {
        format!("dodaiTheme.set('{pre}');")
    };
    println!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Dodai UI · Topcoat</title>
<link rel="stylesheet" href="{GOOGLE_FONTS_URL}">
<style>{css}
main{{max-width:960px;margin:0 auto;padding:var(--dodai-space-xl) var(--dodai-space-md);display:grid;gap:var(--dodai-space-lg)}}
.row{{display:flex;gap:var(--dodai-space-md);flex-wrap:wrap;align-items:center}}
.grid2{{display:grid;align-items:start;grid-template-columns:1fr 1fr;gap:var(--dodai-space-md)}}
section{{display:block}}</style></head><body>{body}
<script>{js}{init}
(function(){{var n=document.getElementById('tname');function u(){{n.textContent=dodaiTheme.current()}}u();document.addEventListener('click',function(){{setTimeout(u,0)}})}})();</script>
</body></html>"##,
        js = switcher_script(&all),
        css = multi_theme_stylesheet(&all)
    );
}
