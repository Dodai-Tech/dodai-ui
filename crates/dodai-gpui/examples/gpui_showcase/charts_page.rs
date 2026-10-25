//! Chart demo tiles (line/bar, pies, donuts, gauges, sparklines, instruments, ...).

use super::*;
use gpui::Div;

/// The chart sections' bodies, built once per render.
pub(crate) struct ChartDemos {
    pub charts_xy: Div,
    pub gauge_styles: Div,
    pub instruments: Div,
    pub pies: Div,
    pub donuts: Div,
    pub sparks: Div,
    pub charts_market: Div,
    pub charts_misc: Div,
    pub charts_more1: Div,
    pub charts_more2: Div,
    pub charts_more3: Div,
}

impl Showcase {
    pub(crate) fn chart_demos(&mut self, me: &Entity<Self>, cx: &mut Context<Self>) -> ChartDemos {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let me = me.clone();
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let col = || div().flex().flex_col().gap(gap);
        let hres = self.heat_res.clamp(1, 8);
        let (hr, hc) = (12 * hres, 32 * hres);
        let heat_cell = (264.0 / hr as f32).clamp(3.0, 22.0);
        let hm = me.clone();
        let heat_block = col()
            .flex_1()
            .min_w(px(480.0))
            .child(
                row()
                    .child(Label::new(format!("Resolution {hc}×{hr}")))
                    .child(
                        Button::new(
                            "heat-grad",
                            format!(
                                "Gradient: {}",
                                dodai_gpui::Gradient::ALL
                                    [self.heat_grad % dodai_gpui::Gradient::ALL.len()]
                                .name()
                            ),
                        )
                        .variant(ButtonVariant::Secondary)
                        .on_click({
                            let m = hm.clone();
                            move |_, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.heat_grad += 1;
                                    cx.notify();
                                })
                            }
                        }),
                    )
                    .child(
                        Button::new("heat-minus", "−")
                            .variant(ButtonVariant::Secondary)
                            .on_click({
                                let m = hm.clone();
                                move |_, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.heat_res = s.heat_res.saturating_sub(1).max(1);
                                        cx.notify();
                                    })
                                }
                            }),
                    )
                    .child(
                        Button::new("heat-plus", "+")
                            .variant(ButtonVariant::Secondary)
                            .on_click({
                                let m = hm.clone();
                                move |_, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.heat_res = (s.heat_res + 1).min(8);
                                        cx.notify();
                                    })
                                }
                            }),
                    ),
            )
            .child(
                Heatmap::new(
                    format!("heat-{hres}"),
                    (0..hr)
                        .map(|r| {
                            (0..hc)
                                .map(|c| {
                                    let (x, y) = (c as f32 / hres as f32, r as f32 / hres as f32);
                                    ((x * 0.21 * 2.0).sin()
                                        + (y * 0.45).cos()
                                        + ((x * 2.0 + y) * 0.13).sin() * 1.5)
                                        + ((x - 16.0).hypot((y - 6.0) * 2.0) * -0.16).exp() * 3.0
                                })
                                .collect()
                        })
                        .collect(),
                )
                .cell(heat_cell)
                .gradient(
                    dodai_gpui::Gradient::ALL[self.heat_grad % dodai_gpui::Gradient::ALL.len()],
                )
                .row_labels((0..hr).map(|r| {
                    if r % (4 * hres) == 0 {
                        format!("{:02}h", r / hres * 2)
                    } else {
                        String::new()
                    }
                }))
                .col_labels((0..hc).map(|c| {
                    if c % (8 * hres) == 0 {
                        format!("{}", c / hres)
                    } else {
                        String::new()
                    }
                })),
            );

        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
        let a = Series::new("Revenue", [12.0, 18.0, 14.0, 26.0, 22.0, 31.0]);
        let b = Series::new("Cost", [8.0, 11.0, 13.0, 15.0, 17.0, 16.0]);
        let charts_xy = col()
            .child(
                row()
                    .child(
                        div().flex_1().min_w(px(300.0)).child(
                            LineChart::new("line", [a.clone(), b.clone()])
                                .labels(months)
                                .area(true)
                                .smooth(true),
                        ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(300.0))
                            .child(BarChart::new("bars", [a.clone(), b.clone()]).labels(months)),
                    ),
            )
            .child(
                row()
                    .child(
                        div().flex_1().min_w(px(300.0)).child(
                            BarChart::new("stack", [a.clone(), b.clone()])
                                .labels(months)
                                .stacked(true),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(300.0)).child(
                            BarChart::new("hbars", [a, b])
                                .labels(months)
                                .horizontal(true),
                        ),
                    ),
            );
        let gauge_styles = {
            use GaugeStyle::*;
            let dials = row()
                .justify_around()
                .flex_wrap()
                .child(Gauge::new(72.0).label("72%").tone(Tone::Success))
                .child(Gauge::new(31.0).tone(Tone::Warning))
                .child(
                    Gauge::new(68.0)
                        .style(Segmented)
                        .size(130.0)
                        .caption("Core temp")
                        .label("68°"),
                )
                .child(
                    Gauge::new(72.0)
                        .style(Dial)
                        .zones(true)
                        .size(150.0)
                        .caption("Load")
                        .label("72"),
                )
                .child(
                    Gauge::new(88.0)
                        .style(Meter)
                        .zones(true)
                        .size(150.0)
                        .caption("Shield")
                        .label("88%"),
                )
                .child(
                    Gauge::new(45.0)
                        .style(Reticle)
                        .size(130.0)
                        .tone(Tone::Info)
                        .caption("Sync")
                        .label("45%"),
                );
            let bars = div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    Gauge::new(82.0)
                        .style(Bar)
                        .size(150.0)
                        .zones(true)
                        .caption("Power")
                        .label("82%"),
                )
                .child(
                    Gauge::new(35.0)
                        .style(Bar)
                        .size(150.0)
                        .tone(Tone::Info)
                        .caption("Link")
                        .label("35%"),
                )
                .child(
                    Gauge::new(14.0)
                        .style(Bar)
                        .size(150.0)
                        .tone(Tone::Danger)
                        .caption("Hull")
                        .label("14%"),
                );
            let more = div()
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    Gauge::new(64.0)
                        .style(Chevron)
                        .size(150.0)
                        .zones(true)
                        .caption("Thrust")
                        .label("64%"),
                )
                .child(
                    Gauge::new(58.0)
                        .style(Level)
                        .size(110.0)
                        .zones(true)
                        .caption("Fuel")
                        .label("58%"),
                );
            let bars = row().justify_around().flex_wrap().child(bars).child(more);
            div().flex().flex_col().gap_6().child(dials).child(bars)
        };
        let instruments = {
            let glow = match self.inst_glow {
                1 => dodai_gpui::ChartGlow::Off,
                2 => dodai_gpui::ChartGlow::Radius(4.0),
                3 => dodai_gpui::ChartGlow::Radius(10.0),
                _ => dodai_gpui::ChartGlow::Theme,
            };
            let trail_picker = {
                let m = me.clone();
                row()
                    .child(Label::new("Trail"))
                    .child(
                        Segmented::new("inst-trail", ["Off", "Subtle", "Default", "Long"])
                            .selected(self.inst_trail)
                            .on_select(move |i, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.inst_trail = i;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Muted::new(
                        "The faded afterimage that follows the live trace to the right.",
                    ))
            };
            let trail = [0.0, 0.5, 1.0, 2.0][self.inst_trail.min(3)];
            let glow_picker = {
                let m = me.clone();
                row()
                    .child(Label::new("Glow"))
                    .child(
                        Segmented::new("inst-glow", ["Theme", "Off", "Soft", "Strong"])
                            .selected(self.inst_glow)
                            .on_select(move |i, _, cx| {
                                m.update(cx, |s, cx| {
                                    s.inst_glow = i;
                                    cx.notify();
                                })
                            }),
                    )
                    .child(Muted::new(
                        "The soft shadow under traces and lit cells. Theme uses the design's own.",
                    ))
            };
            let spec: Vec<f32> = (0..18)
                .map(|i| 0.35 + 0.6 * ((i as f32 * 0.9).sin().abs()))
                .collect();
            let hexes: Vec<f32> = (0..7 * 12)
                .map(|i| {
                    let (r, c) = ((i / 12) as f32, (i % 12) as f32);
                    (((c * 0.55 + r * 0.8).sin() * (r * 0.9 - c * 0.3).cos()) * 0.5 + 0.5).powf(1.4)
                })
                .collect();
            let blips = [
                (0.08, 0.7),
                (0.31, 0.45),
                (0.52, 0.85),
                (0.77, 0.3),
                (0.9, 0.6),
            ]
            .map(|(a, r)| Blip::new(a, r));
            div()
                .flex()
                .flex_col()
                .gap_6()
                .child(glow_picker)
                .child(trail_picker)
                .child(
                    row()
                        .justify_around()
                        .flex_wrap()
                        .child(
                            Oscilloscope::ecg(2, 240)
                                .glow(glow)
                                .trail(trail)
                                .live(true)
                                .size(320.0, 120.0)
                                .caption("Pulse · 72 bpm"),
                        )
                        .child(
                            DotMatrix::new([
                                0.3, 0.55, 0.8, 0.45, 0.95, 0.6, 0.35, 0.7, 0.5, 0.85, 0.4, 0.65,
                            ])
                            .glow(glow)
                            .width(300.0)
                            .caption("Dot matrix"),
                        )
                        .child(
                            div()
                                .flex()
                                .items_end()
                                .gap_4()
                                .child(SignalBars::new(2).glow(glow))
                                .child(SignalBars::new(4).height(40.0).glow(glow))
                                .child(SignalBars::new(5).height(52.0).bars(6).glow(glow)),
                        )
                        .child(
                            Oscilloscope::wave(&[(3.0, 1.0), (7.0, 0.4)], 160)
                                .glow(glow)
                                .trail(trail)
                                .live(true)
                                .caption("CH1 · 2 ms/div"),
                        )
                        .child(
                            Spectrum::new(spec)
                                .glow(glow)
                                .live(true)
                                .size(300.0, 160.0)
                                .caption("Audio · 18 band"),
                        ),
                )
                .child(
                    row()
                        .justify_around()
                        .flex_wrap()
                        .child(
                            RadarScope::new(blips)
                                .glow(glow)
                                .size(220.0)
                                .caption("Proximity"),
                        )
                        .child(
                            HexMap::new(12, hexes)
                                .glow(glow)
                                .width(340.0)
                                .live(true)
                                .caption("Grid load"),
                        )
                        .child(
                            RingStack::new([
                                ("CPU", 78.0),
                                ("MEM", 54.0),
                                ("NET", 31.0),
                                ("GPU", 90.0),
                            ])
                            .glow(glow)
                            .size(160.0),
                        ),
                )
        };
        let tile = |title: &'static str, el: gpui::AnyElement| {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap_2()
                .child(el)
                .child(Muted::new(title))
                .into_any_element()
        };
        let mono = |c: dodai_core::Color| -> Vec<dodai_core::Color> {
            (0..5)
                .map(|i| c.mix(t.palette.background, i as f32 * 0.17))
                .collect()
        };
        let langs = || [("Rust", 52.0), ("Kotlin", 28.0), ("Other", 20.0)];
        let quarters = || [("Q1", 30.0), ("Q2", 24.0), ("Q3", 26.0), ("Q4", 20.0)];
        let pies = row()
            .justify_around()
            .flex_wrap()
            .gap_6()
            .child(tile("Classic", PieChart::new(langs()).into_any_element()))
            .child(tile(
                "Flush slices",
                PieChart::new(langs()).gap(0.0).into_any_element(),
            ))
            .child(tile(
                "Wide gaps",
                PieChart::new(quarters())
                    .gap(6.0)
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Exploded",
                PieChart::new(quarters())
                    .explode(6.0)
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Monochrome",
                PieChart::new([
                    ("A", 40.0),
                    ("B", 25.0),
                    ("C", 18.0),
                    ("D", 11.0),
                    ("E", 6.0),
                ])
                .colors(mono(t.palette.primary))
                .legend(false)
                .into_any_element(),
            ))
            .child(tile(
                "Rose",
                PieChart::new([
                    ("N", 9.0),
                    ("NE", 5.0),
                    ("E", 7.0),
                    ("SE", 3.0),
                    ("S", 8.0),
                    ("SW", 4.0),
                ])
                .style(PieStyle::Rose)
                .gap(1.0)
                .size(190.0)
                .legend(false)
                .into_any_element(),
            ))
            .child(tile(
                "Half pie",
                PieChart::new(langs())
                    .arc(0.75, 0.5)
                    .size(200.0)
                    .legend(false)
                    .into_any_element(),
            ));
        let donuts = row()
            .justify_around()
            .flex_wrap()
            .gap_6()
            .child(tile(
                "Centre value",
                PieChart::new([("A", 3.0), ("B", 2.0), ("C", 1.0)])
                    .donut(0.62)
                    .center("6")
                    .into_any_element(),
            ))
            .child(tile(
                "Thin ring",
                PieChart::new(quarters())
                    .donut(0.84)
                    .center("100")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Thick ring",
                PieChart::new(quarters())
                    .donut(0.42)
                    .gap(4.0)
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Segmented",
                PieChart::new(langs())
                    .style(PieStyle::Segmented)
                    .center("100")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "HUD ring",
                PieChart::new(quarters())
                    .style(PieStyle::Hud)
                    .size(190.0)
                    .center("4")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Segmented half",
                PieChart::new([("Used", 68.0), ("Free", 32.0)])
                    .style(PieStyle::Segmented)
                    .arc(0.75, 0.5)
                    .size(200.0)
                    .center("68%")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "HUD arc",
                PieChart::new([("A", 45.0), ("B", 30.0), ("C", 25.0)])
                    .style(PieStyle::Hud)
                    .arc(0.625, 0.75)
                    .size(190.0)
                    .center("A·B·C")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Half donut",
                PieChart::new([("Used", 68.0), ("Free", 32.0)])
                    .donut(0.7)
                    .arc(0.75, 0.5)
                    .size(200.0)
                    .center("68%")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Three-quarter",
                PieChart::new([("Done", 72.0), ("Left", 28.0)])
                    .donut(0.78)
                    .arc(0.625, 0.75)
                    .colors([t.palette.success, t.palette.border])
                    .center("72")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Monochrome",
                PieChart::new([
                    ("A", 40.0),
                    ("B", 25.0),
                    ("C", 18.0),
                    ("D", 11.0),
                    ("E", 6.0),
                ])
                .donut(0.6)
                .colors(mono(t.palette.secondary))
                .legend(false)
                .into_any_element(),
            ));
        let series = || {
            [
                3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0, 10.0, 14.0, 9.0, 13.0,
            ]
        };
        let spark = |name: &'static str, sp: Sparkline| {
            div()
                .flex_1()
                .min_w(px(110.0))
                .flex()
                .flex_col()
                .gap_1()
                .child(sp.fluid(true))
                .child(Muted::new(name))
        };
        let sparks = div()
            .flex()
            .flex_col()
            .gap_6()
            .child(
                row()
                    .gap_6()
                    .child(spark("Line + area", Sparkline::new(series())))
                    .child(spark("Line only", Sparkline::new(series()).area(false)))
                    .child(spark(
                        "Diamond tip",
                        Sparkline::new(series()).marker(Marker::Diamond),
                    ))
                    .child(spark(
                        "Mean baseline",
                        Sparkline::new(series()).baseline(true),
                    )),
            )
            .child(
                row()
                    .gap_6()
                    .child(spark(
                        "Step",
                        Sparkline::new(series()).style(SparkStyle::Step),
                    ))
                    .child(spark(
                        "Bars",
                        Sparkline::new(series()).style(SparkStyle::Bars),
                    ))
                    .child(spark(
                        "Dots",
                        Sparkline::new(series())
                            .style(SparkStyle::Dots)
                            .marker(Marker::Square),
                    ))
                    .child(spark(
                        "Win / loss",
                        Sparkline::new([
                            3.0, 5.0, 4.0, 8.0, 6.0, 2.0, 7.0, 1.0, 10.0, 4.0, 9.0, 3.0,
                        ])
                        .style(SparkStyle::WinLoss),
                    )),
            )
            .child(
                row()
                    .justify_around()
                    .child(CountUp::new("cu1", 12840.0).prefix("$ ").size(28.0))
                    .child(
                        CountUp::new("cu2", 98.6)
                            .decimals(1)
                            .suffix(" %")
                            .size(28.0),
                    ),
            );
        let w = |e: gpui::AnyElement, px_w: f32| {
            div()
                .flex_1()
                .min_w(px(px_w * 0.7))
                .child(e)
                .into_any_element()
        };
        let walk: Vec<f32> = (0..70)
            .scan(100.0f32, |p, i| {
                *p += ((i * 53 % 17) as f32 - 7.5) * 0.45;
                Some(*p)
            })
            .collect();
        let ohlc: Vec<Candle> = walk
            .windows(2)
            .enumerate()
            .map(|(i, p)| {
                let (o, c) = (p[0], p[1]);
                Candle::new(o, o.max(c) + 0.8, o.min(c) - 0.8, c)
                    .volume(50.0 + (i * 37 % 60) as f32)
            })
            .collect();
        let charts_market = row()
            .child(w(
                Candlestick::new("ext", ohlc.clone())
                    .style(CandleStyle::HeikinAshi)
                    .volume(true)
                    .overlay(CandleOverlay::Sma(10))
                    .overlay(CandleOverlay::Bollinger(20, 2.0))
                    .pane(CandlePane::Rsi(14))
                    .pane(CandlePane::Macd(12, 26, 9))
                    .markers([
                        TradeMarker::buy(20, ohlc[20].low),
                        TradeMarker::sell(45, ohlc[45].high),
                    ])
                    .height(240.0)
                    .title("Heikin-Ashi · volume · SMA · Bollinger · RSI · MACD")
                    .into_any_element(),
                520.0,
            ))
            .child(w(
                RenkoChart::new("renko", walk.clone(), 2.0)
                    .title("Renko · 2.0 bricks")
                    .into_any_element(),
                420.0,
            ))
            .child(w(
                DepthChart::new(
                    "depth",
                    (1..12).map(|i| (100.0 - i as f32 * 0.5, 4.0 + (i * 7 % 5) as f32)),
                    (1..12).map(|i| (100.5 + i as f32 * 0.5, 3.0 + (i * 5 % 6) as f32)),
                )
                .title("Order book depth")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Gantt::new(
                    "gantt",
                    [
                        GanttTask::new("Design", 0.0, 10.0).progress(1.0),
                        GanttTask::new("Build", 8.0, 24.0).progress(0.6).after(0),
                        GanttTask::new("Test", 20.0, 30.0).progress(0.25).after(1),
                        GanttTask::new("Ship", 30.0, 34.0).after(2),
                    ],
                )
                .title("Release plan")
                .format(|v| format!("d{v:.0}"))
                .into_any_element(),
                420.0,
            ))
            .child(w(
                LineChart::new(
                    "dual",
                    [Series::new("Revenue", [12.0, 18.0, 15.0, 26.0, 22.0, 31.0])],
                )
                .secondary([Series::new("Margin %", [0.21, 0.24, 0.2, 0.3, 0.28, 0.33])])
                .labels(months)
                .title("Secondary y axis")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                ChartStack::new()
                    .height(300.0)
                    .pane(3.0, |h| {
                        LineChart::new(
                            "st1",
                            [Series::new("Price", [3.0, 4.0, 3.5, 5.0, 6.0, 5.5])],
                        )
                        .height(h - 24.0)
                        .legend(false)
                        .into_any_element()
                    })
                    .pane(1.0, |h| {
                        BarChart::new("st2", [Series::new("Vol", [2.0, 3.0, 1.0, 4.0, 5.0, 3.0])])
                            .height(h - 24.0)
                            .legend(false)
                            .into_any_element()
                    })
                    .into_any_element(),
                420.0,
            ));
        let charts_misc = row()
            .justify_around()
            .child(RadarChart::new(
                ["Speed", "Power", "Range", "Armor", "Stealth"],
                [
                    Series::new("Unit A", [8.0, 6.0, 7.0, 4.0, 9.0]),
                    Series::new("Unit B", [5.0, 9.0, 4.0, 8.0, 3.0]),
                ],
            ))
            .child(
                div().flex_1().min_w(px(300.0)).child(ScatterPlot::new(
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
            .child(heat_block);

        let charts_more1 = row()
            .child(w(
                StackedAreaChart::new([
                    Series::new("Web", [12.0, 18.0, 14.0, 26.0, 22.0, 31.0]),
                    Series::new("App", [8.0, 11.0, 13.0, 15.0, 17.0, 16.0]),
                    Series::new("API", [4.0, 6.0, 9.0, 8.0, 12.0, 15.0]),
                ])
                .labels(months)
                .title("Traffic by source")
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Histogram::new(
                    "hist",
                    (0..120).map(|i| ((i * 37 % 100) as f32 / 10.0 + (i % 7) as f32).min(18.0)),
                )
                .bins(9)
                .title("Latency distribution")
                .format(|v| format!("{v:.0}ms"))
                .into_any_element(),
                420.0,
            ))
            .child(w(
                Waterfall::new(
                    "wf",
                    [
                        ("Start", 40.0),
                        ("Sales", 25.0),
                        ("Refunds", -12.0),
                        ("Ads", -8.0),
                        ("Upsell", 15.0),
                    ],
                )
                .title("Revenue bridge")
                .format(|v| format!("${v:.0}k"))
                .into_any_element(),
                420.0,
            ));
        let charts_more2 = row()
            .child(w(
                Funnel::new([
                    ("Visits", 1000.0),
                    ("Signups", 620.0),
                    ("Trials", 340.0),
                    ("Paid", 120.0),
                ])
                .title("Conversion funnel")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                Treemap::new(
                    "tm",
                    [
                        ("Rust", 52.0),
                        ("Kotlin", 28.0),
                        ("TS", 18.0),
                        ("Go", 12.0),
                        ("C", 9.0),
                        ("Other", 5.0),
                    ],
                )
                .title("Code by language")
                .into_any_element(),
                380.0,
            ))
            .child(
                RadialBar::new([
                    ("CPU", 72.0, 100.0),
                    ("RAM", 48.0, 100.0),
                    ("Disk", 31.0, 100.0),
                ])
                .into_any_element(),
            );
        let charts_more3 = row()
            .child(w(
                Candlestick::new(
                    "ohlc",
                    [
                        Candle::new(10.0, 14.0, 9.0, 13.0),
                        Candle::new(13.0, 15.0, 11.0, 12.0),
                        Candle::new(12.0, 16.0, 11.5, 15.5),
                        Candle::new(15.5, 17.0, 13.0, 14.0),
                        Candle::new(14.0, 18.0, 13.5, 17.5),
                        Candle::new(17.5, 19.0, 15.0, 16.0),
                    ],
                )
                .labels(months)
                .title("OHLC")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                BoxPlot::new([
                    ("A", vec![2.0, 3.0, 3.5, 4.0, 5.0, 6.0, 9.0]),
                    ("B", vec![4.0, 5.0, 6.0, 6.5, 7.0, 8.0, 12.0]),
                    ("C", vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5]),
                ])
                .title("Box plot")
                .into_any_element(),
                380.0,
            ))
            .child(w(
                BubbleChart::new(
                    "bub",
                    [
                        (
                            "EU",
                            vec![(1.0, 2.0, 30.0), (3.0, 5.0, 80.0), (5.0, 3.0, 50.0)],
                        ),
                        (
                            "US",
                            vec![(2.0, 6.0, 100.0), (4.0, 2.0, 20.0), (6.0, 7.0, 60.0)],
                        ),
                    ],
                )
                .title("Bubble chart")
                .into_any_element(),
                380.0,
            ));

        ChartDemos {
            charts_xy,
            gauge_styles,
            instruments,
            pies,
            donuts,
            sparks,
            charts_market,
            charts_misc,
            charts_more1,
            charts_more2,
            charts_more3,
        }
    }
}
