//! Shaders, laid out like the gpui showcase: a playground (source, effect chain, strength,
//! pixel size), then one section per look of source and of effect, then the preset lists.
//! Gallery cards are one still frame each (a hundred moving pictures would only measure the
//! GPU); the playground and the presets move.

use dodai_shaders::groups::{group, in_group, is_featured, EFFECT_GROUPS, SOURCE_GROUPS};
use dodai_shaders::{presets, ShaderInfo};

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Node, ShaderSpec, TextRole};
use crate::shader_state::{generators, post_effects, FX_ADD, GEN};
use dodai_core::{ButtonVariant as V, Tone};

/// Gallery tile width in px.
const CARD: f32 = 280.0;
const CAT: &str = "Shaders";

/// The preset lists, in page order.
const PRESETS: [(&str, &[presets::Preset]); 6] = [
    ("Retro terminals presets", presets::RETRO),
    ("Ghostty presets", presets::GHOSTTY),
    ("Blade Runner presets", presets::BLADE),
    ("Ghibli presets", presets::GHIBLI),
    ("DMT Hyperspace presets", presets::DMT),
    ("Pixel art presets", presets::PIXEL),
];

macro_rules! entry {
    ($title:expr, $build:expr) => {
        Section::new(CAT, $title, $build)
    };
}

pub(crate) const SECTIONS: [Section; 21] = [
    entry!("Playground", playground),
    entry!(SOURCE_GROUPS[0], source_group::<0>),
    entry!(SOURCE_GROUPS[1], source_group::<1>),
    entry!(SOURCE_GROUPS[2], source_group::<2>),
    entry!(SOURCE_GROUPS[3], source_group::<3>),
    entry!(SOURCE_GROUPS[4], source_group::<4>),
    entry!(SOURCE_GROUPS[5], source_group::<5>),
    entry!(SOURCE_GROUPS[6], source_group::<6>),
    entry!(SOURCE_GROUPS[7], source_group::<7>),
    entry!(EFFECT_GROUPS[0], effect_group::<0>),
    entry!(EFFECT_GROUPS[1], effect_group::<1>),
    entry!(EFFECT_GROUPS[2], effect_group::<2>),
    entry!(EFFECT_GROUPS[3], effect_group::<3>),
    entry!(EFFECT_GROUPS[4], effect_group::<4>),
    entry!(EFFECT_GROUPS[5], effect_group::<5>),
    entry!(PRESETS[0].0, preset_list::<0>),
    entry!(PRESETS[1].0, preset_list::<1>),
    entry!(PRESETS[2].0, preset_list::<2>),
    entry!(PRESETS[3].0, preset_list::<3>),
    entry!(PRESETS[4].0, preset_list::<4>),
    entry!(PRESETS[5].0, preset_list::<5>),
];

/// "Look · Name", how the dropdowns list a shader.
fn option_label(s: &ShaderInfo) -> String {
    format!(
        "{} · {}",
        group(s).unwrap_or_else(|| s.family.name()),
        s.name
    )
}

fn playground(c: &Ctx) -> Node {
    let gens = generators();
    let fxs = post_effects();
    let v = c.values;
    let at = v.index_of(GEN, 0).min(gens.len().saturating_sub(1));
    let source = gens.get(at).map(|s| s.id);
    let effects = v.shader_effects();
    let name = |id: &str| dodai_shaders::by_id(id).map_or(id.to_string(), |s| s.name.to_string());

    let mut chain = vec![
        muted("Chain"),
        tag(source.map_or("?".to_string(), name), Tone::Info, false),
    ];
    for id in &effects {
        chain.push(muted("→"));
        chain.push(
            button(format!("{} ✕", name(id)), V::Outline)
                .counter(format!("sh:rm:{id}"))
                .node(),
        );
    }
    if !effects.is_empty() {
        chain.push(
            button("Clear effects", V::Secondary)
                .counter("sh:clear")
                .node(),
        );
    }
    let step = |label: String, minus: &str, plus: &str| {
        vec![
            label_node(label),
            button("−", V::Secondary)
                .counter(format!("sh:{minus}"))
                .node(),
            button("+", V::Secondary)
                .counter(format!("sh:{plus}"))
                .node(),
        ]
    };
    let mut tune = step(
        format!("Intensity {:.0}%", v.shader_intensity() * 100.0),
        "int-",
        "int+",
    );
    tune.extend(step(
        format!("Pixel {:.0}px", v.shader_pixel()),
        "px-",
        "px+",
    ));
    tune.push(
        button(
            if v.shader_themed() {
                "Colours: design system"
            } else {
                "Colours: shader family"
            },
            V::Outline,
        )
        .counter("sh:themed")
        .node(),
    );

    let mut chain_ids: Vec<&'static str> = source.into_iter().collect();
    chain_ids.extend(effects.iter().copied());
    let mut spec = ShaderSpec::chain(&chain_ids).width(720.0);
    spec.themed = v.shader_themed();
    spec.intensity = v.shader_intensity();
    spec.pixel = v.shader_pixel();

    col(vec![
        muted(format!(
            "Shaders written in Rust (`dodai-shaders`), rendered on: {}. Pick a source, stack effects.",
            dodai_shaders::player::backend_label()
        )),
        row(vec![
            tag("SOURCE", Tone::Info, true),
            select(
                GEN,
                gens.iter().map(|s| option_label(s)).collect(),
                Some(at),
                "Pick a source",
            ),
            muted(format!("{} sources: draw the picture", gens.len())),
        ]),
        row(vec![
            tag("EFFECT", Tone::Warning, true),
            select(
                FX_ADD,
                fxs.iter().map(|s| option_label(s)).collect(),
                None,
                "Add an effect…",
            ),
            muted(format!(
                "{} effects: filter the picture, stacked in order",
                fxs.len()
            )),
        ]),
        row(chain),
        row(tune),
        Node::Shader(spec),
    ])
}

fn label_node(s: String) -> Node {
    crate::b::label(s)
}

/// One gallery card: the still, its name, a featured tag, the blurb and a button to use it.
fn card(c: &Ctx, s: &'static ShaderInfo, chain: &[&'static str], fx: bool) -> Node {
    let chosen = if fx {
        c.values.shader_effects().contains(&s.id)
    } else {
        c.values.shader_source() == Some(s.id)
    };
    let mut head = vec![text(s.name, TextRole::Mono)];
    if is_featured(s) {
        head.push(tag("featured", Tone::Success, false));
    }
    tile(
        CARD,
        vec![
            Node::Shader(ShaderSpec::chain(chain).still().width(CARD)),
            row(head),
            muted(s.desc),
            button(
                match (fx, chosen) {
                    (false, true) => "In the playground",
                    (false, false) => "Use as source",
                    (true, true) => "Remove from chain",
                    (true, false) => "Add to chain",
                },
                if chosen { V::Primary } else { V::Outline },
            )
            .counter(format!("sh:use:{}", s.id))
            .node(),
        ],
    )
}

fn source_group<const N: usize>(c: &Ctx) -> Node {
    row(in_group(SOURCE_GROUPS[N])
        .into_iter()
        .map(|s| card(c, s, &[s.id], false))
        .collect())
}

fn effect_group<const N: usize>(c: &Ctx) -> Node {
    row(in_group(EFFECT_GROUPS[N])
        .into_iter()
        .map(|s| card(c, s, &[s.family.default_source(), s.id], true))
        .collect())
}

fn preset_list<const N: usize>(_: &Ctx) -> Node {
    row(PRESETS[N]
        .1
        .iter()
        .map(|p| {
            tile(
                CARD,
                vec![
                    Node::Shader(ShaderSpec::preset(p.id).width(CARD)),
                    text(p.name, TextRole::Mono),
                    muted(p.desc),
                ],
            )
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_group_has_shaders_and_every_chain_resolves() {
        for g in SOURCE_GROUPS.iter().chain(EFFECT_GROUPS.iter()) {
            assert!(!in_group(g).is_empty(), "{g}");
        }
        for s in dodai_shaders::catalog() {
            assert!(dodai_shaders::by_id(s.family.default_source()).is_some());
        }
        for (_, list) in PRESETS {
            for p in list {
                assert!(!p.shaders().is_empty(), "{}", p.id);
            }
        }
    }

    #[test]
    fn a_spec_keeps_the_generator_once_and_adds_the_effects() {
        let t = dodai_core::themes::neutral();
        let id = in_group(SOURCE_GROUPS[0])[0].id;
        let fx = in_group(EFFECT_GROUPS[0])[0].id;
        let spec = ShaderSpec::chain(&[id, fx]).to_spec(&t);
        assert_eq!(spec.chain.len(), 2);
        assert_eq!(spec.chain[0].id, id);
        assert_eq!(spec.chain[1].id, fx);
        assert!(ShaderSpec::chain(&[]).to_spec(&t).chain.is_empty());
        let p = &presets::RETRO[0];
        assert_eq!(
            ShaderSpec::preset(p.id).to_spec(&t).chain.len(),
            p.shaders().len()
        );
        assert!(!ShaderSpec::chain(&[id]).still().to_spec(&t).animated);
    }
}
