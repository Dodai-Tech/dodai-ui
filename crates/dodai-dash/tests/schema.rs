//! The committed JSON Schema matches the types.
#![cfg(feature = "schema")]

use dodai_dash::{schema, PanelKind};

#[test]
fn the_schema_file_is_current() {
    let committed = include_str!("../schema/dashboard.schema.json");
    assert_eq!(
        committed,
        schema(),
        "regenerate with: cargo run -q -p dodai-dash --features schema --example schema > crates/dodai-dash/schema/dashboard.schema.json"
    );
}

#[test]
fn the_schema_lists_every_panel_kind() {
    let v: serde_json::Value = serde_json::from_str(&schema()).unwrap();
    let kinds = v["$defs"]["PanelKind"]["oneOf"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|k| k["const"].as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    for k in PanelKind::ALL {
        assert!(
            kinds.iter().any(|n| n == k.name()),
            "{} missing from {kinds:?}",
            k.name()
        );
    }
}
