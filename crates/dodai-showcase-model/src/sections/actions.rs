//! Actions: buttons, groups, split buttons, toolbar, snackbar.

use dodai_core::{ButtonVariant as V, ControlSize};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Actions";

pub(crate) const SECTIONS: [Section; 3] = [
    Section {
        category: CAT,
        title: "Buttons",
        build: buttons,
    },
    Section {
        category: CAT,
        title: "Button groups · Split button",
        build: groups,
    },
    Section {
        category: CAT,
        title: "Toolbar · Snackbar",
        build: toolbar,
    },
];

fn buttons(c: &Ctx) -> Node {
    let variants = [
        ("Primary", V::Primary),
        ("Secondary", V::Secondary),
        ("Outline", V::Outline),
        ("Outline 2", V::OutlineSecondary),
        ("Scrim", V::Scrim),
    ];
    let mut first = vec![];
    for (l, v) in variants {
        first.push(button(l, v).node());
    }
    let mut second = vec![
        button("Buy now", V::Outline).node(),
        button("Danger", V::Danger).node(),
        button("Ghost", V::Ghost).node(),
        button("Disabled", V::Secondary).disabled().node(),
        button("Loading…", V::Primary).loading().node(),
        button("Outline…", V::Outline).loading().node(),
    ];
    let hits = c.values.index_of("btn-count", 0);
    second.push(muted(format!("primary clicked {hits}×")));
    let icons = vec![
        button("Create", V::Primary).icon(IconId::Edit).node(),
        button("", V::Secondary).icon(IconId::Edit).node(),
        button("", V::Primary).icon(IconId::Heart).node(),
        button("", V::Outline).icon(IconId::Copy).square().node(),
    ];
    let sizes = |v: V, square: bool| {
        row(ControlSize::ALL
            .iter()
            .map(|&z| {
                let b = button(z.name(), v).size(z);
                if square { b.square() } else { b }.node()
            })
            .collect())
    };
    let first = {
        let mut f = first;
        if let Some(Node::Button(b)) = f.first_mut() {
            b.counter = Some("btn-count".into());
        }
        f
    };
    rows(vec![
        row(first),
        row(second),
        row(icons),
        sizes(V::Primary, false),
        sizes(V::Secondary, true),
    ])
}

fn groups(_: &Ctx) -> Node {
    col(vec![
        muted("Standard"),
        Node::Choice(choice("grp-std", &["Day", "Week", "Month"], 0)),
        muted("Connected (the selected item is a pill)"),
        Node::Choice(choice("grp-conn", &["Day", "Week", "Month", "Year"], 1).connected()),
        muted("Split button"),
        row(vec![
            Node::Split {
                label: "Save".into(),
                variant: V::Primary,
            },
            Node::Split {
                label: "Save".into(),
                variant: V::Secondary,
            },
            Node::Split {
                label: "Save".into(),
                variant: V::Outline,
            },
        ]),
    ])
}

fn toolbar(_: &Ctx) -> Node {
    let icon = |i: IconId| button("", V::Ghost).icon(i).node();
    col(vec![
        line(vec![icon(IconId::Edit), icon(IconId::Copy)]),
        row(vec![
            Node::Card {
                title: String::new(),
                body: Box::new(line(vec![
                    icon(IconId::Edit),
                    icon(IconId::Copy),
                    icon(IconId::Heart),
                ])),
                variant: dodai_core::CardVariant::Elevated,
            },
            Node::Card {
                title: String::new(),
                body: Box::new(line(vec![icon(IconId::Edit), icon(IconId::Copy)])),
                variant: dodai_core::CardVariant::Filled,
            },
        ]),
        Node::Snackbar {
            text: "Message archived".into(),
            action: "Undo".into(),
        },
    ])
}
