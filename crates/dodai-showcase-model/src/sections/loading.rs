//! Loading: spinner, indeterminate bar, skeleton.

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Loading";

pub(crate) const SECTIONS: [Section; 1] = [Section {
    category: CAT,
    title: "Spinner · Indeterminate · Skeleton",
    build: loading,
}];

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
