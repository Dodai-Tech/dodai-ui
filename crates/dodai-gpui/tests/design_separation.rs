//! Components that only one design system ships stay in that design's module: no shared
//! module refers to them, and they are compiled only with that design's feature.

use std::fs;
use std::path::Path;

/// `(design feature, module file, exported type names)`.
const OWNED: &[(&str, &str, &[&str])] = &[
    (
        "material",
        "expressive.rs",
        &["ButtonGroup", "SplitButton", "LoadingIndicator"],
    ),
    (
        "material",
        "overscroll.rs",
        &["Overscroll", "OverscrollFeel"],
    ),
    ("cyberpunk", "scanlines.rs", &["Scanlines"]),
    ("dodai", "frosted.rs", &["FrostedScene"]),
    ("dodai", "page_grid.rs", &["PageGrid"]),
];

fn words(s: &str) -> impl Iterator<Item = &str> {
    s.split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
}

#[test]
fn shared_modules_do_not_use_design_components() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let owned_files: Vec<&str> = OWNED.iter().map(|o| o.1).collect();
    let mut leaks = Vec::new();
    for entry in fs::read_dir(&src).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".rs")
            || name == "lib.rs"
            || name == "icon_names.rs"
            || owned_files.contains(&name.as_str())
        {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        // Doc and line comments may mention them.
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
            .replace("Texture::Scanlines", "");
        for (feature, file, names) in OWNED {
            for w in words(&code) {
                if names.contains(&w) {
                    leaks.push(format!("{name} uses {w} (owned by {feature}: {file})"));
                }
            }
        }
    }
    assert!(
        leaks.is_empty(),
        "design components leak into shared modules:\n{}",
        leaks.join("\n")
    );
}

#[test]
fn design_modules_are_gated_by_their_feature() {
    let lib = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs")).unwrap();
    for (feature, file, _) in OWNED {
        let module = file.trim_end_matches(".rs");
        let lines: Vec<&str> = lib.lines().collect();
        let decl = lines
            .iter()
            .position(|l| l.trim() == format!("pub mod {module};"))
            .unwrap_or_else(|| panic!("{module} is not declared"));
        let attrs: String = lines[..decl]
            .iter()
            .rev()
            .take_while(|l| l.trim().starts_with("#[cfg"))
            .copied()
            .collect();
        assert!(
            attrs.contains(&format!("feature = \"{feature}\"")),
            "{module} must be gated by `{feature}`"
        );
    }
}
