//! Ratchet: geometry comes from the theme (`t.spacing`, `t.shape`, `t.sizes()`), not from
//! `px(<number>)` written into a component. Each file may keep the literals it has today
//! (listed below); a file that gains one fails this test, and one that loses some should lower
//! its number. New files start at zero.
//!
//! Only code before a file's `#[cfg(test)]` counts.

use std::{fs, path::Path};

/// `(file under src/, number of `px(<digit>` lines allowed)`.
const BASELINE: &[(&str, usize)] = &[
    ("content.rs", 1),
    ("controls/bars.rs", 1),
    ("controls/slider.rs", 9),
    ("datatable.rs", 5),
    ("expressive.rs", 1),
    ("extras.rs", 4),
    ("feedback.rs", 5),
    ("forms.rs", 1),
    ("icons.rs", 1),
    ("lib.rs", 1),
    ("markdown.rs", 2),
    ("nav.rs", 1),
    ("navigation.rs", 1),
    ("shell.rs", 1),
    ("text_input.rs", 1),
    ("time/tap.rs", 3),
];

/// Lines of `src` (before its test module) holding a `px(<number>` literal.
fn literals(src: &str) -> usize {
    src.lines()
        .take_while(|l| !l.contains("#[cfg(test)]"))
        .filter(|l| {
            let l = l.trim_start();
            !l.starts_with("//") && has_literal(l)
        })
        .count()
}

fn has_literal(line: &str) -> bool {
    let mut rest = line;
    while let Some(i) = rest.find("px(") {
        let before_ok = i == 0 || !rest[..i].ends_with(|c: char| c.is_alphanumeric() || c == '_');
        let after = &rest[i + 3..];
        if before_ok && after.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
        rest = after;
    }
    false
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, usize)>) {
    let mut entries: Vec<_> = fs::read_dir(dir).unwrap().flatten().collect();
    entries.sort_by_key(|e| e.path());
    for e in entries {
        let p = e.path();
        if p.is_dir() {
            walk(&p, root, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            let n = literals(&fs::read_to_string(&p).unwrap());
            if n > 0 {
                let rel = p
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push((rel, n));
            }
        }
    }
}

#[test]
fn components_gain_no_new_spacing_literals() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut found = Vec::new();
    walk(&root, &root, &mut found);
    let mut bad = Vec::new();
    for (file, n) in &found {
        let allowed = BASELINE
            .iter()
            .find(|(f, _)| f == file)
            .map_or(0, |(_, n)| *n);
        if *n > allowed {
            bad.push(format!("{file}: {n} px(<number>) lines, {allowed} allowed"));
        }
    }
    assert!(
        bad.is_empty(),
        "take the size from the theme (spacing, shape, sizes) instead:\n{}",
        bad.join("\n")
    );
}

#[test]
fn the_detector_sees_a_literal_and_ignores_a_theme_value() {
    assert!(has_literal("let w = px(12.0);"));
    assert!(has_literal("a.w(px(4.0)).h(px(t.spacing.md))"));
    assert!(!has_literal("a.w(px(t.spacing.md))"));
    assert!(!has_literal("let x = tpx(3);"));
    assert_eq!(literals("px(1.0)\n#[cfg(test)]\npx(2.0)"), 1);
}
