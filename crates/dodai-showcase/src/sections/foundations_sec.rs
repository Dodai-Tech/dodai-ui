//! The "Foundations" category sections.

use super::super::*;
use dodai_gpui::IconId;

impl Showcase {
    /// The builders of this category, one per group of sections (built lazily).
    pub(crate) fn groups_foundations_sec() -> Vec<Group> {
        vec![Self::token_sections, Self::icons_section]
    }

    fn token_sections(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        vec![(
            "Foundations",
            "Text · Heading · Label · Mono",
            self.section("Text", cx)
                .child(Heading::new("Heading 1").level(1))
                .child(Heading::new("Heading 2"))
                .child(Heading::new("Heading 3").level(3))
                .child(Label::new("Label text"))
                .child(Muted::new("Muted text"))
                .child(Mono::new("mono_text() // 0xDEADBEEF"))
                .into_any_element(),
            "Text",
        ),
        (
            "Foundations",
            "Colour palette",
            self.section("Colour palette", cx)
                .child(Muted::new(
                    "Every Palette token of the active design system.",
                ))
                .child(palette_view(&t))
                .into_any_element(),
            "Colour palette",
        ),
        (
            "Foundations",
            "Type scale",
            self.section("Type scale", cx)
                .child(Muted::new(
                    "Sizes and families from the theme's Typography.",
                ))
                .child(type_view(&t))
                .child(Muted::new(
                    "Type scale roles (Material 3: exact; others derive them)",
                ))
                .child(type_scale_view(&t))
                .into_any_element(),
            "Type scale",
        ),
        (
            "Foundations",
            "Motion tokens",
            self.section("Motion tokens", cx)
                .child(Muted::new(
                    "Material 3 duration scale (bar = duration) and named easing curves.",
                ))
                .child(motion_tokens_view(&t))
                .into_any_element(),
            "Motion tokens",
        ),
        (
            "Foundations",
            "Spacing · Elevation",
            self.section("Spacing · Elevation", cx)
                .child(Muted::new(
                    "Spacing scale (bars are 8x the value) and surface layers.",
                ))
                .child(spacing_elevation_view(&t))
                .into_any_element(),
            "Spacing · Elevation",
        ),
        (
            "Foundations",
            "Layout · Fluid scales",
            self.section("Layout · Fluid scales", cx)
                .child(Muted::new(
                    "Type and spacing resolved at five viewport widths, the page grid at 800 px, and the media fade.",
                ))
                .child(layout_view(&t))
                .into_any_element(),
            "Layout · Fluid scales",
        ),
        (
            "Foundations",
            "Frosted glass",
            self.section("Frosted glass", cx)
                .child(Muted::new(
                    "Panels over a picture: a blurred copy of it under the design's frosted tint.",
                ))
                .child(frosted_view(&t, self.viewport_w, self.side_width(&t)))
                .into_any_element(),
            "Frosted glass",
        ),
        (
            "Foundations",
            "Shape tokens",
            self.section("Shape tokens", cx)
                .child(Muted::new(
                    "Read-only summary of the theme's shape and effect tokens.",
                ))
                .child(shape_tokens_view(&t))
                .into_any_element(),
            "Shape tokens",
        )]
    }

    fn icons_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
        let query = self.icon_search.read(cx).text().to_string();
        let (named, symbols) = (IconId::ALL.len(), dodai_icons::symbol::Symbol::ALL.len());
        let total = icon_total();
        let intro = format!(
            "{total} icons: {named} named icons shared by every design, each drawn in that design's own style (write one as `Icon::Heart`), plus {symbols} dashboard icons in {} groups, one drawing stroked in each design's own weight and corners. Click an icon for its SVG and code.",
            dodai_gpui::icons::SymbolGroup::ALL.len()
        );
        let mut section = self.section("Icons", cx).child(Muted::new(intro)).child(
            div()
                .w_full()
                .max_w(px(360.0))
                .max_w_full()
                .child(self.icon_search.clone()),
        );
        let filled_toggle = {
            let me_f = me.clone();
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(
                    Checkbox::new("ib-fill", self.icon_filled).on_change(move |v, _, cx| {
                        me_f.update(cx, |s, cx| {
                            s.icon_filled = v;
                            cx.notify();
                        })
                    }),
                )
                .child(Label::new("Filled"))
        };
        section = section
            .child(filled_toggle)
            .child(self.icon_browser(&t, &query, cx));
        if t.look().icons == dodai_core::IconSet::Stroked
            && query.trim().is_empty()
            && self.icon_pick == IconPick::All
        {
            section = section.child(modifier_variants_view(&t));
        }
        out.push(("Foundations", "Icons", section.into_any_element(), "Icons"));
        out
    }
}

impl Showcase {
    /// The one icon catalogue: every catalogue category and every symbol group, drawn the same
    /// way. A block shows a preview until opened; a search lists every match.
    fn icon_browser(
        &mut self,
        t: &dodai_core::Theme,
        query: &str,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        let tag = "ic";
        const PREVIEW: usize = 24;
        const MAX_HITS: usize = 480;
        let q = query.trim().to_lowercase().replace([' ', '-'], "_");
        let searching = !q.is_empty();
        let mut out = div().flex().flex_col().gap(px(t.spacing.lg));
        let mut total = 0usize;
        for src in icon_sources() {
            if !self.icon_pick.shows(src.pick) {
                continue;
            }
            let forced = self.icon_pick == src.pick;
            let hits: Vec<IconEntry> = src
                .entries
                .iter()
                .copied()
                .filter(|e| !searching || e.name().replace('-', "_").contains(q.as_str()))
                .collect();
            if hits.is_empty() {
                continue;
            }
            let key = src.pick;
            let open = forced || self.icon_open.contains(&key);
            let cap = if searching {
                MAX_HITS.saturating_sub(total)
            } else if open {
                hits.len()
            } else {
                PREVIEW
            };
            total += hits.len().min(cap);
            let mut grid = div().flex().flex_wrap().gap(px(t.spacing.xs));
            for (i, &pick) in hits.iter().take(cap).enumerate() {
                let me = cx.entity();
                grid = grid.child(IconTile {
                    id: SharedString::from(format!("{tag}-{}-{i}", src.label)),
                    icon: pick,
                    filled: self.icon_filled,
                    selected: self.icon_sel == Some(pick),
                    on_click: std::rc::Rc::new(move |cx: &mut gpui::App| {
                        me.update(cx, |s, cx| {
                            s.icon_sel = Some(pick);
                            s.icon_msg = None;
                            cx.notify();
                        })
                    }),
                });
            }
            let mut block = div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.md))
                .child(Muted::new(format!(
                    "{} · {}",
                    src.label.to_uppercase(),
                    hits.len()
                )))
                .child(grid);
            if !searching && !forced && hits.len() > PREVIEW {
                block = block.child(
                    div().flex().child(
                        Button::new(
                            SharedString::from(format!("{tag}-more-{}", src.label)),
                            if open {
                                "Show fewer".to_string()
                            } else {
                                format!("Show all {}", hits.len())
                            },
                        )
                        .variant(ButtonVariant::Secondary)
                        .on_click(cx.listener(move |s, _, _, cx| {
                            if !s.icon_open.remove(&key) {
                                s.icon_open.insert(key);
                            }
                            cx.notify();
                        })),
                    ),
                );
            }
            out = out.child(dodai_gpui::lazy(
                SharedString::from(format!("{tag}-cat-{}", src.label)),
                block,
            ));
        }
        if total == 0 {
            out = out.child(Muted::new("No icons match."));
        }
        out
    }
}

impl Showcase {
    /// The selected icon's panel, kept above the page (outside the scroll) while the Icons
    /// section is the one being read, so its download and copy buttons stay in reach.
    pub(crate) fn pinned_icon_panel(
        &mut self,
        t: &dodai_core::Theme,
        cx: &mut Context<Self>,
    ) -> Option<gpui::AnyElement> {
        if self.side_spied.0 != Some("Icons") {
            return None;
        }
        let panel = self.icon_detail(t, cx)?;
        Some(
            div()
                .flex_none()
                .px(px(t.spacing.lg))
                .py(px(t.spacing.sm))
                .border_b_1()
                .border_color(hsla(t.palette.border))
                .child(panel.id("ico-pinned").max_h(px(300.0)).overflow_y_scroll())
                .into_any_element(),
        )
    }

    /// The selected icon: preview, a download, copy actions, and the SVG source on request.
    fn icon_detail(&mut self, t: &dodai_core::Theme, cx: &mut Context<Self>) -> Option<gpui::Div> {
        let pick = self.icon_sel?;
        let name = pick.name().to_string();
        let filled = self.icon_filled;
        let svg = pretty_svg(&pick.svg(t, filled));
        let usage = pick.usage();
        let file = name.replace(
            |c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'),
            "_",
        );
        let (svg_dl, svg_cp, usage_cp, name_cp) =
            (svg.clone(), svg.clone(), usage.clone(), name.clone());
        let mut panel = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .p(px(t.spacing.md))
            .border_1()
            .border_color(hsla(t.palette.border))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(t.spacing.md))
                    .child(
                        div()
                            .size(px(64.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(hsla(t.palette.border))
                            .child(
                                pick.icon()
                                    .size(40.0)
                                    .filled(filled)
                                    .color(hsla(t.palette.primary)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(t.spacing.sm))
                            .child(Label::new(name.clone()))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(t.spacing.sm))
                                    .child(Button::new("ico-dl", "Download SVG").on_click(
                                        cx.listener(move |s, _, _, cx| {
                                            let dir = std::env::var_os("HOME")
                                                .map(std::path::PathBuf::from)
                                                .unwrap_or_default()
                                                .join("Downloads");
                                            let path = dir.join(format!("{file}.svg"));
                                            s.icon_msg = Some(
                                                match std::fs::create_dir_all(&dir)
                                                    .and_then(|_| std::fs::write(&path, &svg_dl))
                                                {
                                                    Ok(()) => format!("Saved {}", path.display()),
                                                    Err(e) => format!("Could not save: {e}"),
                                                },
                                            );
                                            cx.notify();
                                        }),
                                    ))
                                    .child(
                                        Button::new("ico-cp-svg", "Copy SVG")
                                            .variant(ButtonVariant::Secondary)
                                            .on_click(cx.listener(move |s, _, _, cx| {
                                                cx.write_to_clipboard(
                                                    gpui::ClipboardItem::new_string(svg_cp.clone()),
                                                );
                                                s.icon_msg = Some("SVG copied".into());
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("ico-cp-name", "Copy Rust")
                                            .variant(ButtonVariant::Secondary)
                                            .on_click(cx.listener(move |s, _, _, cx| {
                                                cx.write_to_clipboard(
                                                    gpui::ClipboardItem::new_string(
                                                        usage_cp.clone(),
                                                    ),
                                                );
                                                s.icon_msg = Some(format!("Copied `{usage_cp}`"));
                                                let _ = &name_cp;
                                                cx.notify();
                                            })),
                                    )
                                    .child(
                                        Button::new("ico-close", "Close")
                                            .variant(ButtonVariant::Secondary)
                                            .on_click(cx.listener(|s, _, _, cx| {
                                                s.icon_sel = None;
                                                s.icon_msg = None;
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            );
        if let Some(m) = &self.icon_msg {
            panel = panel.child(Muted::new(m.clone()));
        }
        // Below the selected icon, same footer position as the other sections.
        panel = panel.child(div().flex().justify_end().child(code_toggle(
            "ico-code",
            t,
            self.icon_code,
            {
                let me = cx.entity();
                move |on, _, cx| {
                    me.update(cx, |s, cx| {
                        s.icon_code = on;
                        cx.notify();
                    })
                }
            },
        )));
        if self.icon_code {
            panel = panel
                .child(dodai_gpui::CodeBlock::new("ico-usage", usage).language("rust"))
                .child(dodai_gpui::CodeBlock::new("ico-svg", svg).language("xml"));
        }
        Some(panel)
    }
}

/// Lay the one-line SVG out for reading: attributes and path data on their own lines, wrapped at
/// spaces (still valid SVG, and the same drawing).
fn pretty_svg(svg: &str) -> String {
    let flat = svg
        .replace(" xmlns=", "\n  xmlns=")
        .replace(" width=", "\n  width=")
        .replace(" height=", "\n  height=")
        .replace(" viewBox=", "\n  viewBox=")
        .replace("><path", ">\n  <path")
        .replace(" d=\"", "\n    d=\"")
        .replace("/></svg>", "/>\n</svg>");
    let mut out = String::new();
    for line in flat.lines() {
        let mut width = 0;
        for (i, word) in line.split(' ').enumerate() {
            if i > 0 {
                if width + word.len() > 96 {
                    out.push_str("\n      ");
                    width = 6;
                } else {
                    out.push(' ');
                    width += 1;
                }
            }
            out.push_str(word);
            width += word.len();
        }
        out.push('\n');
    }
    out.trim_end().to_string()
}

/// One tile of the icon grid. The selection fades in (fill, border, label) instead of snapping,
/// so it needs the window's frame clock: hence a component, not a plain `div`.
#[derive(IntoElement)]
struct IconTile {
    id: SharedString,
    icon: IconEntry,
    filled: bool,
    selected: bool,
    on_click: std::rc::Rc<dyn Fn(&mut gpui::App)>,
}

/// How many characters of an icon name fit a tile (its label width over the average advance of
/// the extra-small type).
const TILE_NAME_CHARS: usize = 13;

/// `name` cut to `max` characters with an ellipsis when longer.
fn shortened(name: &str, max: usize) -> SharedString {
    if name.chars().count() <= max {
        return SharedString::new(name.to_string());
    }
    let mut s: String = name.chars().take(max.saturating_sub(1)).collect();
    s.push('…');
    s.into()
}

impl RenderOnce for IconTile {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let t = dodai_gpui::theme_of(cx);
        let sel = dodai_gpui::motion::transition(
            window,
            cx,
            SharedString::from(format!("{}-sel", self.id)).into(),
            self.selected,
        );
        let mix = dodai_gpui::motion::mix;
        let primary = hsla(t.palette.primary);
        let border = mix(hsla(t.palette.border), primary, sel);
        let fill = mix(gpui::transparent_black(), primary.opacity(0.14), sel);
        let label = mix(hsla(t.palette.text_muted), primary, sel);
        let n = self.icon.name();
        let on_click = self.on_click;
        div()
            .id(self.id)
            .flex()
            .flex_col()
            .items_center()
            .gap(px(4.0))
            .w(px(92.0))
            .py(px(t.spacing.sm))
            .cursor_pointer()
            .border_1()
            .border_color(border)
            .bg(fill)
            .on_click(move |_, _, cx| on_click(cx))
            .child(
                self.icon
                    .icon()
                    .size(28.0)
                    .filled(self.filled)
                    .color(primary),
            )
            .child(
                // Shortened here, once: gpui's own truncation re-measures on every layout pass,
                // and this grid holds hundreds of names.
                div()
                    .w_full()
                    .px(px(4.0))
                    .whitespace_nowrap()
                    .overflow_hidden()
                    .text_center()
                    .text_size(px(t.typography.size_xs))
                    .text_color(label)
                    .child(shortened(n, TILE_NAME_CHARS)),
            )
    }
}

/// One icon of the catalogue: a named icon every design draws, or an original symbol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum IconEntry {
    /// A catalogue icon (`Icon::Heart`), drawn in each design's own style.
    Named(IconId),
    /// An original symbol (`Icon::symbol(Symbol::OilCan)`), one drawing in every design.
    Symbol(dodai_gpui::icons::Symbol),
}

impl IconEntry {
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Named(i) => i.name(),
            Self::Symbol(s) => s.name(),
        }
    }
    /// The entry called `name`, if any.
    /// A name that is also a named icon (`ambulance`) finds the named icon; `symbol:ambulance`
    /// finds the dashboard symbol.
    pub(crate) fn find(name: &str) -> Option<Self> {
        if let Some(n) = name.strip_prefix("symbol:") {
            return dodai_icons::symbol::Symbol::ALL
                .iter()
                .find(|s| s.name() == n)
                .map(|&s| Self::Symbol(s));
        }
        IconId::ALL
            .iter()
            .find(|i| i.name() == name)
            .map(|&i| Self::Named(i))
            .or_else(|| {
                dodai_icons::symbol::Symbol::ALL
                    .iter()
                    .find(|s| s.name() == name)
                    .map(|&s| Self::Symbol(s))
            })
    }
    fn icon(self) -> dodai_gpui::Icon {
        match self {
            Self::Named(i) => dodai_gpui::Icon::new(i),
            Self::Symbol(s) => dodai_gpui::Icon::symbol(s),
        }
    }
    /// How to write it in Rust.
    fn usage(self) -> String {
        match self {
            Self::Named(i) => format!("Icon::{i:?}"),
            Self::Symbol(s) => format!("Icon::symbol(Symbol::{s:?})"),
        }
    }
    /// Its SVG in the theme's icon pack.
    fn svg(self, t: &dodai_core::Theme, filled: bool) -> String {
        match self {
            Self::Named(i) => dodai_icons::glyph_for_theme(i, t, filled),
            Self::Symbol(s) => {
                Some(s.glyph_styled(dodai_icons::Pack::from(t.look().icons).symbol_style()))
            }
        }
        .map(|g| g.svg(24, "currentColor"))
        .unwrap_or_default()
    }
}

/// Every icon the Icons section lists: the shared catalogue plus the original symbols.
pub(crate) fn icon_total() -> usize {
    IconId::ALL.len() + dodai_icons::symbol::Symbol::ALL.len()
}

/// One block of the catalogue: a category of named icons or a group of symbols.
struct IconSource {
    pick: IconPick,
    label: String,
    entries: Vec<IconEntry>,
}

/// Every block, by label, so no kind or group is privileged by its position in the source:
/// Phosphor's categories, whatever the shared names hold beyond them (the Dodai names Phosphor
/// does not have) and the symbol groups.
fn icon_sources() -> Vec<IconSource> {
    let mut named: Vec<(&'static str, Vec<IconId>)> = dodai_icons::CATEGORIES
        .iter()
        .map(|&(c, ids)| (c, ids.to_vec()))
        .collect();
    let listed: std::collections::HashSet<IconId> =
        named.iter().flat_map(|(_, v)| v.iter().copied()).collect();
    let rest: Vec<IconId> = IconId::ALL
        .iter()
        .copied()
        .filter(|i| !listed.contains(i))
        .collect();
    named.push(("other", rest));
    let mut out: Vec<IconSource> = named
        .into_iter()
        .map(|(c, ids)| IconSource {
            pick: IconPick::Category(c),
            label: capitalised(c),
            entries: ids.into_iter().map(IconEntry::Named).collect(),
        })
        .collect();
    for g in dodai_gpui::icons::SymbolGroup::ALL {
        out.push(IconSource {
            pick: IconPick::Group(g.name()),
            label: capitalised(g.name()),
            entries: dodai_icons::symbol::Symbol::ALL
                .iter()
                .filter(|s| s.group() == *g)
                .map(|&s| IconEntry::Symbol(s))
                .collect(),
        });
    }
    out.retain(|s| !s.entries.is_empty());
    out.sort_by(|a, b| a.label.cmp(&b.label));
    out
}

/// The block of the Icons sidebar list that was picked: what the Icons section shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum IconPick {
    /// Everything.
    #[default]
    All,
    /// One category of named icons.
    Category(&'static str),
    /// One symbol group.
    Group(&'static str),
}

impl IconPick {
    /// What a second click on a picked block returns to.
    pub(crate) fn up(self) -> Self {
        Self::All
    }
    fn shows(self, block: Self) -> bool {
        self == Self::All || self == block
    }
}

/// One row of the Icons list in the sidebar.
pub(crate) struct IconRow {
    /// Nesting level (3: below the section's own jump row).
    pub(crate) depth: u8,
    pub(crate) label: String,
    pub(crate) count: usize,
    pub(crate) pick: IconPick,
}

fn capitalised(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().chain(c).collect())
        .unwrap_or_default()
}

/// The rows under the Icons section: one per block of the single catalogue.
pub(crate) fn icon_tree() -> Vec<IconRow> {
    icon_sources()
        .into_iter()
        .map(|s| IconRow {
            depth: 3,
            label: s.label,
            count: s.entries.len(),
            pick: s.pick,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_is_catalogue_plus_symbols() {
        assert_eq!(
            icon_total(),
            IconId::ALL.len() + dodai_icons::symbol::Symbol::ALL.len()
        );
    }

    #[test]
    fn tree_counts_add_up_to_the_total() {
        assert_eq!(
            icon_tree().iter().map(|r| r.count).sum::<usize>(),
            icon_total()
        );
    }

    #[test]
    fn blocks_are_sorted_by_label_not_by_kind() {
        let labels: Vec<_> = icon_tree().into_iter().map(|r| r.label).collect();
        let mut sorted = labels.clone();
        sorted.sort();
        assert_eq!(labels, sorted);
    }

    #[test]
    fn symbols_are_found_by_name() {
        assert!(matches!(
            IconEntry::find("oil-can"),
            Some(IconEntry::Symbol(_))
        ));
    }

    #[test]
    fn picking_filters_and_folds_up() {
        let m = IconPick::Group("medical");
        assert!(m.shows(m));
        assert!(!m.shows(IconPick::Group("vehicle")));
        assert!(!m.shows(IconPick::Category("other")));
        assert!(IconPick::All.shows(m));
        assert_eq!(m.up(), IconPick::All);
    }
}
