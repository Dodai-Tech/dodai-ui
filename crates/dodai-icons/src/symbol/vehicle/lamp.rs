//! The standard colour of each telltale: what a lit symbol means by its colour on a dash.

use super::Symbol;

/// The colour a telltale lights in, by its meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Lamp {
    /// Danger: stop safely or act now (brakes, oil, belt, airbag).
    Red,
    /// Caution: a fault or a limit to see to soon (engine, ABS, tyres, low fuel).
    Amber,
    /// A system is on and working (dipped beam, indicators, cruise control).
    Green,
    /// Main beam is on.
    Blue,
}

impl Lamp {
    /// A short lower-case name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Red => "red",
            Self::Amber => "amber",
            Self::Green => "green",
            Self::Blue => "blue",
        }
    }
}

/// The telltales most dashes carry, in the order a row shows them: safety first, then the
/// engine, then the lights.
pub const COMMON_TELLTALES: [Symbol; 16] = [
    Symbol::SeatBelt,
    Symbol::Airbag,
    Symbol::ParkingBrake,
    Symbol::BrakeFault,
    Symbol::Abs,
    Symbol::Stability,
    Symbol::TyrePressure,
    Symbol::Battery,
    Symbol::OilCan,
    Symbol::Coolant,
    Symbol::Engine,
    Symbol::Fuel,
    Symbol::TurnLeft,
    Symbol::LowBeam,
    Symbol::HighBeam,
    Symbol::TurnRight,
];

impl Symbol {
    /// The colour this symbol lights in as a telltale, or `None` for a symbol that is not a
    /// warning or indicator lamp (a car outline, the compass, the trip). Pure.
    pub fn lamp(self) -> Option<Lamp> {
        use Symbol::*;
        Some(match self {
            SeatBelt | ParkingBrake | BrakeFault | Airbag | Battery | OilCan | Coolant
            | Steering | DoorOpen | TrunkOpen | HoodOpen | Collision | Hazard | Sos => Lamp::Red,
            Abs | Engine | Fuel | TyrePressure | Washer | Stability | Glow | Frost | Service
            | LaneDeparture | FogRear | AllWheel | Gearbox | TractionBattery | ChildLock
            | Levelling => Lamp::Amber,
            LowBeam | FogFront | SideLights | TurnLeft | TurnRight | Cruise | Limiter
            | LaneKeep | Plug | Recuperation | Eco | Hybrid | StopStart | Trailer => Lamp::Green,
            HighBeam => Lamp::Blue,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_common_telltale_has_a_colour_and_appears_once() {
        for (i, s) in COMMON_TELLTALES.iter().enumerate() {
            assert!(s.lamp().is_some(), "{} has no lamp colour", s.name());
            assert!(!COMMON_TELLTALES[..i].contains(s), "{} twice", s.name());
        }
    }

    #[test]
    fn colours_follow_the_meaning() {
        assert_eq!(Symbol::OilCan.lamp(), Some(Lamp::Red));
        assert_eq!(Symbol::Engine.lamp(), Some(Lamp::Amber));
        assert_eq!(Symbol::TurnLeft.lamp(), Some(Lamp::Green));
        assert_eq!(Symbol::HighBeam.lamp(), Some(Lamp::Blue));
        assert_eq!(Symbol::Car.lamp(), None);
        assert_eq!(Lamp::Amber.name(), "amber");
    }
}
