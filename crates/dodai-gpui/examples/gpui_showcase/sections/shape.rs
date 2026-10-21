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
        // Global: picking a cut changes every component in every theme.
        let corner_picker = {
            use dodai_core::Corners as C;
            let cur = dodai_gpui::corners(cx);
            let over = dodai_gpui::corners_overridden(cx);
            let chip = |id: &'static str, label: &'static str, on: bool, to: Option<C>| {
                Button::new(id, label)
                    .variant(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| match to {
                        Some(c) => dodai_gpui::set_corners(cx, c),
                        None => dodai_gpui::reset_corners(cx),
                    })
            };
            let tog = |i: usize| Some(cur.set(i, !cur.get(i)));
            row()
                .child(Label::new("Cut corner"))
                .child(chip("cc-ul", "UL", cur.get(0), tog(0)))
                .child(chip("cc-ur", "UR", cur.get(1), tog(1)))
                .child(chip("cc-lr", "LR", cur.get(2), tog(2)))
                .child(chip("cc-ll", "LL", cur.get(3), tog(3)))
                .child(chip(
                    "cc-diag",
                    "Diagonal",
                    cur == C::DIAGONAL,
                    Some(C::DIAGONAL),
                ))
                .child(chip(
                    "cc-anti",
                    "Anti-diagonal",
                    cur == C::ANTI_DIAGONAL,
                    Some(C::ANTI_DIAGONAL),
                ))
                .child(chip("cc-left", "Left", cur == C::LEFT, Some(C::LEFT)))
                .child(chip("cc-right", "Right", cur == C::RIGHT, Some(C::RIGHT)))
                .child(chip("cc-all", "All", cur == C::ALL, Some(C::ALL)))
                .child(chip("cc-none", "None", cur == C::NONE, Some(C::NONE)))
                .child(chip("cc-theme", "Theme default", !over, None))
        };
        let shape_gallery =
            || {
                col()
                    .child(
                        row()
                            .child(Button::new("sg-p", "Primary"))
                            .child(
                                Button::new("sg-s", "Secondary").variant(ButtonVariant::Secondary),
                            )
                            .child(Button::new("sg-o", "Outline").variant(ButtonVariant::Outline))
                            .child(Button::new("sg-d", "Danger").variant(ButtonVariant::Danger)),
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
        // Every design: its own corner radius, border and shadow on real components.
        out.push((
            "Shape",
            "Shape gallery",
            section!("Shape gallery")
                .child(Muted::new(
                    "The design system's own corners, borders and shadows on real components, with the tokens behind them.",
                ))
                .child(shape_tokens_view(&t))
                .child(shape_gallery())
                .into_any_element(),
            "Shape gallery",
        ));
        if t.look().luminous {
            out.push((
                "Shape",
                "Glow · Scanlines",
                section!("Glow · Scanlines")
                    .child(Muted::new(
                        "Theme-wide effects. They apply to every component. Cut corners: pick them under Corner shape.",
                    ))
                    .child(shape_props)
                    .into_any_element(),
                "Glow · Scanlines",
            ));
        }
        if t.look().cut_corners {
            out.push((
                "Shape",
                "Corner shape",
                section!("Corner shape")
                    .child(Muted::new(
                        "The corner cut applies to every component in every theme, immediately. \"Theme default\" restores each design system's own shape.",
                    ))
                    .child(corner_picker)
                    .child(shape_gallery())
                    .into_any_element(),
                "Corner shape",
            ));
        }
        if t.look().cut_corners {
            use dodai_gpui::Crop;
            let cur = dodai_gpui::button_crop(cx);
            let gap_cur = dodai_gpui::button_cut_gap(cx);
            let chip = |id: SharedString, label: String, on: bool, f: Box<dyn Fn(&mut App)>| {
                Button::new(id, label)
                    .variant(if on {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Outline
                    })
                    .on_click(move |_, _, cx| f(cx))
            };
            let mut crop_row = row().child(Label::new("Crop"));
            for (name, c) in [
                ("None", None),
                ("Left", Some(Crop::Left)),
                ("Right", Some(Crop::Right)),
            ] {
                crop_row = crop_row.child(chip(
                    SharedString::from(format!("bc-{name}")),
                    name.to_string(),
                    cur == c,
                    Box::new(move |cx| dodai_gpui::set_button_crop(cx, c)),
                ));
            }
            let mut gap_row = row().child(Label::new("Tick gap"));
            for (name, g) in [
                ("Theme default", None),
                ("0 px", Some(0.0_f32)),
                ("2 px", Some(2.0)),
                ("4 px", Some(4.0)),
                ("8 px", Some(8.0)),
            ] {
                gap_row = gap_row.child(chip(
                    SharedString::from(format!("bg-{name}")),
                    name.to_string(),
                    gap_cur == g,
                    Box::new(move |cx| dodai_gpui::set_button_cut_gap(cx, g)),
                ));
            }
            out.push((
                "Shape",
                "Button crop",
                section!("Button crop")
                    .child(Muted::new(
                        "Crop one end of every button: a cut corner plus a double tick on the other side. Pick the corner under Corner shape too; ready-made cropped buttons are under Actions.",
                    ))
                    .child(crop_row)
                    .child(gap_row)
                    .child(shape_gallery())
                    .into_any_element(),
                "Button crop",
            ));
        }
        if t.look().accent_border {
            out.push((
                "Shape",
                "Border",
                section!("Border")
                    .child(Muted::new(
                        "BorderStyle { width, opacity, accent }: the yellow accent and the grey contour swap sides together; set per panel with `.border(..)`, or for every panel with `set_section_border`. Sections use THIN by default.",
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
                                        .border(B::THIN.accent(e))
                                        .child(Label::new(e.name())),
                                ),
                            );
                        }
                        r
                    })
                    .child({
                        use dodai_gpui::{BorderStyle as B, Corner};
                        let cur = dodai_gpui::section_border(cx);
                        let mut r = row().child(Label::new("Accent position (all sections)"));
                        for (i, e) in Corner::ALL.into_iter().enumerate() {
                            r = r.child(
                                Button::new(("bd-acc", i), e.name())
                                    .variant(if cur.accent == e {
                                        ButtonVariant::Primary
                                    } else {
                                        ButtonVariant::Outline
                                    })
                                    .on_click(move |_, _, cx| {
                                        let b: B = dodai_gpui::section_border(cx);
                                        dodai_gpui::set_section_border(cx, b.accent(e));
                                    }),
                            );
                        }
                        r
                    })
                    .child({
                        let b = dodai_gpui::section_border(cx);
                        let theme_shadow = t.effects.hard_shadow.max(0.5);
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
                                    Slider::new("bd-accent", b.accent_width.unwrap_or(theme_shadow))
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
        }
        out
    }
}
