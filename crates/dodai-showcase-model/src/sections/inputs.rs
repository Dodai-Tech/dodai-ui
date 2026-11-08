//! Inputs: toggles, sliders, text fields.

use dodai_core::ToggleKind::{Checkbox, Radio, Switch};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

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

fn toggles(_: &Ctx) -> Node {
    col(vec![
        row(vec![
            toggle(Switch, "sw-a", true, "Notifications").node(),
            toggle(Switch, "sw-b", false, "Dark mode").node(),
            toggle(Switch, "sw-c", true, "Disabled").disabled().node(),
        ]),
        row(vec![
            toggle(Checkbox, "ck-a", true, "Accept terms").node(),
            toggle(Checkbox, "ck-b", false, "Subscribe").node(),
            toggle(Checkbox, "ck-c", false, "Disabled")
                .disabled()
                .node(),
        ]),
        row(vec![
            toggle(Radio, "rd-0", true, "Compact")
                .in_group("radio", 0)
                .node(),
            toggle(Radio, "rd-1", false, "Comfortable")
                .in_group("radio", 1)
                .node(),
            toggle(Radio, "rd-2", false, "Spacious")
                .in_group("radio", 2)
                .node(),
        ]),
    ])
}

fn sliders(_: &Ctx) -> Node {
    col(vec![
        slider("sl-a", 40.0, 0.0, 100.0).node(),
        slider("sl-b", 6.0, 0.0, 10.0).step(1.0).node(),
        Node::Progress {
            value: 0.65,
            tone: None,
        },
        Node::Progress {
            value: 0.35,
            tone: Some(dodai_core::Tone::Warning),
        },
        Node::Progress {
            value: 0.9,
            tone: Some(dodai_core::Tone::Success),
        },
    ])
}

fn inputs(_: &Ctx) -> Node {
    col(vec![
        input("in-name", "handle…").node(),
        input("in-mail", "email")
            .label("Email")
            .value("ada@example.com")
            .node(),
        input("in-notes", "notes (multi-line)…").lines(4).node(),
    ])
}

fn search(_: &Ctx) -> Node {
    input("search", "Search…").icon(IconId::Search).node()
}

fn password(_: &Ctx) -> Node {
    col(vec![
        input("pw", "password").password().node(),
        input("otp", "6-digit code").node(),
    ])
}

fn checkbox_variants(_: &Ctx) -> Node {
    col(vec![
        row(vec![
            toggle(Checkbox, "cv-a", true, "Checked").node(),
            toggle(Checkbox, "cv-b", false, "Unchecked").node(),
            toggle(Checkbox, "cv-c", true, "Disabled checked")
                .disabled()
                .node(),
            toggle(Checkbox, "cv-d", false, "Disabled")
                .disabled()
                .node(),
        ]),
        row(vec![
            toggle(Switch, "cv-e", true, "On").node(),
            toggle(Switch, "cv-f", false, "Off").node(),
        ]),
    ])
}

fn slider_stepped(_: &Ctx) -> Node {
    col(vec![
        muted("Steps of 10"),
        slider("ss-a", 40.0, 0.0, 100.0).step(10.0).node(),
        muted("Steps of 0.5"),
        slider("ss-b", 2.5, 0.0, 5.0).step(0.5).node(),
        muted("Continuous"),
        slider("ss-c", 0.3, 0.0, 1.0).node(),
    ])
}

fn form_field(_: &Ctx) -> Node {
    col(vec![
        input("ff-name", "Full name")
            .label("Name")
            .value("Ada Lovelace")
            .node(),
        muted("As it appears on your passport."),
        input("ff-mail", "you@example.com").label("Email").node(),
        alert(dodai_core::Tone::Danger, "Enter a valid address.", false),
        row(vec![
            button("Cancel", dodai_core::ButtonVariant::Ghost).node(),
            button("Save", dodai_core::ButtonVariant::Primary).node(),
        ]),
    ])
}

fn groups(_: &Ctx) -> Node {
    let chip = |label: &str, key: &str, on: bool| chip(label, key, on).node();
    col(vec![
        muted("Toggle group"),
        row(vec![
            chip("Bold", "tg-b", true),
            chip("Italic", "tg-i", false),
            chip("Underline", "tg-u", false),
        ]),
        muted("Checkbox group"),
        col(vec![
            toggle(Checkbox, "cg-a", true, "Email").node(),
            toggle(Checkbox, "cg-b", true, "Push").node(),
            toggle(Checkbox, "cg-c", false, "SMS").node(),
        ]),
    ])
}

fn tag_field(_: &Ctx) -> Node {
    col(vec![
        row(vec![
            chip("rust", "tf-1", true).node(),
            chip("gpui", "tf-2", true).node(),
            chip("design", "tf-3", true).node(),
        ]),
        input("tf-new", "Add a tag…").node(),
    ])
}

fn pickers(_: &Ctx) -> Node {
    col(vec![
        input("dp-date", "YYYY-MM-DD")
            .label("Date")
            .value("2026-11-08")
            .icon(IconId::Calendar)
            .node(),
        input("dp-time", "HH:MM")
            .label("Time")
            .value("14:30")
            .node(),
        Node::Choice(
            choice(
                "dp-day",
                &["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
                6,
            )
            .connected(),
        ),
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

fn dials(c: &Ctx) -> Node {
    let readout =
        |key: &str, d: f32| body(format!("{:.2}", c.values.num(&crate::Bind::new(key, d))));
    let dial = |n: Node, key: &str, d: f32| crate::b::dial(vec![n, readout(key, d)]);
    let mut items = vec![
        dial(knob("knob-0", 0.3, 0.0, 1.0, 72.0), "knob-0", 0.3),
        dial(knob("knob-1", 0.65, 0.0, 1.0, 96.0), "knob-1", 0.65),
        dial(
            Node::Knob {
                bind: crate::Bind::new("knob-2", 0.5),
                min: 0.0,
                max: 1.0,
                size: 72.0,
                disabled: true,
            },
            "knob-2",
            0.5,
        ),
    ];
    for (i, d) in [0.2_f32, 0.55, 0.9].into_iter().enumerate() {
        let key = format!("vs-{i}");
        items.push(dial(
            slider(&key, d, 0.0, 1.0).vertical().readout(false).node(),
            &key,
            d,
        ));
    }
    row(items)
}

fn range_slider(c: &Ctx) -> Node {
    let lo = crate::Bind::new("rs-lo", 20.0);
    let hi = crate::Bind::new("rs-hi", 70.0);
    let (a, b) = ordered(c.values.num(&lo), c.values.num(&hi));
    col(vec![
        muted("Two handles: the range is the lower and the higher of them."),
        slider("rs-lo", 20.0, 0.0, 100.0).step(5.0).node(),
        slider("rs-hi", 70.0, 0.0, 100.0).step(5.0).node(),
        body(format!("Selected range: {a:.0} to {b:.0}")),
    ])
}

fn date_range(_: &Ctx) -> Node {
    row(vec![
        input("dr-from", "YYYY-MM-DD")
            .label("From")
            .value("2026-11-01")
            .icon(IconId::Calendar)
            .node(),
        input("dr-to", "YYYY-MM-DD")
            .label("To")
            .value("2026-11-30")
            .icon(IconId::Calendar)
            .node(),
    ])
}

fn date_time_range(_: &Ctx) -> Node {
    col(vec![
        row(vec![
            input("dtr-from", "YYYY-MM-DD")
                .label("Start date")
                .value("2026-11-08")
                .node(),
            input("dtr-from-t", "HH:MM")
                .label("Start time")
                .value("09:00")
                .node(),
        ]),
        row(vec![
            input("dtr-to", "YYYY-MM-DD")
                .label("End date")
                .value("2026-11-08")
                .node(),
            input("dtr-to-t", "HH:MM")
                .label("End time")
                .value("17:30")
                .node(),
        ]),
    ])
}

#[cfg(test)]
mod tests {
    use super::ordered;

    #[test]
    fn the_range_is_ordered_whichever_handle_is_higher() {
        assert_eq!(ordered(20.0, 70.0), (20.0, 70.0));
        assert_eq!(ordered(70.0, 20.0), (20.0, 70.0));
        assert_eq!(ordered(5.0, 5.0), (5.0, 5.0));
    }
}
