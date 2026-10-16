//! `cargo run -p legendary-topcoat --example html > showcase.html` then open it.
use legendary_core::themes;
use legendary_topcoat::{multi_theme_stylesheet, switcher_script, GOOGLE_FONTS_URL};

fn main() {
    let all = themes::all();
    println!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Legendary UI · Cyberpunk</title>
<link rel="stylesheet" href="{GOOGLE_FONTS_URL}">
<script>{js}</script>
<style>{css}
main{{max-width:960px;margin:0 auto;padding:var(--lg-space-xl) var(--lg-space-md);display:grid;gap:var(--lg-space-lg)}}
.row{{display:flex;gap:var(--lg-space-md);flex-wrap:wrap;align-items:center}}</style></head><body><main>
<div class="row"><h1 style="flex:1">Night City</h1>
<button class="lg-btn lg-btn--outline" data-lg-theme-prev>◀ Prev</button>
<button class="lg-btn lg-btn--primary" data-lg-theme-next>Next theme ▶</button></div>
<nav class="lg-tabs"><a class="lg-tab lg-tab--active" href="#">Overview</a><a class="lg-tab" href="#">Network</a><a class="lg-tab" href="#">Systems</a></nav>
<div class="lg-panel lg-panel--focus"><div class="lg-panel__body"><div class="lg-panel__title">Status</div>
<p class="lg-muted">Wake up, samurai. We have a city to burn.</p>
<div class="row"><span class="lg-tag lg-tag--success">online</span><span class="lg-tag lg-tag--warning">warn</span><span class="lg-tag lg-tag--danger">breach</span><span class="lg-tag">info</span></div></div></div>
<div class="row"><button class="lg-btn lg-btn--primary">Jack in</button><button class="lg-btn lg-btn--secondary">Hack</button><button class="lg-btn lg-btn--outline-secondary">Scan</button><button class="lg-btn lg-btn--danger">Abort</button><button class="lg-btn lg-btn--ghost">Ghost</button><button class="lg-btn" disabled>Disabled</button></div>
<input class="lg-input" placeholder="search the net…">
<div class="lg-progress"><span style="--v:68%"></span></div>
<table class="lg-table"><thead><tr><th>Corp</th><th>HQ</th><th>Threat</th></tr></thead><tbody>
<tr><td>Arasaka</td><td>Japantown</td><td><span class="lg-tag lg-tag--danger">high</span></td></tr>
<tr><td>Militech</td><td>Santo Domingo</td><td><span class="lg-tag lg-tag--warning">med</span></td></tr></tbody></table>
</main></body></html>"##,
        js = switcher_script(&all),
        css = multi_theme_stylesheet(&all)
    );
}
