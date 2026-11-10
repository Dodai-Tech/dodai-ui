# dodai-dioxus

[Dioxus](https://dioxuslabs.com) 0.7 components for the Dodai UI design system. The look comes
from `dodai-topcoat` (CSS variables and `.dodai-*` classes generated from `dodai_core::Theme`),
so the same stylesheet works on the web, in SSR and in liveview.

```toml
dodai-dioxus = "0.1"   # default features: cyberpunk, material, neobrutalism, dodai
```

```rust,ignore
use dioxus::prelude::*;
use dodai_dioxus::*;

fn app() -> Element {
    rsx! {
        ThemeProvider {
            Panel { Button { "Continue" } }
        }
    }
}
```

`ThemeProvider` installs the stylesheet, a `data-theme` root element and a `ThemeHandle`
context to switch themes with. The component set covers buttons, panels, inputs, tabs, lists,
tables, tags, cards, steppers, trees, ratings, pickers, an axis gizmo and a knob. Components
beyond the first set take their look from `dodai_core::Kit` through the stylesheet.

## Try it

From a checkout of the repository:

```sh
cargo run -p dodai-dioxus --example dioxus_showcase                     # http://127.0.0.1:3030 (DODAI_PORT)
cargo run -p dodai-dioxus --example dioxus_showcase -- --html out.html  # one static page
```

The showcase shows the same pages as the gpui one, built from `dodai-showcase-model`. It runs
as liveview in the example's own process (the desktop renderer needs a `cocoa` version that gpui
pins lower in the same workspace). `DODAI_THEME`, `DODAI_CAT` and `DODAI_FILTER` choose the first page.

## Status

Version 0.1. The component set is checked against the shared showcase by screenshots; mouse
and keyboard behaviour is not covered on every control yet. Needs a recent stable Rust (see
`rust-version`). MIT licensed.
