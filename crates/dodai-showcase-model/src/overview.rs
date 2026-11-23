//! The "All" landing page above the sections (the gpui showcase's overview): a hero with the
//! counts, and a card per category that opens it or jumps to one of its sections.

use dodai_core::{ButtonVariant as V, ControlSize, Theme, Tone};

use crate::b::*;
use crate::catalog::{self, CATEGORIES};
use crate::icons::IconRef;
use crate::node::{Font, Ink, Node};
use crate::{Axis, Gap, Stack};

/// Press key prefix opening a category (`go:c:<index>`).
pub const OPEN_CATEGORY: &str = "go:c:";
/// Press key prefix jumping to a section (`go:s:<title>`).
pub const OPEN_SECTION: &str = "go:s:";

/// Categories that animate all the time and so only show on their own page.
pub const OWN_PAGE: [&str; 3] = ["Loading", "Charts", "Shaders"];

/// What a category holds, in a line.
pub fn blurb(name: &str) -> &'static str {
    match name {
        "Foundations" => "Design tokens: colour, type, spacing.",
        "Shape" => "Corners, borders and cuts.",
        "Surface effects" => "Theme-wide glow, plus textures, glitch, sheen and bloom.",
        "Actions" => "Buttons and triggers.",
        "Inputs" => "Text, choice, date and form controls.",
        "Navigation" => "Menus, tabs, overlays and sidebars.",
        "Data display" => "Tables, progress, media, time and content.",
        "Charts" => "Statistical, market and radial charts.",
        "Animation" => "Entry effects and physics.",
        "Shaders" => "Pixel-shader backgrounds and effects.",
        "Loading" => "Spinners and skeletons.",
        _ => "",
    }
}

/// The overview for `theme`, naming the toolkit that draws it (`here`, highlighted).
pub fn overview(theme: &Theme, here: &str) -> Node {
    let shown: Vec<&catalog::Section> = catalog::sections()
        .iter()
        .filter(|s| (s.shown)(theme))
        .collect();
    let shaders = shown.iter().filter(|s| s.category == "Shaders").count();
    let stats = [
        (shown.len(), "components"),
        (CATEGORIES.len(), "categories"),
        (dodai_core::themes::all().len(), "themes"),
        (crate::icons::total(), "icons"),
        (shaders, "shaders"),
    ];
    let stat_row = row(stats
        .iter()
        .map(|(n, label)| {
            tile(
                120.0,
                vec![
                    styled(
                        n.to_string(),
                        theme.typography.size_xxl,
                        Font::Display,
                        Ink::Primary,
                    ),
                    muted(label.to_uppercase()),
                ],
            )
        })
        .collect());
    let toolkits = row([
        "gpui", "iced", "egui", "slint", "ratatui", "dioxus", "leptos", "yew", "bevy", "web",
    ]
    .iter()
    .map(|n| tag(*n, Tone::Info, *n == here))
    .collect());
    let ty = &theme.typography;
    let hero = Node::Framed {
        border: theme.panel_border,
        body: Box::new(
            col(vec![
                muted("DESIGN SYSTEM · RUST · MULTI-TOOLKIT"),
                Node::Stack(Stack {
                    axis: Axis::Row,
                    wrap: false,
                    gap: Gap::Md,
                    centre: true,
                    width: None,
                    items: vec![
                        Node::Logo(ty.size_xxl * 2.2),
                        styled("Dodai UI", ty.size_xxl * 1.5, Font::Display, Ink::Text),
                    ],
                }),
                styled(
                    "One set of tokens, components, charts and shaders. Swappable design systems, every toolkit.",
                    ty.size_lg,
                    Font::Body,
                    Ink::Muted,
                ),
                toolkits,
                Node::Divider,
                stat_row,
            ])
            .gap(Gap::Md),
        ),
    };
    let cards = CATEGORIES
        .iter()
        .enumerate()
        .map(|(ci, name)| {
            let titles: Vec<&str> = shown
                .iter()
                .filter(|s| s.category == *name)
                .map(|s| s.title)
                .collect();
            let n = titles.len();
            let head = line(vec![
                Node::Glyph {
                    icon: IconRef::Named(catalog::category_icon(name)),
                    size: 20.0,
                    filled: false,
                    ink: theme.palette.primary,
                },
                col(vec![
                    heading(*name, 3),
                    muted(format!("{n} section{}", if n == 1 { "" } else { "s" })),
                ])
                .gap(Gap::Xs),
            ])
            .gap(Gap::Md);
            let chips = row(titles
                .iter()
                .take(4)
                .map(|t| {
                    let short = t.split(" · ").next().unwrap_or(t);
                    button(short, V::Outline)
                        .size(ControlSize::Small)
                        .plain()
                        .counter(format!("{OPEN_SECTION}{t}"))
                        .node()
                })
                .collect())
            .gap(Gap::Xs);
            let open = if n > 4 {
                format!("Open all {n} →")
            } else {
                "Open →".to_string()
            };
            tile(
                300.0,
                vec![Node::Framed {
                    border: theme.panel_border,
                    body: Box::new(
                        col(vec![
                            head,
                            muted(blurb(name)),
                            chips,
                            row(vec![button(open, V::Ghost)
                                .size(ControlSize::Small)
                                .plain()
                                .counter(format!("{OPEN_CATEGORY}{ci}"))
                                .node()]),
                        ])
                        .gap(Gap::Md),
                    ),
                }],
            )
        })
        .collect();
    col(vec![
        hero,
        heading("Browse by category", 2),
        top_row(cards).gap(Gap::Md),
        heading("Everything, in one scroll", 2),
        muted("Every component below. Loading and Shaders animate constantly, so they live on their own pages."),
    ])
    .gap(Gap::Lg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_category_has_a_blurb_and_a_card() {
        for c in CATEGORIES {
            assert!(!blurb(c).is_empty(), "{c}");
        }
        let t = Theme::default();
        let mut opens = 0;
        overview(&t, "iced").walk(&mut |n| {
            if let Node::Button(b) = n {
                if b.counter
                    .as_deref()
                    .is_some_and(|k| k.starts_with(OPEN_CATEGORY))
                {
                    opens += 1;
                }
            }
        });
        assert_eq!(opens, CATEGORIES.len());
    }
}
