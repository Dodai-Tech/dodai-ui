//! The "Actions" category sections.

use super::super::*;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_actions(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
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
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let cut = |b: Button| b;
        let buttons = row()
            .child(cut(Button::new("b-primary", "Primary").on_click(
                cx.listener(|s, _, _, cx| {
                    s.clicks += 1;
                    cx.notify();
                }),
            )))
            .child(cut(
                Button::new("b-secondary", "Secondary").variant(ButtonVariant::Secondary)
            ))
            .child(cut(Button::new("b-elev", "Elevated").elevated(true)))
            .child(cut(
                Button::new("b-outline", "Outline").variant(ButtonVariant::Outline)
            ))
            .child(cut(
                Button::new("b-outline2", "Outline 2").variant(ButtonVariant::OutlineSecondary)
            ))
            .child(cut(
                Button::new("b-scrim", "Scrim").variant(ButtonVariant::Scrim)
            ))
            .children(t.look().cut_corners.then(|| {
                // Phantom Liberty: cropped left or right, with an edge tick.
                row()
                    .child(
                        Button::new("b-scrim-l", "Buy now")
                            .variant(ButtonVariant::Scrim)
                            .crop(dodai_gpui::Crop::Left),
                    )
                    .child(
                        Button::new("b-scrim-r", "Watch trailer")
                            .variant(ButtonVariant::Scrim)
                            .crop(dodai_gpui::Crop::Right),
                    )
                    .child(
                        Button::new("b-crop-both", "Both cuts")
                            .variant(ButtonVariant::Outline)
                            .crop(dodai_gpui::Crop::Left)
                            .corners(dodai_core::Corners::DIAGONAL),
                    )
                    .child(
                        Button::new("b-crop-both-p", "Both cuts")
                            .variant(ButtonVariant::Primary)
                            .crop(dodai_gpui::Crop::Right)
                            .corners(dodai_core::Corners::ANTI_DIAGONAL),
                    )
                    .children(
                        [
                            ("Primary", ButtonVariant::Primary),
                            ("Secondary", ButtonVariant::Secondary),
                            ("Outline", ButtonVariant::Outline),
                            ("Danger", ButtonVariant::Danger),
                        ]
                        .into_iter()
                        .enumerate()
                        .map(|(i, (l, v))| {
                            Button::new(SharedString::from(format!("b-crop-{i}")), l)
                                .variant(v)
                                .crop(if i % 2 == 0 {
                                    dodai_gpui::Crop::Left
                                } else {
                                    dodai_gpui::Crop::Right
                                })
                        }),
                    )
                    .child(
                        IconButton::new("ib-crop-l", "✎")
                            .icon(IconName::Edit)
                            .variant(ButtonVariant::Outline)
                            .crop(dodai_gpui::Crop::Left),
                    )
                    .child(
                        IconButton::new("ib-crop-r", "♥")
                            .icon(IconName::Heart)
                            .variant(ButtonVariant::Primary)
                            .crop(dodai_gpui::Crop::Right),
                    )
                    .child(Fab::new("fab-crop", "+").crop(dodai_gpui::Crop::Left))
                    .child(
                        Fab::new("fab-crop-x", "+")
                            .extended("Create")
                            .crop(dodai_gpui::Crop::Right),
                    )
            }))
            .child(cut(
                Button::new("b-danger", "Danger").variant(ButtonVariant::Danger)
            ))
            .child(cut(
                Button::new("b-ghost", "Ghost").variant(ButtonVariant::Ghost)
            ))
            .child(cut(Button::new("b-dis", "Disabled").disabled(true)))
            .child(cut(Button::new("b-shim", "Loading…").shimmer(true)))
            .child(cut(Button::new("b-shim2", "Outline…")
                .variant(ButtonVariant::Outline)
                .shimmer(true)))
            .child(Muted::new(format!("primary clicked {}×", self.clicks)));
        let w_snack = Snackbar::new("snack", "Message archived").action("Undo", |_, _| {});
        let w_toolbar = Toolbar::new()
            .child(IconButton::new("tb1", "✎").icon(IconName::Edit))
            .child(IconButton::new("tb2", "⎘").icon(IconName::Copy));
        let w_toolbar_float = Toolbar::new()
            .floating(true)
            .child(IconButton::new("tbf1", "✎").icon(IconName::Edit))
            .child(IconButton::new("tbf2", "⎘").icon(IconName::Copy))
            .child(IconButton::new("tbf3", "♥").icon(IconName::Heart));
        let w_toolbar_vibrant = Toolbar::new()
            .floating(true)
            .vibrant(true)
            .child(IconButton::new("tbv1", "✎").icon(IconName::Edit))
            .child(IconButton::new("tbv2", "⎘").icon(IconName::Copy));
        let mut sizes = row().items_center();
        let mut sizes_square = row().items_center();
        let mut icon_sizes = row().items_center();
        for z in dodai_core::ControlSize::ALL {
            sizes = sizes.child(
                Button::new(SharedString::from(format!("sz-{}", z.name())), z.name()).size(z),
            );
            sizes_square = sizes_square.child(
                Button::new(SharedString::from(format!("szq-{}", z.name())), z.name())
                    .size(z)
                    .square(true)
                    .variant(ButtonVariant::Secondary),
            );
            icon_sizes = icon_sizes.child(
                IconButton::new(SharedString::from(format!("szi-{}", z.name())), "✎")
                    .icon(IconName::Edit)
                    .size(z)
                    .variant(ButtonVariant::Secondary),
            );
        }
        out.push((
            "Actions",
            "Buttons · IconButton · FAB",
            section!("Buttons")
                .child(buttons)
                .child(sizes)
                .child(sizes_square)
                .child(icon_sizes)
                .into_any_element(),
            "Buttons",
        ));
        let groups = div()
            .flex()
            .flex_col()
            .gap(gap)
            .child(Muted::new("Standard"))
            .child(
                ButtonGroup::new("bg-std")
                    .labels(["Day", "Week", "Month"])
                    .selected(Some(self.group_sel)),
            )
            .child(Muted::new("Connected (the selected item is a pill)"))
            .child(
                ButtonGroup::new("bg-conn")
                    .connected(true)
                    .labels(["Day", "Week", "Month", "Year"])
                    .selected(Some(self.group_sel))
                    .on_select({
                        let m = me.clone();
                        move |i, _, cx| {
                            m.update(cx, |s, cx| {
                                s.group_sel = i;
                                cx.notify()
                            })
                        }
                    }),
            )
            .child(Muted::new("Split button"))
            .child(
                row()
                    .child(SplitButton::new("sb-filled", "Save"))
                    .child(SplitButton::new("sb-tonal", "Save").variant(ButtonVariant::Secondary))
                    .child(
                        SplitButton::new("sb-out", "Save")
                            .variant(ButtonVariant::Outline)
                            .open(true),
                    )
                    .child(SplitButton::new("sb-crop-l", "Save").crop(dodai_gpui::Crop::Left))
                    .child(SplitButton::new("sb-crop-r", "Save").crop(dodai_gpui::Crop::Right)),
            );
        out.push((
            "Actions",
            "Button groups · Split button",
            section!("Button groups · Split button")
                .child(groups)
                .into_any_element(),
            "Button groups · Split button",
        ));
        out.push((
            "Actions",
            "Toolbar · Snackbar",
            section!("Toolbar · Snackbar")
                .child(w_toolbar)
                .child(row().child(w_toolbar_float).child(w_toolbar_vibrant))
                .child(w_snack)
                .into_any_element(),
            "Toolbar · Snackbar",
        ));
        out
    }
}
