//! The "Vehicle" category: each dash instrument alone with an explanation, then whole dashes.

use super::super::*;
use dodai_gpui::icons::IconId;
use dodai_gpui::icons::Symbol;
use dodai_gpui_instruments::{
    BoostPod, CompassPod, DashQuad, DashTriptych, DualGauge, DualScale, GForcePod, LimitSign,
    MediaPod, MenuRing, NoticePod, PodIcon, PodRow, RingTach, RowsPod, Scale, Severity, SidePod,
    Telltales, Tone, TyrePod,
};

/// A demo value drifting through `lo..=hi` on the animation clock.
fn val(seed: f32, lo: f32, hi: f32) -> f32 {
    dodai_gpui::live::wander_between(seed, lo, hi)
}

fn tach() -> RingTach {
    let rpm = Scale::new(0.0, 8.0)
        .ticks_per(8, 2)
        .zone(7.0, 8.0, Severity::Alarm);
    RingTach::new(rpm, val(30.0, 0.8, 6.6), val(31.0, 0.0, 180.0))
        .marker(4.0)
        .tab("--- km")
        .fuel(0.7, "542 km")
}

fn tells() -> Telltales {
    Telltales::new()
        .push(Symbol::SeatBelt, Tone::Danger)
        .push(Symbol::SeatBelt, Tone::Danger)
        .push(Symbol::SeatBelt, Tone::Danger)
        .push(Symbol::LowBeam, Tone::Success)
        .push(Symbol::ParkingBrake, Tone::Danger)
}

fn volt() -> SidePod {
    SidePod::new(Scale::new(6.0, 18.0).ticks_per(2, 3), val(40.0, 12.0, 14.6))
        .icon(Symbol::Battery)
        .unit("V")
}

fn speed() -> SidePod {
    SidePod::new(
        Scale::new(0.0, 270.0).ticks_per(9, 1),
        val(32.0, 20.0, 230.0),
    )
    .needle(false)
    .unit("km/h")
    .centre(LimitSign::new(50).size(64.0))
    .ends(Some(Tone::Info), Some(Tone::Info))
}

fn oil() -> DualGauge {
    DualGauge::new(
        DualScale::new(
            Scale::new(0.0, 6.0).ticks_per(3, 3),
            val(33.0, 1.0, 5.0),
            "bar",
        ),
        DualScale::new(
            Scale::new(60.0, 150.0).ticks_per(2, 3),
            val(35.0, 70.0, 140.0),
            "°C",
        ),
        Symbol::OilCan,
    )
}

fn coolant() -> SidePod {
    SidePod::new(
        Scale::new(60.0, 130.0)
            .ticks_per(3, 3)
            .zone(115.0, 130.0, Severity::Alarm),
        val(34.0, 75.0, 118.0),
    )
    .icon(Symbol::Coolant)
    .unit("°C")
    .ends(Some(Tone::Info), Some(Tone::Danger))
}

fn tyres() -> TyrePod {
    TyrePod::new().heading("Tyre pressure")
}

fn vehicle_rows() -> RowsPod {
    RowsPod::new([
        PodRow::new(Symbol::OilCan, "8 °C")
            .bar(0.2, Tone::Info)
            .span(),
        PodRow::new(Symbol::Coolant, "9 °C")
            .bar(0.25, Tone::Info)
            .span(),
        PodRow::new(Symbol::OilCan, "1.5 bar").bar(0.35, Tone::Off),
        PodRow::new(Symbol::Battery, "14.9 V").bar(0.8, Tone::Off),
    ])
    .heading("Vehicle")
}

fn trip_rows() -> RowsPod {
    RowsPod::new([
        PodRow::new(Symbol::Stopwatch, "3 min"),
        PodRow::new(Symbol::Trip, "0.0 km"),
        PodRow::new(Symbol::Range, "--- l/100km"),
        PodRow::new(Symbol::Tachometer, "--- km/h"),
    ])
    .heading("Since 19:06")
}

fn today_rows() -> RowsPod {
    RowsPod::new([
        PodRow::new(IconId::Calendar, "1.11.2026"),
        PodRow::new(IconId::Music, "Full album"),
        PodRow::new(PodIcon::from(Symbol::Battery), "0 %").bar(0.0, Tone::Off),
        PodRow::new(IconId::Compass, "NW     N"),
    ])
}

fn notice() -> NoticePod {
    NoticePod::new(Symbol::Washer, "Washer fluid low").text("Top up washer fluid")
}

fn media() -> MediaPod {
    MediaPod::new("Full album").artist("Learn Linux TV")
}

fn dash(d: DashTriptych) -> DashTriptych {
    d.telltales(tells())
        .limit(50)
        .mode("HYBRID AUTO")
        .trip("13.207 km", "6 °C", "19:06")
        .width(820.0)
}

/// An instrument at 240 px with what it shows beside it.
fn shown(face: gpui::AnyElement, text: &str) -> gpui::AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_center()
        .gap_6()
        .child(div().flex_none().w(px(240.0)).h(px(240.0)).child(face))
        .child(div().w(px(380.0)).child(Muted::new(text.to_string())))
        .into_any_element()
}

/// A lone small element, centred in the 240 px square.
fn centred(e: impl IntoElement) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .justify_center()
        .size_full()
        .child(e)
        .into_any_element()
}

impl Showcase {
    /// The "Vehicle" category: one section per instrument, then whole dashes built from them.
    pub(crate) fn sections_vehicle(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        let mut out: Vec<Section> = Vec::new();
        let mut add = |me: &mut Self, title: &'static str, parts: Vec<gpui::AnyElement>| {
            let panel = parts
                .into_iter()
                .fold(me.section(title, cx), |p, e| p.child(e));
            out.push(("Vehicle", title, panel.into_any_element(), "Vehicle"));
        };
        add(
            self,
            "Engine speed",
            vec![shown(
                tach().size(240.0).into_any_element(),
                "The middle of the dash. A ribbed ring carries the revs (0 to 8, in thousands) with a red band at the top and a green marker at the sweet spot; a thin needle crosses it. Inside: the road speed in big figures, a trip tab and, across the open bottom, the fuel level with the range.",
            )],
        );
        add(
            self,
            "Speed",
            vec![shown(
                speed().size(240.0).into_any_element(),
                "A pod with the speed scale only: 0 to 270 in steps of 30 round an open ring, blue caps at both ends, and the current speed limit in the middle. No needle: the speed is the big figure on the engine-speed ring.",
            )],
        );
        add(
            self,
            "Battery voltage",
            vec![shown(
                volt().size(240.0).into_any_element(),
                "The on-board voltage, 6 to 18 V, with the battery symbol and the unit in the middle and a needle on the rim.",
            )],
        );
        add(
            self,
            "Oil pressure and temperature",
            vec![shown(
                oil().size(240.0).into_any_element(),
                "Two readings on one ring. Upper half: oil pressure in bar (0 to 6). Lower arc: oil temperature in °C (60 to 150). Each has its own needle; the oil can sits between the two units.",
            )],
        );
        add(
            self,
            "Coolant temperature",
            vec![shown(
                coolant().size(240.0).into_any_element(),
                "Coolant temperature, 60 to 130 °C. The cold end is blue, the hot end red, and the last part of the scale is a red warning band.",
            )],
        );
        add(
            self,
            "Menu",
            vec![
                shown(
                    MenuRing::new(["Information", "Vehicle", "Oil level"])
                        .selected(0)
                        .chosen(1)
                        .size(240.0)
                        .into_any_element(),
                    "A short list in a ring. A grey bar marks the row under the cursor; the row that is switched on is written in blue.",
                ),
                shown(
                    MenuRing::new(["G-Force", "G-Force Max.", "Reset max."])
                        .selected(1)
                        .pressed(true)
                        .size(240.0)
                        .into_any_element(),
                    "While the button is held the bar fills with the accent colour and gets blue edges.",
                ),
            ],
        );
        add(
            self,
            "Tyre pressure",
            vec![
                shown(
                    tyres()
                        .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                        .low_below(2.3)
                        .size(240.0)
                        .into_any_element(),
                    "The car from above with the pressure of each tyre beside its wheel, in bar. A tyre under the limit is written in the warning colour.",
                ),
                shown(
                    TyrePod::new()
                        .heading("Pressure deviation")
                        .note("Measured from 25 km/h")
                        .size(240.0)
                        .into_any_element(),
                    "Before the car is moving, or when no sensor answers, each tyre shows dashes and a note says from which speed it is measured.",
                ),
            ],
        );
        add(
            self,
            "Vehicle info",
            vec![shown(
                vehicle_rows().size(240.0).into_any_element(),
                "Four levels on dark tiles: oil temperature, coolant temperature, oil pressure and battery voltage. Each has a bar; the temperature bars carry a blue tick at the cold end and a red one at the hot end.",
            )],
        );
        add(
            self,
            "Trip",
            vec![shown(
                trip_rows().size(240.0).into_any_element(),
                "The trip since a time: duration, distance, consumption and average speed, each with its own symbol.",
            )],
        );
        add(
            self,
            "Today",
            vec![shown(
                today_rows().size(240.0).into_any_element(),
                "Several things in one list: the date, the track playing, the battery charge and a heading strip with a marker for the direction.",
            )],
        );
        add(
            self,
            "Message",
            vec![shown(
                notice().size(240.0).into_any_element(),
                "A message: a tab with a back arrow at the top, a symbol, the title and one line of advice. The rim is blue at both ends.",
            )],
        );
        add(
            self,
            "Boost",
            vec![shown(
                BoostPod::new(val(48.0, 0.0, 100.0))
                    .temp("9 °C")
                    .size(240.0)
                    .into_any_element(),
                "The boost (or charge) level as a percentage above a stack of cells that fill from the bottom, the label on the left and the battery temperature on the right.",
            )],
        );
        add(
            self,
            "G-force",
            vec![shown(
                GForcePod::new(val(46.0, -0.8, 0.8), val(47.0, -0.5, 0.9))
                    .size(240.0)
                    .into_any_element(),
                "A dot moves on a dotted cross inside a thin circle. The force sideways and the force forwards or back are written at its four ends.",
            )],
        );
        add(
            self,
            "Media",
            vec![shown(
                media().size(240.0).into_any_element(),
                "What is playing: the cover, the title in blue, the artist or show, and the elapsed and total time.",
            )],
        );
        add(
            self,
            "Compass",
            vec![shown(
                CompassPod::new(val(49.0, 0.0, 359.0))
                    .height("460 m")
                    .size(240.0)
                    .into_any_element(),
                "A card that turns with the heading: the four cardinal points on a thin circle, a blue arrow, the bearing in degrees, the cardinal point and the height.",
            )],
        );
        add(
            self,
            "Limit sign and lamps",
            vec![
                shown(
                    centred(LimitSign::new(50).size(110.0)),
                    "The speed-limit sign: a heavy red ring round the limit. On the dash it sits above the ring where the limit applies.",
                ),
                shown(
                    centred(tells().size(34.0).columns(3)),
                    "Telltale lamps: red for danger, green for on, amber for a fault.",
                ),
            ],
        );
        let gap = px(t.spacing.lg);
        let col = || div().flex().flex_col().gap(gap);
        let quad = |l: [dodai_gpui_instruments::Pod; 2], r: [dodai_gpui_instruments::Pod; 2]| {
            DashQuad::new(tach(), l, r)
                .telltales(tells())
                .limit(50)
                .mode("HYBRID AUTO")
                .width(1000.0)
        };
        let examples = col()
            .child(Muted::new(
                "The classic five instruments: voltage, speed, engine speed, oil and coolant.",
            ))
            .child(quad(
                [volt().into(), speed().into()],
                [oil().into(), coolant().into()],
            ))
            .child(Muted::new(
                "Four readings round the ring: boost, G-force, the trip and a compass.",
            ))
            .child(quad(
                [
                    BoostPod::new(0.0).temp("9 °C").into(),
                    GForcePod::new(val(41.0, -0.9, 0.9), val(42.0, -0.6, 1.1)).into(),
                ],
                [
                    trip_rows().into(),
                    CompassPod::new(val(43.0, 300.0, 359.0))
                        .height("460 m")
                        .into(),
                ],
            ))
            .child(Muted::new("Three instruments: a pair beside the ring."))
            .child(dash(DashTriptych::new(
                tach(),
                tyres()
                    .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                    .low_below(2.3),
                vehicle_rows(),
            )))
            .child(dash(DashTriptych::new(
                tach(),
                TyrePod::new()
                    .heading("Pressure deviation")
                    .note("Measured from 25 km/h"),
                notice(),
            )))
            .child(dash(DashTriptych::new(
                tach(),
                BoostPod::new(0.0).temp("9 °C"),
                MenuRing::new(["Information", "Vehicle", "Oil level"])
                    .selected(0)
                    .chosen(1),
            )))
            .child(dash(DashTriptych::new(
                tach(),
                MenuRing::new(["G-Force", "G-Force Max.", "Reset max."])
                    .selected(0)
                    .chosen(0),
                media(),
            )))
            .child(dash(DashTriptych::new(
                tach(),
                MenuRing::new(["G-Force", "G-Force Max.", "Reset max."]).selected(0),
                today_rows(),
            )))
            .child(dash(DashTriptych::new(
                tach(),
                GForcePod::new(val(44.0, -0.8, 0.8), val(45.0, -0.5, 0.9)),
                CompassPod::new(343.0).height("460 m"),
            )))
            .into_any_element();
        add(self, "Examples", vec![examples]);
        let small: Vec<gpui::AnyElement> = vec![
            volt().size(130.0).into_any_element(),
            oil().size(130.0).into_any_element(),
            tyres().size(130.0).into_any_element(),
            vehicle_rows().size(130.0).into_any_element(),
            notice().size(130.0).into_any_element(),
            BoostPod::new(48.0).size(130.0).into_any_element(),
            GForcePod::new(0.4, -0.3).size(130.0).into_any_element(),
            media().size(130.0).into_any_element(),
            CompassPod::new(45.0).size(130.0).into_any_element(),
        ];
        let strip = div()
            .flex()
            .flex_wrap()
            .gap(px(t.spacing.md))
            .children(small)
            .into_any_element();
        add(self, "Small sizes", vec![strip]);
        out
    }
}
