//! The "Inputs" category sections.

use super::super::*;
use dodai_gpui::CheckboxVariant;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_inputs(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
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
        let me_sw = me.clone();
        let me_ck = me.clone();
        let me_sl = me.clone();
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let tags = row()
            .child(Tag::new("online", Tone::Success).solid(true))
            .child(Tag::new("warn", Tone::Warning).solid(true))
            .child(Tag::new("error", Tone::Danger).solid(true))
            .child(Tag::new("info", Tone::Info).solid(true))
            .child(Tag::new("online", Tone::Success))
            .child(Tag::new("warn", Tone::Warning))
            .child(Tag::new("error", Tone::Danger))
            .child(Tag::new("info", Tone::Info));
        let controls = row()
            .child(Toggle::new("sw", self.switch).on_change(move |v, _, cx| {
                me_sw.update(cx, |s, cx| {
                    s.switch = v;
                    cx.notify();
                })
            }))
            .child(Label::new("Switch"))
            .child(Toggle::new("sw-icons", true).icons(true))
            .child(Toggle::new("sw-icons-off", false).icons(true))
            .child(Label::new("With icons"))
            .child(Toggle::new("sw-dis", true).disabled(true))
            .child(Checkbox::new("ck", self.check).on_change(move |v, _, cx| {
                me_ck.update(cx, |s, cx| {
                    s.check = v;
                    cx.notify();
                })
            }))
            .child(Label::new("Checkbox"))
            .child(Checkbox::new("ck-dis", false).disabled(true));
        let mut radios = row();
        for (i, name) in ["Compact", "Comfortable", "Spacious"]
            .into_iter()
            .enumerate()
        {
            let m = me.clone();
            radios = radios
                .child(
                    Radio::new(("radio", i), self.radio == i).on_change(move |_, _, cx| {
                        m.update(cx, |s, cx| {
                            s.radio = i;
                            cx.notify();
                        })
                    }),
                )
                .child(Label::new(name));
        }
        let sliders = div()
            .flex()
            .flex_col()
            .gap(gap)
            .child(
                Slider::new("slider", self.slider).on_change(move |v, _, cx| {
                    me_sl.update(cx, |s, cx| {
                        s.slider = v;
                        cx.notify();
                    })
                }),
            )
            .child(Mono::new(format!("value {:.2}", self.slider)))
            .child(ProgressBar::new(self.slider))
            .child(ProgressBar::new(0.7).tone(Tone::Success))
            .child(ProgressBar::new(0.4).tone(Tone::Warning))
            .child(ProgressBar::new(0.15).tone(Tone::Danger));
        let mut dials = row().gap(px(24.0));
        for i in 0..3 {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        Knob::new(("knob", i), self.knob[i])
                            .size(if i == 1 { 96.0 } else { 72.0 })
                            .disabled(i == 2)
                            .on_change(move |v, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.knob[i] = v;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Mono::new(format!("{:.2}", self.knob[i]))),
            );
        }
        for i in 0..3 {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        Slider::new(("vslider", i), self.vslider[i])
                            .direction(Direction::Vertical)
                            .length(120.0)
                            .on_change(move |v, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.vslider[i] = v;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Mono::new(format!("{:.2}", self.vslider[i]))),
            );
        }
        {
            let m = me.clone();
            dials = dials.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(gap)
                    .child(
                        RangeSlider::new("vrange", self.vrange.0, self.vrange.1)
                            .direction(Direction::Vertical)
                            .length(120.0)
                            .step(0.1)
                            .ticks(true)
                            .on_change(move |a, b, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.vrange = (a, b);
                                    cx.notify()
                                })
                            }),
                    )
                    .child(Mono::new(format!(
                        "{:.1}–{:.1}",
                        self.vrange.0, self.vrange.1
                    ))),
            );
        }
        let m3 = me.clone();
        let open = !self.search.read(cx).text().is_empty();
        let m = me.clone();
        let search = SearchView::new("search", self.search.clone())
            .items(["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"])
            .open(open)
            .on_clear({
                let m = me.clone();
                move |w, cx| {
                    m.update(cx, |s, cx| {
                        s.search.update(cx, |i, cx| {
                            i.set_text("", cx);
                            let _ = w;
                        })
                    })
                }
            })
            .on_pick(move |_, _, _| {
                let _ = &m;
            });
        let (m1, m2, m3, m4, m5, m6) = (
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
            me.clone(),
        );
        let w_search = search;
        let w_range =
            RangeSlider::new("range", self.range.0, self.range.1).on_change(move |a, b, _, cx| {
                m3.update(cx, |s, cx| {
                    s.range = (a, b);
                    cx.notify()
                })
            });
        let w_range_txt = Muted::new(format!("range {:.2} – {:.2}", self.range.0, self.range.1));
        let w_tags = TagField::new("tags", self.tags.clone()).on_remove(move |i, _, cx| {
            m4.update(cx, |s, cx| {
                s.tags.remove(i);
                cx.notify()
            })
        });
        let (y, mo, sel) = self.date;
        let pickers = row()
            .child(
                DatePicker::new("date", y, mo)
                    .selected(sel)
                    .on_select({
                        let m = m5.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date.2 = Some(d);
                                cx.notify()
                            })
                        }
                    })
                    .on_month(move |y, mo, _, cx| {
                        m5.update(cx, |s, cx| {
                            s.date.0 = y;
                            s.date.1 = mo;
                            cx.notify()
                        })
                    }),
            )
            .child(TimePicker::new("time", self.time.0, self.time.1).on_change(
                move |h, mi, _, cx| {
                    m6.update(cx, |s, cx| {
                        s.time = (h, mi);
                        cx.notify()
                    })
                },
            ));
        let m_range = me.clone();
        let range_picker = row()
            .child(
                DatePicker::new("date-range", y, mo)
                    .range(self.date_range)
                    .on_select({
                        let m = m_range.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date_range = dodai_gpui::range_pick(s.date_range, d);
                                cx.notify()
                            })
                        }
                    })
                    .on_month({
                        let m = m_range.clone();
                        move |y, mo, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date.0 = y;
                                s.date.1 = mo;
                                cx.notify()
                            })
                        }
                    }),
            )
            .child(Muted::new(match self.date_range.span() {
                Some((a, b)) => format!(
                    "{}-{:02}-{:02} → {}-{:02}-{:02}",
                    a.year, a.month, a.day, b.year, b.month, b.day
                ),
                None => match self.date_range.start {
                    Some(a) => format!("from {}-{:02}-{:02}: pick the end", a.year, a.month, a.day),
                    None => "click a start day, then an end day".to_string(),
                },
            }));
        let (ms1, ms2) = (me.clone(), me.clone());
        let (mb1, mb2) = (me.clone(), me.clone());
        let stepped = col()
            .child(
                Slider::new("bub-up", self.bub.0)
                    .range(0.0, 100.0)
                    .bubble(BubblePlacement::Above)
                    .format(|v| format!("{v:.0}%"))
                    .on_change(move |v, _, cx| {
                        mb1.update(cx, |s, cx| {
                            s.bub.0 = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                Slider::new("bub-down", self.bub.1)
                    .range(0.0, 100.0)
                    .step(10.0)
                    .ticks(true)
                    .bubble(BubblePlacement::Below)
                    .format(|v| format!("{v:.0}%"))
                    .on_change(move |v, _, cx| {
                        mb2.update(cx, |s, cx| {
                            s.bub.1 = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                Slider::new("stepped", self.stepped)
                    .range(0.0, 10.0)
                    .step(1.0)
                    .ticks(true)
                    .show_value(true)
                    .on_change(move |v, _, cx| {
                        ms1.update(cx, |s, cx| {
                            s.stepped = v;
                            cx.notify()
                        })
                    }),
            )
            .child(
                Slider::new("db", self.db)
                    .range(-24.0, 6.0)
                    .step(3.0)
                    .ticks(true)
                    .show_value(true)
                    .format(|v| format!("{v:+.0} dB"))
                    .on_change(move |v, _, cx| {
                        ms2.update(cx, |s, cx| {
                            s.db = v;
                            cx.notify()
                        })
                    }),
            );
        out.push((
            "Inputs",
            "Toggle · Checkbox · Radio",
            section!("Toggle · Checkbox · Radio")
                .child(controls)
                .child(radios)
                .into_any_element(),
            "Toggle · Checkbox · Radio",
        ));
        let mut variants = col();
        for v in CheckboxVariant::ALL {
            let name = v.label();
            let id = |k: &str| SharedString::from(format!("cv-{name}-{k}"));
            let cell = |e: gpui::AnyElement| {
                div()
                    .w(px(56.0))
                    .flex()
                    .justify_center()
                    .child(e)
                    .into_any_element()
            };
            variants = variants.child(
                row()
                    .child(div().w(px(110.0)).child(Label::new(name)))
                    .child(cell(
                        Checkbox::new(id("off"), false)
                            .variant(v)
                            .into_any_element(),
                    ))
                    .child(cell(
                        Checkbox::new(id("on"), true).variant(v).into_any_element(),
                    ))
                    .child(cell(
                        Checkbox::new(id("mix"), false)
                            .indeterminate(true)
                            .variant(v)
                            .into_any_element(),
                    ))
                    .child(cell(
                        Checkbox::new(id("dis"), true)
                            .disabled(true)
                            .variant(v)
                            .into_any_element(),
                    ))
                    .child(cell(
                        Radio::new(id("ra"), true).variant(v).into_any_element(),
                    )),
            );
        }
        out.push((
            "Inputs",
            "Checkbox variants",
            section!("Checkbox variants")
                .child(variants)
                .into_any_element(),
            "Checkbox variants",
        ));
        out.push((
            "Inputs",
            "Slider · Progress",
            section!("Slider · Progress")
                .child(sliders)
                .into_any_element(),
            "Slider · Progress",
        ));
        out.push((
            "Inputs",
            "Stepped slider · ticks · formatted value",
            section!("Slider (stepped, ticks, value readout)")
                .child(stepped)
                .into_any_element(),
            "Slider (stepped, ticks, value readout)",
        ));
        out.push((
            "Inputs",
            "Range slider",
            section!("Range slider")
                .child(w_range)
                .child(w_range_txt)
                .into_any_element(),
            "Range slider",
        ));
        out.push((
            "Inputs",
            "Knob · Vertical slider · Vertical range slider",
            section!("Knob · Vertical sliders")
                .child(dials)
                .into_any_element(),
            "Knob · Vertical sliders",
        ));
        out.push((
            "Inputs",
            "Input field",
            section!("Input")
                .child(div().flex_1().min_w(px(260.0)).child(self.in_name.clone()))
                .child(div().flex_1().min_w(px(260.0)).child(self.in_notes.clone()))
                .child(InputBox::new().placeholder("static placeholder box"))
                .child(
                    div()
                        .flex_1()
                        .min_w(px(260.0))
                        .child(self.in_outlined.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(260.0))
                        .child(self.in_filled.clone()),
                )
                .into_any_element(),
            "Input",
        ));
        let full = SearchView::new("search-full", self.search_full.clone())
            .items(["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"])
            .mode(dodai_gpui::SearchMode::FullScreen)
            .open(true);
        out.push((
            "Inputs",
            "Search view · SearchBar",
            section!("Search view")
                .child(w_search)
                .child(Muted::new("Full screen (Material)"))
                .child(
                    div()
                        .relative()
                        .w_full()
                        .h(px(380.0))
                        .overflow_hidden()
                        .child(full),
                )
                .into_any_element(),
            "Search view",
        ));
        out.push((
            "Inputs",
            "Tag field · chips",
            section!("Tag field").child(w_tags).into_any_element(),
            "Tag field",
        ));
        out.push((
            "Inputs",
            "Date picker · Time picker",
            section!("Date / time pickers")
                .child(pickers)
                .into_any_element(),
            "Date / time pickers",
        ));
        out.push((
            "Inputs",
            "Password · One-time code",
            section!("Password · OTP")
                .child(
                    dodai_gpui::FormField::new("Password")
                        .required(true)
                        .help("Shown as dots; copy is disabled")
                        .child(self.pw.clone()),
                )
                .child(
                    dodai_gpui::FormField::new("Verification code")
                        .help("Type or paste 6 digits")
                        .child(dodai_gpui::OtpInput::new("otp", 6)),
                )
                .child(
                    dodai_gpui::FormField::new("PIN (masked)")
                        .child(dodai_gpui::OtpInput::new("pin", 4).masked(true)),
                )
                .into_any_element(),
            "Password · OTP",
        ));
        out.push((
            "Inputs",
            "Form field · label help error validation",
            section!("Form field")
                .child(
                    row()
                        .items_start()
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Name")
                                    .required(true)
                                    .help("As on your ID")
                                    .child(InputBox::new().placeholder("Ada Lovelace")),
                            ),
                        )
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Email")
                                    .error("Enter a valid address")
                                    .child(InputBox::new().placeholder("name@")),
                            ),
                        )
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Username")
                                    .success("Available")
                                    .child(InputBox::new().placeholder("ada")),
                            ),
                        )
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_gpui::FormField::new("Bio")
                                    .warning("Over 80% of the limit")
                                    .child(InputBox::new().placeholder("…")),
                            ),
                        ),
                )
                .into_any_element(),
            "Form field",
        ));
        out.push((
            "Inputs",
            "Toggle group · Checkbox group · select all · switches",
            section!("Toggle group · Checkbox group")
                .child(
                    row()
                        .child(
                            dodai_gpui::ToggleGroup::new("tg1", ["Left", "Center", "Right"])
                                .selected([0]),
                        )
                        .child(
                            dodai_gpui::ToggleGroup::new("tg2", ["Bold", "Italic", "Underline"])
                                .multiple(true)
                                .selected([0, 2]),
                        ),
                )
                .child(
                    row()
                        .items_start()
                        .gap(px(48.0))
                        .child(
                            dodai_gpui::CheckboxGroup::new("cg", ["Email", "SMS", "Push", "Slack"])
                                .select_all("All channels")
                                .checked([0, 2]),
                        )
                        .child(
                            dodai_gpui::CheckboxGroup::new(
                                "sg",
                                ["Wi-Fi", "Bluetooth", "Location"],
                            )
                            .select_all("Everything")
                            .switches(true)
                            .checked([0]),
                        ),
                )
                .into_any_element(),
            "Toggle group · Checkbox group",
        ));
        out.push((
            "Inputs",
            "Date range picker",
            section!("Date range")
                .child(range_picker)
                .into_any_element(),
            "Date range",
        ));
        out
    }
}
