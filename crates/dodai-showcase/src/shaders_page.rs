//! The Shaders category: gallery cards per family, presets and the playground.

use super::*;

/// One showcase section: `(category, title, element, key)`.
/// Builds a few related sections.
pub(crate) type Group = fn(&mut Showcase, &mut gpui::Context<Showcase>) -> Vec<Section>;

pub(crate) type Section = (&'static str, &'static str, gpui::AnyElement, &'static str);

impl Showcase {
    /// Append every Shaders section to `sections`.
    pub(crate) fn shader_sections(
        &mut self,
        me: &Entity<Self>,
        sections: &mut Vec<Section>,
        cx: &mut Context<Self>,
    ) {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let me = me.clone();
        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
            };
        }
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        {
            use dodai_gpui::dodai_shaders::{catalog, Kind};
            use dodai_gpui::{PaletteSource, ShaderView};
            let pal_src = if self.sh_theme {
                PaletteSource::Theme
            } else {
                PaletteSource::Family
            };
            let card = |id: String,
                        shader: &'static dodai_gpui::dodai_shaders::ShaderInfo,
                        fx: bool,
                        overlay: bool| {
                let view = if fx {
                    ShaderView::fx_over_default(id.clone(), shader.id)
                } else {
                    ShaderView::new(id.clone(), shader.id)
                };
                col()
                    .flex_1()
                    .min_w(px(220.0))
                    .max_w(px(340.0))
                    .child({
                        let view = view
                            .palette(pal_src)
                            .pixel(2.0)
                            .rounded(t.shape.radius)
                            .transparent(overlay);
                        // An overlay has no picture of its own: show it on a dark tile.
                        div()
                            .w_full()
                            .flex_none()
                            .rounded(px(t.shape.radius))
                            .overflow_hidden()
                            .when(overlay, |d| d.bg(gpui::rgb(0x0b0b12)))
                            .child(view)
                    })
                    .child(
                        row()
                            .child(Mono::new(shader.name))
                            .children(
                                dodai_gpui::dodai_shaders::groups::is_featured(shader)
                                    .then(|| Tag::new("featured", Tone::Success)),
                            ),
                    )
                    .child(Muted::new(shader.desc))
                    .into_any_element()
            };
            // Playground: pick a generator, stack effects, tune.
            let me_p = me.clone();
            let gens: Vec<_> = catalog()
                .iter()
                .filter(|s| matches!(s.kind, Kind::Generator))
                .collect();
            let fxs: Vec<_> = catalog()
                .iter()
                .filter(|s| matches!(s.kind, Kind::PostFx))
                .collect();
            let opt_label = |s: &dodai_gpui::dodai_shaders::ShaderInfo| {
                format!(
                    "{} · {}",
                    dodai_gpui::dodai_shaders::groups::group(s).unwrap_or_else(|| s.family.name()),
                    s.name
                )
            };
            let gen_ids: Vec<&'static str> = gens.iter().map(|g| g.id).collect();
            let gen_sel = gen_ids.iter().position(|i| *i == self.sh_gen);
            let m_g = me_p.clone();
            let gen_row = row()
                .child(Tag::new("SOURCE", Tone::Info).solid(true))
                .child(
                    div().flex_1().min_w(px(200.0)).max_w(px(380.0)).child(
                        dodai_gpui::Select::new("sh-gen")
                            .options(gens.iter().map(|g| opt_label(g)))
                            .selected(gen_sel)
                            .on_select(move |i, _, cx| {
                                let id = gen_ids[i];
                                m_g.update(cx, |s, cx| {
                                    s.sh_gen = id;
                                    cx.notify();
                                })
                            }),
                    ),
                )
                .child(Muted::new(format!(
                    "{} sources: draw the picture",
                    gens.len()
                )));
            let fx_ids: Vec<&'static str> = fxs.iter().map(|f| f.id).collect();
            let m_f = me_p.clone();
            let mut fx_row = row()
                .child(Tag::new("EFFECT", Tone::Warning).solid(true))
                .child(
                    div().flex_1().min_w(px(200.0)).max_w(px(380.0)).child(
                        dodai_gpui::Select::new("sh-fx")
                            .placeholder("Add an effect…")
                            .options(fxs.iter().map(|f| opt_label(f)))
                            .selected(None)
                            .on_select(move |i, _, cx| {
                                let id = fx_ids[i];
                                m_f.update(cx, |s, cx| {
                                    if !s.sh_fx.contains(&id) {
                                        s.sh_fx.push(id);
                                    }
                                    cx.notify();
                                })
                            }),
                    ),
                )
                .child(Muted::new(format!(
                    "{} effects: filter the picture, stacked in order",
                    fxs.len()
                )));
            // The active chain: click a chip to remove that effect.
            let mut chain_row = row().child(Muted::new("Chain"));
            chain_row = chain_row.child(Tag::new(
                gens.iter()
                    .find(|g| g.id == self.sh_gen)
                    .map_or("?", |g| g.name),
                Tone::Info,
            ));
            for (ci, id) in self.sh_fx.iter().enumerate() {
                let (id, m) = (*id, me_p.clone());
                let name = dodai_gpui::dodai_shaders::by_id(id).map_or(id, |s| s.name);
                chain_row = chain_row.child(Muted::new("→")).child(
                    Button::new(("shx", ci), format!("{name} ✕"))
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sh_fx.retain(|x| *x != id);
                                cx.notify();
                            })
                        }),
                );
            }
            if !self.sh_fx.is_empty() {
                let m = me_p.clone();
                chain_row = chain_row.child(
                    Button::new("shx-clear", "Clear effects")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.sh_fx.clear();
                                cx.notify();
                            })
                        }),
                );
            }
            let _ = &mut fx_row;
            let mut big = ShaderView::new("sh-play", self.sh_gen)
                .palette(pal_src)
                .pixel(self.sh_px)
                .intensity(self.sh_int)
                .rounded(t.shape.radius);
            for f in &self.sh_fx {
                big = big.fx(f);
            }
            let (m1, m2, m3, m4, m5) = (me.clone(), me.clone(), me.clone(), me.clone(), me.clone());
            let tune = row()
                .child(Label::new(format!("Intensity {:.0}%", self.sh_int * 100.0)))
                .child(
                    Button::new("shi-", "−")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m1.update(cx, |s, cx| {
                                s.sh_int = (s.sh_int - 0.1).max(0.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new("shi+", "+")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m2.update(cx, |s, cx| {
                                s.sh_int = (s.sh_int + 0.1).min(1.0);
                                cx.notify();
                            })
                        }),
                )
                .child(Label::new(format!("Pixel {:.0}px", self.sh_px)))
                .child(
                    Button::new("shp-", "−")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m3.update(cx, |s, cx| {
                                s.sh_px = (s.sh_px - 1.0).max(1.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new("shp+", "+")
                        .variant(ButtonVariant::Secondary)
                        .on_click(move |_, _, cx| {
                            m4.update(cx, |s, cx| {
                                s.sh_px = (s.sh_px + 1.0).min(10.0);
                                cx.notify();
                            })
                        }),
                )
                .child(
                    Button::new(
                        "sh-theme",
                        if self.sh_theme {
                            "Colours: design system"
                        } else {
                            "Colours: shader family"
                        },
                    )
                    .variant(ButtonVariant::Outline)
                    .on_click(move |_, _, cx| {
                        m5.update(cx, |s, cx| {
                            s.sh_theme = !s.sh_theme;
                            cx.notify();
                        })
                    }),
                );
            sections.push((
                "Shaders",
                "Playground",
                section!("Shader playground")
                    .child(Muted::new(format!(
                        "Shaders written in Rust (`dodai-shaders`), rendered on: {}. Pick a source, stack effects.",
                        dodai_gpui::shader_backend()
                    )))
                    .child(gen_row)
                    .child(fx_row)
                    .child(chain_row)
                    .child(tune)
                    .child(div().w_full().max_w(px(960.0)).child(big))
                    .into_any_element(),
                "Shader playground",
            ));
            // By what they look like: every source group, then every effect group.
            use dodai_gpui::dodai_shaders::groups::{in_group, EFFECT_GROUPS, SOURCE_GROUPS};
            for (names, fx) in [(&SOURCE_GROUPS[..], false), (&EFFECT_GROUPS[..], true)] {
                for name in names {
                    let mut grid = row().items_start();
                    for s in in_group(name) {
                        grid = grid.child(card(
                            format!("shc-{}", s.id),
                            s,
                            fx,
                            *name == SOURCE_GROUPS[0],
                        ));
                    }
                    sections.push((
                        "Shaders",
                        *name,
                        section!(*name).child(grid).into_any_element(),
                        *name,
                    ));
                }
            }
            // Ready-made chains.
            for (title, list) in [
                (
                    "Retro terminals presets",
                    dodai_gpui::dodai_shaders::presets::RETRO,
                ),
                (
                    "Ghostty presets",
                    dodai_gpui::dodai_shaders::presets::GHOSTTY,
                ),
                (
                    "Blade Runner presets",
                    dodai_gpui::dodai_shaders::presets::BLADE,
                ),
                ("Ghibli presets", dodai_gpui::dodai_shaders::presets::GHIBLI),
                (
                    "DMT Hyperspace presets",
                    dodai_gpui::dodai_shaders::presets::DMT,
                ),
                (
                    "Pixel art presets",
                    dodai_gpui::dodai_shaders::presets::PIXEL,
                ),
            ] {
                let cats: &[&'static str] = &["Shaders"];
                for cat in cats {
                    let mut r = row();
                    for p in list {
                        r = r.child(
                            col()
                                .flex_1()
                                .min_w(px(220.0))
                                .child(
                                    ShaderView::preset(format!("shp-{cat}-{}", p.id), p)
                                        .pixel(2.0)
                                        .rounded(t.shape.radius),
                                )
                                .child(Mono::new(p.name))
                                .child(Muted::new(p.desc)),
                        );
                    }
                    sections.push((
                        *cat,
                        title,
                        section!(title).child(r).into_any_element(),
                        title,
                    ));
                }
            }
        }
    }
}
