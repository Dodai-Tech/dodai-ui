//! Symbols for dashboards and instruments (vehicle, medical, military, industrial, aviation),
//! drawn for Dodai on the 24×24 grid.
//!
//! These are original drawings of the usual ideas (belt, beam, pulse, radar, valve, runway ...):
//! stroked, round-cornered, one weight, each built from a few arcs, circles and lines. They are
//! not traced from, or meant to match, any manufacturer's artwork.
//!
//! ```
//! use dodai_icons::symbol::Symbol;
//! let g = Symbol::OilCan.glyph();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(Symbol::ALL.len(), Symbol::ALL.iter().map(|i| i.name()).collect::<std::collections::HashSet<_>>().len());
//! ```

use crate::pack::{Glyph, Points};
use std::f32::consts::PI;

mod aviation;
mod industrial;
mod medical;
mod military;
mod vehicle;

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

macro_rules! symbols {
    ($($(#[$gm:meta])* $grp:ident = $gname:literal { $($(#[$m:meta])* $v:ident = $name:literal;)+ })+) => {
        /// A family of symbols, matching an instruments page.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Group {
            $($(#[$gm])* $grp,)+
        }
        impl Group {
            /// Every group.
            pub const ALL: &'static [Group] = &[$(Group::$grp),+];
            /// A short lower-case label.
            pub fn name(self) -> &'static str {
                match self { $(Group::$grp => $gname,)+ }
            }
        }
        /// A symbol from any group.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Symbol {
            $($($(#[$m])* $v,)+)+
        }
        impl Symbol {
            /// Every symbol, in catalogue order (group by group).
            pub const ALL: &'static [Symbol] = &[$($(Symbol::$v),+),+];
            /// A short stable kebab-case name, unique across groups.
            pub fn name(self) -> &'static str {
                match self { $($(Symbol::$v => $name,)+)+ }
            }
            /// The group this symbol belongs to.
            pub fn group(self) -> Group {
                match self { $($(Symbol::$v => Group::$grp,)+)+ }
            }
        }
    };
}

symbols! {
    /// Dashboard telltales and cluster symbols.
    Vehicle = "vehicle" {

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
    /// Clinical and monitor symbols.
    Medical = "medical" {
        /// Heartbeat trace.
        Pulse = "pulse";
        /// A heart with a trace through it.
        HeartRate = "heart-rate";
        /// Blood oxygen: a drop with a ring.
        Spo2 = "spo2";
        /// Two lungs on a windpipe.
        Lungs = "lungs";
        /// Clinical thermometer.
        Thermometer = "thermometer";
        /// Cuff, tube and bulb.
        BloodPressure = "blood-pressure";
        /// Syringe.
        Syringe = "syringe";
        /// Capsule.
        Pill = "pill";
        /// Cross on a plate.
        MedCross = "med-cross";
        /// Stethoscope.
        Stethoscope = "stethoscope";
        /// Defibrillator: heart and bolt.
        Defib = "defib";
        /// Infusion bag and line.
        IvDrip = "iv-drip";
        /// Lab flask.
        Flask = "flask";
        /// Adhesive bandage.
        Bandage = "bandage";
        /// Patient bed.
        Bed = "bed";
        /// Wheelchair.
        Wheelchair = "wheelchair";
        /// Breathing wave.
        Respiration = "respiration";
        /// Blood glucose: a drop with a plus.
        Glucose = "glucose";
        /// Three electrode leads.
        EcgLeads = "ecg-leads";
    }
    /// Tactical and sensor symbols.
    Military = "military" {
        /// Sweep over a ring.
        Radar = "radar";
        /// Sight with a ring.
        Crosshair = "crosshair";
        /// Shield with a cross.
        Shield = "shield";
        /// Stacked chevrons.
        Chevrons = "chevrons";
        /// Five-point star.
        Rank = "rank";
        /// Tracked vehicle.
        Tank = "tank";
        /// Ship with a mast.
        Warship = "warship";
        /// Helicopter.
        Rotorcraft = "rotorcraft";
        /// Four-rotor craft.
        Drone = "drone";
        /// Mast with waves.
        Antenna = "antenna";
        /// Binoculars.
        Binoculars = "binoculars";
        /// Flag on a pole.
        Waypoint = "waypoint";
        /// Friendly contact frame.
        Friendly = "friendly";
        /// Hostile contact frame.
        Hostile = "hostile";
        /// Neutral contact frame.
        Neutral = "neutral";
        /// Unknown contact: ring and question mark.
        UnknownContact = "unknown";
        /// Jammer: ring with a bolt.
        Jammer = "jammer";
        /// Missile.
        Missile = "missile";
    }
    /// Plant and machinery symbols.
    Industrial = "industrial" {
        /// Cog.
        Gear = "gear";
        /// Valve with a hand wheel.
        Valve = "valve";
        /// Pump.
        Pump = "pump";
        /// Pipework.
        Pipe = "pipe";
        /// Process vessel.
        Vessel = "vessel";
        /// Motor.
        Motor = "motor";
        /// Conveyor belt.
        Conveyor = "conveyor";
        /// Furnace with flame.
        Furnace = "furnace";
        /// Pressure gauge.
        PressureDial = "pressure-dial";
        /// Flow in a channel.
        Flow = "flow";
        /// Bolt.
        Power = "power";
        /// Two coils.
        Transformer = "transformer";
        /// Crane.
        Crane = "crane";
        /// Robot arm.
        RobotArm = "robot-arm";
        /// Circuit breaker.
        Breaker = "breaker";
        /// Sensing waves.
        Sensor = "sensor";
        /// Hard hat.
        HardHat = "hard-hat";
        /// Beacon siren.
        Siren = "siren";
        /// Funnel filter.
        Filter = "filter";
    }
    /// Flight and airfield symbols.
    Aviation = "aviation" {
        /// Airliner from above.
        Airliner = "airliner";
        /// Runway with centre line.
        Runway = "runway";
        /// Attitude indicator.
        Horizon = "horizon";
        /// Altimeter dial.
        Altimeter = "altimeter";
        /// Wing flaps.
        Flaps = "flaps";
        /// Landing gear.
        LandingGear = "landing-gear";
        /// Control tower.
        Tower = "tower";
        /// Windsock.
        Windsock = "windsock";
        /// Heading arrow.
        Heading = "heading";
        /// Turbine face.
        Turbine = "turbine";
        /// Propeller.
        Propeller = "propeller";
        /// Glide slope.
        GlideSlope = "glide-slope";
        /// Transponder box.
        Transponder = "transponder";
        /// Take-off.
        Takeoff = "takeoff";
        /// Landing.
        Landing = "landing";
        /// Turbulence.
        Turbulence = "turbulence";
        /// Autopilot: jet in a ring.
        Autopilot = "autopilot";
        /// Parachute.
        Parachute = "parachute";
    }
}

impl Symbol {
    /// The drawing, as a stroked [`Glyph`] on the 24×24 grid.
    pub fn glyph(self) -> Glyph {
        Glyph::Stroked(match self.group() {
            Group::Vehicle => vehicle::shapes(self),
            Group::Medical => medical::shapes(self),
            Group::Military => military::shapes(self),
            Group::Industrial => industrial::shapes(self),
            Group::Aviation => aviation::shapes(self),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_symbol_draws_inside_its_box() {
        for &i in Symbol::ALL {
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
        for &i in Symbol::ALL {
            assert!(seen.insert(i.name()), "{} twice", i.name());
            assert!(i
                .name()
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
        }
    }
}
