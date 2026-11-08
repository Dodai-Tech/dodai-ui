# dodai-dioxus

## Showcase

```
cargo run -p dodai-dioxus --example dioxus_showcase      # http://127.0.0.1:3030 (DODAI_PORT)
cargo run -p dodai-dioxus --example dioxus_showcase -- --html out.html   # one static page
```

The same pages and controls as the gpui showcase, built from `dodai-showcase-model`. The live
app runs in this process and talks to the browser over a websocket (dioxus-liveview): the
desktop renderer needs `cocoa` 0.26.1, which gpui pins below in the same workspace. `DODAI_THEME`,
`DODAI_CAT` and `DODAI_FILTER` choose the first page. Components beyond the first set (`Card`,
`Alert`, `Chip`, `Avatar`, `Steps`, `Pagination`, `TreeView`, `DataTable`, `Timeline`, `Rating`,
`Stepper`, `Spinner`, ...) take their look from `dodai_core::Kit` through the stylesheet.
