//! The host's list of metrics: what exists, in which unit, cut by which dimensions.

use serde::{Deserialize, Serialize};

use crate::spec::{Aggregation, PanelKind};

/// One metric a host can show.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricInfo {
    /// The name used in dashboards, for example `gpu.power_w`.
    pub name: String,
    /// One line for pickers and help.
    #[serde(default)]
    pub description: String,
    /// The unit shown by default.
    #[serde(default)]
    pub unit: Option<String>,
    /// What the metric can be filtered and grouped by.
    #[serde(default)]
    pub dimensions: Vec<String>,
    /// The aggregations that make sense; empty means any.
    #[serde(default)]
    pub aggregations: Vec<Aggregation>,
    /// The panel kinds that suit it; empty means any.
    #[serde(default)]
    pub kinds: Vec<PanelKind>,
}

impl MetricInfo {
    /// A metric with a name and nothing else.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            unit: None,
            dimensions: Vec::new(),
            aggregations: Vec::new(),
            kinds: Vec::new(),
        }
    }

    /// Sets the description.
    pub fn describe(mut self, text: impl Into<String>) -> Self {
        self.description = text.into();
        self
    }

    /// Sets the unit.
    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    /// Sets the dimensions.
    pub fn dimensions<S: Into<String>>(mut self, dims: impl IntoIterator<Item = S>) -> Self {
        self.dimensions = dims.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the aggregations.
    pub fn aggregations(mut self, aggs: impl IntoIterator<Item = Aggregation>) -> Self {
        self.aggregations = aggs.into_iter().collect();
        self
    }

    /// Sets the suitable panel kinds.
    pub fn kinds(mut self, kinds: impl IntoIterator<Item = PanelKind>) -> Self {
        self.kinds = kinds.into_iter().collect();
        self
    }
}

/// What a host knows about its own metrics. Implement it over whatever the host has; this crate
/// never invents a metric.
pub trait Catalogue {
    /// The metric called `name`, if the host has it.
    fn metric(&self, name: &str) -> Option<MetricInfo>;

    /// Every metric, for pickers and for suggestions in error messages.
    fn metrics(&self) -> Vec<MetricInfo>;
}

/// A catalogue held in memory; also loadable from a file by a host, since it is serialisable.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StaticCatalogue {
    /// The metrics.
    #[serde(default)]
    pub metrics: Vec<MetricInfo>,
}

impl StaticCatalogue {
    /// A catalogue of `metrics`.
    pub fn new(metrics: impl IntoIterator<Item = MetricInfo>) -> Self {
        Self {
            metrics: metrics.into_iter().collect(),
        }
    }
}

impl Catalogue for StaticCatalogue {
    fn metric(&self, name: &str) -> Option<MetricInfo> {
        self.metrics.iter().find(|m| m.name == name).cloned()
    }

    fn metrics(&self) -> Vec<MetricInfo> {
        self.metrics.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_metrics_by_name() {
        let c = StaticCatalogue::new([
            MetricInfo::new("a.b").unit("W").dimensions(["host"]),
            MetricInfo::new("c.d"),
        ]);
        assert_eq!(c.metric("a.b").unwrap().unit.as_deref(), Some("W"));
        assert!(c.metric("nope").is_none());
        assert_eq!(c.metrics().len(), 2);
    }

    #[test]
    fn a_catalogue_round_trips_through_toml() {
        let c = StaticCatalogue::new([MetricInfo::new("a.b")
            .describe("x")
            .aggregations([Aggregation::Sum, Aggregation::P95])
            .kinds([PanelKind::Line])]);
        let text = toml::to_string(&c).unwrap();
        assert_eq!(toml::from_str::<StaticCatalogue>(&text).unwrap(), c);
    }
}
