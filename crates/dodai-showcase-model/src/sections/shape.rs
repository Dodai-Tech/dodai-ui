//! Shape: how the design's corners, borders and elevation look on the common components.

use dodai_core::{ButtonVariant as V, CardVariant, ControlSize, Tone};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Shape";

pub(crate) const SECTIONS: [Section; 4] = [
    Section {
        category: CAT,
        title: "Shape gallery",
        build: gallery,
    },
    Section {
        category: CAT,
        title: "Roundness · Borders",
        build: roundness,
    },
    Section {
        category: CAT,
        title: "Edges · Depth",
        build: depth,
    },
    Section {
        category: CAT,
        title: "Button crop",
        build: crop,
    },
];

fn gallery(_: &Ctx) -> Node {
    col(vec![
        muted("The design's own shape on every kind of component."),
        row(vec![
            button("Primary", V::Primary).node(),
            button("Outline", V::Outline).node(),
            button("", V::Secondary).icon(IconId::Edit).node(),
            tag("tag", Tone::Info, true),
            tag("soft", Tone::Success, false),
        ]),
        row(vec![
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Elevated")),
                variant: CardVariant::Elevated,
            },
            Node::Card {
                title: "Card".into(),
                body: Box::new(muted("Outlined")),
                variant: CardVariant::Outlined,
            },
            input("shape-field", "Field").node(),
        ]),
    ])
}

fn roundness(c: &Ctx) -> Node {
    let t = c.theme;
    let p = &t.palette;
    let sh = &t.shape;
    let block = |label: String, radius: f32, width: f32| Node::Block {
        w: 96.0 + width * 4.0,
        h: 56.0,
        fill: p.surface_raised,
        border: Some(p.border_strong),
        radius,
        label: Some(label),
    };
    col(vec![
        body(format!(
            "Corner radius {:.0} px · chamfer {:.0} px · border {:.0} px",
            sh.radius, sh.chamfer, sh.border_width
        )),
        row(vec![
            block("0".into(), 0.0, 0.0),
            block(format!("{:.0}", sh.radius * 0.5), sh.radius * 0.5, 0.0),
            block(format!("{:.0}", sh.radius), sh.radius, 0.0),
            block(format!("{:.0}", sh.radius * 2.0), sh.radius * 2.0, 0.0),
            block("pill".into(), 28.0, 0.0),
        ]),
        row(ControlSize::ALL
            .iter()
            .map(|&z| button(z.name(), V::Outline).size(z).node())
            .collect()),
    ])
}

fn depth(c: &Ctx) -> Node {
    let t = c.theme;
    row(vec![
        Node::Card {
            title: "Elevated".into(),
            body: Box::new(muted(format!(
                "hard shadow {:.0} px",
                t.effects.hard_shadow
            ))),
            variant: CardVariant::Elevated,
        },
        Node::Card {
            title: "Filled".into(),
            body: Box::new(muted("flat, raised")),
            variant: CardVariant::Filled,
        },
        Node::Card {
            title: "Outlined".into(),
            body: Box::new(muted("border only")),
            variant: CardVariant::Outlined,
        },
        Node::Popup(Box::new(col(vec![body("Popup"), muted("floating layer")]))),
    ])
}

/// Pick which end of a button the design crops and see it on every variant.
fn crop(c: &Ctx) -> Node {
    use dodai_core::Crop;
    let ends = c.theme.button_crops();
    if ends.is_empty() {
        return muted("This design does not crop buttons.");
    }
    let looks: Vec<_> = ends
        .iter()
        .filter_map(|e| c.theme.button_crop(*e))
        .collect();
    let mut kinds: Vec<&str> = looks.iter().map(|l| l.describe()).collect();
    kinds.dedup();
    let mut labels = vec!["Design", "Off"];
    labels.extend(ends.iter().map(|e| e.name()));
    let pick = choice("bc-crop", &labels, 0);
    let sel = c.values.index(&pick.bind);
    let styled = |b: crate::node::Button| match sel {
        0 => b,
        1 => b.plain(),
        i => b.crop(*ends.get(i - 2).unwrap_or(&Crop::Left)),
    };
    col(vec![
        muted(format!(
            "Crops one end of every button the design's way: {}.",
            kinds.join("; ")
        )),
        Node::Choice(pick),
        row(vec![
            styled(button("Primary", V::Primary)).node(),
            styled(button("Secondary", V::Secondary)).node(),
            styled(button("Outline", V::Outline)).node(),
            styled(button("Scrim", V::Scrim)).node(),
            styled(button("Danger", V::Danger)).node(),
        ]),
    ])
}
