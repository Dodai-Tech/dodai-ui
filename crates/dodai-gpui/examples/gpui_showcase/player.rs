//! Per-section animation player: replay, loop, stop, and the chips that drive them.

use super::*;
use dodai_core::Theme;
use dodai_gpui::{hsla, Muted, Slider};
use dodai_gpui::{EntryKind, EntrySet};
use gpui::SharedString;
use gpui::{div, px, Context};
use std::time::Duration;

use catalog::{caps, section_id};

/// Per-section animation player state.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum Player {
    /// Animate when the section appears or is replayed.
    #[default]
    Idle,
    /// Replay repeatedly.
    Looping,
    /// Show the final state, no entry animation.
    Stopped,
}

impl Showcase {
    /// Entry set applied to a section's components (`None` = app default).
    pub(crate) fn scope_of(&self, title: &'static str, cx: &gpui::App) -> EntrySet {
        let cp = caps(title);
        if !(cp.entry || cp.fill || cp.stagger) {
            return EntrySet::NONE;
        }
        if self.sec_mode.get(title) == Some(&Player::Stopped) {
            return EntrySet::NONE;
        }
        self.sec_kind
            .get(title)
            .copied()
            .unwrap_or(dodai_gpui::motion::current(cx).entry)
    }

    /// Replay one section only (re-keys just its subtree).
    pub(crate) fn bump(&mut self, title: &'static str) {
        *self.sec_replay.entry(title).or_insert(0) += 1;
    }

    /// Total time of one run of the section: longest of the entry effects and
    /// the value-in (fill) animation, plus the stagger across its items.
    pub(crate) fn run_len(&self, title: &'static str, cx: &gpui::App) -> Duration {
        let m = dodai_gpui::motion::current(cx);
        let set = self.sec_kind.get(title).copied().unwrap_or(m.entry);
        let entry = if set.is_empty() {
            Duration::ZERO
        } else {
            set.duration(m.duration)
        };
        let fill = match set.grow_ms() {
            0 if set.is_empty() => Duration::ZERO,
            0 => m.duration,
            ms => Duration::from_millis(ms as u64),
        };
        entry.max(fill) + Duration::from_millis(set.stagger_ms() as u64 * 8)
    }

    /// Replay and mark the section as playing until the run should be over.
    pub(crate) fn replay(&mut self, title: &'static str, cx: &mut Context<Self>) {
        self.bump(title);
        let tok = self.sec_replay.get(title).copied().unwrap_or(0);
        self.sec_playing.insert(title, tok);
        let run = self.run_len(title, cx);
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(run).await;
            let _ = this.update(cx, |s, cx| {
                if s.sec_playing.get(title) == Some(&tok) {
                    s.sec_playing.remove(title);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    pub(crate) fn set_mode(&mut self, title: &'static str, mode: Player, cx: &mut Context<Self>) {
        self.sec_mode.insert(title, mode);
        self.loop_gen += 1;
        self.replay(title, cx);
        if mode == Player::Stopped {
            // Stop shows its final state, then dismisses itself after two seconds.
            let gen = self.loop_gen;
            cx.spawn(async move |this, cx| {
                cx.background_executor().timer(Duration::from_secs(2)).await;
                let _ = this.update(cx, |s, cx| {
                    if s.sec_mode.get(title) == Some(&Player::Stopped) && s.loop_gen == gen {
                        s.sec_mode.remove(title);
                        cx.notify();
                    }
                });
            })
            .detach();
        }
        if mode == Player::Looping {
            let gen = self.loop_gen;
            cx.spawn(async move |this, cx| loop {
                let wait = this
                    .update(cx, |s, cx| {
                        s.run_len(title, cx) + Duration::from_millis(600)
                    })
                    .unwrap_or_default();
                cx.background_executor().timer(wait).await;
                let alive = this
                    .update(cx, |s, cx| {
                        let ok =
                            s.sec_mode.get(title) == Some(&Player::Looping) && s.loop_gen == gen;
                        if ok {
                            s.bump(title);
                            cx.notify();
                        }
                        ok
                    })
                    .unwrap_or(false);
                if !alive {
                    break;
                }
            })
            .detach();
        }
        cx.notify();
    }

    /// Per-section animation chips (hover a chip for its duration popover).
    pub(crate) fn chips(
        &self,
        title: &'static str,
        t: &Theme,
        me: &gpui::Entity<Showcase>,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let tid = section_id(title);
        let base = dodai_gpui::motion::current(cx).entry;
        let kind = Some(self.sec_kind.get(title).copied().unwrap_or(base));
        let chip = |id: (&'static str, u32), label: SharedString, on: bool| {
            div()
                .id(id)
                .px(px(6.0))
                .py(px(1.0))
                .rounded(px(4.0))
                .text_size(px(10.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .text_color(hsla(if on {
                    t.palette.on_primary
                } else {
                    t.palette.text_muted
                }))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(label)
        };
        let mut chips = div()
            .flex()
            .flex_none()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap(px(4.0))
            .child(
                div()
                    .text_size(px(10.0))
                    .text_color(hsla(t.palette.text_muted))
                    .child("Animations"),
            );
        let cp = caps(title);
        if !(cp.entry || cp.fill || cp.stagger) {
            return div().into_any_element();
        }
        for (ki, k) in EntryKind::ALL
            .iter()
            .copied()
            .enumerate()
            .skip(1)
            .filter(|_| cp.entry)
        {
            let on = kind.is_some_and(|s| s.contains(k));
            let ms = kind.map_or(0, |s| s.ms_of(k));
            let open = on
                && (self.pill_hover == Some((title, k as u8))
                    || self.pop_hover == Some((title, k as u8)));
            let me_c = me.clone();
            let me_p = me.clone();
            let uid = tid.wrapping_mul(8).wrapping_add(ki as u32);
            let text: SharedString = if on && ms > 0 {
                format!("{} · {ms}ms", k.label()).into()
            } else {
                k.label().into()
            };
            let mut wrap = div()
                .id(("pw", uid))
                .relative()
                .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                    if *h {
                        s.pill_hover = Some((title, k as u8));
                    } else if s.pill_hover == Some((title, k as u8)) {
                        s.pill_hover = None;
                    }
                    cx.notify();
                }))
                .child(chip(("sec-k", uid), text, on).on_click(move |_, _, cx| {
                    me_c.update(cx, |s, cx| {
                        let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                        s.sec_kind.insert(title, cur.toggled(k));
                        s.replay(title, cx);
                        cx.notify();
                    })
                }));
            if open {
                let popup = div()
                    .id(("pp", uid))
                    .absolute()
                    .top(px(17.0))
                    .right_0()
                    .w(px(230.0))
                    .p(px(t.spacing.md))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(hsla(t.palette.border_strong))
                    .bg(hsla(t.palette.surface_raised))
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                        if *h {
                            s.pop_hover = Some((title, k as u8));
                        } else if s.pop_hover == Some((title, k as u8)) {
                            s.pop_hover = None;
                        }
                        cx.notify();
                    }))
                    .child(Muted::new(format!("{} duration", k.label())))
                    .child(
                        Slider::new(("pd", uid), ms as f32)
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
                                me_p.update(cx, |s, cx| {
                                    let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                                    s.sec_kind.insert(title, cur.with_ms(k, v as u16));
                                    s.replay(title, cx);
                                    cx.notify();
                                })
                            }),
                    );
                wrap = wrap.child(gpui::deferred(popup).with_priority(60));
            }
            chips = chips.child(wrap);
        }
        // Fill (value-in duration) and Stagger (delay between components).
        for (tag, name, help, slot, vmax, vstep) in [
            (
                100u8,
                "Fill",
                "Fill duration (charts, progress, sliders)",
                0u32,
                10000.0,
                100.0,
            ),
            (
                101u8,
                "Stagger",
                "Delay between components in this section",
                6u32,
                1000.0,
                25.0,
            ),
        ]
        .into_iter()
        .filter(|(tag, ..)| if *tag == 100 { cp.fill } else { cp.stagger })
        {
            let gms = kind.map_or(0, |s| {
                if slot == 0 {
                    s.grow_ms()
                } else {
                    s.stagger_ms()
                }
            });
            let key = (title, tag);
            let open = self.pill_hover == Some(key) || self.pop_hover == Some(key);
            let uid = tid.wrapping_mul(8).wrapping_add(slot);
            let me_p = me.clone();
            let me_c = me.clone();
            let text: SharedString = if gms > 0 {
                format!("{name} · {gms}ms").into()
            } else {
                name.into()
            };
            let mut wrap = div()
                .id(("pw", uid))
                .relative()
                .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                    if *h {
                        s.pill_hover = Some(key);
                    } else if s.pill_hover == Some(key) {
                        s.pill_hover = None;
                    }
                    cx.notify();
                }))
                .child(
                    chip(("sec-f", uid), text, gms > 0).on_click(move |_, _, cx| {
                        // Toggle: active → off (default), off → sensible default.
                        me_c.update(cx, |s, cx| {
                            let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                            let v = if gms > 0 {
                                0
                            } else if slot == 0 {
                                800
                            } else {
                                100
                            };
                            let nw = if slot == 0 {
                                cur.with_grow_ms(v)
                            } else {
                                cur.with_stagger_ms(v)
                            };
                            s.sec_kind.insert(title, nw);
                            s.replay(title, cx);
                            cx.notify();
                        })
                    }),
                );
            if open {
                let popup = div()
                    .id(("pp", uid))
                    .absolute()
                    .top(px(17.0))
                    .right_0()
                    .w(px(230.0))
                    .p(px(t.spacing.md))
                    .rounded(px(6.0))
                    .border_1()
                    .border_color(hsla(t.palette.border_strong))
                    .bg(hsla(t.palette.surface_raised))
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .on_hover(cx.listener(move |s, h: &bool, _, cx| {
                        if *h {
                            s.pop_hover = Some(key);
                        } else if s.pop_hover == Some(key) {
                            s.pop_hover = None;
                        }
                        cx.notify();
                    }))
                    .child(Muted::new(help))
                    .child(
                        Slider::new(("pd", uid), gms as f32)
                            .range(0.0, vmax)
                            .step(vstep)
                            .show_value(true)
                            .format(|v| {
                                if v < 1.0 {
                                    "default".to_string()
                                } else {
                                    format!("{v:.0} ms")
                                }
                            })
                            .on_change(move |v, _, cx| {
                                me_p.update(cx, |s, cx| {
                                    let cur = s.sec_kind.get(title).copied().unwrap_or(base);
                                    let nw = if slot == 0 {
                                        cur.with_grow_ms(v as u16)
                                    } else {
                                        cur.with_stagger_ms(v as u16)
                                    };
                                    s.sec_kind.insert(title, nw);
                                    s.replay(title, cx);
                                    cx.notify();
                                })
                            }),
                    );
                wrap = wrap.child(gpui::deferred(popup).with_priority(60));
            }
            chips = chips.child(wrap);
        }
        // Player: play once / loop / stop.
        let mode = self.sec_mode.get(title).copied().unwrap_or_default();
        let btn = |id: u32, icon: dodai_gpui::IconName, on: bool| {
            let fg = hsla(if on {
                t.palette.on_primary
            } else {
                t.palette.text_muted
            });
            div()
                .id(("pl", tid.wrapping_mul(4).wrapping_add(id)))
                .px(px(5.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .cursor_pointer()
                .border_1()
                .border_color(hsla(t.palette.border))
                .when(on, |d| d.bg(hsla(t.palette.primary)))
                .child(dodai_gpui::Icon::new(icon).size(11.0).color(fg))
        };
        let (m1, m2, m3) = (me.clone(), me.clone(), me.clone());
        chips = chips
            .child(div().w(px(8.0)))
            .child(
                btn(
                    1,
                    dodai_gpui::IconName::Play,
                    mode == Player::Idle && self.sec_playing.contains_key(title),
                )
                .on_click(move |_, _, cx| {
                    m1.update(cx, |s, cx| s.set_mode(title, Player::Idle, cx))
                }),
            )
            .child(
                btn(2, dodai_gpui::IconName::Repeat, mode == Player::Looping).on_click(
                    move |_, _, cx| m2.update(cx, |s, cx| s.set_mode(title, Player::Looping, cx)),
                ),
            )
            .child(
                btn(3, dodai_gpui::IconName::Stop, mode == Player::Stopped).on_click(
                    move |_, _, cx| m3.update(cx, |s, cx| s.set_mode(title, Player::Stopped, cx)),
                ),
            );
        chips.into_any_element()
    }
}
