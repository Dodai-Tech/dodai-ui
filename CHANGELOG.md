# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-11-12

The first release: the Dodai UI design system for gpui, iced, bevy and dioxus, with the web
layer, shaders and the shared crates under them. Charts, instruments, the ratatui adapter and the egui,
slint, yew, leptos and freya adapters stay in the repository but are not published yet.

### ✨ Crates
- `dodai-core`: tokens, `Theme`, `ThemeSwitcher`, component styles, WCAG contrast, `motion`,
  DTCG export, and the four designs behind features: Cyberpunk, Material 3, Neo-Brutalism, Dodai.
- `dodai-gpui`, `dodai-iced`, `dodai-bevy`, `dodai-dioxus`: themed component sets for each toolkit,
  all driven by the same tokens (no branching on theme ids).
- `dodai-topcoat`: CSS custom properties and `.dodai-*` classes, one stylesheet per design.
- `dodai-shaders`: 196 pixel shaders written in Rust, with views for gpui, iced, bevy and dioxus.
- `dodai-icons`, `dodai-fx`, `dodai-frost`: the icon packs, surface effects and frosted glass.

### ✨ Features
- Shapes: square, rounded and chamfered controls with glow and hard shadows; the shape tune
  (roundness, cut size, tick gap) and panel borders apply to every toolkit.
- Transparency and frosted glass on any component, resolved once in core (`Glass`, `Surface`).
- Surface effects (grain, halftone, scanlines, sheen, ...) and the busy-button shimmer.
- A shared showcase model (`dodai-showcase-model`) feeds the same pages to all four toolkits:
  Foundations, Shape, Surface effects, Actions, Data display, Navigation, Inputs, Loading,
  Animation and Shaders, with a sidebar that folds to an icon rail.
- bevy: a chamfer material that draws its hard shadow inside the cut outline.

### Known limits
- Mouse and keyboard behaviour has been checked on the gpui showcase only; the other three are
  checked by screenshots.
- bevy has no frosted-glass blur.
- Minimum Rust versions differ per crate; see each crate's `rust-version`.
