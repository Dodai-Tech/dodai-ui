//! Chart demo tiles (line/bar, pies, donuts, gauges, sparklines, instruments, ...).

use super::*;
use gpui::Div;

/// The chart sections' bodies, built once per render.
pub(crate) struct ChartDemos {
    /// Every chart demo as `(section, body)`, in build order; a section may get several
    /// bodies. `catalog::CHART_FAMILIES` decides where each section is listed.
    pub charts: Vec<(&'static str, gpui::AnyElement)>,
    pub gauge_styles: Div,
    /// Instrument demos: (signal lab, sci-fi HUD).
    pub instruments: (Div, Div),
    pub pulse: Div,
}

impl Showcase {
    pub(crate) fn chart_demos(&mut self, me: &Entity<Self>, cx: &mut Context<Self>) -> ChartDemos {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let me = me.clone();
        self.inst_seen = Some(std::time::Instant::now());
        // Live demo data: values wander and streams slide on the animation clock (they pause
        // with it and follow its speed).
        let time = dodai_gpui::motion::time();
        let drift = dodai_gpui::live::drift;
        // One positive value drifting up to ±25 %.
        let dv = |v: f32, key: f32| dodai_gpui::live::wander_between(key, v * 0.75, v * 1.25);
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
                                dodai_gpui_charts::Gradient::ALL
                                    [self.heat_grad % dodai_gpui_charts::Gradient::ALL.len()]
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
                    (0..hr)
                        .map(|r| {
                            (0..hc)
                                .map(|c| {
                                    let (x, y) = (c as f32 / hres as f32, r as f32 / hres as f32);
                                    ((x * 0.21 * 2.0).sin()
                                        + (y * 0.45).cos()
                                        + ((x * 2.0 + y) * 0.13 + time * 0.8).sin() * 1.5)
                                        + ((x - 16.0).hypot((y - 6.0) * 2.0) * -0.16).exp() * 3.0
                                })
                                .collect()
                        })
                        .collect(),
                )
                .cell(heat_cell)
                .gradient(
                    dodai_gpui_charts::Gradient::ALL
                        [self.heat_grad % dodai_gpui_charts::Gradient::ALL.len()],
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
        let a = Series::new("Revenue", drift(&[12.0, 18.0, 14.0, 26.0, 22.0, 31.0], 1.0));
        let b = Series::new("Cost", drift(&[8.0, 11.0, 13.0, 15.0, 17.0, 16.0], 2.0));
        let gauge_styles = self.gauge_style_demos(cx);
        let instruments = self.instrument_demos(&me, cx);
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
        let langs = || {
            [
                ("Rust", dv(52.0, 61.0)),
                ("Kotlin", dv(28.0, 62.0)),
                ("Other", dv(20.0, 63.0)),
            ]
        };
        let quarters = || {
            [
                ("Q1", dv(30.0, 64.0)),
                ("Q2", dv(24.0, 65.0)),
                ("Q3", dv(26.0, 66.0)),
                ("Q4", dv(20.0, 67.0)),
            ]
        };
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
                    ("A", dv(40.0, 68.0)),
                    ("B", dv(25.0, 69.0)),
                    ("C", dv(18.0, 70.0)),
                    ("D", dv(11.0, 71.0)),
                    ("E", dv(6.0, 72.0)),
                ])
                .colors(mono(t.palette.primary))
                .legend(false)
                .into_any_element(),
            ))
            .child(tile(
                "Rose",
                PieChart::new([
                    ("N", dv(9.0, 73.0)),
                    ("NE", dv(5.0, 74.0)),
                    ("E", dv(7.0, 75.0)),
                    ("SE", dv(3.0, 76.0)),
                    ("S", dv(8.0, 77.0)),
                    ("SW", dv(4.0, 78.0)),
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
                PieChart::new([
                    ("A", dv(3.0, 79.0)),
                    ("B", dv(2.0, 80.0)),
                    ("C", dv(1.0, 81.0)),
                ])
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
                PieChart::new([("Used", dv(68.0, 82.0)), ("Free", dv(32.0, 83.0))])
                    .style(PieStyle::Segmented)
                    .arc(0.75, 0.5)
                    .size(200.0)
                    .center("68%")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "HUD arc",
                PieChart::new([
                    ("A", dv(45.0, 84.0)),
                    ("B", dv(30.0, 85.0)),
                    ("C", dv(25.0, 86.0)),
                ])
                .style(PieStyle::Hud)
                .arc(0.625, 0.75)
                .size(190.0)
                .center("A·B·C")
                .legend(false)
                .into_any_element(),
            ))
            .child(tile(
                "Half donut",
                PieChart::new([("Used", dv(68.0, 87.0)), ("Free", dv(32.0, 88.0))])
                    .donut(0.7)
                    .arc(0.75, 0.5)
                    .size(200.0)
                    .center("68%")
                    .legend(false)
                    .into_any_element(),
            ))
            .child(tile(
                "Three-quarter",
                PieChart::new([("Done", dv(72.0, 89.0)), ("Left", dv(28.0, 90.0))])
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
                    ("A", dv(40.0, 91.0)),
                    ("B", dv(25.0, 92.0)),
                    ("C", dv(18.0, 93.0)),
                    ("D", dv(11.0, 94.0)),
                    ("E", dv(6.0, 95.0)),
                ])
                .donut(0.6)
                .colors(mono(t.palette.secondary))
                .legend(false)
                .into_any_element(),
            ))
            .child(tile(
                "Rose (area scale)",
                RoseChart::new([
                    ("N", dv(8.0, 96.0)),
                    ("NE", dv(14.0, 97.0)),
                    ("E", dv(22.0, 98.0)),
                    ("SE", dv(11.0, 99.0)),
                    ("S", dv(17.0, 100.0)),
                    ("SW", dv(25.0, 101.0)),
                    ("W", dv(9.0, 102.0)),
                    ("NW", dv(12.0, 103.0)),
                ])
                .size(200.0)
                .hole(0.15)
                .legend(false)
                .into_any_element(),
            ));
        let series = move || {
            drift(
                &[
                    3.0, 5.0, 4.0, 8.0, 6.0, 9.0, 7.0, 12.0, 10.0, 14.0, 9.0, 13.0,
                ],
                90.0,
            )
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
                    .child(spark("Smooth", Sparkline::new(series()).smooth(8)))
                    .child(spark(
                        "Diamond tip",
                        Sparkline::new(series()).marker(Marker::Diamond),
                    ))
                    .child(spark(
                        "Peak guide",
                        Sparkline::new(series()).guide(SparkGuide::Peak),
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
                    .gap_6()
                    .child(spark(
                        "Live stream",
                        Sparkline::new([]).live(&self.live_a).smooth(4),
                    ))
                    .child(
                        div().flex_1().min_w(px(240.0)).child(
                            BarChart::new([Series::live("Load", &self.live_a)])
                                .max(100.0)
                                .height(110.0)
                                .legend(false),
                        ),
                    )
                    .child(
                        div().flex_1().min_w(px(240.0)).child(
                            LineChart::new([Series::live("Load", &self.live_b)])
                                .area(true)
                                .smooth(true)
                                .y_range(0.0, 100.0)
                                .height(110.0)
                                .legend(false),
                        ),
                    ),
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
        let walk: Vec<f32> = (0..70)
            .scan(100.0f32, |p, i| {
                *p += ((i * 53 % 17) as f32 - 7.5) * 0.45;
                Some(*p)
            })
            .collect();
        // Live: the price series slides, so candles, bricks and the stack all move.
        let walk = drift(&walk, 6.0);
        let ohlc: Vec<Candle> = walk
            .windows(2)
            .enumerate()
            .map(|(i, p)| {
                let (o, c) = (p[0], p[1]);
                Candle::new(o, o.max(c) + 0.8, o.min(c) - 0.8, c)
                    .volume(50.0 + (i * 37 % 60) as f32)
            })
            .collect();

        let days: Vec<f32> = (0..182)
            .map(|i| {
                let x = (i as f32 * 0.37).sin() * (i as f32 * 0.11).cos();
                if i % 9 == 4 {
                    0.0
                } else {
                    (x * 0.5 + 0.5) * 12.0
                }
            })
            .collect();
        let samples = |seed: f32, mean: f32, spread: f32| -> Vec<f32> {
            (0..60)
                .map(|i| {
                    let u = ((i as f32 * 12.9898 + seed * 78.233).sin() * 43758.545)
                        .fract()
                        .abs();
                    let v = ((i as f32 * 4.1414 + seed * 9.31).sin() * 24634.63)
                        .fract()
                        .abs();
                    mean + (u + v - 1.0) * spread
                })
                .collect()
        };
        let pulse = {
            use dodai_gpui_charts::{Pulse, PulseGrid, PulseMode, PulseRhythm};
            let cfg = self.pulse_cfg;
            let pick =
                |id: &'static str, k: usize, label: &'static str, opts: &'static [&'static str]| {
                    let m = me.clone();
                    row()
                        .child(div().w(px(90.0)).child(Label::new(label)))
                        .child(
                            Segmented::new(id, opts.iter().copied())
                                .selected(cfg[k])
                                .on_select(move |i, _, cx| {
                                    m.update(cx, |s, cx| {
                                        s.pulse_cfg[k] = i;
                                        cx.notify();
                                    })
                                }),
                        )
                };
            let rhythm = [
                PulseRhythm::Normal,
                PulseRhythm::AFib,
                PulseRhythm::StElevation,
                PulseRhythm::VFib,
                PulseRhythm::Asystole,
            ][cfg[0].min(4)];
            let mode = [PulseMode::Sweep, PulseMode::Scroll, PulseMode::Static][cfg[1].min(2)];
            let grid = [
                PulseGrid::Paper,
                PulseGrid::Lines,
                PulseGrid::Dots,
                PulseGrid::None,
            ][cfg[2].min(3)];
            let noise = [0.0, 0.03, 0.09][cfg[3].min(2)];
            let rate = [48.0, 72.0, 132.0][cfg[4].min(2)];
            let trail = [0.0, 0.4, 0.8][cfg[5].min(2)];
            let gain = [0.6, 1.0, 1.5][cfg[6].min(2)];
            let controls = col()
                .child(pick(
                    "pc-rhythm",
                    0,
                    "Rhythm",
                    &["Normal", "AFib", "ST elev.", "VFib", "Asystole"],
                ))
                .child(pick("pc-mode", 1, "Mode", &["Sweep", "Scroll", "Static"]))
                .child(pick(
                    "pc-grid",
                    2,
                    "Grid",
                    &["Paper", "Lines", "Dots", "None"],
                ))
                .child(pick("pc-noise", 3, "Noise", &["None", "Light", "Heavy"]))
                .child(pick("pc-rate", 4, "Rate", &["48 bpm", "72 bpm", "132 bpm"]))
                .child(pick("pc-trail", 5, "Trail", &["Off", "Soft", "Long"]))
                .child(pick("pc-gain", 6, "Gain", &["0.6×", "1×", "1.5×"]));
            let main = Pulse::ecg(rate)
                .rhythm(rhythm)
                .mode(mode)
                .grid(grid)
                .noise(noise)
                .trail(trail)
                .gain(gain)
                .variability(if matches!(rhythm, PulseRhythm::AFib) {
                    1.0
                } else {
                    0.08
                })
                .label("ECG · II")
                .show_rate(true)
                .alarm(50.0, 110.0)
                .beat_led(true)
                .markers(true)
                .calibration(true)
                .wander(0.05)
                .footer("25 mm/s · 10 mm/mV · alarm 50–110")
                .size(640.0, 200.0);
            col().child(controls).child(main).child(
                row()
                    .child(
                        Pulse::pleth(72.0)
                            .label("SpO₂ · Pleth")
                            .readout("98", "%")
                            .grid(PulseGrid::Lines)
                            .fill(true)
                            .mode(PulseMode::Scroll)
                            .size(310.0, 110.0),
                    )
                    .child(
                        Pulse::art(72.0)
                            .color(t.palette.tone(Tone::Success))
                            .label("ABP")
                            .readout("120/80", "mmHg")
                            .grid(PulseGrid::Dots)
                            .size(310.0, 110.0),
                    )
                    .child(
                        Pulse::resp(16.0)
                            .label("Resp")
                            .show_rate(true)
                            .grid(PulseGrid::None)
                            .mode(PulseMode::Scroll)
                            .seconds(12.0)
                            .size(310.0, 110.0),
                    )
                    .child(
                        Pulse::eeg()
                            .label("EEG · Fp1")
                            .grid(PulseGrid::Lines)
                            .trail(0.3)
                            .size(310.0, 110.0),
                    ),
            )
        };

        let charts: Vec<(&'static str, gpui::AnyElement)> = vec![
            ("Pie", pies.into_any_element()),
            ("Donut", donuts.into_any_element()),
            ("Sparkline", sparks.into_any_element()),
            (
                "Line · area",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(
                        LineChart::new([a.clone(), b.clone()])
                            .labels(months)
                            .toggleable(true)
                            .patterns(true)
                            .area(true)
                            .smooth(true)
                            .crosshair(true)
                            .annotations([
                                Annotation::hline(20.0).label("Target"),
                                Annotation::x_band(2.0, 3.0).label("Launch"),
                            ]),
                    )
                    .into_any_element(),
            ),
            (
                "Bar",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(
                        BarChart::new([a.clone(), b.clone()])
                            .labels(months)
                            .annotations([
                                Annotation::y_band(10.0, 20.0).label("Plan"),
                                Annotation::hline(25.0).label("Cap"),
                            ]),
                    )
                    .into_any_element(),
            ),
            (
                "Bar",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(
                        BarChart::new([a.clone(), b.clone()])
                            .labels(months)
                            .stacked(true),
                    )
                    .into_any_element(),
            ),
            (
                "Bar",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(BarChart::new([a, b]).labels(months).horizontal(true))
                    .into_any_element(),
            ),
            (
                "Time axis",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(
                        LineChart::new([Series::new(
                            "Requests",
                            drift(&[4.0, 9.0, 7.0, 15.0, 22.0, 18.0, 25.0, 12.0], 5.0),
                        )])
                        .time_axis(
                            // Uneven sampling, 2024-03-04 06:00 UTC onwards: gaps show as gaps.
                            [0.0, 1.0, 1.5, 4.0, 5.0, 9.0, 11.0, 15.0]
                                .map(|h| 1_709_532_000.0 + h * 3600.0),
                        )
                        .crosshair(true)
                        .area(true),
                    )
                    .into_any_element(),
            ),
            (
                "Time axis",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(
                        LineChart::new([Series::new(
                            "Visits",
                            drift(&[3.0, 4.0, 6.0, 5.0, 9.0, 12.0, 11.0, 14.0], 6.0),
                        )])
                        .time_axis(
                            [0.0, 31.0, 90.0, 140.0, 200.0, 280.0, 340.0, 420.0]
                                .map(|d| 1_704_067_200.0 + d * 86_400.0),
                        )
                        .smooth(true),
                    )
                    .into_any_element(),
            ),
            (
                "Candlestick",
                Candlestick::new(ohlc.clone())
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
            ),
            (
                "Renko",
                RenkoChart::new(walk.clone(), 2.0)
                    .title("Renko · 2.0 bricks")
                    .into_any_element(),
            ),
            (
                "Order book depth",
                DepthChart::new(
                    (1..12).map(|i| {
                        (
                            100.0 - i as f32 * 0.5,
                            dv(4.0 + (i * 7 % 5) as f32, i as f32 * 0.7),
                        )
                    }),
                    (1..12).map(|i| {
                        (
                            100.5 + i as f32 * 0.5,
                            dv(3.0 + (i * 5 % 6) as f32, 4.0 + i as f32 * 0.7),
                        )
                    }),
                )
                .title("Order book depth")
                .into_any_element(),
            ),
            (
                "Gantt",
                Gantt::new([
                    GanttTask::new("Design", 0.0, 10.0).progress(1.0),
                    GanttTask::new("Build", 8.0, 24.0)
                        .progress(dv(0.6, 1.0).min(1.0))
                        .after(0),
                    GanttTask::new("Test", 20.0, 30.0)
                        .progress(dv(0.25, 2.0).min(1.0))
                        .after(1),
                    GanttTask::new("Ship", 30.0, 34.0).after(2),
                ])
                .title("Release plan")
                .format(|v| format!("d{v:.0}"))
                .into_any_element(),
            ),
            (
                "Secondary axis",
                LineChart::new([Series::new(
                    "Revenue",
                    drift(&[12.0, 18.0, 15.0, 26.0, 22.0, 31.0], 3.0),
                )])
                .secondary([Series::new(
                    "Margin %",
                    drift(&[0.21, 0.24, 0.2, 0.3, 0.28, 0.33], 4.0),
                )])
                .labels(months)
                .title("Secondary y axis")
                .into_any_element(),
            ),
            (
                "Chart stack",
                ChartStack::new()
                    .height(300.0)
                    .pane(3.0, move |h| {
                        LineChart::new([Series::new(
                            "Price",
                            drift(&[3.0, 4.0, 3.5, 5.0, 6.0, 5.5], 5.0),
                        )])
                        .height(h - 24.0)
                        .legend(false)
                        .into_any_element()
                    })
                    .pane(1.0, move |h| {
                        BarChart::new([Series::new(
                            "Vol",
                            drift(&[2.0, 3.0, 1.0, 4.0, 5.0, 3.0], 6.0),
                        )])
                        .height(h - 24.0)
                        .legend(false)
                        .into_any_element()
                    })
                    .into_any_element(),
            ),
            (
                "Radar",
                RadarChart::new(
                    ["Speed", "Power", "Range", "Armor", "Stealth"],
                    [
                        Series::new("Unit A", drift(&[8.0, 6.0, 7.0, 4.0, 9.0], 7.0)),
                        Series::new("Unit B", drift(&[5.0, 9.0, 4.0, 8.0, 3.0], 8.0)),
                    ],
                )
                .into_any_element(),
            ),
            (
                "Scatter",
                div()
                    .flex_1()
                    .min_w(px(300.0))
                    .child(ScatterPlot::new([
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
                    ]))
                    .into_any_element(),
            ),
            ("Heatmap", heat_block.into_any_element()),
            (
                "Stacked area",
                StackedAreaChart::new([
                    Series::new("Web", drift(&[12.0, 18.0, 14.0, 26.0, 22.0, 31.0], 9.0)),
                    Series::new("App", drift(&[8.0, 11.0, 13.0, 15.0, 17.0, 16.0], 10.0)),
                    Series::new("API", drift(&[4.0, 6.0, 9.0, 8.0, 12.0, 15.0], 11.0)),
                ])
                .labels(months)
                .title("Traffic by source")
                .into_any_element(),
            ),
            (
                "Histogram",
                Histogram::new(
                    (0..120).map(|i| ((i * 37 % 100) as f32 / 10.0 + (i % 7) as f32).min(18.0)),
                )
                .bins(9)
                .kde(true)
                .stats(true)
                .annotations([Annotation::hline(20.0).label("Alert")])
                .title("Latency distribution")
                .format(|v| format!("{v:.0}ms"))
                .into_any_element(),
            ),
            (
                "Waterfall",
                Waterfall::new([
                    ("Start", 40.0),
                    ("Sales", 25.0),
                    ("Refunds", -12.0),
                    ("Ads", -8.0),
                    ("Upsell", 15.0),
                ])
                .title("Revenue bridge")
                .format(|v| format!("${v:.0}k"))
                .into_any_element(),
            ),
            (
                "Funnel",
                Funnel::new([
                    ("Visits", 1000.0),
                    ("Signups", 620.0),
                    ("Trials", 340.0),
                    ("Paid", 120.0),
                ])
                .title("Conversion funnel")
                .into_any_element(),
            ),
            (
                "Treemap",
                Treemap::new([
                    ("Rust", 52.0),
                    ("Kotlin", 28.0),
                    ("TS", 18.0),
                    ("Go", 12.0),
                    ("C", 9.0),
                    ("Other", 5.0),
                ])
                .title("Code by language")
                .into_any_element(),
            ),
            (
                "Radial bar",
                RadialBar::new([
                    ("CPU", 72.0, 100.0),
                    ("RAM", 48.0, 100.0),
                    ("Disk", 31.0, 100.0),
                ])
                .into_any_element(),
            ),
            (
                "Bullet",
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(
                        Bullet::new("Revenue", 270.0)
                            .target(300.0)
                            .ranges([150.0, 225.0, 300.0]),
                    )
                    .child(
                        Bullet::new("Profit", 22.0)
                            .target(26.0)
                            .ranges([15.0, 20.0, 30.0]),
                    )
                    .child(
                        Bullet::new("Orders", 1520.0)
                            .target(1400.0)
                            .ranges([800.0, 1200.0, 1800.0]),
                    )
                    .into_any_element(),
            ),
            (
                "Paired bars",
                PairedBars::new([
                    ("Latency ms", 120.0, 84.0),
                    ("Errors", 42.0, 51.0),
                    ("Users k", 8.0, 11.5),
                ])
                .names("Q1", "Q2")
                .title("Paired bars")
                .into_any_element(),
            ),
            (
                "Lollipop · Dumbbell",
                Lollipop::new([
                    ("Mon", 12.0),
                    ("Tue", 18.0),
                    ("Wed", 9.0),
                    ("Thu", 24.0),
                    ("Fri", 15.0),
                ])
                .title("Lollipop")
                .into_any_element(),
            ),
            (
                "Lollipop · Dumbbell",
                Lollipop::new([
                    ("North", 12.0),
                    ("South", 18.0),
                    ("East", 9.0),
                    ("West", 24.0),
                ])
                .compare([17.0, 14.0, 16.0, 21.0])
                .names("2023", "2024")
                .title("Dumbbell")
                .into_any_element(),
            ),
            (
                "Slope",
                SlopeChart::new([
                    ("Alpha", 12.0, 18.0),
                    ("Beta", 20.0, 15.0),
                    ("Gamma", 9.0, 9.0),
                    ("Delta", 14.0, 22.0),
                ])
                .headers("2023", "2024")
                .title("Slope")
                .into_any_element(),
            ),
            (
                "Sankey",
                Sankey::new([
                    ("Wages", "Budget", 50.0),
                    ("Grants", "Budget", 20.0),
                    ("Sales", "Budget", 30.0),
                    ("Budget", "Staff", 45.0),
                    ("Budget", "Ops", 35.0),
                    ("Budget", "R&D", 20.0),
                    ("Staff", "Payroll", 45.0),
                    ("Ops", "Rent", 15.0),
                    ("Ops", "Tools", 20.0),
                ])
                .title("Sankey")
                .into_any_element(),
            ),
            (
                "Calendar heatmap",
                CalendarHeatmap::new(days)
                    .from_date(2024, 1, 1)
                    .title("Calendar heatmap")
                    .into_any_element(),
            ),
            (
                "Violin",
                ViolinChart::new([
                    ("A", samples(1.0, 10.0, 8.0)),
                    ("B", samples(2.0, 14.0, 4.0)),
                    ("C", samples(3.0, 8.0, 10.0)),
                ])
                .title("Violin")
                .into_any_element(),
            ),
            (
                "Candlestick",
                Candlestick::new([
                    Candle::new(10.0, 14.0, 9.0, 13.0),
                    Candle::new(13.0, 15.0, 11.0, 12.0),
                    Candle::new(12.0, 16.0, 11.5, 15.5),
                    Candle::new(15.5, 17.0, 13.0, 14.0),
                    Candle::new(14.0, 18.0, 13.5, 17.5),
                    Candle::new(17.5, 19.0, 15.0, 16.0),
                ])
                .labels(months)
                .title("OHLC")
                .into_any_element(),
            ),
            (
                "Box plot",
                BoxPlot::new([
                    ("A", vec![2.0, 3.0, 3.5, 4.0, 5.0, 6.0, 9.0]),
                    ("B", vec![4.0, 5.0, 6.0, 6.5, 7.0, 8.0, 12.0]),
                    ("C", vec![1.0, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5]),
                ])
                .notch(true)
                .mean(true)
                .points(true)
                .title("Box plot")
                .into_any_element(),
            ),
        ];
        ChartDemos {
            charts,
            gauge_styles,
            instruments,
            pulse,
        }
    }
}
