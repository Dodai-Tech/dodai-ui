# dodai-cyberpunk-icons

Framework-agnostic icon geometry for Dodai UI: **136 icons** in 7 groups
(interface, arrows, actions, status, data, objects, cyber) on a 24×24 grid.

The pack follows the Cyberpunk design language: no curves, only straight lines,
45° chamfers and octagons, drawn with one uniform stroke.

- `IconName::shapes()` gives stroked polylines; adapters (e.g. `dodai-gpui`'s `Icon`) scale and stroke them in the theme colour.
- `IconName::name()` / `group()` / `from_name()` for galleries and lookups.
- `svg(icon, size, stroke)` and `sprite(stroke)` export SVG (`currentColor`) for non-Rust consumers; `just export` writes them to `dist/icons/`.

No assets, no dependencies.
