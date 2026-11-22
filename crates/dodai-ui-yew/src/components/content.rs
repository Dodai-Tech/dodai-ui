//! Content and composition: Markdown, pickers and palettes, theme selectors, scrolling and
//! layout containers, and the glow, frost and scanline layers.

use super::kit::{Chip, CodeBlock, KitBox, TreeItem, TreeView};
use super::surfaces::{Fab, IconButton};
use super::time::SelectionMarker;
use super::widgets::{InputBox, MenuItem};
use crate::dom::input_value;
use crate::use_theme;
use dodai_ui_core::markdown::{self as md, Align, MdBlock, MdInline};
use dodai_ui_core::Kit;
use dodai_ui_icons::IconId;
use yew::prelude::*;

fn inline(src: &str) -> Html {
    let (text, spans) = md::parse_inline(src);
    let slice = |a: usize, b: usize| text.get(a..b).unwrap_or_default().to_string();
    let mut out: Vec<Html> = Vec::new();
    let mut at = 0;
    for (r, kind) in spans {
        if r.start > at {
            out.push(html! { { slice(at, r.start) } });
        }
        let s = slice(r.start, r.end);
        out.push(match kind {
            MdInline::Bold => html! { <strong>{ s }</strong> },
            MdInline::Italic => html! { <em>{ s }</em> },
            MdInline::Code => html! { <code style="font-family:var(--dodai-font-mono);background:var(--dodai-surface-raised);padding:0 .3em;border-radius:var(--dodai-radius)">{ s }</code> },
            MdInline::Link => html! { <a style="color:var(--dodai-link, var(--dodai-primary))">{ s }</a> },
        });
        at = r.end;
    }
    if at < text.len() {
        out.push(html! { { slice(at, text.len()) } });
    }
    out.into_iter().collect()
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

#[derive(Properties, PartialEq)]
pub struct MarkdownProps {
    pub source: AttrValue,
}

/// Renders Markdown (headings, paragraphs, lists, tables, quotes, code, rules) with the
/// design's fonts and colours, from the parser shared by every toolkit.
#[function_component]
pub fn Markdown(props: &MarkdownProps) -> Html {
    let blocks = md::parse_blocks(&props.source).into_iter().map(|b| match b {
        MdBlock::Heading(l, t) => {
            let style = format!("margin:0;font-family:var(--dodai-font-display);font-size:var(--dodai-size-{});font-weight:inherit", heading_size(l));
            match l {
                1 => html! { <h1 {style}>{ inline(&t) }</h1> },
                2 => html! { <h2 {style}>{ inline(&t) }</h2> },
                3 => html! { <h3 {style}>{ inline(&t) }</h3> },
                _ => html! { <h4 {style}>{ inline(&t) }</h4> },
            }
        }
        MdBlock::Paragraph(t) => html! { <p style="margin:0">{ inline(&t) }</p> },
        MdBlock::Quote(t) => html! {
            <blockquote style="margin:0;padding-left:var(--dodai-space-md);border-left:3px solid var(--dodai-border-strong);color:var(--dodai-text-muted)">{ inline(&t) }</blockquote>
        },
        MdBlock::Code { text, .. } => html! { <CodeBlock code={text} /> },
        MdBlock::Rule => html! { <hr style="width:100%;border:0;border-top:var(--dodai-border-width) solid var(--dodai-border)" /> },
        MdBlock::List(items) => {
            let marks = md::list_markers(&items);
            html! {
                <div style="display:flex;flex-direction:column;gap:2px">
                    { for items.iter().enumerate().map(|(n, it)| html! {
                        <div style={format!("display:flex;gap:var(--dodai-space-xs);padding-left:calc({} * var(--dodai-space-md))", it.depth)}>
                            <span style="color:var(--dodai-text-muted)">{ marks[n].clone() }</span>
                            <span>{ inline(&it.text) }</span>
                        </div>
                    }) }
                </div>
            }
        }
        MdBlock::Table { head, aligns, rows } => {
            let al = |c: usize| align(aligns.get(c).copied().unwrap_or_default());
            html! {
                <table style="border-collapse:collapse">
                    <thead><tr>{ for head.iter().enumerate().map(|(c, h)| html! {
                        <th style={format!("text-align:{};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border-strong)", al(c))}>{ inline(h) }</th>
                    }) }</tr></thead>
                    <tbody>{ for rows.iter().map(|row| html! {
                        <tr>{ for row.iter().enumerate().map(|(c, cell)| html! {
                            <td style={format!("text-align:{};padding:var(--dodai-space-xs) var(--dodai-space-sm);border-bottom:var(--dodai-border-width) solid var(--dodai-border)", al(c))}>{ inline(cell) }</td>
                        }) }</tr>
                    }) }</tbody>
                </table>
            }
        }
    });
    html! { <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);color:var(--dodai-text)">{ for blocks }</div> }
}

#[derive(Properties, PartialEq)]
pub struct SelectOptionProps {
    pub label: AttrValue,
    #[prop_or_default]
    pub selected: bool,
    #[prop_or_default]
    pub onclick: Callback<MouseEvent>,
}

/// One option of a list: a check mark when picked.
#[function_component]
pub fn SelectOption(props: &SelectOptionProps) -> Html {
    html! {
        <KitBox kit={Kit::Row { selected: props.selected }} pressable=true onclick={props.onclick.clone()} style="width:100%;gap:var(--dodai-space-sm);">
            <SelectionMarker selected={props.selected} />
            <span style="flex:1;text-align:left">{ props.label.clone() }</span>
        </KitBox>
    }
}

fn options_list(
    options: &[AttrValue],
    value: &Option<AttrValue>,
    query: &str,
    on: &Callback<AttrValue>,
) -> Html {
    let q = query.to_lowercase();
    options
        .iter()
        .filter(|o| o.to_lowercase().contains(&q))
        .map(|o| {
            let (on, pick) = (on.clone(), o.clone());
            html! { <SelectOption label={o.clone()} selected={value.as_ref() == Some(o)} onclick={Callback::from(move |_| on.emit(pick.clone()))} /> }
        })
        .collect()
}

#[derive(Properties, PartialEq)]
pub struct SelectProps {
    pub options: Vec<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub onselect: Callback<AttrValue>,
}

/// A dropdown field over `options`; opens on press and closes on a pick.
#[function_component]
pub fn Select(props: &SelectProps) -> Html {
    let open = use_state(|| false);
    let on = {
        let (open, cb) = (open.clone(), props.onselect.clone());
        Callback::from(move |v| {
            open.set(false);
            cb.emit(v);
        })
    };
    let shown = props.value.clone().unwrap_or(props.placeholder.clone());
    let toggle = {
        let open = open.clone();
        Callback::from(move |_| open.set(!*open))
    };
    html! {
        <div style="position:relative;display:inline-block;min-width:220px">
            <KitBox kit={Kit::Select} pressable=true onclick={toggle} style="width:100%;justify-content:space-between;">
                <span>{ shown }</span>
                <super::kit::Icon icon={IconId::ChevronDown} size={16} />
            </KitBox>
            if *open {
                <div role="listbox" style="position:absolute;top:100%;left:0;right:0;z-index:41;margin-top:var(--dodai-space-xs)">
                    <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;">{ options_list(&props.options, &props.value, "", &on) }</KitBox>
                </div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ComboboxProps {
    pub options: Vec<AttrValue>,
    #[prop_or_default]
    pub value: Option<AttrValue>,
    #[prop_or_default]
    pub placeholder: AttrValue,
    #[prop_or_default]
    pub onselect: Callback<AttrValue>,
}

/// A field that filters its options as you type and picks one.
#[function_component]
pub fn Combobox(props: &ComboboxProps) -> Html {
    let query = use_state(String::new);
    let open = use_state(|| false);
    let on = {
        let (open, query, cb) = (open.clone(), query.clone(), props.onselect.clone());
        Callback::from(move |v| {
            query.set(String::new());
            open.set(false);
            cb.emit(v);
        })
    };
    let text = if *open {
        (*query).clone()
    } else {
        props
            .value
            .clone()
            .map(|v| v.to_string())
            .unwrap_or_default()
    };
    let (focus, input) = (open.clone(), (open.clone(), query.clone()));
    html! {
        <div style="position:relative;display:inline-block;min-width:220px">
            <input class="dodai-input" role="combobox" aria-expanded={open.to_string()} placeholder={props.placeholder.clone()} value={text}
                onfocus={Callback::from(move |_| focus.set(true))}
                oninput={Callback::from(move |e: InputEvent| { input.1.set(input_value(&e)); input.0.set(true); })} />
            if *open {
                <div role="listbox" style="position:absolute;top:100%;left:0;right:0;z-index:41;margin-top:var(--dodai-space-xs)">
                    <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;">{ options_list(&props.options, &props.value, &query, &on) }</KitBox>
                </div>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CommandProps {
    /// `(label, shortcut)` per command.
    pub commands: Vec<(AttrValue, AttrValue)>,
    #[prop_or_default]
    pub onrun: Callback<AttrValue>,
}

/// A search box over a filtered list of commands with their shortcuts.
#[function_component]
pub fn Command(props: &CommandProps) -> Html {
    let query = use_state(String::new);
    let q = query.to_lowercase();
    let set = query.clone();
    html! {
        <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;min-width:320px;">
            <input class="dodai-input" placeholder="Type a command…" value={(*query).clone()}
                oninput={Callback::from(move |e: InputEvent| set.set(input_value(&e)))} />
            { for props.commands.iter().filter(|(l, _)| l.to_lowercase().contains(&q)).map(|(l, k)| {
                let (cb, label) = (props.onrun.clone(), l.clone());
                html! { <MenuItem label={l.clone()} shortcut={k.clone()} onclick={Callback::from(move |_| cb.emit(label.clone()))} /> }
            }) }
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct CommandPaletteProps {
    pub open: bool,
    pub commands: Vec<(AttrValue, AttrValue)>,
    #[prop_or_default]
    pub onrun: Callback<AttrValue>,
    #[prop_or_default]
    pub ondismiss: Callback<MouseEvent>,
}

/// [`Command`] centred high over a scrim. Controlled.
#[function_component]
pub fn CommandPalette(props: &CommandPaletteProps) -> Html {
    if !props.open {
        return html! {};
    }
    html! {
        <div role="presentation" style="position:fixed;inset:0;z-index:50;display:flex;justify-content:center;padding-top:15vh;background:color-mix(in srgb, var(--dodai-background) 55%, transparent)"
            onclick={props.ondismiss.clone()}>
            <div role="dialog" aria-modal="true" aria-label="Command palette" onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                <Command commands={props.commands.clone()} onrun={props.onrun.clone()} />
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SearchViewProps {
    pub query: AttrValue,
    pub open: bool,
    #[prop_or_default]
    pub onquery: Callback<String>,
    #[prop_or_default]
    pub onclose: Callback<MouseEvent>,
    /// The suggestions or results.
    #[prop_or_default]
    pub children: Html,
}

/// A full-width search that expands into a result list. Controlled.
#[function_component]
pub fn SearchView(props: &SearchViewProps) -> Html {
    html! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-xs)">
            <div style="display:flex;align-items:center;gap:var(--dodai-space-xs)">
                if props.open { <IconButton icon={IconId::ArrowLeft} label="Back" onclick={props.onclose.clone()} /> }
                <InputBox value={props.query.clone()} placeholder="Search" leading={IconId::Search} oninput={props.onquery.clone()} />
            </div>
            if props.open { <div role="listbox" style="display:flex;flex-direction:column">{ props.children.clone() }</div> }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TagFieldProps {
    pub tags: Vec<AttrValue>,
    #[prop_or_default]
    pub onchange: Callback<Vec<AttrValue>>,
}

/// Chips plus a text box: Enter adds a tag, Backspace on an empty box removes the last. Controlled.
#[function_component]
pub fn TagField(props: &TagFieldProps) -> Html {
    let draft = use_state(String::new);
    let (tags, cb) = (props.tags.clone(), props.onchange.clone());
    let (set, key) = (draft.clone(), draft.clone());
    html! {
        <div style="display:flex;flex-wrap:wrap;align-items:center;gap:var(--dodai-space-xs);padding:var(--dodai-space-xs);border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)">
            { for props.tags.iter().enumerate().map(|(i, t)| {
                let (cb, rest) = (cb.clone(), props.tags.iter().enumerate().filter(|(j, _)| *j != i).map(|(_, s)| s.clone()).collect::<Vec<_>>());
                html! { <Chip label={format!("{t} ×")} selected=true onclick={Callback::from(move |_| cb.emit(rest.clone()))} /> }
            }) }
            <input style="flex:1;min-width:8ch;border:0;outline:0;background:transparent;color:inherit;font:inherit" placeholder="Add…" value={(*draft).clone()}
                oninput={Callback::from(move |e: InputEvent| set.set(input_value(&e)))}
                onkeydown={Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
                    "Enter" if !key.trim().is_empty() => {
                        let mut t = tags.clone();
                        t.push(AttrValue::from(key.trim().to_string()));
                        key.set(String::new());
                        cb.emit(t);
                    }
                    "Backspace" if key.is_empty() && !tags.is_empty() => cb.emit(tags[..tags.len() - 1].to_vec()),
                    _ => {}
                })} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ColorPickerProps {
    /// `#rrggbb`.
    pub value: AttrValue,
    #[prop_or_default]
    pub onchange: Callback<String>,
}

/// A swatch and its hex value; the swatch opens the system colour chooser. Controlled.
#[function_component]
pub fn ColorPicker(props: &ColorPickerProps) -> Html {
    let cb = props.onchange.clone();
    html! {
        <div style="display:inline-flex;align-items:center;gap:var(--dodai-space-sm)">
            <input type="color" value={props.value.clone()} aria-label="Colour"
                style="width:var(--dodai-control-height);height:var(--dodai-control-height);padding:0;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:none"
                oninput={Callback::from(move |e: InputEvent| cb.emit(input_value(&e)))} />
            <span style="font-family:var(--dodai-font-mono)">{ props.value.clone() }</span>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectionTrackProps {
    pub options: Vec<AttrValue>,
    pub selected: usize,
    #[prop_or_default]
    pub onselect: Callback<usize>,
}

/// A track of options with a thumb that slides to the picked one.
#[function_component]
pub fn SelectionTrack(props: &SelectionTrackProps) -> Html {
    let (n, sel) = (props.options.len().max(1), props.selected);
    html! {
        <div role="radiogroup" style={format!("position:relative;display:inline-grid;grid-template-columns:repeat({n},1fr);padding:2px;border:var(--dodai-border-width) solid var(--dodai-border);border-radius:var(--dodai-radius);background:var(--dodai-surface-raised)")}>
            <span style={format!("position:absolute;top:2px;bottom:2px;left:calc(2px + {sel} * (100% - 4px) / {n});width:calc((100% - 4px) / {n});border-radius:var(--dodai-radius);background:var(--dodai-primary);transition:left var(--dodai-motion-duration, 160ms) var(--dodai-motion-easing, ease)")}></span>
            { for props.options.iter().enumerate().map(|(i, o)| {
                let ink = if i == sel { "var(--dodai-background)" } else { "var(--dodai-text)" };
                let cb = props.onselect.clone();
                html! {
                    <button role="radio" aria-checked={(i == sel).to_string()} onclick={Callback::from(move |_| cb.emit(i))}
                        style={format!("position:relative;border:0;background:none;font:inherit;padding:var(--dodai-space-xs) var(--dodai-space-md);cursor:pointer;color:{ink}")}>{ o.clone() }</button>
                }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct FabMenuProps {
    pub open: bool,
    #[prop_or_default]
    pub ontoggle: Callback<MouseEvent>,
    /// Items, usually [`Fab`]s with labels.
    #[prop_or_default]
    pub children: Html,
}

/// A floating button that unfolds a column of actions above it. Controlled.
#[function_component]
pub fn FabMenu(props: &FabMenuProps) -> Html {
    html! {
        <div style="display:inline-flex;flex-direction:column-reverse;align-items:flex-end;gap:var(--dodai-space-sm)">
            <Fab icon={if props.open { IconId::Close } else { IconId::Plus }} onclick={props.ontoggle.clone()} />
            if props.open { { props.children.clone() } }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ContainerProps {
    #[prop_or_default]
    pub children: Html,
}

/// A row of slides that scroll and snap.
#[function_component]
pub fn Carousel(props: &ContainerProps) -> Html {
    html! { <div style="display:flex;gap:var(--dodai-space-sm);overflow-x:auto;scroll-snap-type:x mandatory;padding-bottom:var(--dodai-space-xs)">{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct SplitPaneProps {
    pub first: Html,
    pub second: Html,
    /// Share of the first pane, 0.1–0.9.
    #[prop_or(0.5)]
    pub ratio: f32,
}

/// Two panes side by side with a divider.
#[function_component]
pub fn SplitPane(props: &SplitPaneProps) -> Html {
    let r = props.ratio.clamp(0.1, 0.9) * 100.0;
    html! {
        <div style="display:flex;width:100%;height:100%">
            <div style={format!("flex:0 0 {r}%;overflow:auto")}>{ props.first.clone() }</div>
            <div style="width:var(--dodai-border-width);background:var(--dodai-border-strong)"></div>
            <div style="flex:1;overflow:auto">{ props.second.clone() }</div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ScrollAreaProps {
    #[prop_or(AttrValue::Static("100%"))]
    pub height: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// A scrolling box with the design's thin bar.
#[function_component]
pub fn ScrollArea(props: &ScrollAreaProps) -> Html {
    html! { <div style={format!("height:{};overflow:auto;scrollbar-width:thin;scrollbar-color:var(--dodai-border-strong) transparent", props.height)}>{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct PageGridProps {
    #[prop_or(240)]
    pub min: u32,
    #[prop_or_default]
    pub children: Html,
}

/// A responsive grid whose cells are at least `min` px wide.
#[function_component]
pub fn PageGrid(props: &PageGridProps) -> Html {
    html! { <div style={format!("display:grid;grid-template-columns:repeat(auto-fill,minmax({}px,1fr));gap:var(--dodai-space-md)", props.min)}>{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct ResponsiveShellProps {
    /// Rail on wide windows.
    pub rail: Html,
    /// Bar on narrow ones.
    pub bar: Html,
    #[prop_or_default]
    pub children: Html,
}

/// The app frame: a rail beside the content on wide windows, a bar under it on narrow ones.
#[function_component]
pub fn ResponsiveShell(props: &ResponsiveShellProps) -> Html {
    html! {
        <>
            <style>{ ".dodai-shell{display:flex;flex-direction:column-reverse;min-height:100vh}.dodai-shell__rail{display:none}@media (min-width:720px){.dodai-shell{flex-direction:row}.dodai-shell__rail{display:flex}.dodai-shell__bar{display:none}}" }</style>
            <div class="dodai-shell">
                <div class="dodai-shell__bar">{ props.bar.clone() }</div>
                <div class="dodai-shell__rail">{ props.rail.clone() }</div>
                <main style="flex:1;padding:var(--dodai-space-md)">{ props.children.clone() }</main>
            </div>
        </>
    }
}

/// A vertical stream of cards.
#[function_component]
pub fn Feed(props: &ContainerProps) -> Html {
    html! { <div role="feed" style="display:flex;flex-direction:column;gap:var(--dodai-space-md)">{ props.children.clone() }</div> }
}

#[derive(Properties, PartialEq)]
pub struct TreeNavigationProps {
    pub items: Vec<TreeItem>,
}

/// A navigable tree of destinations.
#[function_component]
pub fn TreeNavigation(props: &TreeNavigationProps) -> Html {
    html! { <nav><TreeView items={props.items.clone()} /></nav> }
}

/// The provider's handle: current theme, switching and glow.
pub type DodaiTheme = crate::ThemeContext;

/// One chip per theme; pressing one selects it.
#[function_component]
pub fn ThemeSelector() -> Html {
    let Some(ctx) = use_theme() else {
        return html! {};
    };
    let cur = ctx.theme.id();
    html! {
        <div role="radiogroup" style="display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)">
            { for dodai_ui_core::themes::all().into_iter().map(|t| {
                let (id, select) = (AttrValue::from(t.id().to_string()), ctx.select.clone());
                html! { <Chip label={t.name().to_string()} selected={t.id() == cur} onclick={Callback::from(move |_| select.emit(id.clone()))} /> }
            }) }
        </div>
    }
}

/// One chip per design system (family); pressing one picks the first theme of that family.
#[function_component]
pub fn DesignSystemSelector() -> Html {
    let Some(ctx) = use_theme() else {
        return html! {};
    };
    let current = ctx.theme.meta.family;
    let mut families: Vec<(&'static str, String)> = Vec::new();
    for t in dodai_ui_core::themes::all() {
        if !families.iter().any(|(f, _)| *f == t.meta.family) {
            families.push((t.meta.family, t.id().to_string()));
        }
    }
    html! {
        <div role="radiogroup" style="display:flex;flex-wrap:wrap;gap:var(--dodai-space-xs)">
            { for families.into_iter().map(|(f, first)| {
                let (id, select) = (AttrValue::from(first), ctx.select.clone());
                html! { <Chip label={f} selected={f == current} onclick={Callback::from(move |_| select.emit(id.clone()))} /> }
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DodaiTooltipProps {
    pub text: AttrValue,
    #[prop_or_default]
    pub children: Html,
}

/// A tooltip bubble that shows on hover or focus of its trigger (CSS only).
#[function_component]
pub fn DodaiTooltip(props: &DodaiTooltipProps) -> Html {
    html! {
        <span class="dodai-tip" style="position:relative;display:inline-block">
            <style>{ ".dodai-tip>.dodai-tip__bubble{display:none}.dodai-tip:hover>.dodai-tip__bubble,.dodai-tip:focus-within>.dodai-tip__bubble{display:block}" }</style>
            { props.children.clone() }
            <span class="dodai-tip__bubble" role="tooltip" style="position:absolute;bottom:100%;left:50%;transform:translateX(-50%);margin-bottom:var(--dodai-space-xs);white-space:nowrap;z-index:41">
                <KitBox kit={Kit::Tooltip { rich: false }}>{ props.text.clone() }</KitBox>
            </span>
        </span>
    }
}

/// A halo from the design's glow colour around its children.
#[function_component]
pub fn GlowLayer(props: &ContainerProps) -> Html {
    html! { <div style="filter:drop-shadow(0 0 var(--dodai-glow-blur, 8px) var(--dodai-glow, transparent))">{ props.children.clone() }</div> }
}

/// The design's scanline texture over its parent (set `position:relative` on the parent).
#[function_component]
pub fn Scanlines() -> Html {
    html! { <div aria-hidden="true" style="position:absolute;inset:0;pointer-events:none;opacity:var(--dodai-scanlines);background:repeating-linear-gradient(to bottom,transparent 0,transparent 2px,rgba(0,0,0,.28) 3px)"></div> }
}

#[derive(Properties, PartialEq)]
pub struct FrostLayerProps {
    #[prop_or(16)]
    pub blur: u32,
}

/// A frosted-glass backdrop over its parent (set `position:relative` on the parent).
#[function_component]
pub fn FrostLayer(props: &FrostLayerProps) -> Html {
    let b = props.blur;
    html! { <div aria-hidden="true" style={format!("position:absolute;inset:0;pointer-events:none;backdrop-filter:blur({b}px);-webkit-backdrop-filter:blur({b}px);background:color-mix(in srgb, var(--dodai-surface) 40%, transparent)")}></div> }
}

/// A scene whose content sits on a frosted layer.
#[function_component]
pub fn FrostedScene(props: &ContainerProps) -> Html {
    html! {
        <div style="position:relative;border-radius:var(--dodai-radius);overflow:hidden">
            <FrostLayer />
            <div style="position:relative;padding:var(--dodai-space-md)">{ props.children.clone() }</div>
        </div>
    }
}
