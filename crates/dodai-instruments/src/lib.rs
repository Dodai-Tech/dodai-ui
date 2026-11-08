//! # dodai-instruments
//!
//! Pure, renderer-agnostic instrument logic: scales, zones and alarms, gauge geometry, flight
//! displays, tactical symbols and seven-segment digits. Toolkit modules
//! (feature `gpui`: [`gpui`]) only paint what this crate computes. Number formats and
//! polar maths come from `dodai-charts`.
//!
//! | Module | Contents |
//! |---|---|
//! | [`instrument`] | scales, zones, limits, alarms |
//! | [`gauge`] | lit cells, ticks, zones, needle and cell polygons, hex grids, ring radii |
//! | [`flight`] | attitude horizon, pitch ladder, tapes, sonar rows |
//! | [`segments`] | seven-segment digit geometry |
//! | [`orientation`] | axis gizmo: placement, hit test, orbit drag, standard views |
//! | [`tactical`] | contacts, map symbols, threat wedges, countdown |
#![warn(missing_docs)]

pub mod flight;
pub mod gauge;
pub mod instrument;
pub use dodai_core::orientation;
pub mod segments;
pub mod tactical;

/// Animated instruments for gpui.
#[cfg(feature = "gpui")]
pub mod gpui;
