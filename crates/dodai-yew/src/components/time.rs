//! Dates and times: calendar and pickers, clock, stopwatch, timer and tap tempo, on the shared
//! calendar and clock maths of `dodai-core`. Pickers are controlled; the clocks that count
//! take the elapsed time from the caller so the host owns the timer.

use super::kit::{Icon, KitBox};
use super::surfaces::IconButton;
use super::Button;
use crate::dom::input_value;
use dodai_core::calendar::{self as cal, Date, DateRange};
use dodai_core::clock_math as cm;
use dodai_core::{ButtonVariant, Kit};
use dodai_icons::IconId;
use yew::prelude::*;

const CELL: &str = "display:grid;place-items:center;width:var(--dodai-control-height);height:calc(var(--dodai-control-height) * .75);border:0;background:transparent;color:inherit;font:inherit;border-radius:var(--dodai-radius);cursor:pointer";

#[derive(Properties, PartialEq)]
struct GridProps {
    year: i32,
    month: u32,
    picked: DateRange,
    onpick: Callback<Date>,
}

#[function_component]
fn CalendarGrid(props: &GridProps) -> Html {
    let shown = use_state(|| (props.year, props.month));
    let (y, m) = *shown;
    let rows = cal::month_grid(y, m);
    let (prev, next) = (shown.clone(), shown.clone());
    html! {
        <KitBox kit={Kit::Popup} style="flex-direction:column;align-items:stretch;gap:var(--dodai-space-xs);width:fit-content;">
            <div style="display:flex;align-items:center;justify-content:space-between">
                <IconButton icon={IconId::ChevronLeft} label="Previous month" onclick={Callback::from(move |_| prev.set(cal::shift_month(y, m, -1)))} />
                <span style="font-family:var(--dodai-font-display)">{ format!("{} {y}", cal::MONTHS[(m - 1) as usize]) }</span>
                <IconButton icon={IconId::ChevronRight} label="Next month" onclick={Callback::from(move |_| next.set(cal::shift_month(y, m, 1)))} />
            </div>
            <div style="display:grid;grid-template-columns:repeat(7,auto);justify-items:center">
                { for cal::WEEKDAYS.iter().map(|w| html! { <span style="font-size:var(--dodai-size-xs);color:var(--dodai-text-muted);padding:var(--dodai-space-xs)">{ *w }</span> }) }
                { for rows.iter().flat_map(|r| r.iter()).map(|cell| match cell {
                    Some(day) => {
                        let d = Date::new(y, m, *day);
                        let on = props.picked.contains(d);
                        let extra = if on { "background:var(--dodai-primary);color:var(--dodai-background)" } else { "" };
                        let pick = props.onpick.clone();
                        html! { <button style={format!("{CELL};{extra}")} aria-pressed={on.to_string()} onclick={Callback::from(move |_| pick.emit(d))}>{ *day }</button> }
                    }
                    None => html! { <span></span> },
                }) }
            </div>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct DatePickerProps {
    pub value: Date,
    #[prop_or_default]
    pub onchange: Callback<Date>,
}

/// A month grid with previous and next arrows; picks one day. Controlled.
#[function_component]
pub fn DatePicker(props: &DatePickerProps) -> Html {
    let v = props.value;
    html! { <CalendarGrid year={v.year} month={v.month} picked={DateRange { start: Some(v), end: Some(v) }} onpick={props.onchange.clone()} /> }
}

#[derive(Properties, PartialEq)]
pub struct DateInputProps {
    pub value: Date,
    #[prop_or_default]
    pub onchange: Callback<Date>,
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

/// A typed date field; only a valid date is reported. Controlled.
#[function_component]
pub fn DateInput(props: &DateInputProps) -> Html {
    let v = props.value;
    let cb = props.onchange.clone();
    html! {
        <input class="dodai-input" type="date" aria-label="Date" value={format!("{:04}-{:02}-{:02}", v.year, v.month, v.day)}
            oninput={Callback::from(move |e: InputEvent| if let Some(d) = parse_date(&input_value(&e)) { cb.emit(d) })} />
    }
}

#[derive(Properties, PartialEq)]
pub struct TimePickerProps {
    pub hour: u32,
    pub minute: u32,
    /// Called with `(hour, minute)`.
    #[prop_or_default]
    pub onchange: Callback<(u32, u32)>,
}

/// Hour and minute with steppers that wrap, and an AM/PM label. Controlled.
#[function_component]
pub fn TimePicker(props: &TimePickerProps) -> Html {
    let (h, m) = (props.hour, props.minute);
    let (h12, pm) = cal::to_12h(h);
    let col = |label: &'static str, text: String, up: (u32, u32), down: (u32, u32)| {
        let (a, b) = (props.onchange.clone(), props.onchange.clone());
        html! {
            <div style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-xs)">
                <IconButton icon={IconId::ChevronUp} label={format!("{label} up")} onclick={Callback::from(move |_| a.emit(up))} />
                <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)">{ text }</span>
                <IconButton icon={IconId::ChevronDown} label={format!("{label} down")} onclick={Callback::from(move |_| b.emit(down))} />
            </div>
        }
    };
    html! {
        <KitBox kit={Kit::Popup} style="gap:var(--dodai-space-md);width:fit-content;">
            { col("Hour", format!("{h12:02}"), (cal::step_hour(h, 1), m), (cal::step_hour(h, -1), m)) }
            <span style="font-size:var(--dodai-size-xl)">{ ":" }</span>
            { col("Minute", format!("{m:02}"), (h, cal::step_minute(m, 1)), (h, cal::step_minute(m, -1))) }
            <span style="color:var(--dodai-text-muted)">{ if pm { "PM" } else { "AM" } }</span>
        </KitBox>
    }
}

#[derive(Properties, PartialEq)]
pub struct TimeInputProps {
    pub hour: u32,
    pub minute: u32,
    #[prop_or_default]
    pub onchange: Callback<(u32, u32)>,
}

/// A typed time field. Controlled.
#[function_component]
pub fn TimeInput(props: &TimeInputProps) -> Html {
    let cb = props.onchange.clone();
    html! {
        <input class="dodai-input" type="time" aria-label="Time" value={format!("{:02}:{:02}", props.hour, props.minute)}
            oninput={Callback::from(move |e: InputEvent| {
                let v = input_value(&e);
                if let Some((h, m)) = v.split_once(':') {
                    if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) {
                        if h < 24 && m < 60 { cb.emit((h, m)); }
                    }
                }
            })} />
    }
}

#[derive(Properties, PartialEq)]
pub struct DateTimePickerProps {
    pub date: Date,
    pub hour: u32,
    pub minute: u32,
    #[prop_or_default]
    pub ondate: Callback<Date>,
    #[prop_or_default]
    pub ontime: Callback<(u32, u32)>,
}

/// A date picker above a time picker. Controlled.
#[function_component]
pub fn DateTimePicker(props: &DateTimePickerProps) -> Html {
    html! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);width:fit-content">
            <DatePicker value={props.date} onchange={props.ondate.clone()} />
            <TimePicker hour={props.hour} minute={props.minute} onchange={props.ontime.clone()} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct DateTimeRangePickerProps {
    pub range: DateRange,
    #[prop_or_default]
    pub onchange: Callback<DateRange>,
}

/// A calendar where two clicks pick a span ([`dodai_core::calendar::range_pick`]). Controlled.
#[function_component]
pub fn DateTimeRangePicker(props: &DateTimeRangePickerProps) -> Html {
    let (r, cb) = (props.range, props.onchange.clone());
    let (y, m) = r.start.map_or((2025, 1), |d| (d.year, d.month));
    html! { <CalendarGrid year={y} month={m} picked={r} onpick={Callback::from(move |d| cb.emit(cal::range_pick(r, d)))} /> }
}

#[derive(Properties, PartialEq)]
pub struct ClockProps {
    /// Seconds since midnight.
    pub seconds: f64,
    #[prop_or(160)]
    pub size: u32,
}

/// An analogue clock face with hour, minute and second hands.
#[function_component]
pub fn Clock(props: &ClockProps) -> Html {
    let (h, m, s) = cm::hand_angles(props.seconds);
    let hand = |a: f32, len: f32, w: f32, c: &'static str| {
        let (x, y) = (50.0 + len * a.sin(), 50.0 - len * a.cos());
        html! { <line x1="50" y1="50" x2={x.to_string()} y2={y.to_string()} stroke={c} stroke-width={w.to_string()} stroke-linecap="round" /> }
    };
    html! {
        <svg width={props.size.to_string()} height={props.size.to_string()} viewBox="0 0 100 100" role="img" aria-label={cm::hms(props.seconds as u64)}>
            <circle cx="50" cy="50" r="48" fill="var(--dodai-surface-raised)" stroke="var(--dodai-border-strong)" stroke-width="2" />
            { for (0..12).map(|i| html! { <line x1="50" y1="6" x2="50" y2="10" stroke="var(--dodai-text-muted)" stroke-width="2" transform={format!("rotate({} 50 50)", i * 30)} /> }) }
            { hand(h, 24.0, 3.5, "var(--dodai-text)") }
            { hand(m, 34.0, 2.5, "var(--dodai-text)") }
            { hand(s, 38.0, 1.2, "var(--dodai-primary)") }
            <circle cx="50" cy="50" r="2.5" fill="var(--dodai-primary)" />
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct StopwatchProps {
    /// Elapsed milliseconds, owned by the caller.
    pub ms: u64,
    pub running: bool,
    #[prop_or_default]
    pub ontoggle: Callback<()>,
    #[prop_or_default]
    pub onreset: Callback<()>,
    /// Cumulative millisecond totals at each lap press.
    #[prop_or_default]
    pub laps: Vec<u64>,
}

/// A stopwatch face with start/stop, reset and lap splits. The caller counts the time.
#[function_component]
pub fn Stopwatch(props: &StopwatchProps) -> Html {
    let (t, r) = (props.ontoggle.clone(), props.onreset.clone());
    html! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center">
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)">{ cm::stopwatch_text(props.ms) }</span>
            <div style="display:flex;gap:var(--dodai-space-sm)">
                <Button onclick={Callback::from(move |_| t.emit(()))}>{ if props.running { "Stop" } else { "Start" } }</Button>
                <Button variant={ButtonVariant::Outline} onclick={Callback::from(move |_| r.emit(()))}>{ "Reset" }</Button>
            </div>
            { for cm::lap_splits(&props.laps).into_iter().enumerate().map(|(i, s)| html! {
                <span style="font-family:var(--dodai-font-mono);color:var(--dodai-text-muted)">{ format!("Lap {}  {}", i + 1, cm::stopwatch_text(s)) }</span>
            }) }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TimerProps {
    /// Milliseconds left, owned by the caller.
    pub remaining_ms: u64,
    pub running: bool,
    #[prop_or_default]
    pub ontoggle: Callback<()>,
    #[prop_or_default]
    pub onreset: Callback<()>,
}

/// A countdown face with start/pause and reset. The caller counts the time.
#[function_component]
pub fn Timer(props: &TimerProps) -> Html {
    let (t, r) = (props.ontoggle.clone(), props.onreset.clone());
    html! {
        <div style="display:flex;flex-direction:column;gap:var(--dodai-space-sm);align-items:center">
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xxl)">{ cm::countdown_text(props.remaining_ms) }</span>
            <div style="display:flex;gap:var(--dodai-space-sm)">
                <Button onclick={Callback::from(move |_| t.emit(()))}>{ if props.running { "Pause" } else { "Start" } }</Button>
                <Button variant={ButtonVariant::Outline} onclick={Callback::from(move |_| r.emit(()))}>{ "Reset" }</Button>
            </div>
        </div>
    }
}

/// A tempo tapper: tap in rhythm, read the BPM, its name and the jitter.
#[function_component]
pub fn TapTempo() -> Html {
    let taps = use_state(Vec::<f64>::new);
    let bpm = cm::tap_bpm(&taps);
    let tap = {
        let taps = taps.clone();
        Callback::from(move |_| {
            let now = cm::unix_now();
            let mut t = (*taps).clone();
            if t.last().is_some_and(|l| now - l > 2.5) {
                t.clear();
            }
            t.push(now);
            if t.len() > 8 {
                t.remove(0);
            }
            taps.set(t);
        })
    };
    html! {
        <div style="display:flex;flex-direction:column;align-items:center;gap:var(--dodai-space-sm)">
            <Button onclick={tap}>{ "Tap" }</Button>
            <span style="font-family:var(--dodai-font-mono);font-size:var(--dodai-size-xl)">
                { match bpm { Some(b) => format!("{b:.0} BPM"), None => "— BPM".to_string() } }
            </span>
            if let Some(b) = bpm { <span style="color:var(--dodai-text-muted)">{ cm::tempo_name(b) }</span> }
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct CountUpProps {
    pub value: f64,
    #[prop_or_default]
    pub decimals: usize,
    #[prop_or_default]
    pub unit: AttrValue,
}

/// A big figure with tabular digits and a small unit. The caller animates `value`.
#[function_component]
pub fn CountUp(props: &CountUpProps) -> Html {
    html! {
        <span style="font-family:var(--dodai-font-display);font-size:var(--dodai-size-xxl);font-variant-numeric:tabular-nums">
            { format!("{:.*}", props.decimals, props.value) }
            <span style="font-size:var(--dodai-size-md);color:var(--dodai-text-muted);margin-left:var(--dodai-space-xs)">{ props.unit.clone() }</span>
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct SelectionMarkerProps {
    #[prop_or_default]
    pub selected: bool,
}

/// The check mark in front of a picked option.
#[function_component]
pub fn SelectionMarker(props: &SelectionMarkerProps) -> Html {
    html! {
        <span style="display:inline-flex;width:18px;color:var(--dodai-primary)">
            if props.selected { <Icon icon={IconId::Check} size={16} /> }
        </span>
    }
}
