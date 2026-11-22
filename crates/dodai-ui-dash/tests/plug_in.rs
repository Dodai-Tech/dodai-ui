//! The "Plugging it in" example of the README, as a test: a host with its own catalogue and data.

use std::collections::BTreeMap;

use dodai_ui_dash::{
    resolve, Aggregation, Dashboard, MetricInfo, PanelKind, Point, Query, Series, SeriesSource,
    SourceError, StaticCatalogue, Tone, Variables,
};

struct Data;

impl SeriesSource for Data {
    fn query(&self, q: &Query) -> Result<Vec<Series>, SourceError> {
        let host = q.filter.get("group").cloned().unwrap_or_default();
        let labels = BTreeMap::from([("host".to_string(), format!("{host}-1"))]);
        Ok(vec![Series::new(
            labels,
            vec![
                Point {
                    t: q.from,
                    v: 1200.0,
                },
                Point { t: q.to, v: 3500.0 },
            ],
        )])
    }
}

#[test]
fn validate_resolve_and_read_tones() {
    let catalogue = StaticCatalogue::new([MetricInfo::new("gpu.power_w")
        .describe("Board power")
        .unit("W")
        .dimensions(["host", "gpu", "group"])
        .aggregations([Aggregation::Sum, Aggregation::Avg, Aggregation::Max])
        .kinds([PanelKind::Line, PanelKind::StackedArea, PanelKind::Stat])]);
    let dashboard = Dashboard::import(include_str!("../samples/overview.toml"), None)
        .unwrap()
        .value;

    let overrides: Variables = [("group".to_string(), "rigs".to_string())].into();
    let view = resolve(&dashboard, &overrides, &Data, 1_000_000_000);

    let power = &view.panels[0];
    assert_eq!(power.title, "Power of rigs");
    assert!(power.error.is_none());
    assert_eq!(power.latest(), vec![("rigs-1".to_string(), 3500.0)]);
    let tone = power.tone(3500.0).expect("over the 3000 W threshold");
    assert_eq!(tone, Tone::Warning);
    assert_eq!(
        dodai_ui_core::Tone::from(tone),
        dodai_ui_core::Tone::Warning
    );

    // The sample uses metrics this small catalogue does not have: validation says so.
    let report = dashboard.validate(Some(&catalogue));
    assert!(!report.is_ok());
    assert!(report
        .errors()
        .any(|i| i.message.contains("unknown metric `gpu.temp_c`")));
}
