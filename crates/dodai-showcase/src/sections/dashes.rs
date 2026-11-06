//! The vehicle part of the "Instruments" category: each dash instrument alone with an explanation, then whole dashes.

use super::super::*;
use dodai_gpui::icons::IconId;
use dodai_gpui::icons::Symbol;
use dodai_gpui_instruments::{
    AtLimit, BoostPod, CompassPod, DashQuad, DashTriptych, DualGauge, DualScale, EnergyPod,
    GForcePod, GaugeFace, Layout, LimitSign, Look, MediaPod, MenuRing, NoticePod, PodIcon, PodRow,
    PodStyle, RingTach, RowsPod, Scale, Severity, ShiftLights, SidePod, Telltales, Tone, TyrePod,
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
        .push(Symbol::Abs, Tone::Warning)
        .push(Symbol::HighBeam, Tone::Off)
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
        )
        .label("Pressure")
        .decimals(1),
        DualScale::new(
            Scale::new(60.0, 150.0)
                .ticks_per(2, 3)
                .zone(130.0, 150.0, Severity::Caution),
            val(35.0, 70.0, 140.0),
            "°C",
        )
        .label("Temperature"),
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

/// Pod diameter in the galleries.
const POD: f32 = 200.0;

/// Pod diameter of an instrument alone on its row.
const POD_ALONE: f32 = 320.0;

/// The pod diameter for a row of `n` instruments: bigger when one is alone. Pure.
fn cell(n: usize) -> f32 {
    if n == 1 {
        POD_ALONE
    } else {
        POD
    }
}

/// An explanation, then the instrument in several designs side by side, each with its name.
fn gallery(text: &str, items: Vec<(&'static str, gpui::AnyElement)>) -> gpui::AnyElement {
    let pod = cell(items.len());
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(div().max_w(px(780.0)).child(Muted::new(text.to_string())))
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_6()
                .children(items.into_iter().map(|(name, el)| {
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap_2()
                        .child(
                            div()
                                .flex_none()
                                .w(px(pod))
                                .h(px(pod))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(el),
                        )
                        .child(Muted::new(name))
                })),
        )
        .into_any_element()
}

/// The looks the sections show: the one picked, or every look. Pure.
fn shown(only: Option<Look>) -> Vec<Look> {
    only.map_or_else(|| Look::ALL.to_vec(), |l| vec![l])
}

/// The caption of a look drawn in one of its layouts, e.g. "Vintage ribbon". Pure.
fn caption(look: Look, layout: Layout) -> &'static str {
    match (look, layout) {
        (Look::Vintage, Layout::Meter) => "Vintage meter",
        (Look::Vintage, Layout::Ribbon) => "Vintage ribbon",
        (Look::HeadsUp, Layout::Tape) => "Heads-up tape",
        (Look::HeadsUp, Layout::Cut) => "Heads-up cut",
        (Look::Halo, Layout::Capsule) => "Halo capsule",
        (Look::Halo, Layout::Arc) => "Halo arc",
        (Look::Halo, Layout::Bar) => "Halo bar",
        (Look::Sport, Layout::Sweep) => "Sport sweep",
        (Look::Sport, Layout::Needle) => "Sport needle",
        (Look::Sport, Layout::Ribbed) => "Sport ribbed",
        (Look::Chronograph, Layout::Bezel) => "Chronograph bezel",
        (Look::Chronograph, Layout::Register) => "Chronograph register",
        (Look::Segment, Layout::Bar) => "Segment bar",
        (Look::Segment, Layout::Column) => "Segment column",
        (Look::GlassCockpit, Layout::Tape) => "Glass cockpit tape",
        (Look::GlassCockpit, Layout::Column) => "Glass cockpit column",
        (Look::Line, Layout::Bar) => "Line bar",
        (l, _) => l.name(),
    }
}

/// A reading in each shown [`Look`], each followed by its other layouts (see
/// [`Look::layouts`]): the sport dial comes from `dial(size)`, the rest from
/// `face(look, layout, size)`.
fn looks(
    only: Option<Look>,
    dial: impl FnOnce(f32) -> gpui::AnyElement,
    face: impl Fn(Look, Layout, f32) -> gpui::AnyElement,
) -> Vec<(&'static str, gpui::AnyElement)> {
    let mut dial = Some(dial);
    let looks = shown(only);
    let pod = cell(looks.iter().map(|l| l.layouts().len()).sum());
    let mut out: Vec<(&'static str, gpui::AnyElement)> = Vec::new();
    for l in looks {
        for &layout in l.layouts() {
            let el = match (l, dial.take()) {
                (Look::Sport, Some(d)) => d(pod),
                (_, d) => {
                    dial = d;
                    face(l, layout, pod)
                }
            };
            out.push((caption(l, layout), el));
        }
    }
    out
}

fn speed_face(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    GaugeFace::new(
        Scale::new(0.0, 270.0)
            .ticks_per(9, 1)
            .zone(220.0, 270.0, Severity::Caution),
        val(32.0, 20.0, 230.0),
    )
    .look(look)
    .at_limit(AtLimit::Display)
    .layout(layout)
    .label("Speed")
    .icon(Symbol::Tachometer)
    .unit("km/h")
    .size(size)
    .into_any_element()
}

fn rev_face(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    GaugeFace::new(
        Scale::new(0.0, 8.0)
            .ticks_per(8, 2)
            .zone(6.0, 7.0, Severity::Caution)
            .zone(7.0, 8.0, Severity::Alarm),
        val(30.0, 0.8, 6.6),
    )
    .look(look)
    .at_limit(AtLimit::Display)
    .layout(layout)
    .label("Engine speed")
    .icon(Symbol::Tachometer)
    .unit("×1000 rpm")
    .decimals(1)
    .size(size)
    .into_any_element()
}

fn volt_face(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    GaugeFace::new(
        Scale::new(6.0, 18.0)
            .ticks_per(2, 3)
            .zone(6.0, 11.0, Severity::Alarm)
            .zone(15.0, 18.0, Severity::Caution),
        val(40.0, 12.0, 14.6),
    )
    .look(look)
    .at_limit(AtLimit::Display)
    .layout(layout)
    .label("Battery")
    .icon(Symbol::Battery)
    .unit("V")
    .decimals(1)
    .size(size)
    .into_any_element()
}

fn coolant_face(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    GaugeFace::new(
        Scale::new(60.0, 130.0)
            .ticks_per(3, 3)
            .zone(115.0, 130.0, Severity::Alarm),
        val(34.0, 75.0, 118.0),
    )
    .look(look)
    .at_limit(AtLimit::Display)
    .layout(layout)
    .label("Coolant")
    .icon(Symbol::Coolant)
    .unit("°C")
    .ends(Some(Tone::Info), Some(Tone::Danger))
    .size(size)
    .into_any_element()
}

fn boost_face(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    GaugeFace::new(
        Scale::new(0.0, 100.0)
            .ticks_per(5, 2)
            .zone(80.0, 100.0, Severity::Caution),
        val(48.0, 0.0, 100.0),
    )
    .look(look)
    .at_limit(AtLimit::Display)
    .layout(layout)
    .label("Boost")
    .icon(Symbol::Boost)
    .unit("%")
    .size(size)
    .into_any_element()
}

/// Oil pressure and temperature on two faces: side by side or stacked.
fn oil_faces(look: Look, layout: Layout, size: f32) -> gpui::AnyElement {
    oil()
        .look(look)
        .at_limit(AtLimit::Display)
        .layout(layout)
        .size(size)
        .into_any_element()
}

/// One pod in each shown [`Look`], built by `make(look, size)`.
fn looked(
    only: Option<Look>,
    make: impl Fn(Look, f32) -> gpui::AnyElement,
) -> Vec<(&'static str, gpui::AnyElement)> {
    let looks = shown(only);
    let pod = cell(looks.len());
    looks
        .into_iter()
        .map(|s| (s.name(), make(s, pod)))
        .collect()
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
    /// The vehicle part of "Instruments": one section per instrument, then whole dashes built from them.
    pub(crate) fn sections_vehicle(&mut self, cx: &mut Context<Self>) -> Vec<Section> {
        let t = theme_of(cx);
        // The look picked above the sections (the design's own by default); `None` shows every
        // look.
        let own = t.chart_style().vehicle;
        let only = match self.inst_look {
            0 => None,
            1 => Some(own),
            i => Look::ALL.get(i - 2).copied(),
        };
        let dash_look = only.unwrap_or(own);
        // The cell of the galleries built here, one instrument per look.
        let pod = cell(shown(only).len());
        let mut out: Vec<Section> = Vec::new();
        let mut add = |me: &mut Self, title: &'static str, parts: Vec<gpui::AnyElement>| {
            let panel = parts
                .into_iter()
                .fold(me.section(title, cx), |p, e| p.child(e));
            out.push(("Instruments", title, panel.into_any_element(), "Vehicle"));
        };
        // First every look side by side, whatever is picked: the one place that shows them all.
        add(
            self,
            "Vehicle · Looks",
            vec![gallery(
                "Every vehicle instrument comes in nine looks: the same data, drawn as a sports dash ring, a chronograph, a vintage panel, a heads-up display, a segment LCD, a halo of light, a glass cockpit, a race display or a single line. Here the engine speed in each. Most looks come in more layouts: sport as a sweep, a needle and a ribbed ring, chronograph as a bezel and a register, vintage as a panel meter and a ribbon, heads-up as a cut and a tape, segment LCD as a bar and a column, halo as a capsule, an arc and a bar, glass cockpit as a tape and a column, line as a bar. Pick a look above to see every instrument and the example dashes in it.",
                Look::ALL
                    .into_iter()
                    .map(|l| {
                        let el = match l {
                            Look::Sport => tach().size(POD).into_any_element(),
                            _ => rev_face(l, Layout::Dial, POD),
                        };
                        (l.name(), el)
                    })
                    .collect(),
            )],
        );
        // Dials: the scales round a ring.
        add(
            self,
            "Vehicle · Engine speed",
            vec![gallery(
                "The revs, 0 to 8 in thousands, amber from 6 and red from 7. Sport: the ribbed ring of the dash with the road speed in the middle, a trip tab and the fuel level. The others show the revs alone: a chronograph with a sub-dial, a vintage dial with a spade hand and a drum counter, a heads-up ring, a segment display and a halo of light. Vintage adds a panel meter and a ribbon, heads-up a tape, halo an arc and a bar.",
                looks(only, |z| tach().size(z).into_any_element(), rev_face),
            )],
        );
        add(
            self,
            "Vehicle · Speed",
            vec![gallery(
                "The speed, 0 to 270, amber above 220. Sport: the dash pod in steps of 30 with the speed limit in the middle and no needle. The other looks draw it as a gauge face; each look is followed by its other layouts.",
                looks(only, |z| speed().size(z).into_any_element(), speed_face),
            )],
        );
        add(
            self,
            "Vehicle · Battery voltage",
            vec![gallery(
                "The on-board voltage, 6 to 18 V. Red under 11 V, amber over 15 V.",
                looks(only, |z| volt().size(z).into_any_element(), volt_face),
            )],
        );
        add(
            self,
            "Vehicle · Oil pressure and temperature",
            vec![gallery(
                "Oil pressure (bar) and oil temperature (°C). Sport: two needles on one ring, pressure on the upper half and temperature on the lower arc; its sweep, needle and ribbed layouts and the other looks draw the pressure on one face and the temperature on a strip under it in the look's style; the heads-up tapes stand side by side.",
                looks(only, |z| oil().size(z).into_any_element(), oil_faces),
            )],
        );
        add(
            self,
            "Vehicle · Coolant temperature",
            vec![gallery(
                "Coolant temperature, 60 to 130 °C. The last part of the scale is a red warning band; the figure turns red inside it.",
                looks(only, |z| coolant().size(z).into_any_element(), coolant_face),
            )],
        );
        add(
            self,
            "Vehicle · Boost",
            vec![gallery(
                "The boost (or charge) level in percent. Sport: the cell stack inside a segmented band with a limit mark and the battery temperature. The other looks draw it as a gauge face; each look is followed by its other layouts. Past 80 % the screen looks here colour their whole display amber, an option (at_limit(AtLimit::Display)); by default only the value takes the limit's colour.",
                looks(
                    only,
                    |z| {
                        BoostPod::new(val(48.0, 0.0, 100.0))
                            .temp("9 °C")
                            .size(z)
                            .into_any_element()
                    },
                    boost_face,
                ),
            )],
        );
        add(
            self,
            "Vehicle · Energy",
            vec![
                gallery(
                    "The battery of an electric car: charge, power and range. Sport: a battery of cells under the charge, the power arc growing right from the top while driving. Every other look shows charge, power and range as rows.",
                    looked(only, |s, z| {
                        EnergyPod::new(val(60.0, 55.0, 80.0))
                            .power(val(61.0, 10.0, 120.0))
                            .range("312 km")
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
                gallery(
                    "Plugged in: the power flows back in, so the arc grows left in the success colour. Under a fifth of charge the cells turn amber, under a tenth red.",
                    looked(only, |s, z| {
                        EnergyPod::new(val(62.0, 12.0, 18.0))
                            .power(-48.0)
                            .charging(true)
                            .range("54 km")
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
            ],
        );
        add(
            self,
            "Vehicle · Shift lights",
            vec![
                gallery(
                    "Lights over the engine speed: green, amber, then red towards the shift point, with the gear under them. Each look draws its own lights.",
                    shown(only)
                        .into_iter()
                        .map(|look| {
                            (
                                look.name(),
                                centred(
                                    ShiftLights::new(val(63.0, 4800.0, 6950.0))
                                        .gear("4")
                                        .look(look)
                                        .width(pod * 0.9),
                                ),
                            )
                        })
                        .collect(),
                ),
                gallery(
                    "At the shift point every light turns blue, and so does the gear: change up now.",
                    shown(only)
                        .into_iter()
                        .map(|look| {
                            (
                                look.name(),
                                centred(
                                    ShiftLights::new(7200.0)
                                        .gear("4")
                                        .look(look)
                                        .width(pod * 0.9),
                                ),
                            )
                        })
                        .collect(),
                ),
            ],
        );
        add(
            self,
            "Vehicle · Tyre pressure",
            vec![
                gallery(
                    "The pressure of each tyre in bar: front left and right on top, rear left and right below. A tyre under the limit is written in the warning colour.",
                    looked(only, |s, z| {
                        tyres()
                            .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                            .low_below(2.3)
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
                gallery(
                    "Before the car is moving, or when no sensor answers, each tyre shows dashes and a note says from which speed it is measured.",
                    looked(only, |s, z| {
                        TyrePod::new()
                            .heading("Pressure deviation")
                            .note("Measured from 25 km/h")
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
            ],
        );
        add(
            self,
            "Vehicle · G-force",
            vec![gallery(
                "A dot moves on a dotted cross inside a thin circle. The force sideways and the force forwards or back are written at its four ends.",
                looked(only, |s, z| {
                    GForcePod::new(val(46.0, -0.8, 0.8), val(47.0, -0.5, 0.9))
                        .look(s)
                        .size(z)
                        .into_any_element()
                }),
            )],
        );
        add(
            self,
            "Vehicle · Compass",
            vec![gallery(
                "A card that turns with the heading: the four cardinal points on a thin circle, a blue arrow, the bearing in degrees, the cardinal point and the height.",
                looked(only, |s, z| {
                    CompassPod::new(val(49.0, 0.0, 359.0))
                        .height("460 m")
                        .look(s)
                        .size(z)
                        .into_any_element()
                }),
            )],
        );
        // Lists of readings.
        add(
            self,
            "Vehicle · Vehicle info",
            vec![gallery(
                "Four levels on dark tiles: oil temperature, coolant temperature, oil pressure and battery voltage. Each has a bar; the temperature bars carry a blue tick at the cold end and a red one at the hot end.",
                looked(only, |s, z| vehicle_rows().look(s).size(z).into_any_element()),
            )],
        );
        add(
            self,
            "Vehicle · Trip",
            vec![gallery(
                "The trip since a time: duration, distance, consumption and average speed, each with its own symbol.",
                looked(only, |s, z| trip_rows().look(s).size(z).into_any_element()),
            )],
        );
        add(
            self,
            "Vehicle · Today",
            vec![gallery(
                "Several things in one list: the date, the track playing, the battery charge and a heading strip with a marker for the direction.",
                looked(only, |s, z| today_rows().look(s).size(z).into_any_element()),
            )],
        );
        // Messages, media and menus.
        add(
            self,
            "Vehicle · Message",
            vec![gallery(
                "A message: a tab with a back arrow at the top, a symbol, the title and one line of advice. The rim is blue at both ends.",
                looked(only, |s, z| notice().look(s).size(z).into_any_element()),
            )],
        );
        add(
            self,
            "Vehicle · Media",
            vec![gallery(
                "What is playing: the cover, the title in blue, the artist or show, and the elapsed and total time.",
                looked(only, |s, z| media().look(s).size(z).into_any_element()),
            )],
        );
        add(
            self,
            "Vehicle · Menu",
            vec![
                gallery(
                    "A short list in a ring. A grey bar marks the row under the cursor; the row that is switched on is written in blue.",
                    looked(only, |s, z| {
                        MenuRing::new(["Information", "Vehicle", "Oil level"])
                            .selected(0)
                            .chosen(1)
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
                gallery(
                    "While the button is held the bar fills with the accent colour and gets blue edges.",
                    looked(only, |s, z| {
                        MenuRing::new(["G-Force", "G-Force Max.", "Reset max."])
                            .selected(1)
                            .pressed(true)
                            .look(s)
                            .size(z)
                            .into_any_element()
                    }),
                ),
            ],
        );
        // Signs and lamps.
        add(
            self,
            "Vehicle · Limit sign and lamps",
            vec![
                gallery(
                    "The speed-limit sign: a heavy red ring round the limit. On the dash it sits above the ring where the limit applies.",
                    shown(only)
                        .into_iter()
                        .zip([30, 50, 80, 120, 60, 100, 70, 90, 40])
                        .map(|(look, v)| {
                            (
                                look.name(),
                                centred(LimitSign::new(v).look(look).size(pod * 0.55)),
                            )
                        })
                        .collect(),
                ),
                gallery(
                    "Telltale lamps: red for danger, green for on, amber for a fault, grey when off.",
                    shown(only)
                        .into_iter()
                        .map(|look| (look.name(), centred(tells().look(look).size(pod * 0.17).columns(3))))
                        .collect(),
                ),
            ],
        );
        let gap = px(t.spacing.lg);
        let col = || div().flex().flex_col().gap(gap);
        let quad = |l: [dodai_gpui_instruments::Pod; 2], r: [dodai_gpui_instruments::Pod; 2]| {
            DashQuad::new(tach(), l, r)
                .look(dash_look)
                .telltales(tells())
                .limit(50)
                .mode("HYBRID AUTO")
                .width(1000.0)
        };
        let tri = |d: DashTriptych| dash(d).look(dash_look);
        let examples = col()
            .child(Muted::new(
                "The classic five instruments: voltage, speed, engine speed, oil and coolant.",
            ))
            .child(quad(
                [volt().into(), speed().into()],
                [oil().into(), coolant().into()],
            ))
            .child(Muted::new(
                "The same dash with closed faces: boost, G-force, the trip and a compass.",
            ))
            .child(quad(
                [
                    BoostPod::new(0.0).temp("9 °C").style(PodStyle::Disc).into(),
                    GForcePod::new(val(41.0, -0.9, 0.9), val(42.0, -0.6, 1.1))
                        .style(PodStyle::Disc)
                        .into(),
                ],
                [
                    trip_rows().style(PodStyle::Disc).into(),
                    CompassPod::new(val(43.0, 300.0, 359.0))
                        .height("460 m")
                        .style(PodStyle::Disc)
                        .into(),
                ],
            ))
            .child(Muted::new("Three instruments: a pair beside the ring."))
            .child(tri(DashTriptych::new(
                tach(),
                tyres()
                    .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                    .low_below(2.3),
                vehicle_rows(),
            )))
            .child(tri(DashTriptych::new(
                tach(),
                TyrePod::new()
                    .heading("Pressure deviation")
                    .note("Measured from 25 km/h")
                    .style(PodStyle::Glow),
                notice().style(PodStyle::Glow),
            )))
            .child(tri(DashTriptych::new(
                tach(),
                BoostPod::new(0.0).temp("9 °C"),
                MenuRing::new(["Information", "Vehicle", "Oil level"])
                    .selected(0)
                    .chosen(1),
            )))
            .child(tri(DashTriptych::new(
                tach(),
                MenuRing::new(["G-Force", "G-Force Max.", "Reset max."])
                    .selected(0)
                    .chosen(0)
                    .style(PodStyle::Bezel),
                media().style(PodStyle::Bezel),
            )))
            .child(tri(DashTriptych::new(
                tach(),
                MenuRing::new(["G-Force", "G-Force Max.", "Reset max."]).selected(0),
                today_rows(),
            )))
            .child(tri(DashTriptych::new(
                tach(),
                GForcePod::new(val(44.0, -0.8, 0.8), val(45.0, -0.5, 0.9)),
                CompassPod::new(343.0).height("460 m"),
            )))
            .into_any_element();
        add(self, "Vehicle · Examples", vec![examples]);
        let mut by_look = col();
        for look in shown(only) {
            by_look = by_look.child(Muted::new(look.name())).child(
                quad(
                    [volt().into(), speed().into()],
                    [oil().into(), coolant().into()],
                )
                .look(look),
            );
        }
        add(
            self,
            "Vehicle · One look for the whole dash",
            vec![
                Muted::new("A dash's look reaches its ring, pods, lamps and limit sign.")
                    .into_any_element(),
                by_look.into_any_element(),
            ],
        );
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_instrument_alone_on_its_row_is_bigger() {
        assert_eq!(cell(1), POD_ALONE);
        assert_eq!(cell(2), POD);
        assert_eq!(cell(Look::ALL.len()), POD);
    }

    #[test]
    fn every_layout_has_its_own_caption() {
        let mut all: Vec<_> = Look::ALL
            .into_iter()
            .flat_map(|l| l.layouts().iter().map(move |&y| caption(l, y)))
            .collect();
        let n = all.len();
        all.sort_unstable();
        all.dedup();
        assert_eq!(all.len(), n);
        assert_eq!(caption(Look::Race, Layout::Bar), "Race");
    }

    #[test]
    fn one_look_or_every_look_is_shown() {
        assert_eq!(shown(Some(Look::Race)), vec![Look::Race]);
        assert_eq!(shown(None), Look::ALL.to_vec());
    }
}
