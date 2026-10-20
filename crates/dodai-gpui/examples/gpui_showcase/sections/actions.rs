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
            .child(cut(
                Button::new("b-outline", "Outline").variant(ButtonVariant::Outline)
            ))
            .child(cut(
                Button::new("b-outline2", "Outline 2").variant(ButtonVariant::OutlineSecondary)
            ))
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
        out.push((
            "Actions",
            "Buttons · IconButton · FAB",
            section!("Buttons").child(buttons).into_any_element(),
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
                    ),
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
                .child(w_snack)
                .into_any_element(),
            "Toolbar · Snackbar",
        ));
        out
    }
}
