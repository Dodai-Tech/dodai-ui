//! Navigation: tabs, tree, breadcrumb, pagination, stepper, menus.

use dodai_core::{ButtonVariant as V, CardVariant, ControlSize};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Bind, Node};

const CAT: &str = "Navigation";

pub(crate) const SECTIONS: [Section; 12] = [
    Section::new(
        CAT,
        "Popover · Hover card · Context menu · Dropdown menu",
        popovers,
    )
    .heading("Popover · Hover card · Context menu · Dropdown"),
    Section::new(CAT, "Menu · Sub-menu", menu),
    Section::new(CAT, "Tabs", tabs),
    Section::new(CAT, "Tree", tree_section),
    Section::new(CAT, "Collapsible sidebar", sidebar),
    Section::new(CAT, "Segmented · Breadcrumb · Pagination · Stepper", strips),
    Section::new(CAT, "Carousel", carousel),
    Section::new(CAT, "Snap stack", snap_stack)
        .heading("Snap stack · Vertical pages · Scroll pages"),
    Section::new(CAT, "Modal navigation drawer", drawer),
    Section::new(CAT, "Dialog · Bottom sheet · Side sheet", sheets),
    Section::new(CAT, "App bars", app_bars),
    Section::new(CAT, "App bar · Cards · FAB · Navigation", bars_cards),
];

fn tabs(c: &Ctx) -> Node {
    let t = choice("tabs-main", &["Overview", "Usage", "Billing", "Team"], 0);
    let at = c.values.index(&t.bind);
    col(vec![
        Node::Tabs(t),
        body(match at {
            0 => "Overview: the state of everything at a glance.",
            1 => "Usage: what was used and by whom.",
            2 => "Billing: invoices and payment methods.",
            _ => "Team: people and their roles.",
        }),
    ])
}

fn tree_section(_: &Ctx) -> Node {
    use IconId::*;
    let leaf = |l: &str| tree(l, None, vec![]);
    let docs = vec![
        tree(
            "Getting started",
            Some(Home),
            vec![tree(
                "Install",
                Some(Folder),
                vec![
                    tree("Requirements", Some(Info), vec![]),
                    tree("Feature flags", Some(Settings), vec![]),
                ],
            )],
        ),
        tree(
            "Guides",
            Some(Layers),
            vec![
                tree(
                    "Theming",
                    Some(Folder),
                    vec![
                        leaf("Pick a design"),
                        leaf("Tune shape").selected(),
                        leaf("Control colour"),
                    ],
                ),
                tree("Components", Some(Folder), vec![leaf("Tabs"), leaf("Tree")]).closed(),
            ],
        ),
        tree(
            "Reference",
            Some(Code),
            vec![tree(
                "Tokens",
                Some(Folder),
                vec![leaf("Colour"), leaf("Spacing")],
            )
            .closed()],
        )
        .closed(),
    ];
    let systems = vec![
        tree(
            "Dodai",
            None,
            vec![
                leaf("Light").selected(),
                leaf("Dark"),
                leaf("Ocean"),
                leaf("Forest"),
            ],
        ),
        tree("Material", None, vec![leaf("Light"), leaf("Dark")]).closed(),
        tree("Cyberpunk", None, vec![leaf("Yellow"), leaf("Cyan")]).closed(),
    ];
    let files = || {
        vec![
            tree(
                "crates",
                None,
                vec![tree(
                    "dodai-gpui",
                    None,
                    vec![leaf("tabs.rs"), leaf("layout.rs")],
                )],
            ),
            leaf("README.md"),
        ]
    };
    let card = |title: &str, note: &str, nodes: Vec<crate::node::TreeNode>| {
        Node::Stack(crate::node::Stack {
            axis: crate::node::Axis::Column,
            wrap: false,
            gap: crate::node::Gap::Sm,
            centre: false,
            width: Some(240.0),
            items: vec![label(title), muted(note), Node::Tree(nodes)],
        })
    };
    col(vec![
        muted(
            "TreeNavigation folds a branch on click and selects a leaf; it keeps its own state and changes nothing outside itself. Nesting is unlimited and each node can carry an icon.",
        ),
        top_row(vec![
            card("Three levels, with icons", "Section > group > page.", docs),
            card("Two levels", "Design systems and their themes.", systems),
            card("Files", "Folders and files, no icons.", files()),
            card(
                "TreeView (controlled)",
                "You own the folded state and the selection.",
                files(),
            ),
        ]),
    ])
}

fn strips(c: &Ctx) -> Node {
    col(vec![
        line(vec![Node::Choice(
            choice("group-sel", &["Day", "Week", "Month"], 1).connected(),
        )]),
        Node::Breadcrumb(["Home", "Library", "Data"].map(String::from).to_vec()),
        Node::Pagination {
            pages: 20,
            bind: Bind::new("page", 4),
        },
        Node::Steps {
            steps: vec![
                step("Account", None),
                step("Profile", None),
                step("Done", None),
            ],
            current: 1,
        },
        accordion(
            c,
            "acc",
            &[
                ("What is Dodai?", "A themeable design system for Rust UIs."),
                (
                    "Does it support Material 3?",
                    "Yes, tonal roles, shapes and state layers.",
                ),
            ],
            &[0],
        ),
    ])
}

/// Disclosure rows: a chevron and the title, the body under an open one. Item `i` is open
/// while `{key}-{i}` is set (the ones in `open` at first).
fn accordion(c: &Ctx, key: &str, items: &[(&str, &str)], open: &[usize]) -> Node {
    let mut rows = Vec::new();
    for (i, (title, text)) in items.iter().enumerate() {
        let k = format!("{key}-{i}");
        let shown = c.values.flag(&Bind::new(&k, open.contains(&i)));
        rows.push(line(vec![button(*title, V::Ghost)
            .icon(if shown {
                IconId::ChevronDown
            } else {
                IconId::ChevronRight
            })
            .plain()
            .counter(crate::commands::flip(&k, open.contains(&i)))
            .node()]));
        if shown {
            rows.push(muted(*text));
        }
    }
    stack_col(rows)
}

/// A column with small gaps.
fn stack_col(items: Vec<Node>) -> Node {
    Node::Stack(crate::node::Stack {
        axis: crate::node::Axis::Column,
        wrap: false,
        gap: crate::node::Gap::Sm,
        centre: false,
        width: None,
        items,
    })
}

fn menu(c: &Ctx) -> Node {
    use crate::menus::{item, Entry};
    col(vec![
        muted("Leading icons, shortcuts, and a sub-menu (hover \"Share\")."),
        line(vec![crate::menus::menu(
            c,
            240.0,
            vec![
                item("Edit").icon(IconId::Edit).shortcut("⌘E").entry(),
                item("Copy").icon(IconId::Copy).shortcut("⌘C").entry(),
                item("Share").icon(IconId::Heart).sub().entry(),
                Entry::Separator,
                item("Delete").danger().entry(),
            ],
        )]),
    ])
}

fn popovers(c: &Ctx) -> Node {
    use crate::commands::flip;
    use crate::menus::{item, menu, Entry};
    let p = &c.theme.palette;
    let anchor = |key: &str, trigger: Node, panel: Node| Node::Anchor {
        open: Bind::new(key, false),
        trigger: Box::new(trigger),
        panel: Box::new(panel),
    };
    let popover = Node::Popup(Box::new(Node::Stack(crate::node::Stack {
        axis: crate::node::Axis::Column,
        wrap: false,
        gap: crate::node::Gap::Md,
        centre: false,
        width: Some(240.0),
        items: vec![
            label("Popover"),
            muted("Click outside or press Esc to close."),
            line(vec![button("Action", V::Primary).node()]),
        ],
    })));
    let card = Node::Popup(Box::new(line(vec![
        Node::Avatar("DD".into()),
        col(vec![label("Dodai UI"), muted("Design system for Rust UIs")]),
    ])));
    row(vec![
        anchor(
            "pop",
            button("Popover", V::Outline)
                .counter(flip("pop", false))
                .node(),
            popover,
        ),
        anchor(
            "hover-card",
            styled(
                "@hover-me",
                c.theme.typography.size_md,
                crate::node::Font::Body,
                crate::node::Ink::Link,
            ),
            card,
        ),
        anchor(
            "ctx",
            Node::Block {
                w: 150.0,
                h: 46.0,
                fill: p.background,
                border: Some(p.border_strong),
                radius: c.theme.shape.radius.min(8.0),
                label: Some("Right-click here".into()),
            },
            menu(
                c,
                200.0,
                vec![
                    item("Copy").shortcut("⌘C").entry(),
                    item("Paste").shortcut("⌘V").entry(),
                    Entry::Separator,
                    item("Delete").danger().entry(),
                ],
            ),
        ),
        anchor(
            "dd",
            button("Actions", V::Outline)
                .icon(IconId::ChevronDown)
                .counter(flip("dd", false))
                .node(),
            menu(
                c,
                200.0,
                vec![
                    item("Rename").entry(),
                    item("Duplicate").entry(),
                    Entry::Separator,
                    item("Archive").disabled().entry(),
                    item("Delete").danger().entry(),
                ],
            ),
        ),
    ])
}

/// Home, Mail (3), Saved and Settings: the destinations of the bar, rail and drawer demos.
const DESTINATIONS: [(IconId, &str, Option<&str>); 4] = [
    (IconId::Home, "Home", None),
    (IconId::Mail, "Mail", Some("3")),
    (IconId::Star, "Saved", None),
    (IconId::Settings, "Settings", None),
];

/// The page behind an overlay demo.
fn behind() -> Node {
    muted("Page content behind the overlay.")
}

/// A scrim over the page under a modal layer.
fn scrim(c: &Ctx) -> Option<dodai_core::Color> {
    let mut s = c.theme.palette.overlay;
    s.a = 140;
    Some(s)
}

fn drawer(c: &Ctx) -> Node {
    let at = c.values.index_of("nav", 0);
    let rows = DESTINATIONS
        .iter()
        .enumerate()
        .map(|(i, (icon, name, badge))| {
            let r = list_row(*name).icon(*icon).selected(i == at);
            match badge {
                Some(b) => r.trailing(*b),
                None => r,
            }
        })
        .collect();
    col(vec![
        muted("Scrim + drawer from the edge; picking an item dismisses."),
        Node::Frame {
            h: 260.0,
            body: Box::new(behind()),
            scrim: scrim(c),
            layer: Box::new(Node::Card {
                title: String::new(),
                body: Box::new(Node::Stack(crate::node::Stack {
                    axis: crate::node::Axis::Column,
                    wrap: false,
                    gap: crate::node::Gap::Xs,
                    centre: false,
                    width: Some(260.0),
                    items: vec![Node::List(rows)],
                })),
                variant: CardVariant::Elevated,
                surface: dodai_core::Surface::INHERIT,
            }),
            place: crate::node::Place::Start,
        },
    ])
}

fn sheets(c: &Ctx) -> Node {
    use crate::node::{Axis, Gap, Place, Stack};
    let fixed = |w: f32, items: Vec<Node>| {
        Node::Stack(Stack {
            axis: Axis::Column,
            wrap: false,
            gap: Gap::Md,
            centre: false,
            width: Some(w),
            items,
        })
    };
    let card = |body: Node| Node::Card {
        title: String::new(),
        body: Box::new(body),
        variant: CardVariant::Elevated,
        surface: dodai_core::Surface::INHERIT,
    };
    let dialog = card(fixed(
        360.0,
        vec![
            Node::Stack(Stack {
                axis: Axis::Column,
                wrap: false,
                gap: Gap::Sm,
                centre: true,
                width: None,
                items: vec![
                    Node::Icon(IconId::Heart, 24.0),
                    text(
                        "Discard draft?",
                        crate::TextRole::Type(dodai_core::TypeRole::HeadlineSmall),
                    ),
                ],
            }),
            muted("Your unsent message will be lost. This cannot be undone."),
            line(vec![
                Node::Grow,
                button("Cancel", V::Ghost).node(),
                button("Discard", V::Ghost).node(),
            ]),
        ],
    ));
    let bottom = card(Node::Stack(Stack {
        axis: Axis::Column,
        wrap: false,
        gap: Gap::Md,
        centre: true,
        width: None,
        items: vec![
            Node::Block {
                w: 32.0,
                h: 4.0,
                fill: c.theme.palette.text_muted,
                border: None,
                radius: 2.0,
                label: None,
            },
            text(
                "Share",
                crate::TextRole::Type(dodai_core::TypeRole::TitleLarge),
            ),
            muted("Copy link · Message · Mail"),
            Node::Space(40.0),
            // Spans the frame's width.
            Node::Grow,
        ],
    }));
    let side = card(fixed(
        240.0,
        vec![
            line(vec![
                text(
                    "Filters",
                    crate::TextRole::Type(dodai_core::TypeRole::TitleLarge),
                ),
                Node::Grow,
                button("", V::Ghost).icon(IconId::Close).plain().node(),
            ]),
            muted("Date range · Owner · Status"),
        ],
    ));
    let frame = |h: f32, layer: Node, place: Place| Node::Frame {
        h,
        body: Box::new(behind()),
        scrim: scrim(c),
        layer: Box::new(layer),
        place,
    };
    col(vec![
        frame(300.0, dialog, Place::Centre),
        frame(260.0, bottom, Place::Bottom),
        frame(260.0, side, Place::End),
    ])
}

fn top_bar(title: &str, left: IconId, right: &[IconId]) -> Node {
    let mut items = vec![
        button("", V::Ghost).icon(left).node(),
        text(
            title,
            crate::TextRole::Type(dodai_core::TypeRole::TitleLarge),
        ),
        Node::Space(0.0),
    ];
    items.extend(right.iter().map(|&i| button("", V::Ghost).icon(i).node()));
    Node::Card {
        title: String::new(),
        body: Box::new(line(items)),
        variant: CardVariant::Filled,
        surface: dodai_core::Surface::INHERIT,
    }
}

fn app_bars(_: &Ctx) -> Node {
    use IconId::*;
    col(vec![
        muted("Small"),
        top_bar("Inbox", Menu, &[Search, Settings]),
        muted("Centre aligned"),
        top_bar("Library", Home, &[Heart]),
        muted("Large"),
        Node::Card {
            title: String::new(),
            body: Box::new(col(vec![
                line(vec![button("", V::Ghost).icon(Menu).node()]),
                text(
                    "Settings",
                    crate::TextRole::Type(dodai_core::TypeRole::HeadlineMedium),
                ),
            ])),
            variant: CardVariant::Filled,
            surface: dodai_core::Surface::INHERIT,
        },
    ])
}

fn bars_cards(_: &Ctx) -> Node {
    use IconId::*;
    col(vec![
        top_bar("Dodai", Menu, &[Search]),
        row(vec![
            button("+", V::Primary)
                .size(ControlSize::Large)
                .square()
                .node(),
            button("Compose", V::Primary)
                .icon(Edit)
                .size(ControlSize::Large)
                .node(),
        ]),
        Node::Card {
            title: "Navigation bar".into(),
            body: Box::new(Node::Choice(
                choice("navbar", &["Home", "Search", "Library", "Profile"], 0).connected(),
            )),
            variant: CardVariant::Outlined,
            surface: dodai_core::Surface::INHERIT,
        },
        row(vec![
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Elevated")),
                variant: CardVariant::Elevated,
                surface: dodai_core::Surface::INHERIT,
            },
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Filled")),
                variant: CardVariant::Filled,
                surface: dodai_core::Surface::INHERIT,
            },
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Outlined")),
                variant: CardVariant::Outlined,
                surface: dodai_core::Surface::INHERIT,
            },
        ]),
        Node::List(vec![
            list_row("Home").icon(Home),
            list_row("Library").icon(Folder),
            list_row("Settings").icon(Settings),
        ]),
    ])
}

fn sidebar(c: &Ctx) -> Node {
    use IconId::*;
    let rail = c.values.flag(&Bind::new("demo-rail", false));
    // "Drafts" has no icon on purpose: collapsed, it shows its first letter.
    let items = [
        ("Inbox", Some(Inbox)),
        ("Drafts", None),
        ("Sent", Some(Send)),
        ("Archive", Some(Archive)),
    ];
    let mut side = vec![line(vec![button(
        if rail { "»" } else { "« Collapse" },
        V::Outline,
    )
    .counter(crate::commands::flip("demo-rail", false))
    .node()])];
    if rail {
        side.extend(items.iter().map(|(name, icon)| match icon {
            Some(i) => line(vec![Node::Space(8.0), Node::Icon(*i, 20.0)]),
            None => line(vec![Node::Space(12.0), label(&name[..1])]),
        }));
    } else {
        side.push(Node::List(
            items
                .iter()
                .map(|(name, icon)| {
                    let r = list_row(*name);
                    match icon {
                        Some(i) => r.icon(*i),
                        None => r,
                    }
                })
                .collect(),
        ));
    }
    col(vec![
        muted(
            "ResponsiveShell: drag the edge handle (spring + fling snap), click it, or use the button. Below the breakpoint it becomes a drawer.",
        ),
        Node::Card {
            title: String::new(),
            body: Box::new(Node::Stack(crate::node::Stack {
                axis: crate::node::Axis::Row,
                wrap: false,
                gap: crate::node::Gap::Md,
                centre: false,
                width: None,
                items: vec![
                    Node::Stack(crate::node::Stack {
                        axis: crate::node::Axis::Column,
                        wrap: false,
                        gap: crate::node::Gap::Xs,
                        centre: false,
                        width: Some(if rail { 64.0 } else { 200.0 }),
                        items: side,
                    }),
                    Node::Divider,
                    muted("Content area follows the sidebar width."),
                ],
            })),
            variant: CardVariant::Outlined,
            surface: dodai_core::Surface::INHERIT,
        },
    ])
}

/// The role colours the carousel and snap-stack tiles take, with their names.
fn tiles(c: &Ctx) -> [(dodai_core::Color, &'static str); 5] {
    let r = &c.theme.palette.roles;
    [
        (r.primary_container, "Primary"),
        (r.secondary_container, "Secondary"),
        (r.tertiary_container, "Tertiary"),
        (r.surface_container_high, "Surface"),
        (r.error_container, "Error"),
    ]
}

/// A coloured tile `w`×`h` px with its name.
fn tile(c: &Ctx, (fill, name): (dodai_core::Color, &str), w: f32, h: f32) -> Node {
    Node::Block {
        w,
        h,
        fill,
        border: None,
        radius: c.theme.role_shape(dodai_core::ShapeRole::Carousel).radius,
        label: Some(name.into()),
    }
}

fn carousel(c: &Ctx) -> Node {
    let key = "carousel";
    let at = c.values.index_of(key, 1).min(4);
    let all = tiles(c);
    // Item widths of a 3-up strip per layout: equal, multi-browse (large, medium, small), hero
    // (large and a peek) and centre hero (peeks either side).
    let strip = |widths: &[f32], h: f32| {
        let arrows = |i: IconId, to: usize| {
            button("", V::Ghost)
                .icon(i)
                .plain()
                .counter(crate::commands::set(key, to))
                .node()
        };
        let mut items = vec![arrows(IconId::ChevronLeft, at.saturating_sub(1))];
        items.extend(
            widths
                .iter()
                .enumerate()
                .map(|(n, w)| tile(c, all[(at + n) % all.len()], *w, h)),
        );
        items.push(arrows(IconId::ChevronRight, (at + 1).min(all.len() - 1)));
        col(vec![
            line(items),
            Node::Pagination {
                pages: all.len(),
                bind: Bind::new(key, 1),
            },
        ])
    };
    col(vec![
        strip(&[150.0, 150.0, 150.0], 120.0),
        muted("Multi-browse"),
        strip(&[240.0, 150.0, 60.0], 140.0),
        muted("Hero"),
        strip(&[400.0, 60.0], 140.0),
        muted("Center-aligned hero"),
        strip(&[40.0, 380.0, 40.0], 140.0),
    ])
}

fn snap_stack(c: &Ctx) -> Node {
    let key = "snap";
    let pages = ["Page one", "Page two", "Page three", "Page four"];
    let at = c.values.index_of(key, 0).min(pages.len() - 1);
    let r = &c.theme.palette.roles;
    let fills = [
        r.primary_container,
        r.secondary_container,
        r.tertiary_container,
        r.error_container,
    ];
    let chevron = |i: IconId, to: usize| {
        button("", V::Ghost)
            .icon(i)
            .plain()
            .counter(crate::commands::set(key, to))
            .node()
    };
    col(vec![
        muted(
            "Scroll the wheel or swipe vertically over the stack to turn a page, or use the chevrons; at either end the scroll passes on to the page.",
        ),
        line(vec![
            tile(c, (fills[at], pages[at]), 300.0, 140.0),
            Node::Stack(crate::node::Stack {
                axis: crate::node::Axis::Column,
                wrap: false,
                gap: crate::node::Gap::Xs,
                centre: true,
                width: None,
                items: vec![
                    chevron(IconId::ChevronUp, at.saturating_sub(1)),
                    chevron(IconId::ChevronDown, (at + 1).min(pages.len() - 1)),
                ],
            }),
        ]),
    ])
}
