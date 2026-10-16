# dodai-topcoat

## Shape & glow variables

Components never branch on theme id. Per theme, `theme_vars` emits `--dodai-radius`,
`--dodai-clip` / `--dodai-clip-sm` / `--dodai-clip-inner` / `--dodai-clip-ring` (a
`clip-path: polygon(…)` generated from `Shape::outline` for chamfered themes,
`none` for rounded/square), and `--dodai-glow-shadow` / `--dodai-glow-filter`
(box-shadow for rounded/square, `drop-shadow` for chamfered, since `clip-path`
clips shadows). `multi_theme_stylesheet(&themes::all())` emits a
`:root[data-theme="<id>"]` block for every theme; switch with
`document.documentElement.dataset.theme = id`.

Components: `.dodai-btn--{primary,secondary,outline,outline-secondary,danger,ghost}`,
`.dodai-panel(--focus,--raised)`, `.dodai-tag(--soft,--<tone>)`, `.dodai-badge`, `.dodai-tabs/.dodai-tab`,
`.dodai-switch/.dodai-check/.dodai-radio`, `.dodai-progress`, `.dodai-slider`, `.dodai-input/.dodai-textarea/.dodai-select`,
`.dodai-list`, `.dodai-divider`, `.dodai-tooltip`/`.dodai-tip[data-tip]`, `.dodai-label`, `.dodai-table`, scanlines (`--dodai-scanlines`).

`cargo run -p dodai-topcoat --example html [theme-id] > showcase.html`
