# dodai-ui-iced

Complete [iced](https://iced.rs) 0.14 component library driven only by `dodai_ui_core::Theme`
tokens (shape kind, glow, hard shadow, typography, spacing), with no theme-id branching.
Buttons, panels, tags, tabs, toggler/checkbox/radio, progress, slider, text input, pick
list/menu, list items, divider, scrollable, tooltip, text helpers, frosted glass and a scanline
overlay. Chamfered shapes use canvas widgets; rounded and square shapes use the native widgets.

```toml
dodai-ui-iced = "0.1"   # default features: icons, frost
```

```rust,ignore
use dodai_ui_iced::IcedTheme;
use dodai_ui_core::ButtonVariant;

let t = IcedTheme::new(dodai_ui_core::themes::by_id("cyberpunk").unwrap());
let app_theme = t.iced_theme();                      // the iced::Theme for `application(...).theme(..)`
let ok = iced::widget::button("Continue")
    .style(t.button(ButtonVariant::Primary));        // native; use `t.button_view` for chamfered shapes
```

Features: `icons` (typed icons from `dodai-ui-icons`) and `frost` (frosted glass through
`dodai-ui-frost`), both on by default.

## Try it

From a checkout of the repository:

```sh
cargo run -p dodai-ui-iced --example iced_showcase   # t / T cycle themes
```

The showcase shows the same pages as the gpui one (built from `dodai-ui-showcase-model`).
Environment: `DODAI_THEME`, `DODAI_CAT`, `DODAI_FILTER`, `DODAI_SHOT` (writes a PPM and exits).

## Status

Version 0.1. The component set is checked against the shared showcase by screenshots; mouse
and keyboard behaviour is not covered on every control yet. Needs a recent stable Rust (see
`rust-version`). MIT licensed.
