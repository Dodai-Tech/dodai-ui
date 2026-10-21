//! The "Foundations" category sections.

use super::super::*;
use dodai_gpui::IconId;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_foundations_sec(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        #[allow(unused_macros)]
        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .subheader(self.chips($t, &t, &me, cx))
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
            };
        }
        let mut out: Vec<Section> = Vec::new();

        out.push((
            "Foundations",
            "Text · Heading · Label · Mono",
            section!("Text")
                .child(Heading::new("Heading 1").level(1))
                .child(Heading::new("Heading 2"))
                .child(Heading::new("Heading 3").level(3))
                .child(Label::new("Label text"))
                .child(Muted::new("Muted text"))
                .child(Mono::new("mono_text() // 0xDEADBEEF"))
                .into_any_element(),
            "Text",
        ));
        out.push((
            "Foundations",
            "Colour palette",
            section!("Colour palette")
                .child(Muted::new(
                    "Every Palette token of the active design system.",
                ))
                .child(palette_view(&t))
                .into_any_element(),
            "Colour palette",
        ));
        out.push((
            "Foundations",
            "Type scale",
            section!("Type scale")
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
        ));
        out.push((
            "Foundations",
            "Motion tokens",
            section!("Motion tokens")
                .child(Muted::new(
                    "Material 3 duration scale (bar = duration) and named easing curves.",
                ))
                .child(motion_tokens_view(&t))
                .into_any_element(),
            "Motion tokens",
        ));
        out.push((
            "Foundations",
            "Spacing · Elevation",
            section!("Spacing · Elevation")
                .child(Muted::new(
                    "Spacing scale (bars are 8x the value) and surface layers.",
                ))
                .child(spacing_elevation_view(&t))
                .into_any_element(),
            "Spacing · Elevation",
        ));
        out.push((
            "Foundations",
            "Shape tokens",
            section!("Shape tokens")
                .child(Muted::new(
                    "Read-only summary of the theme's shape and effect tokens.",
                ))
                .child(shape_tokens_view(&t))
                .into_any_element(),
            "Shape tokens",
        ));
        let set = t.look().icons;
        let query = self.icon_search.read(cx).text().to_string();
        let count = dodai_gpui::IconId::ALL.len();
        let intro = format!(
            "{count} icons, the same names in every design system; each draws them in its own style. Write one as `Icon::Heart`. Click an icon for its SVG and code."
        );
        let mut section = section!("Icons").child(Muted::new(intro)).child(
            div()
                .w(px(360.0))
                .max_w_full()
                .child(self.icon_search.clone()),
        );
        if let Some(panel) = self.icon_detail(&t, cx) {
            section = section.child(panel);
        }
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
        if t.look().icons == dodai_core::IconSet::Stroked && query.trim().is_empty() {
            section = section.child(modifier_variants_view(&t));
        }
        out.push(("Foundations", "Icons", section.into_any_element(), "Icons"));
        out
    }
}

impl Showcase {
    /// The Material Symbols catalogue grouped by Material's own categories. A category shows a
    /// preview until opened; a search lists every match.
    fn icon_browser(
        &mut self,
        t: &dodai_core::Theme,
        query: &str,
        cx: &mut Context<Self>,
    ) -> gpui::Div {
        // Phosphor's categories, then whatever the shared names hold beyond them (the Dodai
        // names Phosphor does not have), so each icon is listed once.
        let mut source: Vec<(&'static str, Vec<IconId>)> = dodai_phosphor_icons::CATEGORIES
            .iter()
            .map(|&(c, ids)| (c, ids.to_vec()))
            .collect();
        let listed: std::collections::HashSet<IconId> =
            source.iter().flat_map(|(_, v)| v.iter().copied()).collect();
        let rest: Vec<IconId> = IconId::ALL
            .iter()
            .copied()
            .filter(|i| !listed.contains(i))
            .collect();
        source.push(("other", rest));
        let tag = "ic";
        const PREVIEW: usize = 24;
        const MAX_HITS: usize = 480;
        let q = query.trim().to_lowercase().replace(' ', "_");
        let searching = !q.is_empty();
        let mut out = div().flex().flex_col().gap(px(t.spacing.lg));
        let mut total = 0usize;
        for (cat, picks) in source {
            let hits: Vec<IconId> = picks
                .iter()
                .copied()
                .filter(|p| !searching || p.name().contains(q.as_str()))
                .collect();
            if hits.is_empty() {
                continue;
            }
            let open = self.icon_open.contains(cat);
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
                    id: SharedString::from(format!("{tag}-{cat}-{i}")),
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
                    cat.to_uppercase(),
                    hits.len()
                )))
                .child(grid);
            if !searching && hits.len() > PREVIEW {
                block = block.child(
                    div().flex().child(
                        Button::new(
                            SharedString::from(format!("{tag}-more-{cat}")),
                            if open {
                                "Show fewer".to_string()
                            } else {
                                format!("Show all {}", hits.len())
                            },
                        )
                        .variant(ButtonVariant::Secondary)
                        .on_click(cx.listener(move |s, _, _, cx| {
                            if !s.icon_open.remove(cat) {
                                s.icon_open.insert(cat);
                            }
                            cx.notify();
                        })),
                    ),
                );
            }
            out = out.child(dodai_gpui::lazy(
                SharedString::from(format!("{tag}-cat-{cat}")),
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
    /// The selected icon: preview, a download, copy actions, and the SVG source on request.
    fn icon_detail(&mut self, t: &dodai_core::Theme, cx: &mut Context<Self>) -> Option<gpui::Div> {
        let pick = self.icon_sel?;
        let name = pick.name().to_string();
        let filled = self.icon_filled;
        let svg = dodai::icons::glyph(pick, t, filled)
            .map(|g| g.svg(24, "currentColor"))
            .unwrap_or_default();
        let svg = pretty_svg(&svg);
        let usage = format!("Icon::{pick:?}");
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
                            .size(px(88.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .border_1()
                            .border_color(hsla(t.palette.border))
                            .child(
                                dodai_gpui::Icon::new(pick)
                                    .size(56.0)
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
    icon: IconId,
    filled: bool,
    selected: bool,
    on_click: std::rc::Rc<dyn Fn(&mut gpui::App)>,
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
                dodai_gpui::Icon::new(self.icon)
                    .size(28.0)
                    .filled(self.filled)
                    .color(primary),
            )
            .child({
                // Centred when it fits; long names stay left-aligned so the ellipsis clips
                // the end, not both sides.
                let mut l = div()
                    .w_full()
                    .px(px(4.0))
                    .truncate()
                    .text_size(px(t.typography.size_xs))
                    .text_color(label)
                    .child(n);
                if n.len() <= 12 {
                    l = l.text_center();
                }
                l
            })
    }
}
