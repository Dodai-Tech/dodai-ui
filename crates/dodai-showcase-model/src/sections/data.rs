//! Data display: tags, chips, lists, rating, timeline, tooltips, tables, code, progress.

use dodai_core::{CardVariant, Tone};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::{Bind, Node, TextRole, TimelineEvent};

const CAT: &str = "Data display";

pub(crate) const SECTIONS: [Section; 13] = [
    Section {
        category: CAT,
        title: "Tags",
        build: tags,
    },
    Section {
        category: CAT,
        title: "List items · Chips",
        build: lists_chips,
    },
    Section {
        category: CAT,
        title: "Rating · Stepper · Badges · Tooltips",
        build: misc,
    },
    Section {
        category: CAT,
        title: "List · Divider",
        build: list_divider,
    },
    Section {
        category: CAT,
        title: "Timeline · empty state",
        build: timeline,
    },
    Section {
        category: CAT,
        title: "Tooltip",
        build: tooltip,
    },
    Section {
        category: CAT,
        title: "Data table",
        build: table,
    },
    Section {
        category: CAT,
        title: "Code block · Markdown",
        build: code,
    },
    Section {
        category: CAT,
        title: "Image · Avatar group",
        build: avatars,
    },
    Section {
        category: CAT,
        title: "Circular progress · Banner",
        build: progress_banner,
    },
    Section {
        category: CAT,
        title: "Cards",
        build: cards,
    },
    Section {
        category: CAT,
        title: "Clock · Timer",
        build: clock,
    },
    Section {
        category: CAT,
        title: "Stopwatch · Tap tempo",
        build: stopwatch,
    },
];

const TONES: [(&str, Tone); 4] = [
    ("online", Tone::Success),
    ("warn", Tone::Warning),
    ("error", Tone::Danger),
    ("info", Tone::Info),
];

fn tags(_: &Ctx) -> Node {
    let mut v: Vec<Node> = TONES.iter().map(|(l, t)| tag(*l, *t, true)).collect();
    v.extend(TONES.iter().map(|(l, t)| tag(*l, *t, false)));
    row(v)
}

fn lists_chips(_: &Ctx) -> Node {
    let chip = |label: &str, key: &str, on: bool, icon: Option<IconId>| Node::Chip {
        label: label.into(),
        bind: Bind::new(key, on),
        icon,
    };
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
        row(vec![
            chip("Assist", "chip-assist", false, Some(IconId::Edit)),
            chip("Filter", "chip-off", false, None),
            chip("Filter", "chip-on", true, None),
            chip("Input", "chip-input", false, Some(IconId::Heart)),
            chip("Suggestion", "chip-sugg", false, None),
        ]),
    ])
}

fn misc(_: &Ctx) -> Node {
    row(vec![
        Node::Rating { value: 3.5, max: 5 },
        Node::Stepper(Bind::new("stepper", 3)),
        Node::Tooltip {
            text: "Rich tooltip".into(),
            rich: true,
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
        Node::List(
            ["Inbox", "Drafts", "Sent", "Archive"]
                .into_iter()
                .map(list_row)
                .collect(),
        ),
        Node::Divider,
        muted("Below the divider"),
    ])
}

fn timeline(_: &Ctx) -> Node {
    let ev = |title: &str, time: &str, detail: Option<&str>, tone: Option<Tone>| TimelineEvent {
        title: title.into(),
        time: time.into(),
        detail: detail.map(str::to_string),
        tone,
    };
    col(vec![
        Node::Timeline(vec![
            ev("Charts added", "today", None, Some(Tone::Success)),
            ev("Search view", "today", Some("fuzzy + highlight"), None),
            ev("Build failed", "earlier", None, Some(Tone::Danger)),
        ]),
        Node::Empty {
            title: "Nothing here yet".into(),
            detail: "Items you add will show up in this list.".into(),
        },
    ])
}

fn tooltip(_: &Ctx) -> Node {
    row(vec![
        Node::Tooltip {
            text: "Plain tooltip".into(),
            rich: false,
        },
        Node::Tooltip {
            text: "Title, body and an action.".into(),
            rich: true,
        },
    ])
}

fn table(_: &Ctx) -> Node {
    let r = |a: &str, b: &str, c: &str, d: &str| vec![a.into(), b.into(), c.into(), d.into()];
    Node::Table {
        columns: ["Name", "Role", "Status", "Load"]
            .map(String::from)
            .to_vec(),
        rows: vec![
            r("Ada Lovelace", "Engineer", "Online", "72%"),
            r("Grace Hopper", "Admiral", "Away", "41%"),
            r("Alan Turing", "Researcher", "Online", "88%"),
            r("Linus Torvalds", "Maintainer", "Offline", "13%"),
            r("Margaret Hamilton", "Lead", "Online", "57%"),
        ],
    }
}

fn code(_: &Ctx) -> Node {
    col(vec![
        Node::Code("fn main() {\n    println!(\"hello, dodai\");\n}".into()),
        text(
            "Markdown renders **bold**, _italic_ and `code` inline.",
            TextRole::Body,
        ),
    ])
}

fn avatars(_: &Ctx) -> Node {
    row(["AL", "GH", "AT", "LT", "MH"]
        .map(|s| Node::Avatar(s.into()))
        .to_vec())
}

fn progress_banner(_: &Ctx) -> Node {
    col(vec![
        row(vec![Node::Spinner(32.0), Node::Spinner(48.0)]),
        Node::Alert {
            tone: Tone::Info,
            text: "A new version is available.".into(),
            banner: true,
        },
        Node::Alert {
            tone: Tone::Danger,
            text: "Could not reach the server.".into(),
            banner: true,
        },
    ])
}

fn cards(_: &Ctx) -> Node {
    let card = |title: &str, variant| Node::Card {
        title: title.into(),
        body: Box::new(muted("Supporting text of the card.")),
        variant,
    };
    row(vec![
        card("Elevated", CardVariant::Elevated),
        card("Filled", CardVariant::Filled),
        card("Outlined", CardVariant::Outlined),
    ])
}

fn clock(_: &Ctx) -> Node {
    row(vec![
        text(
            "14:32:07",
            TextRole::Type(dodai_core::TypeRole::DisplaySmall),
        ),
        Node::Tag {
            label: "UTC+1".into(),
            tone: Tone::Info,
            solid: false,
        },
        text(
            "04:59",
            TextRole::Type(dodai_core::TypeRole::HeadlineMedium),
        ),
        muted("timer"),
    ])
}

fn stopwatch(c: &Ctx) -> Node {
    let taps = c.values.index_of("tap-tempo", 0);
    row(vec![
        text(
            "00:12.4",
            TextRole::Type(dodai_core::TypeRole::DisplaySmall),
        ),
        button("Start", dodai_core::ButtonVariant::Primary).node(),
        button("Lap", dodai_core::ButtonVariant::Outline).node(),
        button("Tap", dodai_core::ButtonVariant::Secondary)
            .counter("tap-tempo")
            .node(),
        muted(format!("{taps} taps")),
    ])
}
