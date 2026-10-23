//! [`DatePicker`] (month grid) and [`TimePicker`] (stepper columns).

use crate::button::button_style;
use crate::interact::{track, use_interact};
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::surfaces::IconButton;
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, ControlState};
use gpui::{
    div, px, App, ElementId, IntoElement, ParentElement, RenderOnce, StatefulInteractiveElement,
    Styled, Window,
};
use std::rc::Rc;

// ───────────────────────────── calendar math ─────────────────────────────

/// Calendar date (proleptic Gregorian).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    /// Year.
    pub year: i32,
    /// Month, 1-12.
    pub month: u32,
    /// Day of the month, 1-31.
    pub day: u32,
}

impl Date {
    /// Create a new [`Date`].
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }
}

/// Selected span of days (inclusive). `end` is `None` while the second click is pending.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    /// First day.
    pub start: Option<Date>,
    /// Last day.
    pub end: Option<Date>,
}

impl DateRange {
    /// `(from, to)` once both ends are picked.
    pub fn span(&self) -> Option<(Date, Date)> {
        Some((self.start?, self.end?))
    }
    /// Whether `d` lies inside the picked span (ends included).
    pub fn contains(&self, d: Date) -> bool {
        match (self.start, self.end) {
            (Some(a), Some(b)) => a <= d && d <= b,
            (Some(a), None) => a == d,
            _ => false,
        }
    }
}

/// Next range after clicking `d`: the first click starts, the second ends
/// (clicking an earlier day restarts there), a third click starts over.
pub fn range_pick(r: DateRange, d: Date) -> DateRange {
    match (r.start, r.end) {
        (Some(a), None) if d >= a => DateRange {
            start: Some(a),
            end: Some(d),
        },
        _ => DateRange {
            start: Some(d),
            end: None,
        },
    }
}

/// English month names, January first.
pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
/// Two-letter weekday names, Monday first.
pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

/// Whether `y` is a Gregorian leap year.
pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Number of days in month `m` (1-12) of year `y`.
pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if is_leap(y) => 29,
        _ => 28,
    }
}

/// Weekday of a date, Monday = 0 … Sunday = 6.
pub fn weekday(y: i32, m: u32, d: u32) -> u32 {
    const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let yy = if m < 3 { y - 1 } else { y };
    let sun0 = (yy + yy / 4 - yy / 100 + yy / 400 + T[(m - 1) as usize] + d as i32).rem_euclid(7);
    ((sun0 + 6) % 7) as u32
}

/// Month as rows of 7 cells (Monday first); `None` pads outside the month.
pub fn month_grid(y: i32, m: u32) -> Vec<[Option<u32>; 7]> {
    let lead = weekday(y, m, 1) as usize;
    let n = days_in_month(y, m);
    let mut rows = Vec::new();
    let mut day = 1u32;
    let mut first = true;
    while day <= n {
        let mut row = [None; 7];
        for (i, c) in row.iter_mut().enumerate() {
            if first && i < lead {
                continue;
            }
            if day <= n {
                *c = Some(day);
                day += 1;
            }
        }
        first = false;
        rows.push(row);
    }
    rows
}

/// Previous / next month of `(year, month)`.
pub fn shift_month(y: i32, m: u32, delta: i32) -> (i32, u32) {
    let idx = y * 12 + (m as i32 - 1) + delta;
    (idx.div_euclid(12), (idx.rem_euclid(12) + 1) as u32)
}

type DateFn = Rc<dyn Fn(Date, &mut Window, &mut App)>;
type MonthFn = Rc<dyn Fn(i32, u32, &mut Window, &mut App)>;

/// Controlled month calendar.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    year: i32,
    month: u32,
    selected: Option<Date>,
    range: Option<DateRange>,
    on_select: Option<DateFn>,
    on_month: Option<MonthFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(DatePicker);

impl DatePicker {
    /// `year`/`month` (1-12) is the visible month.
    pub fn new(id: impl Into<ElementId>, year: i32, month: u32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            year,
            month: month.clamp(1, 12),
            selected: None,
            range: None,
            on_select: None,
            on_month: None,
        }
    }
    /// The selected item (controlled).
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    /// Highlight a span of days instead of one. Combine with [`range_pick`] in `on_select`.
    pub fn range(mut self, r: DateRange) -> Self {
        self.range = Some(r);
        self
    }
    crate::callback!(
        /// Called when an item is chosen, with the date.
        on_select, Date
    );
    crate::callback!(
        /// Called with the new visible `(year, month)` when the arrows are used.
        on_month, i32, u32
    );
}

impl DatePicker {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = t.panel(false);
        let (y, m) = (self.year, self.month);
        let nav = |suffix: &'static str, glyph: &'static str, delta: i32| {
            let mut b = IconButton::new((self.id.clone(), suffix), glyph);
            if let Some(f) = self.on_month.clone() {
                let (ny, nm) = shift_month(y, m, delta);
                b = b.on_click(move |_, w, cx| f(ny, nm, w, cx));
            }
            b
        };
        let header = div()
            .flex()
            .items_center()
            .justify_between()
            .child(nav("prev", "‹", -1))
            .child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_lg))
                    .text_color(hsla(p.title))
                    .child(format_label(
                        &t.typography,
                        &format!("{} {}", MONTHS[(m - 1) as usize], y),
                    )),
            )
            .child(nav("next", "›", 1));
        let cell = t.shape.control_height * 0.8;
        let weekdays = div().flex().children(WEEKDAYS.iter().map(|d| {
            div()
                .w(px(cell))
                .flex()
                .justify_center()
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(*d)
        }));
        let mut grid = div().flex().flex_col().gap(px(2.0));
        for (ri, row) in month_grid(y, m).into_iter().enumerate() {
            let mut r = div().flex().gap(px(0.0));
            for (ci, c) in row.into_iter().enumerate() {
                let Some(day) = c else {
                    r = r.child(div().w(px(cell)).h(px(cell)));
                    continue;
                };
                let date = Date::new(y, m, day);
                let on = match self.range {
                    Some(r) => r.start == Some(date) || r.end == Some(date),
                    None => self.selected == Some(date),
                };
                let inside = self.range.is_some_and(|r| r.contains(date)) && !on;
                let id: ElementId = (self.id.clone(), format!("d{ri}{ci}")).into();
                let st = use_interact(&id, window, cx);
                let state = st.read(cx).control_state(false);
                let variant = if on {
                    ButtonVariant::Primary
                } else if inside {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Ghost
                };
                let (mut style, fg) = button_style(&t, variant, state);
                if !on && state == ControlState::Idle {
                    style.stroke = None;
                }
                if inside && state == ControlState::Idle {
                    style.fill = Some(hsla(t.palette.selection));
                }
                let mut el = track(shaped_id(id, &t.shape, style), &st)
                    .w(px(cell))
                    .h(px(cell))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(t.typography.size_md))
                    .text_color(fg)
                    .cursor_pointer()
                    .child(day.to_string());
                if let Some(f) = self.on_select.clone() {
                    el = el.on_click(move |_, w, cx| f(date, w, cx));
                }
                r = r.child(el);
            }
            grid = grid.child(r);
        }
        let style = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true);
        shaped(&t.shape, style)
            .p(px(t.spacing.md))
            .flex()
            .flex_col()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_color(hsla(t.palette.text))
            .child(header)
            .child(weekdays)
            .child(grid)
    }
}

crate::entry_render!(DatePicker);

// ───────────────────────────── time picker ─────────────────────────────

/// Step `hour` by `delta`, wrapping within 24 h.
pub fn step_hour(h: u32, delta: i32) -> u32 {
    (h as i32 + delta).rem_euclid(24) as u32
}

/// Step `minute` by `delta` (wrapping 0-59).
pub fn step_minute(mn: u32, delta: i32) -> u32 {
    (mn as i32 + delta).rem_euclid(60) as u32
}

/// `(hour12, is_pm)` for a 24-hour value.
pub fn to_12h(h: u32) -> (u32, bool) {
    let h12 = match h % 12 {
        0 => 12,
        v => v,
    };
    (h12, h >= 12)
}

type TimeFn = Rc<dyn Fn(u32, u32, &mut Window, &mut App)>;

/// Controlled time picker (hour : minute with steppers, optional 12 h mode).
#[derive(IntoElement)]
pub struct TimePicker {
    id: ElementId,
    hour: u32,
    minute: u32,
    twelve: bool,
    minute_step: i32,
    on_change: Option<TimeFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(TimePicker);

impl TimePicker {
    /// Create a new [`TimePicker`].
    pub fn new(id: impl Into<ElementId>, hour: u32, minute: u32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            hour: hour % 24,
            minute: minute % 60,
            twelve: false,
            minute_step: 5,
            on_change: None,
        }
    }
    /// 12-hour display with AM/PM toggle.
    pub fn twelve_hour(mut self, on: bool) -> Self {
        self.twelve = on;
        self
    }
    /// Minute step of the picker.
    pub fn minute_step(mut self, s: i32) -> Self {
        self.minute_step = s.max(1);
        self
    }
    crate::callback!(
        /// Called when the value changes, with hour and minute.
        on_change, u32, u32
    );
}

impl TimePicker {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = t.panel(false);
        let (h, mn) = (self.hour, self.minute);
        let btn = |suffix: &'static str, glyph: &'static str, dh: i32, dm: i32| {
            let mut b = IconButton::new((self.id.clone(), suffix), glyph);
            if let Some(f) = self.on_change.clone() {
                let (nh, nm) = (step_hour(h, dh), step_minute(mn, dm));
                b = b.on_click(move |_, w, cx| f(nh, nm, w, cx));
            }
            b
        };
        let (shown_h, pm) = if self.twelve { to_12h(h) } else { (h, false) };
        let digits = |txt: String| {
            let st = ShapeStyle::from_tokens(
                &t,
                Some(t.palette.surface_raised),
                Some(p.border),
                false,
                false,
            );
            shaped(&t.shape, st)
                .w(px(t.shape.control_height * 1.6))
                .h(px(t.shape.control_height * 1.4))
                .flex()
                .items_center()
                .justify_center()
                .font_family(first_family(t.typography.display))
                .text_size(px(t.typography.size_xl))
                .text_color(hsla(t.palette.text))
                .child(txt)
        };
        let col = |up: IconButton, txt: String, down: IconButton| {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(t.spacing.xs))
                .child(up)
                .child(digits(txt))
                .child(down)
        };
        let mut row = div()
            .flex()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(col(
                btn("h+", "▲", 1, 0),
                format!("{shown_h:02}"),
                btn("h-", "▼", -1, 0),
            ))
            .child(div().text_size(px(t.typography.size_xl)).child(":"))
            .child(col(
                btn("m+", "▲", 0, self.minute_step),
                format!("{mn:02}"),
                btn("m-", "▼", 0, -self.minute_step),
            ));
        if self.twelve {
            let mut flip = IconButton::new((self.id.clone(), "ampm"), if pm { "PM" } else { "AM" })
                .variant(ButtonVariant::Outline);
            if let Some(f) = self.on_change.clone() {
                let nh = step_hour(h, 12);
                flip = flip.on_click(move |_, w, cx| f(nh, mn, w, cx));
            }
            row = row.child(flip);
        }
        let style = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true);
        shaped(&t.shape, style)
            .p(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .child(row)
    }
}

crate::entry_render!(TimePicker);

// ───────────────────────────── text input mode ─────────────────────────────

/// Parse a typed date: `YYYY-MM-DD`, `YYYY/MM/DD` or `MM/DD/YYYY`; `None` when it is not a real
/// calendar date.
pub fn parse_date(s: &str) -> Option<Date> {
    let s = s.trim();
    let parts: Vec<&str> = s.split(['-', '/', '.']).map(str::trim).collect();
    let [a, b, c] = parts.as_slice() else {
        return None;
    };
    let n = |x: &str| x.parse::<i32>().ok();
    let (y, m, d) = if a.len() == 4 {
        (n(a)?, n(b)?, n(c)?)
    } else if c.len() == 4 {
        (n(c)?, n(a)?, n(b)?)
    } else {
        return None;
    };
    if !(1..=12).contains(&m) || d < 1 || d as u32 > days_in_month(y, m as u32) {
        return None;
    }
    Some(Date::new(y, m as u32, d as u32))
}

/// `YYYY-MM-DD`.
pub fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// Parse a typed time: `HH:MM` (24 h) or `H:MM AM` / `H:MM PM`.
pub fn parse_time(s: &str) -> Option<(u32, u32)> {
    let s = s.trim().to_uppercase();
    let (body, meridiem) = match s.strip_suffix("PM") {
        Some(b) => (b.trim(), Some(true)),
        None => match s.strip_suffix("AM") {
            Some(b) => (b.trim(), Some(false)),
            None => (s.as_str(), None),
        },
    };
    let (h, m) = body.split_once(':')?;
    let (h, m) = (h.trim().parse::<u32>().ok()?, m.trim().parse::<u32>().ok()?);
    if m > 59 {
        return None;
    }
    match meridiem {
        None => (h < 24).then_some((h, m)),
        Some(pm) => ((1..=12).contains(&h)).then(|| ((h % 12) + if pm { 12 } else { 0 }, m)),
    }
}

/// `HH:MM`.
pub fn format_time(h: u32, m: u32) -> String {
    format!("{h:02}:{m:02}")
}

type DateChange = Rc<dyn Fn(Date, &mut Window, &mut App)>;
type TimeChange = Rc<dyn Fn(u32, u32, &mut Window, &mut App)>;

/// Typed date entry (the picker's text-input mode): a labelled field that reports each valid
/// date and shows an error line for text that is not one.
#[derive(IntoElement)]
pub struct DateInput {
    id: ElementId,
    value: Option<Date>,
    on_change: Option<DateChange>,
}

impl DateInput {
    /// Create a new [`DateInput`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            on_change: None,
        }
    }
    /// The current (controlled) date; changing it rewrites the field.
    pub fn value(mut self, d: Option<Date>) -> Self {
        self.value = d;
        self
    }
    crate::callback!(
        /// Called when the value changes, with the date.
        on_change, Date
    );
}

impl RenderOnce for DateInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        text_mode(
            window,
            cx,
            self.id,
            "Date",
            "YYYY-MM-DD",
            "Use YYYY-MM-DD or MM/DD/YYYY",
            self.value.map(format_date),
            |s| parse_date(s).is_some(),
            {
                let f = self.on_change;
                move |s, w, cx| {
                    if let (Some(f), Some(d)) = (&f, parse_date(s)) {
                        f(d, w, cx);
                    }
                }
            },
        )
    }
}

/// Typed time entry: `HH:MM` or `H:MM AM/PM`.
#[derive(IntoElement)]
pub struct TimeInput {
    id: ElementId,
    value: Option<(u32, u32)>,
    on_change: Option<TimeChange>,
}

impl TimeInput {
    /// Create a new [`TimeInput`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            on_change: None,
        }
    }
    /// Current value.
    pub fn value(mut self, hm: Option<(u32, u32)>) -> Self {
        self.value = hm;
        self
    }
    crate::callback!(
        /// Called when the value changes, with hour and minute.
        on_change, u32, u32
    );
}

impl RenderOnce for TimeInput {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        text_mode(
            window,
            cx,
            self.id,
            "Time",
            "HH:MM",
            "Use HH:MM or H:MM AM/PM",
            self.value.map(|(h, m)| format_time(h, m)),
            |s| parse_time(s).is_some(),
            {
                let f = self.on_change;
                move |s, w, cx| {
                    if let (Some(f), Some((h, m))) = (&f, parse_time(s)) {
                        f(h, m, w, cx);
                    }
                }
            },
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn text_mode(
    window: &mut Window,
    cx: &mut App,
    id: ElementId,
    label: &'static str,
    placeholder: &'static str,
    hint: &'static str,
    value: Option<String>,
    valid: impl Fn(&str) -> bool,
    on_text: impl Fn(&str, &mut Window, &mut App) + 'static,
) -> gpui::Div {
    let t = theme_of(cx);
    let input = window.use_keyed_state((id.clone(), "field"), cx, |_, cx| {
        let mut ti = crate::TextInput::new(cx)
            .label(label)
            .placeholder(placeholder)
            .on_change(on_text);
        if let Some(v) = &value {
            ti.set_text(v.clone(), cx);
        }
        ti
    });
    // Rewrite the field only when the controlled value itself changed (not while typing).
    let last = window.use_keyed_state((id, "last"), cx, |_, _| value.clone());
    if *last.read(cx) != value {
        last.update(cx, |l, _| *l = value.clone());
        if let Some(v) = value {
            input.update(cx, |ti, cx| ti.set_text(v, cx));
        }
    }
    let text = input.read(cx).text().to_string();
    let bad = !text.trim().is_empty() && !valid(&text);
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs))
        .w(px(t.shape.control_height * 5.5));
    col = col.child(input);
    if bad {
        col = col.child(
            div()
                .text_size(px(t.typography.size_xs))
                .text_color(hsla(t.palette.danger))
                .child(hint),
        );
    }
    col
}

#[cfg(test)]
mod tests {
    #[test]
    fn typed_dates_and_times() {
        use super::*;
        assert_eq!(parse_date("2025-03-14"), Some(Date::new(2025, 3, 14)));
        assert_eq!(parse_date("3/14/2025"), Some(Date::new(2025, 3, 14)));
        assert_eq!(parse_date("2025-02-30"), None);
        assert_eq!(parse_date("2024-02-29"), Some(Date::new(2024, 2, 29)));
        assert_eq!(parse_date("nonsense"), None);
        assert_eq!(parse_date("2025-13-01"), None);
        assert_eq!(parse_time("14:05"), Some((14, 5)));
        assert_eq!(parse_time("2:05 pm"), Some((14, 5)));
        assert_eq!(parse_time("12:00 AM"), Some((0, 0)));
        assert_eq!(parse_time("12:30 PM"), Some((12, 30)));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("9:75"), None);
        assert_eq!(format_date(Date::new(2025, 3, 4)), "2025-03-04");
        assert_eq!(
            parse_date(&format_date(Date::new(1999, 12, 31))),
            Some(Date::new(1999, 12, 31))
        );
    }

    #[test]
    fn range_pick_cycle() {
        let d = |n| Date::new(2025, 3, n);
        let r = range_pick(DateRange::default(), d(10));
        assert_eq!((r.start, r.end), (Some(d(10)), None));
        let r = range_pick(r, d(5)); // earlier restarts
        assert_eq!((r.start, r.end), (Some(d(5)), None));
        let r = range_pick(r, d(9));
        assert_eq!(r.span(), Some((d(5), d(9))));
        assert!(r.contains(d(7)) && !r.contains(d(10)));
        let r = range_pick(r, d(20)); // third click restarts
        assert_eq!((r.start, r.end), (Some(d(20)), None));
    }

    use super::*;

    #[test]
    fn leap_and_lengths() {
        assert!(is_leap(2024) && !is_leap(1900) && is_leap(2000));
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2023, 4), 30);
    }

    #[test]
    fn weekdays_known_dates() {
        // 2024-01-01 Monday, 2000-01-01 Saturday, 2077-12-10 Friday.
        assert_eq!(weekday(2024, 1, 1), 0);
        assert_eq!(weekday(2000, 1, 1), 5);
        assert_eq!(weekday(2077, 12, 10), 4);
    }

    #[test]
    fn grid_covers_every_day_once() {
        for (y, m) in [(2024, 2), (2023, 2), (2024, 9), (2077, 12)] {
            let days: Vec<u32> = month_grid(y, m)
                .iter()
                .flatten()
                .flatten()
                .copied()
                .collect();
            assert_eq!(days, (1..=days_in_month(y, m)).collect::<Vec<_>>());
        }
        assert_eq!(month_grid(2024, 1)[0][0], Some(1));
    }

    #[test]
    fn month_shift_wraps_years() {
        assert_eq!(shift_month(2024, 12, 1), (2025, 1));
        assert_eq!(shift_month(2024, 1, -1), (2023, 12));
        assert_eq!(shift_month(2024, 5, 0), (2024, 5));
    }

    #[test]
    fn time_stepping() {
        assert_eq!(step_hour(23, 1), 0);
        assert_eq!(step_hour(0, -1), 23);
        assert_eq!(step_minute(58, 5), 3);
        assert_eq!(step_minute(2, -5), 57);
        assert_eq!(to_12h(0), (12, false));
        assert_eq!(to_12h(13), (1, true));
        assert_eq!(to_12h(12), (12, true));
    }
}
