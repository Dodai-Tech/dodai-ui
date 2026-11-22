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
//! use dodai_ui_icons::symbol::Symbol;
//! let g = Symbol::OilCan.glyph();
//! assert!(g.svg(24, "currentColor").starts_with("<svg"));
//! assert_eq!(Symbol::ALL.len(), Symbol::ALL.iter().map(|i| i.name()).collect::<std::collections::HashSet<_>>().len());
//! # }
//! ```

#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
use crate::pack::Points;
use crate::pack::{Glyph, StrokeStyle};
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
#[cfg(feature = "symbols-vehicle")]
pub use vehicle::{Lamp, COMMON_TELLTALES};

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

/// An open arc of radius `r` from `a0` to `a1` degrees, faceted every ~5°.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32) -> Shape {
    let n = (((a1 - a0).abs() / 5.0).ceil() as usize).max(2);
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
#[cfg(any(
    feature = "symbols-military",
    feature = "symbols-vehicle",
    feature = "symbols-medical"
))]
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

/// A smooth closed (or open) curve through `p` (Catmull-Rom), `per` points per span.
#[cfg(any(
    feature = "symbols-vehicle",
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial",
    feature = "symbols-aviation"
))]
fn curve(p: &[(f32, f32)], closed: bool, per: usize) -> Shape {
    let n = p.len();
    let at = |i: isize| -> (f32, f32) {
        if closed {
            p[i.rem_euclid(n as isize) as usize]
        } else {
            p[i.clamp(0, n as isize - 1) as usize]
        }
    };
    let spans = if closed { n } else { n - 1 };
    let mut pts = Vec::new();
    for s in 0..spans as isize {
        let (p0, p1, p2, p3) = (at(s - 1), at(s), at(s + 1), at(s + 2));
        for k in 0..per {
            let t = k as f32 / per as f32;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f32, b: f32, c: f32, d: f32| {
                0.5 * (2.0 * b
                    + (c - a) * t
                    + (2.0 * a - 5.0 * b + 4.0 * c - d) * t2
                    + (3.0 * b - a - 3.0 * c + d) * t3)
            };
            pts.push((f(p0.0, p1.0, p2.0, p3.0), f(p0.1, p1.1, p2.1, p3.1)));
        }
    }
    if !closed {
        pts.push(p[n - 1]);
    }
    (pts, closed)
}

/// Every shape turned `deg` degrees clockwise about `(cx, cy)`.
#[cfg(any(
    feature = "symbols-medical",
    feature = "symbols-military",
    feature = "symbols-industrial"
))]
fn spin(v: Vec<Shape>, cx: f32, cy: f32, deg: f32) -> Vec<Shape> {
    let (s, c) = rad(deg).sin_cos();
    v.into_iter()
        .map(|(pts, closed)| {
            let pts = pts
                .into_iter()
                .map(|(x, y)| {
                    let (dx, dy) = (x - cx, y - cy);
                    (cx + dx * c - dy * s, cy + dx * s + dy * c)
                })
                .collect();
            (pts, closed)
        })
        .collect()
}

/// A drop of blood or water: tip at `(cx, cy - r*1.7)`, round belly of radius `r`.
#[cfg(any(feature = "symbols-medical", feature = "symbols-industrial"))]
fn bead(cx: f32, cy: f32, r: f32) -> Shape {
    let d = r * 1.7;
    let t = (r / d).acos().to_degrees();
    let (a0, a1) = (270.0 + t, 270.0 - t + 360.0);
    let mut pts = vec![(cx, cy - d)];
    pts.extend(arc(cx, cy, r, a0, a1).0);
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
    Battery = "vehicle-battery";
    /// Oil pressure: a can with a drop.
    OilCan = "oil-can";
    /// Coolant temperature: thermometer over two waves.
    Coolant = "coolant";
    /// Engine block.
    Engine = "vehicle-engine";
    /// Fuel pump.
    Fuel = "fuel";
    /// Charging plug.
    Plug = "vehicle-plug";
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
    Key = "vehicle-key";
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
    Eco = "vehicle-eco";
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
    Fan = "vehicle-fan";
    /// Air recirculation.
    Recirculate = "recirculate";
    /// Compass rose.
    Compass = "vehicle-compass";
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
    Car = "vehicle-car";
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
    Van = "vehicle-van";
    /// Pickup with an open bed.
    Pickup = "pickup";
    /// Box truck.
    Truck = "vehicle-truck";
    /// Folding roof opening.
    RoofOpen = "roof-open";
    /// Folding roof closing.
    RoofClosed = "roof-closed";
    /// Surround view: car in a turning arrow.
    View360 = "view-360";
    /// Closed side door.
    Door = "vehicle-door";
    /// Urban driving.
    City = "vehicle-city";
    /// Winding road through hills.
    CountryRoad = "country-road";
    /// Multi-lane motorway.
    Highway = "highway";
    /// Garage.
    Garage = "vehicle-garage";
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
    /// Exhaust fluid for a diesel's emission cleaning: a drop over the exhaust pipe.
    ExhaustFluid = "exhaust-fluid";
    /// Seat.
    Seat = "vehicle-seat";
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
    Window = "vehicle-window";
    /// Mirror fold.
    MirrorFold = "mirror-fold";
    /// Blind-spot monitor.
    BlindSpot = "blind-spot";
    /// Parking sensors.
    ParkSensor = "park-sensor";
    /// Camera.
    Camera = "vehicle-camera";
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
        Pulse = "medical-pulse";
        /// A heart with a trace through it.
        HeartRate = "heart-rate";
        /// Blood oxygen: a drop with a ring.
        Spo2 = "spo2";
        /// Two lungs on a windpipe.
        Lungs = "lungs";
        /// Clinical thermometer.
        Thermometer = "medical-thermometer";
        /// Cuff, tube and bulb.
        BloodPressure = "blood-pressure";
        /// Syringe.
        Syringe = "medical-syringe";
        /// Capsule.
        Pill = "medical-pill";
        /// Cross on a plate.
        MedCross = "med-cross";
        /// Stethoscope.
        Stethoscope = "medical-stethoscope";
        /// Defibrillator: heart and bolt.
        Defib = "defib";
        /// Infusion bag and line.
        IvDrip = "iv-drip";
        /// Lab flask.
        Flask = "medical-flask";
        /// Adhesive bandage.
        Bandage = "bandage";
        /// Patient bed.
        Bed = "medical-bed";
        /// Wheelchair.
        Wheelchair = "medical-wheelchair";
        /// Breathing wave.
        Respiration = "respiration";
        /// Blood glucose: a drop with a plus.
        Glucose = "glucose";
        /// Three electrode leads.
        EcgLeads = "ecg-leads";
        /// Microscope.
        Microscope = "medical-microscope";
        /// Ambulance.
        Ambulance = "medical-ambulance";
        /// Face mask.
        Mask = "mask";
        /// Scalpel.
        Scalpel = "scalpel";
        /// Molar tooth.
        Tooth = "medical-tooth";
        /// Eye.
        Eye = "medical-eye";
        /// Bone.
        Bone = "medical-bone";
        /// Stretcher on wheels.
        Stretcher = "stretcher";
        /// Hospital building with a cross.
        Hospital = "medical-hospital";
        /// Bedside patient monitor.
        Monitor = "patient-monitor";
        /// Dropper.
        Dropper = "dropper";
        /// Test tube with a sample.
        TestTube = "test-tube";
        /// Inhaler with a mist.
        Inhaler = "inhaler";
        /// Underarm crutch.
        Crutch = "crutch";
        /// Double helix.
        Dna = "medical-dna";
        /// Pill bottle.
        PillBottle = "pill-bottle";
        /// Blood bag with a line.
        BloodBag = "blood-bag";
        /// Oxygen cylinder with a valve.
        OxygenTank = "oxygen-tank";
    }
    #[cfg(feature = "symbols-military")]
    /// Tactical and sensor symbols.
    Military = "military" {
        /// Sweep over a ring.
        Radar = "military-radar";
        /// Sight with a ring.
        Crosshair = "military-crosshair";
        /// Shield with a cross.
        Shield = "military-shield";
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
        Drone = "military-drone";
        /// Mast with waves.
        Antenna = "antenna";
        /// Binoculars.
        Binoculars = "military-binoculars";
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
        /// Submarine.
        Submarine = "submarine";
        /// Fighter jet.
        Fighter = "fighter";
        /// Field radio.
        Radio = "military-radio";
        /// Combat helmet.
        Helmet = "helmet";
        /// Medal on a ribbon.
        Medal = "military-medal";
        /// Dome bunker with a firing slit.
        Bunker = "bunker";
        /// Identity tag on a chain.
        Dogtag = "dog-tag";
    }
    #[cfg(feature = "symbols-industrial")]
    /// Plant and machinery symbols.
    Industrial = "industrial" {
        /// Cog.
        Gear = "industrial-gear";
        /// Valve with a hand wheel.
        Valve = "valve";
        /// Pump.
        Pump = "pump";
        /// Pipework.
        Pipe = "industrial-pipe";
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
        Power = "industrial-power";
        /// Two coils.
        Transformer = "transformer";
        /// Crane.
        Crane = "industrial-crane";
        /// Robot arm.
        RobotArm = "robot-arm";
        /// Circuit breaker.
        Breaker = "breaker";
        /// Sensing waves.
        Sensor = "sensor";
        /// Hard hat.
        HardHat = "hard-hat";
        /// Beacon siren.
        Siren = "industrial-siren";
        /// Funnel filter.
        Filter = "industrial-filter";
        /// Smokestack.
        Smokestack = "smokestack";
        /// Solar panel.
        SolarPanel = "solar-panel";
        /// Wind turbine.
        WindTurbine = "wind-turbine";
        /// Forklift with a pallet.
        Forklift = "forklift";
        /// Open-end spanner.
        Wrench = "industrial-wrench";
        /// Hammer.
        Hammer = "industrial-hammer";
        /// Hex nut.
        Nut = "hex-nut";
        /// Ball bearing.
        Bearing = "bearing";
        /// Piston with a connecting rod.
        Piston = "piston";
        /// Power drill.
        Drill = "power-drill";
        /// Oil derrick.
        Derrick = "derrick";
        /// Material hopper.
        Hopper = "hopper";
        /// Padlock for lock-out tag-out.
        Lockout = "lockout";
        /// Cylindrical storage tank.
        StorageTank = "storage-tank";
        /// Crate on a pallet.
        Pallet = "pallet";
        /// Coil spring.
        Spring = "spring";
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
        Parachute = "aviation-parachute";
        /// Route with waypoints.
        FlightPlan = "flight-plan";
        /// Crosswind streaks.
        Crosswind = "crosswind";
        /// Airport beacon.
        Beacon = "beacon";
        /// Helicopter from the side.
        Helicopter = "helicopter";
        /// Hangar with a sliding door.
        Hangar = "hangar";
        /// Glider from above.
        Glider = "glider";
        /// Hot-air balloon.
        Balloon = "hot-air-balloon";
        /// Navigation beacon station.
        Vor = "vor";
        /// Suitcase.
        Suitcase = "aviation-suitcase";
        /// Boarding pass.
        Ticket = "boarding-pass";
        /// Rocket.
        Rocket = "aviation-rocket";
        /// Satellite with solar panels.
        Satellite = "satellite";
        /// Pitot tube with airflow.
        Pitot = "pitot-tube";
        /// Wing section with lift.
        Airfoil = "airfoil";
        /// Air-cargo container.
        Cargo = "cargo-container";
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
        self.glyph_styled(StrokeStyle::ROUND)
    }
    /// The drawing stroked in a design's [`StrokeStyle`].
    #[cfg(any(
        feature = "symbols-vehicle",
        feature = "symbols-medical",
        feature = "symbols-military",
        feature = "symbols-industrial",
        feature = "symbols-aviation"
    ))]
    pub fn glyph_styled(self, style: StrokeStyle) -> Glyph {
        Glyph::Lines(
            style.shape(match self.group() {
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
            }),
            style,
        )
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
    /// The drawing stroked in a design's [`StrokeStyle`] (no symbol group is compiled in).
    #[cfg(not(any(
        feature = "symbols-vehicle",
        feature = "symbols-medical",
        feature = "symbols-military",
        feature = "symbols-industrial",
        feature = "symbols-aviation"
    )))]
    pub fn glyph_styled(self, _: StrokeStyle) -> Glyph {
        match self {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Ink bounds as a share of the box: below, a symbol vanishes; above, it clogs.
    const MIN_INK: f32 = 0.07;
    const MAX_INK: f32 = 0.42;

    #[test]
    fn every_symbol_draws_inside_its_box() {
        for &i in Symbol::ALL {
            let Glyph::Lines(lines, _) = i.glyph() else {
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

    /// Fraction of the 24x24 box covered by ink when stroked at `style.weight`.
    fn ink(lines: &[Shape], weight: f32) -> f32 {
        let segs: Vec<((f32, f32), (f32, f32))> = lines
            .iter()
            .flat_map(|(p, closed)| {
                let n = p.len();
                let last = if *closed { n } else { n.saturating_sub(1) };
                (0..last).map(move |i| (p[i], p[(i + 1) % n]))
            })
            .collect();
        let half = weight / 2.0;
        let steps = 72;
        let mut hit = 0;
        for iy in 0..steps {
            for ix in 0..steps {
                let q = (
                    (ix as f32 + 0.5) * 24.0 / steps as f32,
                    (iy as f32 + 0.5) * 24.0 / steps as f32,
                );
                if segs.iter().any(|&(a, b)| dist(q, a, b) <= half) {
                    hit += 1;
                }
            }
        }
        hit as f32 / (steps * steps) as f32
    }

    fn dist(q: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len2 = dx * dx + dy * dy;
        let t = if len2 == 0.0 {
            0.0
        } else {
            (((q.0 - a.0) * dx + (q.1 - a.1) * dy) / len2).clamp(0.0, 1.0)
        };
        (q.0 - (a.0 + t * dx)).hypot(q.1 - (a.1 + t * dy))
    }

    #[test]
    fn symbols_neither_clog_nor_vanish_in_any_style() {
        for style in [StrokeStyle::ROUND, StrokeStyle::SQUARE, StrokeStyle::BOLD] {
            for &i in Symbol::ALL {
                let Glyph::Lines(lines, _) = i.glyph_styled(style) else {
                    continue;
                };
                let cover = ink(&lines, style.weight);
                assert!(
                    (MIN_INK..=MAX_INK).contains(&cover),
                    "{} covers {cover:.2} of its box at weight {}",
                    i.name(),
                    style.weight
                );
            }
        }
    }

    #[test]
    fn symbols_are_sized_and_centred_on_the_grid() {
        for &i in Symbol::ALL {
            let Glyph::Lines(lines, _) = i.glyph() else {
                continue;
            };
            let pts = lines.iter().flat_map(|(p, _)| p.iter().copied());
            let (mut lo, mut hi) = ((f32::MAX, f32::MAX), (f32::MIN, f32::MIN));
            for (x, y) in pts {
                lo = (lo.0.min(x), lo.1.min(y));
                hi = (hi.0.max(x), hi.1.max(y));
            }
            let extent = (hi.0 - lo.0).max(hi.1 - lo.1);
            let centre = ((lo.0 + hi.0) / 2.0, (lo.1 + hi.1) / 2.0);
            assert!(extent >= 13.5, "{} is only {extent:.1} wide", i.name());
            assert!(
                (centre.0 - 12.0).abs() <= 3.0 && (centre.1 - 12.0).abs() <= 3.0,
                "{} sits off-centre at {centre:?}",
                i.name()
            );
        }
    }

    #[test]
    fn a_design_strokes_the_same_drawing_in_its_own_style() {
        let s = Symbol::ALL[0];
        let soft = s.glyph_styled(StrokeStyle::ROUND).svg(24, "red");
        let hard = s.glyph_styled(StrokeStyle::SQUARE).svg(24, "red");
        assert!(soft.contains("linecap=\"round\""));
        assert!(hard.contains("linecap=\"square\"") && hard.contains("linejoin=\"miter\""));
        let bold = s.glyph_styled(StrokeStyle::BOLD);
        let fine = s.glyph_styled(StrokeStyle::ROUND);
        assert!(
            bold.coverage(48).iter().map(|&b| b as u32).sum::<u32>()
                > fine.coverage(48).iter().map(|&b| b as u32).sum::<u32>()
        );
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

    #[test]
    fn no_symbol_shares_a_name_with_a_named_icon() {
        let named: std::collections::HashSet<_> =
            crate::IconId::ALL.iter().map(|i| i.name()).collect();
        let clash: Vec<_> = Symbol::ALL
            .iter()
            .filter(|s| named.contains(s.name()))
            .map(|s| s.name())
            .collect();
        assert!(clash.is_empty(), "{clash:?}");
    }
}
