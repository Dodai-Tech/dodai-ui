//! Demos for the newer chart families: hierarchy and share, spread, relationship, flow,
//! survey, process and KPI charts.

use super::super::*;
use dodai_gpui::{
    ArcDiagram, Beeswarm, ChartNode, ChordDiagram, ControlChart, DivergingBars, Ecdf, ErrorBars,
    EventTimeline, Hexbin, Icicle, KpiTile, Marimekko, NetworkGraph, ParallelCoordinates,
    ParetoChart, StateSpan, Sunburst, Waffle,
};

/// A repeatable pseudo-random number in `0..1` for sample `i` of stream `seed`.
fn noise(seed: u32, i: u32) -> f32 {
    let mut x = seed.wrapping_mul(0x9E37_79B9) ^ i.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65_535.0
}

/// A rough bell curve sample around `mean` (sum of three uniforms).
fn bell(seed: u32, i: u32, mean: f32, spread: f32) -> f32 {
    let s = noise(seed, i * 3) + noise(seed, i * 3 + 1) + noise(seed, i * 3 + 2);
    mean + (s - 1.5) * spread
}

/// A note above a demo chart.
fn noted(n: &'static str, el: gpui::AnyElement) -> gpui::AnyElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .w_full()
        .child(Muted::new(n))
        .child(el)
        .into_any_element()
}

impl Showcase {
    /// The newer chart demos as `(section, body)`.
    pub(crate) fn more_chart_demos(&self) -> Vec<(&'static str, gpui::AnyElement)> {
        let drift = dodai_gpui::live::drift;
        let budget = ChartNode::branch(
            "Budget",
            [
                ChartNode::branch(
                    "Product",
                    [
                        ChartNode::leaf("Design", 18.0),
                        ChartNode::leaf("Engineering", 46.0),
                        ChartNode::leaf("Research", 12.0),
                    ],
                ),
                ChartNode::branch(
                    "Go to market",
                    [
                        ChartNode::leaf("Sales", 22.0),
                        ChartNode::leaf("Marketing", 15.0),
                    ],
                ),
                ChartNode::branch(
                    "Operations",
                    [
                        ChartNode::leaf("Support", 9.0),
                        ChartNode::branch(
                            "Infra",
                            [
                                ChartNode::leaf("Cloud", 11.0),
                                ChartNode::leaf("Office", 5.0),
                            ],
                        ),
                    ],
                ),
            ],
        );
        let crew = [
            "Ada", "Bo", "Cy", "Dee", "Eli", "Fay", "Gus", "Hal", "Ivy", "Jo",
        ];
        let links = vec![
            (0, 1, 3.0),
            (0, 2, 2.0),
            (1, 2, 2.0),
            (2, 3, 1.0),
            (3, 4, 3.0),
            (4, 5, 2.0),
            (3, 5, 2.0),
            (5, 6, 1.0),
            (6, 7, 3.0),
            (7, 8, 2.0),
            (6, 8, 2.0),
            (8, 9, 1.0),
            (0, 9, 1.0),
        ];
        let hours = |a: f32, b: f32, s: &'static str| StateSpan::new(a, b, s);
        vec![
            (
                "Sunburst",
                noted(
                    "A budget as nested rings: inner ring by area, outer by team.",
                    Sunburst::new(budget.clone()).format(|v| format!("{v:.0}k")).into_any_element(),
                ),
            ),
            (
                "Icicle",
                noted(
                    "The same budget as rows, top level first.",
                    Icicle::new(budget).format(|v| format!("{v:.0}k")).into_any_element(),
                ),
            ),
            (
                "Waffle",
                noted(
                    "100 cells, one per percent of traffic.",
                    Waffle::new([
                        ("Mobile", 58.0),
                        ("Desktop", 31.0),
                        ("Tablet", 8.0),
                        ("Other", 3.0),
                    ])
                    .into_any_element(),
                ),
            ),
            (
                "Marimekko",
                noted(
                    "Column width is a region's revenue, the split its product mix.",
                    Marimekko::new(
                        ["Hardware", "Software", "Services"],
                        [
                            ("North", vec![40.0, 25.0, 15.0]),
                            ("South", vec![12.0, 30.0, 8.0]),
                            ("East", vec![20.0, 10.0, 25.0]),
                            ("West", vec![8.0, 12.0, 6.0]),
                        ],
                    )
                    .format(|v| format!("{v:.0}k"))
                    .into_any_element(),
                ),
            ),
            (
                "Beeswarm",
                noted(
                    "Every response time, per endpoint.",
                    Beeswarm::new(
                        [("/login", 140.0, 30.0), ("/search", 210.0, 70.0), ("/pay", 320.0, 45.0)]
                            .into_iter()
                            .enumerate()
                            .map(|(k, (n, m, s))| (n, (0..40).map(|i| bell(k as u32 + 1, i, m, s)).collect())),
                    )
                    .format(|v| format!("{v:.0} ms"))
                    .into_any_element(),
                ),
            ),
            (
                "ECDF",
                noted(
                    "Share of requests at or under each latency; the lines mark 50 % and 90 %.",
                    Ecdf::new([
                        Series::new("Before", (0..80).map(|i| bell(7, i, 220.0, 120.0)).collect::<Vec<_>>()),
                        Series::new("After", (0..80).map(|i| bell(8, i, 160.0, 70.0)).collect::<Vec<_>>()),
                    ])
                    .format(|v| format!("{v:.0}"))
                    .into_any_element(),
                ),
            ),
            (
                "Error bars",
                noted(
                    "Effect of each change with its 95 % interval; the line is no effect.",
                    ErrorBars::new([
                        ("Bigger button", 4.2, 1.1, 7.3),
                        ("New copy", 1.0, -2.4, 4.1),
                        ("Shorter form", 6.8, 3.9, 9.6),
                        ("Dark hero", -1.6, -4.0, 0.9),
                        ("Free trial", 9.1, 5.0, 13.4),
                    ])
                    .reference(0.0)
                    .format(|v| format!("{v:+.1}%"))
                    .into_any_element(),
                ),
            ),
            (
                "Parallel coordinates",
                noted(
                    "Five cars across four measures; each axis has its own scale.",
                    ParallelCoordinates::new(
                        ["Power", "Weight", "Range", "Price"],
                        [
                            ("Coupe", vec![420.0, 1450.0, 520.0, 98.0]),
                            ("Estate", vec![250.0, 1720.0, 780.0, 54.0]),
                            ("City", vec![110.0, 1080.0, 390.0, 22.0]),
                            ("SUV", vec![340.0, 2150.0, 610.0, 76.0]),
                            ("Roadster", vec![300.0, 1250.0, 450.0, 64.0]),
                        ],
                    )
                    .format(|v| format!("{v:.0}"))
                    .into_any_element(),
                ),
            ),
            (
                "Hexbin",
                noted(
                    "1 500 points binned into hexagons; darker holds more.",
                    Hexbin::new((0..1500u32).map(|i| {
                        let x = bell(11, i, 50.0, 60.0);
                        (x, 0.6 * x + bell(12, i, 10.0, 40.0))
                    }))
                    .format(|v| format!("{v:.0}"))
                    .into_any_element(),
                ),
            ),
            (
                "Network graph",
                noted(
                    "Who works with whom: three teams joined by a few people.",
                    NetworkGraph::new(crew, links.clone())
                        .groups([0, 0, 0, 1, 1, 1, 2, 2, 2, 2])
                        .into_any_element(),
                ),
            ),
            (
                "Chord",
                noted(
                    "Customers moving between plans over a year.",
                    ChordDiagram::new(
                        ["Free", "Basic", "Pro", "Team"],
                        vec![
                            vec![30.0, 18.0, 6.0, 1.0],
                            vec![9.0, 40.0, 14.0, 3.0],
                            vec![2.0, 7.0, 35.0, 9.0],
                            vec![0.0, 1.0, 4.0, 20.0],
                        ],
                    )
                    .into_any_element(),
                ),
            ),
            (
                "Arc diagram",
                noted(
                    "The same team links with everyone in a row.",
                    ArcDiagram::new(crew, links).into_any_element(),
                ),
            ),
            (
                "Pareto",
                noted(
                    "Support tickets by cause; the first few make 80 % of the work.",
                    ParetoChart::new([
                        ("Login", 120.0),
                        ("Billing", 64.0),
                        ("Sync", 210.0),
                        ("Export", 18.0),
                        ("Crash", 41.0),
                        ("Other", 22.0),
                        ("Search", 9.0),
                    ])
                    .into_any_element(),
                ),
            ),
            (
                "Diverging bars",
                noted(
                    "Survey answers around neutral; split 2.5 puts neutral across the centre.",
                    DivergingBars::new(
                        ["Strongly disagree", "Disagree", "Neutral", "Agree", "Strongly agree"],
                        [
                            ("Easy to start", vec![4.0, 9.0, 18.0, 41.0, 28.0]),
                            ("Fast enough", vec![8.0, 17.0, 25.0, 33.0, 17.0]),
                            ("Fair price", vec![19.0, 26.0, 22.0, 24.0, 9.0]),
                            ("Would recommend", vec![3.0, 7.0, 15.0, 38.0, 37.0]),
                        ],
                    )
                    .into_any_element(),
                ),
            ),
            (
                "Diverging bars",
                noted(
                    "A population pyramid: two parts split at 1, raw counts.",
                    DivergingBars::new(
                        ["Men", "Women"],
                        ["70+", "60–69", "50–59", "40–49", "30–39", "20–29", "0–19"]
                            .into_iter()
                            .enumerate()
                            .map(|(i, a)| {
                                let k = i as f32;
                                (a, vec![4.0 + 2.2 * k - 0.12 * k * k, 5.5 + 2.0 * k - 0.13 * k * k])
                            }),
                    )
                    .split(1.0)
                    .percent(false)
                    .format(|v| format!("{v:.1}M"))
                    .into_any_element(),
                ),
            ),
            (
                "Control chart",
                noted(
                    "Fill weight per batch, limits from the first 20: one spike, then a shift from batch 23 that the run rule catches.",
                    ControlChart::new((0..36u32).map(|i| {
                        bell(21, i, if i >= 22 { 503.5 } else { 500.0 }, 3.0) + if i == 15 { 9.0 } else { 0.0 }
                    }))
                    .baseline(20)
                    .format(|v| format!("{v:.1} g"))
                    .into_any_element(),
                ),
            ),
            (
                "Event timeline",
                noted(
                    "Service states through a day (hours).",
                    EventTimeline::new([
                        ("api", vec![hours(0.0, 9.5, "Up"), hours(9.5, 10.2, "Degraded"), hours(10.2, 24.0, "Up")]),
                        ("db", vec![hours(0.0, 3.0, "Up"), hours(3.0, 3.5, "Maintenance"), hours(3.5, 24.0, "Up")]),
                        (
                            "search",
                            vec![
                                hours(0.0, 14.0, "Up"),
                                hours(14.0, 14.6, "Down"),
                                hours(14.6, 16.0, "Degraded"),
                                hours(16.0, 24.0, "Up"),
                            ],
                        ),
                        ("mail", vec![hours(0.0, 24.0, "Up")]),
                    ])
                    .format(|v| format!("{v:.1}h"))
                    .into_any_element(),
                ),
            ),
            (
                "KPI tile",
                div()
                    .flex()
                    .flex_wrap()
                    .gap_8()
                    .child(
                        KpiTile::new("Revenue", drift(&[128_400.0], 31.0)[0])
                            .previous(112_000.0)
                            .trend(drift(&[96.0, 101.0, 99.0, 108.0, 112.0, 118.0, 128.0], 32.0))
                            .format(|v| format!("${:.1}k", v / 1000.0)),
                    )
                    .child(
                        KpiTile::new("Error rate", 0.42)
                            .previous(0.61)
                            .lower_is_better(true)
                            .trend([0.7, 0.66, 0.61, 0.58, 0.5, 0.45, 0.42])
                            .format(|v| format!("{v:.2}%")),
                    )
                    .child(
                        KpiTile::new("Latency p95", 318.0)
                            .previous(276.0)
                            .lower_is_better(true)
                            .trend([260.0, 270.0, 276.0, 290.0, 301.0, 312.0, 318.0])
                            .format(|v| format!("{v:.0} ms")),
                    )
                    .child(KpiTile::new("Active users", 9_412.0).previous(9_405.0).format(|v| format!("{v:.0}")))
                    .into_any_element(),
            ),
        ]
    }
}
