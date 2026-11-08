//! Navigation: tabs, tree, breadcrumb, pagination, stepper, menus.

use dodai_core::{ButtonVariant as V, CardVariant, ControlSize};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Bind, Node};

const CAT: &str = "Navigation";

pub(crate) const SECTIONS: [Section; 11] = [
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
    Node::Tree(vec![
        tree(
            "src",
            Some(Folder),
            vec![
                tree(
                    "components",
                    Some(Folder),
                    vec![
                        tree("button.rs", Some(File), vec![]),
                        tree("tabs.rs", Some(File), vec![]),
                    ],
                ),
                tree("main.rs", Some(File), vec![]),
            ],
        ),
        tree("Cargo.toml", Some(File), vec![]),
        tree("README.md", Some(File), vec![]),
    ])
}

fn strips(_: &Ctx) -> Node {
    col(vec![
        Node::Choice(choice("seg", &["Map", "Satellite", "Terrain"], 0).connected()),
        Node::Breadcrumb(
            ["Home", "Library", "Data", "Report"]
                .map(String::from)
                .to_vec(),
        ),
        Node::Pagination {
            pages: 7,
            bind: Bind::new("page", 2),
        },
        Node::Steps {
            steps: vec![
                step("Account", Some("Create a login")),
                step("Profile", Some("Tell us about you")),
                step("Review", None),
                step("Done", None),
            ],
            current: 1,
        },
    ])
}

fn menu(_: &Ctx) -> Node {
    Node::List(vec![
        list_row("New file").icon(IconId::File).trailing("⌘N"),
        list_row("Open…").icon(IconId::Folder).trailing("⌘O"),
        list_row("Save").icon(IconId::Check).trailing("⌘S"),
        list_row("Settings").icon(IconId::Settings).trailing("⌘,"),
    ])
}

fn popover_menu(rows: Vec<crate::node::ListRow>) -> Node {
    Node::Popup(Box::new(Node::List(rows)))
}

fn popovers(_: &Ctx) -> Node {
    use IconId::*;
    col(vec![
        muted("Popover"),
        Node::Popup(Box::new(col(vec![
            text("Share this page", crate::TextRole::Label),
            muted("Anyone with the link can view."),
            button("Copy link", V::Secondary)
                .size(ControlSize::Small)
                .icon(Copy)
                .node(),
        ]))),
        muted("Hover card"),
        Node::Popup(Box::new(line(vec![
            Node::Avatar("AL".into()),
            col(vec![body("Ada Lovelace"), muted("Analyst · joined 1843")]),
        ]))),
        muted("Context menu"),
        popover_menu(vec![
            list_row("Cut").icon(Edit).trailing("⌘X"),
            list_row("Copy").icon(Copy).trailing("⌘C"),
            list_row("Paste").icon(File).trailing("⌘V"),
        ]),
        muted("Dropdown"),
        popover_menu(vec![list_row("Day"), list_row("Week"), list_row("Month")]),
    ])
}

fn dialog_body(title: &str, text: &str) -> Node {
    col(vec![
        crate::b::text(
            title,
            crate::TextRole::Type(dodai_core::TypeRole::TitleLarge),
        ),
        muted(text),
        row(vec![
            button("Cancel", V::Ghost).node(),
            button("Confirm", V::Primary).node(),
        ]),
    ])
}

fn sheets(_: &Ctx) -> Node {
    col(vec![
        muted("Dialog"),
        Node::Card {
            title: String::new(),
            body: Box::new(dialog_body("Discard draft?", "Your changes will be lost.")),
            variant: CardVariant::Elevated,
        },
        muted("Bottom sheet"),
        Node::Card {
            title: String::new(),
            body: Box::new(col(vec![
                list_row_node("Share", IconId::Copy),
                list_row_node("Edit", IconId::Edit),
                list_row_node("Favourite", IconId::Heart),
            ])),
            variant: CardVariant::Filled,
        },
        muted("Side sheet"),
        Node::Card {
            title: "Filters".into(),
            body: Box::new(col(vec![
                toggle(
                    dodai_core::ToggleKind::Switch,
                    "side-a",
                    true,
                    "Only unread",
                )
                .node(),
                toggle(dodai_core::ToggleKind::Switch, "side-b", false, "Starred").node(),
                button("Apply", V::Primary).node(),
            ])),
            variant: CardVariant::Outlined,
        },
    ])
}

fn list_row_node(label: &str, icon: IconId) -> Node {
    Node::List(vec![list_row(label).icon(icon)])
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
        },
        row(vec![
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Elevated")),
                variant: CardVariant::Elevated,
            },
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Filled")),
                variant: CardVariant::Filled,
            },
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Outlined")),
                variant: CardVariant::Outlined,
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
    let open = c.values.flag(&crate::Bind::new("sidebar-open", true));
    col(vec![
        chip(
            if open { "Expanded" } else { "Collapsed" },
            "sidebar-open",
            true,
        )
        .icon(Menu)
        .node(),
        if open {
            Node::List(vec![
                list_row("Home").icon(Home).trailing("12"),
                list_row("Library").icon(Folder).trailing("3"),
                list_row("Settings").icon(Settings),
            ])
        } else {
            line(vec![
                button("", V::Ghost).icon(Home).node(),
                button("", V::Ghost).icon(Folder).node(),
                button("", V::Ghost).icon(Settings).node(),
            ])
        },
    ])
}

fn carousel(c: &Ctx) -> Node {
    let key = "carousel";
    let at = c.values.index_of(key, 0).min(3);
    let slides = ["Aurora", "Dune", "Harbour", "Pine"];
    col(vec![
        Node::Card {
            title: slides[at].into(),
            body: Box::new(muted(format!("Slide {} of {}", at + 1, slides.len()))),
            variant: CardVariant::Filled,
        },
        Node::Pagination {
            pages: slides.len(),
            bind: crate::Bind::new(key, 0),
        },
    ])
}

fn drawer(_: &Ctx) -> Node {
    use IconId::*;
    Node::Card {
        title: "Dodai".into(),
        body: Box::new(col(vec![
            Node::List(vec![
                list_row("Inbox").icon(Mail).trailing("24"),
                list_row("Starred").icon(Star),
                list_row("Sent").icon(File),
                list_row("Archive").icon(Archive),
            ]),
            Node::Divider,
            Node::List(vec![list_row("Settings").icon(Settings)]),
        ])),
        variant: CardVariant::Elevated,
    }
}
