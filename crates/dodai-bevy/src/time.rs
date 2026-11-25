//! Time and date: clock, stopwatch, countdown timer, tap tempo, count-up readout, date and time
//! inputs and pickers. The maths is [`dodai_core::clock_math`] and [`dodai_core::calendar`].

#![allow(clippy::type_complexity)]

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use dodai_core::calendar::{self, Date, DateRange};
use dodai_core::clock_math;
use dodai_core::{ButtonVariant, CardVariant};

use crate::kit::{kit, kit_pressable, Kit};
use crate::parts::{column, glyph, row, Glyph};
use crate::widgets::{
    button, text, text_input_with, ControlEvent, ControlKind, ControlPointer, TextRole, UiProgress,
    UiText, UiTextInput,
};

type Spawner<'a> = ChildSpawner<'a>;

// ------------------------------------------------------------------ clock

/// An analogue clock with a digital readout under it.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiClock {
    /// UTC offset in minutes (`None`: UTC).
    pub offset_min: Option<i32>,
    /// A fixed Unix time instead of the system clock (tests, screenshots).
    pub fixed: Option<f64>,
}

impl UiClock {
    /// Unix seconds this clock shows now.
    pub fn now(&self) -> f64 {
        self.fixed.unwrap_or_else(clock_math::unix_now)
    }

    /// Its UTC offset in minutes.
    pub fn offset(&self) -> i32 {
        self.offset_min.unwrap_or(0)
    }
}

/// A hand of a [`clock`]: `0` hour, `1` minute, `2` second.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ClockHand(pub u8);

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct ClockReadout;

/// A round clock of `size` px.
pub fn clock(size: f32) -> impl Bundle {
    (
        UiClock {
            offset_min: None,
            fixed: None,
        },
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                kit(Kit::Avatar),
                Node {
                    width: Val::Px(size),
                    height: Val::Px(size),
                    ..default()
                },
                Children::spawn(SpawnWith(move |d: &mut Spawner| {
                    // (length, thickness) of the hour, minute and second hands.
                    for (i, (len, w)) in [(0.28, 4.0), (0.40, 3.0), (0.44, 1.5)]
                        .into_iter()
                        .enumerate()
                    {
                        d.spawn((
                            ClockHand(i as u8),
                            Node {
                                position_type: PositionType::Absolute,
                                left: Val::Px(0.0),
                                top: Val::Px(0.0),
                                width: Val::Px(size),
                                height: Val::Px(size),
                                ..default()
                            },
                            Pickable::IGNORE,
                            children![(
                                kit(Kit::Dot(None)),
                                Node {
                                    position_type: PositionType::Absolute,
                                    left: Val::Px(size / 2.0 - w / 2.0),
                                    top: Val::Px(size / 2.0 - size * len),
                                    width: Val::Px(w),
                                    height: Val::Px(size * len),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            )],
                        ));
                    }
                })),
            ));
            c.spawn((ClockReadout, text("--:--:--", TextRole::Mono)));
        })),
    )
}

pub(crate) fn clock_tick(
    clocks: Query<(&UiClock, &Children)>,
    dials: Query<&Children, Without<UiClock>>,
    mut hands: Query<(&ClockHand, &mut UiTransform)>,
    mut readouts: Query<&mut UiText, With<ClockReadout>>,
) {
    for (c, kids) in &clocks {
        let secs = clock_math::day_secs(c.now(), c.offset());
        let (h, m, s) = clock_math::hand_angles(secs);
        let shown = clock_math::hms(secs as u64);
        for k in kids.iter() {
            if let Ok(mut t) = readouts.get_mut(k) {
                if t.raw != shown {
                    t.raw = shown.clone();
                }
            }
            if let Ok(parts) = dials.get(k) {
                for p in parts.iter() {
                    if let Ok((hand, mut tf)) = hands.get_mut(p) {
                        let a = [h, m, s][usize::from(hand.0)];
                        let want = Rot2::radians(a);
                        if tf.rotation != want {
                            tf.rotation = want;
                        }
                    }
                }
            }
        }
    }
}

// ------------------------------------------------------------------ stopwatch / timer / tap

/// What a button of a stopwatch, timer or tap tempo does.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeAction {
    Start,
    Lap,
    Reset,
    Minus,
    Plus,
    Tap,
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TimeReadout;

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct LapList;

/// Chronometer state.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct UiStopwatch {
    /// Counting.
    pub running: bool,
    /// Milliseconds so far.
    pub ms: u64,
    /// Total at each lap press.
    pub laps: Vec<u64>,
    carry: f64,
}

fn action(label: &'static str, a: TimeAction, v: ButtonVariant) -> impl Bundle {
    (button(label, v), a)
}

/// A stopwatch: start/stop, lap and reset, with the newest laps listed.
pub fn stopwatch() -> impl Bundle {
    (
        UiStopwatch::default(),
        Node {
            row_gap: Val::Px(8.0),
            ..column()
        },
        Children::spawn(SpawnWith(|c: &mut Spawner| {
            c.spawn((
                TimeReadout,
                text(clock_math::stopwatch_text(0), TextRole::Heading),
            ));
            c.spawn((
                row(8.0),
                Children::spawn(SpawnWith(|r: &mut Spawner| {
                    r.spawn(action("Start", TimeAction::Start, ButtonVariant::Primary));
                    r.spawn(action("Lap", TimeAction::Lap, ButtonVariant::Outline));
                    r.spawn(action("Reset", TimeAction::Reset, ButtonVariant::Ghost));
                })),
            ));
            c.spawn((
                LapList,
                column(),
                Children::spawn(SpawnWith(|_: &mut Spawner| {})),
            ));
        })),
    )
}

fn press_of(p: &ControlPointer) -> bool {
    *p == ControlPointer::Pressed
}

pub(crate) fn stopwatch_tick(
    time: Res<Time>,
    mut q: Query<(&mut UiStopwatch, &Children)>,
    mut readouts: Query<&mut UiText, With<TimeReadout>>,
    mut lists: Query<&mut UiText, (With<LapList>, Without<TimeReadout>)>,
) {
    for (mut s, kids) in &mut q {
        if s.running {
            s.carry += f64::from(time.delta_secs()) * 1000.0;
            let whole = s.carry as u64;
            s.carry -= whole as f64;
            s.ms += whole;
        }
        let shown = clock_math::stopwatch_text(s.ms);
        for k in kids.iter() {
            if let Ok(mut t) = readouts.get_mut(k) {
                if t.raw != shown {
                    t.raw = shown.clone();
                }
            }
            let _ = &mut lists;
        }
    }
}

pub(crate) fn stopwatch_buttons(
    pressed: Query<(&TimeAction, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut watches: Query<&mut UiStopwatch>,
    mut labels: Query<&mut UiText>,
    kids: Query<&Children>,
) {
    for (a, p, row_parent) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(row_parent.parent()) else {
            continue;
        };
        let Ok(mut s) = watches.get_mut(root.parent()) else {
            continue;
        };
        match a {
            TimeAction::Start => s.running = !s.running,
            TimeAction::Lap if s.running => {
                let ms = s.ms;
                s.laps.push(ms);
            }
            TimeAction::Reset => {
                s.running = false;
                s.ms = 0;
                s.carry = 0.0;
                s.laps.clear();
            }
            _ => {}
        }
        // The Start button reads Start or Stop.
        if let Ok(buttons) = kids.get(row_parent.parent()) {
            for b in buttons.iter() {
                if let Ok(bc) = kids.get(b) {
                    for l in bc.iter() {
                        if let Ok(mut t) = labels.get_mut(l) {
                            if t.raw == "Start" || t.raw == "Stop" {
                                t.raw = if s.running { "Stop" } else { "Start" }.into();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Splits of a stopwatch as `lap  split` text lines, newest first (at most `n`).
pub fn lap_lines(totals: &[u64], n: usize) -> Vec<String> {
    let splits = clock_math::lap_splits(totals);
    (0..splits.len())
        .rev()
        .take(n)
        .map(|i| format!("{:>2}  {}", i + 1, clock_math::stopwatch_text(splits[i])))
        .collect()
}

/// Countdown state.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct UiTimer {
    /// Counting down.
    pub running: bool,
    /// Milliseconds the countdown started from.
    pub total_ms: u64,
    /// Milliseconds left.
    pub left_ms: u64,
    carry: f64,
}

/// A countdown reached zero.
#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerFinished {
    /// The timer.
    pub entity: Entity,
}

/// A countdown of `minutes` with a progress bar, start/pause, reset and ±1 min.
pub fn timer(minutes: u64) -> impl Bundle {
    let total = minutes * 60_000;
    (
        UiTimer {
            running: false,
            total_ms: total,
            left_ms: total,
            carry: 0.0,
        },
        Node {
            row_gap: Val::Px(8.0),
            ..column()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                TimeReadout,
                text(clock_math::countdown_text(total), TextRole::Heading),
            ));
            c.spawn((crate::widgets::progress(1.0, None), TimerBar));
            c.spawn((
                row(8.0),
                Children::spawn(SpawnWith(|r: &mut Spawner| {
                    r.spawn(action("-1 min", TimeAction::Minus, ButtonVariant::Ghost));
                    r.spawn(action("Start", TimeAction::Start, ButtonVariant::Primary));
                    r.spawn(action("Reset", TimeAction::Reset, ButtonVariant::Outline));
                    r.spawn(action("+1 min", TimeAction::Plus, ButtonVariant::Ghost));
                })),
            ));
        })),
    )
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TimerBar;

pub(crate) fn timer_tick(
    time: Res<Time>,
    mut q: Query<(Entity, &mut UiTimer, &Children)>,
    mut readouts: Query<&mut UiText, With<TimeReadout>>,
    mut bars: Query<&mut UiProgress, With<TimerBar>>,
    mut done: MessageWriter<TimerFinished>,
) {
    for (e, mut t, kids) in &mut q {
        if t.running {
            t.carry += f64::from(time.delta_secs()) * 1000.0;
            let whole = t.carry as u64;
            t.carry -= whole as f64;
            t.left_ms = t.left_ms.saturating_sub(whole);
            if t.left_ms == 0 {
                t.running = false;
                done.write(TimerFinished { entity: e });
            }
        }
        let shown = clock_math::countdown_text(t.left_ms);
        let frac = if t.total_ms == 0 {
            0.0
        } else {
            t.left_ms as f32 / t.total_ms as f32
        };
        for k in kids.iter() {
            if let Ok(mut r) = readouts.get_mut(k) {
                if r.raw != shown {
                    r.raw = shown.clone();
                }
            }
            if let Ok(mut b) = bars.get_mut(k) {
                if (b.value - frac).abs() > 1e-4 {
                    b.value = frac;
                }
            }
        }
    }
}

pub(crate) fn timer_buttons(
    pressed: Query<(&TimeAction, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut timers: Query<&mut UiTimer>,
    mut labels: Query<&mut UiText>,
    kids: Query<&Children>,
) {
    for (a, p, row_parent) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(row_parent.parent()) else {
            continue;
        };
        let Ok(mut t) = timers.get_mut(root.parent()) else {
            continue;
        };
        match a {
            TimeAction::Start if t.left_ms > 0 => t.running = !t.running,
            TimeAction::Reset => {
                t.running = false;
                t.left_ms = t.total_ms;
            }
            TimeAction::Plus => {
                t.total_ms += 60_000;
                t.left_ms += 60_000;
            }
            TimeAction::Minus => {
                t.total_ms = t.total_ms.saturating_sub(60_000).max(60_000);
                t.left_ms = t.left_ms.saturating_sub(60_000).min(t.total_ms);
            }
            _ => {}
        }
        if let Ok(buttons) = kids.get(row_parent.parent()) {
            for b in buttons.iter() {
                if let Ok(bc) = kids.get(b) {
                    for l in bc.iter() {
                        if let Ok(mut lt) = labels.get_mut(l) {
                            if lt.raw == "Start" || lt.raw == "Pause" {
                                lt.raw = if t.running { "Pause" } else { "Start" }.into();
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Tap tempo: press along with the music and read the BPM.
#[derive(Component, Debug, Clone, Default, PartialEq)]
pub struct UiTapTempo {
    /// Tap times, seconds.
    pub taps: Vec<f64>,
}

impl UiTapTempo {
    /// The tempo of the taps so far.
    pub fn bpm(&self) -> Option<f32> {
        clock_math::tap_bpm(&self.taps)
    }
}

/// A tap button with the BPM and the tempo's name.
pub fn tap_tempo() -> impl Bundle {
    (
        UiTapTempo::default(),
        Node {
            row_gap: Val::Px(8.0),
            ..column()
        },
        Children::spawn(SpawnWith(|c: &mut Spawner| {
            c.spawn((TimeReadout, text("-- BPM", TextRole::Heading)));
            c.spawn((
                row(8.0),
                Children::spawn(SpawnWith(|r: &mut Spawner| {
                    r.spawn(action("Tap", TimeAction::Tap, ButtonVariant::Primary));
                    r.spawn(action("Reset", TimeAction::Reset, ButtonVariant::Ghost));
                })),
            ));
        })),
    )
}

pub(crate) fn tap_buttons(
    time: Res<Time>,
    pressed: Query<(&TimeAction, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut taps: Query<(&mut UiTapTempo, &Children)>,
    mut readouts: Query<&mut UiText, With<TimeReadout>>,
) {
    for (a, p, row_parent) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(row_parent.parent()) else {
            continue;
        };
        let Ok((mut t, kids)) = taps.get_mut(root.parent()) else {
            continue;
        };
        match a {
            TimeAction::Tap => t.taps.push(time.elapsed_secs_f64()),
            TimeAction::Reset => t.taps.clear(),
            _ => continue,
        }
        let shown = match t.bpm() {
            Some(b) => format!("{b:.0} BPM · {}", clock_math::tempo_name(b)),
            None => "-- BPM".to_string(),
        };
        for k in kids.iter() {
            if let Ok(mut r) = readouts.get_mut(k) {
                r.raw = shown.clone();
            }
        }
    }
}

/// A number that counts up from 0 to `target` in about a second.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct UiCountUp {
    /// The final value.
    pub target: f32,
    /// Seconds since it started.
    pub t: f32,
    /// Seconds the count takes.
    pub secs: f32,
}

/// A readout that counts up to `target`.
pub fn count_up(target: f32) -> impl Bundle {
    (
        UiCountUp {
            target,
            t: 0.0,
            secs: 1.0,
        },
        text("0", TextRole::Heading),
    )
}

pub(crate) fn count_ups(time: Res<Time>, mut q: Query<(&mut UiCountUp, &mut UiText)>) {
    for (mut c, mut t) in &mut q {
        if c.t < c.secs {
            c.t = (c.t + time.delta_secs()).min(c.secs);
        }
        let f = (c.t / c.secs.max(0.001)).clamp(0.0, 1.0);
        // ease-out cubic
        let eased = 1.0 - (1.0 - f).powi(3);
        let v = c.target * eased;
        let shown = if c.target.fract() == 0.0 {
            format!("{}", v.round() as i64)
        } else {
            format!("{v:.1}")
        };
        if t.raw != shown {
            t.raw = shown;
        }
    }
}

// ------------------------------------------------------------------ date and time inputs

/// `YYYY-MM-DD` as a date; `None` for anything that is not a real day.
pub fn parse_date(s: &str) -> Option<Date> {
    let mut p = s.trim().split('-');
    let (y, m, d) = (p.next()?, p.next()?, p.next()?);
    if p.next().is_some() || y.len() != 4 {
        return None;
    }
    let (y, m, d) = (
        y.parse::<i32>().ok()?,
        m.parse::<u32>().ok()?,
        d.parse::<u32>().ok()?,
    );
    ((1..=12).contains(&m) && d >= 1 && d <= calendar::days_in_month(y, m))
        .then(|| Date::new(y, m, d))
}

/// `HH:MM` as `(hour, minute)`.
pub fn parse_time(s: &str) -> Option<(u32, u32)> {
    let (h, m) = s.trim().split_once(':')?;
    let (h, m) = (h.parse::<u32>().ok()?, m.parse::<u32>().ok()?);
    (h < 24 && m < 60 && !h.to_string().is_empty()).then_some((h, m))
}

/// A date as `YYYY-MM-DD`.
pub fn format_date(d: Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year, d.month, d.day)
}

/// The date a [`date_input`] holds (`None` while the text is not a date).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiDateInput {
    /// The parsed value.
    pub value: Option<Date>,
}

/// The time a [`time_input`] holds.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UiTimeInput {
    /// `(hour, minute)`.
    pub value: Option<(u32, u32)>,
}

/// A text field for a date typed as `YYYY-MM-DD`.
pub fn date_input() -> impl Bundle {
    (
        UiDateInput::default(),
        text_input_with("YYYY-MM-DD", |n| n.min_width = Val::Px(140.0)),
    )
}

/// A text field for a time typed as `HH:MM`.
pub fn time_input() -> impl Bundle {
    (
        UiTimeInput::default(),
        text_input_with("HH:MM", |n| n.min_width = Val::Px(100.0)),
    )
}

pub(crate) fn parse_inputs(
    mut dates: Query<(&UiTextInput, &mut UiDateInput), Changed<UiTextInput>>,
    mut times: Query<(&UiTextInput, &mut UiTimeInput), Changed<UiTextInput>>,
) {
    for (i, mut d) in &mut dates {
        let v = parse_date(&i.value);
        if d.value != v {
            d.value = v;
        }
    }
    for (i, mut t) in &mut times {
        let v = parse_time(&i.value);
        if t.value != v {
            t.value = v;
        }
    }
}

// ------------------------------------------------------------------ pickers

/// A month calendar: pick a day (or a range).
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiDatePicker {
    /// Shown year.
    pub year: i32,
    /// Shown month, 1-12.
    pub month: u32,
    /// The picked day.
    pub selected: Option<Date>,
    /// Picks a span of days instead of one.
    pub range: bool,
    /// The picked span.
    pub picked: DateRange,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DayCell {
    pub row: usize,
    pub col: usize,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MonthStep(pub i32);

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct MonthTitle;

fn picker(date: Date, range: bool) -> impl Bundle {
    (
        UiDatePicker {
            year: date.year,
            month: date.month,
            selected: (!range).then_some(date),
            range,
            picked: DateRange::default(),
        },
        kit(Kit::Card(CardVariant::Outlined)),
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            align_self: AlignSelf::FlexStart,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn((
                Node {
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Children::spawn(SpawnWith(|h: &mut Spawner| {
                    h.spawn((
                        kit_pressable(Kit::Chip { selected: false }),
                        MonthStep(-1),
                        children![glyph(Glyph::ChevronLeft, 16)],
                    ));
                    h.spawn((MonthTitle, text("", TextRole::Control)));
                    h.spawn((
                        kit_pressable(Kit::Chip { selected: false }),
                        MonthStep(1),
                        children![glyph(Glyph::ChevronRight, 16)],
                    ));
                })),
            ));
            c.spawn((
                row(2.0),
                Children::spawn(SpawnWith(|w: &mut Spawner| {
                    for d in calendar::WEEKDAYS {
                        w.spawn((
                            Node {
                                width: Val::Px(36.0),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            children![text(d, TextRole::Caption)],
                        ));
                    }
                })),
            ));
            for r in 0..6 {
                c.spawn((
                    row(2.0),
                    Children::spawn(SpawnWith(move |w: &mut Spawner| {
                        for col in 0..7 {
                            w.spawn((
                                DayCell { row: r, col },
                                kit_pressable(Kit::Page { selected: false }),
                                Node {
                                    width: Val::Px(36.0),
                                    height: Val::Px(32.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                                children![text("", TextRole::Control)],
                            ));
                        }
                    })),
                ));
            }
        })),
    )
}

/// A month calendar showing `date`'s month with `date` picked.
pub fn date_picker(date: Date) -> impl Bundle {
    picker(date, false)
}

/// A month calendar where two clicks pick a span of days.
pub fn date_time_range_picker(date: Date) -> impl Bundle {
    picker(date, true)
}

pub(crate) fn month_steps(
    pressed: Query<(&MonthStep, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut pickers: Query<&mut UiDatePicker>,
) {
    for (s, p, head) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(head.parent()) else {
            continue;
        };
        if let Ok(mut d) = pickers.get_mut(root.parent()) {
            let (y, m) = calendar::shift_month(d.year, d.month, s.0);
            d.year = y;
            d.month = m;
        }
    }
}

pub(crate) fn day_presses(
    pressed: Query<(&DayCell, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut pickers: Query<(Entity, &mut UiDatePicker)>,
    mut out: MessageWriter<ControlEvent>,
) {
    for (cell, p, week) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(week.parent()) else {
            continue;
        };
        let Ok((e, mut d)) = pickers.get_mut(root.parent()) else {
            continue;
        };
        let grid = calendar::month_grid(d.year, d.month);
        let Some(Some(day)) = grid.get(cell.row).map(|w| w[cell.col]) else {
            continue;
        };
        let date = Date::new(d.year, d.month, day);
        if d.range {
            d.picked = calendar::range_pick(d.picked, date);
        } else {
            d.selected = Some(date);
        }
        out.write(ControlEvent {
            entity: e,
            kind: ControlKind::Selected,
        });
    }
}

pub(crate) fn picker_cells(
    pickers: Query<(&UiDatePicker, &Children), Changed<UiDatePicker>>,
    kids: Query<&Children>,
    mut cells: Query<(&DayCell, &mut crate::kit::UiKit, &mut Node, &Children)>,
    mut titles: Query<&mut UiText, With<MonthTitle>>,
    mut labels: Query<&mut UiText, Without<MonthTitle>>,
) {
    for (d, root_kids) in &pickers {
        let grid = calendar::month_grid(d.year, d.month);
        for rk in root_kids.iter() {
            let Ok(inner) = kids.get(rk) else { continue };
            for k in inner.iter() {
                if let Ok(mut t) = titles.get_mut(k) {
                    let s = format!("{} {}", calendar::MONTHS[(d.month - 1) as usize], d.year);
                    if t.raw != s {
                        t.raw = s;
                    }
                }
                if let Ok((c, mut kit, mut node, cell_kids)) = cells.get_mut(k) {
                    let day = grid.get(c.row).and_then(|w| w[c.col]);
                    let on = day.is_some_and(|day| {
                        let date = Date::new(d.year, d.month, day);
                        if d.range {
                            d.picked.contains(date)
                        } else {
                            d.selected == Some(date)
                        }
                    });
                    let next = Kit::Page { selected: on };
                    if kit.0 != next {
                        kit.0 = next;
                    }
                    let disp = if day.is_some() {
                        Display::Flex
                    } else {
                        Display::None
                    };
                    if node.display != disp {
                        node.display = disp;
                    }
                    for l in cell_kids.iter() {
                        if let Ok(mut t) = labels.get_mut(l) {
                            let s = day.map(|x| x.to_string()).unwrap_or_default();
                            if t.raw != s {
                                t.raw = s;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Hour and minute.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiTimePicker {
    /// `0..24`.
    pub hour: u32,
    /// `0..60`.
    pub minute: u32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TimeField {
    Hour(i32),
    Minute(i32),
}

#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct TimeDigits;

/// Hour and minute with a plus and a minus under each.
pub fn time_picker(hour: u32, minute: u32) -> impl Bundle {
    (
        UiTimePicker {
            hour: hour % 24,
            minute: minute % 60,
        },
        kit(Kit::Card(CardVariant::Outlined)),
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            align_self: AlignSelf::FlexStart,
            ..default()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            let step = |f: TimeField, g: Glyph| {
                (
                    kit_pressable(Kit::Chip { selected: false }),
                    f,
                    children![glyph(g, 16)],
                )
            };
            c.spawn((
                row(24.0),
                Children::spawn(SpawnWith(move |r: &mut Spawner| {
                    r.spawn(step(TimeField::Hour(1), Glyph::ChevronUp));
                    r.spawn(step(TimeField::Minute(1), Glyph::ChevronUp));
                })),
            ));
            c.spawn((
                TimeDigits,
                text(
                    format!("{:02}:{:02}", hour % 24, minute % 60),
                    TextRole::Heading,
                ),
            ));
            c.spawn((
                row(24.0),
                Children::spawn(SpawnWith(move |r: &mut Spawner| {
                    r.spawn(step(TimeField::Hour(-1), Glyph::ChevronDown));
                    r.spawn(step(TimeField::Minute(-1), Glyph::ChevronDown));
                })),
            ));
        })),
    )
}

pub(crate) fn time_steps(
    pressed: Query<(&TimeField, &ControlPointer, &ChildOf), Changed<ControlPointer>>,
    parents: Query<&ChildOf>,
    mut pickers: Query<&mut UiTimePicker>,
) {
    for (f, p, rowp) in &pressed {
        if !press_of(p) {
            continue;
        }
        let Ok(root) = parents.get(rowp.parent()) else {
            continue;
        };
        if let Ok(mut t) = pickers.get_mut(root.parent()) {
            match *f {
                TimeField::Hour(d) => t.hour = calendar::step_hour(t.hour, d),
                TimeField::Minute(d) => t.minute = calendar::step_minute(t.minute, d),
            }
        }
    }
}

pub(crate) fn time_digits(
    pickers: Query<(&UiTimePicker, &Children), Changed<UiTimePicker>>,
    mut digits: Query<&mut UiText, With<TimeDigits>>,
) {
    for (t, kids) in &pickers {
        for k in kids.iter() {
            if let Ok(mut d) = digits.get_mut(k) {
                let s = format!("{:02}:{:02}", t.hour, t.minute);
                if d.raw != s {
                    d.raw = s;
                }
            }
        }
    }
}

/// A calendar over a time picker.
pub fn date_time_picker(date: Date, hour: u32, minute: u32) -> impl Bundle {
    (
        Node {
            row_gap: Val::Px(8.0),
            align_self: AlignSelf::FlexStart,
            ..column()
        },
        Children::spawn(SpawnWith(move |c: &mut Spawner| {
            c.spawn(date_picker(date));
            c.spawn(time_picker(hour, minute));
        })),
    )
}

/// Registers the systems above.
pub(crate) fn add(app: &mut App) {
    app.add_message::<TimerFinished>().add_systems(
        Update,
        (
            clock_tick,
            stopwatch_buttons,
            stopwatch_tick,
            timer_buttons,
            timer_tick,
            tap_buttons,
            count_ups,
            parse_inputs,
            month_steps,
            day_presses,
            picker_cells,
            time_steps,
            time_digits,
        )
            .chain()
            .after(crate::kit::press_input),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        crate::testing::component_app()
    }

    fn press(app: &mut App, e: Entity) {
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Hovered;
        *app.world_mut().get_mut::<ControlPointer>(e).unwrap() = ControlPointer::Pressed;
    }

    #[test]
    fn dates_and_times_parse_strictly() {
        assert_eq!(parse_date("2024-02-29"), Some(Date::new(2024, 2, 29)));
        assert_eq!(parse_date("2023-02-29"), None);
        assert_eq!(parse_date("2023-13-01"), None);
        assert_eq!(parse_date("23-1-1"), None);
        assert_eq!(parse_date("2023-01-01-5"), None);
        assert_eq!(parse_time("07:05"), Some((7, 5)));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("7"), None);
        assert_eq!(format_date(Date::new(2025, 3, 9)), "2025-03-09");
    }

    #[test]
    fn date_input_follows_its_text() {
        let mut app = app();
        let e = app.world_mut().spawn(date_input()).id();
        app.update();
        app.world_mut().get_mut::<UiTextInput>(e).unwrap().value = "2025-05-14".into();
        app.update();
        assert_eq!(
            app.world().get::<UiDateInput>(e).unwrap().value,
            Some(Date::new(2025, 5, 14))
        );
        app.world_mut().get_mut::<UiTextInput>(e).unwrap().value = "nope".into();
        app.update();
        assert_eq!(app.world().get::<UiDateInput>(e).unwrap().value, None);
    }

    #[test]
    fn clock_hands_point_at_the_fixed_time() {
        let mut app = app();
        let e = app.world_mut().spawn(clock(100.0)).id();
        // 03:00:00 UTC
        {
            let mut c = app.world_mut().get_mut::<UiClock>(e).unwrap();
            c.fixed = Some(3.0 * 3600.0);
            c.offset_min = Some(0);
        }
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(e).unwrap().iter().collect();
        assert_eq!(app.world().get::<UiText>(kids[1]).unwrap().raw, "03:00:00");
        let dial_parts: Vec<Entity> = app
            .world()
            .get::<Children>(kids[0])
            .unwrap()
            .iter()
            .collect();
        let hour = app
            .world()
            .get::<UiTransform>(dial_parts[0])
            .unwrap()
            .rotation;
        assert!(
            (hour.as_radians() - std::f32::consts::FRAC_PI_2).abs() < 1e-3,
            "{hour:?}"
        );
        let minute = app
            .world()
            .get::<UiTransform>(dial_parts[1])
            .unwrap()
            .rotation;
        assert!(minute.as_radians().abs() < 1e-3);
    }

    #[test]
    fn stopwatch_runs_laps_and_resets() {
        let mut app = app();
        let s = app.world_mut().spawn(stopwatch()).id();
        app.update();
        let buttons_row = app.world().get::<Children>(s).unwrap()[1];
        let b: Vec<Entity> = app
            .world()
            .get::<Children>(buttons_row)
            .unwrap()
            .iter()
            .collect();
        press(&mut app, b[0]);
        app.update();
        assert!(app.world().get::<UiStopwatch>(s).unwrap().running);
        {
            let mut w = app.world_mut().get_mut::<UiStopwatch>(s).unwrap();
            w.ms = 1500;
        }
        press(&mut app, b[1]);
        app.update();
        assert_eq!(app.world().get::<UiStopwatch>(s).unwrap().laps.len(), 1);
        press(&mut app, b[2]);
        app.update();
        let w = app.world().get::<UiStopwatch>(s).unwrap();
        assert!(!w.running && w.ms == 0 && w.laps.is_empty());
        assert_eq!(
            lap_lines(&[1000, 2500], 5),
            vec![" 2  00:01.50", " 1  00:01.00"]
        );
    }

    #[test]
    fn timer_counts_down_and_adjusts_minutes() {
        let mut app = app();
        let t = app.world_mut().spawn(timer(2)).id();
        app.update();
        let row_e = app.world().get::<Children>(t).unwrap()[2];
        let b: Vec<Entity> = app.world().get::<Children>(row_e).unwrap().iter().collect();
        press(&mut app, b[3]);
        app.update();
        assert_eq!(app.world().get::<UiTimer>(t).unwrap().total_ms, 180_000);
        press(&mut app, b[0]);
        app.update();
        assert_eq!(app.world().get::<UiTimer>(t).unwrap().total_ms, 120_000);
        press(&mut app, b[1]);
        app.update();
        assert!(app.world().get::<UiTimer>(t).unwrap().running);
        app.world_mut().get_mut::<UiTimer>(t).unwrap().left_ms = 1;
        std::thread::sleep(std::time::Duration::from_millis(5));
        app.update();
        app.update();
        assert!(!app.world().get::<UiTimer>(t).unwrap().running, "finished");
        assert_eq!(app.world().get::<UiTimer>(t).unwrap().left_ms, 0);
    }

    #[test]
    fn tap_tempo_reads_a_bpm() {
        let mut t = UiTapTempo::default();
        assert_eq!(t.bpm(), None);
        t.taps = vec![0.0, 0.5, 1.0, 1.5];
        assert_eq!(t.bpm().map(|b| b.round()), Some(120.0));
    }

    #[test]
    fn count_up_reaches_its_target() {
        let mut app = app();
        let e = app.world_mut().spawn(count_up(40.0)).id();
        app.update();
        app.world_mut().get_mut::<UiCountUp>(e).unwrap().t = 5.0;
        app.update();
        assert_eq!(app.world().get::<UiText>(e).unwrap().raw, "40");
    }

    #[test]
    fn date_picker_shows_the_month_and_picks_days() {
        let mut app = app();
        let p = app
            .world_mut()
            .spawn(date_picker(Date::new(2025, 3, 10)))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(p).unwrap().iter().collect();
        let head: Vec<Entity> = app
            .world()
            .get::<Children>(kids[0])
            .unwrap()
            .iter()
            .collect();
        assert_eq!(
            app.world().get::<UiText>(head[1]).unwrap().raw,
            "March 2025"
        );
        // next month
        press(&mut app, head[2]);
        app.update();
        app.update();
        assert_eq!(
            app.world().get::<UiText>(head[1]).unwrap().raw,
            "April 2025"
        );
        // pick the first real day of April 2025 (Tuesday = column 1)
        let week0: Vec<Entity> = app
            .world()
            .get::<Children>(kids[2])
            .unwrap()
            .iter()
            .collect();
        press(&mut app, week0[1]);
        app.update();
        app.update();
        assert_eq!(
            app.world().get::<UiDatePicker>(p).unwrap().selected,
            Some(Date::new(2025, 4, 1))
        );
        // column 0 of that week is not a day of April
        assert_eq!(
            app.world().get::<Node>(week0[0]).unwrap().display,
            Display::None
        );
    }

    #[test]
    fn range_picker_takes_two_clicks() {
        let mut app = app();
        let p = app
            .world_mut()
            .spawn(date_time_range_picker(Date::new(2025, 4, 1)))
            .id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(p).unwrap().iter().collect();
        let week0: Vec<Entity> = app
            .world()
            .get::<Children>(kids[2])
            .unwrap()
            .iter()
            .collect();
        let week1: Vec<Entity> = app
            .world()
            .get::<Children>(kids[3])
            .unwrap()
            .iter()
            .collect();
        press(&mut app, week0[1]);
        app.update();
        press(&mut app, week1[2]);
        app.update();
        let r = app.world().get::<UiDatePicker>(p).unwrap().picked;
        assert_eq!(
            r.span(),
            Some((Date::new(2025, 4, 1), Date::new(2025, 4, 9)))
        );
    }

    #[test]
    fn time_picker_wraps() {
        let mut app = app();
        let p = app.world_mut().spawn(time_picker(23, 59)).id();
        app.update();
        let kids: Vec<Entity> = app.world().get::<Children>(p).unwrap().iter().collect();
        let up: Vec<Entity> = app
            .world()
            .get::<Children>(kids[0])
            .unwrap()
            .iter()
            .collect();
        press(&mut app, up[0]);
        app.update();
        press(&mut app, up[1]);
        app.update();
        app.update();
        let t = *app.world().get::<UiTimePicker>(p).unwrap();
        assert_eq!((t.hour, t.minute), (0, 0));
        assert_eq!(app.world().get::<UiText>(kids[1]).unwrap().raw, "00:00");
    }

    #[test]
    fn date_time_picker_holds_both() {
        let mut app = app();
        let p = app
            .world_mut()
            .spawn(date_time_picker(Date::new(2025, 1, 1), 8, 30))
            .id();
        app.update();
        assert_eq!(app.world().get::<Children>(p).unwrap().len(), 2);
    }
}
