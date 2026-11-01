//! The gauge-style demos of the charts page.

use super::*;
use gpui::Div;

impl Showcase {
    /// Every `GaugeStyle` on live values, in three rows.
    pub(crate) fn gauge_style_demos(&self, cx: &mut Context<Self>) -> Div {
        let t = theme_of(cx);
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let dg = |v: f32, key: f32| {
            dodai_gpui::live::wander_between(key, (v - 14.0).max(0.0), (v + 14.0).min(100.0))
        };
        use GaugeStyle::*;
        let dials = row()
            .justify_around()
            .flex_wrap()
            .child(Gauge::new(dg(72.0, 41.0)).unit("%").tone(Tone::Success))
            .child(Gauge::new(dg(31.0, 42.0)).tone(Tone::Warning))
            .child(
                Gauge::new(dg(68.0, 43.0))
                    .style(Segmented)
                    .size(130.0)
                    .caption("Core temp")
                    .unit("°"),
            )
            .child(
                Gauge::new(dg(72.0, 44.0))
                    .style(Dial)
                    .zones(true)
                    .size(150.0)
                    .caption("Load"),
            )
            .child(
                Gauge::new(dg(88.0, 45.0))
                    .style(Meter)
                    .zones(true)
                    .size(150.0)
                    .caption("Shield")
                    .unit("%"),
            )
            .child(
                Gauge::new(dg(45.0, 46.0))
                    .style(Reticle)
                    .size(130.0)
                    .tone(Tone::Info)
                    .caption("Sync")
                    .unit("%"),
            );
        let bars = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Gauge::new(dg(82.0, 47.0))
                    .style(Bar)
                    .size(150.0)
                    .zones(true)
                    .caption("Power")
                    .unit("%"),
            )
            .child(
                Gauge::new(dg(35.0, 48.0))
                    .style(Bar)
                    .size(150.0)
                    .tone(Tone::Info)
                    .caption("Link")
                    .unit("%"),
            )
            .child(
                Gauge::new(dg(14.0, 49.0))
                    .style(Bar)
                    .size(150.0)
                    .tone(Tone::Danger)
                    .caption("Hull")
                    .unit("%"),
            );
        let more = div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                Gauge::new(dg(64.0, 50.0))
                    .style(Chevron)
                    .size(150.0)
                    .zones(true)
                    .caption("Thrust")
                    .unit("%"),
            )
            .child(
                Gauge::new(dg(58.0, 51.0))
                    .style(Level)
                    .size(110.0)
                    .zones(true)
                    .caption("Fuel")
                    .unit("%"),
            );
        let bars = row().justify_around().flex_wrap().child(bars).child(more);
        let extra = row()
            .justify_around()
            .flex_wrap()
            .items_center()
            .child(
                Gauge::new(dg(64.0, 52.0))
                    .style(Ring)
                    .size(120.0)
                    .caption("Disk")
                    .unit("%")
                    .tone(Tone::Info),
            )
            .child(
                Gauge::new(dg(78.0, 53.0))
                    .style(Smooth)
                    .size(130.0)
                    .zones(true)
                    .caption("Speed"),
            )
            .child(
                Gauge::new(dg(55.0, 54.0))
                    .style(Dots)
                    .size(130.0)
                    .caption("Volume")
                    .target(80.0),
            )
            .child(
                Gauge::new(dg(70.0, 55.0))
                    .style(Led)
                    .size(150.0)
                    .zones(true)
                    .caption("Level")
                    .unit("%"),
            )
            .child(
                Gauge::new(dg(34.0, 56.0))
                    .style(Battery)
                    .size(110.0)
                    .caption("Battery")
                    .unit("%")
                    .tone(Tone::Success),
            )
            .child(
                Gauge::new(dg(18.0, 57.0))
                    .style(Battery)
                    .size(110.0)
                    .caption("Low")
                    .unit("%")
                    .tone(Tone::Success),
            )
            .child(
                Gauge::new(dg(62.0, 58.0))
                    .style(Thermometer)
                    .size(120.0)
                    .zones(true)
                    .caption("Temp")
                    .unit("°")
                    .target(75.0),
            )
            .child(
                Gauge::new(dg(60.0, 59.0))
                    .style(Signal)
                    .size(100.0)
                    .caption("Signal")
                    .label("3/5")
                    .tone(Tone::Info),
            );
        div()
            .flex()
            .flex_col()
            .gap_6()
            .child(dials)
            .child(bars)
            .child(extra)
    }
}
