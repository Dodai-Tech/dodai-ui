//! Demos for the status charts: a radio channel plan, health strip, graded buckets, stage
//! success, a gradient scale and a network overview composed from stock parts.

use super::super::*;
use dodai_charts::gpui::{
    BucketBar, GradientScale, LineChart, PieChart, Series, Slot, SlotGrid, SlotState, StageBars,
    StatusStrip,
};
use dodai_core::Tone;
use dodai_gpui::live::{drift, wander_between};
use dodai_gpui::{Avatar, Table};

/// Channel names of the two bands.
const LOW_BAND: [&str; 14] = [
    "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14",
];
const HIGH_BAND: [&str; 25] = [
    "36", "40", "44", "48", "52", "56", "60", "64", "100", "104", "108", "112", "116", "120",
    "124", "128", "132", "136", "140", "144", "149", "153", "157", "161", "165",
];

/// The low band's rows as designed: `(width, blocks)`.
fn low_band() -> Vec<(&'static str, Vec<Slot>)> {
    use SlotState::*;
    let narrow = [
        Open, Excluded, Excluded, Excluded, Excluded, Active, Excluded, Excluded, Excluded,
        Excluded, Open, Excluded, Excluded,
    ];
    let wide = (0..10)
        .map(|k| Slot::new(k as f32 * 1.4, 1.4, if k == 9 { Unavailable } else { Open }))
        .collect();
    vec![
        (
            "20 MHz",
            narrow
                .iter()
                .enumerate()
                .map(|(i, s)| Slot::new(i as f32, 1.0, *s))
                .collect(),
        ),
        ("40 MHz", wide),
    ]
}

/// The high band's rows as designed: restricted from 52 to 140, unavailable above.
fn high_band() -> Vec<(&'static str, Vec<Slot>)> {
    use SlotState::*;
    let at = |first: usize, span: usize, s| Slot::new(first as f32, span as f32, s);
    let by = |span: usize, count: usize, state: &dyn Fn(usize) -> SlotState| {
        (0..count)
            .map(|k| at(k * span, span, state(k)))
            .collect::<Vec<_>>()
    };
    vec![
        (
            "20 MHz",
            by(1, 25, &|k| match k {
                0..=3 => Open,
                4..=18 => Restricted,
                _ => Unavailable,
            }),
        ),
        (
            "40 MHz",
            by(2, 12, &|k| match k {
                0 => Active,
                1 => Open,
                2..=8 => Restricted,
                _ => Unavailable,
            }),
        ),
        (
            "80 MHz",
            by(4, 6, &|k| match k {
                0 => Open,
                1..=3 => Restricted,
                _ => Unavailable,
            }),
        ),
        ("160 MHz", by(8, 2, &|_| Restricted)),
    ]
}

/// Health over the last day (72 twenty-minute samples): a short warning and an outage.
const HEALTH: [(f32, f32, Tone); 5] = [
    (0.0, 5.0, Tone::Success),
    (5.0, 6.0, Tone::Warning),
    (6.0, 66.0, Tone::Success),
    (66.0, 66.6, Tone::Danger),
    (66.6, 72.0, Tone::Success),
];

/// Retry-rate buckets (worst first) and how many radios fall in each.
const RETRY_EDGES: [&str; 7] = ["35%+", "25%", "20%", "15%", "10%", "5%", "0%"];
const RETRIES: [u32; 6] = [0, 0, 0, 0, 1, 0];

/// Connection success per step, in percent; two steps wobble.
fn stages() -> [(&'static str, f32); 4] {
    [
        ("Association", 100.0),
        ("Authentication", wander_between(51.0, 93.0, 100.0)),
        ("Address", wander_between(52.0, 78.0, 100.0)),
        ("Names", 100.0),
    ]
}

/// Signal strength in dBm on a −90 (weak) … −30 (strong) scale.
fn signal() -> GradientScale {
    GradientScale::new(-90.0, -30.0, wander_between(53.0, -78.0, -50.0))
}

/// Traffic per application: `(name, down GB, up GB)`.
const APPS: [(&str, f32, f32); 6] = [
    ("Secure web", 59.0, 1.73),
    ("Video", 5.13, 0.046),
    ("Code hosting", 2.81, 0.233),
    ("Team chat", 2.33, 0.027),
    ("Fast web", 1.67, 0.035),
    ("App store", 0.945, 0.027),
];

/// A spiky overnight traffic trace: quiet with bursts, `n` samples.
fn traffic(n: usize, seed: f32) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let x = i as f32;
            let burst = ((x * 0.37 + seed).sin() * (x * 0.11 + seed * 2.0).cos()).max(0.0);
            let spike = if (i * 7 + seed as usize) % 23 == 0 {
                60.0
            } else {
                0.0
            };
            4.0 + 3.0 * (x * 0.9 + seed).sin().abs() + 90.0 * burst.powi(6) + spike
        })
        .collect()
}

impl Showcase {
    /// One band of the channel plan with the user's exclusions applied; clicking a block
    /// toggles it. `rail` draws only the 20 MHz row as a thin rail (the overview's form).
    fn channel_band(&self, me: &Entity<Self>, band: usize, rail: bool) -> SlotGrid {
        let (caption, slots, rows): ((&str, &str), &[&str], _) = if band == 0 {
            (("2.4 GHz", "2412–2484 MHz"), &LOW_BAND, low_band())
        } else {
            (("5 GHz", "5180–5885 MHz"), &HIGH_BAND, high_band())
        };
        let id = format!("{}-{band}", if rail { "rail" } else { "plan" });
        // A rail is one slot per channel: in use when any width uses it, else as at 20 MHz.
        let rows = if rail {
            let busy = |i: usize| {
                rows.iter().flat_map(|r| &r.1).any(|b| {
                    b.state == SlotState::Active
                        && (b.start..b.start + b.span).contains(&(i as f32))
                })
            };
            let narrow = rows.first().map(|r| r.1.clone()).unwrap_or_default();
            vec![(
                "",
                narrow
                    .into_iter()
                    .map(|b| {
                        let state = if busy(b.start as usize) {
                            SlotState::Active
                        } else {
                            b.state
                        };
                        Slot::new(b.start, b.span, state)
                    })
                    .collect(),
            )]
        } else {
            rows
        };
        let mut grid = SlotGrid::new(slots.iter().copied())
            .id(gpui::SharedString::from(id))
            .compact(rail);
        grid = if rail {
            grid.caption(caption.0, "")
        } else {
            grid.caption(caption.0, caption.1).corner("Channels")
        };
        for (r, (label, blocks)) in
            rows.into_iter()
                .enumerate()
                .take(if rail { 1 } else { usize::MAX })
        {
            let blocks = blocks.into_iter().enumerate().map(|(k, mut b)| {
                let picked = self.excluded_slots.contains(&(band, r, k));
                if picked && matches!(b.state, SlotState::Open | SlotState::Restricted) {
                    b.state = SlotState::Excluded;
                } else if picked && b.state == SlotState::Excluded {
                    b.state = SlotState::Open;
                }
                b
            });
            grid = grid.row(label, blocks);
        }
        let m = me.clone();
        grid.on_pick(move |r, k, _, cx| {
            m.update(cx, |s, cx| {
                if !s.excluded_slots.remove(&(band, r, k)) {
                    s.excluded_slots.insert((band, r, k));
                }
                cx.notify();
            });
        })
    }

    /// Both bands side by side, full plan or rails.
    fn channel_plan(&self, me: &Entity<Self>, t: &dodai_core::Theme, rail: bool) -> gpui::Div {
        let legend = ["In use", "Enabled", "DFS", "Not available", "Excluded"];
        let low = self.channel_band(me, 0, rail);
        let high = self.channel_band(me, 1, rail);
        div()
            .flex()
            .flex_wrap()
            .gap(px(t.spacing.xl))
            .w_full()
            .child(div().flex_1().min_w(px(280.0)).child(low))
            .child(div().flex_1().min_w(px(560.0)).child(if rail {
                high
            } else {
                high.legend(legend)
            }))
    }

    /// One section per status chart, then the overview that composes them.
    pub(crate) fn status_demos(
        &self,
        me: &Entity<Self>,
        cx: &App,
    ) -> Vec<(&'static str, gpui::AnyElement)> {
        let t = theme_of(cx);
        let noted = |note: &'static str, el: gpui::AnyElement| {
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .w_full()
                .child(Muted::new(note))
                .child(el)
                .into_any_element()
        };
        vec![
            (
                "Slot grid",
                noted(
                    "A radio channel plan: click a channel to exclude it from use (again to bring it back).",
                    self.channel_plan(me, &t, false).into_any_element(),
                ),
            ),
            (
                "Status strip",
                noted(
                    "A day of health in one strip: a short warning and an outage.",
                    StatusStrip::new(0.0, 72.0, HEALTH).into_any_element(),
                ),
            ),
            (
                "Bucket bar",
                noted(
                    "Radios by retry rate, worst left; only the 5–10 % bucket holds one.",
                    BucketBar::new(RETRY_EDGES, RETRIES).into_any_element(),
                ),
            ),
            (
                "Stage bars",
                noted(
                    "Connection success per step: it fades to amber below 95 % and to red at 80 %.",
                    StageBars::new(stages()).into_any_element(),
                ),
            ),
            (
                "Gradient scale",
                noted(
                    "Signal strength in dBm: weak on the left, strong on the right.",
                    signal().into_any_element(),
                ),
            ),
            ("Network overview", self.network_overview(me, &t)),
        ]
    }

    /// A home network at a glance, composed from the status charts above and stock charts.
    fn network_overview(&self, me: &Entity<Self>, t: &dodai_core::Theme) -> gpui::AnyElement {
        let n = 72;
        let times = ["2 PM", "6 PM", "10 PM", "2 AM", "6 AM", "10 AM", "Now"];
        let labels = (0..n).map(|i| match i {
            _ if i % 12 == 0 => times[i / 12],
            _ if i == n - 1 => times[6],
            _ => "",
        });
        let tile = |name: &'static str, initials: String| {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(t.spacing.xs))
                .w(px(56.0))
                .child(Avatar::new(initials))
                .child(Muted::new(name))
        };
        let initials =
            |s: &str| -> String { s.split(' ').filter_map(|w| w.chars().next()).collect() };
        let card = || {
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .p(px(t.spacing.md))
                .rounded(px(t.shape.radius))
                .border_1()
                .border_color(hsla(t.palette.border))
        };
        let titled = |s: &'static str, el: gpui::AnyElement| {
            div()
                .flex_1()
                .min_w(px(300.0))
                .flex()
                .flex_col()
                .gap(px(t.spacing.sm))
                .child(Label::new(s))
                .child(el)
        };
        let gb = |v: f32| {
            if v >= 1.0 {
                format!("{v:.1} GB")
            } else {
                format!("{:.0} MB", v * 1000.0)
            }
        };
        let total: f32 = APPS.iter().map(|(_, d, u)| d + u).sum();
        let row = || div().flex().flex_wrap().gap(px(t.spacing.xl));
        div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.md))
            .w_full()
            .child(
                card()
                    .child(
                        LineChart::new([Series::new("Download Mbps", traffic(n, 1.0))])
                            .secondary([Series::new("Clients", drift(&vec![14.0; n], 61.0))])
                            .labels(labels)
                            .height(200.0),
                    )
                    .child(StatusStrip::new(0.0, n as f32, HEALTH)),
            )
            .child(
                card().child(
                    row()
                        .child(titled(
                            "Top clients",
                            div()
                                .flex()
                                .gap(px(t.spacing.xs))
                                .children(
                                    ["Laptop", "Phone", "Tablet", "TV", "Console"]
                                        .map(|c| tile(c, initials(c))),
                                )
                                .into_any_element(),
                        ))
                        .child(titled(
                            "Top apps",
                            div()
                                .flex()
                                .gap(px(t.spacing.xs))
                                .children(APPS.iter().take(5).map(|(a, ..)| tile(a, initials(a))))
                                .into_any_element(),
                        )),
                ),
            )
            .child(card().child(self.channel_plan(me, t, true)))
            .child(
                card().child(
                    row()
                        .child(titled(
                            "Radio retries",
                            BucketBar::new(RETRY_EDGES, RETRIES).into_any_element(),
                        ))
                        .child(titled(
                            "Connectivity",
                            StageBars::new(stages()).into_any_element(),
                        )),
                ),
            )
            .child(card().child(titled(
                "Access point density is good",
                signal().into_any_element(),
            )))
            .child(
                row()
                    .child(
                        card().flex_1().min_w(px(520.0)).child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.lg))
                                .child(
                                    PieChart::new(APPS.iter().map(|(a, d, u)| (*a, d + u)))
                                        .donut(0.82)
                                        .size(140.0)
                                        .center(format!("{total:.1} GB\nTotal traffic"))
                                        .legend(false),
                                )
                                .child(
                                    div().flex_1().child(
                                        Table::new("ov-apps")
                                            .columns(["Application", "Down", "Up", "Traffic"])
                                            .weights([2.0, 1.3, 1.3, 1.3])
                                            .rows(APPS.iter().map(|(a, d, u)| {
                                                vec![a.to_string(), gb(*d), gb(*u), gb(d + u)]
                                            })),
                                    ),
                                ),
                        ),
                    )
                    .child(
                        card().flex_1().min_w(px(360.0)).child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(t.spacing.lg))
                                .child(
                                    PieChart::new([("5 GHz", 11.0), ("2.4 GHz", 6.0)])
                                        .donut(0.82)
                                        .size(140.0)
                                        .center("17\nConnections")
                                        .legend(false),
                                )
                                .child(
                                    div().flex_1().child(
                                        Table::new("ov-radios")
                                            .columns(["Type", "Band", "Experience", "Clients"])
                                            .rows([
                                                ["Wi-Fi 5", "5 GHz", "Excellent", "11"],
                                                ["Wi-Fi 4", "2.4 GHz", "Excellent", "6"],
                                            ]),
                                    ),
                                ),
                        ),
                    ),
            )
            .into_any_element()
    }
}
