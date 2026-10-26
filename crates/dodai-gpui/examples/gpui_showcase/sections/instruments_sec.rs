//! The "Instruments" category: vehicle, industrial and medical readouts.

use super::super::*;
use dodai_gpui::{
    Affiliation, Annunciator, AttitudeIndicator, ClusterKind, Compass, Contact, Countermeasures,
    DialGauge, DialLook, GearIndicator, HeadingTape, InstrumentCluster, LevelBar, Limits,
    MissionTimer, NeedleResponse, Odometer, PowerMeter, Scale, Severity, SonarWaterfall, Station,
    StationState, StoresPanel, TacticalPlot, Threat, ThreatRosette, TirePressure, TrackList,
    VerticalTape, VitalsTile,
};

impl Showcase {
    /// Vehicle, industrial and medical instruments, one section each.
    pub(crate) fn sections_instruments(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        self.inst_seen = Some(std::time::Instant::now());
        let t = theme_of(cx);
        let me = cx.entity();
        let gap = px(t.spacing.md);
        let row = || div().flex().flex_wrap().items_center().gap(gap);
        let mut out: Vec<Section> = Vec::new();
        // Options: each shows only in the sections it changes. Animate switches every
        // instrument's motion (springs, sweeps, flashing); the demo values wander while it is on.
        let response =
            (self.inst_response > 0).then(|| NeedleResponse::ALL[self.inst_response - 1]);
        let live = dodai_gpui::instruments_animated();
        let time = dodai_gpui::animation_time();
        // A value that is `still` normally and drifts smoothly through `lo..=hi` while
        // animated: two slow waves of different speed, offset per `key`.
        let val = move |key: f32, still: f32, lo: f32, hi: f32| {
            if !live {
                return still;
            }
            let w = 0.6 * (time * (0.35 + 0.05 * (key % 5.0)) + key * 1.7).sin()
                + 0.4 * (time * (0.83 + 0.07 * (key % 3.0)) + key * 2.9).sin();
            lo + (hi - lo) * (0.5 + 0.5 * w)
        };
        let (sel, glow_sel) = (self.inst_response, self.inst_glow);
        let option = |label: &'static str, seg: gpui::AnyElement| {
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(Label::new(label))
                .child(seg)
        };
        let bar = |opts: Vec<gpui::AnyElement>| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_4()
                .children(opts)
        };
        let needle_opt = || {
            let m = me.clone();
            option(
                "Needle",
                Segmented::new(
                    "inst-response",
                    ["Design", "Instant", "Snappy", "Smooth", "Heavy", "Bouncy"],
                )
                .selected(sel)
                .on_select(move |i, _, cx| {
                    m.update(cx, |s, cx| {
                        s.inst_response = i;
                        cx.notify();
                    })
                })
                .into_any_element(),
            )
            .into_any_element()
        };
        // "Theme" follows the glow set under Surface effects.
        let glow_opt = || {
            let m = me.clone();
            option(
                "Glow",
                Segmented::new("inst-glow", ["Theme", "Off", "Soft", "Strong"])
                    .selected(glow_sel)
                    .on_select(move |i, _, cx| {
                        m.update(cx, |s, cx| {
                            s.inst_glow = i;
                            dodai_gpui::set_chart_glow(
                                cx,
                                match i {
                                    1 => dodai_gpui::ChartGlow::Off,
                                    2 => dodai_gpui::ChartGlow::Radius(4.0),
                                    3 => dodai_gpui::ChartGlow::Radius(10.0),
                                    _ => dodai_gpui::ChartGlow::Theme,
                                },
                            );
                        })
                    })
                    .into_any_element(),
            )
            .into_any_element()
        };
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
            .child(
                InstrumentCluster::themed()
                    .speed(val(20.0, 251.0, 0.0, 300.0))
                    .rpm(val(21.0, 6.9, 0.8, 7.8))
                    .fuel(val(22.0, 62.0, 10.0, 95.0))
                    .coolant(val(23.0, 96.0, 70.0, 115.0))
                    .oil(val(24.0, 4.8, 2.0, 6.0))
                    .width(700.0),
            );
        for kind in ClusterKind::ALL
            .into_iter()
            .filter(|k| *k != t.chart_style().cluster)
        {
            clusters = clusters.child(Muted::new(kind.name())).child(
                InstrumentCluster::new(kind)
                    .speed(val(20.0, 251.0, 0.0, 300.0))
                    .rpm(val(21.0, 6.9, 0.8, 7.8))
                    .fuel(val(22.0, 62.0, 10.0, 95.0))
                    .coolant(val(23.0, 96.0, 70.0, 115.0))
                    .oil(val(24.0, 4.8, 2.0, 6.0))
                    .width(700.0),
            );
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
                    .child(
                        LevelBar::new(fuel, val(3.0, 18.0, 5.0, 95.0))
                            .response_opt(response)
                            .label("Fuel")
                            .unit("%"),
                    )
                    .child(
                        LevelBar::new(coolant, val(4.0, 108.0, 60.0, 122.0))
                            .response_opt(response)
                            .label("Coolant")
                            .unit("°C"),
                    )
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
                    .child(
                        PowerMeter::new(power, val(5.0, 84.0, -50.0, 110.0)).response_opt(response),
                    )
                    .child(Compass::new(val(6.0, 42.0, 0.0, 360.0)).response_opt(response)),
            );

        out.push((
            "Instruments",
            "Vehicle",
            self.section("Vehicle", cx)
                .child(bar(vec![needle_opt()]))
                .child(Muted::new("Clusters: one coherent style per set."))
                .child(clusters)
                .child(Muted::new("Dials"))
                .child(
                    row()
                        .child(
                            DialGauge::new(speed, val(1.0, 132.0, 0.0, 230.0))
                                .response_opt(response)
                                .label("Speed")
                                .unit("km/h")
                                .size(240.0),
                        )
                        .child(
                            DialGauge::new(rpm, val(2.0, 7.1, 0.8, 7.6))
                                .response_opt(response)
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
            self.section("Industrial", cx)
                .child(bar(vec![needle_opt()]))
                .child(
                    row()
                        .child(
                            DialGauge::new(pressure, val(7.0, 54.0, 5.0, 95.0))
                                .response_opt(response)
                                .label("Header pressure")
                                .unit("bar")
                                .size(190.0),
                        )
                        .child(
                            DialGauge::new(temp, val(8.0, 196.0, 40.0, 235.0))
                                .response_opt(response)
                                .label("Reactor temp")
                                .unit("°C")
                                .size(190.0),
                        )
                        .child(
                            DialGauge::new(flow, val(9.0, 97.0, 10.0, 98.0))
                                .response_opt(response)
                                .label("Feed flow")
                                .unit("m³/h")
                                .size(190.0),
                        )
                        .child(
                            LevelBar::new(tank_a, val(10.0, 72.0, 20.0, 95.0))
                                .response_opt(response)
                                .label("Tank A")
                                .unit("%")
                                .height(130.0),
                        )
                        .child(
                            LevelBar::new(tank_b, val(11.0, 94.0, 30.0, 98.0))
                                .response_opt(response)
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
            self.section("Gauges", cx)
                .child(bar(vec![glow_opt()]))
                .child(gauge_styles)
                .into_any_element(),
            "Gauges",
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
            self.section("Medical", cx)
                .child(bar(vec![glow_opt()]))
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
        // Radar and telemetry instruments (Cyberpunk only, see `catalog::owner`).
        out.push((
            "Instruments",
            "Radar & telemetry",
            self.section("Radar & telemetry", cx)
                .child(bar(vec![glow_opt()]))
                .child(hud)
                .into_any_element(),
            "Radar & telemetry",
        ));
        // Military: tactical plot, threat warning, weapon stations and the mission clock.
        let tracks = [
            Contact::new(0.06, 0.55, Affiliation::Hostile)
                .heading(0.55)
                .id(101),
            Contact::new(0.12, 0.8, Affiliation::Hostile)
                .heading(0.6)
                .id(102),
            Contact::new(0.33, 0.4, Affiliation::Friendly)
                .heading(0.25)
                .id(201),
            Contact::new(0.46, 0.7, Affiliation::Unknown).id(301),
            Contact::new(0.7, 0.5, Affiliation::Neutral)
                .heading(0.9)
                .id(401),
            Contact::new(0.85, 0.85, Affiliation::Friendly)
                .heading(0.1)
                .id(202),
        ];
        let pick_track = {
            let m = me.clone();
            move |i: usize, _: &mut gpui::Window, cx: &mut gpui::App| {
                m.update(cx, |s, cx| {
                    s.track_sel = i;
                    cx.notify();
                })
            }
        };
        let plot = TacticalPlot::new(tracks)
            .selected(Some(self.track_sel))
            .on_select(pick_track.clone())
            .caption("Tactical plot");
        let track_list = TrackList::new(tracks)
            .selected(Some(self.track_sel))
            .on_select(pick_track);
        let pick_station = {
            let m = me.clone();
            move |i: usize, _: &mut gpui::Window, cx: &mut gpui::App| {
                m.update(cx, |s, cx| {
                    s.station_sel = i;
                    cx.notify();
                })
            }
        };
        let rosette = ThreatRosette::new([
            Threat::new(0.05, 0.9),
            Threat::new(0.31, 0.5),
            Threat::new(0.62, 0.2),
            Threat::new(0.84, 0.65),
        ])
        .caption("Threat warning");
        out.push((
            "Instruments",
            "Military",
            self.section("Military", cx)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .justify_around()
                                .gap_6()
                                .child(plot)
                                .child(track_list)
                                .child(rosette)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_6()
                                        .child(MissionTimer::new(754))
                                        .child(MissionTimer::new(48).label("Weapons release"))
                                        .child(Countermeasures::new(24, 6)),
                                ),
                        )
                        .child(
                            SonarWaterfall::new(64, 36)
                                .size(560.0, 220.0)
                                .caption("Sonar waterfall"),
                        )
                        .child(
                            StoresPanel::new([
                                Station::new("AAM", StationState::Armed),
                                Station::new("AAM", StationState::Ready),
                                Station::new("ASM", StationState::Safe),
                                Station::new("BOMB", StationState::Safe),
                                Station::new("FUEL", StationState::Ready),
                                Station::new("GUN", StationState::Empty),
                            ])
                            .selected(Some(self.station_sel))
                            .on_select(pick_station),
                        ),
                )
                .into_any_element(),
            "Military",
        ));
        // Aviation: the flight deck instruments.
        out.push((
            "Instruments",
            "Aviation",
            self.section("Aviation", cx)
                .child(bar(vec![needle_opt()]))
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_6()
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .justify_around()
                                .gap_6()
                                .child(
                                    VerticalTape::new(val(12.0, 312.0, 220.0, 420.0))
                                        .response_opt(response)
                                        .scale(100.0, 10.0, 5)
                                        .label("KT"),
                                )
                                .child(
                                    AttitudeIndicator::new(
                                        val(13.0, 7.0, -12.0, 14.0),
                                        val(14.0, -14.0, -40.0, 40.0),
                                    )
                                    .response_opt(response)
                                    .size(300.0)
                                    .caption("Attitude"),
                                )
                                .child(
                                    VerticalTape::new(val(15.0, 12460.0, 11000.0, 14000.0))
                                        .response_opt(response)
                                        .scale(1000.0, 100.0, 5)
                                        .label("FT")
                                        .ticks_right(true),
                                ),
                        )
                        .child(
                            div().flex().justify_center().child(
                                HeadingTape::new(val(16.0, 47.0, 0.0, 360.0))
                                    .response_opt(response)
                                    .bug(75.0)
                                    .width(480.0),
                            ),
                        ),
                )
                .into_any_element(),
            "Aviation",
        ));
        out.push((
            "Instruments",
            "Signal lab",
            self.section("Signal lab", cx)
                .child(bar(vec![glow_opt()]))
                .child(lab)
                .into_any_element(),
            "Signal lab",
        ));
        out
    }
}
