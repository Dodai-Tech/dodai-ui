# dodai-yew

Yew 0.21 adapter for the Dodai UI design system. Typed function components
over the token-derived CSS of `dodai-topcoat`.

Components: `ThemeProvider`, `ThemeSwitcherBar`, `Button` (all `ButtonVariant`),
`Panel`, `Tag` (tag/badge), `Tabs`, `Toggle`, `Checkbox`, `Radio`, `Progress`,
`Slider`, `TextInput`, `List`/`ListItem`, `Divider`, `Tooltip`, `Heading`,
`Label`, `Muted`, `Mono`.

```rust
html! { <ThemeProvider theme_id="night-city"><Button>{"Go"}</Button></ThemeProvider> }
```

Chamfer/rounded/square, glow (`Theme::with_glow_intensity`), scanlines and
`uppercase_labels` all derive from tokens. Showcase (static HTML via
`ServerRenderer`, with theme buttons and glow +/-, `t`/`T` keys):

    cargo run -p dodai-yew --example yew_showcase > .shots/yew.html
