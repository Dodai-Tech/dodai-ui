//! "Bands · Stream · Bump · Ridgeline · Horizon": the layered chart family.

use super::super::*;
use dodai_charts::gpui::{BumpChart, HorizonChart, RidgelineChart, StackOffset, StackedAreaChart};

/// A smooth, repeatable wobble for demo data: `k` picks the curve, `i` the step.
fn wave(k: usize, i: usize) -> f32 {
    let (k, i) = (k as f32, i as f32);
    (i * 0.21 + k * 1.7).sin() + 0.5 * (i * 0.057 + k).cos() + 0.3 * (i * 0.6 + k * 3.1).sin()
}

/// `n` samples around `centre` spread by `spread`, deterministic (a sum of sines, not noise).
fn samples(centre: f32, spread: f32, n: usize, seed: usize) -> Vec<f32> {
    (0..n)
        .map(|i| centre + spread * (wave(seed, i * 7) + wave(seed + 3, i * 13)) / 2.2)
        .collect()
}

impl Showcase {
    /// The layered chart demos as `(section, body)`.
    pub(crate) fn layered_demos(&self) -> Vec<(&'static str, gpui::AnyElement)> {
        let weeks = ["W1", "W2", "W3", "W4", "W5", "W6", "W7", "W8"];
        // Forecast with its uncertainty widening into the future.
        let actual: Vec<f32> = (0..12).map(|i| 40.0 + 6.0 * wave(1, i * 3)).collect();
        let forecast: Vec<f32> = (0..12).map(|i| 42.0 + 0.8 * i as f32).collect();
        let spread: Vec<f32> = (0..12).map(|i| 2.0 + 0.9 * i as f32).collect();
        let band = LineChart::new([
            Series::new("Actual", actual),
            Series::new("Forecast", forecast.clone()).band(
                forecast.iter().zip(&spread).map(|(f, s)| f - s),
                forecast.iter().zip(&spread).map(|(f, s)| f + s),
            ),
        ])
        .include_zero(false)
        .points(false)
        .y_title("Orders / day")
        .height(180.0);

        let genres = ["Ambient", "Jazz", "Techno", "Folk", "Soul"];
        let layers = || {
            genres.iter().enumerate().map(|(k, g)| {
                Series::new(
                    *g,
                    (0..24).map(move |i| (4.0 + 3.0 * wave(k, i * 2)).max(0.3)),
                )
            })
        };
        let stream = StackedAreaChart::new(layers())
            .offset(StackOffset::Wiggle)
            .legend(false)
            .height(180.0);
        let share = StackedAreaChart::new(layers())
            .offset(StackOffset::Expand)
            .height(180.0);

        let teams = ["Otters", "Herons", "Lynx", "Moths", "Crows"];
        let bump = BumpChart::new(teams.iter().enumerate().map(|(k, n)| {
            let mut total = 0.0;
            Series::new(
                *n,
                (0..weeks.len()).map(move |i| {
                    total += 10.0 + 4.0 * wave(k * 2, i * 5);
                    total
                }),
            )
        }))
        .labels(weeks)
        .format(|v| format!("{v:.0} pts"));

        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];
        let ridge = RidgelineChart::new(months.iter().enumerate().map(|(k, m)| {
            (
                *m,
                samples(2.0 + 3.2 * k as f32, 2.5 + 0.3 * k as f32, 60, k),
            )
        }))
        .format(|v| format!("{v:.0}°"));

        let hosts = ["eu-1", "eu-2", "us-1", "us-2", "ap-1"];
        let horizon = HorizonChart::new(hosts.iter().enumerate().map(|(k, h)| {
            Series::new(
                *h,
                (0..120).map(move |i| 30.0 * wave(k + 5, i) * (1.0 + 0.4 * k as f32 / 4.0)),
            )
        }))
        .bands(3)
        .format(|v| format!("{v:.0} ms"));

        let note = |n: &'static str, el: gpui::AnyElement| {
            div()
                .flex()
                .flex_col()
                .gap_2()
                .w_full()
                .child(Muted::new(n))
                .child(el)
                .into_any_element()
        };
        vec![
            (
                "Band",
                note(
                    "Series::band: a forecast and its widening range, under the line.",
                    band.into_any_element(),
                ),
            ),
            (
                "Bump",
                note(
                    "Standings by week: who overtook whom. Hover a week for the table.",
                    bump.into_any_element(),
                ),
            ),
            (
                "Stream",
                note(
                    "StackOffset::Wiggle: the stack floats so the layers bend least.",
                    stream.into_any_element(),
                ),
            ),
            (
                "100% stacked",
                note(
                    "StackOffset::Expand: every step stretched to 100%.",
                    share.into_any_element(),
                ),
            ),
            (
                "Ridgeline",
                note(
                    "One distribution per month on a shared axis.",
                    ridge.into_any_element(),
                ),
            ),
            (
                "Horizon",
                note(
                    "Latency off its baseline per host, folded into 3 bands.",
                    horizon.into_any_element(),
                ),
            ),
        ]
    }
}
