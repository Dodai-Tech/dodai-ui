//! Actions: buttons, groups, split buttons, toolbar, snackbar.

use dodai_core::{ButtonVariant as V, ControlSize};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Actions";

pub(crate) const SECTIONS: [Section; 3] = [
    Section::new(CAT, "Buttons · IconButton · FAB", buttons).heading("Buttons"),
    Section::new(CAT, "Button groups · Split button", groups).when(material_only),
    Section::new(CAT, "Toolbar · Snackbar", toolbar),
];

/// Sections only Material ships (its button groups and split button).
pub(crate) fn material_only(t: &dodai_core::Theme) -> bool {
    t.meta.family == "material"
}

/// Counter key of the primary button.
const CLICKS: &str = "btn-count";

fn buttons(c: &Ctx) -> Node {
    let mut first = vec![
        button("Primary", V::Primary).counter(CLICKS).node(),
        button("Secondary", V::Secondary).node(),
        button("Elevated", V::Primary).elevated().node(),
        button("Outline", V::Outline).node(),
        button("Outline 2", V::OutlineSecondary).node(),
        button("Scrim", V::Scrim).node(),
    ];
    let crops = crops(c);
    if !crops.is_empty() {
        first.push(row(crops));
    }
    first.extend([
        button("Danger", V::Danger).node(),
        button("Ghost", V::Ghost).node(),
        button("Disabled", V::Primary).disabled().node(),
        button("Loading…", V::Primary).shimmer().node(),
        button("Outline…", V::Outline).shimmer().node(),
        muted(format!("primary clicked {}×", c.values.index_of(CLICKS, 0))),
    ]);
    let sizes = |square: bool| {
        row(ControlSize::ALL
            .iter()
            .map(|&z| {
                if square {
                    button(z.name(), V::Secondary).size(z).square().node()
                } else {
                    button(z.name(), V::Primary).size(z).node()
                }
            })
            .collect())
    };
    let icon_sizes = row(ControlSize::ALL
        .iter()
        .map(|&z| button("", V::Secondary).icon(IconId::Edit).size(z).node())
        .collect());
    col(vec![row(first), sizes(false), sizes(true), icon_sizes])
}

/// Each design's own crops: cut corners with a tick, a leaf, connected ends, a ticket stub.
/// Ends the design has no crop for are left out; none at all gives no row.
fn crops(c: &Ctx) -> Vec<Node> {
    use dodai_core::{Corners, Crop};
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
        out.push(
            button("Both cuts", V::Outline)
                .crop(Crop::Left)
                .corners(Corners::DIAGONAL)
                .node(),
        );
        out.push(
            button("Both cuts", V::Primary)
                .crop(Crop::Right)
                .corners(Corners::ANTI_DIAGONAL)
                .node(),
        );
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
    out.extend([
        button("", V::Outline)
            .icon(IconId::Edit)
            .crop(Crop::Left)
            .node(),
        button("", V::Primary)
            .icon(IconId::Heart)
            .crop(Crop::Right)
            .node(),
        Node::Fab {
            label: None,
            crop: Some(Crop::Left),
        },
        Node::Fab {
            label: Some("Create".into()),
            crop: Some(Crop::Right),
        },
    ]);
    out
}

fn groups(c: &Ctx) -> Node {
    use dodai_core::Crop;
    let split = |variant: V, open: bool, crop: Option<Crop>| Node::Split {
        label: "Save".into(),
        variant,
        open,
        crop,
    };
    let mut splits = vec![
        split(V::Primary, false, None),
        split(V::Secondary, false, None),
        split(V::Outline, true, None),
    ];
    if c.theme.look().cut_corners {
        splits.push(split(V::Primary, false, Some(Crop::Left)));
        splits.push(split(V::Primary, false, Some(Crop::Right)));
    }
    col(vec![
        muted("Standard"),
        Node::Choice(choice(GROUP, &["Day", "Week", "Month"], 0)),
        muted("Connected (the selected item is a pill)"),
        Node::Choice(choice(GROUP, &["Day", "Week", "Month", "Year"], 0).connected()),
        muted("Split button"),
        row(splits),
    ])
}

/// The button groups share one selection, as in the gpui showcase.
const GROUP: &str = "grp-sel";

fn toolbar(_: &Ctx) -> Node {
    use IconId::{Copy, Edit, Heart};
    col(vec![
        Node::Toolbar {
            floating: false,
            vibrant: false,
            icons: vec![Edit, Copy],
        },
        row(vec![
            Node::Toolbar {
                floating: true,
                vibrant: false,
                icons: vec![Edit, Copy, Heart],
            },
            Node::Toolbar {
                floating: true,
                vibrant: true,
                icons: vec![Edit, Copy],
            },
        ]),
        Node::Snackbar {
            text: "Message archived".into(),
            action: "Undo".into(),
        },
    ])
}
