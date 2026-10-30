//! # dodai-charts-core
//!
//! Pure, renderer-agnostic chart logic: scales and ticks, geometry, financial
//! transforms, layouts. Toolkit crates (`dodai-gpui`, ...) only paint what
//! this crate computes, so a new chart is written once for every toolkit.
//!
//! | Module | Contents |
//! |---|---|
//! | [`math`] | ticks, number formats, smoothing, arcs, markers |
//! | [`candles`] | [`Candle`], Heikin-Ashi |
//! | [`indicators`] | SMA, EMA, Bollinger, RSI, MACD |
//! | [`renko`] | Renko bricks |
//! | [`depth`] | order-book cumulative curves |
//! | [`gantt`] | task timeline layout |
//! | [`fan`] | span fan: lifespans as spokes round an arc |
//! | [`stack`] | shared-gutter chart stacking |
//! | [`bins`] | histogram binning |
//! | [`calendar`] | civil-date helpers |
//! | [`axis`] | secondary-axis tick alignment |
//! | [`gradient`] | continuous colour scales |
//! | [`pulse`] | ECG / vital-sign signal generators, beat detection |
//! | [`sankey`] | flow diagram layout |
//! | [`density`] | kernel density estimate |
//! | [`stream`] | fixed-size sample window for live charts |
//! | [`timeaxis`] | calendar-aware time ticks, unix ↔ civil date, UTC-offset labels |
//!
//! Instrument geometry (gauges, flight, tactical, segment displays) is in `dodai-instruments-core`.

pub mod axis;
pub mod bins;
pub mod calendar;
pub mod candles;
pub mod chord;
pub mod control;
pub mod density;
pub mod depth;
pub mod fan;
pub mod force;
pub mod gantt;
pub mod gradient;
pub mod hexbin;
pub mod hierarchy;
pub mod horizon;
pub mod indicators;
pub mod layers;
pub mod math;
pub mod pack;
pub mod pulse;
pub mod rank;
pub mod renko;
pub mod sankey;
pub mod share;
pub mod stack;
pub mod stream;
pub mod swarm;
pub mod timeaxis;

pub use candles::{heikin_ashi, price_range, Candle};
pub use gradient::Gradient;
pub use math::*;
