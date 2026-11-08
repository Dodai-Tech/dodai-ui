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
    let crops = crops(c);
    let mut second = vec![
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
    let mut rs = vec![row(first)];
    if !crops.is_empty() {
        rs.push(row(crops));
    }
    rs.extend([
        row(second),
        row(icons),
        sizes(V::Primary, false),
        sizes(V::Secondary, true),
    ]);
    rows(rs)
}

/// Each design's own crops: cut corners with a tick, a leaf, connected ends, a ticket stub.
/// Ends the design has no crop for are left out; none at all gives an empty row.
fn crops(c: &Ctx) -> Vec<Node> {
    use dodai_core::Crop;
    let t = c.theme;
    if t.button_crops().is_empty() {
        return vec![];
    }
    let has = |e: Crop| t.button_crop(e).is_some();
    let cut = t.look().cut_corners;
    let mut out = vec![
        button("Buy now", V::Scrim).crop(Crop::Left).node(),
        button("Watch trailer", V::Scrim).crop(Crop::Right).node(),
    ];
    if has(Crop::Top) {
        out.push(button("Top", V::Outline).crop(Crop::Top).node());
    }
    if has(Crop::Bottom) {
        out.push(button("Bottom", V::Primary).crop(Crop::Bottom).node());
    }
    if cut {
        out.push(button("Both cuts", V::Outline).crop(Crop::Left).node());
        out.push(button("Both cuts", V::Primary).crop(Crop::Right).node());
    }
    for (i, (l, v)) in [
        ("Primary", V::Primary),
        ("Secondary", V::Secondary),
        ("Outline", V::Outline),
        ("Danger", V::Danger),
    ]
    .into_iter()
    .enumerate()
    {
        let end = if i % 2 == 0 { Crop::Left } else { Crop::Right };
        out.push(button(l, v).crop(end).node());
    }
    out.push(
        button("", V::Outline)
            .icon(IconId::Edit)
            .crop(Crop::Left)
            .node(),
    );
    out.push(
        button("", V::Primary)
            .icon(IconId::Heart)
            .crop(Crop::Right)
            .node(),
    );
    out
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
