# dodai-freya

[Freya](https://github.com/marc2332/freya) (0.3.x, Rust-native GUI on Dioxus + Skia) adapter for the Dodai UI design system. Everything is driven by `dodai-core` tokens; no code branches on a theme id.

## Layers

| Layer | Needs Freya | What |
|---|---|---|
| `style` | no | token → Freya attribute strings (`css`, `ShapeStyle::{background,border,corner_radius,shadow}`, `chamfer_svg`, `text_style`, per-component `*_style`). Unit-tested without a window. |
| `theme`, `components` (feature `ui`, default) | yes | `use_init_dodai`, `Dodai` handle (switcher + glow override), `Dodai*` components. |

Build only the pure layer: `cargo test -p dodai-freya --no-default-features`.

## Shapes & effects

* `ShapeKind::Rounded` → `corner_radius`; `Square` → plain rect; `Chamfered` → absolutely positioned `svg` polygon from `Shape::outline(w,h)` behind the content (explicit pixel `width`/`height` required; estimated from label for buttons/tags/tabs).
* Glow = Freya `shadow` (rect) or blurred SVG polygon (chamfered), from `Effects::glow_color/glow_radius`; `hard_shadow` = offset shadow. `uppercase_labels`/`label_tracking` honoured by headings and labels (never mono).

## API

```rust
let dodai = use_init_dodai(Theme::default()); // root
dodai.next(); dodai.prev(); dodai.select("cyberpunk-cyan"); dodai.select_index(i);
dodai.set_glow_intensity(1.5); dodai.theme();
```
Components: `DodaiButton` (all `ButtonVariant`s, `disabled`), `DodaiPanel`, `DodaiTag`, `DodaiTabs`, `DodaiToggle`, `DodaiCheckbox`, `DodaiRadio`, `DodaiProgress`, `DodaiSlider`, `DodaiTextInput`, `DodaiListItem`, `DodaiDivider`, `DodaiTooltip`, `DodaiTooltipHost`, `DodaiHeading`/`DodaiLabel`/`DodaiMuted`/`DodaiMono`, `DodaiThemeSwitcher`. `shaped()` is the box primitive.

## Showcase

`cargo run -p dodai-freya --example freya_showcase` — `t`/`T` switch theme, `+`/`-` glow, theme buttons, glow buttons.

## Limitations

* `DodaiTextInput` / `DodaiSlider` wrap Freya's `Input`/`Slider`; chamfered themes render the input square.
* Chamfer glow is a blurred SVG polygon (relies on Skia SVG filters).
* No scanline overlay component yet.
