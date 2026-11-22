//! Dates and times: calendar and pickers, clock, stopwatch, timer and tap tempo, on the shared
//! calendar and clock maths of `dodai-ui-core`. Pickers are controlled; the clocks that count
//! take the elapsed time from the caller so the host owns the timer.

use crate::{Icon, KitBox};
use dioxus::prelude::*;
use dodai_ui_core::calendar::{self as cal, Date, DateRange};
use dodai_ui_core::clock_math as cm;
use dodai_ui_core::Kit;
use dodai_ui_icons::IconId;

const CELL: &str = "display:grid;place-items:center;width:var(--dodai-control-height);height:calc(var(--dodai-control-height) * .75);border:0;background:transparent;color:inherit;font:inherit;border-radius:var(--dodai-radius);cursor:pointer";

/// Properties of [`DatePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct DatePickerProps {
    pub value: Date,
    #[props(default)]
    pub onchange: EventHandler<Date>,
}

/// A month grid with previous and next arrows; picks one day. Controlled.
#[component]
pub fn DatePicker(props: DatePickerProps) -> Element {
    let v = props.value;
    let on = props.onchange;
    rsx! {
        CalendarGrid { year: v.year, month: v.month, picked: DateRange { start: Some(v), end: Some(v) },
            onpick: move |d| on.call(d) }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CalendarGridProps {
    year: i32,
    month: u32,
    picked: DateRange,
    onpick: EventHandler<Date>,
}

#[component]
fn CalendarGrid(props: CalendarGridProps) -> Element {
    let mut shown = use_signal(|| (props.year, props.month));
    let (y, m) = shown();
    let rows = cal::month_grid(y, m);
    let pick = props.onpick;
    let picked = props.picked;
    rsx! {
        KitBox { kit: Kit::Popup, style: "flex-direction:column;align-items:stretch;gap:var(--dodai-space-xs);width:fit-content".to_string(),
            div { style: "display:flex;align-items:center;justify-content:space-between",
                crate::IconButton { icon: IconId::ChevronLeft, label: "Previous month", onclick: move |_| shown.set(cal::shift_month(y, m, -1)) }
                span { style: "font-family:var(--dodai-font-display)", "{cal::MONTHS[(m - 1) as usize]} {y}" }
                crate::IconButton { icon: IconId::ChevronRight, label: "Next month", onclick: move |_| shown.set(cal::shift_month(y, m, 1)) }
            }
            div { style: "display:grid;grid-template-columns:repeat(7,auto);justify-items:center",
                for w in cal::WEEKDAYS {
                    span { key: "{w}", style: "font-size:var(--dodai-size-xs);color:var(--dodai-text-muted);padding:var(--dodai-space-xs)", "{w}" }
                }
                for (r, row) in rows.iter().enumerate() {
                    for (c, cell) in row.iter().enumerate() {
                        match cell {
                            Some(day) => {
                                let d = Date::new(y, m, *day);
                                let on = picked.contains(d);
                                let style = format!(
                                    "{CELL};{}",
                                    if on { "background:var(--dodai-primary);color:var(--dodai-background)" } else { "" }
                                );
                                rsx! { button { key: "{r}-{c}", style, aria_pressed: "{on}", onclick: move |_| pick.call(d), "{day}" } }
                            }
                            None => rsx! { span { key: "{r}-{c}" } },
                        }
                    }
                }
            }
        }
    }
}

/// Properties of [`DateInput`].
#[derive(Props, Clone, PartialEq)]
pub struct DateInputProps {
    pub value: Date,
    #[props(default)]
    pub onchange: EventHandler<Date>,
}

fn parse_date(s: &str) -> Option<Date> {
    let mut p = s.split('-');
    let (y, m, d) = (
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
        p.next()?.parse().ok()?,
    );
    (1..=12).contains(&m).then_some(())?;
    (1..=cal::days_in_month(y, m)).contains(&d).then_some(())?;
    Some(Date::new(y, m, d))
}

/// A typed date field (`YYYY-MM-DD`); only a valid date is reported. Controlled.
#[component]
pub fn DateInput(props: DateInputProps) -> Element {
    let v = props.value;
    let on = props.onchange;
    rsx! {
        input { class: "dodai-input", r#type: "date", aria_label: "Date",
            value: "{v.year:04}-{v.month:02}-{v.day:02}",
            oninput: move |e| if let Some(d) = parse_date(&e.value()) { on.call(d) } }
    }
}

/// Properties of [`TimePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct TimePickerProps {
    pub hour: u32,
    pub minute: u32,
    /// Called with `(hour, minute)`.
    #[props(default)]
    pub onchange: EventHandler<(u32, u32)>,
}

/// Hour and minute with steppers that wrap, and an AM/PM label. Controlled.
#[component]
pub fn TimePicker(props: TimePickerProps) -> Element {
    let (h, m, on) = (props.hour, props.minute, props.onchange);
    let (h12, pm) = cal::to_12h(h);
    let col = |label: &'static str, text: String, up: (u32, u32), down: (u32, u32)| {
        rsx! {
            div { style: "display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-xs)",
                crate::IconButton { icon: IconId::ChevronUp, label: format!("{label} up"), onclick: move |_| on.call(up) }
                span { style: "font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)", "{text}" }
                crate::IconButton { icon: IconId::ChevronDown, label: format!("{label} down"), onclick: move |_| on.call(down) }
            }
        }
    };
    rsx! {
        KitBox { kit: Kit::Popup, style: "gap:var(--dodai-space-md);width:fit-content".to_string(),
            {col("Hour", format!("{h12:02}"), (cal::step_hour(h, 1), m), (cal::step_hour(h, -1), m))}
            span { style: "font-size:var(--dodai-size-xl)", ":" }
            {col("Minute", format!("{m:02}"), (h, cal::step_minute(m, 1)), (h, cal::step_minute(m, -1)))}
            span { style: "color:var(--dodai-text-muted)", if pm { "PM" } else { "AM" } }
        }
    }
}

/// Properties of [`TimeInput`].
#[derive(Props, Clone, PartialEq)]
pub struct TimeInputProps {
    pub hour: u32,
    pub minute: u32,
    #[props(default)]
    pub onchange: EventHandler<(u32, u32)>,
}

/// A typed time field (`HH:MM`). Controlled.
#[component]
pub fn TimeInput(props: TimeInputProps) -> Element {
    let on = props.onchange;
    rsx! {
        input { class: "dodai-input", r#type: "time", aria_label: "Time", value: "{props.hour:02}:{props.minute:02}",
            oninput: move |e| {
                let v = e.value();
                if let Some((h, m)) = v.split_once(':') {
                    if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) {
                        if h < 24 && m < 60 { on.call((h, m)); }
                    }
                }
            } }
    }
}

/// Properties of [`DateTimePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct DateTimePickerProps {
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
    #[props(default)]
    pub ondate: EventHandler<Date>,
    #[props(default)]
    pub ontime: EventHandler<(u32, u32)>,
}

/// A date picker above a time picker. Controlled.
#[component]
pub fn DateTimePicker(props: DateTimePickerProps) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-sm);width:fit-content",
            DatePicker { value: props.date, onchange: props.ondate }
            TimePicker { hour: props.hour, minute: props.minute, onchange: props.ontime }
        }
    }
}

/// Properties of [`DateTimeRangePicker`].
#[derive(Props, Clone, PartialEq)]
pub struct DateTimeRangePickerProps {
    pub range: DateRange,
    #[props(default)]
    pub onchange: EventHandler<DateRange>,
}

/// A calendar where two clicks pick a span ([`dodai_ui_core::calendar::range_pick`]). Controlled.
#[component]
pub fn DateTimeRangePicker(props: DateTimeRangePickerProps) -> Element {
    let (r, on) = (props.range, props.onchange);
    let (y, m) = r.start.map_or((2025, 1), |d| (d.year, d.month));
    rsx! {
        CalendarGrid { year: y, month: m, picked: r, onpick: move |d| on.call(cal::range_pick(r, d)) }
    }
}

/// Properties of [`Clock`].
#[derive(Props, Clone, PartialEq)]
pub struct ClockProps {
    /// Seconds since midnight.
    pub seconds: f64,
    #[props(default = 160)]
    pub size: u32,
}

/// An analogue clock face with hour, minute and second hands.
#[component]
pub fn Clock(props: ClockProps) -> Element {
    let (h, m, s) = cm::hand_angles(props.seconds);
    let hand = |a: f32, len: f32, w: f32, c: &'static str| {
        let (x, y) = (50.0 + len * a.sin(), 50.0 - len * a.cos());
        rsx! { line { x1: "50", y1: "50", x2: "{x}", y2: "{y}", stroke: c, stroke_width: "{w}", stroke_linecap: "round" } }
    };
    rsx! {
        svg { width: "{props.size}", height: "{props.size}", view_box: "0 0 100 100", role: "img", "aria-label": "{cm::hms(props.seconds as u64)}",
            circle { cx: "50", cy: "50", r: "48", fill: "var(--dodai-surface-raised)", stroke: "var(--dodai-border-strong)", stroke_width: "2" }
            for i in 0..12 {
                line { key: "{i}", x1: "50", y1: "6", x2: "50", y2: "10", stroke: "var(--dodai-text-muted)", stroke_width: "2",
                    transform: "rotate({i * 30} 50 50)" }
            }
            {hand(h, 24.0, 3.5, "var(--dodai-text)")}
            {hand(m, 34.0, 2.5, "var(--dodai-text)")}
            {hand(s, 38.0, 1.2, "var(--dodai-primary)")}
            circle { cx: "50", cy: "50", r: "2.5", fill: "var(--dodai-primary)" }
        }
    }
}

/// Properties of [`Stopwatch`].
#[derive(Props, Clone, PartialEq)]
pub struct StopwatchProps {
    /// Elapsed milliseconds, owned by the caller.
    pub ms: u64,
    pub running: bool,
    #[props(default)]
    pub ontoggle: EventHandler<()>,
    #[props(default)]
    pub onreset: EventHandler<()>,
    /// Cumulative millisecond totals at each lap press.
    #[props(default)]
    pub laps: Vec<u64>,
}

/// A stopwatch face with start/stop, reset and lap splits. The caller counts the time.
#[component]
pub fn Stopwatch(props: StopwatchProps) -> Element {
    let (t, r) = (props.ontoggle, props.onreset);
    let splits = cm::lap_splits(&props.laps);
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center",
            span { style: "font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)", "{cm::stopwatch_text(props.ms)}" }
            div { style: "display:flex;gap:var(--dodai-space-sm)",
                crate::Button { onclick: move |_| t.call(()), if props.running { "Stop" } else { "Start" } }
                crate::Button { variant: dodai_ui_core::ButtonVariant::Outline, onclick: move |_| r.call(()), "Reset" }
            }
            for (i, s) in splits.iter().enumerate() {
                span { key: "{i}", style: "font-family:var(--dodai-font-mono);color:var(--dodai-text-muted)", "Lap {i + 1}  {cm::stopwatch_text(*s)}" }
            }
        }
    }
}

/// Properties of [`Timer`].
#[derive(Props, Clone, PartialEq)]
pub struct TimerProps {
    /// Milliseconds left, owned by the caller.
    pub remaining_ms: u64,
    pub running: bool,
    #[props(default)]
    pub ontoggle: EventHandler<()>,
    #[props(default)]
    pub onreset: EventHandler<()>,
}

/// A countdown face with start/pause and reset. The caller counts the time.
#[component]
pub fn Timer(props: TimerProps) -> Element {
    let (t, r) = (props.ontoggle, props.onreset);
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center",
            span { style: "font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)", "{cm::countdown_text(props.remaining_ms)}" }
            div { style: "display:flex;gap:var(--dodai-space-sm)",
                crate::Button { onclick: move |_| t.call(()), if props.running { "Pause" } else { "Start" } }
                crate::Button { variant: dodai_ui_core::ButtonVariant::Outline, onclick: move |_| r.call(()), "Reset" }
            }
        }
    }
}

/// A tempo tapper: tap in rhythm, read the BPM, its name and the jitter.
#[component]
pub fn TapTempo() -> Element {
    let mut taps = use_signal(Vec::<f64>::new);
    let bpm = cm::tap_bpm(&taps.read());
    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm)",
            crate::Button { onclick: move |_| {
                    let now = cm::unix_now();
                    let mut t = taps.write();
                    if t.last().is_some_and(|l| now - l > 2.5) { t.clear(); }
                    t.push(now);
                    if t.len() > 8 { t.remove(0); }
                }, "Tap" }
            span { style: "font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)",
                match bpm { Some(b) => format!("{b:.0} BPM"), None => "— BPM".to_string() }
            }
            if let Some(b) = bpm { span { style: "color:var(--dodai-text-muted)", "{cm::tempo_name(b)}" } }
        }
    }
}

/// Properties of [`CountUp`].
#[derive(Props, Clone, PartialEq)]
pub struct CountUpProps {
    pub value: f64,
    #[props(default)]
    pub decimals: usize,
    #[props(default)]
    pub unit: String,
}

/// A big figure with tabular digits and a small unit. The caller animates `value`.
#[component]
pub fn CountUp(props: CountUpProps) -> Element {
    let d = props.decimals;
    rsx! {
        span { style: "font-family:var(--dodai-font-display);font-size:var(--dodai-size-xxl);font-variant-numeric:tabular-nums",
            "{props.value:.d$}"
            span { style: "font-size:var(--dodai-size-md);color:var(--dodai-text-muted);margin-left:var(--dodai-space-xs)", "{props.unit}" }
        }
    }
}

/// An icon shown beside a hint, used where the pickers need a leading mark.
#[component]
pub fn SelectionMarker(#[props(default)] selected: bool) -> Element {
    rsx! {
        span { style: "display:inline-flex;width:18px;color:var(--dodai-primary)",
            if selected { Icon { icon: IconId::Check, size: 16 } }
        }
    }
}
