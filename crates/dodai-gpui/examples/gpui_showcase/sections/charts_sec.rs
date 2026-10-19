//! The "Charts" category sections.

use super::super::*;

impl Showcase {
    #[allow(unused_variables, clippy::redundant_clone, clippy::vec_init_then_push)]
    pub(crate) fn sections_charts_sec(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        #[allow(unused_macros)]
        macro_rules! section {
            ($t:expr) => {
                Panel::new()
                    .title($t)
                    .subheader(self.chips($t, &t, &me, cx))
                    .scope(Some(self.scope_of($t, cx)))
                    .scope_key(self.sec_replay.get($t).copied().unwrap_or(0))
            };
        }
        let mut out: Vec<Section> = Vec::new();
        let ChartDemos {
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
        } = self.chart_demos(&me, cx);
        out.push((
            "Charts",
            "Line · Bar · Area · Stacked charts",
            section!("Line & bar charts")
                .child(charts_xy)
                .into_any_element(),
            "Line & bar charts",
        ));
        out.push((
            "Charts",
            "Pie",
            section!("Pie").child(pies).into_any_element(),
            "Pie",
        ));
        out.push((
            "Charts",
            "Donut",
            section!("Donut").child(donuts).into_any_element(),
            "Donut",
        ));
        out.push((
            "Charts",
            "Gauges",
            section!("Gauges").child(gauge_styles).into_any_element(),
            "Gauges",
        ));
        out.push((
            "Charts",
            "Sparkline",
            section!("Sparkline").child(sparks).into_any_element(),
            "Sparkline",
        ));
        out.push((
            "Charts",
            "Instruments",
            section!("Instruments")
                .child(instruments)
                .into_any_element(),
            "Instruments",
        ));
        out.push((
            "Charts",
            "Stacked area · Histogram · Waterfall",
            section!("Area · histogram · waterfall")
                .child(charts_more1)
                .into_any_element(),
            "Area · histogram · waterfall",
        ));
        out.push((
            "Charts",
            "Funnel · Treemap · Radial bar",
            section!("Funnel · treemap · radial bar")
                .child(charts_more2)
                .into_any_element(),
            "Funnel · treemap · radial bar",
        ));
        out.push((
            "Charts",
            "Candlestick · Box plot · Bubble",
            section!("Candlestick · box plot · bubble")
                .child(charts_more3)
                .into_any_element(),
            "Candlestick · box plot · bubble",
        ));
        out.push((
            "Charts",
            "Market · Planning",
            section!("Market · planning charts")
                .child(charts_market)
                .into_any_element(),
            "Market · planning charts",
        ));
        out.push((
            "Charts",
            "Radar · Scatter · Heatmap",
            section!("Radar · scatter · heatmap")
                .child(charts_misc)
                .into_any_element(),
            "Radar · scatter · heatmap",
        ));
        out
    }
}
