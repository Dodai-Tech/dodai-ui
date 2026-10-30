//! # dodai-instruments-core
//!
//! Pure, renderer-agnostic instrument logic: scales, zones and alarms, gauge geometry, flight
//! displays, tactical symbols and seven-segment digits. Toolkit crates
//! (`dodai-gpui-instruments`, ...) only paint what this crate computes. Number formats and
//! polar maths come from `dodai-charts-core`.
//!
//! | Module | Contents |
//! |---|---|
//! | [`instrument`] | scales, zones, limits, alarms |
//! | [`gauge`] | lit cells, ticks, zones, needle and cell polygons, hex grids, ring radii |
//! | [`flight`] | attitude horizon, pitch ladder, tapes, sonar rows |
//! | [`segments`] | seven-segment digit geometry |
//! | [`tactical`] | contacts, map symbols, threat wedges, countdown |
#![warn(missing_docs)]

pub mod flight;
pub mod gauge;
pub mod instrument;
pub mod segments;
pub mod tactical;
