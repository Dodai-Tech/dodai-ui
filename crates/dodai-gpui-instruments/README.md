# dodai-gpui-instruments

Animated instruments for gpui in the Dodai designs: dials and gauges, segment displays, annunciators and scopes, plus industry sets behind features. The colours come from the active theme, so every instrument follows the design and light or dark mode.

## Setup

Pick the industry sets you need and at least one design:

```toml
[dependencies]
dodai-gpui-instruments = { version = "0.1", default-features = false, features = ["automotive", "dodai"] }
```

Install the themes once at start-up, then use the instruments like any gpui element:

```rust,no_run
use dodai_gpui_instruments::{DashTriptych, Look, MenuRing, RingTach, Scale, SidePod};

fn setup(cx: &mut gpui::App) {
    // Every theme of the enabled designs; `dodai_gpui::cycle_next` switches between them.
    dodai_gpui::install_switcher(cx, dodai_core::switcher());
}

fn dash() -> DashTriptych {
    DashTriptych::new(
        RingTach::new(Scale::new(0.0, 8.0), 2.4, 96.0),
        SidePod::new(Scale::new(0.0, 270.0), 96.0).unit("km/h"),
        MenuRing::new(["Trip", "Media", "Tyres"]),
    )
    .look(Look::Chronograph)
}
```

A reading past a limit colours its value (pointer, sweep, bar or figure) in the limit's colour. Optionally, the looks drawn on a screen (heads-up, segment LCD, halo, glass cockpit, line) colour the whole display with it: `GaugeFace`, `SidePod`, `RingTach`, `BoostPod` and `DualGauge` take `.at_limit(AtLimit::Display)`; the default is `AtLimit::Value`.

Without an installed theme the instruments use the first theme of the enabled designs, or a plain neutral theme when no design is enabled. An app can also install its own `dodai_core::Theme` with `dodai_gpui::install`.

## Features

The default turns on every industry set and no design.

| Feature | What it adds |
|---|---|
| `automotive` | dashes, pods, gauge faces in nine looks (`Look`), tachometer, speedometers, tyre pressure, gear selector (P R N D as a row, a stack or one large figure, manual gear, shift prompt), every drivetrain (`RingTach::drivetrain`: the revs and the fuel, the revs with fuel and charge for a hybrid, or the motor's power from zero and the charge for an electric car), a hybrid's power flow (`PowerFlow`), EV charging with a limit and the time to it, odometer, compass, EV energy, shift lights; the vehicle symbols |
| `aviation` | attitude indicator, heading and vertical tapes, sonar waterfall, countermeasures; the aviation symbols |
| `defense` | tactical plot, track list, threat rosette, stores panel, mission timer, radar scope; the military symbols |
| `industrial` | the process-plant and machine symbols |
| `medical` | vitals tiles, the ECG trace (`Oscilloscope::ecg`); the medical symbols |
| `dodai`, `cyberpunk`, `material`, `neobrutalism` | the designs and their themes |

Dials, gauges, segment displays, annunciators and scopes are always included. Each industry set builds only its own instruments and symbol group, so a vehicle-only app compiles no medical or defence code:

```toml
dodai-gpui-instruments = { version = "0.1", default-features = false, features = ["medical", "material"] }
```

## Looks

The vehicle instruments share one `Look`: Sport, Chronograph, Vintage, Heads-up, Segment LCD, Halo, Glass cockpit, Race and Line. Set it on a single instrument with `.look(..)`, or on a whole `DashTriptych` / `DashQuad` to restyle the ring, pods, lamps and limit sign together. An instrument given no look takes its design's own, `ChartStyle::vehicle` (Material: sport, Cyberpunk: heads-up, Dodai: halo, Neo-Brutalism: segment LCD). A gauge face also takes a `Layout` within its look: `Look::layouts()` lists them (four each. Sport: dial, sweep, track, ribbed, none with a hand; Chronograph: dial, bezel, sector, register; Vintage: dial, bezel, meter, ribbon; Heads-up: dial, cut, arc, tape; Segment LCD: dial, arc, bar, column; Halo: dial, capsule, arc, bar; Glass cockpit: dial, bar, tape, column; Race: dial, arc, bar, column; Line: dial, arc, bar, column), and a look draws its dial for a layout it lacks. Pods of readings (`EnergyPod`, `PowerFlow`, `TyrePod`, `RowsPod`) take a `PodLayout` instead: round, rings, a strip or a column, each drawn in the look's own case and type (the round looks draw the strip and the column as round registers); single readings in a pod (`SidePod`, `BoostPod`) take the gauge `Layout`.

```rust,no_run
use dodai_gpui::icons::Symbol;
use dodai_gpui_instruments::{
    Drivetrain, EnergyPod, Gear, GearSelector, Look, PowerFlow, RingTach, ShiftHint,
    ShiftLights, Telltales,
};
use dodai_instruments_core::instrument::Scale;

// An electric car's battery, recovering 18 kW, in the glass-cockpit look.
let energy = EnergyPod::new(64.0).power(-18.0).range("312 km").look(Look::GlassCockpit);
// Shift lights between 5 000 and 7 000 rpm with the gear, as a race rev bar.
let lights = ShiftLights::new(6400.0).range(5000.0, 7000.0).gear("4").look(Look::Race);
// The common telltales, the belt and main beam lit in their standard colours.
let lamps = Telltales::common().light(Symbol::SeatBelt).light(Symbol::HighBeam);
// An electric car: the motor's power from 60 kW recovered to 150 kW given, and the charge.
let ev = RingTach::new(Scale::new(-60.0, 150.0), 42.0, 88.0)
    .drivetrain(Drivetrain::Electric)
    .charge(0.68, "312 km");
// A hybrid's power flow: engine and motor driving together, the battery at 62 %.
let flow = PowerFlow::new(22.0, 39.0, 62.0);
// Fourth gear shifted by hand, prompting a change up, in the design's own look.
let gear = GearSelector::new(Gear::Manual(4)).hint(ShiftHint::Up);
```

These are display components, not medical or flight-safety devices: do not rely on them to monitor a patient or to fly or operate anything.

See them in the showcase: `cargo run -p dodai-showcase`.
