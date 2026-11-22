# dodai-ui-xilem

[xilem](https://github.com/linebender/xilem) (masonry widgets, vello renderer) adapter for the
Dodai UI design system. **Scaffold:** colour conversion and a themed `button` view; the rest of
the components follow the order in `docs/PARITY.md`.

```rust
let theme = dodai_ui_core::Theme::default();
let view = dodai_ui_xilem::button(&theme, "Continue", ButtonVariant::Primary, |state: &mut State| {});
```

Nothing here looks at a theme id: colours, radius, border width and sizes come from tokens.
