//! A JSON Schema of the dashboard format.

use schemars::{schema_for, JsonSchema};
use serde_json::{json, Value};

use crate::spec::{Bundle, Dashboard};

/// A JSON Schema for a dashboard file: either one [`Dashboard`] or a [`Bundle`]. The same schema
/// describes the TOML form, since both go through one model. Editors can use it for validation
/// and completion.
pub fn schema() -> String {
    let dashboard = to_value(schema_for!(Dashboard));
    let bundle = to_value(schema_for!(Bundle));
    let mut defs = serde_json::Map::new();
    for part in [&dashboard, &bundle] {
        if let Some(Value::Object(d)) = part.get("$defs") {
            defs.extend(d.clone());
        }
    }
    let strip = |mut v: Value| {
        if let Value::Object(o) = &mut v {
            o.remove("$defs");
            o.remove("$schema");
            o.remove("title");
        }
        v
    };
    let doc = json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Dodai dashboard",
        "description": "One dashboard, or a bundle of several.",
        "oneOf": [strip(dashboard), strip(bundle)],
        "$defs": defs,
    });
    let mut out = serde_json::to_string_pretty(&doc).unwrap_or_default();
    out.push('\n');
    out
}

fn to_value<T: JsonSchema + serde::Serialize>(s: T) -> Value {
    serde_json::to_value(s).unwrap_or(Value::Null)
}
