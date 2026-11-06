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
    .look(Look::Chrono)
}
```

A reading past a limit colours its value (pointer, sweep, bar or figure) in the limit's colour. Optionally, the looks drawn on a screen (heads-up, digital, futuristic, glass cockpit) colour the whole display with it: `GaugeFace`, `SidePod`, `RingTach`, `BoostPod` and `DualGauge` take `.at_limit(AtLimit::Display)`; the default is `AtLimit::Value`.

Without an installed theme the instruments use the first theme of the enabled designs, or a plain neutral theme when no design is enabled. An app can also install its own `dodai_core::Theme` with `dodai_gpui::install`.

## Features

The default turns on every industry set and no design.

| Feature | What it adds |
|---|---|
| `automotive` | dashes, pods, gauge faces in eight looks (`Look`), tachometer, speedometers, tyre pressure, gear, odometer, compass, EV energy, shift lights; the vehicle symbols |
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

The vehicle instruments share one `Look`: Classic, Chrono, Vintage, Heads-up, Digital, Futuristic, Glass cockpit and Race. Set it on a single instrument with `.look(..)`, or on a whole `DashTriptych` / `DashQuad` to restyle the ring, pods, lamps and limit sign together.

```rust,no_run
use dodai_gpui_instruments::{EnergyPod, Look, ShiftLights};

// An electric car's battery, recovering 18 kW, in the glass-cockpit look.
let energy = EnergyPod::new(64.0).power(-18.0).range("312 km").look(Look::Cockpit);
// Shift lights between 5 000 and 7 000 rpm with the gear, as a race rev bar.
let lights = ShiftLights::new(6400.0).range(5000.0, 7000.0).gear("4").look(Look::Race);
```

These are display components, not medical or flight-safety devices: do not rely on them to monitor a patient or to fly or operate anything.

See them in the showcase: `cargo run -p dodai-showcase`.
