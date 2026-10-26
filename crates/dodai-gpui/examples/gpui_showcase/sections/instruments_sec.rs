//! The "Instruments" category: vehicle, industrial and medical readouts.

use super::super::*;
use dodai_gpui::{
    Annunciator, ClusterKind, Compass, DialGauge, DialLook, GearIndicator, InstrumentCluster,
    LevelBar, Limits, Odometer, PowerMeter, Scale, Severity, TirePressure, VitalsTile,
};

impl Showcase {
    /// Vehicle, industrial and medical instruments, one section each.
    pub(crate) fn sections_instruments(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        let ChartDemos {
            instruments: (lab, hud),
            pulse,
            gauge_styles,
            ..
        } = self.chart_demos(&me, cx);

        // Cluster sets: one coherent style per cluster, never mixed.
        let mut clusters = div().flex().flex_col().gap(px(t.spacing.lg));
        clusters = clusters
            .child(Muted::new("This design's own cluster"))
            .child(InstrumentCluster::themed().width(700.0));
        for kind in ClusterKind::ALL
            .into_iter()
            .filter(|k| *k != t.chart_style().cluster)
        {
            clusters = clusters
                .child(Muted::new(kind.name()))
                .child(InstrumentCluster::new(kind).width(700.0));
        }
        // Vehicle: a speedometer with a redline tachometer and a fuel/temperature pair.
        let speed = Scale::new(0.0, 240.0)
            .ticks_per(12, 2)
            .zone(200.0, 240.0, Severity::Caution);
        let rpm = Scale::new(0.0, 8.0)
            .ticks_per(8, 2)
            .zone(6.5, 8.0, Severity::Alarm);
        let speed2 = speed.clone();
        let fuel = Scale::new(0.0, 100.0)
            .ticks_per(4, 1)
            .zone(0.0, 12.0, Severity::Alarm)
            .zone(12.0, 25.0, Severity::Caution);
        let coolant = Scale::new(40.0, 130.0)
            .ticks_per(3, 3)
            .zone(105.0, 115.0, Severity::Caution)
            .zone(115.0, 130.0, Severity::Alarm);
        let tyres = Limits {
            low: Some(2.0),
            high: Some(3.0),
            low_low: Some(1.6),
            ..Limits::default()
        };
        let power = Scale::new(-60.0, 120.0)
            .ticks_per(6, 1)
            .zone(100.0, 120.0, Severity::Caution);
        let status = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.lg))
            .child(
                row()
                    .items_start()
                    .child(LevelBar::new(fuel, 18.0).label("Fuel").unit("%"))
                    .child(LevelBar::new(coolant, 108.0).label("Coolant").unit("°C"))
                    .child(TirePressure::new([2.4, 2.5, 1.8, 2.3], tyres)),
            )
            .child(
                row()
                    .child(GearIndicator::automatic(3))
                    .child(Odometer::new(48213.0, 6).unit("km"))
                    .child(Odometer::new(126.4, 4).decimals(1).unit("km trip")),
            )
            .child(
                row()
                    .items_start()
                    .child(PowerMeter::new(power, 84.0))
                    .child(Compass::new(42.0)),
            );

        out.push((
            "Instruments",
            "Vehicle",
            self.section("Vehicle", &t, &me, cx)
                .child(Muted::new("Clusters: one coherent style per set."))
                .child(clusters)
                .child(Muted::new("Dials"))
                .child(
                    row()
                        .child(
                            DialGauge::new(speed, 132.0)
                                .label("Speed")
                                .unit("km/h")
                                .size(240.0),
                        )
                        .child(
                            DialGauge::new(rpm, 7.1)
                                .label("Engine")
                                .unit("×1000 rpm")
                                .decimals(1)
                                .size(240.0),
                        ),
                )
                .child(Muted::new(
                    "The same dial in each look. A design picks its own by default.",
                ))
                .child(
                    row()
                        .child(
                            DialGauge::new(speed2.clone(), 96.0)
                                .look(DialLook::Analog)
                                .label("Analog")
                                .unit("km/h")
                                .tick_labels(false)
                                .size(170.0),
                        )
                        .child(
                            DialGauge::new(speed2.clone(), 96.0)
                                .look(DialLook::Arc)
                                .label("Arc")
                                .unit("km/h")
                                .size(170.0),
                        )
                        .child(
                            DialGauge::new(speed2, 96.0)
                                .look(DialLook::Segmented)
                                .label("Segmented")
                                .unit("km/h")
                                .size(170.0),
                        ),
                )
                .child(Muted::new("Status"))
                .child(status)
                .into_any_element(),
            "Vehicle",
        ));
        // Industrial: process dials with alarm bands and an annunciator panel.
        let limits = |lo: f32, hi: f32, hh: f32| Limits {
            low: Some(lo),
            high: Some(hi),
            high_high: Some(hh),
            ..Limits::default()
        };
        let pressure = limits(10.0, 70.0, 90.0).zones_on(Scale::new(0.0, 100.0).ticks_per(5, 2));
        let temp = limits(20.0, 180.0, 220.0).zones_on(Scale::new(0.0, 250.0).ticks_per(5, 2));
        let flow = limits(15.0, 80.0, 95.0).zones_on(Scale::new(0.0, 100.0).ticks_per(5, 2));
        let tank_a = Scale::new(0.0, 100.0)
            .ticks_per(4, 1)
            .zone(90.0, 100.0, Severity::Caution);
        let tank_b = Scale::new(0.0, 100.0)
            .ticks_per(4, 1)
            .zone(90.0, 100.0, Severity::Alarm);
        out.push((
            "Instruments",
            "Industrial",
            self.section("Industrial", &t, &me, cx)
                .child(
                    row()
                        .child(
                            DialGauge::new(pressure, 54.0)
                                .label("Header pressure")
                                .unit("bar")
                                .size(190.0),
                        )
                        .child(
                            DialGauge::new(temp, 196.0)
                                .label("Reactor temp")
                                .unit("°C")
                                .size(190.0),
                        )
                        .child(
                            DialGauge::new(flow, 97.0)
                                .label("Feed flow")
                                .unit("m³/h")
                                .size(190.0),
                        )
                        .child(
                            LevelBar::new(tank_a, 72.0)
                                .label("Tank A")
                                .unit("%")
                                .height(130.0),
                        )
                        .child(
                            LevelBar::new(tank_b, 94.0)
                                .label("Tank B")
                                .unit("%")
                                .height(130.0),
                        ),
                )
                .child(Annunciator::new([
                    ("Pump 1 run", Severity::Normal, false),
                    ("Pump 2 trip", Severity::Alarm, false),
                    ("Tank high", Severity::Caution, false),
                    ("Door open", Severity::Advisory, false),
                    ("Vibration", Severity::Caution, true),
                    ("Power fail", Severity::Alarm, true),
                    ("Valve closed", Severity::Normal, false),
                    ("Comms", Severity::Normal, false),
                ]))
                .child(Muted::new(
                    "Flashing = unacknowledged, steady = acknowledged. Caution 1 Hz, alarm 2 Hz.",
                ))
                .into_any_element(),
            "Industrial",
        ));

        out.push((
            "Instruments",
            "Gauges",
            self.section("Gauges", &t, &me, cx)
                .child(gauge_styles)
                .into_any_element(),
            "Gauges",
        ));
        out.push((
            "Instruments",
            "Signal lab",
            self.section("Signal lab", &t, &me, cx)
                .child(lab)
                .into_any_element(),
            "Signal lab",
        ));

        // Medical: vital-sign tiles and waveforms.
        let vitals = |lo: f32, hi: f32| Limits {
            low: Some(lo),
            high: Some(hi),
            ..Limits::default()
        };
        out.push((
            "Instruments",
            "Medical",
            self.section("Medical", &t, &me, cx)
                .child(
                    row()
                        .child(VitalsTile::new("HR", 72.0).unit("bpm").limits(vitals(50.0, 110.0)))
                        .child(VitalsTile::new("SpO₂", 91.0).unit("%").limits(vitals(94.0, 100.0)))
                        .child(
                            VitalsTile::new("Temp", 38.9)
                                .unit("°C")
                                .decimals(1)
                                .limits(Limits {
                                    high: Some(38.0),
                                    high_high: Some(38.5),
                                    low: Some(36.0),
                                    ..Limits::default()
                                }),
                        )
                        .child(
                            VitalsTile::new("RR", 28.0)
                                .unit("/min")
                                .limits(vitals(10.0, 24.0))
                                .acknowledged(true),
                        ),
                )
                .child(pulse)
                .child(Muted::new(
                    "Alarm colours come from the theme (warning, danger). An unacknowledged alarm flashes its tile.",
                ))
                .into_any_element(),
            "Medical",
        ));
        // Sci-fi HUD instruments (Cyberpunk only, see `catalog::owner`).
        out.push((
            "Instruments",
            "HUD",
            self.section("Instruments", &t, &me, cx)
                .child(hud)
                .into_any_element(),
            "Instruments",
        ));
        out
    }
}
