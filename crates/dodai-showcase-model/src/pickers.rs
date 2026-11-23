//! Date and time pickers, one-time-code boxes and form fields built from plain nodes, so every
//! toolkit draws them like the gpui widgets without a widget of its own.

use dodai_core::calendar::{month_grid, Date, MONTHS, WEEKDAYS};
use dodai_core::{ButtonVariant, CardVariant, ControlSize, Tone};
use dodai_icons::IconId;

use crate::b::*;
use crate::catalog::Ctx;
use crate::commands;
use crate::node::{Axis, Font, Gap, Ink, Node, Stack};

/// The month the pickers open on (June 2025, as in the gpui showcase), counted in months.
pub const START_MONTH: usize = 2025 * 12 + 5;

/// Width of one day cell px.
const CELL: f32 = 36.0;

/// What clicking a day does.
#[derive(Debug, Clone, Copy)]
pub enum DayPick<'a> {
    /// Picks one day, stored packed under the key.
    One(&'a str),
    /// Picks a span, stored under the key (see [`commands::range`]).
    Span(&'a str),
}

/// A fixed-width column, children centred.
fn cell(width: f32, items: Vec<Node>) -> Node {
    Node::Stack(Stack {
        axis: Axis::Column,
        wrap: false,
        gap: Gap::Xs,
        centre: true,
        width: Some(width),
        items,
    })
}

fn stack(axis: Axis, gap: Gap, centre: bool, items: Vec<Node>) -> Node {
    Node::Stack(Stack {
        axis,
        wrap: false,
        gap,
        centre,
        width: None,
        items,
    })
}

/// A raised card around `body` (the picker's popup surface).
fn surface(body: Node) -> Node {
    Node::Card {
        title: String::new(),
        body: Box::new(body),
        variant: CardVariant::Elevated,
        surface: dodai_core::Surface::INHERIT,
    }
}

fn icon_button(icon: IconId, cmd: String) -> Node {
    button("", ButtonVariant::Ghost)
        .icon(icon)
        .plain()
        .size(ControlSize::Small)
        .counter(cmd)
        .node()
}

/// `(year, month)` shown by the month key.
pub fn month_of(c: &Ctx, month_key: &str) -> (i32, u32) {
    let n = c.values.index_of(month_key, START_MONTH);
    ((n / 12) as i32, (n % 12 + 1) as u32)
}

/// A month calendar: month title with previous / next, weekday names and the day grid.
pub fn calendar(c: &Ctx, month_key: &str, pick: DayPick) -> Node {
    surface(calendar_body(c, month_key, pick))
}

/// A calendar with a time picker under it (one date and time).
pub fn date_time(
    c: &Ctx,
    month_key: &str,
    pick: DayPick,
    time_key: &str,
    time: (u32, u32),
) -> Node {
    surface(stack(
        Axis::Column,
        Gap::Md,
        true,
        vec![
            calendar_body(c, month_key, pick),
            time_columns(c, time_key, time),
        ],
    ))
}

/// [`calendar`] without its card.
fn calendar_body(c: &Ctx, month_key: &str, pick: DayPick) -> Node {
    let (y, m) = month_of(c, month_key);
    let now = c.values.index_of(month_key, START_MONTH) as i64;
    let nav = |icon, delta| icon_button(icon, commands::step(month_key, delta, 0, 99_999, now));
    let title = format!("{} {y}", MONTHS[(m - 1) as usize]).to_uppercase();
    let head = stack(
        Axis::Row,
        Gap::Md,
        true,
        vec![
            nav(IconId::ChevronLeft, -1),
            cell(CELL * 4.0, vec![styled(title, 16.0, Font::Body, Ink::Text)]),
            nav(IconId::ChevronRight, 1),
        ],
    );
    let names = stack(
        Axis::Row,
        Gap::Xs,
        true,
        WEEKDAYS
            .iter()
            .map(|d| cell(CELL, vec![muted(*d)]))
            .collect(),
    );
    let picked = |d: Date| match pick {
        DayPick::One(key) => commands::unpack(c.values.index_of(key, 0)) == Some(d),
        DayPick::Span(key) => {
            let r = commands::date_range(c.values, key);
            r.start == Some(d) || r.end == Some(d)
        }
    };
    let within = |d: Date| match pick {
        DayPick::One(_) => false,
        DayPick::Span(key) => commands::date_range(c.values, key).contains(d),
    };
    let mut rows = vec![head, names];
    for week in month_grid(y, m) {
        let days = week
            .iter()
            .map(|day| match day {
                None => cell(CELL, vec![]),
                Some(n) => {
                    let d = Date::new(y, m, *n);
                    let variant = if picked(d) {
                        ButtonVariant::Primary
                    } else if within(d) {
                        ButtonVariant::Secondary
                    } else {
                        ButtonVariant::Ghost
                    };
                    let cmd = match pick {
                        DayPick::One(key) => commands::set(key, commands::pack(d)),
                        DayPick::Span(key) => commands::range(key, d),
                    };
                    cell(
                        CELL,
                        vec![button(n.to_string(), variant)
                            .plain()
                            .size(ControlSize::Small)
                            .counter(cmd)
                            .node()],
                    )
                }
            })
            .collect();
        rows.push(stack(Axis::Row, Gap::Xs, true, days));
    }
    stack(Axis::Column, Gap::Sm, true, rows)
}

/// Hour : minute with a step button over and under each, `(hour, minute)` stored under
/// `{key}-h` and `{key}-m`.
pub fn time_picker(c: &Ctx, key: &str, default: (u32, u32)) -> Node {
    surface(time_columns(c, key, default))
}

/// [`time_picker`] without its card.
fn time_columns(c: &Ctx, key: &str, default: (u32, u32)) -> Node {
    let (hk, mk) = (format!("{key}-h"), format!("{key}-m"));
    let h = c.values.index_of(&hk, default.0 as usize);
    let m = c.values.index_of(&mk, default.1 as usize);
    let p = &c.theme.palette;
    let digits = |v: usize| Node::Block {
        w: 64.0,
        h: 52.0,
        fill: p.surface_raised,
        border: Some(p.border),
        radius: c.theme.shape.radius,
        label: Some(format!("{v:02}")),
    };
    let steps = |delta: i64| {
        stack(
            Axis::Row,
            Gap::Sm,
            true,
            vec![
                cell(
                    64.0,
                    vec![icon_button(
                        if delta > 0 {
                            IconId::ChevronUp
                        } else {
                            IconId::ChevronDown
                        },
                        commands::step(&hk, delta, 0, 23, h as i64),
                    )],
                ),
                cell(12.0, vec![]),
                cell(
                    64.0,
                    vec![icon_button(
                        if delta > 0 {
                            IconId::ChevronUp
                        } else {
                            IconId::ChevronDown
                        },
                        commands::step(&mk, delta, 0, 59, m as i64),
                    )],
                ),
            ],
        )
    };
    stack(
        Axis::Column,
        Gap::Sm,
        true,
        vec![
            steps(1),
            stack(
                Axis::Row,
                Gap::Sm,
                true,
                vec![
                    cell(64.0, vec![digits(h)]),
                    cell(12.0, vec![styled(":", 22.0, Font::Body, Ink::Text)]),
                    cell(64.0, vec![digits(m)]),
                ],
            ),
            steps(-1),
        ],
    )
}

/// A calendar with a start and an end time under it, sharing the span `range_key`.
pub fn date_time_range(c: &Ctx, month_key: &str, range_key: &str, key: &str) -> Node {
    let time = |label: &str, k: &str, d| {
        stack(
            Axis::Column,
            Gap::Xs,
            false,
            vec![muted(label), surface(time_columns(c, k, d))],
        )
    };
    surface(stack(
        Axis::Column,
        Gap::Sm,
        true,
        vec![
            calendar(c, month_key, DayPick::Span(range_key)),
            stack(
                Axis::Row,
                Gap::Md,
                false,
                vec![
                    time("Start", &format!("{key}-start"), (9, 0)),
                    time("End", &format!("{key}-end"), (17, 30)),
                ],
            ),
        ],
    ))
}

/// One-time-code boxes: `len` boxes holding `value`'s characters (dots when `masked`).
pub fn otp(c: &Ctx, len: usize, value: &str, masked: bool) -> Node {
    let p = &c.theme.palette;
    let chars: Vec<char> = value.chars().collect();
    stack(
        Axis::Row,
        Gap::Sm,
        true,
        (0..len)
            .map(|i| Node::Block {
                w: 46.0,
                h: 58.0,
                fill: p.background,
                border: Some(p.border_strong),
                radius: c.theme.shape.radius,
                label: chars.get(i).map(|ch| {
                    if masked {
                        "•".to_string()
                    } else {
                        ch.to_string()
                    }
                }),
            })
            .collect(),
    )
}

/// The status line under a field.
#[derive(Debug, Clone, Copy)]
pub enum Note<'a> {
    /// Plain help text.
    Help(&'a str),
    /// A tone with its mark (✓ success, ! warning, ✕ error).
    Status(Tone, &'a str),
}

/// A labelled field: label (with a red star when `required`), the field, its note.
pub fn form_field(label_text: &str, required: bool, field: Node, note: Option<Note>) -> Node {
    let mut head = vec![label(label_text)];
    if required {
        head.push(styled("*", 12.0, Font::Body, Ink::Tone(Tone::Danger)));
    }
    let mut items = vec![stack(Axis::Row, Gap::Xs, true, head), field];
    match note {
        Some(Note::Help(t)) => items.push(styled(t, 11.0, Font::Body, Ink::Muted)),
        Some(Note::Status(tone, t)) => {
            let mark = match tone {
                Tone::Success => "✓",
                Tone::Danger => "✕",
                _ => "!",
            };
            items.push(styled(
                format!("{mark} {t}"),
                11.0,
                Font::Body,
                Ink::Tone(tone),
            ));
        }
        None => {}
    }
    stack(Axis::Column, Gap::Xs, false, items)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::Values;

    fn ctx_with<R>(values: &Values, f: impl FnOnce(&Ctx) -> R) -> R {
        let theme = dodai_core::themes::all().remove(0);
        f(&Ctx {
            theme: &theme,
            values,
        })
    }

    #[test]
    fn the_calendar_opens_on_june_2025_and_steps_months() {
        let mut v = Values::default();
        assert_eq!(ctx_with(&v, |c| month_of(c, "cal")), (2025, 6));
        crate::commands::press(
            &mut v,
            &commands::step("cal", 7, 0, 99_999, START_MONTH as i64),
        );
        assert_eq!(ctx_with(&v, |c| month_of(c, "cal")), (2026, 1));
    }
}
