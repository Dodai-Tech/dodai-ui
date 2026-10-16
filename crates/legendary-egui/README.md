# legendary-egui

[egui](https://github.com/emilk/egui) adapter for the Legendary UI design system.
`legendary_egui::install(ctx, &theme)` sets egui visuals/spacing/type sizes from
tokens; painter-drawn widgets draw chamfered / rounded / square shapes from
`Shape::outline`, glow from `Effects` (`Theme::with_glow_intensity`), hard shadow,
optional scanlines and uppercase labels. No theme-id branching.

Widgets: `Button` (all variants, disabled), `Panel`, `Tag`, `Tabs`, `Toggle`,
`Checkbox`, `Radio`, `ProgressBar`, `Slider`, `TextInput`, `ListItem`, `Divider`,
`tooltip`, text helpers (`LegendaryUi::lg_heading/lg_label/lg_muted/lg_mono`),
`paint_scanlines`, `ThemeController` (theme switcher + glow, `t`/`Shift+T`, `+`/`-`).

```rust
let mut ctl = legendary_egui::ThemeController::new();
ctl.install(ctx);          // every frame; no-op if unchanged
ctl.handle_keys(ctx);
```

Showcase: `cargo run -p legendary-egui --example egui_showcase`.
