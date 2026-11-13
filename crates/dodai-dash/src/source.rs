//! What a host answers: series of points for a query. This crate never stores or collects data.

use std::collections::BTreeMap;
use std::fmt;

use crate::interval::Interval;
use crate::spec::Aggregation;

/// One sample: a time in milliseconds since the Unix epoch and a value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// Milliseconds since the Unix epoch.
    pub t: i64,
    /// The value.
    pub v: f64,
}

/// A named run of points: one line of a chart or one row of a table.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Series {
    /// The dimension values that identify the series (empty when the data was not grouped).
    pub labels: BTreeMap<String, String>,
    /// The points, oldest first.
    pub points: Vec<Point>,
}

impl Series {
    /// A series with the given labels and points.
    pub fn new(labels: BTreeMap<String, String>, points: Vec<Point>) -> Self {
        Self { labels, points }
    }

    /// A short name for legends: the label values joined with a slash, or `all`.
    pub fn name(&self) -> String {
        if self.labels.is_empty() {
            "all".to_string()
        } else {
            self.labels.values().cloned().collect::<Vec<_>>().join("/")
        }
    }

    /// The newest value.
    pub fn last(&self) -> Option<f64> {
        self.points.last().map(|p| p.v)
    }

    /// The smallest value, ignoring NaN.
    pub fn min(&self) -> Option<f64> {
        self.points
            .iter()
            .map(|p| p.v)
            .filter(|v| !v.is_nan())
            .reduce(f64::min)
    }

    /// The largest value, ignoring NaN.
    pub fn max(&self) -> Option<f64> {
        self.points
            .iter()
            .map(|p| p.v)
            .filter(|v| !v.is_nan())
            .reduce(f64::max)
    }

    /// The mean value, ignoring NaN.
    pub fn mean(&self) -> Option<f64> {
        let vals: Vec<f64> = self
            .points
            .iter()
            .map(|p| p.v)
            .filter(|v| !v.is_nan())
            .collect();
        if vals.is_empty() {
            None
        } else {
            Some(vals.iter().sum::<f64>() / vals.len() as f64)
        }
    }

    /// The same series moved later in time by `ms` milliseconds (used to lay a comparison
    /// period over the present).
    pub fn shifted(&self, ms: i64) -> Series {
        Series {
            labels: self.labels.clone(),
            points: self
                .points
                .iter()
                .map(|p| Point {
                    t: p.t + ms,
                    v: p.v,
                })
                .collect(),
        }
    }
}

/// What to fetch.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    /// The metric.
    pub metric: String,
    /// Keep only data where each dimension equals the value (variables already substituted).
    pub filter: BTreeMap<String, String>,
    /// One series per value of this dimension; `None` for a single series.
    pub group_by: Option<String>,
    /// How to combine values.
    pub agg: Option<Aggregation>,
    /// Start of the range, milliseconds since the Unix epoch.
    pub from: i64,
    /// End of the range, milliseconds since the Unix epoch.
    pub to: i64,
    /// Width of one point; the source picks a sensible one when `None`.
    pub step: Option<Interval>,
}

/// Why a source could not answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceError(pub String);

impl fmt::Display for SourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SourceError {}

/// Where data comes from. The host implements it over its own storage and does the aggregating;
/// this crate only asks.
pub trait SeriesSource {
    /// Answers `query` with zero or more series.
    fn query(&self, query: &Query) -> Result<Vec<Series>, SourceError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(vals: &[f64]) -> Series {
        Series::new(
            BTreeMap::new(),
            vals.iter()
                .enumerate()
                .map(|(i, v)| Point {
                    t: i as i64 * 1000,
                    v: *v,
                })
                .collect(),
        )
    }

    #[test]
    fn summaries_ignore_nan() {
        let x = s(&[2.0, f64::NAN, 6.0]);
        assert_eq!(x.min(), Some(2.0));
        assert_eq!(x.max(), Some(6.0));
        assert_eq!(x.mean(), Some(4.0));
        assert_eq!(x.last(), Some(6.0));
        assert_eq!(s(&[]).mean(), None);
        assert_eq!(s(&[]).min(), None);
    }

    #[test]
    fn names_come_from_labels() {
        let mut l = BTreeMap::new();
        assert_eq!(Series::new(l.clone(), vec![]).name(), "all");
        l.insert("host".to_string(), "rig-1".to_string());
        l.insert("gpu".to_string(), "0".to_string());
        assert_eq!(Series::new(l, vec![]).name(), "0/rig-1");
    }

    #[test]
    fn shifting_moves_time_only() {
        let x = s(&[1.0, 2.0]).shifted(5_000);
        assert_eq!(x.points[0], Point { t: 5_000, v: 1.0 });
        assert_eq!(x.points[1], Point { t: 6_000, v: 2.0 });
    }
}
