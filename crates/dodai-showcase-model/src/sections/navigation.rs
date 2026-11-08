//! Navigation: tabs, tree, breadcrumb, pagination, stepper, menus.

use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Bind, Node};

const CAT: &str = "Navigation";

pub(crate) const SECTIONS: [Section; 4] = [
    Section {
        category: CAT,
        title: "Tabs",
        build: tabs,
    },
    Section {
        category: CAT,
        title: "Tree",
        build: tree_section,
    },
    Section {
        category: CAT,
        title: "Segmented · Breadcrumb · Pagination · Stepper",
        build: strips,
    },
    Section {
        category: CAT,
        title: "Menu · Sub-menu",
        build: menu,
    },
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
