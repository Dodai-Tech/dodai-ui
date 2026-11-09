//! Inputs: toggles, sliders, text fields, pickers - the gpui showcase's demos and values.

use dodai_core::ToggleKind::{self, Checkbox, Radio, Switch};
use dodai_core::Tone;
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::commands;
use crate::node::{Font, Ink, Node, SliderFormat};
use crate::pickers::{self, DayPick, Note};
use crate::Bind;

const CAT: &str = "Inputs";

pub(crate) const SECTIONS: [Section; 15] = [
    Section::new(CAT, "Toggle · Checkbox · Radio", toggles),
    Section::new(CAT, "Checkbox variants", checkbox_variants).when(|t| t.look().tonal),
    Section::new(
        CAT,
        "Toggle group · Checkbox group · select all · switches",
        groups,
    )
    .heading("Toggle group · Checkbox group"),
    Section::new(CAT, "Slider · Progress", sliders),
    Section::new(
        CAT,
        "Stepped slider · ticks · formatted value",
        slider_stepped,
    )
    .heading("Slider (stepped, ticks, value readout)"),
    Section::new(CAT, "Range slider", range_slider),
    Section::new(CAT, "Knob · Vertical slider · Vertical range slider", dials)
        .heading("Knob · Vertical sliders"),
    Section::new(CAT, "Input field", inputs).heading("Input"),
    Section::new(CAT, "Search view · SearchBar", search).heading("Search view"),
    Section::new(CAT, "Tag field · chips", tag_field).heading("Tag field"),
    Section::new(CAT, "Password · One-time code", password).heading("Password · OTP"),
    Section::new(CAT, "Form field · label help error validation", form_field).heading("Form field"),
    Section::new(CAT, "Date picker · Time picker", pickers).heading("Date / time pickers"),
    Section::new(CAT, "Date range picker", date_range).heading("Date range"),
    Section::new(CAT, "Date-time range picker", date_time_range).heading("Date-time range"),
];

/// The animations a switch, checkbox and radio can move with.
pub(crate) const ANIMATIONS: [&str; 5] = ["Design's own", "Slide", "Stretch", "Glitch", "Lift"];

fn toggles(_: &Ctx) -> Node {
    let radio = |i: usize, name: &str| {
        toggle(Radio, &format!("radio-{i}"), i == 0, name)
            .in_group("radio", i)
            .node()
    };
    col(vec![
        muted("Animation"),
        Node::Choice(choice("choice-animation", &ANIMATIONS, 0).connected()),
        row(vec![
            toggle(Switch, "sw", true, "").node(),
            label("Switch"),
            toggle(Switch, "sw-icons", true, "").icons().node(),
            toggle(Switch, "sw-icons-off", false, "").icons().node(),
            label("With icons"),
            toggle(Switch, "sw-dis", true, "").disabled().node(),
            toggle(Checkbox, "ck", true, "").node(),
            label("Checkbox"),
            toggle(Checkbox, "ck-dis", false, "").disabled().node(),
        ]),
        row(vec![
            radio(0, ""),
            label("Compact"),
            radio(1, ""),
            label("Comfortable"),
            radio(2, ""),
            label("Spacious"),
        ]),
    ])
}

/// Material 3's checkbox sizes and shapes (other designs draw their own box).
const VARIANTS: [&str; 6] = [
    "Standard 40",
    "Compact 24",
    "Dense 18",
    "Large 48",
    "Round",
    "Tonal",
];

fn checkbox_variants(_: &Ctx) -> Node {
    col(VARIANTS
        .iter()
        .map(|name| {
            let id = |k: &str| format!("cv-{name}-{k}");
            let cell = |n: Node| tile(44.0, vec![n]);
            line(vec![
                tile(110.0, vec![label(*name)]),
                cell(toggle(Checkbox, &id("off"), false, "").node()),
                cell(toggle(Checkbox, &id("on"), true, "").node()),
                cell(toggle(Checkbox, &id("mix"), false, "").mixed(true).node()),
                cell(toggle(Checkbox, &id("dis"), true, "").disabled().node()),
                cell(
                    toggle(Radio, &id("ra"), true, "")
                        .in_group(&id("ra"), 0)
                        .node(),
                ),
            ])
        })
        .collect())
}

/// A group of checkboxes or switches under a "select all" that reads "n of m".
fn check_group(
    c: &Ctx,
    key: &str,
    all_label: &str,
    kind: ToggleKind,
    items: &[&str],
    on: &[usize],
) -> Node {
    let keys: Vec<String> = (0..items.len()).map(|i| format!("{key}-{i}")).collect();
    let picked: Vec<bool> = keys
        .iter()
        .enumerate()
        .map(|(i, k)| c.values.flag(&Bind::new(k, on.contains(&i))))
        .collect();
    let n = picked.iter().filter(|p| **p).count();
    let all = n == items.len();
    let head = line(vec![
        toggle(kind, &commands::all(&keys), all, all_label)
            .mixed(n > 0 && !all)
            .node(),
        styled(
            format!("{n} of {}", items.len()),
            11.0,
            Font::Body,
            Ink::Muted,
        ),
    ]);
    let mut rows = vec![head];
    for (i, item) in items.iter().enumerate() {
        rows.push(line(vec![
            Node::Space(20.0),
            toggle(kind, &keys[i], on.contains(&i), *item).node(),
        ]));
    }
    tile(220.0, rows)
}

fn groups(c: &Ctx) -> Node {
    let format = |label: &str, key: &str, on: bool| chip(label, key, on).node();
    col(vec![
        row(vec![
            Node::Choice(choice("tg1", &["Left", "Center", "Right"], 0).connected()),
            line(vec![
                format("Bold", "tg2-0", true),
                format("Italic", "tg2-1", false),
                format("Underline", "tg2-2", true),
            ]),
        ]),
        row(vec![
            check_group(
                c,
                "cg",
                "All channels",
                Checkbox,
                &["Email", "SMS", "Push", "Slack"],
                &[0, 2],
            ),
            check_group(
                c,
                "sg",
                "Everything",
                Switch,
                &["Wi-Fi", "Bluetooth", "Location"],
                &[0],
            ),
        ]),
    ])
}

fn sliders(c: &Ctx) -> Node {
    let v = c.values.num(&Bind::new("slider", 0.4));
    col(vec![
        slider("slider", 0.4, 0.0, 1.0).readout(false).node(),
        styled(format!("value {v:.2}"), 13.0, Font::Mono, Ink::Text),
        muted("Disabled · arrow keys move a focused slider"),
        slider("slider-off", 0.4, 0.0, 1.0)
            .readout(false)
            .disabled()
            .node(),
        Node::Progress {
            value: v,
            tone: None,
        },
        Node::Progress {
            value: 0.7,
            tone: Some(Tone::Success),
        },
        Node::Progress {
            value: 0.4,
            tone: Some(Tone::Warning),
        },
        Node::Progress {
            value: 0.55,
            tone: None,
        },
        Node::Progress {
            value: 0.15,
            tone: Some(Tone::Danger),
        },
    ])
}

fn slider_stepped(_: &Ctx) -> Node {
    col(vec![
        slider("bub-up", 35.0, 0.0, 100.0)
            .format(SliderFormat::Percent)
            .bubble(true)
            .node(),
        slider("bub-down", 60.0, 0.0, 100.0)
            .step(10.0)
            .ticks()
            .format(SliderFormat::Percent)
            .bubble(false)
            .node(),
        slider("stepped", 4.0, 0.0, 10.0).step(1.0).ticks().node(),
        slider("db", -6.0, -24.0, 6.0)
            .step(3.0)
            .ticks()
            .format(SliderFormat::Decibel)
            .node(),
        slider("slider", 0.4, 0.0, 1.0)
            .centered()
            .format(SliderFormat::Balance)
            .node(),
        slider("slider", 0.4, 0.0, 1.0)
            .icon(IconId::Heart)
            .readout(false)
            .node(),
    ])
}

/// The ordered `(low, high)` of a pair of range handles, whichever way round they were set.
pub(crate) fn ordered(a: f32, b: f32) -> (f32, f32) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

fn range_slider(c: &Ctx) -> Node {
    let lo = c.values.num(&Bind::new("range-lo", 0.2));
    let hi = c.values.num(&Bind::new("range-hi", 0.7));
    let (a, b) = ordered(lo, hi);
    col(vec![
        range("range", (0.2, 0.7), 0.0, 1.0).node(),
        muted(format!("range {a:.2} – {b:.2}")),
    ])
}

fn dials(c: &Ctx) -> Node {
    let mono = |s: String| styled(s, 13.0, Font::Mono, Ink::Text);
    let readout = |key: &str, d: f32| mono(format!("{:.2}", c.values.num(&Bind::new(key, d))));
    let mut items = Vec::new();
    for (i, d) in [0.35_f32, 0.7, 0.5].into_iter().enumerate() {
        let key = format!("knob-{i}");
        let size = if i == 1 { 96.0 } else { 72.0 };
        let k = Node::Knob {
            bind: Bind::new(&key, d),
            min: 0.0,
            max: 1.0,
            size,
            disabled: i == 2,
        };
        items.push(dial(vec![k, readout(&key, d)]));
    }
    for (i, d) in [0.3_f32, 0.6, 0.9].into_iter().enumerate() {
        let key = format!("vslider-{i}");
        items.push(dial(vec![
            slider(&key, d, 0.0, 1.0)
                .vertical()
                .length(120.0)
                .readout(false)
                .node(),
            readout(&key, d),
        ]));
    }
    let lo = c.values.num(&Bind::new("vrange-lo", 0.2));
    let hi = c.values.num(&Bind::new("vrange-hi", 0.8));
    let (a, b) = ordered(lo, hi);
    items.push(dial(vec![
        range("vrange", (0.2, 0.8), 0.0, 1.0)
            .step(0.1)
            .ticks()
            .vertical(120.0)
            .node(),
        mono(format!("{a:.1}–{b:.1}")),
    ]));
    row(items)
}

fn inputs(_: &Ctx) -> Node {
    col(vec![
        input("in-name", "handle…").value("Ada Lovelace").node(),
        input("in-notes", "notes (multi-line)…").lines(4).node(),
        input("in-static", "static placeholder box").node(),
        input("in-outlined", "")
            .label("Email")
            .value("ada@dodai.dev")
            .node(),
        input("in-filled", "")
            .label("Name")
            .value("analytical engine")
            .filled()
            .node(),
    ])
}

/// The suggestions both search views filter.
const SUGGESTIONS: [&str; 6] = ["Alpha", "Bravo", "Charlie", "Delta", "Echo", "Foxtrot"];

/// The suggestions holding `query` (case-insensitive); none for an empty query.
pub(crate) fn matches(query: &str) -> Vec<&'static str> {
    let q = query.to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    SUGGESTIONS
        .into_iter()
        .filter(|s| s.to_lowercase().contains(&q))
        .collect()
}

fn results(query: &str, open: bool) -> Option<Node> {
    if !open {
        return None;
    }
    let found = matches(query);
    Some(if found.is_empty() {
        Node::List(vec![list_row("No results")])
    } else {
        Node::List(found.into_iter().map(list_row).collect())
    })
}

fn search(c: &Ctx) -> Node {
    let bar = Bind::new("search", "sl".to_string());
    let full = Bind::new("search-full", "a".to_string());
    let query = c.values.text(&bar).to_string();
    let full_query = c.values.text(&full).to_string();
    let mut items = vec![input("search", "Search…")
        .value("sl")
        .icon(IconId::MagnifyingGlass)
        .node()];
    items.extend(results(&query, !query.is_empty()));
    items.push(muted("Full screen (Material)"));
    let mut full_view = vec![input("search-full", "Search…")
        .value("a")
        .icon(IconId::MagnifyingGlass)
        .node()];
    full_view.extend(results(&full_query, true));
    items.push(col(full_view));
    col(items)
}

fn tag_field(c: &Ctx) -> Node {
    let tags = ["rust", "gpui", "charts"];
    let chips = tags
        .iter()
        .enumerate()
        .filter(|(i, _)| c.values.flag(&Bind::new(format!("tags-{i}"), true)))
        .map(|(i, t)| {
            chip(t.to_uppercase(), &format!("tags-{i}"), true)
                .removable()
                .node()
        })
        .collect();
    Node::Card {
        title: String::new(),
        body: Box::new(line(chips)),
        variant: dodai_core::CardVariant::Outlined,
        surface: dodai_core::Surface::INHERIT,
    }
}

fn password(c: &Ctx) -> Node {
    col(vec![
        pickers::form_field(
            "Password",
            true,
            input("pw", "password").password().node(),
            Some(Note::Help("Shown as dots; copy is disabled")),
        ),
        pickers::form_field(
            "Verification code",
            false,
            pickers::otp(c, 6, "381946", false),
            Some(Note::Help("Type or paste 6 digits")),
        ),
        pickers::form_field(
            "PIN (masked)",
            false,
            pickers::otp(c, 4, "4821", true),
            None,
        ),
    ])
}

/// Whether `mail` looks like an address (`a@b.cd`).
pub(crate) fn valid_email(mail: &str) -> bool {
    mail.contains('@')
        && mail.contains('.')
        && mail.rsplit('.').next().is_some_and(|d| d.len() >= 2)
}

fn form_field(c: &Ctx) -> Node {
    let mail_bind = Bind::new("ff-mail", "ada@dodai.dev".to_string());
    let mail = c.values.text(&mail_bind).to_string();
    let mail_note = if valid_email(&mail) {
        Some(Note::Status(Tone::Success, "Looks good"))
    } else if mail.is_empty() {
        None
    } else {
        Some(Note::Status(Tone::Danger, "Enter a valid address"))
    };
    let field = |n: Node| tile(260.0, vec![n]);
    row(vec![
        field(pickers::form_field(
            "Name",
            true,
            input("ff-name", "Ada Lovelace").node(),
            Some(Note::Help("As on your ID")),
        )),
        field(pickers::form_field(
            "Email",
            false,
            input("ff-mail", "name@").value("ada@dodai.dev").node(),
            mail_note,
        )),
        field(pickers::form_field(
            "Username",
            false,
            input("ff-user", "ada").node(),
            Some(Note::Status(Tone::Success, "Available")),
        )),
        field(pickers::form_field(
            "Bio",
            false,
            input("ff-bio", "…").node(),
            Some(Note::Status(Tone::Warning, "Over 80% of the limit")),
        )),
    ])
}

fn pickers(c: &Ctx) -> Node {
    let picked = commands::unpack(c.values.index_of("date", 0));
    let h = c.values.index_of("time-h", 9);
    let m = c.values.index_of("time-m", 30);
    let date_text = picked.map_or(String::new(), |d| {
        format!("{}-{:02}-{:02}", d.year, d.month, d.day)
    });
    col(vec![
        row(vec![
            pickers::calendar(c, "date-month", DayPick::One("date")),
            pickers::time_picker(c, "time", (9, 30)),
            pickers::date_time(c, "date-month", DayPick::One("date"), "time", (9, 30)),
        ]),
        muted("Text input mode"),
        row(vec![
            input("date-in", "YYYY-MM-DD")
                .label("Date")
                .value(&date_text)
                .icon(IconId::Calendar)
                .node(),
            input("time-in", "HH:MM")
                .label("Time")
                .value(format!("{h:02}:{m:02}"))
                .icon(IconId::Clock)
                .node(),
        ]),
    ])
}

fn date_range(c: &Ctx) -> Node {
    let r = commands::date_range(c.values, "date-range");
    let fmt = |d: dodai_core::calendar::Date| format!("{}-{:02}-{:02}", d.year, d.month, d.day);
    let hint = match (r.start, r.end) {
        (Some(a), Some(b)) => format!("{} → {}", fmt(a), fmt(b)),
        (Some(a), None) => format!("from {}: pick the end", fmt(a)),
        _ => "click a start day, then an end day".to_string(),
    };
    row(vec![
        pickers::calendar(c, "date-month", DayPick::Span("date-range")),
        muted(hint),
    ])
}

fn date_time_range(c: &Ctx) -> Node {
    col(vec![
        // In a row so the card is only as wide as the picker.
        row(vec![pickers::date_time_range(
            c,
            "date-month",
            "date-range",
            "range-time",
        )]),
        muted("Shares its span with the date range above; the start and end times are its own."),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_range_is_ordered_whichever_handle_is_higher() {
        assert_eq!(ordered(20.0, 70.0), (20.0, 70.0));
        assert_eq!(ordered(70.0, 20.0), (20.0, 70.0));
        assert_eq!(ordered(5.0, 5.0), (5.0, 5.0));
    }

    #[test]
    fn search_suggestions_follow_the_query() {
        assert_eq!(matches("a").len(), 4, "Alpha Bravo Charlie Delta");
        assert!(matches("sl").is_empty());
        assert!(matches("").is_empty());
    }

    #[test]
    fn email_validation() {
        assert!(valid_email("ada@dodai.dev"));
        assert!(!valid_email("ada@dodai"));
        assert!(!valid_email("ada"));
    }
}
