//! From a dashboard and a data source to view models: plain data any toolkit can draw.

use std::collections::BTreeMap;

use crate::interval::Interval;
use crate::layout;
use crate::source::{Query, Series, SeriesSource};
use crate::spec::{Dashboard, Panel, PanelKind, Span, Threshold, Tone};
use crate::validate::variable_refs;

/// Values for a dashboard's variables, by name.
pub type Variables = BTreeMap<String, String>;

/// Range used when neither the panel nor the dashboard sets one.
pub const DEFAULT_RANGE: Interval = Interval::secs(3_600);

/// A dashboard ready to draw.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardView {
    /// The dashboard's name.
    pub name: String,
    /// What it is for.
    pub description: Option<String>,
    /// How often to resolve again.
    pub refresh: Option<Interval>,
    /// The panels, in the order of the file; each carries its grid span.
    pub panels: Vec<PanelView>,
}

impl DashboardView {
    /// Indices into [`panels`](Self::panels) in reading order, for narrow screens.
    pub fn reading_order(&self) -> Vec<usize> {
        let spans: Vec<Span> = self.panels.iter().map(|p| p.span).collect();
        layout::reading_order(&spans)
    }
}

/// A panel ready to draw: its data and everything needed to style it.
#[derive(Debug, Clone, PartialEq)]
pub struct PanelView {
    /// Heading, with variables substituted.
    pub title: String,
    /// How to draw it.
    pub kind: PanelKind,
    /// A line of help.
    pub description: Option<String>,
    /// The unit to show.
    pub unit: Option<String>,
    /// Lower end of the scale.
    pub min: Option<f64>,
    /// Upper end of the scale.
    pub max: Option<f64>,
    /// Where it sits on the grid.
    pub span: Span,
    /// Display rules for colouring values.
    pub thresholds: Vec<Threshold>,
    /// The series, one per group (one when not grouped).
    pub series: Vec<Series>,
    /// The comparison period's series, moved onto the present's time axis; empty when the
    /// dashboard sets no comparison.
    pub compared: Vec<Series>,
    /// Set when the panel could not be resolved; the panel is then drawn as an error state and
    /// the rest of the dashboard is unaffected.
    pub error: Option<String>,
}

impl PanelView {
    /// The newest value of each series, with the series name.
    pub fn latest(&self) -> Vec<(String, f64)> {
        self.series
            .iter()
            .filter_map(|s| s.last().map(|v| (s.name(), v)))
            .collect()
    }

    /// The tone `value` is drawn in under this panel's thresholds.
    pub fn tone(&self, value: f64) -> Option<Tone> {
        Threshold::tone_for(&self.thresholds, value)
    }
}

/// Fills in the variables: the dashboard's defaults, overridden by `overrides`.
pub fn variables_with(dashboard: &Dashboard, overrides: &Variables) -> Variables {
    let mut out: Variables = dashboard
        .variables
        .iter()
        .filter_map(|v| v.default.clone().map(|d| (v.name.clone(), d)))
        .collect();
    out.extend(overrides.iter().map(|(k, v)| (k.clone(), v.clone())));
    out
}

/// Replaces `$name` and `${name}` in `text`; fails naming the first variable without a value.
pub fn substitute(text: &str, vars: &Variables) -> Result<String, String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '$' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let braced = chars.get(i + 1) == Some(&'{');
        let start = i + 1 + usize::from(braced);
        let mut end = start;
        while end < chars.len() && (chars[end].is_ascii_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        if end == start {
            out.push('$');
            i += 1;
            continue;
        }
        let name: String = chars[start..end].iter().collect();
        match vars.get(&name) {
            Some(v) => out.push_str(v),
            None => return Err(format!("variable `{name}` has no value")),
        }
        i = end + usize::from(braced && chars.get(end) == Some(&'}'));
    }
    Ok(out)
}

/// Asks `source` for every panel's data and returns the view model. `now_ms` is the end of the
/// time range, in milliseconds since the Unix epoch (passed in, never read from a clock, so the
/// result is reproducible). A panel that fails becomes an error panel; the rest still resolve.
pub fn resolve(
    dashboard: &Dashboard,
    overrides: &Variables,
    source: &dyn SeriesSource,
    now_ms: i64,
) -> DashboardView {
    let vars = variables_with(dashboard, overrides);
    let spans = layout::place(&dashboard.panels);
    let panels = dashboard
        .panels
        .iter()
        .zip(spans)
        .map(|(p, span)| resolve_panel(dashboard, p, span, &vars, source, now_ms))
        .collect();
    DashboardView {
        name: dashboard.name.clone(),
        description: dashboard.description.clone(),
        refresh: dashboard.refresh,
        panels,
    }
}

fn resolve_panel(
    d: &Dashboard,
    p: &Panel,
    span: Span,
    vars: &Variables,
    source: &dyn SeriesSource,
    now_ms: i64,
) -> PanelView {
    let mut view = PanelView {
        title: p.title.clone(),
        kind: p.kind,
        description: p.description.clone(),
        unit: p.unit.clone(),
        min: p.min,
        max: p.max,
        span,
        thresholds: p.thresholds.clone(),
        series: Vec::new(),
        compared: Vec::new(),
        error: None,
    };
    let result = (|| -> Result<(), String> {
        if !variable_refs(&p.title).is_empty() {
            view.title = substitute(&p.title, vars)?;
        }
        let mut filter = BTreeMap::new();
        for (k, v) in &p.filter {
            filter.insert(k.clone(), substitute(v, vars)?);
        }
        let range = p.range.or(d.range).unwrap_or(DEFAULT_RANGE);
        let range_ms = i64::try_from(range.duration().as_millis()).unwrap_or(i64::MAX);
        let query = Query {
            metric: p.metric.clone(),
            filter,
            group_by: p.group_by.clone(),
            agg: p.agg,
            from: now_ms.saturating_sub(range_ms),
            to: now_ms,
            step: p.step,
        };
        view.series = source.query(&query).map_err(|e| e.to_string())?;
        if let Some(c) = d.compare {
            let back = i64::try_from(c.duration().as_millis()).unwrap_or(i64::MAX);
            let earlier = Query {
                from: query.from.saturating_sub(back),
                to: query.to.saturating_sub(back),
                ..query
            };
            view.compared = source
                .query(&earlier)
                .map_err(|e| e.to_string())?
                .iter()
                .map(|s| s.shifted(back))
                .collect();
        }
        Ok(())
    })();
    view.error = result.err();
    view
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::source::{Point, SourceError};
    use crate::spec::Variable;

    struct Fake {
        calls: RefCell<Vec<Query>>,
        fail_metric: Option<&'static str>,
    }

    impl Fake {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
                fail_metric: None,
            }
        }
    }

    impl SeriesSource for Fake {
        fn query(&self, q: &Query) -> Result<Vec<Series>, SourceError> {
            self.calls.borrow_mut().push(q.clone());
            if self.fail_metric == Some(q.metric.as_str()) {
                return Err(SourceError("no such metric".into()));
            }
            Ok(vec![Series::new(
                BTreeMap::new(),
                vec![Point { t: q.from, v: 1.0 }, Point { t: q.to, v: 90.0 }],
            )])
        }
    }

    fn dashboard() -> Dashboard {
        let mut d = Dashboard::new("d");
        d.range = Some("2h".parse().unwrap());
        d.variables.push(Variable {
            name: "group".into(),
            kind: None,
            label: None,
            default: Some("rigs".into()),
        });
        let mut p = Panel::new("Power of $group", PanelKind::Gauge, "gpu.power_w");
        p.filter.insert("group".into(), "$group".into());
        p.thresholds = vec![Threshold {
            above: Some(80.0),
            below: None,
            tone: Tone::Danger,
        }];
        d.panels.push(p);
        d
    }

    #[test]
    fn substitution_handles_both_spellings_and_missing_values() {
        let vars: Variables = [("a".to_string(), "1".to_string())].into();
        assert_eq!(substitute("x $a ${a}y $", &vars).unwrap(), "x 1 1y $");
        assert_eq!(
            substitute("$b", &vars).unwrap_err(),
            "variable `b` has no value"
        );
    }

    #[test]
    fn defaults_are_overridden_by_the_viewer() {
        let d = dashboard();
        assert_eq!(variables_with(&d, &Variables::new())["group"], "rigs");
        let o: Variables = [("group".to_string(), "pcs".to_string())].into();
        assert_eq!(variables_with(&d, &o)["group"], "pcs");
    }

    #[test]
    fn resolving_asks_the_source_with_substituted_filters_and_the_right_range() {
        let src = Fake::new();
        let view = resolve(&dashboard(), &Variables::new(), &src, 10_000_000);
        let q = &src.calls.borrow()[0];
        assert_eq!(q.filter["group"], "rigs");
        assert_eq!(q.to, 10_000_000);
        assert_eq!(q.from, 10_000_000 - 2 * 3_600_000);
        let p = &view.panels[0];
        assert_eq!(p.title, "Power of rigs");
        assert!(p.error.is_none());
        assert_eq!(p.latest(), vec![("all".to_string(), 90.0)]);
        assert_eq!(p.tone(90.0), Some(Tone::Danger));
        assert_eq!(p.tone(10.0), None);
    }

    #[test]
    fn a_comparison_is_queried_earlier_and_shifted_onto_the_present() {
        let mut d = dashboard();
        d.compare = Some("1d".parse().unwrap());
        let src = Fake::new();
        let view = resolve(&d, &Variables::new(), &src, 200_000_000);
        let calls = src.calls.borrow();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].to, 200_000_000 - 86_400_000);
        let cmp = &view.panels[0].compared[0];
        assert_eq!(
            cmp.points.last().unwrap().t,
            200_000_000,
            "shifted back onto now"
        );
    }

    #[test]
    fn a_failing_panel_does_not_stop_the_others() {
        let mut d = dashboard();
        d.panels.push(Panel::new("Bad", PanelKind::Line, "boom"));
        d.panels.push(Panel::new("Unset", PanelKind::Line, "m"));
        d.panels[2].filter.insert("host".into(), "$missing".into());
        let src = Fake {
            calls: RefCell::new(Vec::new()),
            fail_metric: Some("boom"),
        };
        let view = resolve(&d, &Variables::new(), &src, 0);
        assert!(view.panels[0].error.is_none());
        assert_eq!(view.panels[1].error.as_deref(), Some("no such metric"));
        assert_eq!(
            view.panels[2].error.as_deref(),
            Some("variable `missing` has no value")
        );
    }

    #[test]
    fn panels_carry_their_grid_span() {
        let view = resolve(&dashboard(), &Variables::new(), &Fake::new(), 0);
        assert_eq!(
            view.panels[0].span,
            Span {
                x: 0,
                y: 0,
                w: 3,
                h: 3
            }
        );
        assert_eq!(view.reading_order(), vec![0]);
    }
}
