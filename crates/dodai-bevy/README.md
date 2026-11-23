# dodai-bevy

[Bevy](https://bevy.org) 0.20 `bevy_ui` adapter for the **Dodai UI** design system.
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
        c.spawn(button("Continue", ButtonVariant::Primary));
    })
    .run();
```

## API

* `DodaiUiPlugin { scanlines, switcher }` — add after `DefaultPlugins`.
* `DodaiTheme` resource — `cycle_next/prev`, `select(id)`, `select_index`,
  `set_glow_intensity`; derefs to the active `Theme`. Changing it re-skins every
  widget live. `theme_hotkeys` system: `t`/`T` theme, `+`/`-` glow.
* Designs: Cyberpunk, Material, Neo-Brutalism and Dodai (features of the same names).
* Spawn helpers (`impl Bundle`): `button`, `button_disabled`, `panel`, `panel_untitled`,
  `tag`, `tabs`/`tab`, `switch`, `checkbox`, `radio`, `toggle_disabled`, `progress`,
  `slider`, `text_input`, `list_item`, `divider`, `divider_vertical`, `text`,
  `heading`, `label`, `muted`, `mono`, `body`; `UiTooltip(String)` component.
* `icon(IconId, px)` and `symbol(Symbol, px)` / `symbol_state(Symbol, px, SymbolState)`: vector icons and the 264 dashboard symbols, rasterised once per design and size and tinted by the foreground or the state (feature `symbols`). Each design strokes the symbols its own way.
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

The showcase draws the same pages and sections as the gpui one from `dodai-showcase-model`
(Foundations, Actions, Data display, Navigation, Inputs, Loading; charts, instruments and
shaders are not ported). Env: `DODAI_THEME`, `DODAI_CAT`, `DODAI_OPEN`, `DODAI_SCROLL`,
`DODAI_SHOT`.

Alongside the first widgets, `UiKit` boxes (`Kit::Card`, `Alert`, `Chip`, `Avatar`, `Code`,
`Tooltip`, `Row`, `Popup`, `Select`, ...) and the busy indicators `spinner()` / `indeterminate()` are
token-styled and re-skin with the theme; `ButtonSpec` builds sized, square, icon and icon-only buttons.

## Status

Version 0.1. The components are checked against the shared showcase by screenshots; mouse and keyboard behaviour is not covered on every control yet. Needs a recent stable Rust (see `rust-version`). MIT licensed.
