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
//! | [`axis`] | secondary-axis tick alignment |
//! | [`gradient`] | continuous colour scales |

pub mod axis;
pub mod bins;
pub mod candles;
pub mod depth;
pub mod gantt;
pub mod gradient;
pub mod indicators;
pub mod math;
pub mod renko;
pub mod stack;

pub use candles::{heikin_ashi, price_range, Candle};
pub use gradient::Gradient;
pub use math::*;
