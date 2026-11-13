//! Checking a dashboard: always against the format, and against a host's catalogue when given.

use std::collections::HashSet;
use std::fmt;

use crate::catalogue::Catalogue;
use crate::layout;
use crate::spec::{Dashboard, FORMAT_VERSION, GRID_COLUMNS};

/// The most panels one dashboard may have.
pub const MAX_PANELS: usize = 200;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// The dashboard cannot be used as it is.
    Error,
    /// It works, but probably not as intended.
    Warning,
}

/// One finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    /// How serious.
    pub severity: Severity,
    /// The panel the finding is about (index into `panels`), or `None` for the dashboard.
    pub panel: Option<usize>,
    /// What to tell the user.
    pub message: String,
}

impl fmt::Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        match self.panel {
            Some(i) => write!(f, "{level}: panel {}: {}", i + 1, self.message),
            None => write!(f, "{level}: {}", self.message),
        }
    }
}

/// Everything found by [`Dashboard::validate`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Report {
    /// The findings, dashboard-level first, then by panel.
    pub issues: Vec<Issue>,
}

impl Report {
    /// Whether there are no errors (warnings are allowed).
    pub fn is_ok(&self) -> bool {
        !self.issues.iter().any(|i| i.severity == Severity::Error)
    }

    /// The errors.
    pub fn errors(&self) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(|i| i.severity == Severity::Error)
    }

    /// The warnings.
    pub fn warnings(&self) -> impl Iterator<Item = &Issue> {
        self.issues
            .iter()
            .filter(|i| i.severity == Severity::Warning)
    }

    fn error(&mut self, panel: Option<usize>, message: impl Into<String>) {
        self.issues.push(Issue {
            severity: Severity::Error,
            panel,
            message: message.into(),
        });
    }

    fn warn(&mut self, panel: Option<usize>, message: impl Into<String>) {
        self.issues.push(Issue {
            severity: Severity::Warning,
            panel,
            message: message.into(),
        });
    }
}

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// The variable names written as `$name` or `${name}` in `text`.
pub(crate) fn variable_refs(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == '$' {
            let braced = bytes.get(i + 1) == Some(&'{');
            let start = i + 1 + usize::from(braced);
            let mut end = start;
            while end < bytes.len() && (bytes[end].is_ascii_alphanumeric() || bytes[end] == '_') {
                end += 1;
            }
            if end > start {
                out.push(bytes[start..end].iter().collect());
            }
            i = end;
        } else {
            i += 1;
        }
    }
    out
}

impl Dashboard {
    /// Checks the dashboard. Without a catalogue only the format is checked; with one, metric
    /// names, aggregations, dimensions and panel kinds are checked against the host's metrics.
    pub fn validate(&self, catalogue: Option<&dyn Catalogue>) -> Report {
        let mut r = Report::default();

        if self.version == 0 || self.version > FORMAT_VERSION {
            r.error(None, format!("unsupported format version {}", self.version));
        }
        if self.name.trim().is_empty() {
            r.error(None, "the dashboard has no name");
        }
        if self.panels.is_empty() {
            r.warn(None, "the dashboard has no panels");
        }
        if self.panels.len() > MAX_PANELS {
            r.error(
                None,
                format!(
                    "{} panels is over the limit of {MAX_PANELS}",
                    self.panels.len()
                ),
            );
        }
        if let (Some(c), Some(range)) = (self.compare, self.range) {
            if c < range {
                r.warn(
                    None,
                    "the comparison period is shorter than the range, so the two overlap",
                );
            }
        }

        let mut seen = HashSet::new();
        for v in &self.variables {
            if !valid_name(&v.name) {
                r.error(None, format!("variable `{}` is not a valid name (letters, digits and _, not starting with a digit)", v.name));
            } else if !seen.insert(v.name.as_str()) {
                r.error(None, format!("variable `{}` is declared twice", v.name));
            }
        }

        let spans = layout::place(&self.panels);
        for (i, p) in self.panels.iter().enumerate() {
            let at = Some(i);
            if p.title.trim().is_empty() {
                r.warn(at, "the panel has no title");
            }
            if p.metric.trim().is_empty() {
                r.error(at, "the panel names no metric");
            }
            if let Some(s) = p.span {
                if s.w == 0 || s.h == 0 {
                    r.error(at, "the span must be at least 1 column wide and 1 row high");
                } else if s.x + s.w > GRID_COLUMNS {
                    r.error(
                        at,
                        format!(
                            "the span reaches column {} but the grid has {GRID_COLUMNS}",
                            s.x + s.w
                        ),
                    );
                }
            }
            for t in &p.thresholds {
                if t.above.is_none() && t.below.is_none() {
                    r.error(at, "a threshold needs `above`, `below` or both");
                }
                if t.above.is_some_and(f64::is_nan) || t.below.is_some_and(f64::is_nan) {
                    r.error(at, "a threshold bound is not a number");
                }
                if let (Some(a), Some(b)) = (t.above, t.below) {
                    if a >= b {
                        r.warn(
                            at,
                            "a threshold with `above` at or past `below` matches nothing",
                        );
                    }
                }
            }
            if let (Some(lo), Some(hi)) = (p.min, p.max) {
                if lo >= hi {
                    r.error(at, "`min` must be below `max`");
                }
            }
            if let (Some(step), Some(range)) = (p.step, p.range.or(self.range)) {
                if step > range {
                    r.warn(
                        at,
                        "the step is longer than the range, so there is one point at most",
                    );
                }
            }

            for value in p.filter.values().chain(std::iter::once(&p.title)) {
                for name in variable_refs(value) {
                    if !self.variables.iter().any(|v| v.name == name) {
                        r.error(
                            at,
                            format!("`${name}` is used but no variable `{name}` is declared"),
                        );
                    }
                }
            }

            if let Some(cat) = catalogue {
                check_against_catalogue(&mut r, at, p, cat);
            }

            for (j, q) in self.panels.iter().enumerate().skip(i + 1) {
                if p.span.is_some() && q.span.is_some() && spans[i].overlaps(&spans[j]) {
                    r.warn(at, format!("overlaps panel {}", j + 1));
                }
            }
        }
        r
    }
}

fn check_against_catalogue(
    r: &mut Report,
    at: Option<usize>,
    p: &crate::spec::Panel,
    cat: &dyn Catalogue,
) {
    let Some(m) = cat.metric(&p.metric) else {
        if !p.metric.trim().is_empty() {
            let all = cat.metrics();
            let near = nearest(&p.metric, all.iter().map(|m| m.name.as_str()));
            match near {
                Some(n) => r.error(
                    at,
                    format!("unknown metric `{}` (did you mean `{n}`?)", p.metric),
                ),
                None => r.error(at, format!("unknown metric `{}`", p.metric)),
            }
        }
        return;
    };
    if let Some(a) = p.agg {
        if !m.aggregations.is_empty() && !m.aggregations.contains(&a) {
            r.error(
                at,
                format!(
                    "`{}` does not support the `{}` aggregation",
                    m.name,
                    a.name()
                ),
            );
        }
    }
    if !m.dimensions.is_empty() || p.group_by.is_some() || !p.filter.is_empty() {
        if let Some(g) = &p.group_by {
            if !m.dimensions.contains(g) {
                r.error(
                    at,
                    format!(
                        "`{}` cannot be grouped by `{g}` (it has: {})",
                        m.name,
                        list(&m.dimensions)
                    ),
                );
            }
        }
        for k in p.filter.keys() {
            if !m.dimensions.contains(k) {
                r.error(
                    at,
                    format!(
                        "`{}` cannot be filtered by `{k}` (it has: {})",
                        m.name,
                        list(&m.dimensions)
                    ),
                );
            }
        }
    }
    if !m.kinds.is_empty() && !m.kinds.contains(&p.kind) {
        r.warn(
            at,
            format!(
                "a `{}` panel is an unusual fit for `{}`",
                p.kind.name(),
                m.name
            ),
        );
    }
    if let (Some(u), Some(mu)) = (&p.unit, &m.unit) {
        if u != mu {
            r.warn(
                at,
                format!("the unit `{u}` differs from the metric's own `{mu}`"),
            );
        }
    }
}

fn list(items: &[String]) -> String {
    if items.is_empty() {
        "no dimensions".to_string()
    } else {
        items.join(", ")
    }
}

/// The candidate closest to `name` by edit distance, if it is close enough to be a typo.
fn nearest<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    candidates
        .map(|c| (distance(name, c), c))
        .filter(|(d, _)| *d <= 2.max(name.len() / 4))
        .min_by_key(|(d, _)| *d)
        .map(|(_, c)| c)
}

fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut prev = row[0];
        row[0] = i;
        for j in 1..=b.len() {
            let cur = row[j];
            row[j] = (row[j] + 1)
                .min(row[j - 1] + 1)
                .min(prev + usize::from(a[i - 1] != b[j - 1]));
            prev = cur;
        }
    }
    row[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalogue::{MetricInfo, StaticCatalogue};
    use crate::spec::{Aggregation, Panel, PanelKind, Span, Threshold, Tone, Variable};

    fn cat() -> StaticCatalogue {
        StaticCatalogue::new([MetricInfo::new("gpu.power_w")
            .unit("W")
            .dimensions(["host", "gpu"])
            .aggregations([Aggregation::Sum, Aggregation::Avg])
            .kinds([PanelKind::Line, PanelKind::StackedArea])])
    }

    fn with(p: Panel) -> Dashboard {
        let mut d = Dashboard::new("d");
        d.panels.push(p);
        d
    }

    fn messages(r: &Report) -> Vec<String> {
        r.issues.iter().map(|i| i.to_string()).collect()
    }

    #[test]
    fn a_clean_dashboard_has_no_findings() {
        let d = with(Panel::new("Power", PanelKind::Line, "gpu.power_w"));
        let r = d.validate(Some(&cat()));
        assert!(r.is_ok() && r.issues.is_empty(), "{:?}", messages(&r));
    }

    #[test]
    fn dashboard_level_findings() {
        let mut d = Dashboard::new("  ");
        d.version = 9;
        d.variables = vec![
            Variable {
                name: "ok".into(),
                kind: None,
                label: None,
                default: None,
            },
            Variable {
                name: "ok".into(),
                kind: None,
                label: None,
                default: None,
            },
            Variable {
                name: "1bad".into(),
                kind: None,
                label: None,
                default: None,
            },
        ];
        let r = d.validate(None);
        let m = messages(&r).join("\n");
        assert!(!r.is_ok());
        assert!(m.contains("unsupported format version 9"), "{m}");
        assert!(m.contains("no name"), "{m}");
        assert!(m.contains("declared twice"), "{m}");
        assert!(m.contains("`1bad` is not a valid name"), "{m}");
        assert!(m.contains("no panels"), "{m}");
    }

    #[test]
    fn spans_thresholds_and_scale_are_checked() {
        let mut p = Panel::new("t", PanelKind::Gauge, "m");
        p.span = Some(Span {
            x: 10,
            y: 0,
            w: 4,
            h: 1,
        });
        p.thresholds = vec![
            Threshold {
                above: None,
                below: None,
                tone: Tone::Info,
            },
            Threshold {
                above: Some(5.0),
                below: Some(1.0),
                tone: Tone::Info,
            },
        ];
        p.min = Some(10.0);
        p.max = Some(1.0);
        let m = messages(&with(p).validate(None)).join("\n");
        assert!(m.contains("reaches column 14"), "{m}");
        assert!(m.contains("needs `above`"), "{m}");
        assert!(m.contains("matches nothing"), "{m}");
        assert!(m.contains("`min` must be below `max`"), "{m}");
    }

    #[test]
    fn variables_must_be_declared() {
        let mut p = Panel::new("t", PanelKind::Line, "m");
        p.filter.insert("host".into(), "$machine".into());
        p.filter.insert("grp".into(), "${group}".into());
        let mut d = with(p);
        d.variables.push(Variable {
            name: "machine".into(),
            kind: None,
            label: None,
            default: None,
        });
        let m = messages(&d.validate(None)).join("\n");
        assert!(!m.contains("`$machine`"), "{m}");
        assert!(
            m.contains("`$group` is used but no variable `group` is declared"),
            "{m}"
        );
    }

    #[test]
    fn overlapping_explicit_panels_warn() {
        let mut a = Panel::new("a", PanelKind::Line, "m");
        let mut b = a.clone();
        a.span = Some(Span {
            x: 0,
            y: 0,
            w: 6,
            h: 3,
        });
        b.span = Some(Span {
            x: 3,
            y: 1,
            w: 6,
            h: 3,
        });
        let mut d = with(a);
        d.panels.push(b);
        let r = d.validate(None);
        assert!(r.is_ok());
        assert!(messages(&r).iter().any(|m| m.contains("overlaps panel 2")));
    }

    #[test]
    fn the_catalogue_judges_metrics_aggregations_dimensions_kinds_and_units() {
        let mut p = Panel::new("t", PanelKind::Heatmap, "gpu.power_w");
        p.agg = Some(Aggregation::P95);
        p.group_by = Some("region".into());
        p.filter.insert("zone".into(), "a".into());
        p.unit = Some("kW".into());
        let m = messages(&with(p).validate(Some(&cat()))).join("\n");
        assert!(m.contains("does not support the"), "{m}");
        assert!(
            m.contains("cannot be grouped by `region` (it has: host, gpu)"),
            "{m}"
        );
        assert!(m.contains("cannot be filtered by `zone`"), "{m}");
        assert!(m.contains("unusual fit"), "{m}");
        assert!(m.contains("differs from the metric's own `W`"), "{m}");
    }

    #[test]
    fn unknown_metrics_suggest_a_near_name() {
        let d = with(Panel::new("t", PanelKind::Line, "gpu.powr_w"));
        let m = messages(&d.validate(Some(&cat()))).join("\n");
        assert!(m.contains("did you mean `gpu.power_w`"), "{m}");
        let d = with(Panel::new("t", PanelKind::Line, "zzz"));
        let m = messages(&d.validate(Some(&cat()))).join("\n");
        assert!(
            m.contains("unknown metric `zzz`") && !m.contains("did you mean"),
            "{m}"
        );
    }

    #[test]
    fn variable_refs_reads_both_spellings() {
        assert_eq!(variable_refs("a $x b ${y_1} $ $"), vec!["x", "y_1"]);
        assert!(variable_refs("no vars").is_empty());
    }
}
