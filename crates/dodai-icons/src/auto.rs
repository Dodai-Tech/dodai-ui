//! Automotive telltales and cluster symbols, drawn for Dodai on the 24×24 grid.
//!
//! These are original drawings of the usual dashboard ideas (belt, beam, oil, brake, charge ...):
//! stroked, round-cornered, one weight, each built from a few arcs, circles and lines. They are
//! not traced from, or meant to match, any manufacturer's artwork.
//!
//! ```
//! use dodai_icons::auto::AutoIcon;
//! let g = AutoIcon::OilCan.glyph();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(AutoIcon::ALL.len(), AutoIcon::ALL.iter().map(|i| i.name()).collect::<std::collections::HashSet<_>>().len());
//! ```

use crate::pack::{Glyph, Points};
use std::f32::consts::PI;

type Shape = (Points, bool);

/// Degrees to radians with 0° pointing right and angles growing clockwise on screen.
fn rad(d: f32) -> f32 {
    d * PI / 180.0
}

/// An open arc of radius `r` from `a0` to `a1` degrees, faceted every ~12°.
fn arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32) -> Shape {
    let n = (((a1 - a0).abs() / 12.0).ceil() as usize).max(2);
    let pts = (0..=n)
        .map(|i| {
            let a = rad(a0 + (a1 - a0) * i as f32 / n as f32);
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect();
    (pts, false)
}

/// A full circle.
fn circ(cx: f32, cy: f32, r: f32) -> Shape {
    let (mut pts, _) = arc(cx, cy, r, 0.0, 360.0);
    pts.pop();
    (pts, true)
}

/// A polyline.
fn line(p: &[(f32, f32)]) -> Shape {
    (p.to_vec(), false)
}

/// A closed polygon.
fn poly(p: &[(f32, f32)]) -> Shape {
    (p.to_vec(), true)
}

/// A rectangle with rounded corners of radius `r`.
fn rrect(x0: f32, y0: f32, x1: f32, y1: f32, r: f32) -> Shape {
    let mut pts = Vec::new();
    for (cx, cy, a0) in [
        (x1 - r, y0 + r, -90.0),
        (x1 - r, y1 - r, 0.0),
        (x0 + r, y1 - r, 90.0),
        (x0 + r, y0 + r, 180.0),
    ] {
        pts.extend(arc(cx, cy, r, a0, a0 + 90.0).0);
    }
    (pts, true)
}

/// A dot (a tiny closed diamond, so it survives as a stroke).
fn dot(x: f32, y: f32) -> Shape {
    poly(&[(x, y - 0.4), (x + 0.4, y), (x, y + 0.4), (x - 0.4, y)])
}

/// Light rays fanning out from `(x, y)` along `dir` degrees: the beam family's signature.
fn rays(x: f32, y: f32, dir: f32, spread: f32, n: usize, from: f32, to: f32) -> Vec<Shape> {
    (0..n)
        .map(|i| {
            let t = if n == 1 {
                0.0
            } else {
                i as f32 / (n - 1) as f32 - 0.5
            };
            let a = rad(dir + t * spread);
            line(&[
                (x + from * a.cos(), y + from * a.sin()),
                (x + to * a.cos(), y + to * a.sin()),
            ])
        })
        .collect()
}

/// A headlamp bulb: a D-shaped lens open to the right.
fn lamp(cx: f32) -> Shape {
    let mut pts = vec![(cx + 2.0, 6.5)];
    pts.extend(
        arc(cx + 2.0, 12.0, 5.5, -90.0, -270.0)
            .0
            .into_iter()
            .skip(1),
    );
    pts.push((cx + 2.0, 17.5));
    (pts, true)
}

macro_rules! autos {
    ($($(#[$m:meta])* $v:ident = $name:literal;)+) => {
        /// A dashboard symbol.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum AutoIcon {
            $($(#[$m])* $v,)+
        }
        impl AutoIcon {
            /// Every symbol, in catalogue order.
            pub const ALL: &'static [AutoIcon] = &[$(AutoIcon::$v),+];
            /// A short stable kebab-case name.
            pub fn name(self) -> &'static str {
                match self { $(AutoIcon::$v => $name,)+ }
            }
        }
    };
}

autos! {
    /// A seated figure with the belt across.
    SeatBelt = "seat-belt";
    /// Main beam: lamp with level rays.
    HighBeam = "high-beam";
    /// Dipped beam: lamp with rays angled down.
    LowBeam = "low-beam";
    /// Front fog lamp: rays crossed by a wavy line.
    FogFront = "fog-front";
    /// Rear fog lamp: rays pointing back.
    FogRear = "fog-rear";
    /// Side lights: two small lamps.
    SideLights = "side-lights";
    /// Hazard warning: nested triangles.
    Hazard = "hazard";
    /// Turn left.
    TurnLeft = "turn-left";
    /// Turn right.
    TurnRight = "turn-right";
    /// Parking brake: P in a ring with open sides.
    ParkingBrake = "parking-brake";
    /// Brake system fault: exclamation in a ring with open sides.
    BrakeFault = "brake-fault";
    /// Anti-lock braking.
    Abs = "abs";
    /// Airbag: figure with a round bag.
    Airbag = "airbag";
    /// 12 V battery.
    Battery = "battery";
    /// Oil pressure: a can with a drop.
    OilCan = "oil-can";
    /// Coolant temperature: thermometer over two waves.
    Coolant = "coolant";
    /// Engine block.
    Engine = "engine";
    /// Fuel pump.
    Fuel = "fuel";
    /// Charging plug.
    Plug = "plug";
    /// Traction battery with a bolt.
    TractionBattery = "traction-battery";
    /// Tyre pressure: a tyre cut-away with an exclamation mark.
    TyrePressure = "tyre-pressure";
    /// Open door.
    DoorOpen = "door-open";
    /// Open luggage lid.
    TrunkOpen = "trunk-open";
    /// Open bonnet.
    HoodOpen = "hood-open";
    /// Washer fluid.
    Washer = "washer";
    /// Cruise control: dial with a pointer.
    Cruise = "cruise";
    /// Speed limiter: dial with a bar.
    Limiter = "limiter";
    /// Lane keeping: car between two lines.
    LaneKeep = "lane-keep";
    /// Lane departure: lines with a crossing arrow.
    LaneDeparture = "lane-departure";
    /// Collision warning: car with a burst.
    Collision = "collision";
    /// Stability control: car with skid curves.
    Stability = "stability";
    /// All-wheel drive: four wheels with a link.
    AllWheel = "all-wheel";
    /// Steering fault: wheel with an exclamation mark.
    Steering = "steering";
    /// Diesel pre-glow: a coil.
    Glow = "glow";
    /// Ice warning: a snowflake.
    Frost = "frost";
    /// Key.
    Key = "key";
    /// Service due: a spanner.
    Service = "service";
    /// Gearbox: a shift gate.
    Gearbox = "gearbox";
    /// Boost pressure: a turbine.
    Boost = "boost";
    /// Energy recovery: arrows around a bolt.
    Recuperation = "recuperation";
    /// Sport mode: chequered flag.
    Sport = "sport";
    /// Eco mode: a leaf.
    Eco = "eco";
    /// Hybrid: leaf and plug.
    Hybrid = "hybrid";
    /// Emergency call: SOS ring.
    Sos = "sos";
    /// Child lock.
    ChildLock = "child-lock";
    /// Seat heating: seat with waves.
    SeatHeat = "seat-heat";
    /// Front defrost.
    DefrostFront = "defrost-front";
    /// Rear defrost.
    DefrostRear = "defrost-rear";
    /// Fan.
    Fan = "fan";
    /// Air recirculation.
    Recirculate = "recirculate";
    /// Compass rose.
    Compass = "compass";
    /// Altitude: two peaks.
    Altitude = "altitude";
    /// Lateral acceleration: car with a side arrow.
    GForce = "g-force";
    /// Trip meter: two pins and a road.
    Trip = "trip";
    /// Remaining range: a road to the horizon.
    Range = "range";
    /// Trailer.
    Trailer = "trailer";
    /// Headlamp levelling: lamp with a vertical arrow.
    Levelling = "levelling";
    /// Automatic stop-start.
    StopStart = "stop-start";
    /// Speed-limit sign.
    SpeedSign = "speed-sign";
    /// Vehicle ride height.
    RideHeight = "ride-height";
    /// Outside temperature: thermometer.
    OutsideTemp = "outside-temp";
}

impl AutoIcon {
    /// The drawing, as a stroked [`Glyph`] on the 24×24 grid.
    pub fn glyph(self) -> Glyph {
        Glyph::Stroked(shapes(self))
    }
}

/// A car seen from above-front, reused by several symbols: body outline from `x0` to `x1`.
fn car_side(x0: f32, x1: f32, y: f32) -> Vec<Shape> {
    let w = x1 - x0;
    let (a, b) = (x0 + w * 0.26, x0 + w * 0.74);
    let r = (w * 0.11).max(1.6);
    vec![
        line(&[
            (x0 + r * 0.2, y - 0.5),
            (x0, y - 0.5),
            (x0, y - 3.2),
            (x0 + w * 0.2, y - 4.0),
            (x0 + w * 0.34, y - 7.0),
            (x0 + w * 0.7, y - 7.0),
            (x0 + w * 0.86, y - 4.0),
            (x1, y - 3.2),
            (x1, y - 0.5),
            (x1 - r * 0.2, y - 0.5),
        ]),
        line(&[(a + r, y - 0.5), (b - r, y - 0.5)]),
        circ(a, y - 0.5, r),
        circ(b, y - 0.5, r),
    ]
}

fn shapes(i: AutoIcon) -> Vec<Shape> {
    use AutoIcon::*;
    match i {
        SeatBelt => vec![
            circ(10.5, 5.0, 2.0),
            line(&[(9.0, 9.0), (10.5, 15.5), (17.0, 15.5), (17.5, 21.0)]),
            line(&[(6.0, 8.5), (14.5, 12.5)]),
            line(&[(14.5, 12.5), (13.5, 15.5)]),
        ],
        HighBeam => {
            let mut v = vec![lamp(5.5)];
            v.extend(rays(9.5, 12.0, 0.0, 70.0, 5, 2.5, 11.5));
            v
        }
        LowBeam => {
            let mut v = vec![lamp(5.5)];
            v.extend(rays(9.5, 7.5, 20.0, 40.0, 4, 2.5, 11.5));
            v
        }
        FogFront => {
            let mut v = vec![lamp(5.5)];
            v.extend(rays(9.5, 8.0, 0.0, 44.0, 3, 2.5, 11.5));
            v.push(line(&[
                (10.0, 16.0),
                (12.0, 18.0),
                (14.0, 16.0),
                (16.0, 18.0),
                (18.0, 16.0),
                (20.0, 18.0),
            ]));
            v.push(line(&[
                (10.0, 20.5),
                (12.0, 22.5),
                (14.0, 20.5),
                (16.0, 22.5),
                (18.0, 20.5),
                (20.0, 22.5),
            ]));
            v
        }
        FogRear => {
            let mut v: Vec<Shape> = vec![lamp(5.5)]
                .into_iter()
                .map(|(p, c)| (p.into_iter().map(|(x, y)| (24.0 - x, y)).collect(), c))
                .collect();
            v.extend(rays(14.5, 12.0, 180.0, 60.0, 4, 2.5, 11.0));
            v
        }
        SideLights => vec![
            circ(7.0, 12.0, 3.0),
            circ(17.0, 12.0, 3.0),
            line(&[(2.0, 12.0), (3.5, 12.0)]),
            line(&[(20.5, 12.0), (22.0, 12.0)]),
            line(&[(12.0, 6.0), (12.0, 8.0)]),
            line(&[(12.0, 16.0), (12.0, 18.0)]),
        ],
        Hazard => vec![
            poly(&[(12.0, 3.0), (22.0, 20.0), (2.0, 20.0)]),
            poly(&[(12.0, 9.5), (16.5, 17.0), (7.5, 17.0)]),
        ],
        TurnLeft => vec![poly(&[
            (2.5, 12.0),
            (10.0, 4.5),
            (10.0, 9.0),
            (21.0, 9.0),
            (21.0, 15.0),
            (10.0, 15.0),
            (10.0, 19.5),
        ])],
        TurnRight => vec![poly(&[
            (21.5, 12.0),
            (14.0, 4.5),
            (14.0, 9.0),
            (3.0, 9.0),
            (3.0, 15.0),
            (14.0, 15.0),
            (14.0, 19.5),
        ])],
        ParkingBrake => vec![
            circ(12.0, 12.0, 5.0),
            line(&[
                (10.2, 15.2),
                (10.2, 8.8),
                (12.6, 8.8),
                (13.8, 10.0),
                (13.8, 11.0),
                (12.6, 12.2),
                (10.2, 12.2),
            ]),
            arc(12.0, 12.0, 9.0, 130.0, 230.0),
            arc(12.0, 12.0, 9.0, -50.0, 50.0),
        ],
        BrakeFault => vec![
            circ(12.0, 12.0, 5.0),
            line(&[(12.0, 9.0), (12.0, 12.5)]),
            dot(12.0, 15.0),
            arc(12.0, 12.0, 9.0, 130.0, 230.0),
            arc(12.0, 12.0, 9.0, -50.0, 50.0),
        ],
        Abs => vec![
            circ(12.0, 12.0, 4.0),
            line(&[
                (10.0, 12.0),
                (12.0, 10.0),
                (14.0, 12.0),
                (12.0, 14.0),
                (10.0, 12.0),
            ]),
            arc(12.0, 12.0, 8.5, 125.0, 235.0),
            arc(12.0, 12.0, 8.5, -55.0, 55.0),
            line(&[(2.5, 5.0), (5.0, 5.0)]),
            line(&[(19.0, 19.0), (21.5, 19.0)]),
        ],
        Airbag => vec![
            circ(8.0, 6.0, 2.0),
            line(&[(8.0, 9.5), (8.0, 15.0), (13.0, 18.0)]),
            line(&[(8.0, 11.0), (12.0, 13.0)]),
            circ(16.5, 12.0, 4.5),
        ],
        Battery => vec![
            rrect(2.5, 7.0, 21.5, 19.0, 1.5),
            line(&[(6.0, 7.0), (6.0, 4.5), (9.0, 4.5), (9.0, 7.0)]),
            line(&[(15.0, 7.0), (15.0, 4.5), (18.0, 4.5), (18.0, 7.0)]),
            line(&[(6.0, 13.0), (10.0, 13.0)]),
            line(&[(14.0, 13.0), (18.0, 13.0)]),
            line(&[(16.0, 11.0), (16.0, 15.0)]),
        ],
        OilCan => vec![
            line(&[
                (3.0, 9.0),
                (3.0, 17.0),
                (15.0, 17.0),
                (15.0, 11.5),
                (8.5, 9.0),
                (3.0, 9.0),
            ]),
            line(&[(3.0, 9.0), (3.0, 7.0)]),
            line(&[(15.0, 11.5), (21.5, 6.5)]),
            line(&[(8.5, 9.0), (8.5, 7.0), (11.5, 7.0)]),
            poly(&[(20.0, 11.0), (21.5, 13.5), (20.0, 15.0), (18.5, 13.5)]),
            line(&[(6.0, 20.5), (12.0, 20.5)]),
        ],
        Coolant => vec![
            line(&[(8.0, 13.0), (8.0, 5.0), (12.0, 5.0), (12.0, 13.0)]),
            arc(10.0, 16.0, 4.0, -66.0, 246.0),
            line(&[(10.0, 8.0), (10.0, 16.0)]),
            line(&[(14.0, 9.0), (16.0, 9.0)]),
            line(&[(14.0, 6.0), (16.0, 6.0)]),
            line(&[
                (14.5, 15.0),
                (16.5, 17.0),
                (18.5, 15.0),
                (20.5, 17.0),
                (22.0, 15.5),
            ]),
            line(&[(14.5, 19.5), (16.5, 21.5), (18.5, 19.5), (20.5, 21.5)]),
        ],
        Engine => vec![
            line(&[(5.0, 8.0), (9.0, 8.0), (10.5, 6.0), (15.0, 6.0)]),
            line(&[
                (5.0, 8.0),
                (5.0, 17.0),
                (15.5, 17.0),
                (18.0, 14.5),
                (18.0, 8.0),
                (15.0, 8.0),
                (15.0, 6.0),
            ]),
            line(&[(18.0, 11.0), (21.5, 11.0)]),
            line(&[(5.0, 12.5), (2.5, 12.5)]),
            line(&[(2.5, 10.0), (2.5, 15.0)]),
            line(&[(9.0, 12.0), (14.0, 12.0)]),
        ],
        Fuel => vec![
            rrect(4.0, 4.0, 14.0, 20.0, 1.5),
            line(&[(4.0, 9.5), (14.0, 9.5)]),
            line(&[(14.0, 8.0), (18.0, 8.0), (19.5, 10.0), (19.5, 17.0)]),
            line(&[(17.0, 17.0), (21.5, 17.0)]),
            line(&[(2.5, 20.0), (15.5, 20.0)]),
        ],
        Plug => vec![
            line(&[(8.0, 3.0), (8.0, 8.0)]),
            line(&[(16.0, 3.0), (16.0, 8.0)]),
            poly(&[
                (5.5, 8.0),
                (18.5, 8.0),
                (18.5, 12.5),
                (14.0, 17.0),
                (10.0, 17.0),
                (5.5, 12.5),
            ]),
            line(&[(12.0, 17.0), (12.0, 21.5)]),
        ],
        TractionBattery => vec![
            rrect(2.5, 7.0, 20.0, 18.0, 1.5),
            line(&[(20.0, 10.5), (22.0, 10.5), (22.0, 14.5), (20.0, 14.5)]),
            line(&[(12.5, 8.5), (8.5, 12.8), (12.0, 12.8), (10.5, 16.5)]),
        ],
        TyrePressure => vec![
            arc(12.0, 11.0, 8.5, 150.0, 390.0),
            line(&[(4.6, 15.2), (6.0, 18.5), (8.0, 18.5)]),
            line(&[(19.4, 15.2), (18.0, 18.5), (16.0, 18.5)]),
            line(&[(8.0, 18.5), (8.0, 21.0), (16.0, 21.0), (16.0, 18.5)]),
            line(&[(12.0, 7.0), (12.0, 12.0)]),
            dot(12.0, 15.0),
        ],
        DoorOpen => vec![
            line(&[(4.0, 21.0), (4.0, 3.5), (13.5, 3.5)]),
            poly(&[(13.5, 3.5), (20.5, 6.5), (20.5, 21.0), (13.5, 18.5)]),
            line(&[(13.5, 3.5), (13.5, 18.5)]),
            line(&[(16.0, 12.0), (18.0, 12.0)]),
        ],
        TrunkOpen => {
            let mut v = car_side(2.5, 21.5, 19.0);
            v.push(line(&[(18.5, 13.0), (21.0, 4.5), (15.0, 3.5)]));
            v
        }
        HoodOpen => {
            let mut v = car_side(2.5, 21.5, 19.0);
            v.push(line(&[(3.5, 16.2), (2.0, 7.0), (9.0, 5.0)]));
            v
        }
        Washer => vec![
            line(&[
                (3.0, 20.0),
                (6.0, 11.0),
                (18.0, 11.0),
                (21.0, 20.0),
                (3.0, 20.0),
            ]),
            line(&[(8.0, 17.5), (14.0, 13.5)]),
            circ(8.0, 6.0, 0.7),
            circ(12.0, 4.0, 0.7),
            circ(16.0, 6.0, 0.7),
            line(&[(10.0, 8.0), (10.5, 9.5)]),
            line(&[(14.0, 8.0), (13.5, 9.5)]),
        ],
        Cruise => vec![
            arc(12.0, 13.0, 9.0, 150.0, 390.0),
            line(&[(12.0, 13.0), (16.5, 8.5)]),
            circ(12.0, 13.0, 1.3),
            line(&[(2.5, 20.0), (5.0, 20.0)]),
            line(&[(19.0, 20.0), (21.5, 20.0)]),
        ],
        Limiter => vec![
            circ(12.0, 12.0, 9.0),
            line(&[(8.0, 12.0), (16.0, 12.0)]),
            line(&[(12.0, 6.5), (12.0, 8.5)]),
            line(&[(12.0, 15.5), (12.0, 17.5)]),
        ],
        LaneKeep => {
            let mut v = vec![
                line(&[(3.0, 21.0), (7.0, 3.0)]),
                line(&[(21.0, 21.0), (17.0, 3.0)]),
            ];
            v.extend([
                rrect(9.0, 9.0, 15.0, 19.0, 2.5),
                line(&[(9.5, 13.0), (14.5, 13.0)]),
            ]);
            v
        }
        LaneDeparture => vec![
            line(&[(3.0, 21.0), (7.0, 3.0)]),
            line(&[(21.0, 21.0), (17.0, 3.0)]),
            line(&[(4.0, 14.0), (15.0, 14.0)]),
            line(&[(12.0, 10.0), (16.0, 14.0), (12.0, 18.0)]),
        ],
        Collision => {
            let mut v = car_side(2.0, 15.0, 19.0);
            v.push(poly(&[
                (18.0, 3.0),
                (19.0, 6.0),
                (22.0, 5.5),
                (20.0, 8.0),
                (22.5, 10.0),
                (19.0, 10.5),
                (18.0, 13.5),
                (16.5, 10.5),
                (13.5, 11.0),
                (15.5, 8.0),
                (13.5, 5.5),
                (16.5, 6.0),
            ]));
            v
        }
        Stability => {
            let mut v = car_side(5.0, 19.0, 14.0);
            v.push(line(&[(3.0, 18.0), (6.0, 21.0), (9.0, 18.0), (12.0, 21.0)]));
            v.push(line(&[
                (12.0, 18.0),
                (15.0, 21.0),
                (18.0, 18.0),
                (21.0, 21.0),
            ]));
            v
        }
        AllWheel => vec![
            rrect(3.5, 3.5, 8.0, 9.0, 1.2),
            rrect(16.0, 3.5, 20.5, 9.0, 1.2),
            rrect(3.5, 15.0, 8.0, 20.5, 1.2),
            rrect(16.0, 15.0, 20.5, 20.5, 1.2),
            line(&[(8.0, 6.0), (16.0, 6.0)]),
            line(&[(8.0, 18.0), (16.0, 18.0)]),
            line(&[(12.0, 6.0), (12.0, 18.0)]),
        ],
        Steering => vec![
            circ(12.0, 12.0, 9.0),
            arc(12.0, 12.0, 3.0, 0.0, 180.0),
            line(&[(3.5, 11.0), (9.0, 12.0)]),
            line(&[(20.5, 11.0), (15.0, 12.0)]),
            line(&[(12.0, 15.0), (12.0, 21.0)]),
            line(&[(12.0, 5.5), (12.0, 9.0)]),
        ],
        Glow => vec![
            line(&[
                (4.0, 20.0),
                (4.0, 17.0),
                (7.0, 17.0),
                (7.0, 14.0),
                (4.0, 14.0),
                (4.0, 11.0),
                (7.0, 11.0),
                (7.0, 8.0),
                (4.0, 8.0),
            ]),
            arc(14.0, 12.0, 4.0, 90.0, 450.0),
            line(&[(14.0, 3.0), (14.0, 5.5)]),
            line(&[(14.0, 18.5), (14.0, 21.0)]),
            line(&[(20.5, 12.0), (22.0, 12.0)]),
        ],
        Frost => vec![
            line(&[(12.0, 3.0), (12.0, 21.0)]),
            line(&[(4.2, 7.5), (19.8, 16.5)]),
            line(&[(4.2, 16.5), (19.8, 7.5)]),
            line(&[(9.5, 4.5), (12.0, 7.0), (14.5, 4.5)]),
            line(&[(9.5, 19.5), (12.0, 17.0), (14.5, 19.5)]),
        ],
        Key => vec![
            circ(7.5, 12.0, 4.0),
            line(&[(11.5, 12.0), (21.5, 12.0)]),
            line(&[(18.0, 12.0), (18.0, 15.5)]),
            line(&[(21.0, 12.0), (21.0, 14.5)]),
        ],
        Service => vec![
            line(&[(5.0, 19.0), (13.0, 11.0)]),
            line(&[
                (13.0, 11.0),
                (12.0, 7.0),
                (15.5, 3.5),
                (17.0, 3.5),
                (15.0, 6.5),
                (17.5, 9.0),
                (20.5, 7.0),
                (20.5, 8.5),
                (17.0, 12.0),
                (13.0, 11.0),
            ]),
            circ(5.0, 19.5, 1.2),
        ],
        Gearbox => vec![
            circ(6.0, 6.0, 1.6),
            circ(12.0, 6.0, 1.6),
            circ(18.0, 6.0, 1.6),
            circ(6.0, 18.0, 1.6),
            circ(12.0, 18.0, 1.6),
            circ(18.0, 18.0, 1.6),
            line(&[(6.0, 7.6), (6.0, 16.4)]),
            line(&[(12.0, 7.6), (12.0, 16.4)]),
            line(&[(6.0, 12.0), (18.0, 12.0)]),
            line(&[(18.0, 7.6), (18.0, 16.4)]),
        ],
        Boost => {
            let mut v = vec![circ(12.0, 12.0, 2.0)];
            for k in 0..4 {
                let a = 90.0 * k as f32;
                v.push(arc(12.0, 12.0, 8.0, a + 10.0, a + 70.0));
                let e = rad(a + 70.0);
                v.push(line(&[
                    (12.0 + 8.0 * e.cos(), 12.0 + 8.0 * e.sin()),
                    (
                        12.0 + 2.0 * rad(a + 40.0).cos(),
                        12.0 + 2.0 * rad(a + 40.0).sin(),
                    ),
                ]));
            }
            v
        }
        Recuperation => vec![
            arc(12.0, 12.0, 8.5, 200.0, 340.0),
            arc(12.0, 12.0, 8.5, 20.0, 160.0),
            line(&[(16.0, 3.8), (20.5, 5.0), (19.0, 9.5)]),
            line(&[(8.0, 20.2), (3.5, 19.0), (5.0, 14.5)]),
            line(&[(13.0, 8.5), (10.0, 12.2), (13.5, 12.2), (11.0, 15.8)]),
        ],
        Sport => vec![
            line(&[(5.0, 21.5), (5.0, 3.5)]),
            poly(&[(5.0, 4.5), (19.5, 4.5), (19.5, 14.0), (5.0, 14.0)]),
            line(&[(10.0, 4.5), (10.0, 14.0)]),
            line(&[(15.0, 4.5), (15.0, 14.0)]),
            line(&[(5.0, 9.25), (19.5, 9.25)]),
        ],
        Eco => vec![
            poly(&[
                (4.0, 20.0),
                (4.0, 12.0),
                (12.0, 4.0),
                (20.0, 4.0),
                (20.0, 12.0),
                (12.0, 20.0),
            ]),
            line(&[(4.0, 20.0), (14.0, 10.0)]),
            line(&[(11.0, 12.5), (11.0, 8.5)]),
        ],
        Hybrid => vec![
            poly(&[
                (3.0, 18.0),
                (3.0, 12.0),
                (8.0, 7.0),
                (14.0, 7.0),
                (14.0, 12.0),
                (9.0, 17.0),
            ]),
            line(&[(3.0, 18.0), (10.0, 11.0)]),
            line(&[(18.0, 4.0), (18.0, 8.0)]),
            line(&[(21.0, 4.0), (21.0, 8.0)]),
            poly(&[
                (16.5, 8.0),
                (22.5, 8.0),
                (22.5, 11.0),
                (19.5, 14.0),
                (19.5, 14.0),
                (16.5, 11.0),
            ]),
            line(&[(19.5, 14.0), (19.5, 19.0)]),
        ],
        Sos => vec![
            circ(12.0, 12.0, 9.0),
            line(&[
                (8.5, 9.0),
                (7.0, 8.5),
                (6.5, 10.5),
                (9.0, 12.0),
                (8.5, 14.5),
                (6.8, 14.8),
            ]),
            circ(12.0, 12.0, 2.2),
            line(&[
                (17.0, 9.0),
                (15.5, 8.5),
                (15.0, 10.5),
                (17.5, 12.0),
                (17.0, 14.5),
                (15.3, 14.8),
            ]),
        ],
        ChildLock => vec![
            rrect(6.0, 10.0, 18.0, 21.0, 1.5),
            arc(12.0, 10.0, 4.5, 180.0, 360.0),
            circ(12.0, 14.5, 1.3),
            line(&[(12.0, 15.8), (12.0, 18.0)]),
        ],
        SeatHeat => vec![
            line(&[(4.0, 3.0), (6.5, 14.0), (14.0, 14.0), (14.5, 21.0)]),
            line(&[(6.5, 14.0), (5.5, 17.0)]),
            line(&[(16.0, 3.0), (14.5, 5.0), (16.0, 7.0), (14.5, 9.0)]),
            line(&[(19.5, 3.0), (18.0, 5.0), (19.5, 7.0), (18.0, 9.0)]),
            line(&[(14.0, 14.0), (20.0, 14.0)]),
        ],
        DefrostFront => vec![
            line(&[(3.0, 19.0), (6.0, 7.0), (18.0, 7.0), (21.0, 19.0)]),
            line(&[(3.0, 19.0), (21.0, 19.0)]),
            line(&[(9.0, 16.0), (8.0, 13.5), (9.0, 11.0)]),
            line(&[(12.0, 16.0), (11.0, 13.5), (12.0, 11.0)]),
            line(&[(15.0, 16.0), (14.0, 13.5), (15.0, 11.0)]),
        ],
        DefrostRear => vec![
            rrect(3.0, 6.0, 21.0, 19.0, 2.5),
            line(&[(6.5, 15.5), (8.0, 13.0), (6.5, 10.5)]),
            line(&[(10.5, 15.5), (12.0, 13.0), (10.5, 10.5)]),
            line(&[(14.5, 15.5), (16.0, 13.0), (14.5, 10.5)]),
        ],
        Fan => {
            let mut v = vec![circ(12.0, 12.0, 1.6)];
            for k in 0..3 {
                let a = 120.0 * k as f32 - 90.0;
                v.push(arc(12.0, 12.0, 4.5, a - 30.0, a + 40.0));
                let p = |d: f32, r: f32| (12.0 + r * rad(d).cos(), 12.0 + r * rad(d).sin());
                v.push(line(&[
                    p(a + 40.0, 4.5),
                    p(a + 20.0, 9.5),
                    p(a - 20.0, 9.5),
                    p(a - 30.0, 4.5),
                ]));
            }
            v
        }
        Recirculate => vec![
            arc(12.0, 13.0, 7.5, 200.0, 520.0),
            line(&[(8.0, 5.0), (4.0, 6.5), (5.5, 10.5)]),
            line(&[(15.0, 11.0), (12.0, 15.0), (9.0, 11.0)]),
        ],
        Compass => vec![
            circ(12.0, 12.0, 9.0),
            poly(&[(12.0, 5.0), (14.5, 12.0), (12.0, 10.5), (9.5, 12.0)]),
            line(&[(9.5, 12.0), (12.0, 19.0), (14.5, 12.0)]),
        ],
        Altitude => vec![
            line(&[
                (2.0, 20.0),
                (9.0, 7.0),
                (13.0, 14.0),
                (15.5, 10.5),
                (22.0, 20.0),
                (2.0, 20.0),
            ]),
            line(&[(7.0, 10.5), (9.0, 12.5), (11.0, 10.5)]),
        ],
        GForce => {
            let mut v = car_side(4.0, 16.0, 18.0);
            v.push(line(&[(18.0, 9.0), (22.0, 9.0)]));
            v.push(line(&[(20.0, 6.5), (22.5, 9.0), (20.0, 11.5)]));
            v
        }
        Trip => vec![
            circ(6.0, 6.0, 2.2),
            circ(18.0, 18.0, 2.2),
            line(&[
                (8.2, 6.0),
                (13.0, 6.0),
                (13.0, 12.0),
                (7.0, 12.0),
                (7.0, 18.0),
                (15.8, 18.0),
            ]),
        ],
        Range => vec![
            line(&[(8.0, 21.0), (11.0, 8.0)]),
            line(&[(16.0, 21.0), (13.0, 8.0)]),
            line(&[(12.0, 20.0), (12.0, 17.0)]),
            line(&[(12.0, 14.0), (12.0, 11.0)]),
            line(&[(2.5, 8.0), (21.5, 8.0)]),
            arc(12.0, 8.0, 5.0, 180.0, 360.0),
        ],
        Trailer => vec![
            rrect(8.0, 6.0, 21.0, 16.0, 1.5),
            circ(14.0, 18.5, 2.0),
            line(&[(8.0, 12.0), (3.0, 12.0)]),
            circ(2.8, 12.0, 0.9),
        ],
        Levelling => {
            let mut v = vec![lamp(5.0)];
            v.extend(rays(9.0, 12.0, 0.0, 30.0, 2, 2.5, 6.0));
            v.push(line(&[(19.0, 4.0), (19.0, 20.0)]));
            v.push(line(&[(16.5, 7.0), (19.0, 4.0), (21.5, 7.0)]));
            v.push(line(&[(16.5, 17.0), (19.0, 20.0), (21.5, 17.0)]));
            v
        }
        StopStart => vec![
            circ(12.0, 12.0, 9.0),
            line(&[(9.5, 8.0), (9.5, 16.0)]),
            line(&[(14.5, 8.0), (14.5, 16.0)]),
            arc(12.0, 12.0, 11.5, -30.0, 30.0),
        ],
        SpeedSign => vec![
            circ(12.0, 12.0, 9.5),
            circ(12.0, 12.0, 6.5),
            line(&[(10.0, 9.5), (11.5, 8.5), (11.5, 15.5)]),
            line(&[
                (13.0, 9.0),
                (15.0, 9.0),
                (15.0, 15.5),
                (13.0, 15.5),
                (13.0, 9.0),
            ]),
        ],
        RideHeight => {
            let mut v = car_side(2.5, 17.0, 17.0);
            v.push(line(&[(20.0, 4.0), (20.0, 20.0)]));
            v.push(line(&[(18.0, 6.5), (20.0, 4.0), (22.0, 6.5)]));
            v.push(line(&[(18.0, 17.5), (20.0, 20.0), (22.0, 17.5)]));
            v
        }
        OutsideTemp => vec![
            line(&[(6.0, 14.0), (6.0, 5.0), (10.0, 5.0), (10.0, 14.0)]),
            arc(8.0, 17.0, 4.0, -66.0, 246.0),
            line(&[(8.0, 8.0), (8.0, 17.0)]),
            line(&[(14.0, 6.0), (17.0, 6.0)]),
            line(&[(14.0, 10.0), (17.0, 10.0)]),
            line(&[(14.0, 14.0), (17.0, 14.0)]),
            circ(20.0, 5.5, 1.2),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symbol_draws_inside_its_box() {
        for &i in AutoIcon::ALL {
            let Glyph::Stroked(lines) = i.glyph() else {
                panic!("{} is not stroked", i.name())
            };
            assert!(!lines.is_empty(), "{} is empty", i.name());
            for (pts, _) in &lines {
                for &(x, y) in pts {
                    assert!(
                        x.is_finite() && (0.0..=24.0).contains(&x) && (0.0..=24.0).contains(&y),
                        "{} leaves the grid at ({x}, {y})",
                        i.name()
                    );
                }
            }
        }
    }

    #[test]
    fn names_are_unique_kebab_case() {
        let mut seen = std::collections::HashSet::new();
        for &i in AutoIcon::ALL {
            assert!(seen.insert(i.name()), "{} twice", i.name());
            assert!(i.name().chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
    }
}
