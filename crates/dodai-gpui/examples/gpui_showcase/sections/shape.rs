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
        let scan = scanlines_enabled(cx);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let cut = |b: Button| b;
        let corner_picker = {
            use dodai_core::Corners as C;
            let me2 = me.clone();
            let cur = self.btn_corners;
            let chip = |id: &'static str,
                        label: &'static str,
                        on: bool,
                        f: Box<dyn Fn(Option<C>) -> Option<C>>| {
                let m = me2.clone();
                Button::new(id, label)
                    .variant(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.btn_corners = f(s.btn_corners);
                            cx.notify()
                        })
                    })
            };
            let base = cur.unwrap_or(C::BOTTOM_RIGHT);
            let tog = |i: usize| -> Box<dyn Fn(Option<C>) -> Option<C>> {
                Box::new(move |c| {
                    let b = c.unwrap_or(C::BOTTOM_RIGHT);
                    Some(b.set(i, !b.get(i)))
                })
            };
            let preset =
                |c: Option<C>| -> Box<dyn Fn(Option<C>) -> Option<C>> { Box::new(move |_| c) };
            row()
                .child(Label::new("Cut corner"))
                .child(chip("cc-ul", "UL", cur.is_some() && base.get(0), tog(0)))
                .child(chip("cc-ur", "UR", cur.is_some() && base.get(1), tog(1)))
                .child(chip("cc-lr", "LR", cur.is_some() && base.get(2), tog(2)))
                .child(chip("cc-ll", "LL", cur.is_some() && base.get(3), tog(3)))
                .child(chip(
                    "cc-diag",
                    "Diagonal",
                    cur == Some(C::DIAGONAL),
                    preset(Some(C::DIAGONAL)),
                ))
                .child(chip(
                    "cc-anti",
                    "Anti-diagonal",
                    cur == Some(C::ANTI_DIAGONAL),
                    preset(Some(C::ANTI_DIAGONAL)),
                ))
                .child(chip(
                    "cc-left",
                    "Left",
                    cur == Some(C::LEFT),
                    preset(Some(C::LEFT)),
                ))
                .child(chip(
                    "cc-right",
                    "Right",
                    cur == Some(C::RIGHT),
                    preset(Some(C::RIGHT)),
                ))
                .child(chip(
                    "cc-all",
                    "All",
                    cur == Some(C::ALL),
                    preset(Some(C::ALL)),
                ))
                .child(chip(
                    "cc-none",
                    "None",
                    cur == Some(C::NONE),
                    preset(Some(C::NONE)),
                ))
                .child(chip("cc-theme", "Theme", cur.is_none(), preset(None)))
                .child(
                    Button::new("cc-apply", "Set as theme")
                        .variant(ButtonVariant::Outline)
                        .on_click(move |_, _, cx| {
                            let c = me2.read(cx).btn_corners;
                            if let Some(c) = c {
                                dodai_gpui::set_corners(cx, c);
                            }
                        }),
                )
        };
        let ov = self.btn_corners;
        let sc = move |b: Button| match ov {
            Some(c) => b.corners(c),
            None => b,
        };
        let sc_tag = move |t: Tag| match ov {
            Some(c) => t.corners(c),
            None => t,
        };
        let sc_card = move |c: Card| match ov {
            Some(k) => c.corners(k),
            None => c,
        };
        let sc_panel = move |p: Panel| match ov {
            Some(k) => p.corners(k),
            None => p,
        };
        let shape_gallery = col()
            .child(
                row()
                    .child(sc(Button::new("sg-p", "Primary")))
                    .child(sc(
                        Button::new("sg-s", "Secondary").variant(ButtonVariant::Secondary)
                    ))
                    .child(sc(
                        Button::new("sg-o", "Outline").variant(ButtonVariant::Outline)
                    ))
                    .child(sc(
                        Button::new("sg-d", "Danger").variant(ButtonVariant::Danger)
                    )),
            )
            .child(
                row()
                    .child(sc_tag(Tag::new("online", Tone::Success).solid(true)))
                    .child(sc_tag(Tag::new("warn", Tone::Warning)))
                    .child(sc_tag(Tag::new("error", Tone::Danger).solid(true))),
            )
            .child(
                row()
                    .items_start()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(220.0))
                            .child(sc_card(Card::new(CardVariant::Outlined).headline("Card"))),
                    )
                    .child(div().flex_1().min_w(px(220.0)).child(sc_card(
                        Card::new(CardVariant::Elevated).headline("Elevated card"),
                    )))
                    .child(div().flex_1().min_w(px(220.0)).child(sc_panel(
                        Panel::new().title("Panel").child(Muted::new("Panel body")),
                    ))),
            );
        let shape_props = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .child(
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
            )
            .child(
                row()
                    .child(Label::new("Scanlines"))
                    .child(Toggle::new("scan", scan).on_change(move |on, _, cx| {
                        set_scanlines(cx, on);
                    }))
                    .child(Muted::new("key: l")),
            );
        let m = me.clone();
        out.push((
                "Shape",
                "Glow · Scanlines",
                section!("Glow · Scanlines")
                    .child(Muted::new(
                        "Theme-wide effects. They apply to every component. Cut corners: pick below, then \"Set as theme\".",
                    ))
                    .child(shape_props)
                    .into_any_element(),
                "Glow · Scanlines",
            ));
        out.push((
                "Shape",
                "Corner shape",
                section!("Corner shape")
                    .child(Muted::new(
                        "Override the corner cut on any component. \"Theme\" follows the design system's own shape.",
                    ))
                    .child(corner_picker)
                    .child(shape_gallery)
                    .into_any_element(),
                "Corner shape",
            ));
        out.push((
                "Shape",
                "Border",
                section!("Border")
                    .child(Muted::new(
                        "BorderStyle { width, opacity }: set per panel with `.border(..)`, or for every panel with `set_section_border`. Sections use THIN by default.",
                    ))
                    .child({
                        use dodai_gpui::BorderStyle as B;
                        let mut r = row();
                        for (name, b) in [
                            ("NONE", B::NONE),
                            ("SUBTLE", B::SUBTLE),
                            ("THIN (default)", B::THIN),
                            ("NORMAL", B::NORMAL),
                            ("BOLD", B::BOLD),
                        ] {
                            r = r.child(
                                div().flex_1().min_w(px(140.0)).child(
                                    Panel::new().border(b).child(
                                        col()
                                            .child(Label::new(name))
                                            .child(Muted::new(format!(
                                                "{:.2}px · {:.0}%",
                                                b.width,
                                                b.opacity * 100.0
                                            ))),
                                    ),
                                ),
                            );
                        }
                        r
                    })
                    .child({
                        use dodai_gpui::{BorderStyle as B, Corner};
                        let mut r = row();
                        for e in Corner::ALL {
                            r = r.child(
                                div().flex_1().min_w(px(110.0)).child(
                                    Panel::new()
                                        .border(B::THIN.accent(Some(e)))
                                        .child(Label::new(e.name())),
                                ),
                            );
                        }
                        r
                    })
                    .child({
                        use dodai_gpui::{BorderStyle as B, Corner};
                        let cur = dodai_gpui::section_border(cx);
                        let mut r = row().child(Label::new("Accent line (all sections)"));
                        let mut opts: Vec<Option<Corner>> = vec![None];
                        opts.extend(Corner::ALL.map(Some));
                        for (i, o) in opts.into_iter().enumerate() {
                            r = r.child(
                                Button::new(
                                    ("bd-acc", i),
                                    o.map_or("None", |e| e.name()),
                                )
                                .variant(if cur.accent == o {
                                    ButtonVariant::Primary
                                } else {
                                    ButtonVariant::Outline
                                })
                                .on_click(move |_, _, cx| {
                                    let b: B = dodai_gpui::section_border(cx);
                                    dodai_gpui::set_section_border(cx, b.accent(o));
                                }),
                            );
                        }
                        r
                    })
                    .child({
                        let b = dodai_gpui::section_border(cx);
                        row()
                            .child(Label::new("Grey / yellow thickness"))
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-width", b.width)
                                        .range(0.0, 4.0)
                                        .step(0.25)
                                        .show_value(true)
                                        .format(|v| format!("grey {v:.2}px"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                dodai_gpui::BorderStyle { width: v.clamp(0.0, 8.0), ..b },
                                            );
                                        }),
                                ),
                            )
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-accent", b.accent_width)
                                        .range(0.5, 6.0)
                                        .step(0.25)
                                        .show_value(true)
                                        .format(|v| format!("yellow {v:.2}px"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                b.accent_width(v),
                                            );
                                        }),
                                ),
                            )
                            .child(
                                div().w(px(180.0)).child(
                                    Slider::new("bd-opacity", b.opacity)
                                        .range(0.0, 1.0)
                                        .step(0.05)
                                        .show_value(true)
                                        .format(|v| format!("{v:.2}"))
                                        .on_change(move |v, _, cx| {
                                            let b = dodai_gpui::section_border(cx);
                                            dodai_gpui::set_section_border(
                                                cx,
                                                dodai_gpui::BorderStyle { opacity: v.clamp(0.0, 1.0), ..b },
                                            );
                                        }),
                                ),
                            )
                    })
                    .into_any_element(),
                "Border",
            ));
        out
    }
}
