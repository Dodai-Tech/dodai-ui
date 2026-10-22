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
                ("Theme", None),
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
                "Which corners every component cuts, in every design. \"Theme\" restores each design's own.",
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
                        "The corner cut applies to every component in every theme, immediately. Pick \"Theme\" to restore each design's own shape.",
                    ))
                    .child(corner_picker)
                    .child(shape_gallery())
                    .into_any_element(),
                "Corner shape",
            ));
        }
        if t.look().cut_corners {
            use dodai_gpui::Crop;
            let crops = [None, Some(Crop::Left), Some(Crop::Right)];
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
                Segmented::new("bc-crop", ["Off", "Left", "Right"])
                    .selected(crop_sel)
                    .on_select(move |i, _, cx| dodai_gpui::set_button_crop(cx, crops[i]))
                    .into_any_element(),
            );
            let gap_row = setting(
                "Tick gap",
                "Space between the two tick bars, left open in the border.",
                Segmented::new("bc-gap", ["Theme", "0 px", "2 px", "4 px", "8 px"])
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
