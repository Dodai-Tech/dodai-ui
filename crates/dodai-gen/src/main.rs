//! `cargo run -p dodai-gen` writes the generated files; `-- --check` fails when one is stale.

use dodai_gen::{format, render, TARGETS};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let check = std::env::args().any(|a| a == "--check");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let specs = dodai_core::component_spec::all();
    let mut stale = Vec::new();
    for t in TARGETS {
        let text = match format(&render(&specs, t)) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{}: {e}", t.name);
                return ExitCode::FAILURE;
            }
        };
        let path = root.join(t.path);
        if check {
            if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                stale.push(t.path);
            }
        } else if let Err(e) = std::fs::write(&path, text) {
            eprintln!("{}: {e}", t.path);
            return ExitCode::FAILURE;
        } else {
            println!("wrote {}", t.path);
        }
    }
    if stale.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "out of date, run `cargo run -p dodai-gen`: {}",
            stale.join(", ")
        );
        ExitCode::FAILURE
    }
}
