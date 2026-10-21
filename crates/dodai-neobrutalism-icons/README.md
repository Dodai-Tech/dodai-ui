# dodai-neobrutalism-icons

Bold, chunky icons for the Dodai Neo-Brutalism design system, as vector data.
Artwork is [Phosphor Icons](https://phosphoricons.com) (MIT): the `bold` weight is the
outline icon, the `fill` weight its `-fill` variant. Names follow Dodai / Material spelling.

```rust
let home = dodai_neobrutalism_icons::get("home").unwrap();
let polys = home.polygons(); // 24×24 polygons, fill with the non-zero rule
```

`scripts/generate.py` rebuilds `src/bold.rs` from a Phosphor checkout; `scripts/names.py`
maps Dodai names to Phosphor names. Licence: MIT, © Phosphor Icons (see `LICENSE-MIT`).
