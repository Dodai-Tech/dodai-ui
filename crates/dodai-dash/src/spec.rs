//! The dashboard format: what a dashboard **is**, as plain serialisable data.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::interval::Interval;

/// The newest format version this crate reads and writes.
pub const FORMAT_VERSION: u32 = 1;

/// Width of the layout grid, in columns.
pub const GRID_COLUMNS: u32 = 12;

fn current_version() -> u32 {
    FORMAT_VERSION
}

/// One dashboard: a title, optional variables and time settings, and panels on a grid.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Dashboard {
    /// Format version of this file; see [`FORMAT_VERSION`].
    #[serde(default = "current_version")]
    pub version: u32,
    /// Display name.
    pub name: String,
    /// What the dashboard is for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// What the dashboard is about, in the host's words (a host may use it to pick where the
    /// dashboard applies). This crate attaches no meaning to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// How often the host should re-query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh: Option<Interval>,
    /// The default time range shown, ending now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Interval>,
    /// Also show the same range this long ago, aligned, for comparison.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare: Option<Interval>,
    /// Values the viewer can pick, used in filters as `$name`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<Variable>,
    /// The panels.
    #[serde(default, alias = "panel")]
    pub panels: Vec<Panel>,
}

/// A value chosen by the viewer and referenced as `$name` in panel filters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Variable {
    /// The name used after the `$`: letters, digits and `_`, not starting with a digit.
    pub name: String,
    /// Which dimension of the host's data this picks (for example a host's own word for a
    /// machine or a region); lets the host offer the right choices.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    /// Text shown to the viewer; the name when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The value used until the viewer picks another.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// One panel: a metric, how to cut it and how to draw it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Panel {
    /// Heading of the panel.
    pub title: String,
    /// How to draw it.
    pub kind: PanelKind,
    /// The metric to show, by name from the host's catalogue.
    pub metric: String,
    /// A line of help shown with the panel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Split the metric into one series per value of this dimension.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Keep only data where each dimension equals the value; `$name` refers to a variable.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub filter: BTreeMap<String, String>,
    /// How to combine values (over the group, and over each step).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agg: Option<Aggregation>,
    /// The unit shown; defaults to the metric's own unit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Overrides the dashboard's time range for this panel.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<Interval>,
    /// Width of one point of a series.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step: Option<Interval>,
    /// Lower end of the scale (gauges, bars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Upper end of the scale (gauges, bars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Where the panel sits on the grid; placed automatically when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub span: Option<Span>,
    /// Display rules that colour a value; the last matching entry wins, so list them from mild
    /// to severe.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thresholds: Vec<Threshold>,
}

/// How a panel is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum PanelKind {
    /// A line over time.
    Line,
    /// A filled line over time.
    Area,
    /// Filled lines stacked on each other.
    StackedArea,
    /// Bars, one per series or per step.
    Bar,
    /// A dial or bar showing one value on a scale.
    Gauge,
    /// One big number, with a small trend.
    Stat,
    /// A grid of coloured cells.
    Heatmap,
    /// Nested rectangles sized by value.
    Treemap,
    /// Flows between groups.
    Sankey,
    /// Rows and columns.
    Table,
    /// States over time as coloured spans.
    StateTimeline,
}

impl PanelKind {
    /// Every kind, in documentation order.
    pub const ALL: [PanelKind; 11] = [
        Self::Line,
        Self::Area,
        Self::StackedArea,
        Self::Bar,
        Self::Gauge,
        Self::Stat,
        Self::Heatmap,
        Self::Treemap,
        Self::Sankey,
        Self::Table,
        Self::StateTimeline,
    ];

    /// The name used in files and messages (`stacked_area`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Line => "line",
            Self::Area => "area",
            Self::StackedArea => "stacked_area",
            Self::Bar => "bar",
            Self::Gauge => "gauge",
            Self::Stat => "stat",
            Self::Heatmap => "heatmap",
            Self::Treemap => "treemap",
            Self::Sankey => "sankey",
            Self::Table => "table",
            Self::StateTimeline => "state_timeline",
        }
    }

    /// The size, in grid cells, used when a panel has no [`Span`].
    pub const fn default_size(self) -> (u32, u32) {
        match self {
            Self::Stat => (3, 2),
            Self::Gauge => (3, 3),
            Self::Table | Self::Treemap | Self::Sankey | Self::Heatmap => (6, 4),
            _ => (6, 3),
        }
    }
}

/// How values are combined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Aggregation {
    /// Sum.
    Sum,
    /// Mean.
    Avg,
    /// Smallest.
    Min,
    /// Largest.
    Max,
    /// Number of samples.
    Count,
    /// Most recent value.
    Last,
    /// Median.
    P50,
    /// 95th percentile.
    P95,
    /// 99th percentile.
    P99,
    /// Change per second of a counter.
    Rate,
    /// Change over the step of a counter.
    Delta,
}

impl Aggregation {
    /// The name used in files and messages (`p95`).
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sum => "sum",
            Self::Avg => "avg",
            Self::Min => "min",
            Self::Max => "max",
            Self::Count => "count",
            Self::Last => "last",
            Self::P50 => "p50",
            Self::P95 => "p95",
            Self::P99 => "p99",
            Self::Rate => "rate",
            Self::Delta => "delta",
        }
    }
}

/// A rectangle on the layout grid, in cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Span {
    /// Column of the left edge, from 0.
    pub x: u32,
    /// Row of the top edge, from 0.
    pub y: u32,
    /// Width in columns, 1 to [`GRID_COLUMNS`].
    pub w: u32,
    /// Height in rows, at least 1.
    pub h: u32,
}

impl Span {
    /// Whether the two rectangles share any cell.
    pub fn overlaps(&self, other: &Span) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    /// The row just below the rectangle.
    pub const fn bottom(&self) -> u32 {
        self.y + self.h
    }
}

/// A semantic status, mapped by each toolkit to the colour of the active design.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Tone {
    /// Neutral information.
    Info,
    /// Good.
    Success,
    /// Needs attention.
    Warning,
    /// Bad.
    Danger,
}

impl From<Tone> for dodai_core::Tone {
    fn from(t: Tone) -> Self {
        match t {
            Tone::Info => Self::Info,
            Tone::Success => Self::Success,
            Tone::Warning => Self::Warning,
            Tone::Danger => Self::Danger,
        }
    }
}

/// A display rule: values in a range are drawn in a tone. It colours; it never alerts.
///
/// With both bounds set, the value must be above `above` **and** below `below` (a band).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Threshold {
    /// Matches values greater than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub above: Option<f64>,
    /// Matches values less than this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub below: Option<f64>,
    /// The tone to draw with.
    pub tone: Tone,
}

impl Threshold {
    /// Whether `value` falls in this rule's range.
    pub fn matches(&self, value: f64) -> bool {
        if self.above.is_none() && self.below.is_none() {
            return false;
        }
        self.above.is_none_or(|a| value > a) && self.below.is_none_or(|b| value < b)
    }

    /// The tone for `value` under `rules`: the last matching rule, if any.
    pub fn tone_for(rules: &[Threshold], value: f64) -> Option<Tone> {
        rules
            .iter()
            .rev()
            .find(|r| r.matches(value))
            .map(|r| r.tone)
    }
}

/// Several dashboards in one file, for sharing a set.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Bundle {
    /// Format version of this file; see [`FORMAT_VERSION`].
    #[serde(default = "current_version")]
    pub version: u32,
    /// The dashboards.
    pub dashboards: Vec<Dashboard>,
}

/// One fixed (not variable) filter value in a dashboard, which the importing host may need to
/// map to something that exists on its side.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedFilter {
    /// The dimension.
    pub dimension: String,
    /// The value the file names.
    pub value: String,
}

impl Dashboard {
    /// A dashboard with a name and no panels.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            version: FORMAT_VERSION,
            name: name.into(),
            description: None,
            scope: None,
            refresh: None,
            range: None,
            compare: None,
            variables: Vec::new(),
            panels: Vec::new(),
        }
    }

    /// Every fixed filter value used by the panels, without duplicates, sorted. After importing
    /// a file made elsewhere, a host can check these against its own data and offer a mapping.
    pub fn fixed_filters(&self) -> Vec<FixedFilter> {
        let mut out: Vec<FixedFilter> = self
            .panels
            .iter()
            .flat_map(|p| p.filter.iter())
            .filter(|(_, v)| !v.contains('$'))
            .map(|(k, v)| FixedFilter {
                dimension: k.clone(),
                value: v.clone(),
            })
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// Replaces a fixed filter value everywhere; returns how many filters changed.
    pub fn remap_filter(&mut self, dimension: &str, from: &str, to: &str) -> usize {
        let mut n = 0;
        for p in &mut self.panels {
            if let Some(v) = p.filter.get_mut(dimension) {
                if v == from {
                    *v = to.to_string();
                    n += 1;
                }
            }
        }
        n
    }
}

impl Panel {
    /// A panel of `kind` showing `metric`.
    pub fn new(title: impl Into<String>, kind: PanelKind, metric: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            kind,
            metric: metric.into(),
            description: None,
            group_by: None,
            filter: BTreeMap::new(),
            agg: None,
            unit: None,
            range: None,
            step: None,
            min: None,
            max: None,
            span: None,
            thresholds: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_overlap_only_when_they_share_a_cell() {
        let a = Span {
            x: 0,
            y: 0,
            w: 6,
            h: 3,
        };
        assert!(a.overlaps(&Span {
            x: 5,
            y: 2,
            w: 3,
            h: 3
        }));
        assert!(!a.overlaps(&Span {
            x: 6,
            y: 0,
            w: 3,
            h: 3
        }));
        assert!(!a.overlaps(&Span {
            x: 0,
            y: 3,
            w: 6,
            h: 1
        }));
        assert_eq!(a.bottom(), 3);
    }

    #[test]
    fn thresholds_pick_the_last_match_and_support_bands() {
        let rules = [
            Threshold {
                above: Some(60.0),
                below: None,
                tone: Tone::Warning,
            },
            Threshold {
                above: Some(80.0),
                below: None,
                tone: Tone::Danger,
            },
        ];
        assert_eq!(Threshold::tone_for(&rules, 50.0), None);
        assert_eq!(Threshold::tone_for(&rules, 70.0), Some(Tone::Warning));
        assert_eq!(Threshold::tone_for(&rules, 90.0), Some(Tone::Danger));
        let band = Threshold {
            above: Some(10.0),
            below: Some(20.0),
            tone: Tone::Success,
        };
        assert!(band.matches(15.0) && !band.matches(25.0) && !band.matches(5.0));
        let empty = Threshold {
            above: None,
            below: None,
            tone: Tone::Info,
        };
        assert!(!empty.matches(1.0), "a rule without bounds matches nothing");
    }

    #[test]
    fn fixed_filters_skip_variables_and_remap_changes_them() {
        let mut d = Dashboard::new("x");
        let mut p = Panel::new("a", PanelKind::Line, "m");
        p.filter.insert("host".into(), "rig-1".into());
        p.filter.insert("group".into(), "$g".into());
        d.panels.push(p.clone());
        d.panels.push(p);
        assert_eq!(
            d.fixed_filters(),
            vec![FixedFilter {
                dimension: "host".into(),
                value: "rig-1".into()
            }]
        );
        assert_eq!(d.remap_filter("host", "rig-1", "box-9"), 2);
        assert_eq!(d.fixed_filters()[0].value, "box-9");
    }

    #[test]
    fn names_match_the_serialised_form() {
        for k in PanelKind::ALL {
            let json = serde_json::to_string(&k).unwrap();
            assert_eq!(json, format!("\"{}\"", k.name()));
        }
        for a in [Aggregation::P95, Aggregation::Sum, Aggregation::Delta] {
            assert_eq!(
                serde_json::to_string(&a).unwrap(),
                format!("\"{}\"", a.name())
            );
        }
    }

    #[test]
    fn tones_map_to_the_core_tones() {
        assert_eq!(
            dodai_core::Tone::from(Tone::Danger),
            dodai_core::Tone::Danger
        );
        assert_eq!(dodai_core::Tone::from(Tone::Info), dodai_core::Tone::Info);
    }
}
