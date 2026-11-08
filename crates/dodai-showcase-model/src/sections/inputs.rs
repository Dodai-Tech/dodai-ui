//! Inputs: toggles, sliders, text fields.

use dodai_core::ToggleKind::{Checkbox, Radio, Switch};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::{Ctx, Section};
use crate::node::Node;

const CAT: &str = "Inputs";

pub(crate) const SECTIONS: [Section; 5] = [
    Section {
        category: CAT,
        title: "Toggle · Checkbox · Radio",
        build: toggles,
    },
    Section {
        category: CAT,
        title: "Slider · Progress",
        build: sliders,
    },
    Section {
        category: CAT,
        title: "Input",
        build: inputs,
    },
    Section {
        category: CAT,
        title: "Search view",
        build: search,
    },
    Section {
        category: CAT,
        title: "Password · OTP",
        build: password,
    },
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
