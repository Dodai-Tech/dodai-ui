//! The shipped samples stay valid, and TOML and JSON stay equivalent.

use dodai_dash::{
    convert, import, Aggregation, Catalogue, Dashboard, Document, Format, MetricInfo,
    StaticCatalogue,
};

const OVERVIEW_TOML: &str = include_str!("../samples/overview.toml");
const OVERVIEW_JSON: &str = include_str!("../samples/overview.json");
const BUNDLE_TOML: &str = include_str!("../samples/bundle.toml");

fn catalogue() -> StaticCatalogue {
    use Aggregation::{Avg, Delta, Max, Sum};
    StaticCatalogue::new([
        MetricInfo::new("gpu.power_w")
            .unit("W")
            .dimensions(["host", "gpu", "group"])
            .aggregations([Sum, Avg, Max]),
        MetricInfo::new("gpu.temp_c")
            .unit("C")
            .dimensions(["host", "gpu", "group"])
            .aggregations([Max, Avg]),
        MetricInfo::new("serve.tokens_per_s")
            .dimensions(["host", "model", "group"])
            .aggregations([Sum, Avg]),
        MetricInfo::new("energy.wh")
            .unit("Wh")
            .dimensions(["host", "group"])
            .aggregations([Delta, Sum]),
    ])
}

#[test]
fn the_overview_sample_is_clean() {
    let parsed = Dashboard::import(OVERVIEW_TOML, Some(Format::Toml)).unwrap();
    assert!(
        parsed.unknown_fields.is_empty(),
        "{:?}",
        parsed.unknown_fields
    );
    let report = parsed.value.validate(Some(&catalogue()));
    assert!(
        report.issues.is_empty(),
        "{:?}",
        report
            .issues
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert_eq!(parsed.value.panels.len(), 4);
    assert_eq!(parsed.value.fixed_filters().len(), 0);
}

#[test]
fn the_json_sample_is_the_toml_sample() {
    let a = Dashboard::import(OVERVIEW_TOML, None).unwrap().value;
    let b = Dashboard::import(OVERVIEW_JSON, None).unwrap().value;
    assert_eq!(a, b);
    assert_eq!(
        convert(OVERVIEW_TOML, None, Format::Json).unwrap(),
        OVERVIEW_JSON
    );
}

#[test]
fn the_bundle_sample_holds_two_valid_dashboards() {
    let parsed = import(BUNDLE_TOML, None).unwrap();
    assert!(
        parsed.unknown_fields.is_empty(),
        "{:?}",
        parsed.unknown_fields
    );
    let Document::Bundle(b) = parsed.value else {
        panic!("expected a bundle")
    };
    assert_eq!(b.dashboards.len(), 2);
    for d in &b.dashboards {
        let r = d.validate(Some(&catalogue() as &dyn Catalogue));
        assert!(r.is_ok(), "{}: {:?}", d.name, r.issues);
    }
}
