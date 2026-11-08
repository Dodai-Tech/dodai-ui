# dodai-iced

Complete iced 0.14 component library driven only by `dodai_core::Theme`
tokens (shape kind, glow, hard shadow, typography, spacing) — no theme-id
branching. Buttons, panels, tags, tabs, toggler/checkbox/radio, progress,
slider, text input, pick list/menu, list items, divider, scrollable, tooltip,
text helpers and a scanline overlay. Chamfered shapes use canvas widgets;
rounded/square use native widgets.

```
cargo run -p dodai-iced --example iced_showcase   # t / T cycle themes
# The same pages as the gpui showcase, from dodai-showcase-model (charts, instruments and
# shaders are not ported). Env: DODAI_THEME, DODAI_CAT, DODAI_FILTER, DODAI_SHOT (PPM + exit).
```
