# dodai-topcoat

## Shape & glow variables

Components never branch on theme id. Per theme, `theme_vars` emits `--lg-radius`,
`--lg-clip` / `--lg-clip-sm` / `--lg-clip-inner` / `--lg-clip-ring` (a
`clip-path: polygon(…)` generated from `Shape::outline` for chamfered themes,
`none` for rounded/square), and `--lg-glow-shadow` / `--lg-glow-filter`
(box-shadow for rounded/square, `drop-shadow` for chamfered, since `clip-path`
clips shadows). `multi_theme_stylesheet(&themes::all())` emits a
`:root[data-theme="<id>"]` block for every theme; switch with
`document.documentElement.dataset.theme = id`.

Components: `.lg-btn--{primary,secondary,outline,outline-secondary,danger,ghost}`,
`.lg-panel(--focus,--raised)`, `.lg-tag(--soft,--<tone>)`, `.lg-badge`, `.lg-tabs/.lg-tab`,
`.lg-switch/.lg-check/.lg-radio`, `.lg-progress`, `.lg-slider`, `.lg-input/.lg-textarea/.lg-select`,
`.lg-list`, `.lg-divider`, `.lg-tooltip`/`.lg-tip[data-tip]`, `.lg-label`, `.lg-table`, scanlines (`--lg-scanlines`).

`cargo run -p dodai-topcoat --example html [theme-id] > showcase.html`
