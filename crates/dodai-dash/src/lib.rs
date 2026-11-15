//! # dodai-dash
//!
//! A dashboard engine for the Dodai UI design system. It knows what a dashboard **is** and how
//! to move one between people and programs; it does not collect, store or draw anything.
//!
//! - **The format** ([`Dashboard`], [`Panel`], [`Bundle`]): panels on a 12-column grid, variables
//!   written as `$name`, time ranges, display thresholds in semantic [`Tone`]s.
//! - **Import and export** ([`import`], [`convert`], [`Dashboard::export`]): TOML and JSON through
//!   one model, converted without loss; size-limited, version-checked, unknown fields reported,
//!   nothing executed. With the `schema` feature, `schema()` gives a JSON Schema.
//! - **Validation** ([`Dashboard::validate`]): the format always, and the host's metrics when a
//!   [`Catalogue`] is given.
//! - **Resolution** ([`resolve`]): asks the host's [`SeriesSource`] for each panel's data and
//!   returns a [`DashboardView`] of plain data. A toolkit crate turns that into charts.
//!
//! The host supplies the [`Catalogue`] (its metrics, units and dimensions) and the
//! [`SeriesSource`] (its data, already aggregated). Everything about what the metrics mean, alert
//! rules, storage and rendering stays outside.
//!
//! ```
//! use dodai_dash::{convert, Dashboard, Format};
//!
//! let toml = r#"
//! name = "Overview"
//! range = "1h"
//!
//! [[panels]]
//! title = "Power"
//! kind = "stacked_area"
//! metric = "gpu.power_w"
//! group_by = "host"
//! "#;
//!
//! let dash = Dashboard::import(toml, None).unwrap().value;
//! assert_eq!(dash.panels.len(), 1);
//!
//! // The same dashboard as JSON, and back, without loss.
//! let json = convert(toml, None, Format::Json).unwrap();
//! assert_eq!(Dashboard::import(&json, None).unwrap().value, dash);
//! ```

mod catalogue;
mod format;
mod interval;
mod layout;
mod source;
mod spec;
mod validate;
mod view;

#[cfg(feature = "schema")]
mod schema;

pub use catalogue::{Catalogue, MetricInfo, StaticCatalogue};
pub use format::{convert, import, Document, Format, FormatError, Parsed, MAX_IMPORT_BYTES};
pub use interval::{Interval, IntervalError};
pub use layout::{place, reading_order};
pub use source::{Point, Query, Series, SeriesSource, SourceError};
pub use spec::{
    Aggregation, Bundle, Dashboard, FixedFilter, Panel, PanelKind, Span, Threshold, Tone, Variable,
    FORMAT_VERSION, GRID_COLUMNS,
};
pub use validate::{Issue, Report, Severity, MAX_PANELS};
pub use view::{
    resolve, substitute, variables_with, DashboardView, PanelView, Variables, DEFAULT_RANGE,
};

#[cfg(feature = "schema")]
pub use schema::schema;
