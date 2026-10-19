# dodai-material-icons

The full [Material Symbols](https://fonts.google.com/icons) catalogue as
**dependency-free vector data**: 3,900+ icons, each with a `-fill` variant, in
the three official styles. Self-contained: no font, no I/O, no dependencies.

```toml
dodai-material-icons = "0.1"                              # outlined (default)
dodai-material-icons = { version = "0.1", features = ["rounded"] }
dodai-material-icons = { version = "0.1", features = ["all"] }
```

```rust
use dodai_material_icons::outlined;

let home = outlined::get("home").unwrap();
let svg = home.svg();            // standalone SVG, fill = currentColor
let polys = home.polygons();     // flattened 24×24 polygons (non-zero fill)
let on = outlined::get("favorite-fill");
```

Styles are Cargo features so you compile only what you use (`outlined`,
`rounded`, `sharp`). Weight 400, 24 px optical size.

Regenerate from upstream with `scripts/generate.py` (see the header).

Icon artwork © Google, licensed under the Apache License 2.0
(`LICENSE-APACHE`). Part of [Dodai UI](https://github.com/sorinirimies/dodai-ui).
