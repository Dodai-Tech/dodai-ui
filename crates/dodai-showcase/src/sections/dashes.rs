//! The vehicle part of the "Instruments" category: each dash instrument alone with an explanation, then whole dashes.

use super::super::*;
use dodai_gpui::icons::IconId;
use dodai_gpui::icons::Symbol;
use dodai_instruments::gpui::{
    AtLimit, BoostPod, CompassPod, DashQuad, DashTriptych, Drivetrain, DualGauge, DualScale,
    EnergyPod, GForcePod, GaugeFace, Gear, GearLayout, GearSelector, Layout, LimitSign, Look,
    MediaPod, MenuRing, NoticePod, PodIcon, PodLayout, PodRow, PodStack, PowerFlow, RingTach,
    RowsPod, Scale, Severity, ShiftHint, ShiftLights, SidePod, Telltales, Tone, TyrePod,
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
        .light(Symbol::SeatBelt)
        .light(Symbol::Abs)
        .push(Symbol::HighBeam, Tone::Off)
        .light(Symbol::LowBeam)
        .light(Symbol::ParkingBrake)
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
    .sweep(false)
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

/// An electric car's power ring: 60 kW recovered to 150 kW given, lit from zero.
fn ev_tach() -> RingTach {
    RingTach::new(
        Scale::new(-60.0, 150.0).ticks_per(7, 3),
        val(44.0, -40.0, 120.0),
        val(31.0, 0.0, 180.0),
    )
    .drivetrain(Drivetrain::Electric)
    .charge(0.68, "312 km")
    .tab("Range")
}

/// The diesel's exhaust fluid, amber when low.
fn fluid() -> SidePod {
    SidePod::new(
        Scale::new(0.0, 100.0)
            .ticks_per(4, 1)
            .zone(0.0, 10.0, Severity::Caution),
        val(45.0, 18.0, 64.0),
    )
    .icon(Symbol::ExhaustFluid)
    .unit("%")
}

/// The diesel particulate filter's load, amber when it needs a long drive to clean.
fn particulate() -> SidePod {
    SidePod::new(
        Scale::new(0.0, 100.0)
            .ticks_per(4, 1)
            .zone(80.0, 100.0, Severity::Caution),
        val(46.0, 30.0, 88.0),
    )
    .icon(Symbol::Particulate)
    .unit("%")
}

/// A hybrid's power flow, drifting through its modes.
fn flow() -> PowerFlow {
    PowerFlow::new(
        val(47.0, -20.0, 60.0).max(0.0),
        val(48.0, -30.0, 40.0),
        62.0,
    )
}

fn notice() -> NoticePod {
    NoticePod::new(Symbol::Washer, "Washer fluid low").text("Top up washer fluid")
}

fn media() -> MediaPod {
    MediaPod::new("Full album").artist("Learn Linux TV")
}

fn dash(d: DashTriptych, mode: &'static str) -> DashTriptych {
    d.telltales(tells())
        .limit(50)
        .mode(mode)
        .trip("13.207 km", "6 °C", "19:06")
        .width(820.0)
}

/// The drivetrains the example dashes can show: name and the mode text of the display.
pub(crate) const DRIVES: [(&str, &str); 4] = [
    ("Petrol", "SPORT"),
    ("Diesel", "DIESEL"),
    ("Hybrid", "HYBRID AUTO"),
    ("Electric", "RANGE"),
];

/// Seconds each drivetrain is shown while the picker is on Auto.
const DRIVE_SECS: f32 = 20.0;

/// Seconds the old drivetrain takes to fade out over the new one.
const FADE_SECS: f32 = 2.0;

/// How opaque the outgoing drivetrain still is `now` seconds into an Auto tour, or `None` once
/// the new one stands alone. Eases both ends. Pure.
fn outgoing(now: f32) -> Option<f32> {
    let k = (now.rem_euclid(DRIVE_SECS) / FADE_SECS).clamp(0.0, 1.0);
    (k < 1.0).then_some(1.0 - k * k * (3.0 - 2.0 * k))
}

/// `make(drive)`; on an Auto tour, for the first moments of a new drivetrain the previous one
/// is drawn over it and fades away, so the change is a dissolve and not a cut.
fn crossfade(
    drive: usize,
    auto: bool,
    now: f32,
    make: &dyn Fn(usize) -> gpui::AnyElement,
) -> gpui::AnyElement {
    match outgoing(now).filter(|_| auto && now >= DRIVE_SECS) {
        Some(o) => div()
            .relative()
            .child(make(drive))
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .opacity(o)
                    .child(make((drive + DRIVES.len() - 1) % DRIVES.len())),
            )
            .into_any_element(),
        None => make(drive),
    }
}

/// The drivetrain to show: picker entry `sel` of [`DRIVES`], or for the entry past them (Auto)
/// each in turn as `now` seconds pass. Pure.
fn drive_index(sel: usize, now: f32) -> usize {
    if sel < DRIVES.len() {
        sel
    } else {
        (now / DRIVE_SECS) as usize % DRIVES.len()
    }
}

/// The ring of drivetrain `i` of [`DRIVES`]: revs, the charge beside them for a hybrid, the
/// motor's power for an electric car.
fn drive_tach(i: usize) -> RingTach {
    match i {
        2 => tach().drivetrain(Drivetrain::Hybrid).charge(0.62, "38 km"),
        3 => ev_tach(),
        _ => tach(),
    }
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
    gallery_rows(text, vec![items])
}

/// A gallery whose rows start afresh: one look per row, each wrapping on its own when the
/// width runs out.
fn gallery_rows(text: &str, rows: Vec<Vec<(&'static str, gpui::AnyElement)>>) -> gpui::AnyElement {
    let pod = cell(rows.iter().map(Vec::len).sum());
    div()
        .flex()
        .flex_col()
        .gap_4()
        .child(div().max_w(px(780.0)).child(Muted::new(text.to_string())))
        .children(rows.into_iter().map(|items| {
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
                }))
        }))
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
        (Look::Sport, Layout::Track) => "Sport track",
        (Look::Chronograph, Layout::Sector) => "Chronograph sector",
        (Look::Vintage, Layout::Bezel) => "Vintage bezel",
        (Look::HeadsUp, Layout::Arc) => "Heads-up arc",
        (Look::Segment, Layout::Arc) => "Segment arc",
        (Look::GlassCockpit, Layout::Bar) => "Glass cockpit bar",
        (Look::Race, Layout::Arc) => "Race arc",
        (Look::Race, Layout::Bar) => "Race bar",
        (Look::Race, Layout::Column) => "Race column",
        (Look::Line, Layout::Arc) => "Line arc",
        (Look::Line, Layout::Column) => "Line column",
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
) -> Vec<Vec<(&'static str, gpui::AnyElement)>> {
    let mut dial = Some(dial);
    let looks = shown(only);
    let pod = cell(looks.iter().map(|l| l.layouts().len()).sum());
    let mut out = Vec::new();
    for l in looks {
        let mut row: Vec<(&'static str, gpui::AnyElement)> = Vec::new();
        for &layout in l.layouts() {
            let el = match (l, dial.take()) {
                (Look::Sport, Some(d)) => d(pod),
                (_, d) => {
                    dial = d;
                    face(l, layout, pod)
                }
            };
            row.push((caption(l, layout), el));
        }
        out.push(row);
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

/// Each shown [`Look`] on its own row, in every [`PodLayout`]: the round pod captioned with the
/// look, then the rings, the strip and the column, built by `make(look, layout, size)`.
fn pod_rows(
    only: Option<Look>,
    make: impl Fn(Look, PodLayout, f32) -> gpui::AnyElement,
) -> Vec<Vec<(&'static str, gpui::AnyElement)>> {
    let looks = shown(only);
    let pod = cell(looks.len() * PodLayout::ALL.len());
    looks
        .into_iter()
        .map(|l| {
            PodLayout::ALL
                .into_iter()
                .map(|y| {
                    let name = if y == PodLayout::Round {
                        l.name()
                    } else {
                        y.name()
                    };
                    (name, make(l, y, pod))
                })
                .collect()
        })
        .collect()
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
        // The look picked above the sections; `None` ("All", the default) shows every look in
        // order, and the whole dashes in the design's own.
        let only = self
            .inst_look
            .checked_sub(1)
            .and_then(|i| Look::ALL.get(i).copied());
        let dash_look = only.unwrap_or(t.chart_style().vehicle);
        // The cell of the galleries built here, one instrument per look.
        let pod = cell(shown(only).len());
        let mut out: Vec<Section> = Vec::new();
        let mut add = |me: &mut Self, title: &'static str, parts: Vec<gpui::AnyElement>| {
            let panel = parts
                .into_iter()
                .fold(me.section(title, cx), |p, e| p.child(e));
            out.push(("Instruments", title, panel.into_any_element(), "Vehicle"));
        };
        // Dials: the scales round a ring.
        add(
            self,
            "Vehicle · Engine speed",
            vec![gallery_rows(
                "Every vehicle instrument comes in nine looks, in this order: a sports dash ring, a chronograph, a vintage panel, a heads-up display, a segment LCD, a halo of light, a glass cockpit, a race display and a single line. Pick one above to see every instrument and the dashes in it. Each look comes in four layouts, one look per row. Here the revs, 0 to 8 in thousands, amber from 6 and red from 7. Sport: the ribbed ring of the dash with the road speed in the middle, a trip tab and the fuel level, then a sweep, a track of dots and grooves, none with a hand. The others show the revs alone: chronograph as a dial, a bezel, a sector dial and a register; vintage as a plate, a chrome bezel, a panel meter and a ribbon; heads-up as a ring, a cut, an arc and a tape; segment as a dial, an arc, a bar and a column; halo as a ring, a capsule, an arc and a bar; glass cockpit as an arc, a bar, a tape and a column; race as a plate, an arc, a bar and a column; line as a ring, an arc, a bar and a column.",
                looks(only, |z| tach().look(Look::Sport).size(z).into_any_element(), rev_face),
            )],
        );
        add(
            self,
            "Vehicle · Speed",
            vec![gallery_rows(
                "The speed, 0 to 270, amber above 220. Sport: the dash pod in steps of 30 with the speed limit in the middle and no needle. The other looks draw it as a gauge face; each look is followed by its other layouts.",
                looks(only, |z| speed().look(Look::Sport).size(z).into_any_element(), speed_face),
            )],
        );
        add(
            self,
            "Vehicle · Battery voltage",
            vec![gallery_rows(
                "The on-board voltage, 6 to 18 V. Red under 11 V, amber over 15 V.",
                looks(
                    only,
                    |z| volt().look(Look::Sport).size(z).into_any_element(),
                    volt_face,
                ),
            )],
        );
        add(
            self,
            "Vehicle · Oil pressure and temperature",
            vec![gallery_rows(
                "Oil pressure (bar) and oil temperature (°C). Sport: two needles on one ring, pressure on the upper half and temperature on the lower arc; its sweep, track and ribbed layouts and the other looks draw the pressure on one face and the temperature on a strip under it in the look's style; the heads-up tapes stand side by side.",
                looks(only, |z| oil().look(Look::Sport).size(z).into_any_element(), oil_faces),
            )],
        );
        add(
            self,
            "Vehicle · Coolant temperature",
            vec![gallery_rows(
                "Coolant temperature, 60 to 130 °C. The last part of the scale is a red warning band; the figure turns red inside it.",
                looks(only, |z| coolant().look(Look::Sport).size(z).into_any_element(), coolant_face),
            )],
        );
        add(
            self,
            "Vehicle · Boost",
            vec![gallery_rows(
                "The boost (or charge) level in percent. Sport: the cell stack inside a segmented band with a limit mark and the battery temperature. The other looks draw it as a gauge face; each look is followed by its other layouts. Past 80 % the screen looks here colour their whole display amber, an option (at_limit(AtLimit::Display)); by default only the value takes the limit's colour.",
                looks(
                    only,
                    |z| {
                        BoostPod::new(val(48.0, 0.0, 100.0))
                            .look(Look::Sport)
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
                gallery_rows(
                    "The battery of an electric car: charge, power and range. Sport: a battery of cells under the charge, the power arc growing right from the top while driving. Every other look shows charge, power and range as rows. Each look draws the pod round, as rings, as a strip and as a column (PodLayout); the round looks make the strip and the column round registers.",
                    pod_rows(only, |s, y, z| {
                        EnergyPod::new(val(60.0, 55.0, 80.0))
                            .power(val(61.0, 10.0, 120.0))
                            .range("312 km")
                            .look(s)
                            .layout(y)
                            .size(z)
                            .into_any_element()
                    }),
                ),
                gallery_rows(
                    "Plugged in: the power flows back in, so the arc grows left in the success colour, and the foot counts down to the charge limit marked on the battery. Under a fifth of charge the cells turn amber, under a tenth red.",
                    pod_rows(only, |s, y, z| {
                        EnergyPod::new(val(62.0, 12.0, 18.0))
                            .power(-48.0)
                            .charging(true)
                            .limit(80.0)
                            .time_to_full("42 min")
                            .range("54 km")
                            .look(s)
                            .layout(y)
                            .size(z)
                            .into_any_element()
                    }),
                ),
            ],
        );
        add(
            self,
            "Vehicle · Gear",
            vec![gallery(
                "The gear selector in each look: P R N D as a row with drive lit, shifted by hand with a prompt to change up, then as a stack in reverse and the engaged gear alone with a prompt to change down. Shifted by hand the lit position reads M and the gear.",
                shown(only)
                    .into_iter()
                    .map(|look| {
                        let sel = |g: Gear| GearSelector::new(g).look(look).size(16.0);
                        let col = div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(t.spacing.md))
                            .child(sel(Gear::Drive))
                            .child(sel(Gear::Manual(4)).hint(ShiftHint::Up))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(t.spacing.lg))
                                    .child(sel(Gear::Reverse).layout(GearLayout::Stack).size(12.0))
                                    .child(
                                        sel(Gear::Manual(3))
                                            .layout(GearLayout::Single)
                                            .hint(ShiftHint::Down),
                                    ),
                            );
                        (look.name(), centred(col))
                    })
                    .collect(),
            )],
        );
        add(
            self,
            "Vehicle · Power flow",
            vec![gallery_rows(
                "A hybrid's power between the engine, the battery and the wheels, the mode as the heading: electric, engine, boost with both, recovering into the battery, or the engine charging it. Sport draws the paths; the other looks list the engine, the motor and the charge; every look also as rings, a strip and a column.",
                pod_rows(only, |s, y, z| flow().look(s).layout(y).size(z).into_any_element()),
            )],
        );
        add(
            self,
            "Vehicle · Diesel",
            vec![
                gallery_rows(
                    "The exhaust fluid a diesel needs to clean its exhaust, amber under a tenth; each look in its four layouts.",
                    looks(
                        only,
                        |z| fluid().look(Look::Sport).size(z).into_any_element(),
                        |l, y, z| fluid().look(l).layout(y).size(z).into_any_element(),
                    ),
                ),
                gallery_rows(
                    "The particulate filter's load, amber from 80 %: time for a long drive to burn it clean.",
                    looks(
                        only,
                        |z| particulate().look(Look::Sport).size(z).into_any_element(),
                        |l, y, z| particulate().look(l).layout(y).size(z).into_any_element(),
                    ),
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
                gallery_rows(
                    "The pressure of each tyre in bar: front left and right on top, rear left and right below. A tyre under the limit is written in the warning colour. As rings, a strip or a column each tyre is a row with a bar, short and amber when soft.",
                    pod_rows(only, |s, y, z| {
                        tyres()
                            .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                            .low_below(2.3)
                            .look(s)
                            .layout(y)
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
            vec![gallery_rows(
                "Four levels on dark tiles: oil temperature, coolant temperature, oil pressure and battery voltage. Each has a bar; the temperature bars carry a blue tick at the cold end and a red one at the hot end.",
                pod_rows(only, |s, y, z| {
                    vehicle_rows().look(s).layout(y).size(z).into_any_element()
                }),
            )],
        );
        add(
            self,
            "Vehicle · Trip",
            vec![gallery_rows(
                "The trip since a time: duration, distance, consumption and average speed, each with its own symbol.",
                pod_rows(only, |s, y, z| {
                    trip_rows().look(s).layout(y).size(z).into_any_element()
                }),
            )],
        );
        add(
            self,
            "Vehicle · Today",
            vec![gallery_rows(
                "Several things in one list: the date, the track playing, the battery charge and a heading strip with a marker for the direction.",
                pod_rows(only, |s, y, z| {
                    today_rows().look(s).layout(y).size(z).into_any_element()
                }),
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
                    "The common telltales, each lit in its standard colour: red for danger, amber for a fault, green for a system on, blue for main beam, grey when off.",
                    shown(only)
                        .into_iter()
                        .map(|look| {
                            let row = Telltales::common()
                                .light(Symbol::SeatBelt)
                                .light(Symbol::Abs)
                                .light(Symbol::Engine)
                                .light(Symbol::LowBeam)
                                .light(Symbol::HighBeam)
                                .light(Symbol::TurnLeft);
                            (look.name(), centred(row.look(look).size(pod * 0.12).columns(4)))
                        })
                        .collect(),
                ),
            ],
        );
        let gap = px(t.spacing.lg);
        let col = || div().flex().flex_col().gap(gap);
        let auto = self.inst_drive >= DRIVES.len();
        let drive = drive_index(self.inst_drive, dodai_gpui::motion::now());
        let drive_name = DRIVES[drive].0;
        let play = |secs: f32| auto.then_some(secs);
        let now = dodai_gpui::motion::now();
        let quad = |d: usize,
                    l: [dodai_instruments::gpui::Pod; 2],
                    r: [dodai_instruments::gpui::Pod; 2]| {
            DashQuad::new(drive_tach(d), l, r)
                .look(dash_look)
                .telltales(tells())
                .limit(50)
                .mode(DRIVES[d].1)
                .width(1000.0)
        };
        let tri = |d: usize, t: DashTriptych| dash(t, DRIVES[d].1).look(dash_look);
        // One dash of each size; every side display is a stack of the look's pods (all of them
        // between the stacks), and scrolling, dragging or the chevrons snap the next into place.
        let energy = || {
            EnergyPod::new(68.0)
                .power(val(44.0, -40.0, 120.0))
                .range("312 km")
        };
        let since = || {
            RowsPod::new([
                PodRow::new(Symbol::Stopwatch, "3 min"),
                PodRow::new(Symbol::Trip, "0.0 km"),
                PodRow::new(Symbol::Range, "--- kWh/100km"),
                PodRow::new(Symbol::Recuperation, "0.0 kWh"),
            ])
            .heading("Since 19:06")
        };
        let gforce = |k: f32| GForcePod::new(val(k, -0.9, 0.9), val(k + 1.0, -0.6, 1.1));
        let compass = || CompassPod::new(val(43.0, 300.0, 359.0)).height("460 m");
        let boost = || BoostPod::new(0.0).temp("9 °C");
        let examples = {
            let five = |d: usize| {
                quad(
                    d,
                    [
                        PodStack::new("ex5-l0")
                            .autoplay(play(8.0))
                            .pod(volt())
                            .pod(boost())
                            .pod(fluid())
                            .pod(energy())
                            .pod(tyres())
                            .into(),
                        PodStack::new("ex5-l1")
                            .autoplay(play(10.0))
                            .pod(speed())
                            .pod(gforce(41.0))
                            .pod(flow())
                            .pod(compass())
                            .into(),
                    ],
                    [
                        PodStack::new("ex5-r0")
                            .autoplay(play(12.0))
                            .pod(oil())
                            .pod(trip_rows())
                            .pod(particulate())
                            .pod(since())
                            .into(),
                        PodStack::new("ex5-r1")
                            .autoplay(play(14.0))
                            .pod(coolant())
                            .pod(compass())
                            .pod(notice())
                            .pod(today_rows())
                            .into(),
                    ],
                )
                .into_any_element()
            };
            let three = |d: usize| {
                tri(
                    d,
                    DashTriptych::new(
                        drive_tach(d),
                        PodStack::new("ex3-l")
                            .autoplay(play(10.0))
                            .pod(
                                tyres()
                                    .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                                    .low_below(2.3),
                            )
                            .pod(
                                TyrePod::new()
                                    .heading("Pressure deviation")
                                    .note("Measured from 25 km/h"),
                            )
                            .pod(boost())
                            .pod(gforce(44.0))
                            .pod(energy())
                            .pod(flow())
                            .pod(
                                MenuRing::new(["G-Force", "G-Force Max.", "Reset max."])
                                    .selected(0)
                                    .chosen(0),
                            ),
                        PodStack::new("ex3-r")
                            .autoplay(play(13.0))
                            .pod(vehicle_rows())
                            .pod(notice())
                            .pod(media())
                            .pod(today_rows())
                            .pod(trip_rows())
                            .pod(since())
                            .pod(compass())
                            .pod(
                                MenuRing::new(["Information", "Vehicle", "Oil level"])
                                    .selected(0)
                                    .chosen(1),
                            ),
                    ),
                )
                .into_any_element()
            };
            col()
            .child(Muted::new(
                format!("Five instruments round the ring, here with a {} drivetrain (the Drive picker above; Auto cycles them, centre ring included, and turns the pages by itself). Scroll, drag or use the chevrons on a side display to turn its pages.", drive_name.to_lowercase()),
            ))
            .child(crossfade(drive, auto, now, &five))
            .child(Muted::new(
                "Three instruments: a stack of displays on each side of the ring.",
            ))
            .child(crossfade(drive, auto, now, &three))
            .into_any_element()
        };
        add(self, "Vehicle · Examples", vec![examples]);
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
        assert_eq!(
            caption(Look::Race, Layout::Tape),
            "Race",
            "a layout a look lacks is its dial"
        );
    }

    #[test]
    fn auto_drive_cycles_through_every_drivetrain() {
        assert_eq!(drive_index(2, 999.0), 2);
        let auto = DRIVES.len();
        let seen: Vec<_> = (0..DRIVES.len())
            .map(|k| drive_index(auto, k as f32 * DRIVE_SECS + 1.0))
            .collect();
        assert_eq!(seen, [0, 1, 2, 3]);
        assert_eq!(drive_index(auto, DRIVES.len() as f32 * DRIVE_SECS), 0);
    }

    #[test]
    fn a_new_drivetrain_dissolves_in() {
        assert_eq!(outgoing(0.0), Some(1.0));
        assert!(outgoing(FADE_SECS * 0.5).is_some_and(|o| (o - 0.5).abs() < 1e-3));
        assert_eq!(outgoing(FADE_SECS), None);
        assert_eq!(outgoing(DRIVE_SECS + 0.0), Some(1.0));
    }

    #[test]
    fn one_look_or_every_look_is_shown() {
        assert_eq!(shown(Some(Look::Race)), vec![Look::Race]);
        assert_eq!(shown(None), Look::ALL.to_vec());
    }
}
