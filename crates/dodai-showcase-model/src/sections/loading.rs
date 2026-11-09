//! Loading: spinners, indeterminate and determinate bars, skeletons, and Material's loading
//! indicator.

use dodai_core::{ButtonVariant as V, ControlSize, Tone};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Loading";

pub(crate) const SECTIONS: [Section; 2] = [
    Section::new(CAT, "Spinner · Indeterminate · Skeleton", loading).heading("Loading"),
    Section::new(CAT, "Loading indicator", indicator).when(super::actions::material_only),
];

/// Play / loop / stop and the duration of one run (the gpui panel's sub-header).
fn controls() -> Node {
    let mode = |icon: IconId, on: bool| {
        let v = if on { V::Primary } else { V::Outline };
        button("", v).icon(icon).size(ControlSize::ALL[0]).node()
    };
    line(vec![
        mode(IconId::Play, false),
        mode(IconId::Repeat, true),
        mode(IconId::Stop, false),
        Node::Space(10.0),
        muted("Duration per run"),
        tile(
            220.0,
            vec![slider("ld-dur", 0.0, 0.0, 10_000.0).step(100.0).node()],
        ),
    ])
}

fn loading(c: &Ctx) -> Node {
    let skeleton_disc = Node::Block {
        w: 48.0,
        h: 48.0,
        fill: c
            .theme
            .kit(dodai_core::Kit::Skeleton, false)
            .fill
            .unwrap_or(c.theme.palette.surface_raised),
        border: None,
        radius: 24.0,
        label: None,
    };
    col(vec![
        controls(),
        label("Spinner"),
        row(vec![
            Node::Spinner(24.0),
            Node::Spinner(24.0),
            Node::Spinner(24.0),
            muted("900 ms · 450 ms · 1.8 s ease-in-out (unless a duration is set)"),
        ]),
        label("Indeterminate bar"),
        Node::Indeterminate,
        Node::Indeterminate,
        label("Determinate progress"),
        Node::Progress {
            value: 0.35,
            tone: None,
        },
        Node::Progress {
            value: 0.7,
            tone: Some(Tone::Success),
        },
        label("Skeleton placeholders"),
        line(vec![
            skeleton_disc,
            col(vec![
                Node::Skeleton { w: 0.0, h: 14.0 },
                Node::Skeleton { w: 260.0, h: 14.0 },
                Node::Skeleton { w: 180.0, h: 14.0 },
            ])
            .gap(crate::node::Gap::Sm),
        ])
        .gap(crate::node::Gap::Px(16.0))
        .start(),
        muted("Play runs one cycle, Loop repeats, Stop holds a still frame. Loading demos live on this page only (not on All) to keep idle CPU at zero."),
    ])
}

fn indicator(_: &Ctx) -> Node {
    row(vec![
        Node::Spinner(40.0),
        Node::Card {
            title: String::new(),
            body: Box::new(Node::Spinner(40.0)),
            variant: dodai_core::CardVariant::Filled,
            surface: dodai_core::Surface::INHERIT,
        },
    ])
}
