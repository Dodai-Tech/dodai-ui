//! The "Shape" category sections.

use super::super::*;

impl Showcase {
    /// The builders of this category, one per group of sections (built lazily).
    pub(crate) fn groups_shape() -> Vec<Group> {
        vec![
            Self::gallery_section,
            Self::presets_section,
            Self::component_textures_section,
            Self::container_textures_section,
            Self::frost_section,
            Self::light_section,
            Self::edges_section,
            Self::ripple_section,
            Self::signature_section,
            Self::roundness_section,
            Self::corner_shape_section,
            Self::crop_section,
            Self::border_section,
        ]
    }

    fn gallery_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        // Every design: its own corner radius, border and shadow on real components.
        vec![(
            "Shape",
            "Shape gallery",
            self.section("Shape gallery", cx)
                .child(Muted::new(
                    "The design system's own corners, borders and shadows on real components, with the tokens behind them.",
                ))
                .child(shape_tokens_view(&t))
                .child(shape_gallery(gap, "gallery"))
                .into_any_element(),
            "Shape gallery",
        )]
    }

    fn presets_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        use dodai_core::{SurfaceClass, SurfaceFx};
        let fx = dodai_gpui::surface_fx();
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
            0 => "Off: no texture or effect is drawn on any component.".to_string(),
            1 => format!("Default: the {family} design's own mix."),
            _ => "Custom: your own mix of textures and effects.".to_string(),
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
        // A toggle per class: filled with a check when on, outlined when off.
        let chip = |c: SurfaceClass| {
            let on = fx.targets.contains(c);
            Button::new(
                gpui::SharedString::from(format!("sfx-t-{}", c.label())),
                if on {
                    format!("\u{2713} {}", c.label())
                } else {
                    c.label().to_string()
                },
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
        let shortcut = |id: &'static str, label: &'static str, to: dodai_core::SurfaceTargets| {
            Button::new(id, label)
                .variant(ButtonVariant::Ghost)
                .on_click(move |_, _, cx| {
                    let mut f = dodai_gpui::surface_fx();
                    f.targets = to;
                    dodai_gpui::set_surface_fx(cx, f);
                })
        };
        // Name and one-line meaning on the left, the toggles on the right.
        let group = |title: &'static str, meaning: &'static str, container: bool| {
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.lg))
                .child(
                    div()
                        .w(px(200.0))
                        .flex_none()
                        .flex()
                        .flex_col()
                        .child(Label::new(title))
                        .child(Muted::new(meaning)),
                )
                .child(
                    row().children(
                        SurfaceClass::ALL
                            .iter()
                            .filter(|c| c.is_container() == container)
                            .map(|c| chip(*c)),
                    ),
                )
        };
        let section = self
            .section("Presets · Apply to", cx)
            .child(Label::new("Mode"))
            .child(div().w_1_2().child(presets))
            .child(Muted::new(status))
            .child(Divider::new())
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(div().flex_1().child(Label::new("Apply effects to")))
                    .child(
                        row()
                            .gap(px(t.spacing.xs))
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
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.lg))
                    .child(group(
                "Controls",
                "Things you press or type in",
                false,
            ))
                    .child(group(
                "Containers",
                "Things that hold other content",
                true,
            ))
            )
            .child(Muted::new(
                "Default is buttons, cards and panels. Picking a texture or effect below changes the mode to Custom.",
            ));
        out.push((
            "Surface effects",
            "Presets · Apply to",
            section.into_any_element(),
            "Presets · Apply to",
        ));
        out
    }

    fn component_textures_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        use dodai_core::Texture;
        let owned = t.surface_textures();
        let fx = dodai_gpui::surface_fx();
        out.push((
                "Surface effects",
                "Component textures",
                self.section("Component textures", cx)
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
                    .child(fx_amount(gap, "sfx-strength", "Strength", fx.strength, 1.0, |f, v| f.strength = v))
                    .child(fx_amount(gap, "sfx-scale", "Scale", fx.scale, 4.0, |f, v| f.scale = v.max(0.25)))
                    .children(t.surface_effects().neon.then(|| fx_amount(gap, "sfx-neon", "Lit lines", fx.neon, 1.0, |f, v| f.neon = v)))
                    .child(fx_amount(gap, "sfx-clear", "Clear centre", fx.clear, 1.0, |f, v| f.clear = v))
                    .child(fx_preview(gap, "sfx-tx"))
                    .into_any_element(),
                "Component textures",
            ));
        out
    }

    fn container_textures_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        use dodai_core::Texture;
        let owned = t.surface_textures();
        let fx = dodai_gpui::surface_fx();
        out.push((
                "Surface effects",
                "Container textures",
                self.section("Container textures", cx)
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
                    .child(fx_amount(gap, "sfx-cstrength", "Strength", fx.container_strength, 1.0, |f, v| f.container_strength = v))
                    .child(fx_amount(gap, "sfx-cscale", "Scale", fx.container_scale, 4.0, |f, v| f.container_scale = v.max(0.25)))
                    .children(t.surface_effects().neon.then(|| fx_amount(gap, "sfx-cneon", "Lit lines", fx.neon, 1.0, |f, v| f.neon = v)))
                    .child(fx_amount(gap, "sfx-cclear", "Clear centre", fx.clear, 1.0, |f, v| f.clear = v))
                    .into_any_element(),
                "Container textures",
            ));
        out
    }

    fn frost_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let fx = dodai_gpui::surface_fx();
        let own = dodai_core::Surface::new().frost(1.0);
        let backdrop = Segmented::new("sfx-frost-bg", ["Canvas", "Picture"])
            .selected(usize::from(dodai_gpui::has_wallpaper()))
            .on_select(|i, _, cx| {
                if i == 1 {
                    let (w, h) = (1200, 750);
                    let p = theme_of(cx).palette;
                    dodai_gpui::set_wallpaper(cx, glass_picture(&p, w, h), w, h);
                } else {
                    dodai_gpui::clear_wallpaper(cx);
                }
            });
        let labelled = |name: &'static str, control: gpui::AnyElement| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(gap)
                .child(div().w(px(120.0)).flex_none().child(Label::new(name)))
                .child(control)
        };
        vec![(
            "Surface effects",
            "Frosted glass",
            self.section("Frosted glass", cx)
                .child(Muted::new(
                    "Any surface can be frosted: its fill turns as see-through as the design's glass and a blurred copy of the backdrop shows under it. The slider frosts every container (cards, panels, bars, this panel too); a component can frost itself with Surface::new().frost(1.0), as the buttons and the card below do.",
                ))
                .child(fx_amount(gap, "sfx-frost", "Frost", fx.frost, 1.0, |f, v| f.frost = v))
                .child(labelled("Backdrop", backdrop.into_any_element()))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(gap)
                        .child(Button::new("sfx-frost-p", "Primary").surface(own))
                        .child(
                            Button::new("sfx-frost-s", "Secondary")
                                .variant(ButtonVariant::Secondary)
                                .surface(own),
                        )
                        .child(
                            Button::new("sfx-frost-o", "Outline")
                                .variant(ButtonVariant::Outline)
                                .surface(own),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(gap)
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                Card::new(CardVariant::Elevated)
                                    .headline("Frosted card")
                                    .surface(own),
                            ),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(220.0))
                                .child(Card::new(CardVariant::Elevated).headline("Plain card").surface(dodai_core::Surface::new().frost(0.0))),
                        ),
                )
                .into_any_element(),
            "Frosted glass",
        )]
    }

    fn light_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        let fx = dodai_gpui::surface_fx();
        let mut light = self.section("Light FX", cx).child(Muted::new(
            "Glow and animated accents on components and containers. 0 turns an effect off.",
        ));
        if t.look().luminous {
            light = light.child(shape_props(&t, glow_intensity(cx)));
        }
        let ef = t.surface_effects();
        let any_light = ef.glitch || ef.sheen || ef.flicker || ef.bloom || ef.pulse || ef.chase;
        if t.look().luminous || any_light {
            light = light
                .children(ef.glitch.then(|| {
                    fx_amount(gap, "sfx-glitch", "Glitch", fx.glitch, 1.0, |f, v| {
                        f.glitch = v
                    })
                }))
                .children(ef.sheen.then(|| {
                    fx_amount(gap, "sfx-sheen", "Sheen", fx.sheen, 1.0, |f, v| f.sheen = v)
                }))
                .children(ef.flicker.then(|| {
                    fx_amount(gap, "sfx-flicker", "Flicker", fx.flicker, 1.0, |f, v| {
                        f.flicker = v
                    })
                }))
                .children(ef.bloom.then(|| {
                    fx_amount(gap, "sfx-bloom", "Bloom", fx.bloom, 1.0, |f, v| f.bloom = v)
                }))
                .children(ef.pulse.then(|| {
                    fx_amount(gap, "sfx-pulse", "Pulse", fx.pulse, 1.0, |f, v| f.pulse = v)
                }))
                .children(ef.chase.then(|| {
                    fx_amount(gap, "sfx-chase", "Chase", fx.chase, 1.0, |f, v| f.chase = v)
                }));
            out.push((
                "Surface effects",
                "Light FX",
                light.into_any_element(),
                "Light FX",
            ));
        }
        out
    }

    fn edges_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        let fx = dodai_gpui::surface_fx();
        let ef = t.surface_effects();
        if ef.edge || ef.brackets || ef.depth {
            out.push((
                    "Surface effects",
                    "Edges · Depth",
                    self.section("Edges · Depth", cx)
                        .child(Muted::new(
                            "How the edges look: light falling inward, corner brackets and an inner shadow. Clear centre (under textures) keeps text clean.",
                        ))
                        .children(ef.edge.then(|| {
                            fx_amount(gap, "sfx-edge", "Edge light", fx.edge, 1.0, |f, v| f.edge = v)
                        }))
                        .children(ef.brackets.then(|| {
                            fx_amount(gap, "sfx-brackets", "Brackets", fx.brackets, 1.0, |f, v| f.brackets = v)
                        }))
                        .children(ef.depth.then(|| {
                            fx_amount(gap, "sfx-depth", "Depth", fx.depth, 1.0, |f, v| f.depth = v)
                        }))
                        .child(fx_preview(gap, "sfx-ed"))
                        .into_any_element(),
                    "Edges · Depth",
                ));
        }
        out
    }

    fn ripple_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        let fx = dodai_gpui::surface_fx();
        let ef = t.surface_effects();
        if ef.ripple {
            let styles = dodai_core::PressStyle::ALL;
            out.push((
                    "Surface effects",
                    "Click ripple",
                    self.section("Click ripple", cx)
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
                        .child(fx_amount(gap, "sfx-ripple", "Strength", fx.ripple, 1.0, |f, v| {
                            f.ripple = v
                        }))
                        .child(fx_preview(gap, "sfx-press-demo"))
                        .into_any_element(),
                    "Click ripple",
                ));
        }
        out
    }

    fn signature_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.surface_textures().is_empty() {
            return out;
        }
        let fx = dodai_gpui::surface_fx();
        let ef = t.surface_effects();
        if ef.tint || ef.pop || ef.tape || ef.gloss || ef.strip || ef.wash || ef.rim {
            out.push((
                    "Surface effects",
                    "Signature",
                    self.section("Signature", cx)
                        .child(Muted::new(
                            "Effects that belong to this design only: tonal wash, a hard pop block and tape, a clear-coat gloss, a light strip and a gradient wash.",
                        ))
                        .children(ef.tint.then(|| {
                            fx_amount(gap, "sfx-tint", "Tonal wash", fx.tint, 1.0, |f, v| f.tint = v)
                        }))
                        .children(ef.pop.then(|| {
                            fx_amount(gap, "sfx-pop", "Pop", fx.pop, 1.0, |f, v| f.pop = v)
                        }))
                        .children(ef.tape.then(|| {
                            fx_amount(gap, "sfx-tape", "Tape", fx.tape, 1.0, |f, v| f.tape = v)
                        }))
                        .children(ef.gloss.then(|| {
                            fx_amount(gap, "sfx-gloss", "Gloss", fx.gloss, 1.0, |f, v| f.gloss = v)
                        }))
                        .children(ef.strip.then(|| {
                            fx_amount(gap, "sfx-strip", "Light strip", fx.strip, 1.0, |f, v| f.strip = v)
                        }))
                        .children(ef.wash.then(|| {
                            fx_amount(gap, "sfx-wash", "Gradient wash", fx.wash, 1.0, |f, v| f.wash = v)
                        }))
                        .children(ef.rim.then(|| {
                            fx_amount(gap, "sfx-rim", "Gradient rim", fx.rim, 1.0, |f, v| f.rim = v)
                        }))
                        .child(fx_preview(gap, "sfx-sig"))
                        .into_any_element(),
                    "Signature",
                ));
        }
        out
    }

    fn roundness_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        use dodai_core::{
            BorderTint, CornerStyle, GradientMode, RoundnessGroup, ShadowDir, ShapeKind,
        };
        let t = theme_of(cx);
        let tune = dodai_gpui::shape_tune();
        let percent: fn(f32) -> String = |v| format!("{:.0}%", v * 100.0);
        let mut sec = self.section("Roundness · Borders", cx).child(Muted::new(
            "Applies to every component in every design, immediately. \"Default\" is each design's own.",
        ));
        if t.shape.kind() != ShapeKind::Chamfered {
            sec = sec.child(tune_slider(
                &t,
                "Roundness",
                "0 is square, 100% is the design's own corners, more is rounder (up to 200%).",
                ("rb-round", 0),
                (tune.roundness, 2.0, 0.05),
                percent,
                |tn, v| tn.roundness = v,
            ));
            for (k, g) in RoundnessGroup::ALL.into_iter().enumerate() {
                sec = sec.child(tune_slider(
                    &t,
                    g.label(),
                    "On top of the global roundness; 100% leaves this group alone.",
                    ("rb-group", k),
                    (tune.groups[k], 2.0, 0.05),
                    percent,
                    move |tn, v| tn.groups[k] = v,
                ));
            }
        }
        const STYLES: [(&str, Option<CornerStyle>); 4] = [
            ("Design", None),
            ("Round", Some(CornerStyle::Round)),
            ("Cut", Some(CornerStyle::Cut)),
            ("Scoop", Some(CornerStyle::Scoop)),
        ];
        const WIDTHS: [(&str, Option<f32>); 5] = [
            ("Default", None),
            ("None", Some(0.0)),
            ("Hairline", Some(1.0)),
            ("Medium", Some(2.0)),
            ("Thick", Some(3.0)),
        ];
        const DENSITIES: [(&str, f32); 3] = [("Compact", 0.8), ("Default", 1.0), ("Roomy", 1.25)];
        let style_sel = STYLES.iter().position(|s| s.1 == tune.corner_style);
        let width_sel = WIDTHS.iter().position(|w| w.1 == tune.border_width);
        let tint_sel = BorderTint::ALL.iter().position(|b| *b == tune.tint);
        sec = sec
            .child(tune_choice(
                &t,
                "Corner style",
                "Round, straight cut or concave scoop, on every design (Cut and Scoop take each role's size).",
                "rb-style",
                (STYLES.map(|s| s.0).to_vec(), style_sel.unwrap_or(0)),
                |tn, i| tn.corner_style = STYLES[i].1,
            ))
            .child(tune_choice(
                &t,
                "Border width",
                "Thickness of every outline.",
                "rb-width",
                (WIDTHS.map(|w| w.0).to_vec(), width_sel.unwrap_or(usize::MAX)),
                |tn, i| tn.border_width = WIDTHS[i].1,
            ))
            .child(tune_choice(
                &t,
                "Border colour",
                "Which colour outlines take.",
                "rb-tint",
                (BorderTint::ALL.map(|b| b.label()).to_vec(), tint_sel.unwrap_or(0)),
                |tn, i| tn.tint = BorderTint::ALL[i],
            ))
            .child(setting(
                &t,
                "Control colour",
                "Which colour filled controls take: buttons, checked boxes, on switches, slider fills, the selected tab. Taken from the current theme and kept at 3:1 on the page. Also in the top bar.",
                div()
                    .flex()
                    .child(control_colour_picker("rb-fill", cx))
                    .into_any_element(),
            ))
            .child(tune_choice(
                &t,
                "Outline all",
                "Give filled components without a border one too.",
                "rb-all",
                (vec!["Off", "On"], usize::from(tune.outline_all)),
                |tn, i| tn.outline_all = i == 1,
            ))
            .child(tune_choice(
                &t,
                "Density",
                "Control height and spacing: compact, as designed, or roomy. Type and icons keep their size.",
                "rb-density",
                (
                    DENSITIES.map(|d| d.0).to_vec(),
                    DENSITIES
                        .iter()
                        .position(|d| (d.1 - tune.density).abs() < 1e-3)
                        .unwrap_or(usize::MAX),
                ),
                |tn, i| tn.density = DENSITIES[i].1,
            ));
        if t.surface_effects().wash {
            sec = sec.child(tune_choice(
                &t,
                "Gradient",
                "How strongly the design's gradient shows: the canvas mesh, the wash and the rim.",
                "rb-gradient",
                (
                    GradientMode::ALL.map(|g| g.label()).to_vec(),
                    GradientMode::ALL
                        .iter()
                        .position(|g| *g == tune.gradient)
                        .unwrap_or(1),
                ),
                |tn, i| tn.gradient = GradientMode::ALL[i],
            ));
        }
        if t.effects.hard_shadow > 0.0 {
            let cur = tune.hard_shadow.unwrap_or(t.effects.hard_shadow);
            let dir = ShadowDir::ALL.iter().position(|d| *d == tune.shadow_dir);
            sec = sec
                .child(tune_slider(
                    &t,
                    "Shadow offset",
                    "How far the hard shadow is offset, in px. Only designs with hard shadows have one.",
                    ("rb-shadow", 0),
                    (cur, 12.0, 1.0),
                    |v| format!("{v:.0}px"),
                    |tn, v| tn.hard_shadow = Some(v),
                ))
                .child(tune_choice(
                    &t,
                    "Shadow direction",
                    "Which way the hard shadow falls.",
                    "rb-shadow-dir",
                    (ShadowDir::ALL.map(|d| d.label()).to_vec(), dir.unwrap_or(0)),
                    |tn, i| tn.shadow_dir = ShadowDir::ALL[i],
                ));
        }
        let sec = sec
            .child(
                Button::new("rb-reset", "Reset")
                    .variant(ButtonVariant::Outline)
                    .on_click(|_, _, cx| {
                        dodai_gpui::set_shape_tune(cx, dodai_core::ShapeTune::DEFAULT)
                    }),
            )
            .child(shape_gallery(px(t.spacing.md), "round"));
        vec![(
            "Shape",
            "Roundness · Borders",
            sec.into_any_element(),
            "Roundness · Borders",
        )]
    }

    fn corner_shape_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        if t.look().cut_corners {
            out.push((
                "Shape",
                "Corner shape",
                self.section("Corner shape", cx)
                    .child(Muted::new(
                        "The corner cut applies to every component in every theme, immediately. Pick \"Default\" to restore each design's own shape.",
                    ))
                    .child(corner_picker(&t, cx))
                    .child(shape_gallery(gap, "corner"))
                    .into_any_element(),
                "Corner shape",
            ));
        }
        out
    }

    fn crop_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        {
            use dodai_gpui::Crop;
            // Segments: 0 hands the choice back to the design, 1 is "no crop", then these four.
            let crops = [Crop::Left, Crop::Right, Crop::Top, Crop::Bottom];
            let gaps = [None, Some(0.0_f32), Some(2.0), Some(4.0), Some(8.0)];
            let crop_sel = match dodai_gpui::button_crop(cx) {
                None => 0,
                Some(None) => 1,
                Some(Some(c)) => 2 + crops.iter().position(|x| *x == c).unwrap_or(0),
            };
            let gap_sel = gaps
                .iter()
                .position(|g| *g == dodai_gpui::button_cut_gap(cx))
                .unwrap_or(0);
            let crop_row = setting(&t,
                "Crop end",
                "Cuts one end of every button and adds a double tick on the other. Left cuts the top left, Right the bottom right.",
                Segmented::new("bc-crop", ["Design", "Off", "Left", "Right", "Top", "Bottom"])
                    .selected(crop_sel)
                    .on_select(move |i, _, cx| match i {
                        0 => dodai_gpui::reset_button_crop(cx),
                        1 => dodai_gpui::set_button_crop(cx, None),
                        i => dodai_gpui::set_button_crop(cx, Some(crops[i - 2])),
                    })
                    .into_any_element(),
            );
            let gap_row = setting(
                &t,
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
                self.section("Button crop", cx)
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
        out
    }

    fn border_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let mut out: Vec<Section> = Vec::new();
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
                div().w_full().max_w(px(280.0)).child(
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
                self.section("Border", cx)
                    .child(Muted::new(
                        "Every section frame is a grey contour plus an accent line on the opposite corner. These settings change all sections at once; a single panel can use `.border(..)`.",
                    ))
                    .child(setting(&t,
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
                    .child(setting(&t,
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
                    .child(setting(&t,
                        "Grey width",
                        "Thickness of the grey contour in px.",
                        slider("bd-width", cur.width, 0.0, 4.0, " px", |b, v| B {
                            width: v.clamp(0.0, 8.0),
                            ..b
                        })
                        .into_any_element(),
                    ))
                    .child(setting(&t,
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
                    .child(setting(&t,
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

/// Apply `edit` to the current global shape tune.
/// The control colour choices of the active theme as designed (each design, accent and mode
/// offers its own; look-alikes of the default are left out), writing into the shape tune.
pub(crate) fn control_colour_picker(id: &'static str, cx: &App) -> Segmented {
    use dodai_core::ControlColour;
    use dodai_gpui::ActiveDodai as _;
    let offered = ControlColour::offered(cx.dodai().theme());
    let current = dodai_gpui::shape_tune().control_colour;
    let sel = offered.iter().position(|c| *c == current).unwrap_or(0);
    let labels: Vec<&'static str> = offered.iter().map(|c| c.label()).collect();
    Segmented::new(id, labels)
        .selected(sel)
        .on_select(move |i, _, cx| edit_tune(cx, |tune| tune.control_colour = offered[i]))
}

fn edit_tune(cx: &mut App, edit: impl FnOnce(&mut dodai_core::ShapeTune)) {
    let mut tune = dodai_gpui::shape_tune();
    edit(&mut tune);
    dodai_gpui::set_shape_tune(cx, tune);
}

/// A setting with a slider; `range` is `(value, max, step)`, `set` writes the value into the tune.
fn tune_slider(
    t: &dodai_core::Theme,
    label: &'static str,
    hint: &'static str,
    id: (&'static str, usize),
    range: (f32, f32, f32),
    unit: fn(f32) -> String,
    set: impl Fn(&mut dodai_core::ShapeTune, f32) + 'static,
) -> gpui::Div {
    let (value, max, step) = range;
    let slider = Slider::new(id, value)
        .range(0.0, max)
        .step(step)
        .show_value(true)
        .format(unit)
        .on_change(move |v, _, cx| edit_tune(cx, |tune| set(tune, v)));
    setting(
        t,
        label,
        hint,
        div()
            .w_full()
            .max_w(px(280.0))
            .child(slider)
            .into_any_element(),
    )
}

/// A setting with a segmented choice; `choice` is `(labels, selected)`.
fn tune_choice(
    t: &dodai_core::Theme,
    label: &'static str,
    hint: &'static str,
    id: &'static str,
    choice: (Vec<&'static str>, usize),
    set: impl Fn(&mut dodai_core::ShapeTune, usize) + 'static,
) -> gpui::Div {
    let (labels, selected) = choice;
    let seg = Segmented::new(id, labels)
        .selected(selected)
        .on_select(move |i, _, cx| edit_tune(cx, |tune| set(tune, i)));
    setting(t, label, hint, div().flex().child(seg).into_any_element())
}

/// One labelled setting: fixed label column, the control, a muted hint underneath.
fn setting(
    t: &dodai_core::Theme,
    label: &'static str,
    hint: &'static str,
    control: gpui::AnyElement,
) -> gpui::Div {
    // Label column and control wrap onto two lines when the page is narrow; the control column
    // may shrink to the page width so its hint wraps instead of overflowing.
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap_x(px(t.spacing.lg))
        .gap_y(px(t.spacing.xs))
        .child(
            div()
                .w(px(120.0))
                .flex_none()
                .pt(px(t.spacing.sm))
                .child(Label::new(label)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_grow()
                .flex_shrink()
                .flex_basis(px(340.0))
                .max_w_full()
                .gap(px(t.spacing.xs))
                .child(control)
                .child(Muted::new(hint)),
        )
}

/// Global: picking a cut changes every component in every theme.
fn corner_picker(t: &dodai_core::Theme, cx: &mut App) -> gpui::Div {
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
    setting(t,
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
}

/// The same row of buttons, tags and cards, to show a shape setting on real components.
fn shape_gallery(gap: gpui::Pixels, scope: &str) -> gpui::Div {
    let row = || div().flex().flex_wrap().items_center().gap(gap);
    let col = || div().flex().flex_col().gap(gap);
    let id = |k: &str| gpui::SharedString::from(format!("{scope}-sg-{k}"));
    col()
        .child(
            row()
                .child(Button::new(id("p"), "Primary"))
                .child(Button::new(id("s"), "Secondary").variant(ButtonVariant::Secondary))
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
                    div()
                        .flex_1()
                        .min_w(px(220.0))
                        .child(Card::new(CardVariant::Elevated).headline("Elevated card")),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(220.0))
                        .child(Panel::new().title("Panel").child(Muted::new("Panel body"))),
                ),
        )
}

/// The glow slider shown on luminous designs.
fn shape_props(t: &dodai_core::Theme, glow: f32) -> gpui::Div {
    let row = || {
        div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(t.spacing.md))
    };
    div().flex().flex_col().gap(px(t.spacing.sm)).child(
        row().child(Label::new("Glow")).child(
            div().w_full().max_w(px(200.0)).child(
                Slider::new("glow-slider", glow)
                    .range(0.0, 2.0)
                    .step(0.05)
                    .show_value(true)
                    .format(|v| format!("{v:.2}"))
                    .on_change(move |v, _, cx| set_glow_intensity(cx, v)),
            ),
        ),
    )
}

/// A labelled slider that edits one strength of the global surface effects.
fn fx_amount(
    gap: gpui::Pixels,
    id: &'static str,
    name: &'static str,
    v: f32,
    max: f32,
    set: fn(&mut dodai_core::SurfaceFx, f32),
) -> gpui::Div {
    let slider = Slider::new(id, v)
        .range(0.0, max)
        .step(0.05)
        .show_value(true)
        .format(|v| format!("{v:.2}"))
        .on_change(move |v, _, cx| {
            let mut f = dodai_gpui::surface_fx();
            set(&mut f, v);
            dodai_gpui::set_surface_fx(cx, f);
        });
    div()
        .flex()
        .flex_wrap()
        .items_start()
        .gap(gap)
        .child(
            div()
                .w(px(120.0))
                .flex_none()
                .pt(px(4.0))
                .child(Label::new(name)),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .flex_grow()
                .flex_shrink()
                .flex_basis(px(260.0))
                .max_w_full()
                .gap(px(2.0))
                .child(div().w_full().max_w(px(260.0)).child(slider))
                .child(Muted::new(fx_hint(id))),
        )
}

/// A picture with hard edges in the theme's colours, so the frosted blur shows: bands of the
/// accents across the canvas and a few discs.
pub(crate) fn glass_picture(p: &dodai_core::Palette, w: usize, h: usize) -> Vec<u8> {
    let bands = [p.primary, p.info, p.success, p.warning, p.danger];
    let discs = [
        (0.2, 0.3, 0.12, p.warning),
        (0.7, 0.6, 0.18, p.primary),
        (0.45, 0.85, 0.1, p.success),
    ];
    let mut out = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let band = ((u + v * 0.6) * 6.0) as usize % bands.len();
            let mut c = bands[band].mix(p.background, 0.35);
            for (cx, cy, r, colour) in discs {
                if ((u - cx) * w as f32 / h as f32).hypot(v - cy) < r {
                    c = colour;
                }
            }
            out.extend([c.r, c.g, c.b, 255]);
        }
    }
    out
}

/// What an effect slider does, keyed by its id (the `sfx-c…` twins share the text).
fn fx_hint(id: &str) -> &'static str {
    let key = id.trim_start_matches("sfx-");
    let key = match key {
        "cstrength" | "cscale" | "cneon" | "cclear" => &key[1..],
        k => k,
    };
    match key {
        "strength" => "How visible the pattern is. 0 hides it.",
        "scale" => "Size of the pattern. 1 is the design's own.",
        "neon" => "Brightness of the lit pattern lines.",
        "clear" => "Keeps the middle clean so labels stay readable.",
        "glitch" => "Short digital tears, now and then.",
        "sheen" => "A slow light sweep across the surface.",
        "flicker" => "Unsteady light, like a failing tube.",
        "bloom" => "A soft halo around bright parts.",
        "pulse" => "Slow breathing of the glow.",
        "chase" => "Light that travels along the border.",
        "edge" => "Light falling in from the borders.",
        "brackets" => "Marks in the four corners.",
        "depth" => "An inner shadow that sinks the surface.",
        "ripple" => "How strong the click ripple plays.",
        "tint" => "A tonal wash over the whole surface.",
        "pop" => "A hard offset block behind the surface.",
        "tape" => "A strip of tape across a corner.",
        "gloss" => "A clear-coat reflection across the top.",
        "strip" => "A thin strip of light.",
        "wash" => "The design's gradient tinting the surface.",
        "rim" => "The outline lit by the gradient, brightest top left.",
        "frost" => "See-through fill over a blurred backdrop. 0 is an opaque fill.",
        _ => "",
    }
}

/// Three real buttons, so each surface effect shows on components.
fn fx_preview(gap: gpui::Pixels, p: &'static str) -> gpui::Div {
    let row = || div().flex().flex_wrap().items_center().gap(gap);
    let id = |n: &str| gpui::SharedString::from(format!("{p}-{n}"));
    row()
        .child(Button::new(id("b1"), "Primary"))
        .child(Button::new(id("b2"), "Secondary").variant(ButtonVariant::Secondary))
        .child(Button::new(id("b3"), "Outline").variant(ButtonVariant::Outline))
}

/// The pinned strip above the Shape and Surface-effects pages: real components that restyle the
/// moment any control below changes, so nothing has to be scrolled to see its effect.
pub(crate) fn live_preview(t: &dodai_core::Theme, narrow: bool) -> gpui::Div {
    let gap = px(t.spacing.md);
    let row = || div().flex().flex_wrap().items_center().gap(gap);
    let card = |k: &'static str, v: CardVariant, h: &'static str| {
        div()
            .w(px(150.0))
            .child(Card::new(v).headline(h))
            .id(gpui::SharedString::from(format!("pv-{k}")))
    };
    div()
        .flex()
        .flex_none()
        .flex_col()
        .gap(px(t.spacing.sm))
        .px(px(if narrow { t.spacing.md } else { t.spacing.lg }))
        .py(px(if narrow { t.spacing.sm } else { t.spacing.md }))
        .border_b_1()
        .border_color(hsla(t.palette.border))
        .children((!narrow).then(|| {
            Muted::new(
                "Live preview: every shape and effect setting below is applied here at once.",
            )
        }))
        .child(
            row()
                .child(Button::new("pv-p", "Primary"))
                .child(Button::new("pv-s", "Secondary").variant(ButtonVariant::Secondary))
                .child(Button::new("pv-o", "Outline").variant(ButtonVariant::Outline))
                .child(Button::new("pv-d", "Danger").variant(ButtonVariant::Danger))
                .child(Tag::new("online", Tone::Success).solid(true))
                .child(Tag::new("warn", Tone::Warning)),
        )
        // The cards row is dropped on narrow screens: the pinned strip must not eat the page.
        .children((!narrow).then(|| {
            row()
                .child(card("c", CardVariant::Outlined, "Card"))
                .child(card("e", CardVariant::Elevated, "Elevated"))
                .child(
                    div()
                        .w(px(150.0))
                        .child(Panel::new().title("Panel").child(Muted::new("body"))),
                )
        }))
}
