//! Surface effects: the app-wide surface (textures, light, edges, ripple, the designs' own
//! signature effects), as on the gpui page. Each control writes an `sfx:` value; the mix is
//! read back with [`Showcase::surface_fx`](crate::Showcase::surface_fx).

use dodai_core::{ButtonVariant as V, CardVariant, PressStyle, Surface, SurfaceClass, Theme};

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Gap, Node};
use crate::state::{GLOW_KEY, TARGETS};
use crate::surface_state::{self as sfx, amount, mix};

const CAT: &str = "Surface effects";

pub(crate) const SECTIONS: [Section; 8] = [
    Section::new(CAT, "Presets · Apply to", presets).when(textured),
    Section::new(CAT, "Component textures", component_textures).when(textured),
    Section::new(CAT, "Container textures", container_textures).when(textured),
    Section::new(CAT, "Frosted glass", frost),
    Section::new(CAT, "Light FX", light).when(lit),
    Section::new(CAT, "Edges · Depth", edges).when(edged),
    Section::new(CAT, "Click ripple", ripple).when(rippled),
    Section::new(CAT, "Signature", signature).when(signed),
];

/// The design draws surface textures at all (every surface section but the glass needs one).
fn textured(t: &Theme) -> bool {
    !t.surface_textures().is_empty()
}

fn lit(t: &Theme) -> bool {
    let e = t.surface_effects();
    textured(t)
        && (t.look().luminous || e.glitch || e.sheen || e.flicker || e.bloom || e.pulse || e.chase)
}

fn edged(t: &Theme) -> bool {
    let e = t.surface_effects();
    textured(t) && (e.edge || e.brackets || e.depth)
}

fn rippled(t: &Theme) -> bool {
    textured(t) && t.surface_effects().ripple
}

fn signed(t: &Theme) -> bool {
    let e = t.surface_effects();
    textured(t) && (e.tint || e.pop || e.tape || e.gloss || e.strip || e.wash || e.rim)
}

/// A labelled slider of the mix (label column, slider with its readout).
fn fx_slider(c: &Ctx, key: &str) -> Node {
    let Some(a) = amount(key) else {
        return Node::Space(0.0);
    };
    let mut f = mix(c.values, c.theme);
    let v = *(a.field)(&mut f);
    labelled(
        a.name,
        col(vec![
            slider(a.key, v, 0.0, a.max).step(0.05).node(),
            muted(a.hint()),
        ]),
    )
}

/// `control` after a 120 px label column.
fn labelled(name: &str, control: Node) -> Node {
    row(vec![
        tile(120.0, vec![label(name)]),
        tile(320.0, vec![control]),
    ])
}

/// The three buttons that show the mix.
fn preview() -> Node {
    row(vec![
        button("Primary", V::Primary).node(),
        button("Secondary", V::Secondary).node(),
        button("Outline", V::Outline).node(),
    ])
}

fn presets(c: &Ctx) -> Node {
    let t = c.theme;
    let fx = mix(c.values, t);
    let family = {
        let mut ch = t.meta.family.chars();
        ch.next()
            .map(|f| f.to_uppercase().chain(ch).collect::<String>())
            .unwrap_or_default()
    };
    let mode = sfx::mode(&fx, t);
    let status = match mode {
        0 => "Off: no texture or effect is drawn on any component.".to_string(),
        1 => format!("Default: the {family} design's own mix."),
        _ => "Custom: your own mix of textures and effects.".to_string(),
    };
    let target = |cl: SurfaceClass| {
        let on = fx.targets.contains(cl);
        let text = if on {
            format!("\u{2713} {}", cl.label())
        } else {
            cl.label().to_string()
        };
        chip(text, &sfx::target_key(cl), on).node()
    };
    let group = |title: &str, meaning: &str, container: bool| {
        row(vec![
            tile(200.0, vec![label(title), muted(meaning)]),
            row(SurfaceClass::ALL
                .iter()
                .filter(|cl| cl.is_container() == container)
                .map(|cl| target(*cl))
                .collect()),
        ])
        .gap(Gap::Lg)
    };
    let shortcut = |label: &str, to: &str| {
        button(label, V::Ghost)
            .counter(format!("{TARGETS}{to}"))
            .node()
    };
    col(vec![
        label("Mode"),
        Node::Choice(choice(sfx::MODE, &["Off", "Default", "Custom"], mode).connected()),
        muted(status),
        Node::Divider,
        row(vec![
            label("Apply effects to"),
            line(vec![
                shortcut("Default", "default"),
                shortcut("All", "all"),
                shortcut("None", "none"),
            ])
            .gap(Gap::Xs),
        ]),
        col(vec![
            group("Controls", "Things you press or type in", false),
            group("Containers", "Things that hold other content", true),
        ])
        .gap(Gap::Lg),
        muted("Default is buttons, cards and panels. Picking a texture or effect below changes the mode to Custom."),
    ])
}

fn component_textures(c: &Ctx) -> Node {
    let t = c.theme;
    let list = sfx::component_textures(t);
    let labels: Vec<&str> = list.iter().map(|x| x.label()).collect();
    let at = list
        .iter()
        .position(|x| *x == mix(c.values, t).texture)
        .unwrap_or(0);
    let mut out = vec![
        muted("A pattern on UI components: buttons, inputs and tags. The buttons below wear it. Keep it fine and subtle so labels stay readable."),
        Node::Tabs(choice(sfx::TEXTURE, &labels, at)),
        fx_slider(c, "sfx:strength"),
        fx_slider(c, "sfx:scale"),
    ];
    if t.surface_effects().neon {
        out.push(fx_slider(c, "sfx:neon"));
    }
    out.extend([fx_slider(c, "sfx:clear"), preview()]);
    col(out)
}

fn container_textures(c: &Ctx) -> Node {
    let t = c.theme;
    let list = sfx::container_textures(t);
    let labels: Vec<&str> = list.iter().map(|x| x.label()).collect();
    let at = list
        .iter()
        .position(|x| *x == mix(c.values, t).container_texture)
        .unwrap_or(0);
    let mut out = vec![
        muted("A pattern on containers: cards, panels and bars. This panel is a container, so it shows the chosen pattern behind these controls."),
        Node::Tabs(choice(sfx::CONTAINER_TEXTURE, &labels, at)),
        fx_slider(c, "sfx:container_strength"),
        fx_slider(c, "sfx:container_scale"),
    ];
    if t.surface_effects().neon {
        out.push(fx_slider(c, "sfx:neon"));
    }
    out.push(fx_slider(c, "sfx:clear"));
    col(out)
}

fn frost(c: &Ctx) -> Node {
    let own = Surface::new().frost(1.0);
    let card = |title: &str, surface: Surface| {
        tile(
            260.0,
            vec![Node::Card {
                title: title.into(),
                body: Box::new(muted("")),
                variant: CardVariant::Elevated,
                surface,
            }],
        )
    };
    col(vec![
        muted("Any surface can be frosted: its fill turns as see-through as the design's glass and a blurred copy of the backdrop shows under it. The slider frosts every container (cards, panels, bars, this panel too); a component can frost itself with Surface::new().frost(1.0), as the buttons and the card below do."),
        fx_slider(c, "sfx:frost"),
        labelled(
            "Backdrop",
            Node::Choice(
                choice(sfx::WALLPAPER, &["Canvas", "Picture"], usize::from(sfx::wallpaper(c.values)))
                    .connected(),
            ),
        ),
        row(vec![
            button("Primary", V::Primary).surface(own).node(),
            button("Secondary", V::Secondary).surface(own).node(),
            button("Outline", V::Outline).surface(own).node(),
        ]),
        row(vec![
            card("Frosted card", own),
            card("Plain card", Surface::new().frost(0.0)),
        ]),
    ])
}

fn light(c: &Ctx) -> Node {
    let t = c.theme;
    let e = t.surface_effects();
    let mut out = vec![muted(
        "Glow and animated accents on components and containers. 0 turns an effect off.",
    )];
    if t.look().luminous {
        out.push(labelled(
            "Glow",
            slider(GLOW_KEY, t.effects.glow_intensity, 0.0, 2.0)
                .step(0.05)
                .node(),
        ));
    }
    for (on, key) in [
        (e.glitch, "sfx:glitch"),
        (e.sheen, "sfx:sheen"),
        (e.flicker, "sfx:flicker"),
        (e.bloom, "sfx:bloom"),
        (e.pulse, "sfx:pulse"),
        (e.chase, "sfx:chase"),
    ] {
        if on {
            out.push(fx_slider(c, key));
        }
    }
    col(out)
}

fn edges(c: &Ctx) -> Node {
    let e = c.theme.surface_effects();
    let mut out = vec![muted("How the edges look: light falling inward, corner brackets and an inner shadow. Clear centre (under textures) keeps text clean.")];
    for (on, key) in [
        (e.edge, "sfx:edge"),
        (e.brackets, "sfx:brackets"),
        (e.depth, "sfx:depth"),
    ] {
        if on {
            out.push(fx_slider(c, key));
        }
    }
    out.push(preview());
    col(out)
}

fn ripple(c: &Ctx) -> Node {
    let styles: Vec<&str> = PressStyle::ALL.iter().map(|p| p.label()).collect();
    let at = PressStyle::ALL
        .iter()
        .position(|p| *p == mix(c.values, c.theme).press)
        .unwrap_or(0);
    col(vec![
        muted("Press any control below: the ripple plays from the touch point. Pick how it plays and how strong it is."),
        labelled(
            "Style",
            Node::Choice(choice(sfx::PRESS, &styles, at).connected()),
        ),
        fx_slider(c, "sfx:ripple"),
        preview(),
    ])
}

fn signature(c: &Ctx) -> Node {
    let e = c.theme.surface_effects();
    let mut out = vec![muted("Effects that belong to this design only: tonal wash, a hard pop block and tape, a clear-coat gloss, a light strip and a gradient wash.")];
    for (on, key) in [
        (e.tint, "sfx:tint"),
        (e.pop, "sfx:pop"),
        (e.tape, "sfx:tape"),
        (e.gloss, "sfx:gloss"),
        (e.strip, "sfx:strip"),
        (e.wash, "sfx:wash"),
        (e.rim, "sfx:rim"),
    ] {
        if on {
            out.push(fx_slider(c, key));
        }
    }
    out.push(preview());
    col(out)
}
