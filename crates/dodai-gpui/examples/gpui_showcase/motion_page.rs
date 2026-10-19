//! The Loading and Animation categories: indicator demos and the easing / entry catalogue.

use super::*;

impl Showcase {
    pub(crate) fn loading_section(&mut self, cx: &mut Context<Self>) -> Panel {
        let me = cx.entity();
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let (pb, lms) = (self.ld_pb, self.ld_ms);
        let run = self.sec_replay.get("Loading").copied().unwrap_or(0);
        // Duration override (0 = each indicator's own default).
        let mo = move |ms: u64, ease: Easing| {
            Motion::new(
                Duration::from_millis(if lms > 0 { lms as u64 } else { ms }),
                if lms > 0 { Easing::Linear } else { ease },
            )
        };
        let lbtn = |id: u32, icon: dodai_gpui::IconName, on: bool| {
            let fg = hsla(if on {
                t.palette.on_primary
            } else {
                t.palette.text_muted
            });
            div()
                .id(("ldb", id))
                .px(px(5.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(dodai_gpui::Icon::new(icon).size(11.0).color(fg))
        };
        let (l1, l2, l3, l4) = (me.clone(), me.clone(), me.clone(), me.clone());
        let ld_controls = div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .child(
                lbtn(
                    1,
                    dodai_gpui::IconName::Play,
                    pb == dodai_gpui::Playback::Once,
                )
                .on_click(move |_, _, cx| {
                    l1.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Once;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(
                lbtn(
                    2,
                    dodai_gpui::IconName::Repeat,
                    pb == dodai_gpui::Playback::Loop,
                )
                .on_click(move |_, _, cx| {
                    l2.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Loop;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(
                lbtn(
                    3,
                    dodai_gpui::IconName::Stop,
                    pb == dodai_gpui::Playback::Stopped,
                )
                .on_click(move |_, _, cx| {
                    l3.update(cx, |s, cx| {
                        s.ld_pb = dodai_gpui::Playback::Stopped;
                        s.bump("Loading");
                        cx.notify()
                    })
                }),
            )
            .child(div().w(px(10.0)))
            .child(Muted::new("Duration per run"))
            .child(
                div().w(px(220.0)).child(
                    Slider::new("ld-dur", lms as f32)
                        .range(0.0, 10000.0)
                        .step(100.0)
                        .show_value(true)
                        .format(|v| {
                            if v < 1.0 {
                                "default".to_string()
                            } else {
                                format!("{v:.0} ms")
                            }
                        })
                        .on_change(move |v, _, cx| {
                            l4.update(cx, |s, cx| {
                                s.ld_ms = v as u16;
                                s.bump("Loading");
                                cx.notify()
                            })
                        }),
                ),
            );
        let pbk = pb;
        let loading = Panel::new()
            .title("Loading")
            .subheader(ld_controls)
            .child(Label::new("Spinner"))
            .child(
                row()
                    .child(Spinner::new(("ld-sp1", run)).motion(mo(900, Easing::Linear)).playback(pbk))
                    .child(Spinner::new(("ld-sp2", run)).motion(mo(450, Easing::Linear)).playback(pbk))
                    .child(
                        Spinner::new(("ld-sp3", run))
                            .motion(mo(1800, Easing::EaseInOut))
                            .playback(pbk),
                    )
                    .child(Muted::new("900 ms · 450 ms · 1.8 s ease-in-out (unless a duration is set)")),
            )
            .child(Label::new("Indeterminate bar"))
            .child(IndeterminateBar::new(("ld-ib1", run)).motion(mo(1400, Easing::Linear)).playback(pbk))
            .child(IndeterminateBar::new(("ld-ib2", run)).motion(mo(700, Easing::EaseInOut)).playback(pbk))
            .child(Label::new("Determinate progress"))
            .child(ProgressBar::new(0.35))
            .child(ProgressBar::new(0.7).tone(Tone::Success))
            .child(Label::new("Skeleton placeholders"))
            .child(
                row()
                    .items_start()
                    .gap(px(16.0))
                    .child(
                        Skeleton::new(("ld-av", run))
                            .width(48.0)
                            .height(48.0)
                            .circle(true)
                            .motion(mo(1600, Easing::Linear))
                            .playback(pbk),
                    )
                    .child(
                        col()
                            .flex_1()
                            .child(Skeleton::new(("ld-l1", run)).height(14.0).motion(mo(1600, Easing::Linear)).playback(pbk))
                            .child(Skeleton::new(("ld-l2", run)).height(14.0).width(260.0).motion(mo(1600, Easing::Linear)).playback(pbk))
                            .child(Skeleton::new(("ld-l3", run)).height(14.0).width(180.0).motion(mo(1600, Easing::Linear)).playback(pbk)),
                    ),
            )
            .child(Muted::new(
                "Play runs one cycle, Loop repeats, Stop holds a still frame. Loading demos live on this page only (not on All) to keep idle CPU at zero.",
            ));

        // Animation page: one Panel per entry type (curve · live example · text).
        loading
    }

    /// `(sections, titles)` for the Animation category.
    pub(crate) fn animation_sections(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Vec<gpui::AnyElement>, Vec<&'static str>) {
        let me = cx.entity();
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let me = me.clone();
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let anim_defs: [(&'static str, EntryKind, Easing, &'static str); 12] = [
            (
                EntryKind::Fade.label(),
                EntryKind::Fade,
                Easing::EaseOut,
                "Opacity 0 → 1. The default entry style: quiet, never shifts layout. Ease-out starts fast and settles.",
            ),
            (
                EntryKind::Rise.label(),
                EntryKind::Rise,
                Easing::CubicBezier(0.2, 0.8, 0.2, 1.0),
                "Fades in while moving up 14 px. A snappy cubic-bezier(.2,.8,.2,1) gives a quick start and a soft landing.",
            ),
            (
                EntryKind::Drop.label(),
                EntryKind::Drop,
                Easing::EaseInOut,
                "Fades in while dropping down 14 px. Ease-in-out accelerates, then decelerates; good for menus and toasts.",
            ),
            (
                EntryKind::SlideLeft.label(),
                EntryKind::SlideLeft,
                Easing::CubicBezier(0.7, 0.0, 0.3, 1.0),
                "Fades in sliding from the left. A strong ease-in-out (.7,0,.3,1) feels deliberate; fits sidebars and drawers.",
            ),
            (
                EntryKind::SlideRight.label(),
                EntryKind::SlideRight,
                Easing::EaseIn,
                "Fades in sliding from the right. Ease-in starts slowly and finishes fast; use for trailing panels.",
            ),
            (
                "Spring",
                EntryKind::Rise,
                Easing::SPRING,
                "Physics-based easing: a damped spring (stiffness 170, damping 12). Overshoots its target and settles; entry effects clamp at 1.0, value animations (charts, sliders) show the overshoot.",
            ),
            (
                "Spring · stiff",
                EntryKind::Rise,
                Easing::SPRING_STIFF,
                "Stiffness 400, damping 26: snaps into place with a tiny overshoot. Used by the Toggle thumb.",
            ),
            (
                "Spring · wobbly",
                EntryKind::Rise,
                Easing::SPRING_WOBBLY,
                "Stiffness 120, damping 6: loose, lingering wobble. Playful; use sparingly.",
            ),
            (
                "Spring · gentle",
                EntryKind::Drop,
                Easing::SPRING_GENTLE,
                "Stiffness 100, damping 20: heavily damped, no overshoot, soft landing.",
            ),
            (
                "Bounce",
                EntryKind::Drop,
                Easing::Bounce,
                "Ball drop: falls to the target and bounces to rest. Fits dropped-in toasts and badges.",
            ),
            (
                "Elastic",
                EntryKind::SlideLeft,
                Easing::Elastic,
                "Rubber band: shoots past the target and wobbles with a decaying sine.",
            ),
            (
                "Back · anticipate",
                EntryKind::SlideRight,
                Easing::Back(1.7),
                "Back overshoots the target about 10% and eases back; Anticipate (the mirror) winds up first by pulling away before moving.",
            ),
        ];
        let anim_titles: Vec<&'static str> = anim_defs.iter().map(|d| d.0).collect();
        let mut anim_sections = Vec::new();
        for (i, (title, kind, ease, text)) in anim_defs.into_iter().enumerate() {
            let n = self.sec_replay.get(title).copied().unwrap_or(0);
            let me_r = me.clone();
            let ctrl = ease_ctrl(ease);
            let (x1, y1, x2, y2) = ctrl.unwrap_or((0.0, 0.0, 1.0, 1.0));
            let ymax = if ctrl.is_none() { 1.4 } else { 1.0 };
            let (border, accent, muted) = (
                hsla(t.palette.border),
                hsla(t.palette.primary),
                hsla(t.palette.text_muted),
            );
            let make_plot = move |p: f32| {
                gpui::canvas(
                    |_, _, _| (),
                    move |b: Bounds<gpui::Pixels>, _, w, _| {
                        let pad = 10.0;
                        let side = f32::from(b.size.width.min(b.size.height)) - 2.0 * pad;
                        let at = |x: f32, y: f32| {
                            gpui::point(
                                b.origin.x + px(pad + x * side),
                                b.origin.y + px(pad + (1.0 - y / ymax) * side),
                            )
                        };
                        let line =
                            |pts: &[(f32, f32)], width: f32, c: gpui::Hsla, w: &mut Window| {
                                let mut p = gpui::PathBuilder::stroke(px(width));
                                for (k, &(x, y)) in pts.iter().enumerate() {
                                    if k == 0 {
                                        p.move_to(at(x, y));
                                    } else {
                                        p.line_to(at(x, y));
                                    }
                                }
                                if let Ok(path) = p.build() {
                                    w.paint_path(path, c);
                                }
                            };
                        line(
                            &[(0.0, 0.0), (1.0, 0.0), (1.0, ymax), (0.0, ymax), (0.0, 0.0)],
                            1.0,
                            border,
                            w,
                        );
                        line(&[(0.0, 0.0), (1.0, 1.0)], 1.0, border.opacity(0.5), w);
                        if ctrl.is_some() {
                            line(&[(0.0, 0.0), (x1, y1)], 1.0, muted, w);
                            line(&[(1.0, 1.0), (x2, y2)], 1.0, muted, w);
                        } else {
                            line(&[(0.0, 1.0), (1.0, 1.0)], 1.0, muted.opacity(0.5), w);
                        }
                        let pts = |upto: f32| -> Vec<(f32, f32)> {
                            let n = ((48.0 * upto).ceil() as usize).max(1);
                            (0..=n)
                                .map(|k| {
                                    let x = (k as f32 / n as f32) * upto;
                                    (x, ease.apply(x).clamp(0.0, ymax))
                                })
                                .collect()
                        };
                        line(&pts(1.0), 2.0, accent.opacity(0.25), w);
                        line(&pts(p), 3.0, accent, w);
                        // Playhead: x = time, y = eased progress.
                        let y = ease.apply(p).clamp(0.0, ymax);
                        line(&[(p, 0.0), (p, y)], 1.0, accent.opacity(0.5), w);
                        line(&[(0.0, y), (p, y)], 1.0, accent.opacity(0.5), w);
                        w.paint_quad(gpui::fill(
                            Bounds::new(
                                at(p, y) - gpui::point(px(4.5), px(4.5)),
                                gpui::size(px(9.0), px(9.0)),
                            ),
                            gpui::white(),
                        ));
                        for (hx, hy) in [(x1, y1), (x2, y2)].into_iter().filter(|_| ctrl.is_some())
                        {
                            let c = at(hx.clamp(0.0, 1.0), hy.clamp(0.0, 1.0));
                            w.paint_quad(gpui::fill(
                                Bounds::new(
                                    c - gpui::point(px(3.0), px(3.0)),
                                    gpui::size(px(6.0), px(6.0)),
                                ),
                                accent,
                            ));
                        }
                    },
                )
                .size_full()
            };
            let motion = Motion::new(Duration::from_millis(700), ease).entry(kind);
            let demo = div()
                .id(("an-demo", (i as u32) * 1000 + n))
                .w_full()
                .min_w(px(220.0))
                .h(px(110.0))
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(dodai_gpui::entry_animation(
                    div().w(px(170.0)).child(
                        Card::new(CardVariant::Elevated)
                            .child(Label::new(format!("{title} entry"))),
                    ),
                    "an-card",
                    motion,
                ));
            anim_sections.push(
                Panel::new()
                    .title(title)
                    .child(
                        row()
                            .items_start()
                            .gap(px(24.0))
                            .child(
                                div()
                                    .id(("an-plot", (i as u32) * 1000 + n))
                                    .w(px(200.0))
                                    .h(px(200.0))
                                    .flex_none()
                                    .with_animation(
                                        ("an-plot-a", (i as u32) * 1000 + n),
                                        gpui::Animation::new(Duration::from_millis(1400)),
                                        move |el, p| {
                                            // hold at the end for the last 50%
                                            el.child(make_plot((p * 2.0).min(1.0)))
                                        },
                                    ),
                            )
                            .child(col().flex_1().min_w(px(240.0)).child(demo).child(
                                Button::new(("an-replay", i as u32), "Replay").on_click(
                                    move |_, _, cx| {
                                        me_r.update(cx, |s, cx| {
                                            s.bump(title);
                                            cx.notify();
                                        })
                                    },
                                ),
                            ))
                            .child(div().flex_1().min_w(px(220.0)).child(Muted::new(format!(
                                "{text}  {}",
                                if ctrl.is_some() {
                                    format!("cubic-bezier({x1}, {y1}, {x2}, {y2})")
                                } else {
                                    String::new()
                                }
                            )))),
                    )
                    .into_any_element(),
            );
        }
        (anim_sections, anim_titles)
    }
}
