//! # dodai-charts
//!
//! Renderer-agnostic chart logic (the root modules) and a chart set per toolkit, each behind its
//! own feature: `gpui` ([`gpui`], interactive animated charts drawn in the active Dodai design).
//!
//! Pure chart logic: scales and ticks, geometry, financial
//! transforms, layouts. Toolkit modules only paint what
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
//! | [`shards`] | shard map: an outline cut into value-sized convex cells |
//! | [`cascade`] | split flow: a quantity splitting down through levels |
//! | [`peaks`] | peak range: overlapping series as mountains |
//! | [`milestones`] | time scatter: record frontier, trails, nearest point |
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
//! Instrument geometry (gauges, flight, tactical, segment displays) is in `dodai-instruments`.

pub mod axis;
pub mod bins;
pub mod calendar;
pub mod candles;
pub mod cascade;
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
pub use dodai_core::math;
pub mod milestones;
pub mod pack;
pub mod peaks;
pub mod pulse;
pub mod rank;
pub mod renko;
pub mod sankey;
pub mod shards;
pub mod share;
pub mod stack;
pub use dodai_core::stream;
pub mod swarm;
pub mod timeaxis;

pub use candles::{heikin_ashi, price_range, Candle};
pub use gradient::Gradient;
pub use math::*;

/// Interactive, animated charts for gpui.
#[cfg(feature = "gpui")]
pub mod gpui;
