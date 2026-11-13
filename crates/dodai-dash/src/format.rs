//! Import and export: TOML and JSON, one model, no loss between them.

use std::fmt;

use serde::de::{DeserializeOwned, IgnoredAny};
use serde::{Deserialize, Serialize};

use crate::spec::{Bundle, Dashboard, FORMAT_VERSION};

/// The largest text [`import`] accepts, to keep a hostile file from costing memory.
pub const MAX_IMPORT_BYTES: usize = 1 << 20;

/// A text format for dashboards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// TOML: friendly to edit by hand, keeps nothing but the data.
    Toml,
    /// JSON: friendly to tools and scripts.
    Json,
}

impl Format {
    /// The format for a file extension (`toml`, `json`), without the dot, any case.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_ascii_lowercase().as_str() {
            "toml" => Some(Self::Toml),
            "json" => Some(Self::Json),
            _ => None,
        }
    }

    /// Guesses from the content: JSON starts with `{`, anything else is read as TOML.
    pub fn detect(text: &str) -> Self {
        if text.trim_start().starts_with('{') {
            Self::Json
        } else {
            Self::Toml
        }
    }

    /// The file extension, without the dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Toml => "toml",
            Self::Json => "json",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Toml => "TOML",
            Self::Json => "JSON",
        })
    }
}

/// Why a file could not be imported or a dashboard exported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The text is larger than [`MAX_IMPORT_BYTES`].
    TooLarge {
        /// Size of the text.
        bytes: usize,
    },
    /// The text is not valid in the format, or does not match the dashboard shape.
    Syntax {
        /// The format that was tried.
        format: Format,
        /// What went wrong, with a position when the parser gave one.
        message: String,
    },
    /// The file declares a version this crate does not understand.
    Version {
        /// The version in the file.
        found: u32,
    },
    /// The dashboard could not be written.
    Write {
        /// The format that was tried.
        format: Format,
        /// What went wrong.
        message: String,
    },
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooLarge { bytes } => {
                write!(
                    f,
                    "{bytes} bytes is over the {MAX_IMPORT_BYTES}-byte import limit"
                )
            }
            Self::Syntax { format, message } => {
                write!(f, "not a valid {format} dashboard: {message}")
            }
            Self::Version { found } => write!(
                f,
                "format version {found} is not supported (this build reads 1 to {FORMAT_VERSION})"
            ),
            Self::Write { format, message } => write!(f, "could not write {format}: {message}"),
        }
    }
}

impl std::error::Error for FormatError {}

/// A parsed value and what the parser noticed but let pass.
#[derive(Debug, Clone, PartialEq)]
pub struct Parsed<T> {
    /// The value.
    pub value: T,
    /// Fields in the file that this version does not know, by path (for example
    /// `panels[2].colour`); they are dropped on re-export.
    pub unknown_fields: Vec<String>,
}

/// What a file holds: one dashboard or a bundle of several.
#[derive(Debug, Clone, PartialEq)]
pub enum Document {
    /// A single dashboard.
    Dashboard(Dashboard),
    /// Several dashboards.
    Bundle(Bundle),
}

impl Document {
    /// The dashboards in the file, in order.
    pub fn into_dashboards(self) -> Vec<Dashboard> {
        match self {
            Self::Dashboard(d) => vec![d],
            Self::Bundle(b) => b.dashboards,
        }
    }
}

fn parse<T: DeserializeOwned>(text: &str, format: Format) -> Result<Parsed<T>, FormatError> {
    let syntax = |message: String| FormatError::Syntax { format, message };
    let mut unknown = Vec::new();
    let value = match format {
        Format::Json => {
            let mut de = serde_json::Deserializer::from_str(text);
            let value = serde_ignored::deserialize(&mut de, |p| unknown.push(p.to_string()))
                .map_err(|e| syntax(e.to_string()))?;
            de.end().map_err(|e| syntax(e.to_string()))?;
            value
        }
        Format::Toml => {
            let de = toml::de::Deserializer::parse(text).map_err(|e| syntax(e.to_string()))?;
            serde_ignored::deserialize(de, |p| unknown.push(p.to_string()))
                .map_err(|e| syntax(e.to_string()))?
        }
    };
    Ok(Parsed {
        value,
        unknown_fields: unknown,
    })
}

#[derive(Deserialize)]
struct Probe {
    #[serde(default)]
    dashboards: Option<IgnoredAny>,
}

fn check_version(found: u32) -> Result<(), FormatError> {
    if found == 0 || found > FORMAT_VERSION {
        Err(FormatError::Version { found })
    } else {
        Ok(())
    }
}

/// Reads a dashboard or a bundle. `hint` is the format when the caller knows it (from a file
/// extension, say); otherwise it is detected from the content. The text must be at most
/// [`MAX_IMPORT_BYTES`] and carry a supported version. Nothing is run and nothing is looked up:
/// the result is data only. Validate it afterwards with [`Dashboard::validate`].
pub fn import(text: &str, hint: Option<Format>) -> Result<Parsed<Document>, FormatError> {
    if text.len() > MAX_IMPORT_BYTES {
        return Err(FormatError::TooLarge { bytes: text.len() });
    }
    let format = hint.unwrap_or_else(|| Format::detect(text));
    let probe: Parsed<Probe> = parse(text, format)?;
    if probe.value.dashboards.is_some() {
        let b = parse::<Bundle>(text, format)?;
        check_version(b.value.version)?;
        for d in &b.value.dashboards {
            check_version(d.version)?;
        }
        Ok(Parsed {
            value: Document::Bundle(b.value),
            unknown_fields: b.unknown_fields,
        })
    } else {
        let d = parse::<Dashboard>(text, format)?;
        check_version(d.value.version)?;
        Ok(Parsed {
            value: Document::Dashboard(d.value),
            unknown_fields: d.unknown_fields,
        })
    }
}

fn write<T: Serialize>(value: &T, format: Format) -> Result<String, FormatError> {
    let fail = |message: String| FormatError::Write { format, message };
    match format {
        Format::Toml => toml::to_string_pretty(value).map_err(|e| fail(e.to_string())),
        Format::Json => {
            let mut s = serde_json::to_string_pretty(value).map_err(|e| fail(e.to_string()))?;
            s.push('\n');
            Ok(s)
        }
    }
}

/// Converts a file between formats; `from` is detected when `None`. Unknown fields are dropped.
pub fn convert(text: &str, from: Option<Format>, to: Format) -> Result<String, FormatError> {
    match import(text, from)?.value {
        Document::Dashboard(d) => d.export(to),
        Document::Bundle(b) => b.export(to),
    }
}

impl Dashboard {
    /// Writes the dashboard as `format`.
    pub fn export(&self, format: Format) -> Result<String, FormatError> {
        write(self, format)
    }

    /// Reads a single dashboard (not a bundle); see [`import`] for the rules.
    pub fn import(text: &str, hint: Option<Format>) -> Result<Parsed<Dashboard>, FormatError> {
        let parsed = import(text, hint)?;
        match parsed.value {
            Document::Dashboard(d) => Ok(Parsed {
                value: d,
                unknown_fields: parsed.unknown_fields,
            }),
            Document::Bundle(_) => Err(FormatError::Syntax {
                format: hint.unwrap_or_else(|| Format::detect(text)),
                message: "this file is a bundle of dashboards, not one dashboard".into(),
            }),
        }
    }
}

impl Bundle {
    /// A bundle of `dashboards`.
    pub fn new(dashboards: Vec<Dashboard>) -> Self {
        Self {
            version: FORMAT_VERSION,
            dashboards,
        }
    }

    /// Writes the bundle as `format`.
    pub fn export(&self, format: Format) -> Result<String, FormatError> {
        write(self, format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spec::{Panel, PanelKind};

    fn sample() -> Dashboard {
        let mut d = Dashboard::new("Overview");
        d.range = Some("1h".parse().unwrap());
        let mut p = Panel::new("Power", PanelKind::StackedArea, "gpu.power_w");
        p.filter.insert("group".into(), "rigs".into());
        d.panels.push(p);
        d
    }

    #[test]
    fn extension_and_content_pick_a_format() {
        assert_eq!(Format::from_extension("TOML"), Some(Format::Toml));
        assert_eq!(Format::from_extension("json"), Some(Format::Json));
        assert_eq!(Format::from_extension("yaml"), None);
        assert_eq!(Format::detect("  {\"a\":1}"), Format::Json);
        assert_eq!(Format::detect("name = \"x\""), Format::Toml);
        assert_eq!(Format::detect("[[panels]]"), Format::Toml);
        assert_eq!(Format::Toml.extension(), "toml");
    }

    #[test]
    fn a_dashboard_survives_both_formats() {
        let d = sample();
        for f in [Format::Toml, Format::Json] {
            let text = d.export(f).unwrap();
            let back = Dashboard::import(&text, None).unwrap();
            assert_eq!(back.value, d, "{f}");
            assert!(
                back.unknown_fields.is_empty(),
                "{f}: {:?}",
                back.unknown_fields
            );
        }
    }

    #[test]
    fn convert_goes_there_and_back_without_loss() {
        let d = sample();
        let toml = d.export(Format::Toml).unwrap();
        let json = convert(&toml, None, Format::Json).unwrap();
        let toml2 = convert(&json, None, Format::Toml).unwrap();
        assert_eq!(Dashboard::import(&toml2, None).unwrap().value, d);
    }

    #[test]
    fn the_panel_table_name_may_be_singular_in_toml() {
        let text = "name = \"x\"\n[[panel]]\ntitle = \"t\"\nkind = \"line\"\nmetric = \"m\"\n";
        let d = Dashboard::import(text, None).unwrap().value;
        assert_eq!(d.panels.len(), 1);
    }

    #[test]
    fn bundles_hold_several_and_are_told_apart() {
        let b = Bundle::new(vec![sample(), Dashboard::new("Two")]);
        for f in [Format::Toml, Format::Json] {
            let text = b.export(f).unwrap();
            match import(&text, None).unwrap().value {
                Document::Bundle(back) => assert_eq!(back, b, "{f}"),
                other => panic!("{f}: expected a bundle, got {other:?}"),
            }
            assert!(
                Dashboard::import(&text, None).is_err(),
                "{f}: a bundle is not one dashboard"
            );
        }
        assert_eq!(
            import(&b.export(Format::Json).unwrap(), None)
                .unwrap()
                .value
                .into_dashboards()
                .len(),
            2
        );
    }

    #[test]
    fn unknown_fields_are_reported_not_fatal() {
        let text = "name = \"x\"\ncolour = \"red\"\n[[panels]]\ntitle = \"t\"\nkind = \"line\"\nmetric = \"m\"\nglow = 3\n";
        let p = Dashboard::import(text, None).unwrap();
        assert_eq!(p.value.name, "x");
        assert!(
            p.unknown_fields.iter().any(|f| f == "colour"),
            "{:?}",
            p.unknown_fields
        );
        assert!(
            p.unknown_fields.iter().any(|f| f.contains("glow")),
            "{:?}",
            p.unknown_fields
        );
    }

    #[test]
    fn broken_text_gives_a_syntax_error_with_the_format() {
        let e = import("name = ", None).unwrap_err();
        assert!(
            matches!(
                e,
                FormatError::Syntax {
                    format: Format::Toml,
                    ..
                }
            ),
            "{e}"
        );
        let e = import("{\"name\": }", None).unwrap_err();
        assert!(
            matches!(
                e,
                FormatError::Syntax {
                    format: Format::Json,
                    ..
                }
            ),
            "{e}"
        );
        let e = import("{\"name\":\"x\"} trailing", None).unwrap_err();
        assert!(
            matches!(
                e,
                FormatError::Syntax {
                    format: Format::Json,
                    ..
                }
            ),
            "{e}"
        );
        let e = import(
            "{\"name\":\"x\",\"panels\":[{\"title\":\"t\",\"kind\":\"pie\",\"metric\":\"m\"}]}",
            None,
        )
        .unwrap_err();
        assert!(
            e.to_string().contains("pie"),
            "an unknown panel kind names itself: {e}"
        );
    }

    #[test]
    fn future_and_zero_versions_are_refused() {
        let e = import("version = 99\nname = \"x\"\n", None).unwrap_err();
        assert_eq!(e, FormatError::Version { found: 99 });
        let e = import("version = 0\nname = \"x\"\n", None).unwrap_err();
        assert_eq!(e, FormatError::Version { found: 0 });
        let e = import(
            "{\"version\":1,\"dashboards\":[{\"version\":7,\"name\":\"x\"}]}",
            None,
        )
        .unwrap_err();
        assert_eq!(e, FormatError::Version { found: 7 });
    }

    #[test]
    fn a_missing_version_means_the_current_one() {
        assert_eq!(
            import("name = \"x\"\n", None)
                .unwrap()
                .value
                .into_dashboards()[0]
                .version,
            FORMAT_VERSION
        );
    }

    #[test]
    fn oversize_text_is_refused_before_parsing() {
        let big = "#".repeat(MAX_IMPORT_BYTES + 1);
        assert!(matches!(
            import(&big, None),
            Err(FormatError::TooLarge { .. })
        ));
    }

    #[test]
    fn a_wrong_hint_fails_cleanly() {
        let json = sample().export(Format::Json).unwrap();
        assert!(import(&json, Some(Format::Toml)).is_err());
    }
}
