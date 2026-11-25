# Design spec (OpenPencil)

Dodai ships its design as data, not as a hand-drawn file. Everything below is generated from the
same sources the toolkits read, so the design and the code cannot drift silently. gpui is the
reference implementation; each component row in the spec links its gpui source.

## What is in the repo

| path | what | tracked |
| --- | --- | --- |
| `dist/design/<system>/<theme>.tokens.json` | tokens per design system (a folder) and theme (a file), W3C design-token format | yes |
| `dist/design/components.json` | catalog: every gpui component, its summary, whether it draws anything, and the toolkits that also have it (from `docs/PARITY.md`) | yes |
| `dist/design/motion.json` | the motion data every toolkit animates from: duration scale, sampled easing curves, entry effects, overscroll feels (`dodai-showcase-model --example motion`) | yes |
| `dist/design/spec/<system>.md` | written spec per system: colour roles, typography, spacing, shape, effects, motion, surfaces, sizes, look switches, navigation, button looks, surface families, charts, components | yes |
| `dist/design/dodai.fig` | design file for [OpenPencil](https://github.com/open-pencil/open-pencil) (also opens in Figma-compatible tools) | no, build it |
| `scripts/design.nu`, `scripts/design/build.js` | the generator | yes |

The four systems are Cyberpunk, Material, Neo-Brutalism and Dodai.

## The design file

One variable collection per system, one mode per theme (the design's own theme is the default
mode). Switching a mode in the editor recolours everything bound to variables. Pages per system:

| page | what it draws |
| --- | --- |
| Foundations | colour roles as swatches, type scale, spacing, radius, scalar tokens |
| Button | every variant and state |
| Surfaces | one component set per Kit family (the `kit/*` tokens gpui draws from) |
| Controls | Switch, Checkbox, Radio, Slider, Progress, Input, Tab (off/on) |
| Feedback | Badge, Alert, Chip, Card, Avatar, Snackbar, Dialog, with the recipes of `dodai-core/src/component.rs` |
| Components | a frame for every other component of the catalog (inputs, pickers, actions, navigation, data display, overlays, shells, effect layers, typography, the axis gizmo with its home, plus and minus buttons) and the hover, focus, error and disabled states of the interactive ones. Glyphs are icons of the design's own `dodai-icons` pack, not font characters |
| Charts | every chart of the showcase, drawn from the showcase model's plots |
| Icons | all 1769 icons of the design's pack (Stroked for Cyberpunk, Bold for Neo-Brutalism, Material Symbols for Material and Dodai), vectors bound to the text colour |
| Symbols | the 264 dashboard symbols by group (vehicle, medical, military, industrial, aviation) |
| Surfaces & glass | the design's frosted glass (tint, blur, saturation, edge) over the showcase backdrop at three blur radii, and every texture and effect the design owns, as the same vectors the web toolkits draw |
| Shaders | a thumbnail of all 199 shaders by family, rendered on the GPU by the code every toolkit runs |
| Motion | the duration scale, all 18 easing curves, every entry effect at five points and the overscroll feels, drawn from `motion.json` |
| Behaviour | the components that draw nothing themselves (`EntryScope`, `Follow`, `Interact`, `Lazy`, `Live`, `PageGrid`, `Feed`, `DodaiTheme`), each with what it does |

Plus one **Catalog** page listing each component and the toolkits that have it. Fills, strokes,
radii and sizes are bound to variables. Shapes baked from code (chart plots, textures, effects,
shader thumbnails, the glass backdrop) carry the colours of the design's own theme, which is why
that theme is the default mode.

## Regenerate

Needs `nu`, `bun` (runs `@open-pencil/cli`), a Rust toolchain and a GPU (the shader thumbnails
render on it). The generated inputs are not tracked: `dist/plots/` (charts), `dist/icons-design/`
(`dodai-icons --example design_icons`), `dist/surfaces/` (`dodai-showcase-model --example surfaces`)
and `dist/shaders/` (`dodai-showcase-model --example shaders`); `just design` writes them first.

```sh
just design                  # tokens export, catalog, .fig, spec, verify
nu scripts/design.nu catalog # dist/design/components.json
nu scripts/design.nu build   # dist/design/dodai.fig
nu scripts/design.nu spec    # dist/design/spec/<system>.md
nu scripts/design.nu verify  # .fig carries exactly the token variables
nu scripts/design.nu check   # catalog and spec are up to date (what CI runs)
```

Run `just design` and commit the result after changing a token, a component, or a toolkit's
coverage. CI fails on a stale catalog or spec.

## Keeping it aligned

- **Tokens** are the contract: `verify` fails when the `.fig` variable count differs from the token
  files, so a token added in core but not exported is caught.
- **Components**: add the gpui widget first, then its recipe to `scripts/design/build.js` (read
  variables only, never literal values). Toolkit coverage shows in `docs/PARITY.md` and the spec.
- **New design system or token group**: add it to the `groups` list in `scripts/design.nu` and to
  `scripts/design/build.js`.

## Coverage is enforced

`just design` fails when a catalog component has neither a frame on the Components page nor a row on
the Behaviour page, and when a frame uses an icon the design's pack does not have. So adding a gpui
component without drawing it (or listing it as behaviour in `scripts/lib/parity.nu`) breaks the build.

## Not covered yet

- Motion is drawn as curves and keyframe poses, not as clickable prototypes.
- The backdrop blur of the glass panes is a native effect: the editor shows it, but PNG exports
  of the file may not.
- Shaders are shown in their family's palette, not once per design palette.
