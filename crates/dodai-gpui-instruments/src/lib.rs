//! # dodai-gpui-instruments
//!
//! Animated instruments for [gpui](https://gpui.rs), drawn in the active Dodai design through
//! `dodai-gpui`'s theme: needles ease to new values with the design's response, alarms flash,
//! scopes sweep. Geometry comes from `dodai-charts-core`.
//!
//! Always included: dials and gauges ([`DialGauge`], [`Gauge`] in 15 [`GaugeStyle`]s), segment
//! displays, annunciators, and the scopes ([`Oscilloscope`], [`Spectrum`],
//! [`DotMatrix`], [`SignalBars`], [`HexMap`], [`RingStack`]). By industry (all on by default):
//!
//! | Feature | Instruments |
//! |---|---|
//! | `automotive` | tachometer, speedometers, tire pressure, gear, odometer, power meter, level bar, compass, EV energy, shift lights |
//! | `aviation` | attitude indicator, heading and vertical tapes, sonar waterfall, countermeasures |
//! | `defense` | tactical plot, track list, threat rosette, stores panel, mission timer, radar scope |
//! | `industrial` | the process-plant and machine symbols (dials and annunciators are always included) |
//! | `medical` | vitals tiles, the ECG trace (`Oscilloscope::ecg`) and the medical symbols |
//!
//! Each feature also turns on the matching symbol group of `dodai-icons`. The instruments are
//! display components, not medical or flight-safety devices: do not rely on them to monitor a
//! patient or to fly or operate anything.
//!
//! ```no_run
//! use dodai_gpui_instruments::{DialGauge, Scale};
//! let dial = DialGauge::new(Scale::new(0.0, 120.0), 72.0);
//! ```
#![warn(missing_docs)]

pub mod dial;
mod draw;
pub mod gauge;
pub mod gauges;
pub(crate) mod gauges_extra;
pub mod gradient;
pub mod scope;
pub mod segment;

#[cfg(feature = "automotive")]
#[cfg(feature = "automotive")]
pub mod dash;
#[cfg(feature = "automotive")]
pub mod faces;
#[cfg(feature = "aviation")]
pub mod flight;
#[cfg(feature = "automotive")]
mod frames;
#[cfg(feature = "automotive")]
#[cfg(feature = "automotive")]
pub mod look;
#[cfg(feature = "automotive")]
mod mfd;
#[cfg(feature = "automotive")]
pub(crate) mod pod_math;
#[cfg(feature = "automotive")]
pub mod pods;
#[cfg(feature = "automotive")]
mod race;
#[cfg(feature = "automotive")]
mod shapes;
#[cfg(feature = "automotive")]
pub mod shift;
#[cfg(feature = "defense")]
pub mod tactical;
#[cfg(feature = "automotive")]
pub mod vehicle;

#[cfg(feature = "medical")]
pub use dial::VitalsTile;
pub use dial::{severity_color, Annunciator, DialGauge};
pub use dodai_gpui::plot::{chart_glow, set_chart_glow, ChartGlow};
pub use dodai_gpui::Live;
pub use dodai_instruments_core::instrument::{Alarm, Limits, Scale, Severity};
#[cfg(feature = "defense")]
pub use dodai_instruments_core::tactical::{Affiliation, Contact};
pub use gauge::Gauge;
pub use gauges::GaugeStyle;
pub use gradient::{GradientBar, GradientDial, GradientRing, Ramp};
#[cfg(feature = "defense")]
pub use scope::{Blip, RadarScope};
pub use scope::{DotMatrix, HexMap, Oscilloscope, RingStack, SignalBars, Spectrum};
pub use segment::SegmentDisplay;

#[cfg(feature = "automotive")]
#[cfg(feature = "automotive")]
pub use dash::{
    DashTriptych, LimitSign, MenuRing, Pod, PodStyle, RingTach, SidePod, Telltales, Tone,
};
#[cfg(feature = "automotive")]
pub use faces::GaugeFace;
#[cfg(feature = "aviation")]
pub use flight::{AttitudeIndicator, Countermeasures, HeadingTape, SonarWaterfall, VerticalTape};
#[cfg(feature = "automotive")]
#[cfg(feature = "automotive")]
pub use look::{AtLimit, Layout, Look};
#[cfg(feature = "automotive")]
pub use pods::{
    BoostPod, CompassPod, DashQuad, DualGauge, DualScale, EnergyPod, GForcePod, MediaPod,
    NoticePod, PodIcon, PodRow, RowsPod, TyrePod,
};
#[cfg(feature = "automotive")]
pub use shift::ShiftLights;
#[cfg(feature = "defense")]
pub use tactical::{
    MissionTimer, Station, StationState, StoresPanel, TacticalPlot, Threat, ThreatRosette,
    TrackList,
};
#[cfg(feature = "automotive")]
pub use vehicle::{Compass, GearIndicator, LevelBar, Odometer, PowerMeter, TirePressure};

/// Compiles the README's examples as doctests so they cannot rot.
#[cfg(all(doctest, feature = "automotive"))]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
