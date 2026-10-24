//! The **implementation** behind each showcase section: the source of the `dodai-gpui`
//! components a section demonstrates, read back from the crate's own files at compile time.
//!
//! Components are detected from the section's own source (`Name::new(` for every type that
//! `dodai-gpui` defines), and a type's code is its definition plus every `impl` block for it.

/// `(file name, source)` of every component file of `dodai-gpui`.
const FILES: &[(&str, &str)] = &[
    ("button.rs", include_str!("../../src/button.rs")),
    ("carousel.rs", include_str!("../../src/carousel.rs")),
    (
        "charts_distribution.rs",
        include_str!("../../src/charts_distribution.rs"),
    ),
    ("charts_flow.rs", include_str!("../../src/charts_flow.rs")),
    (
        "charts_market.rs",
        include_str!("../../src/charts_market.rs"),
    ),
    (
        "charts_part_whole.rs",
        include_str!("../../src/charts_part_whole.rs"),
    ),
    ("charts_trend.rs", include_str!("../../src/charts_trend.rs")),
    ("charts.rs", include_str!("../../src/charts.rs")),
    ("charts_bar.rs", include_str!("../../src/charts_bar.rs")),
    ("charts_line.rs", include_str!("../../src/charts_line.rs")),
    ("charts_spark.rs", include_str!("../../src/charts_spark.rs")),
    ("charts_pie.rs", include_str!("../../src/charts_pie.rs")),
    (
        "charts_radial.rs",
        include_str!("../../src/charts_radial.rs"),
    ),
    ("charts_grid.rs", include_str!("../../src/charts_grid.rs")),
    ("command.rs", include_str!("../../src/command.rs")),
    ("content.rs", include_str!("../../src/content.rs")),
    ("markdown.rs", include_str!("../../src/markdown.rs")),
    (
        "controls/bars.rs",
        include_str!("../../src/controls/bars.rs"),
    ),
    (
        "controls/choice.rs",
        include_str!("../../src/controls/choice.rs"),
    ),
    (
        "controls/slider.rs",
        include_str!("../../src/controls/slider.rs"),
    ),
    ("count.rs", include_str!("../../src/count.rs")),
    (
        "instrument_charts.rs",
        include_str!("../../src/instrument_charts.rs"),
    ),
    ("data.rs", include_str!("../../src/data.rs")),
    ("datatable.rs", include_str!("../../src/datatable.rs")),
    ("display.rs", include_str!("../../src/display.rs")),
    ("extras.rs", include_str!("../../src/extras.rs")),
    ("feedback.rs", include_str!("../../src/feedback.rs")),
    ("forms.rs", include_str!("../../src/forms.rs")),
    ("gauges.rs", include_str!("../../src/gauges.rs")),
    ("icons.rs", include_str!("../../src/icons.rs")),
    ("interact.rs", include_str!("../../src/interact.rs")),
    ("knob.rs", include_str!("../../src/knob.rs")),
    ("layout.rs", include_str!("../../src/layout.rs")),
    ("lazy.rs", include_str!("../../src/lazy.rs")),
    ("list.rs", include_str!("../../src/list.rs")),
    ("logic.rs", include_str!("../../src/logic.rs")),
    ("expressive.rs", include_str!("../../src/expressive.rs")),
    ("menu.rs", include_str!("../../src/menu.rs")),
    ("motion.rs", include_str!("../../src/motion.rs")),
    ("nav.rs", include_str!("../../src/nav.rs")),
    ("navigation.rs", include_str!("../../src/navigation.rs")),
    ("overlay.rs", include_str!("../../src/overlay.rs")),
    ("overscroll.rs", include_str!("../../src/overscroll.rs")),
    ("paint.rs", include_str!("../../src/paint.rs")),
    ("panel.rs", include_str!("../../src/panel.rs")),
    ("picker.rs", include_str!("../../src/picker.rs")),
    ("popover.rs", include_str!("../../src/popover.rs")),
    ("range.rs", include_str!("../../src/range.rs")),
    ("scanlines.rs", include_str!("../../src/scanlines.rs")),
    ("search.rs", include_str!("../../src/search.rs")),
    ("select.rs", include_str!("../../src/select.rs")),
    ("shader_view.rs", include_str!("../../src/shader_view.rs")),
    ("sheets.rs", include_str!("../../src/sheets.rs")),
    ("shell.rs", include_str!("../../src/shell.rs")),
    ("spy.rs", include_str!("../../src/spy.rs")),
    ("surfaces.rs", include_str!("../../src/surfaces.rs")),
    ("switcher.rs", include_str!("../../src/switcher.rs")),
    ("tabs.rs", include_str!("../../src/tabs.rs")),
    ("tag.rs", include_str!("../../src/tag.rs")),
    ("text_input.rs", include_str!("../../src/text_input.rs")),
    ("text.rs", include_str!("../../src/text.rs")),
    ("time/clock.rs", include_str!("../../src/time/clock.rs")),
    (
        "time/stopwatch.rs",
        include_str!("../../src/time/stopwatch.rs"),
    ),
    ("time/tap.rs", include_str!("../../src/time/tap.rs")),
    ("time/timer.rs", include_str!("../../src/time/timer.rs")),
];

/// Source of every showcase section file, to find what a section uses.
const SECTIONS: &[&str] = &[
    include_str!("sections/actions.rs"),
    include_str!("sections/data_display.rs"),
    include_str!("sections/inputs.rs"),
    include_str!("sections/navigation.rs"),
    include_str!("sections/charts_sec.rs"),
    include_str!("sections/foundations_sec.rs"),
    include_str!("sections/shape.rs"),
];

/// Layout and chrome helpers that every section uses; not worth inspecting.
const SKIP: &[&str] = &["Panel", "Muted", "Heading", "Theme", "Self"];

/// The source a section is built from: the statements that define the widgets it shows
/// (`let buttons = …`, which sit above the section) plus its own `self.section("title", …)` body.
fn section_text(title: &str) -> Option<String> {
    let marker = format!("self.section(\"{title}\"");
    SECTIONS.iter().find_map(|src| {
        let start = src.find(&marker)?;
        let rest = &src[start + marker.len()..];
        let end = rest.find("self.section(\"").unwrap_or(rest.len());
        let body = &rest[..end];
        let above = &src[..start];
        let mut text = String::new();
        let mut seen: Vec<&str> = Vec::new();
        for tok in body.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
            if tok.is_empty() || seen.contains(&tok) {
                continue;
            }
            seen.push(tok);
            for pat in [format!("let {tok} ="), format!("let mut {tok} =")] {
                // The statement may sit at any indentation (inside a block or closure).
                let Some(at) = above.rfind(&pat) else {
                    continue;
                };
                let line = above[..at].rfind('\n').map_or(0, |n| n + 1);
                let indent = &above[line..at];
                if !indent.chars().all(|c| c == ' ') {
                    continue;
                }
                let def = &above[line..];
                let next = format!("\n{indent}let ");
                let stop = def[1..].find(&next).map_or(def.len(), |e| e + 1);
                text.push_str(&def[..stop]);
                text.push('\n');
                break;
            }
        }
        text.push_str(body);
        Some(text)
    })
}

/// Whether a type called `name` is defined in `dodai-gpui`.
fn defined(name: &str) -> bool {
    FILES.iter().any(|(_, src)| {
        src.lines().any(|l| {
            ["pub struct ", "pub enum "]
                .iter()
                .any(|k| l.strip_prefix(k).is_some_and(|r| ident_start(r) == name))
        })
    })
}

/// The leading identifier of `s`.
fn ident_start(s: &str) -> &str {
    let end = s
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    &s[..end]
}

/// Components a section demonstrates, in order of first use.
pub(crate) fn components(title: &str) -> Vec<String> {
    let Some(text) = section_text(title) else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    let mut rest = text.as_str();
    while let Some(i) = rest.find("::new(") {
        let before = &rest[..i];
        let start = before
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |p| p + 1);
        let name = &before[start..];
        if name.chars().next().is_some_and(char::is_uppercase)
            && !SKIP.contains(&name)
            && !out.iter().any(|n| n == name)
            && defined(name)
        {
            out.push(name.to_string());
        }
        rest = &rest[i + "::new(".len()..];
    }
    out
}

/// Net `{` minus `}` of a line, ignoring strings, chars and `//` comments.
fn brace_delta(line: &str) -> i32 {
    let (mut d, mut in_str, mut esc) = (0, false, false);
    let b: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == '"' {
                in_str = false;
            }
        } else if c == '"' {
            in_str = true;
        } else if c == '/' && b.get(i + 1) == Some(&'/') {
            break;
        } else if c == '\'' && b.get(i + 2) == Some(&'\'') {
            i += 2;
        } else if c == '{' {
            d += 1;
        } else if c == '}' {
            d -= 1;
        }
        i += 1;
    }
    d
}

/// Whether a top-level item header belongs to `name`: its definition or an `impl` for it.
fn is_item_of(header: &str, name: &str) -> bool {
    let h = header.trim_start();
    let h = h.strip_prefix("pub ").unwrap_or(h);
    for k in ["struct ", "enum "] {
        if let Some(r) = h.strip_prefix(k) {
            return ident_start(r) == name;
        }
    }
    let Some(r) = h.strip_prefix("impl") else {
        return false;
    };
    // `impl Name`, `impl<T> Name`, `impl Trait for Name`.
    let r = if r.starts_with('<') {
        r.find('>').map_or(r, |i| &r[i + 1..])
    } else {
        r
    };
    let target = match r.find(" for ") {
        Some(i) => &r[i + 5..],
        None => r,
    };
    ident_start(target.trim_start()) == name
}

/// A type's definition and every `impl` block for it, in file order (docs and attributes
/// included), joined by blank lines. `None` if `name` is not defined in `dodai-gpui`.
pub(crate) fn implementation(name: &str) -> Option<String> {
    let (_, src) = FILES.iter().find(|(_, src)| {
        src.lines().any(|l| {
            ["pub struct ", "pub enum "]
                .iter()
                .any(|k| l.strip_prefix(k).is_some_and(|r| ident_start(r) == name))
        })
    })?;
    let lines: Vec<&str> = src.lines().collect();
    let mut blocks: Vec<String> = Vec::new();
    let mut i = 0;
    let mut pending: Vec<&str> = Vec::new();
    while i < lines.len() {
        let l = lines[i];
        // Docs and attributes at the top level attach to the next item.
        if l.starts_with("///") || l.starts_with("#[") {
            pending.push(l);
            i += 1;
            continue;
        }
        let top =
            !l.is_empty() && !l.starts_with(' ') && !l.starts_with('}') && !l.starts_with("//");
        if !top {
            pending.clear();
            i += 1;
            continue;
        }
        // Consume the whole item.
        let start = i;
        let (mut depth, mut seen) = (0i32, false);
        loop {
            let d = brace_delta(lines[i]);
            depth += d;
            seen |= lines[i].contains('{');
            let done = if seen {
                depth <= 0
            } else {
                lines[i].trim_end().ends_with(';')
            };
            if done || i + 1 >= lines.len() {
                break;
            }
            i += 1;
        }
        if is_item_of(l, name) {
            let mut b: Vec<&str> = std::mem::take(&mut pending);
            b.extend(&lines[start..=i]);
            blocks.push(b.join("\n"));
        }
        pending.clear();
        i += 1;
    }
    (!blocks.is_empty()).then(|| blocks.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_is_found_with_its_impls() {
        let code = implementation("Button").expect("Button");
        assert!(code.contains("pub struct Button"));
        assert!(code.contains("impl Button"));
        assert!(code.contains("fn build"));
        assert!(!code.contains("pub struct IconButton"));
    }

    #[test]
    fn unknown_types_have_no_implementation() {
        assert!(implementation("NoSuchThing").is_none());
    }

    #[test]
    fn sections_list_their_components() {
        let c = components("Buttons");
        assert!(c.iter().any(|n| n == "Button"), "{c:?}");
        assert!(c.iter().any(|n| n == "Fab"), "{c:?}");
        assert!(!c.iter().any(|n| n == "Panel"));
    }

    #[test]
    fn every_listed_component_has_code() {
        for (_, src) in SECTIONS.iter().map(|s| ((), s)) {
            for l in src
                .lines()
                .filter_map(|l| l.split("self.section(\"").nth(1))
            {
                let title = l.split('"').next().unwrap_or("");
                for c in components(title) {
                    assert!(implementation(&c).is_some(), "{title}: {c}");
                }
            }
        }
    }
}
