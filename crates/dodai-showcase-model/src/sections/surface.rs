//! Surface effects: the theme-wide glow and scanlines, adjustable live. (The gpui showcase also
//! has per-component textures, glitch, sheen and frost; those need a per-toolkit renderer.)

use dodai_core::ToggleKind::Switch;
use dodai_core::{ButtonVariant as V, CardVariant, Tone};

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;
use crate::state::{GLOW_KEY, SCANLINES_KEY};

const CAT: &str = "Surface effects";

pub(crate) const SECTIONS: [Section; 2] = [
    Section {
        category: CAT,
        title: "Glow",
        build: glow,
    },
    Section {
        category: CAT,
        title: "Scanlines",
        build: scanlines,
    },
];

fn sample(label: &str) -> Node {
    row(vec![
        button(label, V::Primary).node(),
        button("Outline", V::Outline).node(),
        input("fx-field", "Focus me").node(),
        Node::Card {
            title: "Card".into(),
            body: Box::new(muted("Lit edge")),
            variant: CardVariant::Elevated,
        },
        tag("live", Tone::Success, true),
    ])
}

fn glow(c: &Ctx) -> Node {
    let own = c.theme.effects.glow_intensity;
    col(vec![
        muted("How strongly primary and focused controls glow. 0 turns the glow off; designs with no glow of their own stay flat."),
        slider(GLOW_KEY, own, 0.0, 2.0).node(),
        sample("Primary"),
    ])
}

fn scanlines(c: &Ctx) -> Node {
    col(vec![
        muted("Fine horizontal CRT lines over the whole page, where the design allows them."),
        toggle(
            Switch,
            SCANLINES_KEY,
            c.theme.effects.scanlines,
            "Scanlines",
        )
        .node(),
        sample("Primary"),
    ])
}
