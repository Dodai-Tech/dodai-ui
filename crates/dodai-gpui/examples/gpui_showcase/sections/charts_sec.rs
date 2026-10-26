//! The "Charts" category sections.

use super::super::*;

impl Showcase {
    pub(crate) fn sections_charts_sec(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        let mut out: Vec<Section> = Vec::new();
        let ChartDemos {
            charts_xy,
            gauge_styles,
            pies,
            donuts,
            sparks,
            charts_market,
            charts_misc,
            charts_more1,
            charts_more2,
            charts_more3,
            charts_new,
            ..
        } = self.chart_demos(&me, cx);
        out.push((
            "Charts",
            "Line · Bar · Area · Stacked charts",
            self.section("Line & bar charts", &t, &me, cx)
                .child({
                    let m = me.clone();
                    div().flex().flex_wrap().items_center().gap(px(t.spacing.md))
                        .child(Label::new("Series colours"))
                        .child(
                            Segmented::new("chart-mono", ["Colour", "Monochrome"])
                                .selected(usize::from(dodai_gpui::chart_monochrome()))
                                .on_select(move |i, _, cx| {
                                    dodai_gpui::set_chart_monochrome(cx, i == 1);
                                    m.update(cx, |_, cx| cx.notify());
                                }),
                        )
                        .child(Muted::new(
                            "Monochrome reads by lightness alone, for print or colour-blind viewing. Applies to every chart.",
                        ))
                })
                .child(charts_xy)
                .into_any_element(),
            "Line & bar charts",
        ));
        out.push((
            "Charts",
            "Pie",
            self.section("Pie", &t, &me, cx)
                .child(pies)
                .into_any_element(),
            "Pie",
        ));
        out.push((
            "Charts",
            "Donut",
            self.section("Donut", &t, &me, cx)
                .child(donuts)
                .into_any_element(),
            "Donut",
        ));
        out.push((
            "Charts",
            "Gauges",
            self.section("Gauges", &t, &me, cx)
                .child(gauge_styles)
                .into_any_element(),
            "Gauges",
        ));
        out.push((
            "Charts",
            "Sparkline",
            self.section("Sparkline", &t, &me, cx)
                .child(sparks)
                .into_any_element(),
            "Sparkline",
        ));
        out.push((
            "Charts",
            "Stacked area · Histogram · Waterfall",
            self.section("Area · histogram · waterfall", &t, &me, cx)
                .child(charts_more1)
                .into_any_element(),
            "Area · histogram · waterfall",
        ));
        out.push((
            "Charts",
            "Funnel · Treemap · Radial bar",
            self.section("Funnel · treemap · radial bar", &t, &me, cx)
                .child(charts_more2)
                .into_any_element(),
            "Funnel · treemap · radial bar",
        ));
        out.push((
            "Charts",
            "Candlestick · Box plot · Bubble",
            self.section("Candlestick · box plot · bubble", &t, &me, cx)
                .child(charts_more3)
                .into_any_element(),
            "Candlestick · box plot · bubble",
        ));
        out.push((
            "Charts",
            "Bullet · Paired · Lollipop · Slope",
            self.section("Bullet · paired · lollipop · slope", &t, &me, cx)
                .child(charts_new.0)
                .into_any_element(),
            "Bullet · paired · lollipop · slope",
        ));
        out.push((
            "Charts",
            "Sankey · Calendar · Violin · Rose",
            self.section("Sankey · calendar · violin · rose", &t, &me, cx)
                .child(charts_new.1)
                .into_any_element(),
            "Sankey · calendar · violin · rose",
        ));
        out.push((
            "Charts",
            "Market · Planning",
            self.section("Market · planning charts", &t, &me, cx)
                .child(charts_market)
                .into_any_element(),
            "Market · planning charts",
        ));
        out.push((
            "Charts",
            "Radar · Scatter · Heatmap",
            self.section("Radar · scatter · heatmap", &t, &me, cx)
                .child(charts_misc)
                .into_any_element(),
            "Radar · scatter · heatmap",
        ));
        out
    }
}
