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
//! | [`stack`] | shared-gutter chart stacking |
//! | [`bins`] | histogram binning |
//! | [`calendar`] | civil-date helpers |
//! | [`axis`] | secondary-axis tick alignment |
//! | [`gradient`] | continuous colour scales |
//! | [`pulse`] | ECG / vital-sign signal generators, beat detection |
//! | [`sankey`] | flow diagram layout |
//! | [`density`] | kernel density estimate |
//! | [`instrument`] | scales, zones, limits, alarms |
//! | [`flight`] | attitude horizon, pitch ladder, tapes, sonar rows |
//! | [`gauge`] | lit cells, ticks, zones, needle and cell polygons, hex grids, ring radii |
//! | [`segments`] | seven-segment digit geometry |
//! | [`tactical`] | contacts, map symbols, threat wedges, countdown |
//! | [`stream`] | fixed-size sample window for live charts |
//! | [`timeaxis`] | calendar-aware time ticks, unix ↔ civil date, UTC-offset labels |

pub mod axis;
pub mod bins;
pub mod calendar;
pub mod candles;
pub mod density;
pub mod depth;
pub mod flight;
pub mod gantt;
pub mod gauge;
pub mod gradient;
pub mod indicators;
pub mod instrument;
pub mod math;
pub mod pulse;
pub mod renko;
pub mod sankey;
pub mod segments;
pub mod stack;
pub mod stream;
pub mod tactical;
pub mod timeaxis;

pub use candles::{heikin_ashi, price_range, Candle};
pub use gradient::Gradient;
pub use math::*;
