# dodai-gpui-instruments

Animated instruments for gpui in the Dodai designs: dials and gauges (15 styles), segment displays, annunciators and scopes, plus industry sets behind features (all on by default):

| Feature | Instruments |
|---|---|
| `automotive` | clusters, tachometer, speedometers, tire pressure, gear, odometer, power meter, level bar, compass |
| `aviation` | attitude indicator, heading and vertical tapes, sonar waterfall, countermeasures |
| `defense` | tactical plot, track list, threat rosette, stores panel, mission timer, radar scope |
| `industrial` | the process-plant and machine symbols (dials and annunciators are always included) |
| `medical` | vitals tiles, the ECG trace (`Oscilloscope::ecg`), the medical symbols |

Each feature also turns on the matching symbol group of `dodai-icons`. These are display components, not medical or flight-safety devices: do not rely on them to monitor a patient or to fly or operate anything.

See them in the showcase: `cargo run -p dodai-showcase`.
