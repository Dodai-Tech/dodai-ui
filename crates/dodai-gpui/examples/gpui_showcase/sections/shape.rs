//! The "Shape" category sections.

use super::super::*;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_shape(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        #[allow(unused_macros)]
        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .subheader(self.chips($t, &t, &me, cx))
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
            };
        }
        let mut out: Vec<Section> = Vec::new();
        let gap = px(t.spacing.md);
        let glow = glow_intensity(cx);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let cut = |b: Button| b;
        // One labelled setting: fixed label column, the control, a muted hint underneath.
        let setting = |label: &'static str, hint: &'static str, control: gpui::AnyElement| {
            div()
                .flex()
                .items_start()
                .gap(px(t.spacing.lg))
                .child(
                    div()
                        .w(px(120.0))
                        .pt(px(t.spacing.sm))
                        .child(Label::new(label)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(t.spacing.xs))
                        .child(control)
                        .child(Muted::new(hint)),
                )
        };
        // Global: picking a cut changes every component in every theme.
        let corner_picker = {
            use dodai_core::Corners as C;
            let cur = dodai_gpui::corners(cx);
            let over = dodai_gpui::corners_overridden(cx);
            type Preset = (&'static str, Option<C>);
            let sets: [Preset; 5] = [
                ("Default", None),
                ("None", Some(C::NONE)),
                ("All", Some(C::ALL)),
                ("Diagonal", Some(C::DIAGONAL)),
                ("Anti-diagonal", Some(C::ANTI_DIAGONAL)),
            ];
            let singles: [Preset; 6] = [
                ("Top left", Some(C::TOP_LEFT)),
                ("Top right", Some(C::TOP_RIGHT)),
                ("Bottom right", Some(C::BOTTOM_RIGHT)),
                ("Bottom left", Some(C::BOTTOM_LEFT)),
                ("Left", Some(C::LEFT)),
                ("Right", Some(C::RIGHT)),
            ];
            let pos = |list: &[Preset]| {
                if !over {
                    return list.iter().position(|p| p.1.is_none());
                }
                list.iter().position(|p| p.1 == Some(cur))
            };
            let apply = |c: Option<C>, cx: &mut App| match c {
                Some(c) => dodai_gpui::set_corners(cx, c),
                None => dodai_gpui::reset_corners(cx),
            };
            setting(
                "Cut corners",
                "Which corners every component cuts, in every design. \"Default\" restores each design's own.",
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.sm))
                    .child(
                        div().flex().child(
                            Segmented::new("cut-sets", sets.map(|p| p.0))
                                .selected(pos(&sets).unwrap_or(usize::MAX))
                                .on_select(move |i, _, cx| apply(sets[i].1, cx)),
                        ),
                    )
                    .child(
                        div().flex().child(
                            Segmented::new("cut-singles", singles.map(|p| p.0))
                                .selected(pos(&singles).unwrap_or(usize::MAX))
                                .on_select(move |i, _, cx| apply(singles[i].1, cx)),
                        ),
                    )
                    .into_any_element(),
            )
        };
        let shape_gallery =
            |scope: &str| {
                let id = |k: &str| gpui::SharedString::from(format!("{scope}-sg-{k}"));
                col()
                    .child(
                        row()
                            .child(Button::new(id("p"), "Primary"))
                            .child(
                                Button::new(id("s"), "Secondary").variant(ButtonVariant::Secondary),
                            )
                            .child(Button::new(id("o"), "Outline").variant(ButtonVariant::Outline))
                            .child(Button::new(id("d"), "Danger").variant(ButtonVariant::Danger)),
                    )
                    .child(
                        row()
                            .child(Tag::new("online", Tone::Success).solid(true))
                            .child(Tag::new("warn", Tone::Warning))
                            .child(Tag::new("error", Tone::Danger).solid(true)),
                    )
                    .child(
                        row()
                            .items_start()
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(220.0))
                                    .child(Card::new(CardVariant::Outlined).headline("Card")),
                            )
                            .child(
                                div().flex_1().min_w(px(220.0)).child(
                                    Card::new(CardVariant::Elevated).headline("Elevated card"),
                                ),
                            )
                            .child(div().flex_1().min_w(px(220.0)).child(
                                Panel::new().title("Panel").child(Muted::new("Panel body")),
                            )),
                    )
            };
        let shape_props = div().flex().flex_col().gap(px(t.spacing.sm)).child(
            row().child(Label::new("Glow")).child(
                div().w(px(200.0)).child(
                    Slider::new("glow-slider", glow)
                        .range(0.0, 2.0)
                        .step(0.05)
                        .show_value(true)
                        .format(|v| format!("{v:.2}"))
                        .on_change(move |v, _, cx| set_glow_intensity(cx, v)),
                ),
            ),
        );
        let m = me.clone();
        // Every design: its own corner radius, border and shadow on real components.
        out.push((
            "Shape",
            "Shape gallery",
            section!("Shape gallery")
                .child(Muted::new(
                    "The design system's own corners, borders and shadows on real components, with the tokens behind them.",
                ))
                .child(shape_tokens_view(&t))
                .child(shape_gallery("gallery"))
                .into_any_element(),
            "Shape gallery",
        ));
        // Textures and effects belong to the designs that declare them.
        if !t.surface_textures().is_empty() {
            use dodai_core::{SurfaceClass, SurfaceFx, Texture};
            let owned = t.surface_textures();
            let fx = dodai_gpui::surface_fx();
            let amount = move |id: &'static str,
                               name: &'static str,
                               v: f32,
                               max: f32,
                               set: fn(&mut SurfaceFx, f32)| {
                row()
                    .child(div().w(px(90.0)).child(Label::new(name)))
                    .child(
                        div().w(px(220.0)).child(
                            Slider::new(id, v)
                                .range(0.0, max)
                                .step(0.05)
                                .show_value(true)
                                .format(|v| format!("{v:.2}"))
                                .on_change(move |v, _, cx| {
                                    let mut f = dodai_gpui::surface_fx();
                                    set(&mut f, v);
                                    dodai_gpui::set_surface_fx(cx, f);
                                }),
                        ),
                    )
            };
            let picker = |id: &'static str,
                          cur: Texture,
                          components: bool,
                          set: fn(&mut SurfaceFx, Texture)| {
                row().children(
                    Texture::ALL
                        .iter()
                        .copied()
                        .filter(|x| (!components || x.for_components()) && owned.contains(x))
                        .map(|x| {
                            let btn = Button::new(
                                gpui::SharedString::from(format!("{id}-{}", x.label())),
                                x.label(),
                            )
                            .variant(if x == cur {
                                ButtonVariant::Primary
                            } else {
                                ButtonVariant::Outline
                            })
                            .on_click(move |_, _, cx| {
                                let mut f = dodai_gpui::surface_fx();
                                set(&mut f, x);
                                dodai_gpui::set_surface_fx(cx, f);
                            });
                            // The button previews its own texture.
                            div()
                                .relative()
                                .child(btn)
                                .child(dodai_gpui::texture_swatch(&t, x))
                        }),
                )
            };
            // One mode selector: Off / design preset / custom (any hand edit lands on Custom).
            let preset = t.surface_preset();
            let mode = if fx.is_none() {
                0
            } else if fx == preset {
                1
            } else {
                2
            };
            let family = {
                let mut c = t.meta.family.chars();
                c.next()
                    .map(|f| f.to_uppercase().chain(c).collect::<String>())
                    .unwrap_or_default()
            };
            let status = match mode {
                0 => "Off: nothing is drawn.".to_string(),
                1 => format!("Default: the {family} design's own mix."),
                _ => "Custom mix.".to_string(),
            };
            let presets = Segmented::new("sfx-mode", ["Off", "Default", "Custom"])
                .selected(mode)
                .on_select(|i, _, cx| match i {
                    0 => {
                        let mut f = SurfaceFx::NONE;
                        f.targets = dodai_gpui::surface_fx().targets;
                        dodai_gpui::set_surface_fx(cx, f);
                    }
                    1 => dodai_gpui::apply_surface_preset(cx),
                    _ => {}
                });
            let chip = |c: SurfaceClass| {
                let on = fx.targets.contains(c);
                Button::new(
                    gpui::SharedString::from(format!("sfx-t-{}", c.label())),
                    c.label(),
                )
                .variant(if on {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Outline
                })
                .on_click(move |_, _, cx| {
                    let mut f = dodai_gpui::surface_fx();
                    f.targets = f.targets.with(c, !on);
                    dodai_gpui::set_surface_fx(cx, f);
                })
            };
            let shortcut =
                |id: &'static str, label: &'static str, to: dodai_core::SurfaceTargets| {
                    div()
                        .id(id)
                        .cursor_pointer()
                        .flex_none()
                        .whitespace_nowrap()
                        .text_size(px(t.typography.size_sm))
                        .text_color(dodai_gpui::hsla(t.palette.link))
                        .underline()
                        .hover(|s| s.opacity(0.7))
                        .child(label)
                        .on_click(move |_, _, cx| {
                            let mut f = dodai_gpui::surface_fx();
                            f.targets = to;
                            dodai_gpui::set_surface_fx(cx, f);
                        })
                };
            let targets = row()
                .children(SurfaceClass::ALL.iter().map(|c| chip(*c)))
                .child(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .gap(px(t.spacing.md))
                        .child(shortcut(
                            "sfx-t-default",
                            "Default",
                            dodai_core::SurfaceTargets::DEFAULT,
                        ))
                        .child(shortcut(
                            "sfx-t-all",
                            "All",
                            dodai_core::SurfaceTargets::ALL,
                        ))
                        .child(shortcut(
                            "sfx-t-none",
                            "None",
                            dodai_core::SurfaceTargets::NONE,
                        )),
                );
            let id = |p: &'static str, n: &str| gpui::SharedString::from(format!("{p}-{n}"));
            let preview_components = |p: &'static str| {
                row()
                    .child(Button::new(id(p, "b1"), "Primary"))
                    .child(Button::new(id(p, "b2"), "Secondary").variant(ButtonVariant::Secondary))
                    .child(Button::new(id(p, "b3"), "Outline").variant(ButtonVariant::Outline))
            };
            out.push((
                "Surface effects",
                "Presets · Apply to",
                section!("Presets · Apply to")
                    .child(Muted::new(status))
                    .child(div().w_1_2().child(presets))
                    .child(Label::new("Apply to"))
                    .child(targets)
                    .child(Muted::new(
                        "Default is buttons, cards and panels. Picking a texture or effect below changes the mode to Custom.",
                    ))
                    .into_any_element(),
                "Presets · Apply to",
            ));
            out.push((
                "Surface effects",
                "Component textures",
                section!("Component textures")
                    .child(Muted::new(
                        "A pattern on UI components: buttons, inputs and tags. The buttons below wear it. Keep it fine and subtle so labels stay readable.",
                    ))
                    .child({
                        // Components get a tab bar; the three buttons below wear the texture.
                        let list: Vec<Texture> = Texture::ALL
                            .iter()
                            .copied()
                            .filter(|x| x.for_components() && owned.contains(x))
                            .collect();
                        let sel = list.iter().position(|x| *x == fx.texture).unwrap_or(0);
                        Tabs::new("sfx-tex")
                            .scrollable(true)
                            .labels(list.iter().map(|x| x.label()))
                            .selected(sel)
                            .on_select(move |i, _, cx| {
                                let mut f = dodai_gpui::surface_fx();
                                f.texture = list[i];
                                dodai_gpui::set_surface_fx(cx, f);
                            })
                    })
                    .child(amount("sfx-strength", "Strength", fx.strength, 1.0, |f, v| f.strength = v))
                    .child(amount("sfx-scale", "Scale", fx.scale, 4.0, |f, v| f.scale = v.max(0.25)))
                    .children(t.surface_effects().neon.then(|| amount("sfx-neon", "Lit lines", fx.neon, 1.0, |f, v| f.neon = v)))
                    .child(amount("sfx-clear", "Clear centre", fx.clear, 1.0, |f, v| f.clear = v))
                    .child(preview_components("sfx-tx"))
                    .into_any_element(),
                "Component textures",
            ));
            out.push((
                "Surface effects",
                "Container textures",
                section!("Container textures")
                    .child(Muted::new(
                        "A pattern on containers: cards, panels and bars. This panel is a container, so it shows the chosen pattern behind these controls.",
                    ))
                    .child({
                        // Same tab bar as components; this panel is itself a container, so it
                        // wears the pattern.
                        let list: Vec<Texture> = Texture::ALL
                            .iter()
                            .copied()
                            .filter(|x| owned.contains(x))
                            .collect();
                        let sel = list
                            .iter()
                            .position(|x| *x == fx.container_texture)
                            .unwrap_or(0);
                        Tabs::new("sfx-ctex")
                            .scrollable(true)
                            .labels(list.iter().map(|x| x.label()))
                            .selected(sel)
                            .on_select(move |i, _, cx| {
                                let mut f = dodai_gpui::surface_fx();
                                f.container_texture = list[i];
                                dodai_gpui::set_surface_fx(cx, f);
                            })
                    })
                    .child(amount("sfx-cstrength", "Strength", fx.container_strength, 1.0, |f, v| f.container_strength = v))
                    .child(amount("sfx-cscale", "Scale", fx.container_scale, 4.0, |f, v| f.container_scale = v.max(0.25)))
                    .children(t.surface_effects().neon.then(|| amount("sfx-cneon", "Lit lines", fx.neon, 1.0, |f, v| f.neon = v)))
                    .child(amount("sfx-cclear", "Clear centre", fx.clear, 1.0, |f, v| f.clear = v))
                    .into_any_element(),
                "Container textures",
            ));
            let mut light = section!("Light FX").child(Muted::new(
                "Glow and animated accents on components and containers. 0 turns an effect off.",
            ));
            if t.look().luminous {
                light = light.child(shape_props);
            }
            let ef = t.surface_effects();
            let any_light = ef.glitch || ef.sheen || ef.flicker || ef.bloom || ef.pulse || ef.chase;
            if t.look().luminous || any_light {
                light =
                    light
                        .children(ef.glitch.then(|| {
                            amount("sfx-glitch", "Glitch", fx.glitch, 1.0, |f, v| f.glitch = v)
                        }))
                        .children(ef.sheen.then(|| {
                            amount("sfx-sheen", "Sheen", fx.sheen, 1.0, |f, v| f.sheen = v)
                        }))
                        .children(ef.flicker.then(|| {
                            amount("sfx-flicker", "Flicker", fx.flicker, 1.0, |f, v| {
                                f.flicker = v
                            })
                        }))
                        .children(ef.bloom.then(|| {
                            amount("sfx-bloom", "Bloom", fx.bloom, 1.0, |f, v| f.bloom = v)
                        }))
                        .children(ef.pulse.then(|| {
                            amount("sfx-pulse", "Pulse", fx.pulse, 1.0, |f, v| f.pulse = v)
                        }))
                        .children(ef.chase.then(|| {
                            amount("sfx-chase", "Chase", fx.chase, 1.0, |f, v| f.chase = v)
                        }));
                out.push((
                    "Surface effects",
                    "Light FX",
                    light.into_any_element(),
                    "Light FX",
                ));
            }
            if ef.edge || ef.brackets || ef.depth {
                out.push((
                    "Surface effects",
                    "Edges · Depth",
                    section!("Edges · Depth")
                        .child(Muted::new(
                            "How the edges look: light falling inward, corner brackets and an inner shadow. Clear centre (under textures) keeps text clean.",
                        ))
                        .children(ef.edge.then(|| {
                            amount("sfx-edge", "Edge light", fx.edge, 1.0, |f, v| f.edge = v)
                        }))
                        .children(ef.brackets.then(|| {
                            amount("sfx-brackets", "Brackets", fx.brackets, 1.0, |f, v| f.brackets = v)
                        }))
                        .children(ef.depth.then(|| {
                            amount("sfx-depth", "Depth", fx.depth, 1.0, |f, v| f.depth = v)
                        }))
                        .child(preview_components("sfx-ed"))
                        .into_any_element(),
                    "Edges · Depth",
                ));
            }
            if ef.ripple {
                let styles = dodai_core::PressStyle::ALL;
                out.push((
                    "Surface effects",
                    "Click ripple",
                    section!("Click ripple")
                        .child(Muted::new(
                            "Press any control below: the ripple plays from the touch point. Pick how it plays and how strong it is.",
                        ))
                        .child(
                            row()
                                .child(div().w(px(90.0)).child(Label::new("Style")))
                                .child(
                                    Segmented::new("sfx-press", styles.map(|p| p.label()))
                                        .selected(
                                            styles
                                                .iter()
                                                .position(|p| *p == fx.press)
                                                .unwrap_or(usize::MAX),
                                        )
                                        .on_select(move |i, _, cx| {
                                            let mut f = dodai_gpui::surface_fx();
                                            f.press = styles[i];
                                            dodai_gpui::set_surface_fx(cx, f);
                                        }),
                                ),
                        )
                        .child(amount("sfx-ripple", "Strength", fx.ripple, 1.0, |f, v| {
                            f.ripple = v
                        }))
                        .child(preview_components("sfx-press-demo"))
                        .into_any_element(),
                    "Click ripple",
                ));
            }
            if ef.tint || ef.pop || ef.tape || ef.gloss || ef.strip {
                out.push((
                    "Surface effects",
                    "Signature",
                    section!("Signature")
                        .child(Muted::new(
                            "Effects that belong to this design only: tonal wash, a hard pop block and tape, a clear-coat gloss and a light strip.",
                        ))
                        .children(ef.tint.then(|| {
                            amount("sfx-tint", "Tonal wash", fx.tint, 1.0, |f, v| f.tint = v)
                        }))
                        .children(ef.pop.then(|| {
                            amount("sfx-pop", "Pop", fx.pop, 1.0, |f, v| f.pop = v)
                        }))
                        .children(ef.tape.then(|| {
                            amount("sfx-tape", "Tape", fx.tape, 1.0, |f, v| f.tape = v)
                        }))
                        .children(ef.gloss.then(|| {
                            amount("sfx-gloss", "Gloss", fx.gloss, 1.0, |f, v| f.gloss = v)
                        }))
                        .children(ef.strip.then(|| {
                            amount("sfx-strip", "Light strip", fx.strip, 1.0, |f, v| f.strip = v)
                        }))
                        .child(preview_components("sfx-sig"))
                        .into_any_element(),
                    "Signature",
                ));
            }
        }
        {
            use dodai_core::{BorderTint, ShapeKind, ShapeTune};
            let tune = dodai_gpui::shape_tune();
            let round_ok = t.shape.kind() != ShapeKind::Chamfered;
            let widths: [(&str, Option<f32>); 5] = [
                ("Default", None),
                ("None", Some(0.0)),
                ("Hairline", Some(1.0)),
                ("Medium", Some(2.0)),
                ("Thick", Some(3.0)),
            ];
            let width_sel = widths
                .iter()
                .position(|w| w.1 == tune.border_width)
                .unwrap_or(usize::MAX);
            let tint_sel = BorderTint::ALL
                .iter()
                .position(|b| *b == tune.tint)
                .unwrap_or(0);
            let mut sec = section!("Roundness · Borders").child(Muted::new(
                "Applies to every component in every design, immediately. \"Default\" is each design's own.",
            ));
            if round_ok {
                sec = sec.child(setting(
                    "Roundness",
                    "0 is square, 100% is the design's own corners, more is rounder (up to 300%).",
                    div()
                        .w(px(280.0))
                        .child(
                            Slider::new("rb-round", tune.roundness)
                                .range(0.0, 3.0)
                                .step(0.05)
                                .show_value(true)
                                .format(|v| format!("{:.0}%", v * 100.0))
                                .on_change(|v, _, cx| {
                                    let t = dodai_gpui::shape_tune();
                                    dodai_gpui::set_shape_tune(cx, ShapeTune { roundness: v, ..t });
                                }),
                        )
                        .into_any_element(),
                ));
            }
            sec = sec
                .child(setting(
                    "Border width",
                    "Thickness of every outline.",
                    div()
                        .flex()
                        .child(
                            Segmented::new("rb-width", widths.map(|w| w.0))
                                .selected(width_sel)
                                .on_select(move |i, _, cx| {
                                    let t = dodai_gpui::shape_tune();
                                    dodai_gpui::set_shape_tune(
                                        cx,
                                        ShapeTune {
                                            border_width: widths[i].1,
                                            ..t
                                        },
                                    );
                                }),
                        )
                        .into_any_element(),
                ))
                .child(setting(
                    "Border colour",
                    "Which colour outlines take.",
                    div()
                        .flex()
                        .child(
                            Segmented::new("rb-tint", BorderTint::ALL.map(|b| b.label()))
                                .selected(tint_sel)
                                .on_select(move |i, _, cx| {
                                    let t = dodai_gpui::shape_tune();
                                    dodai_gpui::set_shape_tune(
                                        cx,
                                        ShapeTune {
                                            tint: BorderTint::ALL[i],
                                            ..t
                                        },
                                    );
                                }),
                        )
                        .into_any_element(),
                ))
                .child(setting(
                    "Outline all",
                    "Give filled components without a border one too.",
                    div()
                        .flex()
                        .child(
                            Segmented::new("rb-all", ["Off", "On"])
                                .selected(usize::from(tune.outline_all))
                                .on_select(|i, _, cx| {
                                    let t = dodai_gpui::shape_tune();
                                    dodai_gpui::set_shape_tune(
                                        cx,
                                        ShapeTune {
                                            outline_all: i == 1,
                                            ..t
                                        },
                                    );
                                }),
                        )
                        .into_any_element(),
                ))
                .children((t.effects.hard_shadow > 0.0).then(|| {
                    let cur = tune.hard_shadow.unwrap_or(t.effects.hard_shadow);
                    div()
                        .flex()
                        .flex_col()
                        .gap(gap)
                        .child(setting(
                            "Shadow offset",
                            "How far the hard shadow is offset, in px. Only designs with hard shadows have one.",
                            div()
                                .w(px(280.0))
                                .child(
                                    Slider::new("rb-shadow", cur)
                                        .range(0.0, 12.0)
                                        .step(1.0)
                                        .show_value(true)
                                        .format(|v| format!("{v:.0}px"))
                                        .on_change(|v, _, cx| {
                                            let t = dodai_gpui::shape_tune();
                                            dodai_gpui::set_shape_tune(
                                                cx,
                                                ShapeTune { hard_shadow: Some(v), ..t },
                                            );
                                        }),
                                )
                                .into_any_element(),
                        ))
                        .child(setting(
                            "Shadow direction",
                            "Which way the hard shadow falls.",
                            div()
                                .flex()
                                .child(
                                    Segmented::new(
                                        "rb-shadow-dir",
                                        dodai_core::ShadowDir::ALL.map(|d| d.label()),
                                    )
                                    .selected(
                                        dodai_core::ShadowDir::ALL
                                            .iter()
                                            .position(|d| *d == tune.shadow_dir)
                                            .unwrap_or(0),
                                    )
                                    .on_select(move |i, _, cx| {
                                        let t = dodai_gpui::shape_tune();
                                        dodai_gpui::set_shape_tune(
                                            cx,
                                            ShapeTune {
                                                shadow_dir: dodai_core::ShadowDir::ALL[i],
                                                ..t
                                            },
                                        );
                                    }),
                                )
                                .into_any_element(),
                        ))
                }))
                .child(
                    Button::new("rb-reset", "Reset")
                        .variant(ButtonVariant::Outline)
                        .on_click(|_, _, cx| dodai_gpui::set_shape_tune(cx, ShapeTune::DEFAULT)),
                )
                .child(shape_gallery("round"));
            out.push((
                "Shape",
                "Roundness · Borders",
                sec.into_any_element(),
                "Roundness · Borders",
            ));
        }
        if t.look().cut_corners {
            out.push((
                "Shape",
                "Corner shape",
                section!("Corner shape")
                    .child(Muted::new(
                        "The corner cut applies to every component in every theme, immediately. Pick \"Default\" to restore each design's own shape.",
                    ))
                    .child(corner_picker)
                    .child(shape_gallery("corner"))
                    .into_any_element(),
                "Corner shape",
            ));
        }
        if t.look().cut_corners {
            use dodai_gpui::Crop;
            let crops = [
                None,
                Some(Crop::Left),
                Some(Crop::Right),
                Some(Crop::Top),
                Some(Crop::Bottom),
            ];
            let gaps = [None, Some(0.0_f32), Some(2.0), Some(4.0), Some(8.0)];
            let crop_sel = crops
                .iter()
                .position(|c| *c == dodai_gpui::button_crop(cx))
                .unwrap_or(0);
            let gap_sel = gaps
                .iter()
                .position(|g| *g == dodai_gpui::button_cut_gap(cx))
                .unwrap_or(0);
            let crop_row = setting(
                "Crop end",
                "Cuts one end of every button and adds a double tick on the other. Left cuts the top left, Right the bottom right.",
                Segmented::new("bc-crop", ["Default", "Left", "Right", "Top", "Bottom"])
                    .selected(crop_sel)
                    .on_select(move |i, _, cx| dodai_gpui::set_button_crop(cx, crops[i]))
                    .into_any_element(),
            );
            let gap_row = setting(
                "Tick gap",
                "Space between the two tick bars, left open in the border.",
                Segmented::new("bc-gap", ["Default", "0 px", "2 px", "4 px", "8 px"])
                    .selected(gap_sel)
                    .on_select(move |i, _, cx| dodai_gpui::set_button_cut_gap(cx, gaps[i]))
                    .into_any_element(),
            );
            let preview = div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .child(Muted::new("Preview"))
                .child(
                    row()
                        .child(Button::new("bp-p", "Primary"))
                        .child(Button::new("bp-s", "Secondary").variant(ButtonVariant::Secondary))
                        .child(Button::new("bp-o", "Outline").variant(ButtonVariant::Outline))
                        .child(Button::new("bp-c", "Scrim").variant(ButtonVariant::Scrim))
                        .child(Button::new("bp-d", "Danger").variant(ButtonVariant::Danger)),
                );
            out.push((
                "Shape",
                "Button crop",
                section!("Button crop")
                    .child(Muted::new(
                        "Settings apply to every button below and across the showcase. Ready-made cropped buttons are under Actions.",
                    ))
                    .child(crop_row)
                    .child(gap_row)
                    .child(preview)
                    .into_any_element(),
                "Button crop",
            ));
        }
        if t.look().accent_border {
            use dodai_gpui::{AccentCorner, BorderStyle as B};
            let cur = dodai_gpui::section_border(cx);
            let presets: [(&str, B); 5] = [
                ("None", B::NONE),
                ("Subtle", B::SUBTLE),
                ("Thin", B::THIN),
                ("Normal", B::NORMAL),
                ("Bold", B::BOLD),
            ];
            let preset_sel = presets
                .iter()
                .position(|(_, b)| b.width == cur.width && b.opacity == cur.opacity)
                .unwrap_or(usize::MAX);
            let corner_sel = AccentCorner::ALL
                .iter()
                .position(|c| *c == cur.accent)
                .unwrap_or(usize::MAX);
            let slider = |id: &'static str,
                          v: f32,
                          lo: f32,
                          hi: f32,
                          unit: &'static str,
                          f: fn(B, f32) -> B| {
                div().w(px(280.0)).child(
                    Slider::new(id, v)
                        .range(lo, hi)
                        .step(0.25)
                        .show_value(true)
                        .format(move |v| format!("{v:.2}{unit}"))
                        .on_change(move |v, _, cx| {
                            let b = dodai_gpui::section_border(cx);
                            dodai_gpui::set_section_border(cx, f(b, v));
                        }),
                )
            };
            let theme_shadow = t.effects.hard_shadow.max(0.5);
            out.push((
                "Shape",
                "Border",
                section!("Border")
                    .child(Muted::new(
                        "Every section frame is a grey contour plus an accent line on the opposite corner. These settings change all sections at once; a single panel can use `.border(..)`.",
                    ))
                    .child(setting(
                        "Preset",
                        "Width and strength of the grey contour.",
                        div()
                            .flex()
                            .child(
                                Segmented::new("bd-preset", presets.map(|p| p.0))
                                    .selected(preset_sel)
                                    .on_select(move |i, _, cx| {
                                        let c = dodai_gpui::section_border(cx);
                                        let p = presets[i].1;
                                        dodai_gpui::set_section_border(
                                            cx,
                                            B {
                                                accent: c.accent,
                                                accent_width: c.accent_width,
                                                ..p
                                            },
                                        );
                                    }),
                            )
                            .into_any_element(),
                    ))
                    .child(setting(
                        "Accent corner",
                        "Where the bright line sits. The grey contour moves to the opposite side.",
                        div()
                            .flex()
                            .child(
                                Segmented::new("bd-corner", AccentCorner::ALL.map(|c| c.name()))
                                    .selected(corner_sel)
                                    .on_select(move |i, _, cx| {
                                        let b = dodai_gpui::section_border(cx);
                                        dodai_gpui::set_section_border(cx, b.accent(AccentCorner::ALL[i]));
                                    }),
                            )
                            .into_any_element(),
                    ))
                    .child(setting(
                        "Grey width",
                        "Thickness of the grey contour in px.",
                        slider("bd-width", cur.width, 0.0, 4.0, " px", |b, v| B {
                            width: v.clamp(0.0, 8.0),
                            ..b
                        })
                        .into_any_element(),
                    ))
                    .child(setting(
                        "Accent width",
                        "Thickness of the bright line in px.",
                        slider(
                            "bd-accent",
                            cur.accent_width.unwrap_or(theme_shadow),
                            0.5,
                            6.0,
                            " px",
                            |b, v| b.accent_width(v),
                        )
                        .into_any_element(),
                    ))
                    .child(setting(
                        "Grey opacity",
                        "0 hides the contour, 1 is fully solid.",
                        slider("bd-opacity", cur.opacity, 0.0, 1.0, "", |b, v| B {
                            opacity: v.clamp(0.0, 1.0),
                            ..b
                        })
                        .into_any_element(),
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(t.spacing.sm))
                            .child(Muted::new("Presets, side by side"))
                            .child({
                                let mut r = row();
                                for (name, b) in presets {
                                    r = r.child(
                                        div().flex_1().min_w(px(140.0)).child(
                                            Panel::new().border(b.accent(cur.accent)).child(
                                                col().child(Label::new(name)).child(Muted::new(
                                                    format!(
                                                        "{:.2} px · {:.0}%",
                                                        b.width,
                                                        b.opacity * 100.0
                                                    ),
                                                )),
                                            ),
                                        ),
                                    );
                                }
                                r
                            }),
                    )
                    .into_any_element(),
                "Border",
            ));
        }
        out
    }
}
