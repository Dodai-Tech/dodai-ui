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

pub use dodai_core::calendar::{
    days_in_month, is_leap, month_grid, range_pick, shift_month, step_hour, step_minute, to_12h,
    weekday, Date, DateRange, MONTHS, WEEKDAYS,
};

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
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), self.selected);
        self.selected = own.get();
        let (state, inner) = (own.setter(), self.on_select.take());
        self.on_select = Some(std::rc::Rc::new(move |d, w, cx| {
            crate::own::choose(&state, cx, Some(d));
            if let Some(f) = &inner {
                f(d, w, cx);
            }
        }));
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
        let cell = t.scale(dodai_core::Scale::Compact);
        let weekdays = div().flex().children(WEEKDAYS.iter().map(|d| {
            div()
                .w(px(cell))
                .flex()
                .justify_center()
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(*d)
        }));
        let mut grid = div().flex().flex_col().gap(px(t.spacing.half()));
        for (ri, row) in month_grid(y, m).into_iter().enumerate() {
            let mut r = div().flex().gap(gpui::Pixels::ZERO);
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
                let mut el = track(shaped_id(id, &t.shape, style), &st, cx)
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
    fn build(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let own = crate::own::Own::new(window, cx, self.id.clone(), (self.hour, self.minute));
        (self.hour, self.minute) = own.get();
        let (state, inner) = (own.setter(), self.on_change.take());
        self.on_change = Some(std::rc::Rc::new(move |h, m, w, cx| {
            crate::own::choose(&state, cx, (h, m));
            if let Some(f) = &inner {
                f(h, m, w, cx);
            }
        }));
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
                .w(px(t.scale(dodai_core::Scale::Wide)))
                .h(px(t.scale(dodai_core::Scale::Tall)))
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

// ───────────────────────────── date + time ─────────────────────────────

/// `YYYY-MM-DD HH:MM` for a date and time of day, or just the time when no date is chosen.
pub fn format_date_time(d: Option<Date>, hour: u32, minute: u32) -> String {
    let time = format!("{:02}:{:02}", hour % 24, minute % 60);
    match d {
        Some(d) => format!("{:04}-{:02}-{:02} {time}", d.year, d.month, d.day),
        None => time,
    }
}

/// Controlled calendar and time of day in one panel, with the chosen moment read out below.
#[derive(IntoElement)]
pub struct DateTimePicker {
    id: ElementId,
    year: i32,
    month: u32,
    selected: Option<Date>,
    hour: u32,
    minute: u32,
    twelve: bool,
    minute_step: i32,
    on_date: Option<DateFn>,
    on_month: Option<MonthFn>,
    on_time: Option<TimeFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(DateTimePicker);

impl DateTimePicker {
    /// `year`/`month` (1-12) is the visible month; the time starts at 00:00.
    pub fn new(id: impl Into<ElementId>, year: i32, month: u32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            year,
            month: month.clamp(1, 12),
            selected: None,
            hour: 0,
            minute: 0,
            twelve: false,
            minute_step: 5,
            on_date: None,
            on_month: None,
            on_time: None,
        }
    }
    /// The selected day (controlled).
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    /// The time of day (controlled).
    pub fn time(mut self, hour: u32, minute: u32) -> Self {
        self.hour = hour % 24;
        self.minute = minute % 60;
        self
    }
    /// 12-hour display with AM/PM toggle.
    pub fn twelve_hour(mut self, on: bool) -> Self {
        self.twelve = on;
        self
    }
    /// Minute step of the time stepper.
    pub fn minute_step(mut self, s: i32) -> Self {
        self.minute_step = s.max(1);
        self
    }
    crate::callback!(
        /// Called when a day is chosen.
        on_date, Date
    );
    crate::callback!(
        /// Called with the new visible `(year, month)` when the arrows are used.
        on_month, i32, u32
    );
    crate::callback!(
        /// Called when the time changes, with hour and minute.
        on_time, u32, u32
    );
}

impl DateTimePicker {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = t.panel(false);
        let mut cal = DatePicker::new((self.id.clone(), "date"), self.year, self.month)
            .selected(self.selected);
        cal.corners = self.corners;
        cal.on_select = self.on_date.clone();
        cal.on_month = self.on_month.clone();
        let mut time = TimePicker::new((self.id.clone(), "time"), self.hour, self.minute)
            .twelve_hour(self.twelve)
            .minute_step(self.minute_step);
        time.corners = self.corners;
        time.on_change = self.on_time.clone();
        let style = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true);
        shaped(&t.shape, style)
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_color(hsla(t.palette.text))
            .child(cal)
            .child(time)
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(format_date_time(self.selected, self.hour, self.minute)),
            )
    }
}

crate::entry_render!(DateTimePicker);

// ───────────────────────────── date-time range ─────────────────────────────

/// `YYYY-MM-DD HH:MM → YYYY-MM-DD HH:MM` for a picked span with a start and an end time.
/// While the end day is pending only the start is read out; with no day, a prompt.
pub fn format_date_time_range(r: DateRange, start: (u32, u32), end: (u32, u32)) -> String {
    match (r.start, r.end) {
        (Some(a), Some(b)) => format!(
            "{} → {}",
            format_date_time(Some(a), start.0, start.1),
            format_date_time(Some(b), end.0, end.1)
        ),
        (Some(a), None) => format!(
            "{} → pick the end day",
            format_date_time(Some(a), start.0, start.1)
        ),
        _ => "pick the start day, then the end day".to_string(),
    }
}

/// Controlled range calendar with a start and an end time of day in one panel, the chosen
/// span read out below. Feed day clicks through [`range_pick`] in `on_select`.
#[derive(IntoElement)]
pub struct DateTimeRangePicker {
    id: ElementId,
    year: i32,
    month: u32,
    range: DateRange,
    start: (u32, u32),
    end: (u32, u32),
    twelve: bool,
    minute_step: i32,
    on_select: Option<DateFn>,
    on_month: Option<MonthFn>,
    on_start_time: Option<TimeFn>,
    on_end_time: Option<TimeFn>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(DateTimeRangePicker);

impl DateTimeRangePicker {
    /// `year`/`month` (1-12) is the visible month; both times start at 00:00.
    pub fn new(id: impl Into<ElementId>, year: i32, month: u32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            year,
            month: month.clamp(1, 12),
            range: DateRange::default(),
            start: (0, 0),
            end: (0, 0),
            twelve: false,
            minute_step: 5,
            on_select: None,
            on_month: None,
            on_start_time: None,
            on_end_time: None,
        }
    }
    /// The picked span of days (controlled).
    pub fn range(mut self, r: DateRange) -> Self {
        self.range = r;
        self
    }
    /// Time of day on the first day (controlled).
    pub fn start_time(mut self, hour: u32, minute: u32) -> Self {
        self.start = (hour % 24, minute % 60);
        self
    }
    /// Time of day on the last day (controlled).
    pub fn end_time(mut self, hour: u32, minute: u32) -> Self {
        self.end = (hour % 24, minute % 60);
        self
    }
    /// 12-hour display with AM/PM toggle.
    pub fn twelve_hour(mut self, on: bool) -> Self {
        self.twelve = on;
        self
    }
    /// Minute step of both time steppers.
    pub fn minute_step(mut self, s: i32) -> Self {
        self.minute_step = s.max(1);
        self
    }
    crate::callback!(
        /// Called when a day is clicked; pass it to [`range_pick`].
        on_select, Date
    );
    crate::callback!(
        /// Called with the new visible `(year, month)` when the arrows are used.
        on_month, i32, u32
    );
    crate::callback!(
        /// Called when the start time changes, with hour and minute.
        on_start_time, u32, u32
    );
    crate::callback!(
        /// Called when the end time changes, with hour and minute.
        on_end_time, u32, u32
    );
}

impl DateTimeRangePicker {
    fn build(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let p = t.panel(false);
        let mut cal =
            DatePicker::new((self.id.clone(), "date"), self.year, self.month).range(self.range);
        cal.corners = self.corners;
        cal.on_select = self.on_select.clone();
        cal.on_month = self.on_month.clone();
        let clock = |key: &'static str, (h, m): (u32, u32), f: Option<TimeFn>| {
            let mut tp = TimePicker::new((self.id.clone(), key), h, m)
                .twelve_hour(self.twelve)
                .minute_step(self.minute_step);
            tp.corners = self.corners;
            tp.on_change = f;
            tp
        };
        let caption = |s: &'static str| {
            div()
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(s)
        };
        let times = div()
            .flex()
            .flex_wrap()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .child(caption("Start"))
                    .child(clock("start", self.start, self.on_start_time.clone())),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(t.spacing.xs))
                    .child(caption("End"))
                    .child(clock("end", self.end, self.on_end_time.clone())),
            );
        let style = ShapeStyle::from_tokens(&t, Some(p.background), Some(p.border), false, true);
        shaped(&t.shape, style)
            .p(px(t.spacing.sm))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(t.spacing.sm))
            .font_family(first_family(t.typography.body))
            .text_color(hsla(t.palette.text))
            .child(cal)
            .child(times)
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(format_date_time_range(self.range, self.start, self.end)),
            )
    }
}

crate::entry_render!(DateTimeRangePicker);

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
    disabled: bool,
    on_change: Option<DateChange>,
}

impl DateInput {
    /// Create a new [`DateInput`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            disabled: false,
            on_change: None,
        }
    }
    /// The current (controlled) date; changing it rewrites the field.
    pub fn value(mut self, d: Option<Date>) -> Self {
        self.value = d;
        self
    }
    /// Disable the field.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
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
            self.disabled,
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
    disabled: bool,
    on_change: Option<TimeChange>,
}

impl TimeInput {
    /// Create a new [`TimeInput`].
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: None,
            disabled: false,
            on_change: None,
        }
    }
    /// Current value.
    pub fn value(mut self, hm: Option<(u32, u32)>) -> Self {
        self.value = hm;
        self
    }
    /// Disable the field.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
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
            self.disabled,
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
    disabled: bool,
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
    input.update(cx, |ti, cx| ti.set_disabled(disabled, cx));
    let text = input.read(cx).text().to_string();
    let bad = !text.trim().is_empty() && !valid(&text);
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(t.spacing.xs))
        .w(px(t.scale(dodai_core::Scale::Popup)));
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
    fn date_time_reads_out_both_parts() {
        use super::*;
        assert_eq!(
            format_date_time(Some(Date::new(2025, 3, 4)), 9, 5),
            "2025-03-04 09:05"
        );
        assert_eq!(format_date_time(None, 25, 61), "01:01");
    }

    #[test]
    fn date_time_range_reads_out_each_stage() {
        use super::*;
        let a = Date::new(2025, 3, 4);
        let b = Date::new(2025, 3, 9);
        let none = DateRange::default();
        assert_eq!(
            format_date_time_range(none, (9, 0), (17, 30)),
            "pick the start day, then the end day"
        );
        let half = range_pick(none, a);
        assert_eq!(
            format_date_time_range(half, (9, 0), (17, 30)),
            "2025-03-04 09:00 → pick the end day"
        );
        assert_eq!(
            format_date_time_range(range_pick(half, b), (9, 0), (17, 30)),
            "2025-03-04 09:00 → 2025-03-09 17:30"
        );
    }

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

    use super::*;

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
