//! Dates and times: calendar and pickers, clock, stopwatch, timer and tap tempo, on the shared
//! calendar and clock maths of `dodai-ui-core`. State comes in as props; the host owns timing.

use super::kit::{Icon, KitBox};
use super::surfaces::IconButton;
use super::Button;
use dodai_ui_core::calendar::{self as cal, Date, DateRange};
use dodai_ui_core::clock_math as cm;
use dodai_ui_core::{ButtonVariant, Kit};
use dodai_ui_icons::IconId;
use leptos::prelude::*;

const CELL: &str = "display:grid;place-items:center;width:var(--dodai-control-height);height:calc(var(--dodai-control-height) * .75);border:0;background:transparent;color:inherit;font:inherit;border-radius:var(--dodai-radius);cursor:pointer";

fn grid(year: i32, month: u32, picked: DateRange) -> impl IntoView {
    let cells = cal::month_grid(year, month)
        .into_iter()
        .flat_map(|row| row.into_iter())
        .map(|cell| match cell {
            Some(day) => {
                let on = picked.contains(Date::new(year, month, day));
                let extra = if on { "background:var(--dodai-primary);color:var(--dodai-background)" } else { "" };
                view! { <button style=format!("{CELL};{extra}") aria-pressed=on.to_string()>{day}</button> }.into_any()
            }
            None => view! { <span></span> }.into_any(),
        })
        .collect_view();
    let heads = cal::WEEKDAYS
        .iter()
        .map(|w| view! { <span style="font-size:var(--dodai-size-xs);color:var(--dodai-text-muted);padding:var(--dodai-space-xs)">{*w}</span> })
        .collect_view();
    view! {
        <KitBox kit=Kit::Popup style="flex-direction:column;align-items:stretch;gap:var(--dodai-space-xs);width:fit-content;">
            <div style="display:flex;align-items:center;justify-content:space-between">
                <IconButton icon=IconId::ChevronLeft label="Previous month" />
                <span style="font-family:var(--dodai-font-display)">{format!("{} {year}", cal::MONTHS[(month - 1) as usize])}</span>
                <IconButton icon=IconId::ChevronRight label="Next month" />
            </div>
            <div style="display:grid;grid-template-columns:repeat(7,auto);justify-items:center">{heads}{cells}</div>
        </KitBox>
    }
}

/// A month grid with previous and next arrows; the picked day is filled.
#[component]
pub fn DatePicker(value: Date) -> impl IntoView {
    grid(
        value.year,
        value.month,
        DateRange {
            start: Some(value),
            end: Some(value),
        },
    )
}

/// A typed date field; the browser's date control.
#[component]
pub fn DateInput(value: Date) -> impl IntoView {
    view! { <input class="dodai-input" type="date" aria-label="Date" value=format!("{:04}-{:02}-{:02}", value.year, value.month, value.day) /> }
}

fn time_column(label: &'static str, text: String) -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-xs)">
            <IconButton icon=IconId::ChevronUp label=format!("{label} up") />
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)">{text}</span>
            <IconButton icon=IconId::ChevronDown label=format!("{label} down") />
        </div>
    }
}

/// Hour and minute with steppers, and an AM/PM label.
#[component]
pub fn TimePicker(hour: u32, minute: u32) -> impl IntoView {
    let (h12, pm) = cal::to_12h(hour);
    view! {
        <KitBox kit=Kit::Popup style="gap:var(--dodai-space-md);width:fit-content;">
            {time_column("Hour", format!("{h12:02}"))}
            <span style="font-size:var(--dodai-size-xl)">":"</span>
            {time_column("Minute", format!("{minute:02}"))}
            <span style="color:var(--dodai-text-muted)">{if pm { "PM" } else { "AM" }}</span>
        </KitBox>
    }
}

/// A typed time field; the browser's time control.
#[component]
pub fn TimeInput(hour: u32, minute: u32) -> impl IntoView {
    view! { <input class="dodai-input" type="time" aria-label="Time" value=format!("{hour:02}:{minute:02}") /> }
}

/// A date picker above a time picker.
#[component]
pub fn DateTimePicker(date: Date, hour: u32, minute: u32) -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);width:fit-content">
            <DatePicker value=date />
            <TimePicker hour=hour minute=minute />
        </div>
    }
}

/// A calendar with a picked span (see [`dodai_ui_core::calendar::range_pick`]).
#[component]
pub fn DateTimeRangePicker(range: DateRange) -> impl IntoView {
    let (y, m) = range.start.map_or((2025, 1), |d| (d.year, d.month));
    grid(y, m, range)
}

/// An analogue clock face; `seconds` since midnight.
#[component]
pub fn Clock(seconds: f64, #[prop(default = 160)] size: u32) -> impl IntoView {
    let (h, m, s) = cm::hand_angles(seconds);
    let hand = |a: f32, len: f32, w: f32, c: &'static str| {
        let (x, y) = (50.0 + len * a.sin(), 50.0 - len * a.cos());
        view! { <line x1="50" y1="50" x2=x.to_string() y2=y.to_string() stroke=c stroke-width=w.to_string() stroke-linecap="round" /> }
    };
    let ticks = (0..12)
        .map(|i| view! { <line x1="50" y1="6" x2="50" y2="10" stroke="var(--dodai-text-muted)" stroke-width="2" transform=format!("rotate({} 50 50)", i * 30) /> })
        .collect_view();
    view! {
        <svg width=size.to_string() height=size.to_string() viewBox="0 0 100 100" role="img" aria-label=cm::hms(seconds as u64)>
            <circle cx="50" cy="50" r="48" fill="var(--dodai-surface-raised)" stroke="var(--dodai-border-strong)" stroke-width="2" />
            {ticks}
            {hand(h, 24.0, 3.5, "var(--dodai-text)")}
            {hand(m, 34.0, 2.5, "var(--dodai-text)")}
            {hand(s, 38.0, 1.2, "var(--dodai-primary)")}
            <circle cx="50" cy="50" r="2.5" fill="var(--dodai-primary)" />
        </svg>
    }
}

/// A stopwatch face; `ms` and `laps` (cumulative ms per lap press) are the caller's.
#[component]
pub fn Stopwatch(ms: u64, running: bool, #[prop(optional)] laps: Vec<u64>) -> impl IntoView {
    let splits = cm::lap_splits(&laps)
        .into_iter()
        .enumerate()
        .map(|(i, s)| view! { <span style="font-family:var(--dodai-font-mono);color:var(--dodai-text-muted)">{format!("Lap {}  {}", i + 1, cm::stopwatch_text(s))}</span> })
        .collect_view();
    view! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center">
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)">{cm::stopwatch_text(ms)}</span>
            <div style="display:flex;gap:var(--dodai-space-sm)">
                <Button>{if running { "Stop" } else { "Start" }}</Button>
                <Button variant=ButtonVariant::Outline>"Reset"</Button>
            </div>
            {splits}
        </div>
    }
}

/// A countdown face; `remaining_ms` is the caller's.
#[component]
pub fn Timer(remaining_ms: u64, running: bool) -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center">
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)">{cm::countdown_text(remaining_ms)}</span>
            <div style="display:flex;gap:var(--dodai-space-sm)">
                <Button>{if running { "Pause" } else { "Start" }}</Button>
                <Button variant=ButtonVariant::Outline>"Reset"</Button>
            </div>
        </div>
    }
}

/// A tempo tapper showing the BPM of `taps` (unix seconds of each tap), its name and nothing
/// until two taps exist.
#[component]
pub fn TapTempo(#[prop(optional)] taps: Vec<f64>) -> impl IntoView {
    let bpm = cm::tap_bpm(&taps);
    view! {
        <div style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm)">
            <Button>"Tap"</Button>
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)">
                {match bpm { Some(b) => format!("{b:.0} BPM"), None => "— BPM".to_string() }}
            </span>
            {bpm.map(|b| view! { <span style="color:var(--dodai-text-muted)">{cm::tempo_name(b)}</span> })}
        </div>
    }
}

/// A big figure with tabular digits and a small unit.
#[component]
pub fn CountUp(
    value: f64,
    #[prop(optional)] decimals: usize,
    #[prop(optional, into)] unit: String,
) -> impl IntoView {
    view! {
        <span style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-xxl);font-variant-numeric:tabular-nums">
            {format!("{value:.decimals$}")}
            <span style="font-size:var(--dodai-size-md);color:var(--dodai-text-muted);margin-left:var(--dodai-space-xs)">{unit}</span>
        </span>
    }
}

/// The check mark in front of a picked option.
#[component]
pub fn SelectionMarker(#[prop(optional)] selected: bool) -> impl IntoView {
    view! {
        <span style="display:inline-flex;width:18px;color:var(--dodai-primary)">
            {selected.then(|| view! { <Icon icon=IconId::Check size=16 /> })}
        </span>
    }
}
