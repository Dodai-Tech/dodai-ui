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

Every design system is bundled (`cyberpunk`, `material`, `neobrutalism`, `dodai` features). Also:

- **Shape tuning**: `ThemeController::tune` (`dodai_core::ShapeTune`): roundness, border width and
  colour, outline-all, hard shadow offset and direction.
- **Surface effects**: `ThemeController::surface` (`dodai_core::Surface`) draws textures and light
  effects over buttons, panels, tags and inputs; each design draws only what it owns. The
  geometry is `dodai-fx`'s, shared with every other toolkit (glitch, flicker and lit lines
  included).
- **Icons** (`icons` feature): `Icon::new(IconId::Home).size(24.0)` draws in the design's own pack.

Showcase: `cargo run -p dodai-egui --example egui_showcase`.
