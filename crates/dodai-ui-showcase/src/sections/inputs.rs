//! The "Inputs" category sections.

use super::super::*;
use dodai_ui_gpui::CheckboxVariant;

impl Showcase {
    /// The builders of this category, one per group of sections (built lazily).
    pub(crate) fn groups_inputs() -> Vec<Group> {
        vec![
            Self::toggle_section,
            Self::variants_section,
            Self::group_section,
            Self::slider_section,
            Self::stepped_section,
            Self::range_section,
            Self::knob_section,
            Self::input_section,
            Self::search_section,
            Self::tag_field_section,
            Self::password_section,
            Self::form_field_section,
            Self::picker_section,
            Self::date_range_section,
        ]
    }

    fn toggle_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let me_sw = me.clone();
        let me_ck = me.clone();
        let a = self.animation;
        let controls = row()
            .child(
                Toggle::new("sw", self.switch)
                    .animation(a)
                    .on_change(move |v, _, cx| {
                        me_sw.update(cx, |s, cx| {
                            s.switch = v;
                            cx.notify();
                        })
                    }),
            )
            .child(Label::new("Switch"))
            .child(Toggle::new("sw-icons", true).icons(true))
            .child(Toggle::new("sw-icons-off", false).icons(true))
            .child(Label::new("With icons"))
            .child(Toggle::new("sw-dis", true).disabled(true))
            .child(
                Checkbox::new("ck", self.check)
                    .animation(a)
                    .on_change(move |v, _, cx| {
                        me_ck.update(cx, |s, cx| {
                            s.check = v;
                            cx.notify();
                        })
                    }),
            )
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
                    Radio::new(("radio", i), self.radio == i)
                        .animation(a)
                        .on_change(move |_, _, cx| {
                            m.update(cx, |s, cx| {
                                s.radio = i;
                                cx.notify();
                            })
                        }),
                )
                .child(Label::new(name));
        }
        out.push((
            "Inputs",
            "Toggle · Checkbox · Radio",
            self.section("Toggle · Checkbox · Radio", cx)
                .child(Muted::new("Animation"))
                .child(self.animation_picker("choice-animation", cx))
                .child(controls)
                .child(radios)
                .into_any_element(),
            "Toggle · Checkbox · Radio",
        ));
        out
    }

    fn variants_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let col = || div().flex().flex_col().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let mut variants = col();
        for v in CheckboxVariant::ALL {
            let name = v.label();
            let id = |k: &str| SharedString::from(format!("cv-{name}-{k}"));
            let cell = |e: gpui::AnyElement| {
                div()
                    .w(px(44.0))
                    .flex_none()
                    .flex()
                    .justify_center()
                    .child(e)
                    .into_any_element()
            };
            // One non-wrapping row: the label column gives way before the cells wrap.
            variants = variants.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(t.spacing.xs))
                    .child(div().flex_1().min_w(px(64.0)).child(Label::new(name)))
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
        // Sizes and shapes are Material 3 options; other designs ignore them.
        if t.look().tonal {
            out.push((
                "Inputs",
                "Checkbox variants",
                self.section("Checkbox variants", cx)
                    .child(variants)
                    .into_any_element(),
                "Checkbox variants",
            ));
        }
        out
    }

    fn slider_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let mut out: Vec<Section> = Vec::new();
        let me_sl = me.clone();
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
            .child(Muted::new("Disabled · arrow keys move a focused slider"))
            .child(Slider::new("slider-off", 0.4).disabled(true))
            .child(ProgressBar::new(self.slider))
            .child(ProgressBar::new(0.7).tone(Tone::Success))
            .child(ProgressBar::new(0.4).tone(Tone::Warning))
            .child(ProgressBar::new(0.55).wavy(true))
            .child(ProgressBar::new(0.15).tone(Tone::Danger));
        out.push((
            "Inputs",
            "Slider · Progress",
            self.section("Slider · Progress", cx)
                .child(sliders)
                .into_any_element(),
            "Slider · Progress",
        ));
        out
    }

    fn stepped_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let col = || div().flex().flex_col().gap(gap);
        let mut out: Vec<Section> = Vec::new();
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
            )
            .child(
                Slider::new("centered", self.slider)
                    .centered(true)
                    .show_value(true)
                    .format(|v| format!("balance {:+.0}", (v - 0.5) * 200.0))
                    .on_change({
                        let m = me.clone();
                        move |v, _, cx| {
                            m.update(cx, |s, cx| {
                                s.slider = v;
                                cx.notify()
                            })
                        }
                    }),
            )
            .child(
                Slider::new("inset", self.slider)
                    .icon(dodai_ui_gpui::IconName::Heart)
                    .on_change({
                        let m = me.clone();
                        move |v, _, cx| {
                            m.update(cx, |s, cx| {
                                s.slider = v;
                                cx.notify()
                            })
                        }
                    }),
            );
        out.push((
            "Inputs",
            "Stepped slider · ticks · formatted value",
            self.section("Slider (stepped, ticks, value readout)", cx)
                .child(stepped)
                .into_any_element(),
            "Slider (stepped, ticks, value readout)",
        ));
        out
    }

    fn range_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
        let m3 = me.clone();
        let w_range =
            RangeSlider::new("range", self.range.0, self.range.1).on_change(move |a, b, _, cx| {
                m3.update(cx, |s, cx| {
                    s.range = (a, b);
                    cx.notify()
                })
            });
        let w_range_txt = Muted::new(format!("range {:.2} – {:.2}", self.range.0, self.range.1));
        out.push((
            "Inputs",
            "Range slider",
            self.section("Range slider", cx)
                .child(w_range)
                .child(w_range_txt)
                .into_any_element(),
            "Range slider",
        ));
        out
    }

    fn knob_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
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
                            .orientation(Orientation::Vertical)
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
                            .orientation(Orientation::Vertical)
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
        out.push((
            "Inputs",
            "Knob · Vertical slider · Vertical range slider",
            self.section("Knob · Vertical sliders", cx)
                .child(dials)
                .into_any_element(),
            "Knob · Vertical sliders",
        ));
        out
    }

    fn input_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        vec![(
            "Inputs",
            "Input field",
            self.section("Input", cx)
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
        )]
    }

    fn search_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
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
            .on_select(move |_, _, _| {
                let _ = &m;
            });
        let w_search = search;
        let full = SearchView::new("search-full", self.search_full.clone())
            .items(["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"])
            .mode(dodai_ui_gpui::SearchMode::FullScreen)
            .open(true);
        out.push((
            "Inputs",
            "Search view · SearchBar",
            self.section("Search view", cx)
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
        out
    }

    fn tag_field_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
        let m4 = me.clone();
        let w_tags = TagField::new("tags", self.tags.clone()).on_remove(move |i, _, cx| {
            m4.update(cx, |s, cx| {
                s.tags.remove(i);
                cx.notify()
            })
        });
        out.push((
            "Inputs",
            "Tag field · chips",
            self.section("Tag field", cx)
                .child(w_tags)
                .into_any_element(),
            "Tag field",
        ));
        out
    }

    fn picker_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let (m5, m6) = (me.clone(), me.clone());
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
            ))
            .child(
                dodai_ui_gpui::DateTimePicker::new("datetime", y, mo)
                    .selected(sel)
                    .time(self.time.0, self.time.1)
                    .twelve_hour(true)
                    .on_date({
                        let m = me.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date = (d.year, d.month, Some(d));
                                cx.notify()
                            })
                        }
                    })
                    .on_month({
                        let m = me.clone();
                        move |y, mo, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date.0 = y;
                                s.date.1 = mo;
                                cx.notify()
                            })
                        }
                    })
                    .on_time({
                        let m = me.clone();
                        move |h, mi, _, cx| {
                            m.update(cx, |s, cx| {
                                s.time = (h, mi);
                                cx.notify()
                            })
                        }
                    }),
            );
        let typed = row()
            .items_start()
            .child(
                dodai_ui_gpui::DateInput::new("date-in")
                    .value(sel)
                    .on_change({
                        let m = me.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date = (d.year, d.month, Some(d));
                                cx.notify()
                            })
                        }
                    }),
            )
            .child(
                dodai_ui_gpui::TimeInput::new("time-in")
                    .value(Some(self.time))
                    .on_change({
                        let m = me.clone();
                        move |h, mi, _, cx| {
                            m.update(cx, |s, cx| {
                                s.time = (h, mi);
                                cx.notify()
                            })
                        }
                    }),
            );
        out.push((
            "Inputs",
            "Date picker · Time picker",
            self.section("Date / time pickers", cx)
                .child(pickers)
                .child(Muted::new("Text input mode"))
                .child(typed)
                .into_any_element(),
            "Date / time pickers",
        ));
        out
    }

    fn password_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        vec![(
            "Inputs",
            "Password · One-time code",
            self.section("Password · OTP", cx)
                .child(
                    dodai_ui_gpui::FormField::new("Password")
                        .required(true)
                        .help("Shown as dots; copy is disabled")
                        .child(self.pw.clone()),
                )
                .child(
                    dodai_ui_gpui::FormField::new("Verification code")
                        .help("Type or paste 6 digits")
                        .child(dodai_ui_gpui::OtpInput::new("otp", 6).value(self.demo_otp.clone())),
                )
                .child(
                    dodai_ui_gpui::FormField::new("PIN (masked)").child(
                        dodai_ui_gpui::OtpInput::new("pin", 4)
                            .masked(true)
                            .value(self.demo_pin.clone()),
                    ),
                )
                .into_any_element(),
            "Password · OTP",
        )]
    }

    fn form_field_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        vec![(
            "Inputs",
            "Form field · label help error validation",
            self.section("Form field", cx)
                .child(
                    row()
                        .items_start()
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_ui_gpui::FormField::new("Name")
                                    .required(true)
                                    .help("As on your ID")
                                    .child(InputBox::new().placeholder("Ada Lovelace")),
                            ),
                        )
                        .child(div().flex_1().min_w(px(220.0)).child({
                            let mail = self.demo_email.clone();
                            let ok = mail.contains('@')
                                && mail
                                    .rsplit('.')
                                    .next()
                                    .is_some_and(|d| mail.contains('.') && d.len() >= 2);
                            let f = dodai_ui_gpui::FormField::new("Email");
                            let f = if ok {
                                f.success("Looks good")
                            } else if mail.is_empty() {
                                f
                            } else {
                                f.error("Enter a valid address")
                            };
                            f.child(InputBox::new().placeholder("name@").child(Mono::new(mail)))
                        }))
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_ui_gpui::FormField::new("Username")
                                    .success("Available")
                                    .child(InputBox::new().placeholder("ada")),
                            ),
                        )
                        .child(
                            div().flex_1().min_w(px(220.0)).child(
                                dodai_ui_gpui::FormField::new("Bio")
                                    .warning("Over 80% of the limit")
                                    .child(InputBox::new().placeholder("…")),
                            ),
                        ),
                )
                .into_any_element(),
            "Form field",
        )]
    }

    fn group_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        vec![(
            "Inputs",
            "Toggle group · Checkbox group · select all · switches",
            self.section("Toggle group · Checkbox group", cx)
                .child(
                    row()
                        .child(
                            dodai_ui_gpui::ToggleGroup::new("tg1", ["Left", "Center", "Right"])
                                .selected([0]),
                        )
                        .child(
                            dodai_ui_gpui::ToggleGroup::new("tg2", ["Bold", "Italic", "Underline"])
                                .multiple(true)
                                .selected([0, 2]),
                        ),
                )
                .child(
                    row()
                        .items_start()
                        .gap(px(48.0))
                        .child(
                            dodai_ui_gpui::CheckboxGroup::new(
                                "cg",
                                ["Email", "SMS", "Push", "Slack"],
                            )
                            .select_all("All channels")
                            .selected([0, 2]),
                        )
                        .child(
                            dodai_ui_gpui::CheckboxGroup::new(
                                "sg",
                                ["Wi-Fi", "Bluetooth", "Location"],
                            )
                            .select_all("Everything")
                            .switches(true)
                            .selected([0]),
                        ),
                )
                .into_any_element(),
            "Toggle group · Checkbox group",
        )]
    }

    fn date_range_section(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.demo_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let (y, mo, _) = self.date;
        let m_range = me.clone();
        let range_picker = row()
            .child(
                DatePicker::new("date-range", y, mo)
                    .range(self.date_range)
                    .on_select({
                        let m = m_range.clone();
                        move |d, _, cx| {
                            m.update(cx, |s, cx| {
                                s.date_range = dodai_ui_gpui::range_pick(s.date_range, d);
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
        out.push((
            "Inputs",
            "Date range picker",
            self.section("Date range", cx)
                .child(range_picker)
                .into_any_element(),
            "Date range",
        ));
        let ((sh, sm), (eh, em)) = self.range_times;
        let timed = dodai_ui_gpui::DateTimeRangePicker::new("date-time-range", y, mo)
            .range(self.date_range)
            .start_time(sh, sm)
            .end_time(eh, em)
            .on_select({
                let m = me.clone();
                move |d, _, cx| {
                    m.update(cx, |s, cx| {
                        s.date_range = dodai_ui_gpui::range_pick(s.date_range, d);
                        cx.notify()
                    })
                }
            })
            .on_month({
                let m = me.clone();
                move |y, mo, _, cx| {
                    m.update(cx, |s, cx| {
                        s.date.0 = y;
                        s.date.1 = mo;
                        cx.notify()
                    })
                }
            })
            .on_start_time({
                let m = me.clone();
                move |h, mi, _, cx| {
                    m.update(cx, |s, cx| {
                        s.range_times.0 = (h, mi);
                        cx.notify()
                    })
                }
            })
            .on_end_time({
                let m = me.clone();
                move |h, mi, _, cx| {
                    m.update(cx, |s, cx| {
                        s.range_times.1 = (h, mi);
                        cx.notify()
                    })
                }
            });
        out.push((
            "Inputs",
            "Date-time range picker",
            self.section("Date-time range", cx)
                .child(row().items_start().child(timed))
                .child(Muted::new(
                    "Shares its span with the date range above; the start and end times are its own.",
                ))
                .into_any_element(),
            "Date-time range",
        ));
        out
    }
}
