//! A speed test in the netrunner style: live download and upload bars, the phase stepper,
//! the run summary and the trend over past runs. Built from the stock charts only.

use super::super::*;

/// The test's phases, in order, with how long each runs (seconds of animation time).
const PHASES: [(&str, f32); 7] = [
    ("Locate", 1.0),
    ("Server", 1.0),
    ("Latency", 1.5),
    ("Download", 8.0),
    ("Upload", 8.0),
    ("Jitter", 1.5),
    ("Done", 3.0),
];
/// Indices into [`PHASES`].
const DOWNLOAD: usize = 3;
const UPLOAD: usize = 4;
/// Seconds between samples, and how many a panel shows (just over one phase).
const EVERY: f32 = 0.15;
pub(crate) const WINDOW: usize = 56;
/// Where each transfer levels off, in Mbps.
const TOP: [f32; 2] = [420.0, 96.0];

/// The phase the looping test is in, and how long it has been in it.
fn phase(now: f32) -> (usize, f32) {
    let cycle: f32 = PHASES.iter().map(|p| p.1).sum();
    let mut at = now.rem_euclid(cycle);
    for (i, (_, d)) in PHASES.iter().enumerate() {
        if at < *d {
            return (i, at);
        }
        at -= d;
    }
    (PHASES.len() - 1, 0.0)
}

/// A transfer `secs` in: ramps up as the connection opens, then wobbles around its top speed.
fn rate(top: f32, secs: f32) -> f32 {
    let wobble = 0.05 * (secs * 7.3).sin() + 0.03 * (secs * 17.1 + 1.0).sin();
    top * (1.0 - (-secs / 1.1).exp()) * (1.0 + wobble)
}

/// Past runs (oldest first): download and upload in Mbps.
const HISTORY: [(f32, f32); 14] = [
    (388.0, 88.0),
    (402.0, 91.0),
    (215.0, 64.0),
    (395.0, 90.0),
    (410.0, 93.0),
    (377.0, 85.0),
    (120.0, 41.0),
    (399.0, 92.0),
    (418.0, 95.0),
    (405.0, 89.0),
    (366.0, 84.0),
    (412.0, 94.0),
    (390.0, 90.0),
    (421.0, 96.0),
];

impl Showcase {
    /// Feeds the transfer windows: each fills during its own phase, rests at zero before it
    /// (so the last run slides out) and holds still after it.
    pub(crate) fn speed_step(&mut self) {
        let (now, at) = phase(dodai_gpui::motion::time());
        for (k, live) in [&self.speed_down, &self.speed_up].into_iter().enumerate() {
            let own = [DOWNLOAD, UPLOAD][k];
            if now <= own {
                live.advance(EVERY, |_| if now == own { rate(TOP[k], at) } else { 0.0 });
            }
        }
    }

    /// The section body.
    pub(crate) fn throughput_demo(&self, cx: &mut Context<Self>) -> gpui::Div {
        let t = theme_of(cx);
        let (now, _) = phase(dodai_gpui::motion::time());
        let colours = [t.palette.success, t.palette.accent];
        let mono = dodai_gpui::text::first_family(t.typography.mono);
        let gap = px(t.spacing.md);
        let windows = [&self.speed_down, &self.speed_up];
        // Mean of the samples a transfer has taken so far (its resting zeros left out).
        let mean = |w: &dodai_gpui::Live| {
            let v: Vec<f32> = w.values().into_iter().filter(|v| *v > 0.5).collect();
            (!v.is_empty()).then(|| v.iter().sum::<f32>() / v.len() as f32)
        };
        let figure =
            |v: Option<f32>, unit: &str| v.map_or("—".to_string(), |v| format!("{v:.1} {unit}"));

        let transfer = |k: usize| {
            let w = windows[k];
            let values = w.values();
            let peak = values.iter().copied().fold(0.0, f32::max);
            let current = values.last().copied().unwrap_or(0.0);
            let title = ["↓ Download", "↑ Upload"][k];
            Panel::new()
                .focused(now == [DOWNLOAD, UPLOAD][k])
                .refine(|d| d.flex_1().min_w(px(260.0)))
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .text_color(dodai_gpui::hsla(colours[k]))
                                .font_family(mono.clone())
                                .child(title),
                        )
                        .child(Muted::new(format!("peak {peak:.1}"))),
                )
                .child(
                    div()
                        .font_family(mono.clone())
                        .text_size(px(t.typography.size_xxl))
                        .child(format!("{current:.1} Mbps")),
                )
                // The meter's scale rounds up from the peak, so it holds still while it fills.
                .child(
                    Bullet::new("", current)
                        .target(peak)
                        .nice_scale(true)
                        .color(colours[k])
                        .height(12.0),
                )
                .child(
                    Sparkline::new([])
                        .live(w)
                        .guide(SparkGuide::Peak)
                        .style(SparkStyle::Bars)
                        .color(colours[k])
                        .fluid(true)
                        .size(260.0, 150.0),
                )
        };

        let metric = |label: &'static str, value: String| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(Muted::new(label))
                .child(
                    div()
                        .font_family(mono.clone())
                        .text_size(px(t.typography.size_lg))
                        .child(value),
                )
        };
        let done = now >= PHASES.len() - 1;
        let summary = Panel::new().title("Result").child(
            div()
                .flex()
                .flex_wrap()
                .items_end()
                .gap(px(t.spacing.xl))
                .child(metric("Download", figure(mean(windows[0]), "Mbps")))
                .child(metric("Upload", figure(mean(windows[1]), "Mbps")))
                .child(metric("Ping", figure((now > 2).then_some(11.6), "ms")))
                .child(metric("Jitter", figure((now > 5).then_some(1.8), "ms")))
                .child(if done {
                    Tag::new("Excellent", Tone::Success).into_any_element()
                } else {
                    Muted::new("testing…").into_any_element()
                }),
        );

        // The same two windows as lines, the way a dashboard plots them.
        let lines = Panel::new().title("Throughput").child(
            LineChart::new([
                Series::live("Download", windows[0]).color(colours[0]),
                Series::live("Upload", windows[1]).color(colours[1]),
            ])
            .area(true)
            .y_range(0.0, 500.0)
            .y_title("Mbps")
            .x_title(format!("Last {:.0} s", WINDOW as f32 * EVERY))
            .height(150.0),
        );

        let trend = |k: usize| {
            let values: Vec<f32> = HISTORY.iter().map(|h| [h.0, h.1][k]).collect();
            let latest = values.last().copied().unwrap_or(0.0);
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w(px(220.0))
                .gap_1()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(Muted::new(["Download trend", "Upload trend"][k]))
                        .child(
                            div()
                                .text_color(dodai_gpui::hsla(colours[k]))
                                .font_family(mono.clone())
                                .text_size(px(t.typography.size_sm))
                                .child(format!("{latest:.1} Mbps")),
                        ),
                )
                .child(
                    Sparkline::new(values)
                        .guide(SparkGuide::Mean)
                        .style(SparkStyle::Bars)
                        .color(colours[k])
                        .fluid(true)
                        .size(220.0, 64.0),
                )
        };
        let history = Panel::new().title("History").child(
            div()
                .flex()
                .flex_wrap()
                .gap(gap)
                .child(trend(0))
                .child(trend(1)),
        );

        div()
            .flex()
            .flex_col()
            .gap(gap)
            // "Done" is the rest after the last step, not a step: every step is ticked then.
            .child(Stepper::new(
                PHASES[..PHASES.len() - 1].iter().map(|p| p.0),
                now,
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(gap)
                    .child(transfer(0))
                    .child(transfer(1)),
            )
            .child(summary)
            .child(lines)
            .child(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_test_runs_its_phases_in_order_and_loops() {
        assert_eq!(phase(0.0).0, 0);
        assert_eq!(phase(3.6).0, DOWNLOAD);
        assert_eq!(phase(11.6).0, UPLOAD);
        let cycle: f32 = PHASES.iter().map(|p| p.1).sum();
        assert_eq!(phase(cycle + 0.1).0, 0);
        assert!(
            rate(100.0, 0.0).abs() < 1e-3,
            "a transfer starts from nothing"
        );
        assert!(
            (rate(100.0, 6.0) - 100.0).abs() < 10.0,
            "and levels off near its top"
        );
    }
}
