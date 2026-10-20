# dodai-motion

Toolkit-agnostic motion primitives for the Dodai UI design system. No dependencies, no
rendering: everything is a pure function of progress `0..1` (or of elapsed time for `Tween`).

- `Easing`: linear, ease-in/out, CSS cubic-bezier, damped springs, bounce, elastic, back, anticipate.
- `Curve`: composable curves (`window`, `reverse`, `ping_pong`, `then`, `pipe`, `stagger`).
- `EntryKind` / `EntrySet`: fade, rise, drop and slide effects, combinable, each with its own duration.
- `Motion`: duration + easing + delay + entry — what a design system hands to its components.
- `Tween`: a scalar that eases from where it is to a new target.

Design systems (`dodai-cyberpunk`, `dodai-material`) pick their own `Motion`; toolkit adapters
(`dodai-gpui`, …) turn it into their native animation API.
