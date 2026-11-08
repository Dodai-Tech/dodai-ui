//! Foundations: text, palette, type scale, spacing, shape, motion, icons.

use dodai_core::{Tone, TypeRole};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Node, TextRole};

const CAT: &str = "Foundations";

pub(crate) const SECTIONS: [Section; 7] = [
    Section {
        category: CAT,
        title: "Text",
        build: text_section,
    },
    Section {
        category: CAT,
        title: "Colour palette",
        build: palette,
    },
    Section {
        category: CAT,
        title: "Type scale",
        build: type_scale,
    },
    Section {
        category: CAT,
        title: "Motion tokens",
        build: motion,
    },
    Section {
        category: CAT,
        title: "Spacing · Elevation",
        build: spacing,
    },
    Section {
        category: CAT,
        title: "Shape tokens",
        build: shape,
    },
    Section {
        category: CAT,
        title: "Icons",
        build: icons,
    },
];

fn text_section(_: &Ctx) -> Node {
    col(vec![
        text("Heading", TextRole::Type(TypeRole::HeadlineMedium)),
        text("Subheading", TextRole::Type(TypeRole::TitleLarge)),
        body("Body text sets the reading size: the quick brown fox jumps over the lazy dog."),
        muted("Muted text carries secondary information."),
        label("Label"),
        text("Caption · the smallest size", TextRole::Caption),
        text("mono: let answer = 42;", TextRole::Mono),
    ])
}

fn palette(c: &Ctx) -> Node {
    let p = &c.theme.palette;
    let swatches = |names: &[(&str, dodai_core::Color)]| {
        row(names
            .iter()
            .map(|(n, col)| Node::Swatch {
                name: n.to_string(),
                color: *col,
            })
            .collect())
    };
    col(vec![
        muted("Surfaces"),
        swatches(&[
            ("background", p.background),
            ("surface", p.surface),
            ("surface raised", p.surface_raised),
            ("overlay", p.overlay),
            ("border", p.border),
            ("border strong", p.border_strong),
        ]),
        muted("Text"),
        swatches(&[
            ("text", p.text),
            ("muted", p.text_muted),
            ("disabled", p.text_disabled),
            ("link", p.link),
        ]),
        muted("Brand"),
        swatches(&[
            ("primary", p.primary),
            ("primary hover", p.primary_hover),
            ("on primary", p.on_primary),
            ("secondary", p.secondary),
            ("on secondary", p.on_secondary),
            ("accent", p.accent),
        ]),
        muted("Status"),
        swatches(&[
            ("info", p.info),
            ("success", p.success),
            ("warning", p.warning),
            ("danger", p.danger),
        ]),
        muted("Series"),
        row(p
            .series
            .iter()
            .enumerate()
            .map(|(i, col)| Node::Swatch {
                name: format!("series {}", i + 1),
                color: *col,
            })
            .collect()),
    ])
}

fn type_scale(c: &Ctx) -> Node {
    col(TypeRole::ALL
        .iter()
        .map(|&r| {
            let s = c.theme.type_style(r);
            line(vec![
                Node::Block {
                    w: 150.0,
                    h: 0.0,
                    fill: dodai_core::Color::rgba(0, 0, 0, 0),
                    border: None,
                    radius: 0.0,
                    label: Some(format!("{} · {:.0}", r.name(), s.size)),
                },
                text("The quick brown fox", TextRole::Type(r)),
            ])
        })
        .collect())
}

fn motion(c: &Ctx) -> Node {
    let m = &c.theme.motion;
    col(vec![
        muted("How this design moves."),
        body(format!(
            "Duration  {:.0} ms",
            m.duration.as_secs_f32() * 1000.0
        )),
        body(format!("Easing  {:?}", m.easing)),
        body(format!("Entry  {:?}", m.entry)),
    ])
}

fn spacing(c: &Ctx) -> Node {
    let t = c.theme;
    let p = &t.palette;
    let sp = &t.spacing;
    let bar = |name: &str, v: f32| {
        line(vec![
            Node::Block {
                w: 70.0,
                h: 0.0,
                fill: dodai_core::Color::rgba(0, 0, 0, 0),
                border: None,
                radius: 0.0,
                label: Some(format!("{name} · {v:.0}")),
            },
            Node::Block {
                w: v.max(2.0) * 4.0,
                h: 14.0,
                fill: p.primary,
                border: None,
                radius: 0.0,
                label: None,
            },
        ])
    };
    col(vec![
        muted("Spacing steps (×4)"),
        bar("xs", sp.xs),
        bar("sm", sp.sm),
        bar("md", sp.md),
        bar("lg", sp.lg),
        bar("xl", sp.xl),
        muted("Elevation"),
        row((0..4u8)
            .map(|level| Node::Card {
                title: format!("Level {level}"),
                body: Box::new(muted("raised")),
                variant: if level == 0 {
                    dodai_core::CardVariant::Outlined
                } else {
                    dodai_core::CardVariant::Elevated
                },
            })
            .collect()),
    ])
}

fn shape(c: &Ctx) -> Node {
    let t = c.theme;
    let sh = &t.shape;
    let p = &t.palette;
    let block = |label: String, radius: f32| Node::Block {
        w: 96.0,
        h: 56.0,
        fill: p.surface_raised,
        border: Some(p.border_strong),
        radius,
        label: Some(label),
    };
    col(vec![
        body(format!(
            "Corners {:?} · radius {:.0} · chamfer {:.0} · border {:.0} · control {:.0}",
            sh.corners, sh.radius, sh.chamfer, sh.border_width, sh.control_height
        )),
        row(vec![
            block("square".into(), 0.0),
            block("radius".into(), sh.radius),
            block("full".into(), 28.0),
        ]),
    ])
}

fn icons(_: &Ctx) -> Node {
    use IconId::*;
    let ids = [
        Home, Search, Settings, Star, Heart, Edit, Copy, Check, Close, Menu, Folder, File, Bell,
        Person, Mail, Calendar, Cloud, Code, Dashboard, Download, Eye, Flag, Bookmark, Archive,
    ];
    col(vec![
        muted("The shared icon vocabulary, drawn in each design's own style."),
        row(ids.iter().map(|&i| Node::Icon(i, 24.0)).collect()),
        row(vec![
            tag("success", Tone::Success, false),
            tag("danger", Tone::Danger, false),
        ]),
    ])
}
