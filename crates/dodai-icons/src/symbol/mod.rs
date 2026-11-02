//! Symbols for dashboards and instruments (vehicle, medical, military, industrial, aviation),
//! drawn for Dodai on the 24×24 grid.
//!
//! These are original drawings of the usual ideas (belt, beam, pulse, radar, valve, runway ...):
//! stroked, round-cornered, one weight, each built from a few arcs, circles and lines. They are
//! not traced from, or meant to match, any manufacturer's artwork.
//!
//! Each group is a feature (`symbols-vehicle`, `symbols-medical`, `symbols-military`,
//! `symbols-industrial`, `symbols-aviation`; `symbols` is all five), so a build carries only the
//! groups it uses. [`Symbol::ALL`] and [`Group::ALL`] list what is compiled in.
//!
//! ```
//! # #[cfg(feature = "symbols-vehicle")]
//! # {
//! use dodai_icons::symbol::Symbol;
//! let g = Symbol::OilCan.glyph();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(Symbol::ALL.len(), Symbol::ALL.iter().map(|i| i.name()).collect::<std::collections::HashSet<_>>().len());
//! # }
//! ```

use crate::pack::Glyph;
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
use crate::pack::Points;
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
use std::f32::consts::PI;

#[cfg(feature = "symbols-aviation")]
mod aviation;
#[cfg(feature = "symbols-industrial")]
mod industrial;
#[cfg(feature = "symbols-medical")]
mod medical;
#[cfg(feature = "symbols-military")]
mod military;
#[cfg(feature = "symbols-vehicle")]
mod vehicle;

#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
type Shape = (Points, bool);

/// Degrees to radians with 0° pointing right and angles growing clockwise on screen.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn rad(d: f32) -> f32 {
    d * PI / 180.0
}

/// An open arc of radius `r` from `a0` to `a1` degrees, faceted every ~12°.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
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
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn circ(cx: f32, cy: f32, r: f32) -> Shape {
    let (mut pts, _) = arc(cx, cy, r, 0.0, 360.0);
    pts.pop();
    (pts, true)
}

/// A polyline.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn line(p: &[(f32, f32)]) -> Shape {
    (p.to_vec(), false)
}

/// A closed polygon.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn poly(p: &[(f32, f32)]) -> Shape {
    (p.to_vec(), true)
}

/// A rectangle with rounded corners of radius `r`.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
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
#[cfg(any(feature = "symbols-military", feature = "symbols-vehicle"))]
fn dot(x: f32, y: f32) -> Shape {
    poly(&[(x, y - 0.4), (x + 0.4, y), (x, y + 0.4), (x - 0.4, y)])
}

/// Light rays fanning out from `(x, y)` along `dir` degrees: the beam family's signature.
#[cfg(feature = "symbols-vehicle")]
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
#[cfg(feature = "symbols-vehicle")]
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
    ($(#[cfg($gc:meta)] $(#[$gm:meta])* $grp:ident = $gname:literal { $($(#[$m:meta])* $v:ident = $name:literal;)+ })+) => {
        /// A family of symbols, matching an instruments page.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Group {
            $($(#[$gm])* #[cfg($gc)] $grp,)+
        }
        impl Group {
            /// Every group.
            pub const ALL: &'static [Group] = &[$(#[cfg($gc)] Group::$grp),+];
            /// A short lower-case label.
            pub fn name(self) -> &'static str {
                match self { $(#[cfg($gc)] Group::$grp => $gname,)+ }
            }
        }
        /// A symbol from any group.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum Symbol {
            $($($(#[$m])* #[cfg($gc)] $v,)+)+
        }
        impl Symbol {
            /// Every symbol, in catalogue order (group by group).
            pub const ALL: &'static [Symbol] = &[$($(#[cfg($gc)] Symbol::$v),+),+];
            /// A short stable kebab-case name, unique across groups.
            pub fn name(self) -> &'static str {
                match self { $($(#[cfg($gc)] Symbol::$v => $name,)+)+ }
            }
            /// The group this symbol belongs to.
            pub fn group(self) -> Group {
                match self { $($(#[cfg($gc)] Symbol::$v => Group::$grp,)+)+ }
            }
        }
    };
}

symbols! {
    #[cfg(feature = "symbols-vehicle")]
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
    /// Hatchback, side view.
    Car = "car";
    /// Fastback coupé.
    Coupe = "coupe";
    /// Convertible with the hood folded.
    Convertible = "convertible";
    /// Three-box saloon.
    Saloon = "saloon";
    /// Estate with a long roof.
    Estate = "estate";
    /// Sport utility vehicle.
    Suv = "suv";
    /// Van.
    Van = "van";
    /// Pickup with an open bed.
    Pickup = "pickup";
    /// Box truck.
    Truck = "truck";
    /// Folding roof opening.
    RoofOpen = "roof-open";
    /// Folding roof closing.
    RoofClosed = "roof-closed";
    /// Surround view: car in a turning arrow.
    View360 = "view-360";
    /// Closed side door.
    Door = "door";
    /// Urban driving.
    City = "city";
    /// Winding road through hills.
    CountryRoad = "country-road";
    /// Multi-lane motorway.
    Highway = "highway";
    /// Garage.
    Garage = "garage";
    /// Hill descent control.
    HillDescent = "hill-descent";
    /// Off-road mode.
    OffRoad = "off-road";
    /// Race circuit.
    Circuit = "circuit";
    /// Workshop lift.
    Lift = "lift";
    /// Battery empty.
    BatteryEmpty = "battery-empty";
    /// Battery a quarter full.
    BatteryQuarter = "battery-quarter";
    /// Battery half full.
    BatteryHalf = "battery-half";
    /// Battery three quarters full.
    BatteryThreeQuarters = "battery-three-quarters";
    /// Battery full.
    BatteryFull = "battery-full";
    /// Battery empty, running on fuel.
    BatteryEmptyFuel = "battery-empty-fuel";
    /// Charging in progress.
    Charging = "charging";
    /// State of charge.
    ChargeLevel = "charge-level";
    /// Charging station.
    ChargeStation = "charge-station";
    /// Charging network.
    ChargeNetwork = "charge-network";
    /// Charging socket.
    ChargeSocket = "charge-socket";
    /// Car charging.
    CarCharging = "car-charging";
    /// Carbon dioxide.
    Co2 = "co2";
    /// Emission class bars.
    Co2Class = "co2-class";
    /// Exhaust emission.
    Exhaust = "exhaust";
    /// Particulate filter.
    Particulate = "particulate";
    /// Seat.
    Seat = "seat";
    /// Seat ventilation.
    SeatCool = "seat-cool";
    /// Steering wheel.
    SteeringWheel = "steering-wheel";
    /// Heated steering wheel.
    SteeringHeat = "steering-heat";
    /// Horn.
    Horn = "horn";
    /// Leather upholstery.
    Leather = "leather";
    /// Parking light.
    ParkingLight = "parking-light";
    /// Rev counter.
    Tachometer = "tachometer";
    /// Tyre.
    Tyre = "tyre";
    /// Weight.
    Weight = "weight";
    /// Two spanners.
    Wrenches = "wrenches";
    /// North arrow.
    NorthArrow = "north-arrow";
    /// Stopwatch.
    Stopwatch = "stopwatch";
    /// Engine displacement: piston in a cylinder.
    Displacement = "displacement";
    /// Spare parts box.
    Parts = "parts";
    /// Service technician.
    Technician = "technician";
    /// Climate: sun and snowflake.
    Climate = "climate";
    /// Air distribution.
    AirFlow = "air-flow";
    /// Cabin ventilation.
    CabinVent = "cabin-vent";
    /// Timed preconditioning.
    Preheat = "preheat";
    /// Windscreen wiper.
    Wiper = "wiper";
    /// Sunroof.
    Sunroof = "sunroof";
    /// Power window.
    Window = "window";
    /// Mirror fold.
    MirrorFold = "mirror-fold";
    /// Blind-spot monitor.
    BlindSpot = "blind-spot";
    /// Parking sensors.
    ParkSensor = "park-sensor";
    /// Camera.
    Camera = "camera";
    /// Adaptive cruise following distance.
    FollowDistance = "follow-distance";
    /// Brake pad wear.
    BrakeWear = "brake-wear";
    /// Rest break: a cup.
    RestBreak = "rest-break";
    /// Head-up display.
    Hud = "hud";
    /// Wireless phone charging.
    PhoneCharge = "phone-charge";
    /// Phone as key.
    DigitalKey = "digital-key";
    /// Automatic main beam.
    AutoBeam = "auto-beam";
    /// Odometer.
    Odometer = "odometer";
    }
    #[cfg(feature = "symbols-medical")]
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
    #[cfg(feature = "symbols-military")]
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
    #[cfg(feature = "symbols-industrial")]
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
    #[cfg(feature = "symbols-aviation")]
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
    #[cfg(any(
        feature = "symbols-vehicle",
        feature = "symbols-medical",
        feature = "symbols-military",
        feature = "symbols-industrial",
        feature = "symbols-aviation"
    ))]
    pub fn glyph(self) -> Glyph {
        Glyph::Stroked(match self.group() {
            #[cfg(feature = "symbols-vehicle")]
            Group::Vehicle => vehicle::shapes(self),
            #[cfg(feature = "symbols-medical")]
            Group::Medical => medical::shapes(self),
            #[cfg(feature = "symbols-military")]
            Group::Military => military::shapes(self),
            #[cfg(feature = "symbols-industrial")]
            Group::Industrial => industrial::shapes(self),
            #[cfg(feature = "symbols-aviation")]
            Group::Aviation => aviation::shapes(self),
        })
    }
    /// The drawing (no symbol group is compiled in, so there is no symbol to draw).
    #[cfg(not(any(
        feature = "symbols-vehicle",
        feature = "symbols-medical",
        feature = "symbols-military",
        feature = "symbols-industrial",
        feature = "symbols-aviation"
    )))]
    pub fn glyph(self) -> Glyph {
        match self {}
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
