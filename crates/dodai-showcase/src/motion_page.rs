//! The Loading and Animation categories: indicator demos and the easing / entry catalogue.

use super::*;
use std::sync::Arc;

/// Length of one easing demo loop: the curve and the entry card play in the first half and hold
/// their end for the second (as the other toolkits' Animation page does).
const LOOP: Duration = Duration::from_millis(2800);
/// Seconds a looping mixin rests at its end before it plays again.
const MIXIN_REST: f32 = 0.9;

/// The repeating clock of an easing demo.
fn loop_animation() -> gpui::Animation {
    gpui::Animation::new(LOOP).repeat()
}

/// Progress `0..=1` of an easing demo at `k` (`0..1` of [`LOOP`]): plays, then holds the end.
fn loop_progress(k: f32) -> f32 {
    (k * 2.0).clamp(0.0, 1.0)
}

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
            .child(Label::new("Spinners"))
            .children(spinner_groups(&theme_of(cx)).into_iter().enumerate().map(|(d, (name, set))| {
                let material = name.ends_with("Material 3");
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .child(Muted::new(name))
                    .child({
                        let cell = |spinner: gpui::AnyElement, label: String| {
                            div()
                                .flex()
                                .flex_col()
                                .items_center()
                                .gap(px(6.0))
                                .child(div().size(px(56.0)).flex().items_center().justify_center().child(spinner))
                                .child(Muted::new(label))
                        };
                        let mut r = row().items_end().gap(px(28.0));
                        for (i, st) in set.iter().enumerate() {
                            let sp = Spinner::new(("ld-set", run * 64 + (d * 8 + i) as u32))
                                .style(*st)
                                .size(56.0)
                                .motion(mo(900, Easing::Linear))
                                .playback(pbk);
                            r = r.child(cell(sp.into_any_element(), format!("{st:?}").to_lowercase()));
                        }
                        if material {
                            r = r
                                .child(cell(
                                    LoadingIndicator::new(("ld-li", run)).into_any_element(),
                                    "loading indicator".into(),
                                ))
                                .child(cell(
                                    LoadingIndicator::new(("ld-lic", run)).contained(true).into_any_element(),
                                    "contained".into(),
                                ));
                        }
                        r
                    })
            }))
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
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
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
            // The entry card replays with the curve, forever: no replay button.
            let demo = div()
                .id(("an-demo", i as u32))
                .w_full()
                .min_w(px(220.0))
                .h(px(110.0))
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .id(("an-card", i as u32))
                        .relative()
                        .w(px(170.0))
                        .child(
                            Card::new(CardVariant::Elevated)
                                .child(Label::new(format!("{title} entry"))),
                        )
                        .with_animation(("an-card-a", i as u32), loop_animation(), move |el, k| {
                            let (o, dx, dy) =
                                kind.frame(ease.apply(loop_progress(k)).clamp(0.0, 1.0));
                            el.left(px(dx)).top(px(dy)).opacity(o)
                        }),
                );
            anim_sections.push(
                Panel::new()
                    .title(title)
                    .child(
                        row()
                            .items_start()
                            .gap(px(24.0))
                            .child(
                                div()
                                    .id(("an-plot", i as u32))
                                    .w(px(200.0))
                                    .h(px(200.0))
                                    .flex_none()
                                    .with_animation(
                                        ("an-plot-a", i as u32),
                                        loop_animation(),
                                        move |el, k| el.child(make_plot(loop_progress(k))),
                                    ),
                            )
                            .child(col().flex_1().min_w(px(240.0)).child(demo))
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
        let mut anim_titles = anim_titles;
        anim_sections.push(self.dynamics_physics(cx));
        anim_titles.push("Dynamics · physics models");
        anim_sections.push(self.dynamics_mixins(cx));
        anim_titles.push("Dynamics · one-shot mixins");
        (anim_sections, anim_titles)
    }

    /// Plots of the closed-form physics models plus a live lane for each.
    fn dynamics_physics(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use dodai_gpui::dodai_motion::{rubber_band, Bounce, Decay, Personality, Shake};
        type F = Arc<dyn Fn(f32) -> f32 + Send + Sync>;
        let t = theme_of(cx);
        let (border, accent, muted) = (
            hsla(t.palette.border),
            hsla(t.palette.primary),
            hsla(t.palette.text_muted),
        );
        let lane = |name: &'static str, note: String, f: F, lo: f32, hi: f32| {
            let f2 = f.clone();
            let plot = gpui::canvas(
                |_, _, _| (),
                move |b: Bounds<gpui::Pixels>, _, w, _| {
                    let (bw, bh) = (f32::from(b.size.width), f32::from(b.size.height));
                    let at = |x: f32, y: f32| {
                        gpui::point(
                            b.origin.x + px(4.0 + x * (bw - 8.0)),
                            b.origin.y + px(4.0 + (1.0 - (y - lo) / (hi - lo)) * (bh - 8.0)),
                        )
                    };
                    let stroke = |pts: Vec<(f32, f32)>, c: gpui::Hsla, w: &mut Window| {
                        let mut p = gpui::PathBuilder::stroke(px(2.0));
                        for (k, (x, y)) in pts.into_iter().enumerate() {
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
                    stroke(vec![(0.0, 0.0), (1.0, 0.0)], border, w);
                    stroke(
                        (0..=80)
                            .map(|k| (k as f32 / 80.0, f2(k as f32 / 80.0)))
                            .collect(),
                        accent,
                        w,
                    );
                },
            )
            .size_full();
            let f3 = f.clone();
            let dot = div().size(px(14.0)).rounded_full().bg(accent);
            let live = div()
                .id(("dyn-live", section_id(name)))
                .relative()
                .w(px(220.0))
                .h(px(34.0))
                .child(dot.absolute().top(px(10.0)).with_animation(
                    ("dyn-live-a", section_id(name)),
                    gpui::Animation::new(Duration::from_millis(2600)).repeat(),
                    move |el, p| {
                        let y = (f3(p.min(1.0)) - lo) / (hi - lo);
                        el.left(px(4.0 + y * 200.0))
                    },
                ));
            div()
                .flex()
                .items_center()
                .gap(px(16.0))
                .child(div().w(px(150.0)).h(px(70.0)).flex_none().child(plot))
                .child(live)
                .child(
                    div()
                        .flex_1()
                        .min_w(px(180.0))
                        .child(Label::new(name))
                        .child(Muted::new(note)),
                )
        };
        // The spring each personality gives a design, so the lane marked "this design" is the
        // one its components really use.
        let spring_of = |p: Personality| -> F {
            let s = p.spring();
            Arc::new(move |x| s.at(x * 1.6, 0.0, 0.0, 1.0).0)
        };
        let mine = t.motion.personality;
        let feel = |p: Personality, what: &str| {
            let tag = if p == mine {
                " This design's feel."
            } else {
                ""
            };
            format!("{what}{tag}")
        };
        let decay = Decay::new(0.0, 4.0, 3.5);
        let ball = Bounce::new(100.0, 900.0, 0.6);
        let dur = ball.duration();
        let shake = Shake::new(5.0, 3.5);
        let rows = vec![
            lane(
                "Spring · snappy",
                feel(Personality::Snappy, "No overshoot: the fastest arrival."),
                spring_of(Personality::Snappy),
                0.0,
                1.4,
            ),
            lane(
                "Spring · smooth",
                feel(Personality::Smooth, "A hair of overshoot, calm."),
                spring_of(Personality::Smooth),
                0.0,
                1.4,
            ),
            lane(
                "Spring · playful",
                feel(Personality::Playful, "Clear overshoot and ring-down."),
                spring_of(Personality::Playful),
                0.0,
                1.4,
            ),
            lane(
                "Fling (decay)",
                "Exponential friction: coasts to rest at v0 / friction, then snap to a page."
                    .into(),
                Arc::new(move |x| decay.position(x * 1.6)),
                0.0,
                1.4,
            ),
            lane(
                "Dropped ball",
                "Gravity with restitution: each bounce keeps 60 % of the speed.".into(),
                Arc::new(move |x| 1.0 - ball.at(x * dur) / 100.0),
                0.0,
                1.05,
            ),
            lane(
                "Shake",
                "A decaying sine: invalid input, doorbell wobble.".into(),
                Arc::new(move |x| 0.5 + 0.5 * shake.at(x * 1.5)),
                0.0,
                1.0,
            ),
            lane(
                "Rubber band",
                "Drag past an edge with resistance: output never reaches the limit.".into(),
                Arc::new(|x| rubber_band(x * 6.0, 1.0, 0.55)),
                0.0,
                1.0,
            ),
        ];
        let _ = muted;
        let mut col = div().flex().flex_col().gap(px(t.spacing.md));
        for r in rows {
            col = col.child(r);
        }
        let m = &t.motion;
        let summary = format!(
            "This design: {} springs, values travel {}, {} ms base duration.",
            m.personality.label(),
            format!("{:?}", m.travel).to_lowercase(),
            m.duration.as_millis()
        );
        Panel::new()
            .title("Dynamics · physics models")
            .child(Muted::new(summary))
            .child(col)
            .into_any_element()
    }

    /// One-shot mixins: each card plays its `Clip<Pose>`, rests, and plays again.
    fn dynamics_mixins(&mut self, cx: &mut Context<Self>) -> gpui::AnyElement {
        use dodai_gpui::dodai_motion as dm;
        let t = theme_of(cx);
        let p = t.motion.personality;
        let title: &'static str = "Dynamics · one-shot mixins";
        let reduced = false;
        let defs: Vec<(&'static str, &'static str, dm::Clip<dm::Pose>)> = vec![
            ("Shake", "invalid input", dm::shake(p)),
            ("Nudge", "gentle tap", dm::nudge(p)),
            ("Attention bounce", "look at me", dm::attention_bounce(p)),
            ("Flash on change", "value changed", dm::flash_on_change()),
            ("Pop in", "springy appear", dm::pop_in(p)),
        ];
        let glow = hsla(t.palette.primary);
        let mut row = div().flex().flex_wrap().gap(px(t.spacing.md));
        for (name, note, clip) in defs {
            let clip = if reduced { clip.reduced() } else { clip };
            let secs = clip.duration().max(0.2);
            let period = secs + MIXIN_REST;
            let id = section_id(name);
            row = row.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .w(px(170.0))
                    .child(
                        div()
                            .h(px(90.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .id(("mx-card", id))
                                    .px(px(14.0))
                                    .py(px(10.0))
                                    .rounded(px(t.shape.radius.min(10.0)))
                                    .border_1()
                                    .border_color(hsla(t.palette.border))
                                    .bg(hsla(t.palette.surface_raised))
                                    .child(Label::new(name))
                                    .with_animation(
                                        ("mx-a", id),
                                        gpui::Animation::new(Duration::from_secs_f32(period))
                                            .repeat(),
                                        move |el, k| {
                                            let pose = clip.at((k * period).min(secs));
                                            el.relative()
                                                .left(px(pose.dx))
                                                .top(px(pose.dy))
                                                .opacity(pose.opacity)
                                                .border_color(glow.opacity(pose.glow))
                                        },
                                    ),
                            ),
                    )
                    .child(Muted::new(note)),
            );
        }
        Panel::new()
            .title(title)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.md))
                    .child(Muted::new(format!(
                        "Personality: {} (from the active theme's motion). Each mixin is a Clip<Pose>; the adapter only applies dx, dy, opacity and glow.",
                        p.label()
                    )))
                    .child(row),
            )
            .into_any_element()
    }
}

/// The spinners every design shares, then the current design's own.
fn spinner_groups(t: &dodai_core::Theme) -> Vec<(String, Vec<dodai_core::SpinnerStyle>)> {
    let design = match t.id().split('-').next().unwrap_or_default() {
        "dodai" => "Dodai",
        "cyberpunk" => "Cyberpunk",
        "neobrutalism" | "neobrutal" => "Neo-Brutalism",
        "material" => "Material 3",
        _ => "This design",
    };
    vec![
        (
            "Shared, in every design".into(),
            dodai_core::SpinnerStyle::SHARED.to_vec(),
        ),
        (format!("Only in {design}"), t.look().spinners.to_vec()),
    ]
}

#[cfg(test)]
mod loop_tests {
    use super::loop_progress;

    #[test]
    fn an_easing_demo_plays_in_the_first_half_and_holds_its_end() {
        assert_eq!(loop_progress(0.0), 0.0);
        assert_eq!(loop_progress(0.25), 0.5);
        assert_eq!(loop_progress(0.5), 1.0);
        assert_eq!(loop_progress(0.9), 1.0);
    }
}
