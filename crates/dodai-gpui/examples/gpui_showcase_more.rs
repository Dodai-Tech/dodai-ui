//! `cargo run -p dodai-gpui --example gpui_showcase_more`
//!
//! M3 gaps, beyond-M3 components and charts. Keys: `t` / `T` cycle themes.

use dodai_core::{Theme, Tone};
use dodai_gpui::{
    cycle_next, cycle_prev, hsla, install, theme_of, BarChart, Date, DatePicker,
    DodaiStyled, EmptyState, Gauge, Heatmap, IconBadge, IconButton, LineChart, Muted,
    NumberStepper, Panel, PieChart, RadarChart, RangeSlider, Rating, RichTooltip, Scanlines,
    ScatterPlot, SearchView, Series, Snackbar, Sparkline, TagField, TextInput, TimePicker,
    Timeline, TimelineEvent, Toolbar, TreeItem, TreeView,
};
use gpui::{
    div, prelude::*, px, size, App, Application, Bounds, Context, Entity, FocusHandle,
    KeyDownEvent, Window, WindowBounds, WindowOptions,
};

struct More {
    focus: FocusHandle,
    search: Entity<TextInput>,
    rating: u32,
    stepper: i32,
    range: (f32, f32),
    date: (i32, u32, Option<Date>),
    time: (u32, u32),
    tags: Vec<&'static str>,
}

impl Render for More {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let section = |title: &'static str| Panel::new().title(title);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);

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
        let widgets = col()
            .child(search)
            .child(
                row()
                    .child(
                        Rating::new("rating", self.rating).on_change(move |v, _, cx| {
                            m1.update(cx, |s, cx| {
                                s.rating = v;
                                cx.notify()
                            })
                        }),
                    )
                    .child(
                        NumberStepper::new("step", self.stepper)
                            .range(0, 20)
                            .on_change(move |v, _, cx| {
                                m2.update(cx, |s, cx| {
                                    s.stepper = v;
                                    cx.notify()
                                })
                            }),
                    )
                    .child(
                        div()
                            .id("rich")
                            .text_color(hsla(t.palette.link))
                            .tooltip(RichTooltip::build(
                                "Rich tooltip",
                                "Title, body and an action.",
                                Some("Learn more"),
                            ))
                            .child("Hover (rich)"),
                    )
                    .child(IconBadge::new(IconButton::new("bell", "🔔")).count(12, Tone::Danger))
                    .child(IconBadge::new(IconButton::new("mail", "✉")).dot(Tone::Success)),
            )
            .child(
                RangeSlider::new("range", self.range.0, self.range.1).on_change(
                    move |a, b, _, cx| {
                        m3.update(cx, |s, cx| {
                            s.range = (a, b);
                            cx.notify()
                        })
                    },
                ),
            )
            .child(Muted::new(format!(
                "range {:.2} – {:.2}",
                self.range.0, self.range.1
            )))
            .child(
                TagField::new("tags", self.tags.clone()).on_remove(move |i, _, cx| {
                    m4.update(cx, |s, cx| {
                        s.tags.remove(i);
                        cx.notify()
                    })
                }),
            )
            .child(Snackbar::new("snack", "Message archived").action("Undo", |_, _| {}))
            .child(
                Toolbar::new()
                    .child(IconButton::new("tb1", "✎"))
                    .child(IconButton::new("tb2", "⎘")),
            );

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

        let tree = TreeView::new(
            "tree",
            vec![
                TreeItem::new("src")
                    .expanded(true)
                    .child(TreeItem::new("lib.rs"))
                    .child(TreeItem::new("charts.rs")),
                TreeItem::new("examples").child(TreeItem::new("gpui_showcase.rs")),
            ],
        );
        let timeline = Timeline::new([
            TimelineEvent::new("Charts added")
                .time("today")
                .tone(Tone::Success),
            TimelineEvent::new("Search view")
                .time("today")
                .detail("fuzzy + highlight"),
            TimelineEvent::new("Build failed")
                .time("earlier")
                .tone(Tone::Danger),
        ]);

        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
        let a = Series::new("Revenue", [12.0, 18.0, 14.0, 26.0, 22.0, 31.0]);
        let b = Series::new("Cost", [8.0, 11.0, 13.0, 15.0, 17.0, 16.0]);
        let charts = col()
            .child(
                row()
                    .child(
                        div().w(px(420.0)).child(
                            LineChart::new("line", [a.clone(), b.clone()])
                                .labels(months)
                                .area(true)
                                .smooth(true),
                        ),
                    )
                    .child(
                        div()
                            .w(px(420.0))
                            .child(BarChart::new("bars", [a.clone(), b.clone()]).labels(months)),
                    ),
            )
            .child(
                row()
                    .child(
                        div().w(px(420.0)).child(
                            BarChart::new("stack", [a.clone(), b.clone()])
                                .labels(months)
                                .stacked(true),
                        ),
                    )
                    .child(
                        div().w(px(420.0)).child(
                            BarChart::new("hbars", [a, b])
                                .labels(months)
                                .horizontal(true),
                        ),
                    ),
            )
            .child(
                row()
                    .child(PieChart::new([
                        ("Rust", 52.0),
                        ("Kotlin", 28.0),
                        ("Other", 20.0),
                    ]))
                    .child(
                        PieChart::new([("A", 3.0), ("B", 2.0), ("C", 1.0)])
                            .donut(0.62)
                            .center("6"),
                    )
                    .child(Gauge::new(72.0).label("72%").tone(Tone::Success))
                    .child(Gauge::new(31.0).tone(Tone::Warning))
                    .child(Sparkline::new([3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0])),
            )
            .child(
                row()
                    .child(RadarChart::new(
                        ["Speed", "Power", "Range", "Armor", "Stealth"],
                        [
                            Series::new("Unit A", [8.0, 6.0, 7.0, 4.0, 9.0]),
                            Series::new("Unit B", [5.0, 9.0, 4.0, 8.0, 3.0]),
                        ],
                    ))
                    .child(
                        div().w(px(380.0)).child(ScatterPlot::new(
                            "sc",
                            [
                                (
                                    "Group 1",
                                    (0..12)
                                        .map(|i| (i as f32, (i * i % 7) as f32 + 1.0))
                                        .collect(),
                                ),
                                (
                                    "Group 2",
                                    (0..12)
                                        .map(|i| (i as f32 + 0.5, (i * 3 % 8) as f32))
                                        .collect(),
                                ),
                            ],
                        )),
                    )
                    .child(
                        Heatmap::new(
                            "heat",
                            (0..5)
                                .map(|r| (0..8).map(|c| ((r * 3 + c * 5) % 11) as f32).collect())
                                .collect(),
                        )
                        .row_labels(["Mon", "Tue", "Wed", "Thu", "Fri"])
                        .col_labels(["0", "3", "6", "9", "12", "15", "18", "21"]),
                    ),
            );

        let body = col()
            .p(px(t.spacing.lg))
            .child(section("Search · rating · stepper · badges · tags · snackbar").child(widgets))
            .child(section("Date / time pickers").child(pickers))
            .child(
                section("Tree · timeline · empty state").child(
                    row()
                        .child(div().w(px(240.0)).child(tree))
                        .child(div().w(px(280.0)).child(timeline))
                        .child(
                            EmptyState::new("∅", "Nothing here")
                                .message("Add something to get started."),
                        ),
                ),
            )
            .child(section("Charts").child(charts));

        div()
            .id("root")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|_, e: &KeyDownEvent, _, cx| {
                if e.keystroke.key == "t" {
                    if e.keystroke.modifiers.shift {
                        cycle_prev(cx)
                    } else {
                        cycle_next(cx)
                    }
                }
            }))
            .relative()
            .size_full()
            .dodai_background(&t)
            .child(
                div()
                    .id("scroll")
                    .size_full()
                    .overflow_y_scroll()
                    .child(body),
            )
            .child(Scanlines::new())
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        install(cx, Theme::default());
        let bounds = Bounds::centered(None, size(px(1000.0), px(800.0)), cx);
        let win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| {
                    let search = cx.new(|cx| TextInput::new(cx).placeholder("Search…"));
                    cx.new(|cx| More {
                        focus: cx.focus_handle(),
                        search,
                        rating: 3,
                        stepper: 5,
                        range: (0.2, 0.7),
                        date: (2025, 6, None),
                        time: (9, 30),
                        tags: vec!["rust", "gpui", "charts"],
                    })
                },
            )
            .expect("open window");
        win.update(cx, |s, w, _| w.focus(&s.focus)).ok();
        cx.activate(true);
    });
}
