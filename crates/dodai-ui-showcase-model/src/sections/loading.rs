//! Loading: spinners, indeterminate and determinate bars, skeletons, and Material's loading
//! indicator.

use dodai_ui_core::{ButtonVariant as V, ControlSize};
use dodai_ui_icons::IconId;

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
            .kit(dodai_ui_core::Kit::Skeleton, false)
            .fill
            .unwrap_or(c.theme.palette.surface_raised),
        border: None,
        radius: 24.0,
        label: None,
    };
    let design = match c.theme.id().split('-').next().unwrap_or_default() {
        "dodai" => "Dodai",
        "cyberpunk" => "Cyberpunk",
        "neobrutalism" | "neobrutal" => "Neo-Brutalism",
        "material" => "Material 3",
        _ => "this design",
    };
    let spinners = |set: &[dodai_ui_core::SpinnerStyle]| {
        line(
            set.iter()
                .map(|st| {
                    col(vec![
                        Node::SpinnerOf {
                            size: 40.0,
                            style: *st,
                        },
                        muted(format!("{st:?}").to_lowercase()),
                    ])
                    .gap(crate::node::Gap::Sm)
                })
                .collect(),
        )
        .gap(crate::node::Gap::Px(28.0))
        .start()
    };
    let card = |title: &str, body: Node| Node::Card {
        title: title.to_string(),
        body: Box::new(body),
        variant: dodai_ui_core::CardVariant::Outlined,
        surface: dodai_ui_core::Surface::INHERIT,
    };
    let shared = card(
        "Shared, in every design",
        col(vec![
            muted("Spinners"),
            spinners(&dodai_ui_core::SpinnerStyle::SHARED),
            muted("Progress"),
            Node::Progress {
                value: 0.35,
                tone: None,
            },
            Node::Indeterminate,
            muted("Skeleton placeholders"),
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
        ]),
    );
    let mut own = vec![muted("Spinners"), spinners(c.theme.look().spinners)];
    if !c.theme.look().progress_bars.is_empty() {
        own.push(muted("Progress"));
    }
    for st in c.theme.look().progress_bars {
        own.push(muted(format!("{st:?}").to_lowercase()));
        own.push(Node::StyledProgress {
            style: *st,
            value: Some(0.6),
        });
        own.push(Node::StyledProgress {
            style: *st,
            value: None,
        });
    }
    if c.theme.look().tonal {
        own.push(muted("Progress"));
        own.push(Node::WavyProgress { value: Some(0.6) });
        own.push(Node::WavyProgress { value: None });
        own.push(
            line(vec![
                Node::CircularProgress {
                    value: Some(0.6),
                    wavy: false,
                },
                Node::CircularProgress {
                    value: None,
                    wavy: false,
                },
                Node::CircularProgress {
                    value: Some(0.6),
                    wavy: true,
                },
                Node::CircularProgress {
                    value: None,
                    wavy: true,
                },
            ])
            .gap(crate::node::Gap::Px(28.0))
            .start(),
        );
    }
    col(vec![
        controls(),
        shared,
        card(&format!("Only in {design}"), col(own)),
        muted("Play runs one cycle, Loop repeats, Stop holds a still frame. Loading demos live on this page only (not on All) to keep idle CPU at zero."),
    ])
}

fn indicator(_: &Ctx) -> Node {
    row(vec![
        Node::Spinner(40.0),
        Node::Card {
            title: String::new(),
            body: Box::new(Node::Spinner(40.0)),
            variant: dodai_ui_core::CardVariant::Filled,
            surface: dodai_ui_core::Surface::INHERIT,
        },
    ])
}
