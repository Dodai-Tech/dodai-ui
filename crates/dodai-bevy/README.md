# dodai-bevy

[Bevy](https://bevy.org) 0.18 `bevy_ui` adapter for the **Dodai UI** design system.
All visuals derive from `dodai-core` tokens — shapes follow `ShapeKind`
(square / rounded / chamfered via `Shape::outline`-equivalent shader maths), glow
follows `Effects` (`Theme::with_glow_intensity`), text case follows
`Typography::uppercase_labels`. No branching on theme ids.

```rust
use bevy::prelude::*;
use dodai_bevy::prelude::*;

App::new()
    .add_plugins((DefaultPlugins, DodaiUiPlugin::default()))
    .add_systems(Startup, |mut c: Commands| {
        c.spawn(Camera2d);
        c.spawn(button("Jack in", ButtonVariant::Primary));
    })
    .run();
```

## API

* `DodaiUiPlugin { scanlines, switcher }` — add after `DefaultPlugins`.
* `DodaiTheme` resource — `cycle_next/prev`, `select(id)`, `select_index`,
  `set_glow_intensity`; derefs to the active `Theme`. Changing it re-skins every
  widget live. `theme_hotkeys` system: `t`/`T` theme, `+`/`-` glow.
* Spawn helpers (`impl Bundle`): `button`, `button_disabled`, `panel`, `panel_untitled`,
  `tag`, `tabs`/`tab`, `switch`, `checkbox`, `radio`, `toggle_disabled`, `progress`,
  `slider`, `text_input`, `list_item`, `divider`, `divider_vertical`, `text`,
  `heading`, `label`, `muted`, `mono`, `body`; `UiTooltip(String)` component.
* `ControlEvent { entity, kind }` message (`Pressed`, `Toggled`, `TabSelected`,
  `Selected`, `Slid`, `TextChanged`); `FocusedInput` resource.
* `Surface` — the themed box primitive; `surface::resolve` is the pure token →
  paint mapping; `ChamferMaterial` / `ScanlineMaterial` are `UiMaterial`s.

## Notes / limits

* Chamfered surfaces are drawn by a `UiMaterial` shader; the **glow** is a
  `BoxShadow`, i.e. rectangular (it does not follow the cut corners).
* The bundled default font (FiraMono subset) is used for every role; token font
  families, `label_tracking` and `hard_shadow` are not applied. It lacks some
  glyphs (e.g. `·`).
* The text input is minimal (append / backspace / click-to-focus, no caret movement).

```
CARGO_TARGET_DIR=target/bevy cargo run -p dodai-bevy --example bevy_showcase
```
