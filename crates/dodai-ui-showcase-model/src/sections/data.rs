//! Data display: tags, chips, lists, rating, timeline, tooltips, tables, code, progress.

use dodai_ui_core::Tone;
use dodai_ui_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Align, Bind, Column, Gap, Node, Table, TimelineEvent};

const CAT: &str = "Data display";

pub(crate) const SECTIONS: [Section; 13] = [
    Section::new(CAT, "Tags · Chips · Badges", tags).heading("Tags"),
    Section::new(CAT, "List items · Chips", lists_chips),
    Section::new(CAT, "Badges · Rating · Tooltips", misc)
        .heading("Rating · Stepper · Badges · Tooltips"),
    Section::new(CAT, "List · Divider", list_divider),
    Section::new(CAT, "Timeline · Empty state", timeline).heading("Timeline · empty state"),
    Section::new(CAT, "Tooltip · Rich tooltip", tooltip).heading("Tooltip"),
    Section::new(
        CAT,
        "Data table · sort filter select resize virtual scroll",
        table,
    )
    .heading("Data table"),
    Section::new(CAT, "Code block · Markdown · rich text", code).heading("Code block · Markdown"),
    Section::new(CAT, "Image · media · Avatar group", images).heading("Image · Avatar group"),
    Section::new(CAT, "Circular progress · Banner", progress_banner),
    Section::new(CAT, "Clock · Timer", clock).live(),
    Section::new(CAT, "Stopwatch · Tap tempo", stopwatch).live(),
    Section::new(CAT, "Scroll area · styled scrollbar", scroll_area).heading("Scroll area"),
];

const TONES: [(&str, Tone); 4] = [
    ("online", Tone::Success),
    ("warn", Tone::Warning),
    ("error", Tone::Danger),
    ("info", Tone::Info),
];

/// The picture the image demos show.
pub const SAMPLE_IMAGE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../dodai-ui-showcase/assets/sample.png"
);

fn tags(_: &Ctx) -> Node {
    let mut v: Vec<Node> = TONES.iter().map(|(l, t)| tag(*l, *t, true)).collect();
    v.extend(TONES.iter().map(|(l, t)| tag(*l, *t, false)));
    row(v)
}

fn lists_chips(_: &Ctx) -> Node {
    col(vec![
        col(vec![
            muted("One line"),
            Node::List(vec![list_row("Inbox").icon(IconId::Home)]),
            muted("Two lines · trailing text"),
            Node::List(vec![list_row("Weekly digest")
                .detail("Everything that changed this week")
                .trailing("10:24")
                .avatar("WD")]),
            muted("Three lines"),
            Node::List(vec![list_row("Design review")
                .overline("Tomorrow")
                .detail("Bring the colour schemes and the new app bars")
                .icon(IconId::Heart)]),
        ])
        .gap(Gap::Px(0.0)),
        row(vec![
            chip("Assist", "chip-assist", false)
                .icon(IconId::Edit)
                .node(),
            chip("Elevated", "chip-elevated", false)
                .icon(IconId::Edit)
                .elevated()
                .node(),
            chip("Filter", "chip-off", false).filter().node(),
            chip("Filter", "chip-on", true).filter().node(),
            chip("Input", "chip-input", false)
                .icon(IconId::Heart)
                .removable()
                .node(),
            chip("Suggestion", "chip-sugg", false).node(),
        ]),
    ])
}

fn misc(_: &Ctx) -> Node {
    row(vec![
        Node::Rating { value: 3.5, max: 5 },
        Node::Stepper(Bind::new("stepper", 5)),
        Node::Anchor {
            open: Bind::new("tip-rich", false),
            trigger: Box::new(
                button("Hover (rich)", dodai_ui_core::ButtonVariant::Ghost)
                    .plain()
                    .counter(crate::commands::flip("tip-rich", false))
                    .node(),
            ),
            panel: Box::new(Node::Tooltip {
                text: "Title, body and an action.".into(),
                rich: true,
                title: Some("Rich tooltip".into()),
                action: Some("Learn more".into()),
            }),
        },
        Node::Badged {
            icon: IconId::Bell,
            count: Some((12, Tone::Danger)),
            dot: None,
        },
        Node::Badged {
            icon: IconId::Mail,
            count: None,
            dot: Some(Tone::Success),
        },
    ])
}

fn list_divider(_: &Ctx) -> Node {
    col(vec![
        Node::Pick {
            bind: Bind::new("list-pick", 0),
            items: ["Inbox", "Drafts", "Sent", "Archive"]
                .map(String::from)
                .to_vec(),
        },
        Node::Divider,
    ])
}

fn timeline(_: &Ctx) -> Node {
    let ev = |title: &str, time: &str, detail: Option<&str>, tone: Option<Tone>| TimelineEvent {
        title: title.into(),
        time: time.into(),
        detail: detail.map(str::to_string),
        tone,
    };
    row(vec![
        tile(
            220.0,
            vec![Node::Timeline(vec![
                ev("Charts added", "today", None, Some(Tone::Success)),
                ev("Search view", "today", Some("fuzzy + highlight"), None),
                ev("Build failed", "earlier", None, Some(Tone::Danger)),
            ])],
        ),
        Node::Empty {
            icon: "∅".into(),
            title: "Nothing here".into(),
            detail: "Add something to get started.".into(),
        },
    ])
}

fn tooltip(_c: &Ctx) -> Node {
    col(vec![
        line(vec![Node::Anchor {
            open: Bind::new("tip", false),
            trigger: Box::new(
                button("Hover me", dodai_ui_core::ButtonVariant::Ghost)
                    .plain()
                    .counter(crate::commands::flip("tip", false))
                    .node(),
            ),
            panel: Box::new(Node::Tooltip {
                text: "Themed tooltip".into(),
                rich: false,
                title: None,
                action: None,
            }),
        }]),
        Node::Tooltip {
            text: "Plain tooltip".into(),
            rich: false,
            title: None,
            action: None,
        },
        Node::Tooltip {
            text: "Supporting text, 12 px corners, elevation 2.".into(),
            rich: true,
            title: Some("Rich tooltip".into()),
            action: Some("Learn more".into()),
        },
    ])
}

/// Key of the data table's filter field.
const TABLE_FILTER: &str = "dt-filter";

/// The 300 demo rows of the data table (as the gpui showcase builds them).
pub(crate) fn table_rows() -> Vec<Vec<String>> {
    let roles = ["Engineer", "Designer", "Analyst", "Manager"];
    let regions = ["EMEA", "APAC", "AMER"];
    (0..300)
        .map(|i| {
            vec![
                format!("User {:03}", (i * 37) % 300),
                roles[i % 4].to_string(),
                regions[(i / 3) % 3].to_string(),
                format!("{}", (i * 7919) % 1000),
            ]
        })
        .collect()
}

/// The rows any of whose cells contain `query` (case-insensitive; empty keeps all).
pub(crate) fn filter_rows(rows: Vec<Vec<String>>, query: &str) -> Vec<Vec<String>> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return rows;
    }
    rows.into_iter()
        .filter(|r| r.iter().any(|c| c.to_lowercase().contains(&q)))
        .collect()
}

fn table(c: &Ctx) -> Node {
    let filter = Bind::new(TABLE_FILTER, String::new());
    let query = c.values.text(&filter).to_string();
    let column = |title: &str, width: f32, align: Align| Column {
        title: title.into(),
        width: Some(width),
        align,
    };
    col(vec![
        row(vec![
            input(TABLE_FILTER, "Filter rows…").node(),
            muted("click a header to sort · drag a column edge · click rows to select"),
        ]),
        Node::Table(Table {
            columns: vec![
                column("Name", 160.0, Align::Left),
                column("Role", 140.0, Align::Left),
                column("Region", 110.0, Align::Left),
                column("Score", 100.0, Align::Right),
            ],
            rows: filter_rows(table_rows(), &query),
            height: Some(260.0),
        }),
    ])
}

const CODE: &str = "use std::collections::HashMap;\n\n/// Count words.\nfn count(text: &str) -> HashMap<String, usize> {\n    let mut m = HashMap::new();\n    for w in text.split_whitespace() {\n        *m.entry(w.to_lowercase()).or_insert(0) += 1;\n    }\n    m // 42 buckets max\n}";

const MARKDOWN: &str = "# Markdown\n\nSupports **bold**, *italic*, `inline code` and [links](https://example.com).\n\n- headings and paragraphs\n- nested lists\n  - indent two spaces\n    1. numbered inside\n    2. restarts per level\n- fenced code and tables\n\n| Token | Value | Note |\n|:--|--:|:-:|\n| radius | 12 | rounded |\n| chamfer | 0 | none |\n\n> quotes read muted\n\n```rust\nfn main() {\n    let n: u32 = 42; // answer\n    println!(\"{n}\");\n}\n```";

fn code(c: &Ctx) -> Node {
    col(vec![
        Node::Code {
            text: CODE.into(),
            lang: "rust".into(),
        },
        crate::md::document(c.theme, MARKDOWN),
    ])
}

fn images(_: &Ctx) -> Node {
    let image = |w: f32, h: f32, video: bool, contain: bool, caption: &str| Node::Image {
        src: Some(SAMPLE_IMAGE.into()),
        video,
        contain,
        w,
        h,
        caption: Some(caption.into()),
        on_image: false,
    };
    let group = |names: &[&str], max: Option<usize>, size: f32| Node::AvatarGroup {
        names: names.iter().map(|s| s.to_string()).collect(),
        max,
        size,
    };
    col(vec![
        row(vec![
            image(260.0, 160.0, false, false, "Image (cover)"),
            image(260.0, 160.0, true, false, "Video poster"),
            image(160.0, 160.0, false, true, "Contain"),
        ])
        .start(),
        row(vec![
            group(&["AL", "BK", "CM", "DN"], None, 40.0),
            group(&["AL", "BK", "CM", "DN", "EO", "FP", "GQ"], Some(4), 40.0),
            group(&["AL", "BK", "CM", "DN", "EO"], Some(3), 28.0),
        ]),
    ])
}

fn progress_banner(c: &Ctx) -> Node {
    use crate::widgets::{elapsed, Ring};
    let (t, now) = (c.theme, elapsed());
    let banner = |tone, text: &str, action: Option<&str>, dismissible| Node::Alert {
        tone,
        text: text.into(),
        banner: true,
        action: action.map(str::to_string),
        dismissible,
    };
    col(vec![
        row(vec![
            Ring::new(0.25).node(t, now),
            Ring {
                tone: Some(Tone::Success),
                size: Some(72.0),
                thickness: Some(7.0),
                ..Ring::new(0.72)
            }
            .node(t, now),
            Ring {
                tone: Some(Tone::Info),
                size: Some(40.0),
                thickness: Some(4.0),
                ..Ring::new(1.0)
            }
            .node(t, now),
            Ring::spinning().node(t, now),
            Ring {
                wavy: true,
                size: Some(64.0),
                ..Ring::new(0.6)
            }
            .node(t, now),
            Ring {
                wavy: true,
                size: Some(64.0),
                ..Ring::spinning()
            }
            .node(t, now),
        ]),
        banner(
            Tone::Info,
            "A new version is available.",
            Some("Update"),
            true,
        ),
        banner(Tone::Warning, "Your trial ends in 3 days.", None, true),
        banner(Tone::Danger, "Connection lost. Retrying…", None, false),
    ])
}

fn clock(c: &Ctx) -> Node {
    use crate::widgets::{clock, elapsed, local_offset_minutes, timer};
    row(vec![
        clock(c.theme, 180.0, local_offset_minutes()),
        timer(c.theme, c.values, elapsed()),
    ])
}

fn stopwatch(c: &Ctx) -> Node {
    use crate::widgets::{elapsed, stopwatch, tap_tempo};
    let now = elapsed();
    row(vec![
        stopwatch(c.theme, c.values, now),
        tap_tempo(c.theme, c.values, now),
    ])
}

fn scroll_area(_: &Ctx) -> Node {
    scroll(
        200.0,
        col((1..=40)
            .map(|i| label(format!("Row {i} · drag the thumb or use the wheel")))
            .collect())
        .gap(Gap::Px(8.0)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_has_300_rows_and_the_filter_narrows_them() {
        let rows = table_rows();
        assert_eq!(rows.len(), 300);
        assert_eq!(rows[1][0], "User 037");
        let hits = filter_rows(rows.clone(), "  apac ");
        assert!(!hits.is_empty() && hits.len() < 300);
        assert!(hits.iter().all(|r| r[2] == "APAC"));
        assert_eq!(filter_rows(rows.clone(), "").len(), 300);
        assert!(filter_rows(rows, "nobody").is_empty());
    }

    #[test]
    fn the_sample_picture_is_there() {
        assert!(std::path::Path::new(SAMPLE_IMAGE).exists());
    }
}
