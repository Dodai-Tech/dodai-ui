//! Content and composition: Markdown, pickers and palettes, theme selectors, scrolling and
//! layout containers, and the glow, frost and scanline layers.

use super::kit::{Chip, CodeBlock, KitBox, TreeItem, TreeView};
use super::surfaces::{Fab, IconButton};
use super::time::SelectionMarker;
use super::widgets::{InputBox, MenuItem};
use dodai_core::markdown::{self as md, Align, MdBlock, MdInline};
use dodai_core::Kit;
use dodai_icons::IconId;
use leptos::prelude::*;

fn inline(src: &str) -> AnyView {
    let (text, spans) = md::parse_inline(src);
    let slice = |a: usize, b: usize| text.get(a..b).unwrap_or_default().to_string();
    let mut out: Vec<AnyView> = Vec::new();
    let mut at = 0;
    for (r, kind) in spans {
        if r.start > at {
            out.push(slice(at, r.start).into_any());
        }
        let s = slice(r.start, r.end);
        out.push(match kind {
            MdInline::Bold => view! { <strong>{s}</strong> }.into_any(),
            MdInline::Italic => view! { <em>{s}</em> }.into_any(),
            MdInline::Code => view! { <code style="font-family:var(--dodai-font-mono);background:var(--dodai-surface-raised);padding:0 .3em;border-radius:var(--dodai-radius)">{s}</code> }.into_any(),
            MdInline::Link => view! { <a style="color:var(--dodai-link, var(--dodai-primary))">{s}</a> }.into_any(),
        });
        at = r.end;
    }
    if at < text.len() {
        out.push(slice(at, text.len()).into_any());
    }
    out.into_any()
}

fn heading_size(level: u8) -> &'static str {
    match level {
        1 => "xxl",
        2 => "xl",
        3 => "lg",
        _ => "md",
    }
}

fn align(a: Align) -> &'static str {
    match a {
        Align::Left => "left",
        Align::Center => "center",
        Align::Right => "right",
    }
}

/// Renders Markdown (headings, paragraphs, lists, tables, quotes, code, rules) with the
/// design's fonts and colours, from the parser shared by every toolkit.
#[component]
pub fn Markdown(#[prop(into)] source: String) -> impl IntoView {
    let blocks = md::parse_blocks(&source)
        .into_iter()
        .map(|b| match b {
            MdBlock::Heading(l, t) => {
                let style = format!("margin:0;font-family:var(--dodai-font-display);font-size:var(--dodai-size-{});font-weight:inherit", heading_size(l));
                match l {
                    1 => view! { <h1 style=style>{inline(&t)}</h1> }.into_any(),
                    2 => view! { <h2 style=style>{inline(&t)}</h2> }.into_any(),
                    3 => view! { <h3 style=style>{inline(&t)}</h3> }.into_any(),
                    _ => view! { <h4 style=style>{inline(&t)}</h4> }.into_any(),
                }
            }
            MdBlock::Paragraph(t) => view! { <p style="margin:0">{inline(&t)}</p> }.into_any(),
            MdBlock::Quote(t) => view! {
                <blockquote style="margin:0;padding-left:var(--dodai-space-md);border-left:3px solid var(--dodai-border-strong);color:var(--dodai-text-muted)">{inline(&t)}</blockquote>
            }.into_any(),
            MdBlock::Code { text, .. } => view! { <CodeBlock code=text /> }.into_any(),
            MdBlock::Rule => view! { <hr style="width:100%;border:0;border-top:var(--dodai-border-width) solid var(--dodai-border)" /> }.into_any(),
            MdBlock::List(items) => {
                let marks = md::list_markers(&items);
                let rows = items
                    .iter()
                    .enumerate()
                    .map(|(n, it)| view! {
                        <div style=format!("display:flex;gap:var(--dodai-space-xs);padding-left:calc({} * var(--dodai-space-md))", it.depth)>
                            <span style="color:var(--dodai-text-muted)">{marks[n].clone()}</span>
                            <span>{inline(&it.text)}</span>
                        </div>
                    })
                    .collect_view();
                view! { <div style="display:flex;flex-direction:column;gap:2px">{rows}</div> }.into_any()
            }
            MdBlock::Table { head, aligns, rows } => {
                let al = |c: usize| align(aligns.get(c).copied().unwrap_or_default());
                let th = head
                    .iter()
                    .enumerate()
                    .map(|(c, h)| view! { <th style=format!("text-align:{};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border-strong)", al(c))>{inline(h)}</th> })
                    .collect_view();
                let tr = rows
                    .iter()
                    .map(|row| {
                        let tds = row
                            .iter()
                            .enumerate()
                            .map(|(c, cell)| view! { <td style=format!("text-align:{};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border)", al(c))>{inline(cell)}</td> })
                            .collect_view();
                        view! { <tr>{tds}</tr> }
                    })
                    .collect_view();
                view! { <table style="border-collapse:collapse"><thead><tr>{th}</tr></thead><tbody>{tr}</tbody></table> }.into_any()
            }
        })
        .collect_view();
    view! { <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);color:var(--dodai-text)">{blocks}</div> }
}

/// One option of a list: a check mark when picked.
#[component]
pub fn SelectOption(
    #[prop(into)] label: String,
    #[prop(optional)] selected: bool,
) -> impl IntoView {
    view! {
        <KitBox kit=Kit::Row { selected } pressable=true style="width:100%;gap:var(--dodai-space-sm);">
            <SelectionMarker selected=selected />
            <span style="flex:1;text-align:left">{label}</span>
        </KitBox>
    }
}

fn options_list(options: Vec<String>, value: Option<String>, query: &str) -> impl IntoView {
    let q = query.to_lowercase();
    options
        .into_iter()
        .filter(|o| o.to_lowercase().contains(&q))
        .map(|o| {
            let on = value.as_deref() == Some(o.as_str());
            view! { <SelectOption label=o selected=on /> }
        })
        .collect_view()
}

/// A dropdown field over `options`, shown open or closed.
#[component]
pub fn Select(
    options: Vec<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional)] open: bool,
    #[prop(optional, into)] placeholder: String,
) -> impl IntoView {
    let shown = value.clone().unwrap_or(placeholder);
    let list = open.then(|| view! {
        <div role="listbox" style="position:absolute;top:100%;left:0;right:0;z-index:41;margin-top:var(--dodai-space-xs)">
            <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;">{options_list(options, value, "")}</KitBox>
        </div>
    });
    view! {
        <div style="position:relative;display:inline-block;min-width:220px">
            <KitBox kit=Kit::Select pressable=true style="width:100%;justify-content:space-between;">
                <span>{shown}</span>
                <super::kit::Icon icon=IconId::ChevronDown size=16 />
            </KitBox>
            {list}
        </div>
    }
}

/// A field that filters `options` by `query` and lists the matches.
#[component]
pub fn Combobox(
    options: Vec<String>,
    #[prop(optional, into)] value: Option<String>,
    #[prop(optional, into)] query: String,
    #[prop(optional, into)] placeholder: String,
) -> impl IntoView {
    let text = if query.is_empty() {
        value.clone().unwrap_or_default()
    } else {
        query.clone()
    };
    view! {
        <div style="position:relative;display:inline-block;min-width:220px">
            <input class="dodai-input" role="combobox" aria-expanded="true" placeholder=placeholder value=text />
            <div role="listbox" style="position:absolute;top:100%;left:0;right:0;z-index:41;margin-top:var(--dodai-space-xs)">
                <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;">{options_list(options, value, &query)}</KitBox>
            </div>
        </div>
    }
}

/// A search box over a filtered list of `(label, shortcut)` commands.
#[component]
pub fn Command(
    commands: Vec<(String, String)>,
    #[prop(optional, into)] query: String,
) -> impl IntoView {
    let q = query.to_lowercase();
    let rows = commands
        .into_iter()
        .filter(|(l, _)| l.to_lowercase().contains(&q))
        .map(|(l, k)| view! { <MenuItem label=l shortcut=k /> })
        .collect_view();
    view! {
        <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;min-width:320px;">
            <input class="dodai-input" placeholder="Type a command…" value=query />
            {rows}
        </KitBox>
    }
}

/// [`Command`] centred high over a scrim.
#[component]
pub fn CommandPalette(open: bool, commands: Vec<(String, String)>) -> impl IntoView {
    open.then(|| view! {
        <div role="presentation" style="position:fixed;inset:0;z-index:50;display:flex;justify-content:center;padding-top:15vh;background:color-mix(in srgb, var(--dodai-background) 55%, transparent)">
            <div role="dialog" aria-modal="true" aria-label="Command palette"><Command commands=commands /></div>
        </div>
    })
}

/// A full-width search that expands into a result list.
#[component]
pub fn SearchView(#[prop(into)] query: String, open: bool, children: Children) -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            <div style="display:flex;align-items:center;gap:var(--dodai-space-xs)">
                {open.then(|| view! { <IconButton icon=IconId::ArrowLeft label="Back" /> })}
                <InputBox value=query placeholder="Search" leading=IconId::Search />
            </div>
            {open.then(|| view! { <div role="listbox" style="display:flex;flex-direction:column">{children()}</div> })}
        </div>
    }
}

/// Chips plus a text box for adding tags.
#[component]
pub fn TagField(tags: Vec<String>) -> impl IntoView {
    let chips = tags
        .into_iter()
        .map(|t| view! { <Chip label=format!("{t} ×") selected=true /> })
        .collect_view();
    view! {
        <div style="display:flex;flex-wrap:wrap;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs);border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)">
            {chips}
            <input style="flex:1;min-width:8ch;border:0;outline:0;background:transparent;color:inherit;font:inherit" placeholder="Add…" />
        </div>
    }
}

/// A swatch and its hex value (`#rrggbb`); the swatch opens the system colour chooser.
#[component]
pub fn ColorPicker(#[prop(into)] value: String) -> impl IntoView {
    view! {
        <div style="display:inline-flex;align-items:center;gap:var(--dodai-space-sm)">
            <input type="color" value=value.clone() aria-label="Colour"
                style="width:var(--dodai-control-height);height:var(--dodai-control-height);padding:0;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:none" />
            <span style="font-family:var(--dodai-font-mono)">{value}</span>
        </div>
    }
}

/// A track of options with a thumb under the picked one.
#[component]
pub fn SelectionTrack(options: Vec<String>, selected: usize) -> impl IntoView {
    let n = options.len().max(1);
    let buttons = options
        .into_iter()
        .enumerate()
        .map(|(i, o)| {
            let ink = if i == selected { "var(--dodai-background)" } else { "var(--dodai-text)" };
            view! {
                <button role="radio" aria-checked=(i == selected).to_string()
                    style=format!("position:relative;border:0;background:none;font:inherit;padding:var(--dodai-space-xs) var(--dodai-space-md);cursor:pointer;color:{ink}")>{o}</button>
            }
        })
        .collect_view();
    view! {
        <div role="radiogroup" style=format!("position:relative;display:inline-grid;grid-template-columns:repeat({n},1fr);padding:2px;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)")>
            <span style=format!("position:absolute;top:2px;bottom:2px;left:calc(2px + {selected} * (100% - 4px) / {n});width:calc((100% - 4px) / {n});border-radius:var(--dodai-radius);background:var(--dodai-primary);transition:left var(--dodai-motion-duration, 160ms) var(--dodai-motion-easing, ease)")></span>
            {buttons}
        </div>
    }
}

/// A floating button that unfolds a column of actions above it.
#[component]
pub fn FabMenu(open: bool, children: Children) -> impl IntoView {
    view! {
        <div style="display:inline-flex;flex-direction:column-reverse;align-items:flex-end;gap:var(--dodai-space-sm)">
            <Fab icon=if open { IconId::Close } else { IconId::Plus } />
            {open.then(children)}
        </div>
    }
}

/// A row of slides that scroll and snap.
#[component]
pub fn Carousel(children: Children) -> impl IntoView {
    view! { <div style="display:flex;gap:var(--dodai-space-sm);overflow-x:auto;scroll-snap-type:x mandatory;padding-bottom:var(--dodai-space-xs)">{children()}</div> }
}

/// Two panes side by side with a divider; `ratio` is the first pane's share (0.1–0.9).
#[component]
pub fn SplitPane(
    #[prop(into)] first: ViewFn,
    #[prop(into)] second: ViewFn,
    #[prop(default = 0.5)] ratio: f32,
) -> impl IntoView {
    let r = ratio.clamp(0.1, 0.9) * 100.0;
    view! {
        <div style="display:flex;width:100%;height:100%">
            <div style=format!("flex:0 0 {r}%;overflow:auto")>{first.run()}</div>
            <div style="width:var(--dodai-border-width);background:var(--dodai-border-strong)"></div>
            <div style="flex:1;overflow:auto">{second.run()}</div>
        </div>
    }
}

/// A scrolling box with the design's thin bar.
#[component]
pub fn ScrollArea(
    #[prop(default = "100%".to_string(), into)] height: String,
    children: Children,
) -> impl IntoView {
    view! { <div style=format!("height:{height};overflow:auto;scrollbar-width:thin;scrollbar-color:var(--dodai-border-strong) transparent")>{children()}</div> }
}

/// A responsive grid whose cells are at least `min` px wide.
#[component]
pub fn PageGrid(#[prop(default = 240)] min: u32, children: Children) -> impl IntoView {
    view! { <div style=format!("display:grid;grid-template-columns:repeat(auto-fill,minmax({min}px,1fr));gap:var(--dodai-space-md)")>{children()}</div> }
}

/// The app frame: a rail beside the content on wide windows, a bar under it on narrow ones.
#[component]
pub fn ResponsiveShell(
    #[prop(into)] rail: ViewFn,
    #[prop(into)] bar: ViewFn,
    children: Children,
) -> impl IntoView {
    view! {
        <style>".dodai-shell{display:flex;flex-direction:column-reverse;min-height:100vh}.dodai-shell__rail{display:none}@media (min-width:720px){.dodai-shell{flex-direction:row}.dodai-shell__rail{display:flex}.dodai-shell__bar{display:none}}"</style>
        <div class="dodai-shell">
            <div class="dodai-shell__bar">{bar.run()}</div>
            <div class="dodai-shell__rail">{rail.run()}</div>
            <main style="flex:1;padding:var(--dodai-space-md)">{children()}</main>
        </div>
    }
}

/// A vertical stream of cards.
#[component]
pub fn Feed(children: Children) -> impl IntoView {
    view! { <div role="feed" style="display:flex;flex-direction:column;gap:var(--dodai-space-md)">{children()}</div> }
}

/// A navigable tree of destinations.
#[component]
pub fn TreeNavigation(items: Vec<TreeItem>) -> impl IntoView {
    view! { <nav><TreeView items=items /></nav> }
}

/// The provider's handle: the active theme and its resolved surface.
pub type DodaiTheme = crate::GlassContext;

fn current_theme_id() -> Option<(String, String)> {
    use_context::<crate::GlassContext>()
        .map(|g| (g.theme.id().to_string(), g.theme.meta.family.to_string()))
}

/// One chip per theme; the active one is selected.
#[component]
pub fn ThemeSelector() -> impl IntoView {
    let (cur, _) = current_theme_id().unwrap_or_default();
    let chips = dodai_core::themes::all()
        .into_iter()
        .map(|t| view! { <Chip label=t.name().to_string() selected=t.id() == cur /> })
        .collect_view();
    view! { <div role="radiogroup" style="display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)">{chips}</div> }
}

/// One chip per design system (family); the active one is selected.
#[component]
pub fn DesignSystemSelector() -> impl IntoView {
    let (_, family) = current_theme_id().unwrap_or_default();
    let mut seen: Vec<String> = Vec::new();
    for t in dodai_core::themes::all() {
        if !seen.iter().any(|f| f == t.meta.family) {
            seen.push(t.meta.family.to_string());
        }
    }
    let chips = seen
        .into_iter()
        .map(|f| view! { <Chip label=f.clone() selected=f == family /> })
        .collect_view();
    view! { <div role="radiogroup" style="display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)">{chips}</div> }
}

/// Theme arrows and glow controls in one bar.
#[component]
pub fn ThemeSwitcherBar() -> impl IntoView {
    let name = use_context::<crate::GlassContext>()
        .map(|g| g.theme.name().to_string())
        .unwrap_or_default();
    view! {
        <div style="display:flex;flex-wrap:wrap;align-items:center;gap:var(--dodai-space-sm)">
            <IconButton icon=IconId::ChevronLeft label="Previous theme" />
            <span style="min-width:12ch;text-align:center">{name}</span>
            <IconButton icon=IconId::ChevronRight label="Next theme" />
            <IconButton icon=IconId::Minus label="Less glow" />
            <IconButton icon=IconId::Plus label="More glow" />
        </div>
    }
}

/// A tooltip bubble that shows on hover or focus of its trigger (CSS only).
#[component]
pub fn DodaiTooltip(#[prop(into)] text: String, children: Children) -> impl IntoView {
    view! {
        <span class="dodai-tip" style="position:relative;display:inline-block">
            <style>".dodai-tip>.dodai-tip__bubble{display:none}.dodai-tip:hover>.dodai-tip__bubble,.dodai-tip:focus-within>.dodai-tip__bubble{display:block}"</style>
            {children()}
            <span class="dodai-tip__bubble" role="tooltip" style="position:absolute;bottom:100%;left:50%;transform:translateX(-50%);margin-bottom:var(--dodai-space-xs);white-space:nowrap;z-index:41">
                <KitBox kit=Kit::Tooltip { rich: false }>{text}</KitBox>
            </span>
        </span>
    }
}

/// A halo from the design's glow colour around its children.
#[component]
pub fn GlowLayer(children: Children) -> impl IntoView {
    view! { <div style="filter:drop-shadow(0 0 var(--dodai-glow-blur, 8px) var(--dodai-glow, transparent))">{children()}</div> }
}

/// The design's scanline texture over its parent (set `position:relative` on the parent).
#[component]
pub fn Scanlines() -> impl IntoView {
    view! { <div aria-hidden="true" style="position:absolute;inset:0;pointer-events:none;opacity:var(--dodai-scanlines);background:repeating-linear-gradient(to bottom,transparent 0,transparent 2px,rgba(0,0,0,.28) 3px)"></div> }
}

/// A frosted-glass backdrop over its parent (set `position:relative` on the parent).
#[component]
pub fn FrostLayer(#[prop(default = 16)] blur: u32) -> impl IntoView {
    view! { <div aria-hidden="true" style=format!("position:absolute;inset:0;pointer-events:none;backdrop-filter:blur({blur}px);-webkit-backdrop-filter:blur({blur}px);background:color-mix(in srgb, var(--dodai-surface) 40%, transparent)")></div> }
}

/// A scene whose content sits on a frosted layer.
#[component]
pub fn FrostedScene(children: Children) -> impl IntoView {
    view! {
        <div style="position:relative;border-radius:var(--dodai-radius);overflow:hidden">
            <FrostLayer />
            <div style="position:relative;padding:var(--dodai-space-md)">{children()}</div>
        </div>
    }
}
