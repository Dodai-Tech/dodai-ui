# dodai-leptos

Leptos 0.8 (SSR) adapter for Dodai UI. Components emit `lg-*` classes; theme,
shape (square/rounded/chamfered) and glow come from CSS variables generated from
theme tokens by `dodai-topcoat` — no theme-id branching.

- `ThemeProvider { themes, theme }` — `data-theme` scope + stylesheet
- `install_css(&themes)` / `install_css_with(&themes, &GLOW_LEVELS)` — multi-theme CSS, optional `data-glow` levels, scanlines (`data-scanlines="1"`)
- Components: `Button` (all `ButtonVariant` + `disabled`), `Panel`, `Tag`, `Tabs`, `Toggle`, `Checkbox`, `Radio`, `Slider`, `Progress`, `TextInput`, `List`/`ListItem`, `Divider`, `Tooltip`, `Heading`, `Label`, `Muted`, `Mono`
- `class` module: pure variant → class mapping

```sh
cargo run -p dodai-leptos --example leptos_showcase out.html [theme-id]
```
