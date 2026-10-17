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
    pub year: i32,
    pub month: u32,
    pub day: u32,
}

impl Date {
    pub fn new(year: i32, month: u32, day: u32) -> Self {
        Self { year, month, day }
    }
}

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
pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

pub fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

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
    on_select: Option<DateFn>,
    on_month: Option<MonthFn>,
    corners: Option<dodai_core::Corners>,
}

impl DatePicker {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl DatePicker {
    /// `year`/`month` (1-12) is the visible month.
    pub fn new(id: impl Into<ElementId>, year: i32, month: u32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            year,
            month: month.clamp(1, 12),
            selected: None,
            on_select: None,
            on_month: None,
        }
    }
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    pub fn on_select(mut self, f: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
    /// Called with the new visible `(year, month)` when the arrows are used.
    pub fn on_month(mut self, f: impl Fn(i32, u32, &mut Window, &mut App) + 'static) -> Self {
        self.on_month = Some(Rc::new(f));
        self
    }
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
                let on = self.selected == Some(date);
                let id: ElementId = (self.id.clone(), format!("d{ri}{ci}")).into();
                let st = use_interact(&id, window, cx);
                let state = st.read(cx).control_state(false);
                let variant = if on {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                };
                let (mut style, fg) = button_style(&t, variant, state);
                if !on && state == ControlState::Idle {
                    style.stroke = None;
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

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "DatePicker", motion)
    }
}

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

impl TimePicker {
    /// Cut these corners instead of the theme's.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
}

impl TimePicker {
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
    pub fn minute_step(mut self, s: i32) -> Self {
        self.minute_step = s.max(1);
        self
    }
    pub fn on_change(mut self, f: impl Fn(u32, u32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
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

impl RenderOnce for TimePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let motion = crate::motion::Motion::of(cx);
        crate::motion::entry(self.build(window, cx), "TimePicker", motion)
    }
}

#[cfg(test)]
mod tests {
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
