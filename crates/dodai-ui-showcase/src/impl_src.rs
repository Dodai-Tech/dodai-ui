//! The **implementation** behind each showcase section: the source of the `dodai-ui-gpui`
//! components a section demonstrates, read back from the crate's own files at compile time.
//!
//! Components are detected from the section's own source (`Name::new(` for every type that
//! `dodai-ui-gpui` defines), and a type's code is its definition plus every `impl` block for it.

/// `(file name, source)` of every component file of `dodai-ui-gpui` and `dodai-ui-charts`.
const FILES: &[(&str, &str)] = &[
    (
        "dodai-ui-gpui/src/button.rs",
        include_str!("../../dodai-ui-gpui/src/button.rs"),
    ),
    (
        "dodai-ui-gpui/src/carousel.rs",
        include_str!("../../dodai-ui-gpui/src/carousel.rs"),
    ),
    (
        "dodai-ui-gpui/src/command.rs",
        include_str!("../../dodai-ui-gpui/src/command.rs"),
    ),
    (
        "dodai-ui-gpui/src/content.rs",
        include_str!("../../dodai-ui-gpui/src/content.rs"),
    ),
    (
        "dodai-ui-gpui/src/markdown.rs",
        include_str!("../../dodai-ui-gpui/src/markdown.rs"),
    ),
    (
        "dodai-ui-gpui/src/controls/bars.rs",
        include_str!("../../dodai-ui-gpui/src/controls/bars.rs"),
    ),
    (
        "dodai-ui-gpui/src/controls/choice.rs",
        include_str!("../../dodai-ui-gpui/src/controls/choice.rs"),
    ),
    (
        "dodai-ui-gpui/src/controls/slider.rs",
        include_str!("../../dodai-ui-gpui/src/controls/slider.rs"),
    ),
    (
        "dodai-ui-gpui/src/count.rs",
        include_str!("../../dodai-ui-gpui/src/count.rs"),
    ),
    (
        "dodai-ui-gpui/src/data.rs",
        include_str!("../../dodai-ui-gpui/src/data.rs"),
    ),
    (
        "dodai-ui-gpui/src/datatable.rs",
        include_str!("../../dodai-ui-gpui/src/datatable.rs"),
    ),
    (
        "dodai-ui-gpui/src/display.rs",
        include_str!("../../dodai-ui-gpui/src/display.rs"),
    ),
    (
        "dodai-ui-gpui/src/extras.rs",
        include_str!("../../dodai-ui-gpui/src/extras.rs"),
    ),
    (
        "dodai-ui-gpui/src/feedback.rs",
        include_str!("../../dodai-ui-gpui/src/feedback.rs"),
    ),
    (
        "dodai-ui-gpui/src/forms.rs",
        include_str!("../../dodai-ui-gpui/src/forms.rs"),
    ),
    (
        "dodai-ui-gpui/src/icons.rs",
        include_str!("../../dodai-ui-gpui/src/icons.rs"),
    ),
    (
        "dodai-ui-gpui/src/interact.rs",
        include_str!("../../dodai-ui-gpui/src/interact.rs"),
    ),
    (
        "dodai-ui-gpui/src/orientation.rs",
        include_str!("../../dodai-ui-gpui/src/orientation.rs"),
    ),
    (
        "dodai-ui-gpui/src/knob.rs",
        include_str!("../../dodai-ui-gpui/src/knob.rs"),
    ),
    (
        "dodai-ui-gpui/src/layout.rs",
        include_str!("../../dodai-ui-gpui/src/layout.rs"),
    ),
    (
        "dodai-ui-gpui/src/lazy.rs",
        include_str!("../../dodai-ui-gpui/src/lazy.rs"),
    ),
    (
        "dodai-ui-gpui/src/list.rs",
        include_str!("../../dodai-ui-gpui/src/list.rs"),
    ),
    (
        "dodai-ui-gpui/src/logic.rs",
        include_str!("../../dodai-ui-gpui/src/logic.rs"),
    ),
    (
        "dodai-ui-gpui/src/expressive.rs",
        include_str!("../../dodai-ui-gpui/src/expressive.rs"),
    ),
    (
        "dodai-ui-gpui/src/menu.rs",
        include_str!("../../dodai-ui-gpui/src/menu.rs"),
    ),
    (
        "dodai-ui-gpui/src/motion.rs",
        include_str!("../../dodai-ui-gpui/src/motion.rs"),
    ),
    (
        "dodai-ui-gpui/src/nav.rs",
        include_str!("../../dodai-ui-gpui/src/nav.rs"),
    ),
    (
        "dodai-ui-gpui/src/navigation.rs",
        include_str!("../../dodai-ui-gpui/src/navigation.rs"),
    ),
    (
        "dodai-ui-gpui/src/overlay.rs",
        include_str!("../../dodai-ui-gpui/src/overlay.rs"),
    ),
    (
        "dodai-ui-gpui/src/overscroll.rs",
        include_str!("../../dodai-ui-gpui/src/overscroll.rs"),
    ),
    (
        "dodai-ui-gpui/src/paint.rs",
        include_str!("../../dodai-ui-gpui/src/paint.rs"),
    ),
    (
        "dodai-ui-gpui/src/panel.rs",
        include_str!("../../dodai-ui-gpui/src/panel.rs"),
    ),
    (
        "dodai-ui-gpui/src/picker.rs",
        include_str!("../../dodai-ui-gpui/src/picker.rs"),
    ),
    (
        "dodai-ui-gpui/src/popover.rs",
        include_str!("../../dodai-ui-gpui/src/popover.rs"),
    ),
    (
        "dodai-ui-gpui/src/range.rs",
        include_str!("../../dodai-ui-gpui/src/range.rs"),
    ),
    (
        "dodai-ui-gpui/src/scanlines.rs",
        include_str!("../../dodai-ui-gpui/src/scanlines.rs"),
    ),
    (
        "dodai-ui-gpui/src/search.rs",
        include_str!("../../dodai-ui-gpui/src/search.rs"),
    ),
    (
        "dodai-ui-gpui/src/select.rs",
        include_str!("../../dodai-ui-gpui/src/select.rs"),
    ),
    (
        "dodai-ui-shaders/src/gpui.rs",
        include_str!("../../dodai-ui-shaders/src/gpui.rs"),
    ),
    (
        "dodai-ui-gpui/src/sheets.rs",
        include_str!("../../dodai-ui-gpui/src/sheets.rs"),
    ),
    (
        "dodai-ui-gpui/src/shell.rs",
        include_str!("../../dodai-ui-gpui/src/shell.rs"),
    ),
    (
        "dodai-ui-gpui/src/snap_stack.rs",
        include_str!("../../dodai-ui-gpui/src/snap_stack.rs"),
    ),
    (
        "dodai-ui-gpui/src/spy.rs",
        include_str!("../../dodai-ui-gpui/src/spy.rs"),
    ),
    (
        "dodai-ui-gpui/src/surfaces.rs",
        include_str!("../../dodai-ui-gpui/src/surfaces.rs"),
    ),
    (
        "dodai-ui-gpui/src/switcher.rs",
        include_str!("../../dodai-ui-gpui/src/switcher.rs"),
    ),
    (
        "dodai-ui-gpui/src/tabs.rs",
        include_str!("../../dodai-ui-gpui/src/tabs.rs"),
    ),
    (
        "dodai-ui-gpui/src/tag.rs",
        include_str!("../../dodai-ui-gpui/src/tag.rs"),
    ),
    (
        "dodai-ui-gpui/src/text_input.rs",
        include_str!("../../dodai-ui-gpui/src/text_input.rs"),
    ),
    (
        "dodai-ui-gpui/src/text.rs",
        include_str!("../../dodai-ui-gpui/src/text.rs"),
    ),
    (
        "dodai-ui-gpui/src/time/clock.rs",
        include_str!("../../dodai-ui-gpui/src/time/clock.rs"),
    ),
    (
        "dodai-ui-gpui/src/time/stopwatch.rs",
        include_str!("../../dodai-ui-gpui/src/time/stopwatch.rs"),
    ),
    (
        "dodai-ui-gpui/src/time/tap.rs",
        include_str!("../../dodai-ui-gpui/src/time/tap.rs"),
    ),
    (
        "dodai-ui-gpui/src/time/timer.rs",
        include_str!("../../dodai-ui-gpui/src/time/timer.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/annotate.rs",
        include_str!("../../dodai-ui-charts/src/gpui/annotate.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/bar.rs",
        include_str!("../../dodai-ui-charts/src/gpui/bar.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/bubble_timeline.rs",
        include_str!("../../dodai-ui-charts/src/gpui/bubble_timeline.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/cascade.rs",
        include_str!("../../dodai-ui-charts/src/gpui/cascade.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/common.rs",
        include_str!("../../dodai-ui-charts/src/gpui/common.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/compare.rs",
        include_str!("../../dodai-ui-charts/src/gpui/compare.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/distribution.rs",
        include_str!("../../dodai-ui-charts/src/gpui/distribution.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/extra.rs",
        include_str!("../../dodai-ui-charts/src/gpui/extra.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/fan.rs",
        include_str!("../../dodai-ui-charts/src/gpui/fan.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/flow.rs",
        include_str!("../../dodai-ui-charts/src/gpui/flow.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/focus.rs",
        include_str!("../../dodai-ui-charts/src/gpui/focus.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/grid.rs",
        include_str!("../../dodai-ui-charts/src/gpui/grid.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/hierarchy.rs",
        include_str!("../../dodai-ui-charts/src/gpui/hierarchy.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/kpi.rs",
        include_str!("../../dodai-ui-charts/src/gpui/kpi.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/layered.rs",
        include_str!("../../dodai-ui-charts/src/gpui/layered.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/mod.rs",
        include_str!("../../dodai-ui-charts/src/gpui/mod.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/line.rs",
        include_str!("../../dodai-ui-charts/src/gpui/line.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/market.rs",
        include_str!("../../dodai-ui-charts/src/gpui/market.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/milestones.rs",
        include_str!("../../dodai-ui-charts/src/gpui/milestones.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/network.rs",
        include_str!("../../dodai-ui-charts/src/gpui/network.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/packed.rs",
        include_str!("../../dodai-ui-charts/src/gpui/packed.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/part_whole.rs",
        include_str!("../../dodai-ui-charts/src/gpui/part_whole.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/peaks.rs",
        include_str!("../../dodai-ui-charts/src/gpui/peaks.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/pie.rs",
        include_str!("../../dodai-ui-charts/src/gpui/pie.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/process.rs",
        include_str!("../../dodai-ui-charts/src/gpui/process.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/pulse.rs",
        include_str!("../../dodai-ui-charts/src/gpui/pulse.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/radar.rs",
        include_str!("../../dodai-ui-charts/src/gpui/radar.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/shards.rs",
        include_str!("../../dodai-ui-charts/src/gpui/shards.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/spark.rs",
        include_str!("../../dodai-ui-charts/src/gpui/spark.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/spread.rs",
        include_str!("../../dodai-ui-charts/src/gpui/spread.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/status.rs",
        include_str!("../../dodai-ui-charts/src/gpui/status.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/survey.rs",
        include_str!("../../dodai-ui-charts/src/gpui/survey.rs"),
    ),
    (
        "dodai-ui-charts/src/gpui/trend.rs",
        include_str!("../../dodai-ui-charts/src/gpui/trend.rs"),
    ),
];

/// Every showcase source that builds sections or the demos they show. The catalog, the code
/// panel and the snippets are left out: they name sections without building them.
const SECTIONS: &[&str] = &[
    include_str!("sections/actions.rs"),
    include_str!("sections/data_display.rs"),
    include_str!("sections/inputs.rs"),
    include_str!("sections/navigation.rs"),
    include_str!("sections/charts_sec.rs"),
    include_str!("sections/charts_layered.rs"),
    include_str!("sections/charts_more.rs"),
    include_str!("sections/charts_status.rs"),
    include_str!("sections/throughput.rs"),
    include_str!("sections/foundations_sec.rs"),
    include_str!("sections/shape.rs"),
    include_str!("charts_page.rs"),
];

/// Layout and chrome helpers that every section uses; not worth inspecting.
const SKIP: &[&str] = &["Panel", "Muted", "Heading", "Theme", "Self"];

/// Showcase helpers not followed when collecting a section's source: the section frame itself.
const NO_FOLLOW: &[&str] = &["section", "with_code", "theme_of"];

/// Longest helper function (bytes) followed into; longer ones build whole pages.
const MAX_HELPER: usize = 8_000;

/// How many levels of showcase helper functions a section's source is followed into.
const FOLLOW_DEPTH: usize = 2;

/// The source a section is built from: its own body (`self.section("title", …)` up to the next
/// section, or the call or tuple that names it, like `("Bar", …)` or `add(self, "Title", …)`),
/// the statements above it that define the widgets it shows (`let buttons = …`), and the
/// showcase helper functions it calls.
fn section_text(title: &str) -> Option<String> {
    // Each piece of source keeps the file it came from: a free helper is looked up there.
    let mut chunks: Vec<(usize, String)> = Vec::new();
    for (k, src) in SECTIONS.iter().enumerate() {
        for (start, body) in section_bodies(src, title) {
            let mut text = lets_above(fn_scope(&src[..start]), body);
            text.push_str(body);
            chunks.push((k, text));
        }
    }
    if chunks.is_empty() {
        return None;
    }
    let mut followed: Vec<(usize, &str)> = Vec::new();
    let mut frontier = chunks.clone();
    for _ in 0..FOLLOW_DEPTH {
        let mut next: Vec<(usize, String)> = Vec::new();
        for (k, text) in &frontier {
            for tok in idents(text) {
                if NO_FOLLOW.contains(&tok) {
                    continue;
                }
                let found = match helper_use(text, tok) {
                    None => None,
                    Some(Helper::Method) => SECTIONS
                        .iter()
                        .enumerate()
                        .find_map(|(j, src)| fn_body(src, tok).map(|b| (j, b))),
                    Some(Helper::Free) => fn_body(SECTIONS[*k], tok).map(|b| (*k, b)),
                };
                // A whole page builder would list every component of the page.
                let Some((j, body)) = found.filter(|(_, b)| b.len() <= MAX_HELPER) else {
                    continue;
                };
                if followed.contains(&(j, body)) {
                    continue;
                }
                followed.push((j, body));
                next.push((j, body.to_string()));
            }
        }
        chunks.extend(next.iter().cloned());
        frontier = next;
    }
    Some(
        chunks
            .into_iter()
            .map(|(_, t)| t)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// How a section's source uses a showcase helper function.
enum Helper {
    /// `self.name(…)`: a showcase method, defined in any file.
    Method,
    /// `name(…)` or `name` passed as a value: a function of the same file.
    Free,
}

/// Whether `text` calls `name` as a showcase helper (`name(…)`, `self.name(…)`) or passes it
/// as a function value (`(…, name)`), rather than calling a component's builder method
/// (`.name(…)`) or a path (`x::name`).
fn helper_use(text: &str, name: &str) -> Option<Helper> {
    let mut out = None;
    for (i, _) in text.match_indices(name) {
        let before = &text[..i];
        let after = &text[i + name.len()..];
        let word_start = !before.ends_with(|c: char| c.is_alphanumeric() || c == '_');
        let word_end = !after.starts_with(|c: char| c.is_alphanumeric() || c == '_');
        if !(word_start && word_end) || before.ends_with("::") || before.ends_with("fn ") {
            continue;
        }
        let after = after.trim_start();
        if before.ends_with("self.") {
            if after.starts_with('(') {
                return Some(Helper::Method);
            }
        } else if !before.ends_with('.')
            && (after.starts_with('(')
                || (before.trim_end().ends_with([',', '(']) && after.starts_with([',', ')'])))
        {
            out = Some(Helper::Free);
        }
    }
    out
}

/// The distinct identifiers of `text`, in order.
fn idents(text: &str) -> Vec<&str> {
    let mut seen: Vec<&str> = Vec::new();
    for tok in text.split(|c: char| !(c.is_alphanumeric() || c == '_')) {
        if !tok.is_empty() && !seen.contains(&tok) {
            seen.push(tok);
        }
    }
    seen
}

/// `(start, body)` of every place in `src` that builds the section `title`.
fn section_bodies<'a>(src: &'a str, title: &str) -> Vec<(usize, &'a str)> {
    let lit = format!("\"{title}\"");
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = src[from..].find(&lit) {
        let at = from + i;
        from = at + lit.len();
        if let Some((open, close)) = enclosing_parens(src, at) {
            out.push((open, &src[open..close]));
            // One body per call or tuple, even when it names the section twice.
            from = from.max(close);
        }
    }
    if !out.is_empty() {
        return out;
    }
    // A section built as a statement (`let x = self.section("title", cx)…;`): its body runs
    // to the next section.
    let marker = format!("self.section({lit}");
    let Some(start) = src.find(&marker) else {
        return out;
    };
    let rest = &src[start + marker.len()..];
    let end = rest.find("self.section(\"").unwrap_or(rest.len());
    vec![(start, &rest[..end])]
}

/// Byte offsets and bytes of every bracket and `;` in `src` outside strings, chars and comments.
fn brackets(src: &str) -> Vec<(usize, u8)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'/' if b.get(i + 1) == Some(&b'/') => {
                i = src[i..].find('\n').map_or(b.len(), |n| i + n);
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i = src[i..].find("*/").map_or(b.len(), |n| i + n + 1);
            }
            b'r' if matches!(b.get(i + 1), Some(b'"' | b'#'))
                && (i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_')) =>
            {
                let hashes = b[i + 1..].iter().take_while(|c| **c == b'#').count();
                let close = format!("\"{}", "#".repeat(hashes));
                let body = i + 2 + hashes;
                i = src[body.min(b.len())..]
                    .find(&close)
                    .map_or(b.len(), |n| body + n + close.len() - 1);
            }
            b'"' => {
                i += 1;
                while i < b.len() && b[i] != b'"' {
                    i += if b[i] == b'\\' { 2 } else { 1 };
                }
            }
            b'\'' if b.get(i + 1) == Some(&b'\\') => {
                i = src[i + 2..].find('\'').map_or(b.len(), |n| i + 2 + n);
            }
            b'\'' if b.get(i + 2) == Some(&b'\'') => i += 2,
            c @ (b'(' | b')' | b'[' | b']' | b'{' | b'}' | b';') => out.push((i, c)),
            _ => {}
        }
        i += 1;
    }
    out
}

/// The `(`…`)` group around byte `at`, or `None` when the innermost group around it is a list
/// or block (a table of names rather than a demo). A group that is the section frame's own
/// argument list (`section("…", cx)`) widens to the group around that call.
fn enclosing_parens(src: &str, at: usize) -> Option<(usize, usize)> {
    let mut marks = brackets(src);
    marks.retain(|(_, c)| *c != b';');
    let split = marks.partition_point(|(i, _)| *i < at);
    let mut depth = 0i32;
    let mut open = None;
    for (k, &(i, c)) in marks[..split].iter().enumerate().rev() {
        match c {
            b')' | b']' | b'}' => depth += 1,
            _ if depth > 0 => depth -= 1,
            b'(' if src[..i].ends_with("section") => {}
            b'(' => {
                open = Some((k, i));
                break;
            }
            _ => return None,
        }
    }
    let (k, open) = open?;
    let mut depth = 0i32;
    for &(i, c) in &marks[k..] {
        match c {
            b'(' | b'[' | b'{' => depth += 1,
            _ => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, i + 1));
                }
            }
        }
    }
    None
}

/// The tail of `above` from the start of the function it ends in, so only that function's
/// `let` statements are searched.
fn fn_scope(above: &str) -> &str {
    let mut at = above.len();
    for line in above.lines().rev() {
        at -= line.len() + 1;
        let l = line.trim_start();
        let l = l
            .strip_prefix("pub(crate) ")
            .or_else(|| l.strip_prefix("pub "))
            .unwrap_or(l);
        if l.starts_with("fn ") {
            return &above[at.min(above.len())..];
        }
    }
    above
}

/// The `let name = …` statements that define identifiers `body` uses: from `scope` (the
/// function the section sits in) first, else from any showcase source. A name bound by
/// destructuring (`field: (name, …)`) is looked up by its field.
fn lets_above(scope: &str, body: &str) -> String {
    let mut text = String::new();
    for tok in idents(body) {
        let field = destructured_field(scope, tok);
        let names = [Some(tok), field].into_iter().flatten();
        // Only names the scope pulls out of a struct are defined elsewhere (where the
        // struct was filled); any other name would just match an unrelated `let`.
        let found = names
            .clone()
            .find_map(|n| let_statement(scope, n))
            .or_else(|| {
                // Look where the struct is filled in (`Demos {` outside a `let` pattern).
                let ty = destructured(scope, tok)?;
                let filled = format!("{ty} {{");
                let src = SECTIONS.iter().find(|src| {
                    src.match_indices(&filled)
                        .any(|(i, _)| !src[..i].trim_end().ends_with("let"))
                })?;
                names.clone().find_map(|n| let_statement(src, n))
            });
        if let Some(stmt) = found {
            text.push_str(stmt);
            text.push('\n');
            if stmt.starts_with("let mut ") {
                for again in reassignments(scope, tok) {
                    text.push_str(again);
                    text.push('\n');
                }
            }
        }
    }
    text
}

/// The last `let name = …;` statement in `src`.
fn let_statement<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let at = [format!("let {name} ="), format!("let mut {name} =")]
        .iter()
        .filter_map(|p| statement_starts(src, p).last())
        .max()?;
    statement_at(src, at)
}

/// The later `name = …;` statements of `scope` that rebuild a `let mut name`.
fn reassignments<'a>(scope: &'a str, name: &str) -> Vec<&'a str> {
    let pat = format!("{name} = ");
    let starts: Vec<usize> = statement_starts(scope, &pat).collect();
    starts
        .into_iter()
        .filter_map(|at| statement_at(scope, at))
        .collect()
}

/// Offsets of the lines of `src` that start (after indentation) with `pat`.
fn statement_starts<'a>(src: &'a str, pat: &'a str) -> impl Iterator<Item = usize> + 'a {
    src.match_indices(pat).map(|(i, _)| i).filter(move |&i| {
        let line = src[..i].rfind('\n').map_or(0, |n| n + 1);
        src[line..i].chars().all(|c| c == ' ')
    })
}

/// The statement of `src` starting at `at`, up to its `;` (or the end of its block).
fn statement_at(src: &str, at: usize) -> Option<&str> {
    let mut depth = 0i32;
    for (i, c) in brackets(&src[at..]) {
        match c {
            b';' if depth == 0 => return Some(&src[at..=at + i]),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => {
                depth -= 1;
                // The block the statement sits in closes: it was its tail expression.
                if depth < 0 {
                    return Some(&src[at..at + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The struct `scope` destructures `name` from (`let Demos { name, .. } = …` gives `Demos`).
fn destructured<'a>(scope: &'a str, name: &str) -> Option<&'a str> {
    scope.match_indices("let ").find_map(|(i, _)| {
        let rest = &scope[i + 4..];
        if !rest.starts_with(|c: char| c.is_ascii_uppercase()) {
            return None;
        }
        let pattern = &rest[..rest.find("} =")?];
        let ty = ident_start(rest);
        (pattern.contains('{') && idents(pattern).contains(&name)).then_some(ty)
    })
}

/// The field a destructuring pattern in `scope` binds `name` from (`field: (name, …)`).
fn destructured_field<'a>(scope: &'a str, name: &str) -> Option<&'a str> {
    scope.match_indices(": (").find_map(|(i, _)| {
        let inner = &scope[i + 3..];
        let inner = &inner[..inner.find(')')?];
        let binds = inner.split(',').any(|p| p.trim() == name);
        let start = scope[..i]
            .rfind(|c: char| !(c.is_alphanumeric() || c == '_'))
            .map_or(0, |p| p + 1);
        (binds && start < i).then(|| &scope[start..i])
    })
}

/// The whole definition of the function `name` in `src` (free function or method).
fn fn_body<'a>(src: &'a str, name: &str) -> Option<&'a str> {
    let at = [format!("fn {name}("), format!("fn {name}<")]
        .iter()
        .find_map(|p| src.find(p.as_str()))?;
    let mut depth = 0i32;
    let mut opened = false;
    for (i, c) in brackets(&src[at..]) {
        match c {
            b'{' => {
                depth += 1;
                opened = true;
            }
            b'}' => {
                depth -= 1;
                if opened && depth == 0 {
                    return Some(&src[at..=at + i]);
                }
            }
            _ => {}
        }
    }
    None
}

/// Whether a type called `name` is defined in `dodai-ui-gpui`.
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
    // The sources are static and scanning them takes milliseconds: do it once per title.
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<String, Vec<String>>> =
            Default::default();
    }
    if let Some(hit) = CACHE.with(|c| c.borrow().get(title).cloned()) {
        return hit;
    }
    let out = scan_components(title);
    CACHE.with(|c| c.borrow_mut().insert(title.to_string(), out.clone()));
    out
}

fn scan_components(title: &str) -> Vec<String> {
    let Some(text) = section_text(title) else {
        return Vec::new();
    };
    // Every `Name::constructor(` (`Button::new(`, `Pulse::ecg(`) of a type the crates define.
    let mut out: Vec<String> = Vec::new();
    for (i, _) in text.match_indices("::") {
        let after = &text[i + 2..];
        let ctor = ident_start(after);
        if !ctor.starts_with(|c: char| c.is_ascii_lowercase())
            || !after[ctor.len()..].starts_with('(')
        {
            continue;
        }
        let before = &text[..i];
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
/// included), joined by blank lines. `None` if `name` is not defined in `dodai-ui-gpui`.
pub(crate) fn implementation(name: &str) -> Option<String> {
    thread_local! {
        static CACHE: std::cell::RefCell<std::collections::HashMap<String, Option<String>>> =
            Default::default();
    }
    if let Some(hit) = CACHE.with(|c| c.borrow().get(name).cloned()) {
        return hit;
    }
    let out = scan_implementation(name);
    CACHE.with(|c| c.borrow_mut().insert(name.to_string(), out.clone()));
    out
}

fn scan_implementation(name: &str) -> Option<String> {
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

    /// Every chart section, as the showcase lists them.
    fn chart_titles() -> Vec<&'static str> {
        let mut titles: Vec<&'static str> = crate::catalog::CHART_FAMILIES
            .iter()
            .flat_map(|(_, s)| s.iter().copied())
            .collect();
        titles.sort_unstable();
        titles.dedup();
        titles
    }

    #[test]
    fn chart_sections_show_their_code() {
        let titles = chart_titles();
        assert!(titles.len() > 60, "{titles:?}");
        for title in titles {
            let comps = components(title);
            assert!(!comps.is_empty(), "{title}: no components");
            for c in comps {
                assert!(implementation(&c).is_some(), "{title}: {c}");
            }
        }
    }

    #[test]
    fn a_section_lists_only_its_own_components() {
        assert_eq!(components("Sunburst"), ["Sunburst"]);
    }

    #[test]
    fn helpers_are_told_from_builder_methods() {
        assert!(matches!(
            helper_use("x.child(row())", "row"),
            Some(Helper::Free)
        ));
        assert!(matches!(
            helper_use("self.demo(cx)", "demo"),
            Some(Helper::Method)
        ));
        assert!(matches!(
            helper_use("looks(a, face)", "face"),
            Some(Helper::Free)
        ));
        assert!(helper_use("p.rate(3.0)", "rate").is_none());
        assert!(helper_use("Pulse::rate(3.0)", "rate").is_none());
        assert!(helper_use("show_rate(true)", "rate").is_none());
    }

    #[test]
    fn brackets_skip_strings_chars_and_comments() {
        let marks: Vec<u8> = brackets("f(\"(\", '{', // )\n x)")
            .into_iter()
            .map(|m| m.1)
            .collect();
        assert_eq!(marks, b"()");
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
