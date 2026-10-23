//! Usage snippets shown under showcase sections ("</> Code").
//!
//! Each function below is real, compiled code: if a component's API changes this
//! file stops building, so the examples can't rot. The text between
//! `// snippet: <section>` and `// end snippet` is what the showcase displays
//! (read back from this very file with `include_str!`).

#![allow(dead_code, clippy::unused_unit)]

use super::*;

// snippet: Buttons
fn buttons() -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(Button::new("save", "Save").on_click(|_, _, _| println!("saved")))
        .child(Button::new("cancel", "Cancel").variant(ButtonVariant::Outline))
        .child(Button::new("delete", "Delete").variant(ButtonVariant::Danger))
        .child(Button::new("busy", "Loading…").shimmer(true))
        .child(Button::new("off", "Disabled").disabled(true))
}
// end snippet

// snippet: Toggle · Checkbox · Radio
fn toggles() -> impl IntoElement {
    div()
        .flex()
        .gap_3()
        .child(Toggle::new("sw", true).on_change(|on, _, _| println!("switch {on}")))
        .child(Checkbox::new("ck", false).on_change(|on, _, _| println!("check {on}")))
        .child(Radio::new("radio-a", true).on_change(|_, _, _| println!("picked")))
}
// end snippet

// snippet: Slider · Progress
fn sliders() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(Slider::new("volume", 0.4).on_change(|v, _, _| println!("{v:.2}")))
        .child(
            Slider::new("steps", 4.0)
                .range(0.0, 10.0)
                .step(1.0)
                .ticks(true)
                .show_value(true),
        )
}
// end snippet

// snippet: Range slider
fn range() -> impl IntoElement {
    RangeSlider::new("price", 0.2, 0.7).on_change(|lo, hi, _, _| println!("{lo:.2}..{hi:.2}"))
}
// end snippet

// snippet: Tabs
fn tabs() -> impl IntoElement {
    Tabs::new("tabs")
        .labels(["Overview", "Billing", "Settings"])
        .selected(0)
        .on_select(|i, _, _| println!("tab {i}"))
}
// end snippet

// snippet: Tags
fn tags() -> impl IntoElement {
    div()
        .flex()
        .gap_2()
        .child(Tag::new("online", Tone::Success))
        .child(Tag::new("warning", Tone::Warning).solid(true))
        .child(Tag::new("error", Tone::Danger).solid(true))
}
// end snippet

// snippet: Rating · Stepper · Badges · Tooltips
fn rating() -> impl IntoElement {
    Rating::new("rating", 3.5)
        .half(true)
        .on_change(|stars, _, _| println!("{stars} stars"))
}
// end snippet

// snippet: Circular progress · Banner
fn progress() -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(dodai_gpui::CircularProgress::new(0.72))
        .child(dodai_gpui::CircularProgress::indeterminate())
        .child(dodai_gpui::Banner::new(
            "update",
            Tone::Info,
            "A new version is available.",
        ))
}
// end snippet

// snippet: Code block · Markdown
fn code() -> impl IntoElement {
    div()
        .child(dodai_gpui::CodeBlock::new("code", "fn main() {}").language("rust"))
        .child(dodai_gpui::Markdown::new("md", "**bold** and `code`"))
}
// end snippet

/// All the source text of this file, for [`snippet`].
const SOURCE: &str = include_str!("snippets.rs");

/// The displayed code for a section title, dedented; `None` if it has no snippet.
pub(crate) fn snippet(section: &str) -> Option<String> {
    extract(SOURCE, section)
}

/// Every section title that has a snippet.
pub(crate) fn titles() -> Vec<&'static str> {
    SOURCE
        .lines()
        .filter_map(|l| l.trim().strip_prefix("// snippet: "))
        .collect()
}

fn extract(src: &str, section: &str) -> Option<String> {
    let marker = format!("// snippet: {section}");
    let mut lines = src.lines().skip_while(|l| l.trim() != marker).skip(1);
    let body: Vec<&str> = lines
        .by_ref()
        .take_while(|l| l.trim() != "// end snippet")
        .collect();
    if body.is_empty() {
        return None;
    }
    Some(body.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_snippet_is_well_formed() {
        let starts = SOURCE
            .lines()
            .filter(|l| l.trim().starts_with("// snippet: "))
            .count();
        let ends = SOURCE
            .lines()
            .filter(|l| l.trim() == "// end snippet")
            .count();
        assert_eq!(starts, ends, "unbalanced snippet markers");
        for t in titles() {
            let code = snippet(t).unwrap_or_default();
            assert!(code.lines().count() >= 2, "{t}: empty snippet");
            assert!(code.contains("fn ") && code.contains('{'), "{t}: {code}");
        }
    }

    /// A typo in a marker would silently hide its button; catch it.
    #[test]
    fn every_snippet_matches_a_real_section() {
        let sources = [
            include_str!("sections/actions.rs"),
            include_str!("sections/data_display.rs"),
            include_str!("sections/inputs.rs"),
            include_str!("sections/navigation.rs"),
            include_str!("sections/charts_sec.rs"),
            include_str!("sections/foundations_sec.rs"),
            include_str!("sections/shape.rs"),
        ]
        .concat();
        for t in titles() {
            assert!(
                sources.contains(&format!("section!(\"{t}\")")),
                "no section titled {t:?}"
            );
        }
    }

    #[test]
    fn unknown_sections_have_no_snippet() {
        assert!(snippet("No such section").is_none());
    }
}
