//! Loading: spinner, indeterminate bar, skeleton.

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Loading";

pub(crate) const SECTIONS: [Section; 2] = [
    Section::new(CAT, "Spinner · Indeterminate · Skeleton", loading),
    Section::new(CAT, "Loading indicator", indicator),
];

fn loading(_: &Ctx) -> Node {
    col(vec![
        row(vec![Node::Spinner(24.0), Node::Spinner(40.0)]),
        Node::Indeterminate,
        muted("Skeleton"),
        Node::Skeleton { w: 0.0, h: 16.0 },
        Node::Skeleton { w: 240.0, h: 16.0 },
        Node::Skeleton { w: 160.0, h: 16.0 },
    ])
}

fn indicator(_: &Ctx) -> Node {
    row(vec![
        Node::Spinner(40.0),
        Node::Card {
            title: String::new(),
            body: Box::new(Node::Spinner(40.0)),
            variant: dodai_core::CardVariant::Filled,
        },
    ])
}
