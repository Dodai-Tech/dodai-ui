# Design spec (OpenPencil)

Dodai ships its design as data, not as a hand-drawn file. Everything below is generated from the
same sources the toolkits read, so the design and the code cannot drift silently. gpui is the
reference implementation; each component row in the spec links its gpui source.

## What is in the repo

| path | what | tracked |
| --- | --- | --- |
| `dist/design/<system>/<theme>.tokens.json` | tokens per design system (a folder) and theme (a file), W3C design-token format | yes |
| `dist/design/components.json` | catalog: every gpui component and the toolkits that also have it (from `docs/PARITY.md`) | yes |
| `dist/design/spec/<system>.md` | written spec per system: colour roles, typography, spacing, shape, effects, motion, surfaces, sizes, look switches, navigation, button looks, surface families, charts, components | yes |
| `dist/design/dodai.fig` | design file for [OpenPencil](https://github.com/open-pencil/open-pencil) (also opens in Figma-compatible tools) | no, build it |
| `scripts/design.nu`, `scripts/design/build.js` | the generator | yes |

The four systems are Cyberpunk, Material, Neo-Brutalism and Dodai.

## The design file

One variable collection per system, one mode per theme: switching a mode in the editor recolours
that system's components. Pages per system:

- **Foundations**: colour roles as swatches, type scale, spacing, radius, scalar tokens.
- **Button**: every variant and state.
- **Surfaces**: the surface families.
- **Controls**: Switch, Checkbox, Radio, Slider, Progress, Input, Tab (off/on variants).
- **Feedback**: Badge, Alert, Chip, Card, Avatar, Snackbar, Dialog, using the same recipes as gpui
  (`badge`, `alert`, `chip`, `card`, `snackbar`, `dialog` in `dodai-core/src/component.rs`).
- **Components**: about 60 more showcase components (inputs, actions, navigation, data display,
  overlays), each built only from role, size and shape variables.
- **Charts**: every chart of the showcase, drawn from the showcase model's plots.

Plus one **Catalog** page listing each component and the toolkits that have it. Fills, strokes,
radii and sizes are bound to variables; chart shapes carry the default theme's resolved colours.

## Regenerate

Needs `nu`, `bun` (runs `@open-pencil/cli`) and a Rust toolchain.

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

## Not covered yet

The file draws roughly 80 of the 128 catalogued components, with one static state each (no hover,
focus or disabled variants outside Button and Controls). The rest, the date/time pickers, DataTable,
Command palette, shells and the effect layers, are listed in the spec and the Catalog page but not
drawn. Spacing and motion are
documented as tokens only.
