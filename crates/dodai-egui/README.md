# dodai-egui

[egui](https://github.com/emilk/egui) adapter for the Dodai UI design system.
`dodai_egui::install(ctx, &theme)` sets egui visuals/spacing/type sizes from
tokens; painter-drawn widgets draw chamfered / rounded / square shapes from
`Shape::outline`, glow from `Effects` (`Theme::with_glow_intensity`), hard shadow,
optional scanlines and uppercase labels. No theme-id branching.

Widgets: `Button` (all variants, disabled), `Panel`, `Tag`, `Tabs`, `Toggle`,
`Checkbox`, `Radio`, `ProgressBar`, `Slider`, `TextInput`, `ListItem`, `Divider`,
`tooltip`, text helpers (`DodaiUi::dodai_heading/dodai_label/dodai_muted/dodai_mono`),
`paint_scanlines`, `ThemeController` (theme switcher + glow, `t`/`Shift+T`, `+`/`-`).

```rust
let mut ctl = dodai_egui::ThemeController::new();
ctl.install(ctx);          // every frame; no-op if unchanged
ctl.handle_keys(ctx);
```

Showcase: `cargo run -p dodai-egui --example egui_showcase`.
