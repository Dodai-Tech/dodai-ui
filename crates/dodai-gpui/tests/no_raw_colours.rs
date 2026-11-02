//! Regression guard: components take their colours from the theme, so a new theme reaches
//! every widget. Raw colour literals in the sources of `dodai-gpui`, `dodai-gpui-charts` and
//! `dodai-gpui-instruments` (outside `#[cfg(test)]` code) fail this test
//! unless they are on the allow-list below, with the reason they are theme-neutral.
//!
//! `hsla(t.palette.x)` is fine: that is the converter from a theme colour, not a literal.

use std::{fs, path::Path};

/// `(file under a crate's src, text on the line, why a literal is right here)`.
const ALLOWED: &[(&str, &str, &str)] = &[
    (
        "dodai-gpui/src/nav.rs",
        "gpui::transparent_black()",
        "no fill",
    ),
    (
        "dodai-gpui-charts/src/common.rs",
        "gpui::transparent_black()",
        "text shaped only to measure it, never painted",
    ),
    (
        "dodai-gpui-charts/src/grid.rs",
        "Color::rgba(r, gr, b, 255)",
        "converts a gradient sample (data), not a design colour",
    ),
    (
        "dodai-gpui-instruments/src/gradient.rs",
        "Color::rgba(0, 0, 0, 0)",
        "a ramp without stops has no colour at all (fully transparent), not a design colour",
    ),
];

/// Does `line` hold a raw colour literal? (`hsla(` followed by a digit is a literal; followed by
/// anything else it is the theme-colour converter.)
fn literal(line: &str) -> bool {
    let l = line.trim_start();
    if l.starts_with("//") {
        return false;
    }
    let mut rest = l;
    while let Some(i) = rest.find("hsla(") {
        let after = &rest[i + 5..];
        let before_ok = i == 0 || !rest[..i].ends_with(|c: char| c.is_alphanumeric() || c == '_');
        let gpui_path = rest[..i].ends_with("gpui::");
        if (before_ok || gpui_path) && after.trim_start().starts_with(|c: char| c.is_ascii_digit())
        {
            return true;
        }
        rest = after;
    }
    let hex = l
        .as_bytes()
        .windows(8)
        .any(|w| w[0] == b'0' && w[1] == b'x' && w[2..].iter().all(|b| b.is_ascii_hexdigit()));
    hex || l.contains("white()")
        || l.contains("black()")
        || l.contains("rgba(")
        || l.contains("rgb(")
        || l.contains("opaque_grey")
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn components_use_theme_colours_not_literals() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut files = Vec::new();
    for c in ["dodai-gpui", "dodai-gpui-charts", "dodai-gpui-instruments"] {
        rust_files(&crates.join(c).join("src"), &mut files);
    }
    assert!(files.len() > 100, "found only {} files", files.len());
    let mut bad = Vec::new();
    let mut used = vec![false; ALLOWED.len()];
    for path in files {
        let name = path
            .strip_prefix(crates)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if name.ends_with("/testing.rs") {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap();
        let code = text.split("#[cfg(test)]").next().unwrap();
        for (n, line) in code.lines().enumerate() {
            if !literal(line) {
                continue;
            }
            match ALLOWED
                .iter()
                .position(|(f, frag, _)| *f == name && line.contains(frag))
            {
                Some(i) => used[i] = true,
                None => bad.push(format!("{name}:{}: {}", n + 1, line.trim())),
            }
        }
    }
    assert!(
        bad.is_empty(),
        "raw colour literals (use a theme token, or add to ALLOWED with a reason):\n{}",
        bad.join("\n")
    );
    for (i, u) in used.iter().enumerate() {
        assert!(*u, "stale allow-list entry: {:?}", ALLOWED[i]);
    }
}

#[test]
fn the_detector_catches_literals() {
    assert!(literal("let c = hsla(0.5, 0.5, 0.5, 1.0);"));
    assert!(literal("x.bg(gpui::hsla(0.0, 0.0, 0.0, 0.3))"));
    assert!(literal("let c = gpui::white();"));
    assert!(literal("rgb(0xff00aa)"));
    assert!(literal("let c = 0xff00aa;"));
    assert!(!literal("let c = hsla(t.palette.text);"));
    assert!(!literal("// hsla(0.0, 0.0, 0.0, 1.0)"));
    assert!(!literal("fn hsla(c: Color) -> Hsla {"));
}
