# dodai-ui

One dependency for the [Dodai UI](https://github.com/Dodai-Tech/dodai-ui) design systems:
Cyberpunk, Material 3, Neo-Brutalism and Dodai, across iced, gpui, egui, bevy, slint, dioxus,
leptos, yew, tui (ratatui) and topcoat. Everything beyond the engine is a feature.

```toml
dodai-ui = { version = "0.1", features = ["iced", "cyberpunk", "material"] }
```

```rust
use dodai_ui::core::ThemeSwitcher;
use dodai_ui::iced::IcedTheme;
```

Features: designs `dodai` (default), `cyberpunk`, `material`, `neobrutalism`, `all-designs`;
blocks `icons`, `charts`, `dash`, `fx`, `frost`, `shaders`; toolkits `iced`, `gpui`, `egui`,
`bevy`, `slint`, `dioxus`, `leptos`, `yew`, `tui`, `topcoat`.

Freya and xilem are separate workspaces: depend on `dodai-freya` / `dodai-xilem` directly.
