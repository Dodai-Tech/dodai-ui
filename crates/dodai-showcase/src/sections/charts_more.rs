//! Demos for the newer chart families: hierarchy and share, spread, relationship, flow,
//! survey, process and KPI charts.

use super::super::*;
use dodai_gpui_charts::{
    ArcDiagram, Beeswarm, Bubble, BubbleSwarm, BubbleTimeline, ChartNode, ChordDiagram, CirclePack,
    ControlChart, DivergingBars, Ecdf, ErrorBars, EventTimeline, FanOrder, FanSpan, FlowStep,
    Hexbin, Icicle, KpiTile, Marimekko, Milestone, NetworkGraph, Outline, ParallelCoordinates,
    ParetoChart, PeakRange, ProportionalSquares, ShardGroup, ShardMap, SpanFan, SplitFlow,
    StateSpan, Sunburst, TimeScatter, Waffle,
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
                    Icicle::new(budget.clone()).format(|v| format!("{v:.0}k")).into_any_element(),
                ),
            ),
            (
                "Circle packing",
                noted(
                    "The same budget as circles inside circles: each team by area, each area a ring around its teams.",
                    CirclePack::new(budget).format(|v| format!("{v:.0}k")).into_any_element(),
                ),
            ),
            (
                "Proportional squares",
                noted(
                    "Yearly spend, one square per item: sizes three orders of magnitude apart stay comparable.",
                    ProportionalSquares::new([
                        (
                            "Public",
                            vec![
                                ("Health", drift(&[1_840.0], 41.0)[0]),
                                ("Pensions", 1_310.0),
                                ("Schools", 760.0),
                                ("Defence", 420.0),
                                ("Transport", 260.0),
                                ("Science", 95.0),
                                ("Arts", 18.0),
                            ],
                        ),
                        (
                            "Private",
                            vec![
                                ("Cloud", 980.0),
                                ("Phones", 640.0),
                                ("Games", 210.0),
                                ("Streaming", 130.0),
                                ("Pet food", 72.0),
                                ("Fonts", 4.0),
                            ],
                        ),
                    ])
                    .format(|v| format!("{v:.0}bn"))
                    .into_any_element(),
                ),
            ),
            (
                "Bubble swarm",
                noted(
                    "Ventures by yearly growth, sized by revenue: the leaders and the giants at a glance.",
                    BubbleSwarm::new([
                        (
                            "Energy",
                            vec![
                                ("Sol", (34.0, 820.0)),
                                ("Gridly", (12.0, 2_400.0)),
                                ("Tidal", (58.0, 140.0)),
                                ("Volt", (21.0, 610.0)),
                            ],
                        ),
                        (
                            "Health",
                            vec![
                                ("Pulse", (44.0, 390.0)),
                                ("Medi", (9.0, 3_100.0)),
                                ("Gene", (71.0, 90.0)),
                                ("Care", (26.0, 1_200.0)),
                                ("Dose", (18.0, 220.0)),
                            ],
                        ),
                        (
                            "Money",
                            vec![
                                ("Ledger", (15.0, 1_800.0)),
                                ("Coin", (63.0, 300.0)),
                                ("Pay", (37.0, 950.0)),
                                ("Fund", (29.0, 180.0)),
                                ("Vault", (5.0, 2_700.0)),
                            ],
                        ),
                    ])
                    .format(|v| format!("{v:.0}"))
                    .into_any_element(),
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
                "Span fan",
                noted(
                    "Empires: spoke length is how long each lasted (rings mark the centuries), the heavy stretch its peak, the dot and name its largest extent (million km², by area). Approximate figures.",
                    SpanFan::new(
                        [
                        ("Akkadian", -2334.0, -2154.0, (-2250.0, -2200.0), 0.7, "Near East"),
                        ("Egypt (New Kingdom)", -1550.0, -1077.0, (-1450.0, -1350.0), 1.0, "Africa"),
                        ("Neo-Assyrian", -911.0, -609.0, (-670.0, -640.0), 1.4, "Near East"),
                        ("Achaemenid", -550.0, -330.0, (-500.0, -480.0), 5.5, "Near East"),
                        ("Maurya", -322.0, -185.0, (-260.0, -232.0), 3.4, "South Asia"),
                        ("Seleucid", -312.0, -63.0, (-301.0, -240.0), 3.9, "Near East"),
                        ("Han", -202.0, 220.0, (-100.0, 0.0), 6.0, "East Asia"),
                        ("Parthian", -247.0, 224.0, (-50.0, 50.0), 2.8, "Near East"),
                        ("Roman", -27.0, 476.0, (98.0, 180.0), 5.0, "Europe"),
                        ("Sasanian", 224.0, 651.0, (580.0, 620.0), 3.5, "Near East"),
                        ("Gupta", 320.0, 550.0, (375.0, 450.0), 3.5, "South Asia"),
                        ("Byzantine", 330.0, 1453.0, (527.0, 565.0), 2.8, "Europe"),
                        ("Tang", 618.0, 907.0, (660.0, 715.0), 5.4, "East Asia"),
                        ("Umayyad", 661.0, 750.0, (720.0, 740.0), 11.1, "Near East"),
                        ("Abbasid", 750.0, 1258.0, (786.0, 861.0), 11.1, "Near East"),
                        ("Song", 960.0, 1279.0, (1000.0, 1100.0), 3.1, "East Asia"),
                        ("Seljuk", 1037.0, 1194.0, (1072.0, 1092.0), 3.9, "Near East"),
                        ("Mongol", 1206.0, 1368.0, (1260.0, 1294.0), 24.0, "Steppe"),
                        ("Golden Horde", 1240.0, 1502.0, (1300.0, 1360.0), 6.0, "Steppe"),
                        ("Ottoman", 1299.0, 1922.0, (1520.0, 1683.0), 5.2, "Near East"),
                        ("Ming", 1368.0, 1644.0, (1400.0, 1450.0), 6.5, "East Asia"),
                        ("Inca", 1438.0, 1533.0, (1493.0, 1527.0), 2.0, "Americas"),
                        ("Portuguese", 1415.0, 1999.0, (1550.0, 1600.0), 10.4, "Europe"),
                        ("Spanish", 1492.0, 1976.0, (1580.0, 1640.0), 13.7, "Europe"),
                        ("Mughal", 1526.0, 1857.0, (1650.0, 1707.0), 4.0, "South Asia"),
                        ("British", 1583.0, 1997.0, (1900.0, 1922.0), 35.5, "Europe"),
                        ("Qing", 1636.0, 1912.0, (1735.0, 1796.0), 14.7, "East Asia"),
                        ("Russian", 1721.0, 1917.0, (1866.0, 1895.0), 22.8, "Europe"),
                        ]
                        .into_iter()
                        .map(|(name, from, to, (p, q), area, region)| {
                            FanSpan::new(name, from, to).peak(p, q).magnitude(area).group(region)
                        }),
                    )
                    .order(FanOrder::Start)
                    .height(560.0)
                    .format(|v| if v < 0.0 { format!("{:.0} BCE", -v) } else { format!("{v:.0}") })
                    .magnitude_format(|v| format!("{v:.1}M km²"))
                    .into_any_element(),
                ),
            ),
            (
                "Time scatter",
                div()
                    .flex()
                    .flex_col()
                    .gap_8()
                    .w_full()
                    .child(noted(
                        "Suspension bridges: main span by year opened, bubble area the bridge's length, rings carry rail too; the line is the trend. Approximate figures.",
                        TimeScatter::new(
                            [
                            ("Menai", 1826.0, 176.0, 0.4, "Europe", false),
                            ("Clifton", 1864.0, 214.0, 0.4, "Europe", false),
                            ("Brooklyn", 1883.0, 486.0, 1.8, "Americas", false),
                            ("Williamsburg", 1903.0, 488.0, 2.2, "Americas", true),
                            ("Ambassador", 1929.0, 564.0, 2.3, "Americas", false),
                            ("George Washington", 1931.0, 1067.0, 1.4, "Americas", false),
                            ("Golden Gate", 1937.0, 1280.0, 2.7, "Americas", false),
                            ("Lions Gate", 1938.0, 473.0, 1.8, "Americas", false),
                            ("Tacoma Narrows", 1950.0, 853.0, 1.8, "Americas", false),
                            ("Mackinac", 1957.0, 1158.0, 8.0, "Americas", false),
                            ("Forth Road", 1964.0, 1006.0, 2.5, "Europe", false),
                            ("Verrazzano-Narrows", 1964.0, 1298.0, 4.2, "Americas", false),
                            ("25 de Abril", 1966.0, 1013.0, 2.3, "Europe", true),
                            ("Severn", 1966.0, 988.0, 1.6, "Europe", false),
                            ("Bosphorus", 1973.0, 1074.0, 1.6, "Asia", false),
                            ("Humber", 1981.0, 1410.0, 2.2, "Europe", false),
                            ("Tsing Ma", 1997.0, 1377.0, 2.2, "Asia", true),
                            ("Great Belt East", 1998.0, 1624.0, 6.8, "Europe", false),
                            ("Akashi Kaikyo", 1998.0, 1991.0, 3.9, "Asia", false),
                            ("Runyang", 2005.0, 1490.0, 4.7, "Asia", false),
                            ("Xihoumen", 2009.0, 1650.0, 2.6, "Asia", false),
                            ("Hardanger", 2013.0, 1310.0, 1.4, "Europe", false),
                            ("Yavuz Sultan Selim", 2016.0, 1408.0, 2.2, "Asia", true),
                            ("Yangsigang", 2019.0, 1700.0, 4.1, "Asia", false),
                            ("1915 Canakkale", 2022.0, 2023.0, 4.6, "Asia", false),
                            ]
                            .into_iter()
                            .map(|(name, year, span, km, region, rail)| {
                                Milestone::new(name, year, span).size(km).group(region).hollow(rail)
                            }),
                        )
                        .trend(true)
                        .reference(1000.0, "1 km span")
                        .reference(2000.0, "2 km span")
                        .height(440.0)
                        .x_format(|v| format!("{v:.0}"))
                        .format(|v| format!("{v:.0} m"))
                        .size_format(|v| format!("{v:.1} km long"))
                        .into_any_element(),
                    ))
                    .child(noted(
                        "Cost overrun of the Summer and Winter Games by host city (real terms, approximate), each season's cities joined in time order.",
                        TimeScatter::new(
                            [
                            ("Montreal", 1976.0, 720.0, "Summer"),
                            ("Barcelona", 1992.0, 266.0, "Summer"),
                            ("Atlanta", 1996.0, 151.0, "Summer"),
                            ("Sydney", 2000.0, 90.0, "Summer"),
                            ("Athens", 2004.0, 49.0, "Summer"),
                            ("Beijing", 2008.0, 2.0, "Summer"),
                            ("London", 2012.0, 76.0, "Summer"),
                            ("Rio de Janeiro", 2016.0, 352.0, "Summer"),
                            ("Lake Placid", 1980.0, 324.0, "Winter"),
                            ("Sarajevo", 1984.0, 118.0, "Winter"),
                            ("Calgary", 1988.0, 65.0, "Winter"),
                            ("Albertville", 1992.0, 137.0, "Winter"),
                            ("Lillehammer", 1994.0, 277.0, "Winter"),
                            ("Nagano", 1998.0, 56.0, "Winter"),
                            ("Salt Lake City", 2002.0, 24.0, "Winter"),
                            ("Turin", 2006.0, 80.0, "Winter"),
                            ("Vancouver", 2010.0, 13.0, "Winter"),
                            ("Sochi", 2014.0, 289.0, "Winter"),
                            ]
                            .into_iter()
                            .map(|(city, year, over, season)| Milestone::new(city, year, over).group(season)),
                        )
                        .trails(true)
                        .reference(0.0, "on budget")
                        .reference(100.0, "double the budget")
                        .height(360.0)
                        .x_format(|v| format!("{v:.0}"))
                        .format(|v| format!("{v:+.0}%"))
                        .into_any_element(),
                    ))
                    .into_any_element(),
            ),
            (
                "Bubble timeline",
                noted(
                    "Big volcanic eruptions since 1800: bubble area the volume thrown out (km³ of rock and ash, approximate), coloured by region. Click a region to hide it.",
                    BubbleTimeline::new([
                        Bubble::new("Tambora", 1815.3, 150.0).group("Indonesia").note("the year after went without a summer across much of the north"),
                        Bubble::new("Galunggung", 1822.8, 1.0).group("Indonesia"),
                        Bubble::new("Cosigüina", 1835.1, 5.0).group("Americas"),
                        Bubble::new("Shiveluch", 1854.1, 1.0).group("Pacific & Asia"),
                        Bubble::new("Askja", 1875.2, 2.0).group("Europe & Atlantic").note("drove a wave of emigration from Iceland"),
                        Bubble::new("Krakatau", 1883.6, 20.0).group("Indonesia").note("heard thousands of kilometres away"),
                        Bubble::new("Tarawera", 1886.4, 2.0).group("Pacific & Asia"),
                        Bubble::new("Santa María", 1902.8, 8.5).group("Americas"),
                        Bubble::new("Mont Pelée", 1902.3, 0.2).group("Americas").note("destroyed the town of Saint-Pierre"),
                        Bubble::new("Ksudach", 1907.2, 2.4).group("Pacific & Asia"),
                        Bubble::new("Novarupta", 1912.4, 13.5).group("Americas").note("the century's largest eruption"),
                        Bubble::new("Sakurajima", 1914.0, 1.5).group("Pacific & Asia").note("joined the island to the mainland"),
                        Bubble::new("Katla", 1918.8, 0.7).group("Europe & Atlantic"),
                        Bubble::new("Kelud", 1919.4, 0.2).group("Indonesia"),
                        Bubble::new("Quizapu", 1932.3, 9.5).group("Americas"),
                        Bubble::new("Hekla", 1947.2, 0.2).group("Europe & Atlantic"),
                        Bubble::new("Lamington", 1951.1, 1.0).group("Pacific & Asia"),
                        Bubble::new("Bezymianny", 1956.2, 1.0).group("Pacific & Asia"),
                        Bubble::new("Agung", 1963.2, 1.0).group("Indonesia"),
                        Bubble::new("St Helens", 1980.4, 1.2).group("Americas").note("the side of the mountain collapsed first"),
                        Bubble::new("El Chichón", 1982.3, 2.3).group("Americas"),
                        Bubble::new("Pinatubo", 1991.5, 10.0).group("Pacific & Asia").note("cooled the globe by about half a degree for a year"),
                        Bubble::new("Hudson", 1991.7, 4.3).group("Americas"),
                        Bubble::new("Rabaul", 1994.7, 0.3).group("Pacific & Asia"),
                        Bubble::new("Chaitén", 2008.3, 1.0).group("Americas"),
                        Bubble::new("Eyjafjallajökull", 2010.3, 0.27).group("Europe & Atlantic").note("grounded flights across Europe for days"),
                        Bubble::new("Merapi", 2010.8, 0.1).group("Indonesia"),
                        Bubble::new("Cordón Caulle", 2011.4, 1.0).group("Americas"),
                        Bubble::new("Kelud", 2014.1, 0.5).group("Indonesia"),
                        Bubble::new("Calbuco", 2015.3, 0.3).group("Americas"),
                        Bubble::new("Hunga Tonga", 2022.0, 6.5).group("Pacific & Asia").note("its blast wave circled the globe several times"),
                    ])
                    .size_label("km³ thrown out")
                    .x_format(|y| format!("{y:.0}"))
                    .format(|v| if v < 1.0 { format!("{v:.1} km³") } else { format!("{v:.0} km³") })
                    .max_radius(110.0)
                    .height(300.0)
                    .into_any_element(),
                ),
            ),
            (
                "Peak range",
                noted(
                    "Pollen season: a typical year's weekly airborne pollen by plant (illustrative, mid-latitude city).",
                    PeakRange::new(
                        [
                        ("Hazel", 40.0, 7.0, 2.5),
                        ("Alder", 70.0, 10.0, 2.5),
                        ("Ash", 90.0, 15.0, 2.5),
                        ("Birch", 180.0, 16.0, 2.0),
                        ("Oak", 110.0, 19.0, 2.5),
                        ("Grass", 220.0, 25.0, 4.0),
                        ("Plantain", 60.0, 27.0, 4.0),
                        ("Nettle", 120.0, 29.0, 5.0),
                        ("Mugwort", 80.0, 32.0, 2.5),
                        ("Ragweed", 140.0, 36.0, 2.5),
                        ]
                        .into_iter()
                        .map(|(name, top, week, width): (&str, f32, f32, f32)| {
                            Series::new(
                                name,
                                (0..52u32).map(|w| {
                                    let d = (w as f32 - week) / width;
                                    let wobble = 0.85 + 0.3 * noise(week as u32, w);
                                    top * (-0.5 * d * d).exp() * wobble
                                }),
                            )
                        }),
                    )
                    .x_labels(["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"])
                    .lanes(3)
                    .height(320.0)
                    .format(|v| format!("{v:.0} grains/m³"))
                    .into_any_element(),
                ),
            ),
            (
                "Shard map",
                noted(
                    "Languages by first-language speakers (millions, approximate), grouped by family.",
                    ShardMap::new([
                        ShardGroup::new("Indo-European", [("Spanish", 485.0), ("English", 380.0), ("Hindi", 345.0), ("Portuguese", 236.0), ("Bengali", 234.0), ("Russian", 148.0), ("Punjabi", 113.0), ("Marathi", 83.0), ("German", 75.0), ("French", 74.0), ("Urdu", 70.0), ("Italian", 64.0), ("Persian", 57.0), ("Gujarati", 57.0), ("Polish", 40.0), ("Ukrainian", 33.0)]),
                        ShardGroup::new("Sino-Tibetan", [("Mandarin", 940.0), ("Cantonese", 86.0), ("Wu", 83.0), ("Min Nan", 50.0), ("Jin", 47.0), ("Burmese", 33.0)]),
                        ShardGroup::new("Afro-Asiatic", [("Arabic", 380.0), ("Hausa", 50.0), ("Amharic", 33.0), ("Somali", 22.0)]),
                        ShardGroup::new("Dravidian", [("Telugu", 83.0), ("Tamil", 79.0), ("Kannada", 44.0), ("Malayalam", 37.0)]),
                        ShardGroup::new("Austronesian", [("Javanese", 68.0), ("Indonesian", 43.0), ("Sundanese", 32.0), ("Tagalog", 28.0), ("Malay", 19.0)]),
                        ShardGroup::new("Turkic", [("Turkish", 85.0), ("Uzbek", 32.0), ("Azerbaijani", 24.0)]),
                        ShardGroup::new("Japonic", [("Japanese", 123.0)]),
                        ShardGroup::new("Koreanic", [("Korean", 81.0)]),
                        ShardGroup::new("Austroasiatic", [("Vietnamese", 86.0), ("Khmer", 17.0)]),
                        ShardGroup::new("Niger-Congo", [("Yoruba", 46.0), ("Igbo", 31.0), ("Swahili", 18.0), ("Zulu", 12.0)]),
                    ])
                    .outline(Outline::Circle)
                    .height(600.0)
                    .format(|v| format!("{v:.0}M"))
                    .into_any_element(),
                ),
            ),
            (
                "Split flow",
                div()
                    .flex()
                    .flex_col()
                    .gap_8()
                    .w_full()
                    .child(noted(
                    "Where 100 litres of rain falling on a paved city go (a typical urban water balance, rounded).",
                    SplitFlow::new([
                        FlowStep::new("Rainfall", 100.0),
                        FlowStep::new("Runoff", 55.0).from("Rainfall").group("Over the ground"),
                        FlowStep::new("Evaporation", 30.0).from("Rainfall").group("Back to the air"),
                        FlowStep::new("Infiltration", 15.0).from("Rainfall").group("Into the ground"),
                        FlowStep::new("Storm drains", 40.0).from("Runoff").group("Over the ground"),
                        FlowStep::new("Streams", 15.0).from("Runoff").group("Over the ground"),
                        FlowStep::new("Shallow groundwater", 10.0).from("Infiltration").group("Into the ground"),
                        FlowStep::new("Deep aquifer", 5.0).from("Infiltration").group("Into the ground"),
                        FlowStep::new("Straight to rivers", 28.0).from("Storm drains").group("Over the ground"),
                        FlowStep::new("Treated", 12.0).from("Storm drains").group("Treated"),
                    ])
                    .height(520.0)
                    .format(|v| format!("{v:.0} L"))
                    .into_any_element(),
                ))
                .child(noted(
                    "The tall form, .max_width(560.0) with .height(820.0): where 100 units of fresh water drawn from rivers and aquifers go (illustrative, rounded global shares).",
                    SplitFlow::new([
                        FlowStep::new("Fresh water drawn", 100.0),
                        FlowStep::new("Farming", 70.0).from("Fresh water drawn").group("Farming"),
                        FlowStep::new("Industry", 19.0).from("Fresh water drawn").group("Industry"),
                        FlowStep::new("Homes", 11.0).from("Fresh water drawn").group("Homes"),
                        FlowStep::new("Taken up by crops", 40.0).from("Farming").group("Farming"),
                        FlowStep::new("Lost from canals", 15.0).from("Farming").group("Back to nature"),
                        FlowStep::new("Field runoff", 15.0).from("Farming").group("Back to nature"),
                        FlowStep::new("Cooling", 15.0).from("Industry").group("Industry"),
                        FlowStep::new("Used up", 4.0).from("Industry").group("Industry"),
                        FlowStep::new("Wastewater", 9.0).from("Homes").group("Homes"),
                        FlowStep::new("Drunk & cooked", 2.0).from("Homes").group("Homes"),
                        FlowStep::new("Returned warm", 15.0).from("Cooling").group("Back to nature"),
                        FlowStep::new("Treated", 5.0).from("Wastewater").group("Back to nature"),
                        FlowStep::new("Untreated", 4.0).from("Wastewater").group("Polluted"),
                    ])
                    .max_width(560.0)
                    .height(820.0)
                    .format(|v| format!("{v:.0} units"))
                    .into_any_element(),
                ))
                .into_any_element(),
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
