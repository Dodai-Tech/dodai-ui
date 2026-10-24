# dodai-aurora

**Aurora** is a design system for [Dodai UI](https://github.com/sorinirimies/dodai-ui) built on frosted glass and gradients: deep indigo ink on a pearl canvas (or lavender-white on deep night), 16 px controls, 30 px containers, hairline borders, tinted soft shadows and a violet-through-blue-to-teal gradient as its signature accent.

```rust
let all = dodai_aurora::variants(); // light + dark
let t = dodai_aurora::light();
```

Depends only on `dodai-core`. Uses the Material Symbols icons and the `Inter` font when installed.

## Tokens

`dodai_aurora::tokens` holds the complete token set: palette (light/dark with alpha ramps and frosted fills), type scale, spacing, radii, blur, shadows, motion, the page grid, the media fade and the aurora gradient (`AURORA_LIGHT`, `AURORA_DARK`). The theme is built from it.

Frosted glass is rendered by `dodai_gpui::FrostedScene`: `gpui` cannot blur what is already drawn, so the scene owns its picture and each panel shows a blurred copy of it under the frosted tint.
