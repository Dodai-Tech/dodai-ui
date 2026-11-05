//! The dash gallery: whole dashes built from the pods of a digital cluster, and the pods alone.

use super::super::*;
use dodai_gpui::icons::IconId;
use dodai_gpui::icons::Symbol;
use dodai_gpui_instruments::{
    BoostPod, CompassPod, DashQuad, DashTriptych, GForcePod, MediaPod, MenuRing, NoticePod,
    PodIcon, PodRow, RingTach, RowsPod, Scale, Severity, SidePod, Telltales, Tone, TyrePod,
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
    SidePod::new(Scale::new(6.0, 18.0).ticks_per(4, 1), val(40.0, 12.0, 14.6))
        .icon(Symbol::Battery)
        .unit("V")
}

fn speed() -> SidePod {
    SidePod::new(
        Scale::new(0.0, 270.0).ticks_per(9, 1),
        val(32.0, 20.0, 230.0),
    )
    .reading("")
    .unit("km/h")
    .ends(Some(Tone::Info), Some(Tone::Info))
}

fn oil() -> SidePod {
    SidePod::new(Scale::new(0.0, 8.0).ticks_per(4, 2), val(33.0, 1.0, 6.5))
        .icon(Symbol::OilCan)
        .unit("bar")
        .ends(Some(Tone::Info), Some(Tone::Danger))
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
    TyrePod::new()
        .heading("Tyre pressure")
        .note("Measured from 25 km/h")
}

fn vehicle_rows() -> RowsPod {
    RowsPod::new([
        PodRow::new(Symbol::OilCan, "8 °C").bar(0.2, Tone::Info),
        PodRow::new(Symbol::Coolant, "9 °C").bar(0.25, Tone::Info),
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
        PodRow::new(PodIcon::from(Symbol::Battery), "0 %"),
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

/// Whole dashes, then each new pod on its own.
pub(crate) fn dash_gallery(t: &dodai_core::Theme) -> gpui::AnyElement {
    let gap = px(t.spacing.lg);
    let col = || div().flex().flex_col().gap(gap);
    let wrap = || div().flex().flex_wrap().items_center().gap(gap);
    let quad = |l: [dodai_gpui_instruments::Pod; 2], r: [dodai_gpui_instruments::Pod; 2]| {
        DashQuad::new(tach(), l, r)
            .telltales(tells())
            .limit(50)
            .mode("HYBRID AUTO")
            .width(1000.0)
    };
    col()
        .child(Muted::new(
            "Four pods around the ring: battery, speed, oil and coolant.",
        ))
        .child(quad(
            [volt().into(), speed().into()],
            [oil().into(), coolant().into()],
        ))
        .child(Muted::new(
            "Four readings: boost, G-force, the trip list and a compass.",
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
        .child(Muted::new("Three-pod dashes: each left and right pair."))
        .child(dash(DashTriptych::new(
            tach(),
            tyres()
                .bars([Some(2.4), Some(2.4), Some(2.3), Some(2.2)])
                .low_below(2.3),
            vehicle_rows(),
        )))
        .child(dash(DashTriptych::new(tach(), tyres(), notice())))
        .child(dash(DashTriptych::new(
            tach(),
            BoostPod::new(0.0).temp("9 °C"),
            MenuRing::new(["Information", "Vehicle", "Oil level"]).selected(1),
        )))
        .child(dash(DashTriptych::new(
            tach(),
            MenuRing::new(["G-Force", "G-Force Max.", "Reset max."]),
            media(),
        )))
        .child(dash(DashTriptych::new(
            tach(),
            MenuRing::new(["G-Force", "G-Force Max.", "Reset max."]),
            today_rows(),
        )))
        .child(dash(DashTriptych::new(
            tach(),
            GForcePod::new(val(44.0, -0.8, 0.8), val(45.0, -0.5, 0.9)),
            CompassPod::new(343.0).height("460 m"),
        )))
        .child(Muted::new("The pods alone."))
        .child(
            wrap()
                .child(
                    tyres()
                        .bars([Some(2.4), Some(2.4), Some(2.2), Some(2.4)])
                        .low_below(2.3)
                        .size(240.0),
                )
                .child(vehicle_rows().size(240.0))
                .child(trip_rows().size(240.0))
                .child(notice().size(240.0))
                .child(GForcePod::new(val(46.0, -0.8, 0.8), val(47.0, -0.5, 0.9)).size(240.0))
                .child(
                    BoostPod::new(val(48.0, 0.0, 100.0))
                        .temp("9 °C")
                        .size(240.0),
                )
                .child(media().size(240.0))
                .child(
                    CompassPod::new(val(49.0, 0.0, 359.0))
                        .height("460 m")
                        .size(240.0),
                ),
        )
        .into_any_element()
}
