//! Shape: the design's corners, borders and elevation on the common components, and the
//! app-wide tuning of them (kept in [`crate::tune_state`], applied by `Showcase::theme`).

use dodai_core::{
    AccentCorner, BorderTint, ButtonVariant as V, CardVariant, ControlColour, GradientMode,
    RoundnessGroup, ShadowDir, ShapeKind, Theme, Tone,
};

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Node, SliderFormat};
use crate::tune_state as ts;
use crate::Gap;

const CAT: &str = "Shape";

pub(crate) const SECTIONS: [Section; 5] = [
    Section::new(CAT, "Shape gallery", gallery),
    Section::new(CAT, "Roundness · Borders", roundness),
    Section::new(CAT, "Corner shape", corner_shape).when(|t| t.look().cut_corners),
    Section::new(CAT, "Button crop", crop).when(|t| !t.button_crops().is_empty()),
    Section::new(CAT, "Border", border).when(|t| t.look().accent_border),
];

/// The same buttons, tags and cards, to show a shape setting on real components.
fn shape_gallery() -> Node {
    let card = |title: &str, body: Option<&str>, variant| Node::Card {
        title: title.into(),
        body: Box::new(body.map_or(Node::Space(0.0), muted)),
        variant,
        surface: dodai_core::Surface::INHERIT,
    };
    let cell = |n: Node| tile(220.0, vec![n]);
    col(vec![
        row(vec![
            button("Primary", V::Primary).node(),
            button("Secondary", V::Secondary).node(),
            button("Outline", V::Outline).node(),
            button("Danger", V::Danger).node(),
        ]),
        row(vec![
            tag("online", Tone::Success, true),
            tag("warn", Tone::Warning, false),
            tag("error", Tone::Danger, true),
        ]),
        top_row(vec![
            cell(card("Card", None, CardVariant::Outlined)),
            cell(card("Elevated card", None, CardVariant::Elevated)),
            cell(card("Panel", Some("Panel body"), CardVariant::Filled)),
        ])
        .gap(Gap::Md),
    ])
}

fn gallery(c: &Ctx) -> Node {
    col(vec![
        muted("The design system's own corners, borders and shadows on real components, with the tokens behind them."),
        crate::sections::foundations::shape_tokens_view(c.theme),
        shape_gallery(),
    ])
}

/// One labelled setting: a fixed label column, the control, a muted hint underneath.
fn setting(name: &str, hint: impl Into<String>, control: Node) -> Node {
    line(vec![
        tile(120.0, vec![label(name)]),
        col(vec![control, muted(hint.into())]).gap(Gap::Xs),
    ])
    .gap(Gap::Lg)
    .start()
}

/// A setting with a segmented choice.
fn pick(name: &str, hint: &str, key: &str, labels: &[&str], sel: usize) -> Node {
    setting(
        name,
        hint,
        Node::Choice(choice(key, labels, sel).connected()),
    )
}

/// A setting with a slider from 0 to `max`.
fn tune_slider(
    name: &str,
    hint: &str,
    key: &str,
    now: f32,
    max: f32,
    step: f32,
    f: SliderFormat,
) -> Node {
    setting(
        name,
        hint,
        Node::Slider(
            slider(key, now, 0.0, max)
                .step(step)
                .format(f)
                .length(280.0),
        ),
    )
}

fn roundness(c: &Ctx) -> Node {
    let t: &Theme = c.theme;
    let tune = ts::tune(c.values);
    let mut out = vec![muted(
        "Applies to every component in every design, immediately. \"Default\" is each design's own.",
    )];
    if t.shape.kind() != ShapeKind::Chamfered {
        out.push(tune_slider(
            "Roundness",
            "0 is square, 100% is the design's own corners, more is rounder (up to 200%).",
            ts::ROUND,
            tune.roundness,
            2.0,
            0.05,
            SliderFormat::Ratio,
        ));
        for (k, g) in RoundnessGroup::ALL.into_iter().enumerate() {
            out.push(tune_slider(
                g.label(),
                "On top of the global roundness; 100% leaves this group alone.",
                &ts::group_key(k),
                tune.groups[k],
                2.0,
                0.05,
                SliderFormat::Ratio,
            ));
        }
    }
    let at = |k: &str, d: usize| c.values.index_of(k, d);
    out.push(pick(
        "Corner style",
        "Round, straight cut or concave scoop, on every design (Cut and Scoop take each role's size).",
        ts::STYLE,
        &ts::STYLES.map(|s| s.0),
        at(ts::STYLE, 0),
    ));
    out.push(pick(
        "Border width",
        "Thickness of every outline.",
        ts::WIDTH,
        &ts::WIDTHS.map(|w| w.0),
        at(ts::WIDTH, 0),
    ));
    out.push(pick(
        "Border colour",
        "Which colour outlines take.",
        ts::TINT,
        &BorderTint::ALL.map(|b| b.label()),
        at(ts::TINT, 0),
    ));
    let offered = ControlColour::offered(t);
    let now = ts::colour(c.values);
    let labels: Vec<&str> = offered.iter().map(|o| o.label()).collect();
    out.push(Node::Choice(
        choice(
            ts::COLOUR_PICK,
            &labels,
            offered.iter().position(|o| *o == now).unwrap_or(0),
        )
        .connected(),
    ));
    let colour = out.pop().unwrap_or(Node::Space(0.0));
    out.push(setting(
        "Control colour",
        "Which colour filled controls take: buttons, checked boxes, on switches, slider fills, the selected tab. Taken from the current theme and kept at 3:1 on the page. Also in the top bar.",
        colour,
    ));
    out.push(pick(
        "Outline all",
        "Give filled components without a border one too.",
        ts::OUTLINE_ALL,
        &["Off", "On"],
        at(ts::OUTLINE_ALL, 0),
    ));
    out.push(pick(
        "Density",
        "Control height and spacing: compact, as designed, or roomy. Type and icons keep their size.",
        ts::DENSITY,
        &ts::DENSITIES.map(|d| d.0),
        at(ts::DENSITY, 1),
    ));
    if t.surface_effects().wash {
        out.push(pick(
            "Gradient",
            "How strongly the design's gradient shows: the canvas mesh, the wash and the rim.",
            ts::GRADIENT,
            &GradientMode::ALL.map(|g| g.label()),
            at(ts::GRADIENT, 1),
        ));
    }
    if t.effects.hard_shadow > 0.0 {
        out.push(tune_slider(
            "Shadow offset",
            "How far the hard shadow is offset, in px. Only designs with hard shadows have one.",
            ts::SHADOW,
            t.effects.hard_shadow,
            12.0,
            1.0,
            SliderFormat::Px,
        ));
        out.push(pick(
            "Shadow direction",
            "Which way the hard shadow falls.",
            ts::SHADOW_DIR,
            &ShadowDir::ALL.map(|d| d.label()),
            at(ts::SHADOW_DIR, 0),
        ));
    }
    out.push(row(vec![button("Reset", V::Outline)
        .counter(ts::RESET)
        .node()]));
    out.push(shape_gallery());
    col(out)
}

/// The frame of every section: contour width and strength, and the accent corner.
fn border(c: &Ctx) -> Node {
    let cur = ts::border(c.values);
    let preset = ts::BORDERS
        .iter()
        .position(|(_, b)| b.width == cur.width && b.opacity == cur.opacity)
        .unwrap_or(usize::MAX);
    let corner = AccentCorner::ALL
        .iter()
        .position(|a| *a == cur.accent)
        .unwrap_or(usize::MAX);
    let slide = |name: &str, hint: &str, key: &str, now: f32, lo: f32, hi: f32, f| {
        setting(
            name,
            hint,
            Node::Slider(slider(key, now, lo, hi).step(0.25).format(f).length(280.0)),
        )
    };
    let presets = ts::BORDERS.map(|(name, b)| {
        tile(
            140.0,
            vec![Node::Framed {
                border: b.accent(cur.accent),
                body: Box::new(col(vec![
                    label(name),
                    muted(format!("{:.2} px · {:.0}%", b.width, b.opacity * 100.0)),
                ])),
            }],
        )
    });
    col(vec![
        muted("Every section frame is a grey contour plus an accent line on the opposite corner. These settings change all sections at once; a single panel can use `.border(..)`."),
        setting(
            "Preset",
            "Width and strength of the grey contour.",
            Node::Choice(choice(ts::BD_PRESET, &ts::BORDERS.map(|b| b.0), preset).connected()),
        ),
        setting(
            "Accent corner",
            "Where the bright line sits. The grey contour moves to the opposite side.",
            Node::Choice(
                choice(ts::BD_ACCENT, &AccentCorner::ALL.map(|a| a.name()), corner).connected(),
            ),
        ),
        slide(
            "Grey width",
            "Thickness of the grey contour in px.",
            ts::BD_WIDTH,
            cur.width,
            0.0,
            4.0,
            SliderFormat::PxFine,
        ),
        slide(
            "Accent width",
            "Thickness of the bright line in px.",
            ts::BD_ACCENT_WIDTH,
            cur.accent_width
                .unwrap_or(c.theme.effects.hard_shadow.max(0.5)),
            0.5,
            6.0,
            SliderFormat::PxFine,
        ),
        slide(
            "Grey opacity",
            "0 hides the contour, 1 is fully solid.",
            ts::BD_OPACITY,
            cur.opacity,
            0.0,
            1.0,
            SliderFormat::Fine,
        ),
        col(vec![muted("Presets, side by side"), row(presets.to_vec())]).gap(Gap::Sm),
    ])
}

fn corner_shape(c: &Ctx) -> Node {
    let cur = c.values.index_of(ts::CORNER, 0);
    let sets: Vec<&str> = ts::CORNERS[..ts::SETS].iter().map(|p| p.0).collect();
    let singles: Vec<&str> = ts::CORNERS[ts::SETS..].iter().map(|p| p.0).collect();
    let sel_sets = if cur < ts::SETS { cur } else { usize::MAX };
    let sel_singles = cur.checked_sub(ts::SETS).unwrap_or(usize::MAX);
    col(vec![
        muted("The corner cut applies to every component in every theme, immediately. Pick \"Default\" to restore each design's own shape."),
        setting(
            "Cut corners",
            "Which corners every component cuts, in every design. \"Default\" restores each design's own.",
            col(vec![
                Node::Choice(choice(ts::CORNER_SETS, &sets, sel_sets).connected()),
                Node::Choice(choice(ts::CORNER_SINGLES, &singles, sel_singles).connected()),
            ])
            .gap(Gap::Sm),
        ),
        shape_gallery(),
    ])
}

fn crop(c: &Ctx) -> Node {
    let t = c.theme;
    let ends = t.button_crops();
    let mut kinds: Vec<&str> = ends
        .iter()
        .filter_map(|e| t.button_crop(*e))
        .map(|l| l.describe())
        .collect();
    kinds.dedup();
    let mut labels = vec!["Design", "Off"];
    labels.extend(ends.iter().map(|e| e.name()));
    let name = t.spec.get().map_or("design's", |s| s.name());
    col(vec![
        muted("Settings apply to every button below and across the showcase. Ready-made cropped buttons are under Actions."),
        pick(
            "Crop end",
            &format!("Crops one end of every button the {name} way: {}.", kinds.join("; ")),
            ts::CROP,
            &labels,
            c.values.index_of(ts::CROP, 0),
        ),
        col(vec![
            muted("Preview"),
            row(vec![
                button("Primary", V::Primary).node(),
                button("Secondary", V::Secondary).node(),
                button("Outline", V::Outline).node(),
                button("Scrim", V::Scrim).node(),
                button("Danger", V::Danger).node(),
            ]),
        ])
        .gap(Gap::Sm),
    ])
}
