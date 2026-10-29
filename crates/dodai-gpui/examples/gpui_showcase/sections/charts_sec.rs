//! The "Charts" category: one section per chart, grouped by family (`catalog::CHART_FAMILIES`).

use super::super::*;

impl Showcase {
    pub(crate) fn sections_charts_sec(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        let ChartDemos { charts, .. } = self.chart_demos(&me, cx);
        let mut bodies: Vec<(&'static str, gpui::AnyElement)> = charts;
        bodies.extend(self.layered_demos());
        bodies.push((
            "Live throughput",
            div()
                .flex()
                .flex_col()
                .gap(px(t.spacing.md))
                .w_full()
                .child(Muted::new(
                    "A network speed test in the netrunner style: each transfer fills its own window during its phase. Stock Sparkline (peak guide), Bullet (clean scale, peak tick), LineChart (axis titles), Stepper and Panel only.",
                ))
                .child(self.throughput_demo(cx))
                .into_any_element(),
        ));
        // The colour switch sits with the first chart; it applies to every chart.
        let mut mono = Some({
            let m = me.clone();
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(t.spacing.md))
                .w_full()
                .child(Label::new("Series colours"))
                .child(
                    Segmented::new("chart-mono", ["Colour", "Monochrome"])
                        .selected(usize::from(dodai_gpui::chart_monochrome()))
                        .on_select(move |i, _, cx| {
                            dodai_gpui::set_chart_monochrome(cx, i == 1);
                            m.update(cx, |_, cx| cx.notify());
                        }),
                )
                .child(div().w_full().child(Muted::new(
                    "Monochrome reads by lightness alone, for print or colour-blind viewing. Applies to every chart.",
                )))
        });
        let mut out: Vec<Section> = Vec::new();
        for (_, sections) in CHART_FAMILIES {
            for &name in sections {
                let mut body = div().flex().flex_wrap().items_start().gap(px(t.spacing.md));
                let mut any = false;
                // Move this section's bodies out, keeping the rest in order.
                let mut i = 0;
                while i < bodies.len() {
                    if bodies[i].0 == name {
                        let (_, el) = bodies.remove(i);
                        body = body.child(div().flex_1().min_w(px(320.0)).child(el));
                        any = true;
                    } else {
                        i += 1;
                    }
                }
                debug_assert!(any, "chart section {name} has no demo");
                out.push((
                    "Charts",
                    name,
                    self.section(name, cx)
                        .children((name == "Line · area").then(|| mono.take()).flatten())
                        .child(body)
                        .into_any_element(),
                    name,
                ));
            }
        }
        debug_assert!(
            bodies.is_empty(),
            "chart demos not listed in CHART_FAMILIES: {:?}",
            bodies.iter().map(|(n, _)| *n).collect::<Vec<_>>()
        );
        out
    }
}
