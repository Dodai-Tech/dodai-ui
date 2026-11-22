//! Converts a dashboard file between TOML and JSON: `convert <file> [toml|json]`.
//! Without the second argument the other format is used.

use dodai_ui_dash::{convert, import, Format};

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: convert <file> [toml|json]");
        std::process::exit(2);
    };
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(1);
    });
    let hint = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .and_then(Format::from_extension);
    let target = match args.next().as_deref() {
        Some(name) => Format::from_extension(name).unwrap_or_else(|| {
            eprintln!("unknown format `{name}` (use toml or json)");
            std::process::exit(2);
        }),
        None => match hint.unwrap_or_else(|| Format::detect(&text)) {
            Format::Toml => Format::Json,
            Format::Json => Format::Toml,
        },
    };
    match import(&text, hint) {
        Ok(parsed) => {
            for f in &parsed.unknown_fields {
                eprintln!("warning: unknown field `{f}` is dropped");
            }
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
    match convert(&text, hint, target) {
        Ok(out) => print!("{out}"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
