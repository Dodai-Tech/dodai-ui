//! Content and composition: Markdown, pickers and palettes, theme selectors, scrolling and
//! layout containers, and the glow, frost and scanline layers.

use crate::{KitBox, SelectionMarker};
use dioxus::prelude::*;
use dodai_ui_core::markdown::{self as md, Align, MdBlock, MdInline};
use dodai_ui_core::Kit;
use dodai_ui_icons::IconId;

fn inline(src: &str) -> Element {
    let (text, spans) = md::parse_inline(src);
    let mut out: Vec<Element> = Vec::new();
    let mut at = 0;
    let slice = |a: usize, b: usize| text.get(a..b).unwrap_or_default().to_string();
    for (r, kind) in spans {
        if r.start > at {
            out.push(rsx! { "{slice(at, r.start)}" });
        }
        let s = slice(r.start, r.end);
        out.push(match kind {
            MdInline::Bold => rsx! { strong { "{s}" } },
            MdInline::Italic => rsx! { em { "{s}" } },
            MdInline::Code => {
                rsx! { code { style: "font-family:var(--dodai-font-mono);background:var(--dodai-surface-raised);padding:0 .3em;border-radius:var(--dodai-radius)", "{s}" } }
            }
            MdInline::Link => rsx! { a { style: "color:var(--dodai-link, var(--dodai-primary))", "{s}" } },
        });
        at = r.end;
    }
    if at < text.len() {
        out.push(rsx! { "{slice(at, text.len())}" });
    }
    rsx! { {out.into_iter()} }
}

/// Renders Markdown (headings, paragraphs, lists, tables, quotes, code, rules) with the
/// design's fonts and colours, from the parser shared by every toolkit.
#[component]
pub fn Markdown(source: String) -> Element {
    let blocks = md::parse_blocks(&source);
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-sm);color:var(--dodai-text)",
            for (i, b) in blocks.into_iter().enumerate() {
                match b {
                    MdBlock::Heading(level, t) => rsx! {
                        div { key: "{i}", role: "heading", "aria-level": "{level}",
                            style: "font-family:var(--dodai-font-display);font-size:var(--dodai-size-{heading_size(level)})", {inline(&t)} }
                    },
                    MdBlock::Paragraph(t) => rsx! { p { key: "{i}", style: "margin:0", {inline(&t)} } },
                    MdBlock::Quote(t) => rsx! {
                        blockquote { key: "{i}", style: "margin:0;padding-left:var(--dodai-space-md);border-left:3px solid var(--dodai-border-strong);color:var(--dodai-text-muted)", {inline(&t)} }
                    },
                    MdBlock::Code { text, .. } => rsx! { crate::CodeBlock { key: "{i}", code: text } },
                    MdBlock::Rule => rsx! { hr { key: "{i}", style: "width:100%;border:0;border-top:var(--dodai-border-width) solid var(--dodai-border)" } },
                    MdBlock::List(items) => {
                        let marks = md::list_markers(&items);
                        rsx! {
                            div { key: "{i}", style: "display:flex;flex-direction:column;gap:2px",
                                for (n, it) in items.iter().enumerate() {
                                    div { key: "{n}", style: "display:flex;gap:var(--dodai-space-xs);padding-left:calc({it.depth} * var(--dodai-space-md))",
                                        span { style: "color:var(--dodai-text-muted)", "{marks[n]}" }
                                        span { {inline(&it.text)} }
                                    }
                                }
                            }
                        }
                    }
                    MdBlock::Table { head, aligns, rows } => {
                        let al = move |c: usize| match aligns.get(c).copied().unwrap_or_default() {
                            Align::Left => "left",
                            Align::Center => "center",
                            Align::Right => "right",
                        };
                        rsx! {
                            table { key: "{i}", style: "border-collapse:collapse",
                                thead { tr { for (c, h) in head.iter().enumerate() {
                                    th { key: "{c}", style: "text-align:{al(c)};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border-strong)", {inline(h)} }
                                } } }
                                tbody { for (r, row) in rows.iter().enumerate() {
                                    tr { key: "{r}", for (c, cell) in row.iter().enumerate() {
                                        td { key: "{c}", style: "text-align:{al(c)};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border)", {inline(cell)} }
                                    } }
                                } }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn heading_size(level: u8) -> &'static str {
    match level {
        1 => "xxl",
        2 => "xl",
        3 => "lg",
        _ => "md",
    }
}

/// Properties of [`SelectOption`].
#[derive(Props, Clone, PartialEq)]
pub struct SelectOptionProps {
    pub label: String,
    #[props(default)]
    pub selected: bool,
    #[props(default)]
    pub onclick: EventHandler<MouseEvent>,
}

/// One option of a list: a check mark when picked.
#[component]
pub fn SelectOption(props: SelectOptionProps) -> Element {
    let on = props.onclick;
    rsx! {
        KitBox { kit: Kit::Row { selected: props.selected }, style: "width:100%;gap:var(--dodai-space-sm)".to_string(), onclick: move |e| on.call(e),
            SelectionMarker { selected: props.selected }
            span { style: "flex:1;text-align:left", "{props.label}" }
        }
    }
}

/// Properties of [`Combobox`].
#[derive(Props, Clone, PartialEq)]
pub struct ComboboxProps {
    pub options: Vec<String>,
    pub value: Option<String>,
    #[props(default)]
    pub placeholder: String,
    #[props(default)]
    pub onselect: EventHandler<String>,
}

/// A field that filters its options as you type and picks one.
#[component]
pub fn Combobox(props: ComboboxProps) -> Element {
    let mut query = use_signal(String::new);
    let mut open = use_signal(|| false);
    let q = query().to_lowercase();
    let shown: Vec<String> = props
        .options
        .iter()
        .filter(|o| o.to_lowercase().contains(&q))
        .cloned()
        .collect();
    let on = props.onselect;
    let text = if open() {
        query()
    } else {
        props.value.clone().unwrap_or_default()
    };
    rsx! {
        div { style: "position:relative;display:inline-block;min-width:220px",
            input { class: "dodai-input", role: "combobox", "aria-expanded": "{open()}", placeholder: props.placeholder, value: text,
                onfocus: move |_| open.set(true),
                oninput: move |e| { query.set(e.value()); open.set(true); } }
            if open() {
                div { role: "listbox", style: "position:absolute;top:100%;left:0;right:0;z-index:41;margin-top:var(--dodai-space-xs)",
                    KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch".to_string(),
                        for o in shown {
                            SelectOption { key: "{o}", label: o.clone(), selected: props.value.as_deref() == Some(o.as_str()),
                                onclick: move |_| { on.call(o.clone()); query.set(String::new()); open.set(false); } }
                        }
                    }
                }
            }
        }
    }
}

/// Properties of [`Command`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandProps {
    /// `(label, shortcut)` per command.
    pub commands: Vec<(String, String)>,
    #[props(default)]
    pub onrun: EventHandler<String>,
}

/// A search box over a filtered list of commands with their shortcuts.
#[component]
pub fn Command(props: CommandProps) -> Element {
    let mut query = use_signal(String::new);
    let q = query().to_lowercase();
    let on = props.onrun;
    rsx! {
        KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch;min-width:320px".to_string(),
            input { class: "dodai-input", placeholder: "Type a command…", value: query(), oninput: move |e| query.set(e.value()) }
            for (label, key) in props.commands.into_iter().filter(|(l, _)| l.to_lowercase().contains(&q)) {
                crate::MenuItem { key: "{label}", label: label.clone(), shortcut: (!key.is_empty()).then_some(key), onclick: move |_| on.call(label.clone()) }
            }
        }
    }
}

/// Properties of [`CommandPalette`].
#[derive(Props, Clone, PartialEq)]
pub struct CommandPaletteProps {
    pub open: bool,
    pub commands: Vec<(String, String)>,
    #[props(default)]
    pub onrun: EventHandler<String>,
    #[props(default)]
    pub ondismiss: EventHandler<MouseEvent>,
}

/// [`Command`] centred high over a scrim. Controlled.
#[component]
pub fn CommandPalette(props: CommandPaletteProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let dismiss = props.ondismiss;
    rsx! {
        div { role: "presentation", style: "position:fixed;inset:0;z-index:50;display:flex;justify-content:center;padding-top:15vh;background:color-mix(in srgb, var(--dodai-background) 55%, transparent)",
            onclick: move |e| dismiss.call(e),
            div { role: "dialog", "aria-modal": "true", "aria-label": "Command palette", onclick: move |e| e.stop_propagation(),
                Command { commands: props.commands, onrun: props.onrun }
            }
        }
    }
}

/// Properties of [`SearchView`].
#[derive(Props, Clone, PartialEq)]
pub struct SearchViewProps {
    pub query: String,
    pub open: bool,
    #[props(default)]
    pub onquery: EventHandler<String>,
    #[props(default)]
    pub onclose: EventHandler<MouseEvent>,
    /// The suggestions or results.
    pub children: Element,
}

/// A full-width search that expands into a result list. Controlled.
#[component]
pub fn SearchView(props: SearchViewProps) -> Element {
    let close = props.onclose;
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-xs)",
            div { style: "display:flex;align-items:center;gap:var(--dodai-space-xs)",
                if props.open { crate::IconButton { icon: IconId::ArrowLeft, label: "Back", onclick: move |e| close.call(e) } }
                crate::InputBox { value: props.query, placeholder: "Search", leading: Some(IconId::Search), oninput: props.onquery }
            }
            if props.open { div { role: "listbox", style: "display:flex;flex-direction:column", {props.children} } }
        }
    }
}

/// Properties of [`TagField`].
#[derive(Props, Clone, PartialEq)]
pub struct TagFieldProps {
    pub tags: Vec<String>,
    #[props(default)]
    pub onchange: EventHandler<Vec<String>>,
}

/// Chips plus a text box: Enter adds a tag, Backspace on an empty box removes the last. Controlled.
#[component]
pub fn TagField(props: TagFieldProps) -> Element {
    let mut draft = use_signal(String::new);
    let (tags, on) = (props.tags.clone(), props.onchange);
    let last = tags.clone();
    rsx! {
        div { style: "display:flex;flex-wrap:wrap;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs);border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)",
            for (i, t) in props.tags.iter().enumerate() {
                {let rest: Vec<String> = props.tags.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, s)| s.clone()).collect();
                rsx! { crate::Chip { key: "{t}", selected: true, label: format!("{t} ×"), onclick: move |_| on.call(rest.clone()) } }}
            }
            input { style: "flex:1;min-width:8ch;border:0;outline:0;background:transparent;color:inherit;font:inherit", placeholder: "Add…", value: draft(),
                oninput: move |e| draft.set(e.value()),
                onkeydown: move |e| match e.key() {
                    Key::Enter if !draft().trim().is_empty() => {
                        let mut t = tags.clone();
                        t.push(draft().trim().to_string());
                        draft.set(String::new());
                        on.call(t);
                    }
                    Key::Backspace if draft().is_empty() && !last.is_empty() => on.call(last[..last.len() - 1].to_vec()),
                    _ => {}
                } }
        }
    }
}

/// Properties of [`ColorPicker`].
#[derive(Props, Clone, PartialEq)]
pub struct ColorPickerProps {
    /// `#rrggbb`.
    pub value: String,
    #[props(default)]
    pub onchange: EventHandler<String>,
}

/// A swatch and its hex value; the swatch opens the system colour chooser. Controlled.
#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    let on = props.onchange;
    rsx! {
        div { style: "display:inline-flex;align-items:center;gap:var(--dodai-space-sm)",
            input { r#type: "color", value: props.value.clone(), "aria-label": "Colour",
                style: "width:var(--dodai-control-height);height:var(--dodai-control-height);padding:0;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:none",
                oninput: move |e| on.call(e.value()) }
            span { style: "font-family:var(--dodai-font-mono)", "{props.value}" }
        }
    }
}

/// Properties of [`SelectionTrack`].
#[derive(Props, Clone, PartialEq)]
pub struct SelectionTrackProps {
    pub options: Vec<String>,
    pub selected: usize,
    #[props(default)]
    pub onselect: EventHandler<usize>,
}

/// A track of options with a thumb that slides to the picked one.
#[component]
pub fn SelectionTrack(props: SelectionTrackProps) -> Element {
    let n = props.options.len().max(1);
    let on = props.onselect;
    rsx! {
        div { role: "radiogroup", style: "position:relative;display:inline-grid;grid-template-columns:repeat({n},1fr);padding:2px;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)",
            span { style: "position:absolute;top:2px;bottom:2px;left:calc(2px + {props.selected} * (100% - 4px) / {n});width:calc((100% - 4px) / {n});border-radius:var(--dodai-radius);background:var(--dodai-primary);transition:left var(--dodai-motion-duration, 160ms) var(--dodai-motion-easing, ease)" }
            for (i, o) in props.options.iter().enumerate() {
                {let ink = if i == props.selected { "var(--dodai-background)" } else { "var(--dodai-text)" };
                rsx! { button { key: "{o}", role: "radio", "aria-checked": "{i == props.selected}",
                    style: "position:relative;border:0;background:none;font:inherit;padding:var(--dodai-space-xs) var(--dodai-space-md);cursor:pointer;color:{ink}",
                    onclick: move |_| on.call(i), "{o}" } }}
            }
        }
    }
}

/// Properties of [`FabMenu`].
#[derive(Props, Clone, PartialEq)]
pub struct FabMenuProps {
    pub open: bool,
    #[props(default)]
    pub ontoggle: EventHandler<MouseEvent>,
    /// Items, usually [`Fab`](crate::Fab)s with labels.
    pub children: Element,
}

/// A floating button that unfolds a column of actions above it. Controlled.
#[component]
pub fn FabMenu(props: FabMenuProps) -> Element {
    let t = props.ontoggle;
    rsx! {
        div { style: "display:inline-flex;flex-direction:column-reverse;align-items:flex-end;gap:var(--dodai-space-sm)",
            crate::Fab { icon: if props.open { IconId::Close } else { IconId::Plus }, onclick: move |e| t.call(e) }
            if props.open { {props.children} }
        }
    }
}

/// Properties of [`Carousel`].
#[derive(Props, Clone, PartialEq)]
pub struct CarouselProps {
    pub children: Element,
}

/// A row of slides that scroll and snap.
#[component]
pub fn Carousel(props: CarouselProps) -> Element {
    rsx! {
        div { style: "display:flex;gap:var(--dodai-space-sm);overflow-x:auto;scroll-snap-type:x mandatory;padding-bottom:var(--dodai-space-xs)",
            {props.children}
        }
    }
}

/// Properties of [`SplitPane`].
#[derive(Props, Clone, PartialEq)]
pub struct SplitPaneProps {
    pub first: Element,
    pub second: Element,
    /// Share of the first pane, 0.1..=0.9.
    #[props(default = 0.5)]
    pub ratio: f32,
}

/// Two panes side by side with a divider.
#[component]
pub fn SplitPane(props: SplitPaneProps) -> Element {
    let r = props.ratio.clamp(0.1, 0.9) * 100.0;
    rsx! {
        div { style: "display:flex;width:100%;height:100%",
            div { style: "flex:0 0 {r}%;overflow:auto", {props.first} }
            div { style: "width:var(--dodai-border-width);background:var(--dodai-border-strong)" }
            div { style: "flex:1;overflow:auto", {props.second} }
        }
    }
}

/// A scrolling box with the design's thin bar.
#[component]
pub fn ScrollArea(
    #[props(default = "100%".to_string())] height: String,
    children: Element,
) -> Element {
    rsx! { div { style: "height:{height};overflow:auto;scrollbar-width:thin;scrollbar-color:var(--dodai-border-strong) transparent", {children} } }
}

/// A responsive grid whose cells are at least `min` px wide.
#[component]
pub fn PageGrid(#[props(default = 240)] min: u32, children: Element) -> Element {
    rsx! { div { style: "display:grid;grid-template-columns:repeat(auto-fill,minmax({min}px,1fr));gap:var(--dodai-space-md)", {children} } }
}

/// Properties of [`ResponsiveShell`].
#[derive(Props, Clone, PartialEq)]
pub struct ResponsiveShellProps {
    /// Rail on wide windows, bar on narrow ones.
    pub rail: Element,
    pub bar: Element,
    pub children: Element,
}

/// The app frame: a navigation rail beside the content on wide windows, a bar under it on
/// narrow ones (a CSS container query, so no script).
#[component]
pub fn ResponsiveShell(props: ResponsiveShellProps) -> Element {
    rsx! {
        style { ".dodai-shell{{display:flex;flex-direction:column-reverse;min-height:100vh}}.dodai-shell__rail{{display:none}}@media (min-width:720px){{.dodai-shell{{flex-direction:row}}.dodai-shell__rail{{display:flex}}.dodai-shell__bar{{display:none}}}}" }
        div { class: "dodai-shell",
            div { class: "dodai-shell__bar", {props.bar} }
            div { class: "dodai-shell__rail", {props.rail} }
            main { style: "flex:1;padding:var(--dodai-space-md)", {props.children} }
        }
    }
}

/// A vertical stream of cards that grows downward.
#[component]
pub fn Feed(children: Element) -> Element {
    rsx! { div { role: "feed", style: "display:flex;flex-direction:column;gap:var(--dodai-space-md)", {children} } }
}

/// Properties of [`TreeNavigation`].
#[derive(Props, Clone, PartialEq)]
pub struct TreeNavigationProps {
    pub items: Vec<crate::TreeItem>,
}

/// A navigable tree of destinations.
#[component]
pub fn TreeNavigation(props: TreeNavigationProps) -> Element {
    rsx! { nav { crate::TreeView { items: props.items } } }
}

/// One button per theme of the design the caller names (or all themes), from the provider.
#[component]
pub fn ThemeSelector() -> Element {
    let mut t = crate::use_theme();
    let current = t.theme().id();
    rsx! {
        div { role: "radiogroup", style: "display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)",
            for (id, name) in t.list() {
                crate::Chip { key: "{id}", selected: id == current, label: name.to_string(), onclick: move |_| { t.select(id); } }
            }
        }
    }
}

/// One button per design system (family); picks the first theme of that family.
#[component]
pub fn DesignSystemSelector() -> Element {
    let mut t = crate::use_theme();
    let current = t.theme().meta.family;
    let mut families: Vec<(&'static str, &'static str)> = Vec::new();
    for th in dodai_ui_core::themes::all() {
        if !families.iter().any(|(f, _)| *f == th.meta.family) {
            families.push((th.meta.family, th.id()));
        }
    }
    rsx! {
        div { role: "radiogroup", style: "display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)",
            for (family, first) in families {
                crate::Chip { key: "{family}", selected: family == current, label: family.to_string(), onclick: move |_| { t.select(first); } }
            }
        }
    }
}

/// Theme buttons and glow controls in one bar (the provider's handle drives both).
#[component]
pub fn ThemeSwitcherBar() -> Element {
    let mut t = crate::use_theme();
    let g = t.glow();
    rsx! {
        div { style: "display:flex;flex-wrap:wrap;align-items:center;gap:var(--dodai-space-sm)",
            crate::IconButton { icon: IconId::ChevronLeft, label: "Previous theme", onclick: move |_| t.prev() }
            span { style: "min-width:12ch;text-align:center", "{t.theme().name()}" }
            crate::IconButton { icon: IconId::ChevronRight, label: "Next theme", onclick: move |_| t.next() }
            crate::IconButton { icon: IconId::Minus, label: "Less glow", onclick: move |_| t.set_glow(g - 0.5) }
            span { style: "font-family:var(--dodai-font-mono)", "{g:.1}" }
            crate::IconButton { icon: IconId::Plus, label: "More glow", onclick: move |_| t.set_glow(g + 0.5) }
        }
    }
}

/// The provider's handle: current theme, switching and glow.
pub type DodaiTheme = crate::ThemeHandle;

/// A tooltip bubble that shows on hover or focus of its trigger.
#[component]
pub fn DodaiTooltip(text: String, children: Element) -> Element {
    let mut shown = use_signal(|| false);
    rsx! {
        span { style: "position:relative;display:inline-block",
            onmouseenter: move |_| shown.set(true), onmouseleave: move |_| shown.set(false),
            onfocusin: move |_| shown.set(true), onfocusout: move |_| shown.set(false),
            {children}
            if shown() {
                span { role: "tooltip", style: "position:absolute;bottom:100%;left:50%;transform:translateX(-50%);margin-bottom:var(--dodai-space-xs);white-space:nowrap;z-index:41",
                    KitBox { kit: Kit::Tooltip { rich: false }, "{text}" }
                }
            }
        }
    }
}

/// A halo from the design's glow colour around its children (none on designs without glow).
#[component]
pub fn GlowLayer(children: Element) -> Element {
    rsx! { div { style: "filter:drop-shadow(0 0 var(--dodai-glow-blur, 8px) var(--dodai-glow, transparent))", {children} } }
}

/// The design's scanline texture over its parent (set `position:relative` on the parent).
#[component]
pub fn Scanlines() -> Element {
    rsx! { div { aria_hidden: "true", style: "position:absolute;inset:0;pointer-events:none;opacity:var(--dodai-scanlines);background:repeating-linear-gradient(to bottom,transparent 0,transparent 2px,rgba(0,0,0,.28) 3px)" } }
}

/// A frosted-glass backdrop over its parent (set `position:relative` on the parent).
#[component]
pub fn FrostLayer(#[props(default = 16)] blur: u32) -> Element {
    rsx! { div { aria_hidden: "true", style: "position:absolute;inset:0;pointer-events:none;backdrop-filter:blur({blur}px);-webkit-backdrop-filter:blur({blur}px);background:color-mix(in srgb, var(--dodai-surface) 40%, transparent)" } }
}

/// A scene whose content sits on a frosted layer over the app backdrop.
#[component]
pub fn FrostedScene(children: Element) -> Element {
    rsx! {
        div { style: "position:relative;border-radius:var(--dodai-radius);overflow:hidden",
            FrostLayer {}
            div { style: "position:relative;padding:var(--dodai-space-md)", {children} }
        }
    }
}
