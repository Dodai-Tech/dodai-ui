//! Time components: [`Clock`] (analog + digital), [`Timer`] (countdown),
//! [`Stopwatch`] (chronometer with laps) and [`TapTempo`] (measure the BPM of
//! music by tapping, with a visual metronome).
//!
//! All state lives in GPUI keyed state, so give each one a unique id. They
//! redraw every frame while running and are idle otherwise.

use crate::button::Button;
use crate::display::CircularProgress;
use crate::paint::shaped;
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, Tone};
use gpui::{
    canvas, div, point, px, App, ElementId, IntoElement, ParentElement, PathBuilder, RenderOnce,
    SharedString, Styled, Window,
};
use std::rc::Rc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ───────────────────────────── pure helpers ─────────────────────────────

/// `HH:MM:SS` from whole seconds (wraps at 24 h).
pub fn hms(secs: u64) -> String {
    let s = secs % 86_400;
    format!("{:02}:{:02}:{:02}", s / 3600, s / 60 % 60, s % 60)
}

/// Chronometer text: `MM:SS.cc`, or `H:MM:SS.cc` from one hour on.
pub fn stopwatch_text(ms: u64) -> String {
    let (cs, s) = (ms / 10 % 100, ms / 1000);
    if s >= 3600 {
        format!("{}:{:02}:{:02}.{:02}", s / 3600, s / 60 % 60, s % 60, cs)
    } else {
        format!("{:02}:{:02}.{:02}", s / 60, s % 60, cs)
    }
}

/// Countdown text, rounding **up** so it reads `00:00` only when done.
pub fn countdown_text(ms: u64) -> String {
    let s = ms.div_ceil(1000);
    if s >= 3600 {
        hms(s)
    } else {
        format!("{:02}:{:02}", s / 60, s % 60)
    }
}

/// Clock-hand angles in radians, clockwise from 12 o'clock, for seconds since
/// midnight: `(hour, minute, second)`.
pub fn hand_angles(day_secs: f64) -> (f32, f32, f32) {
    let t = std::f64::consts::TAU;
    let d = day_secs.rem_euclid(86_400.0);
    (
        ((d / 43_200.0) * t) as f32,
        ((d % 3600.0) / 3600.0 * t) as f32,
        ((d % 60.0) / 60.0 * t) as f32,
    )
}

/// Split totals (ms at each lap press) into per-lap durations.
pub fn lap_splits(totals: &[u64]) -> Vec<u64> {
    let mut prev = 0;
    totals
        .iter()
        .map(|&t| {
            let d = t.saturating_sub(prev);
            prev = t;
            d
        })
        .collect()
}

/// BPM from tap timestamps (seconds). Uses the last 8 intervals; `None` below two taps.
pub fn tap_bpm(taps: &[f64]) -> Option<f32> {
    if taps.len() < 2 {
        return None;
    }
    let from = taps.len().saturating_sub(9);
    let w = &taps[from..];
    let span = w[w.len() - 1] - w[0];
    if span <= 0.0 {
        return None;
    }
    Some((60.0 * (w.len() - 1) as f64 / span) as f32)
}

/// Steadiness of the taps: standard deviation of the intervals relative to
/// their mean (0 = perfectly even). `None` below three taps.
pub fn tap_jitter(taps: &[f64]) -> Option<f32> {
    if taps.len() < 3 {
        return None;
    }
    let iv: Vec<f64> = taps.windows(2).map(|w| w[1] - w[0]).collect();
    let mean = iv.iter().sum::<f64>() / iv.len() as f64;
    if mean <= 0.0 {
        return None;
    }
    let var = iv.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / iv.len() as f64;
    Some((var.sqrt() / mean) as f32)
}

/// Italian tempo marking for a BPM.
pub fn tempo_name(bpm: f32) -> &'static str {
    match bpm {
        b if b < 40.0 => "Grave",
        b if b < 60.0 => "Largo",
        b if b < 76.0 => "Adagio",
        b if b < 108.0 => "Andante",
        b if b < 120.0 => "Moderato",
        b if b < 156.0 => "Allegro",
        b if b < 176.0 => "Vivace",
        _ => "Presto",
    }
}

/// Beat index (counting from 0) and fraction through it after `elapsed` seconds.
pub fn beat_phase(elapsed: f64, bpm: f32) -> (u64, f32) {
    let beats = elapsed.max(0.0) * bpm as f64 / 60.0;
    (beats.floor() as u64, beats.fract() as f32)
}

/// Seconds since midnight for a Unix time and UTC offset in minutes.
pub fn day_secs(unix: f64, offset_min: i32) -> f64 {
    (unix + offset_min as f64 * 60.0).rem_euclid(86_400.0)
}

/// Lit segments (bits a,b,c,d,e,f,g = 0..6) of a seven-segment digit.
pub fn segments(d: u8) -> u8 {
    const T: [u8; 10] = [
        0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
        0b1111111, 0b1101111,
    ];
    T[(d % 10) as usize]
}

/// Civil date `(year, month 1-12, day 1-31)` from days since 1970-01-01.
pub fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// Weekday name from days since 1970-01-01 (a Thursday).
pub fn weekday_name(days: i64) -> &'static str {
    ["THU", "FRI", "SAT", "SUN", "MON", "TUE", "WED"][days.rem_euclid(7) as usize]
}

/// `THU · 14 MAY 2025` for a Unix time and UTC offset in minutes.
pub fn date_text(unix: f64, offset_min: i32) -> String {
    const M: [&str; 12] = [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ];
    let days = ((unix + offset_min as f64 * 60.0) / 86_400.0).floor() as i64;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{} · {} {} {}",
        weekday_name(days),
        d,
        M[(m - 1) as usize],
        y
    )
}

fn unix_now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// The machine's current UTC offset in minutes (0 where it can't be read).
pub fn local_utc_offset_minutes() -> i32 {
    #[cfg(unix)]
    {
        #[repr(C)]
        struct Tm {
            f: [i32; 9],
            gmtoff: i64,
            zone: *const u8,
        }
        extern "C" {
            fn localtime_r(t: *const i64, out: *mut Tm) -> *mut Tm;
        }
        let t = unix_now() as i64;
        let mut tm = Tm {
            f: [0; 9],
            gmtoff: 0,
            zone: std::ptr::null(),
        };
        // SAFETY: `Tm` matches the C `struct tm` layout on glibc/musl/macOS/BSD;
        // `localtime_r` only writes into it and is thread-safe.
        let ok = unsafe { !localtime_r(&t, &mut tm).is_null() };
        if ok {
            return (tm.gmtoff / 60) as i32;
        }
    }
    0
}

type VoidCb = Rc<dyn Fn(&mut Window, &mut App)>;
type Cb<T> = Rc<dyn Fn(T, &mut Window, &mut App)>;

fn id2(id: &ElementId, s: &'static str) -> ElementId {
    (id.clone(), s).into()
}

// ───────────────────────────── clock ─────────────────────────────

/// Which clock face to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ClockFace {
    /// Follow the theme: cut-corner themes get the futuristic digital face,
    /// the others the analog dial.
    #[default]
    Auto,
    Analog,
    /// Glowing seven-segment readout with ghost segments and a seconds bar.
    Digital,
}

impl ClockFace {
    /// Whether this face resolves to the digital readout for a theme shape.
    pub fn is_digital(self, kind: dodai_core::ShapeKind) -> bool {
        match self {
            ClockFace::Digital => true,
            ClockFace::Analog => false,
            ClockFace::Auto => kind == dodai_core::ShapeKind::Chamfered,
        }
    }
}

/// Live clock: analog dial, or a futuristic seven-segment face (see [`ClockFace`]).
///
/// ```ignore
/// Clock::new("clock").size(180.0).local()
/// ```
#[derive(IntoElement)]
pub struct Clock {
    face: ClockFace,
    size: f32,
    seconds: bool,
    digital: bool,
    offset_min: i32,
}

impl Clock {
    pub fn new(_id: impl Into<ElementId>) -> Self {
        Self {
            face: ClockFace::Auto,
            size: 180.0,
            seconds: true,
            digital: true,
            offset_min: 0,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Pick the face (default [`ClockFace::Auto`]: follows the theme).
    pub fn face(mut self, f: ClockFace) -> Self {
        self.face = f;
        self
    }
    /// Show the second hand (default on).
    pub fn seconds(mut self, on: bool) -> Self {
        self.seconds = on;
        self
    }
    /// Show `HH:MM:SS` under the face (default on).
    pub fn digital(mut self, on: bool) -> Self {
        self.digital = on;
        self
    }
    /// Offset from UTC in minutes (default 0 = UTC).
    pub fn utc_offset_minutes(mut self, m: i32) -> Self {
        self.offset_min = m;
        self
    }
    /// Use this machine's time zone.
    pub fn local(self) -> Self {
        let m = local_utc_offset_minutes();
        self.utc_offset_minutes(m)
    }
}

impl RenderOnce for Clock {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let digital = self.face.is_digital(t.shape.kind());
        if digital {
            return digital_face(&self, &t).into_any_element();
        }
        let (size, seconds, off) = (self.size, self.seconds, self.offset_min);
        let (ink, accent, tick) = (
            hsla(t.palette.text),
            hsla(t.palette.primary),
            hsla(t.palette.border_strong),
        );
        let now = day_secs(unix_now(), off);
        let face = div().relative().flex_none().w(px(size)).h(px(size)).child(
            canvas(
                |_, _, _| (),
                move |b, _, window, _| {
                    let r = size / 2.0;
                    let (o, c) = (b.origin, (size / 2.0, size / 2.0));
                    let (hour, min, sec) = hand_angles(day_secs(unix_now(), off));
                    let pt = |a: f32, len: f32| {
                        // 0 rad = 12 o'clock, clockwise.
                        let (s, co) = a.sin_cos();
                        point(o.x + px(c.0 + len * s), o.y + px(c.1 - len * co))
                    };
                    let mut line = |a: f32, from: f32, to: f32, w: f32, col| {
                        let mut pb = PathBuilder::stroke(px(w));
                        pb.add_polygon(&[pt(a, from), pt(a, to)], false);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, col);
                        }
                    };
                    for i in 0..60 {
                        let a = i as f32 * std::f32::consts::TAU / 60.0;
                        if i % 5 == 0 {
                            line(a, r * 0.84, r * 0.96, 2.0, ink);
                        } else {
                            line(a, r * 0.92, r * 0.96, 1.0, tick);
                        }
                    }
                    line(hour, 0.0, r * 0.5, 4.0, ink);
                    line(min, 0.0, r * 0.74, 3.0, ink);
                    if seconds {
                        line(sec, -r * 0.14, r * 0.86, 1.5, accent);
                    }
                    line(0.0, 0.0, 0.0, 7.0, accent); // hub dot
                    window.request_animation_frame();
                },
            )
            .absolute()
            .size_full(),
        );
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(face);
        if self.digital {
            let s = now as u64;
            let txt = if seconds {
                hms(s)
            } else {
                hms(s)[..5].to_string()
            };
            col = col.child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_xl))
                    .text_color(ink)
                    .child(txt),
            );
        }
        col.into_any_element()
    }
}

/// One seven-segment digit; `slant` shears it forward for a sporty look.
#[allow(clippy::too_many_arguments)]
fn paint_digit(
    window: &mut Window,
    o: gpui::Point<gpui::Pixels>,
    (x, y, w, h, th): (f32, f32, f32, f32, f32),
    digit: u8,
    on: gpui::Hsla,
    off: gpui::Hsla,
    halo: f32,
) {
    let slant = 0.12;
    let pt = |px_: f32, py: f32| point(o.x + px(px_ + (h - py) * slant), o.y + px(py));
    let mut poly = |pts: [(f32, f32); 6], c: gpui::Hsla| {
        let v: Vec<_> = pts.iter().map(|&(a, b)| pt(a, b)).collect();
        let mut pb = PathBuilder::fill();
        pb.add_polygon(&v, true);
        if let Ok(p) = pb.build() {
            window.paint_path(p, c);
        }
    };
    let (g, t2) = (th * 0.16, th * 0.5);
    let (l, r, top, mid, bot) = (x + t2, x + w - t2, y + t2, y + h / 2.0, y + h - t2);
    let horiz = |yc: f32, grow: f32| {
        let (a, b, t3) = (l + g - grow, r - g + grow, t2 + grow);
        [
            (a, yc),
            (a + t3, yc - t3),
            (b - t3, yc - t3),
            (b, yc),
            (b - t3, yc + t3),
            (a + t3, yc + t3),
        ]
    };
    let vert = |xc: f32, y0: f32, y1: f32, grow: f32| {
        let (a, b, t3) = (y0 + g - grow, y1 - g + grow, t2 + grow);
        [
            (xc, a),
            (xc + t3, a + t3),
            (xc + t3, b - t3),
            (xc, b),
            (xc - t3, b - t3),
            (xc - t3, a + t3),
        ]
    };
    let bits = segments(digit);
    let shapes = |grow: f32| {
        [
            horiz(top, grow),
            vert(r, top, mid, grow),
            vert(r, mid, bot, grow),
            horiz(bot, grow),
            vert(l, mid, bot, grow),
            vert(l, top, mid, grow),
            horiz(mid, grow),
        ]
    };
    for (i, s) in shapes(0.0).into_iter().enumerate() {
        if bits & (1 << i) == 0 {
            poly(s, off);
        }
    }
    if halo > 0.0 {
        let mut c = on;
        c.a *= 0.22 * halo;
        for (i, s) in shapes(th * 0.35).into_iter().enumerate() {
            if bits & (1 << i) != 0 {
                poly(s, c);
            }
        }
    }
    for (i, s) in shapes(0.0).into_iter().enumerate() {
        if bits & (1 << i) != 0 {
            poly(s, on);
        }
    }
}

/// Futuristic face: glowing seven-segment `HH:MM` + `SS`, ghost segments, a
/// 60-cell seconds bar and the date, in the theme's panel shape.
fn digital_face(c: &Clock, t: &dodai_core::Theme) -> gpui::Div {
    let (off_min, seconds) = (c.offset_min, c.seconds);
    let width = c.size * 2.0;
    let unit = width / 7.1;
    let (dw, dh) = (unit, unit * 1.9);
    let bar_h = (dh * 0.09).max(4.0);
    let height = dh + bar_h * 2.6;
    let (hi, lo) = (hsla(t.palette.primary), hsla(t.palette.secondary));
    let mut ghost = hsla(t.palette.border);
    ghost.a *= 0.55;
    let halo = 0.4 + t.effects.glow_intensity;
    let now = unix_now();
    let date = format!(
        "{}  ·  UTC{}{:02}:{:02}",
        date_text(now, off_min),
        if off_min < 0 { '-' } else { '+' },
        off_min.abs() / 60,
        off_min.abs() % 60
    );
    let display = div()
        .relative()
        .flex_none()
        .w(px(width))
        .h(px(height))
        .child(
            canvas(
                |_, _, _| (),
                move |b, _, window, _| {
                    let o = b.origin;
                    let secs_f = day_secs(unix_now(), off_min);
                    let s = secs_f as u64;
                    let (hh, mm, ss) = ((s / 3600) as u8, (s / 60 % 60) as u8, (s % 60) as u8);
                    let th = dw * 0.17;
                    let gap = dw * 0.18;
                    let mut x = 0.0;
                    for d in [hh / 10, hh % 10] {
                        paint_digit(window, o, (x, 0.0, dw, dh, th), d, hi, ghost, halo);
                        x += dw + gap;
                    }
                    // blinking colon
                    let lit = secs_f.fract() < 0.5;
                    let slant = |y: f32| (dh - y) * 0.12;
                    for y in [dh * 0.32, dh * 0.68] {
                        let r = th * 0.55;
                        let cx = x + dw * 0.1 + slant(y);
                        let v = [
                            point(o.x + px(cx - r), o.y + px(y)),
                            point(o.x + px(cx), o.y + px(y - r)),
                            point(o.x + px(cx + r), o.y + px(y)),
                            point(o.x + px(cx), o.y + px(y + r)),
                        ];
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&v, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, if lit { hi } else { ghost });
                        }
                    }
                    x += dw * 0.4;
                    for d in [mm / 10, mm % 10] {
                        paint_digit(window, o, (x, 0.0, dw, dh, th), d, hi, ghost, halo);
                        x += dw + gap;
                    }
                    if seconds {
                        x += gap;
                        let (sw, sh) = (dw * 0.55, dh * 0.55);
                        let y0 = dh - sh;
                        for d in [ss / 10, ss % 10] {
                            paint_digit(window, o, (x, y0, sw, sh, th * 0.6), d, lo, ghost, halo);
                            x += sw + gap * 0.6;
                        }
                    }
                    // seconds bar: 60 cells, filled up to the current second
                    let cell = width / 60.0;
                    for i in 0..60u8 {
                        let x0 = i as f32 * cell;
                        let c = if i <= ss {
                            if i == ss {
                                lo
                            } else {
                                hi
                            }
                        } else {
                            ghost
                        };
                        let y0 = dh + bar_h * 1.3;
                        let v = [
                            point(o.x + px(x0 + 1.0), o.y + px(y0)),
                            point(o.x + px(x0 + cell - 1.0), o.y + px(y0)),
                            point(o.x + px(x0 + cell - 1.0), o.y + px(y0 + bar_h)),
                            point(o.x + px(x0 + 1.0), o.y + px(y0 + bar_h)),
                        ];
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&v, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, c);
                        }
                    }
                    window.request_animation_frame();
                },
            )
            .absolute()
            .size_full(),
        );
    let mut frame = shaped(&t.shape, crate::panel::panel_style(t, false))
        .p(px(t.spacing.lg))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(t.spacing.sm))
        .child(display);
    if c.digital {
        frame = frame.child(
            div()
                .font_family(first_family(t.typography.mono))
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(date),
        );
    }
    frame
}

// ───────────────────────────── timer ─────────────────────────────

struct TimerState {
    total_ms: u64,
    elapsed: Duration,
    started: Option<Instant>,
    fired: bool,
}

impl TimerState {
    fn elapsed_ms(&self) -> u64 {
        let live = self.started.map(|s| s.elapsed()).unwrap_or_default();
        ((self.elapsed + live).as_millis() as u64).min(self.total_ms)
    }
}

/// Countdown timer with a progress ring, start/pause/reset and ±1 min.
///
/// ```ignore
/// Timer::new("t").minutes(5).on_finish(|_, _| println!("ding"))
/// ```
#[derive(IntoElement)]
pub struct Timer {
    id: ElementId,
    total_ms: u64,
    size: f32,
    on_finish: Option<VoidCb>,
}

impl Timer {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            total_ms: 60_000,
            size: 160.0,
            on_finish: None,
        }
    }
    pub fn seconds(mut self, s: u64) -> Self {
        self.total_ms = s * 1000;
        self
    }
    pub fn minutes(self, m: u64) -> Self {
        self.seconds(m * 60)
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    crate::callback!(
        /// Called once when the countdown reaches zero.
        on_finish
    );
}

impl RenderOnce for Timer {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let total = self.total_ms;
        let st = window.use_keyed_state(self.id.clone(), cx, move |_, _| TimerState {
            total_ms: total,
            elapsed: Duration::ZERO,
            started: None,
            fired: false,
        });
        let (ms, total, running, done) = {
            let s = st.read(cx);
            (s.elapsed_ms(), s.total_ms, s.started.is_some(), s.fired)
        };
        if running && ms >= total && !done {
            st.update(cx, |s, _| {
                s.elapsed = Duration::from_millis(s.total_ms);
                s.started = None;
                s.fired = true;
            });
            if let Some(f) = &self.on_finish {
                f(window, cx);
            }
        }
        let (running, done) = (st.read(cx).started.is_some(), st.read(cx).fired);
        if running {
            window.request_animation_frame();
        }
        let left = total.saturating_sub(ms);
        let frac = if total == 0 {
            0.0
        } else {
            left as f32 / total as f32
        };
        let tone = if done || frac < 0.1 {
            Some(Tone::Danger)
        } else if running {
            None
        } else {
            Some(Tone::Info)
        };
        let ring = div()
            .relative()
            .flex_none()
            .w(px(self.size))
            .h(px(self.size))
            .flex()
            .items_center()
            .justify_center()
            .child({
                let mut p = CircularProgress::new(frac)
                    .size(self.size)
                    .thickness(7.0)
                    .label(false);
                if let Some(k) = tone {
                    p = p.tone(k);
                }
                p
            })
            .child(
                div()
                    .absolute()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(self.size * 0.2))
                    .text_color(hsla(if done {
                        t.palette.danger
                    } else {
                        t.palette.text
                    }))
                    .child(countdown_text(left)),
            );
        let id = self.id.clone();
        let (s1, s2, s3, s4) = (st.clone(), st.clone(), st.clone(), st.clone());
        let go = Button::new(
            id2(&id, "go"),
            if running {
                "Pause"
            } else if done {
                "Done"
            } else {
                "Start"
            },
        )
        .variant(if running {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Primary
        })
        .disabled(done)
        .on_click(move |_, _, cx| {
            s1.update(cx, |s, cx| {
                if let Some(at) = s.started.take() {
                    s.elapsed += at.elapsed();
                } else if s.elapsed_ms() < s.total_ms {
                    s.started = Some(Instant::now());
                }
                cx.notify();
            })
        });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    s.elapsed = Duration::ZERO;
                    s.started = None;
                    s.fired = false;
                    cx.notify();
                })
            });
        let adj = |label: &'static str, key: &'static str, d: i64, s: gpui::Entity<TimerState>| {
            Button::new(id2(&id, key), label)
                .variant(ButtonVariant::Outline)
                .disabled(running)
                .on_click(move |_, _, cx| {
                    s.update(cx, |s, cx| {
                        s.total_ms =
                            (s.total_ms as i64 + d * 60_000).clamp(60_000, 99 * 60_000) as u64;
                        s.elapsed = Duration::ZERO;
                        s.fired = false;
                        cx.notify();
                    })
                })
        };
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(ring)
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(adj("−1 min", "minus", -1, s3))
                    .child(go)
                    .child(reset)
                    .child(adj("+1 min", "plus", 1, s4)),
            )
    }
}

// ───────────────────────────── stopwatch ─────────────────────────────

struct SwState {
    acc: Duration,
    started: Option<Instant>,
    laps: Vec<u64>,
}

impl SwState {
    fn ms(&self) -> u64 {
        let live = self.started.map(|s| s.elapsed()).unwrap_or_default();
        (self.acc + live).as_millis() as u64
    }
}

/// Chronometer: start/stop, lap, reset; shows the newest laps with splits.
#[derive(IntoElement)]
pub struct Stopwatch {
    id: ElementId,
    max_laps: usize,
}

impl Stopwatch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            max_laps: 5,
        }
    }
    /// How many of the newest laps to list (default 5).
    pub fn max_laps(mut self, n: usize) -> Self {
        self.max_laps = n;
        self
    }
}

impl RenderOnce for Stopwatch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| SwState {
            acc: Duration::ZERO,
            started: None,
            laps: vec![],
        });
        let (ms, running, laps) = {
            let s = st.read(cx);
            (s.ms(), s.started.is_some(), s.laps.clone())
        };
        if running {
            window.request_animation_frame();
        }
        let id = self.id.clone();
        let (s1, s2, s3) = (st.clone(), st.clone(), st.clone());
        let go = Button::new(id2(&id, "go"), if running { "Stop" } else { "Start" })
            .variant(if running {
                ButtonVariant::Danger
            } else {
                ButtonVariant::Primary
            })
            .on_click(move |_, _, cx| {
                s1.update(cx, |s, cx| {
                    if let Some(at) = s.started.take() {
                        s.acc += at.elapsed();
                    } else {
                        s.started = Some(Instant::now());
                    }
                    cx.notify();
                })
            });
        let lap = Button::new(id2(&id, "lap"), "Lap")
            .variant(ButtonVariant::Outline)
            .disabled(!running)
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    let ms = s.ms();
                    s.laps.push(ms);
                    cx.notify();
                })
            });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .disabled(running || (ms == 0 && laps.is_empty()))
            .on_click(move |_, _, cx| {
                s3.update(cx, |s, cx| {
                    s.acc = Duration::ZERO;
                    s.started = None;
                    s.laps.clear();
                    cx.notify();
                })
            });
        let splits = lap_splits(&laps);
        let mut list = div().flex().flex_col().gap(px(2.0)).w_full();
        for i in (0..laps.len()).rev().take(self.max_laps) {
            list = list.child(
                div()
                    .flex()
                    .justify_between()
                    .gap(px(t.spacing.lg))
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(SharedString::from(format!("LAP {}", i + 1)))
                    .child(SharedString::from(format!(
                        "+{}",
                        stopwatch_text(splits[i])
                    )))
                    .child(SharedString::from(stopwatch_text(laps[i]))),
            );
        }
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .font_family(first_family(t.typography.mono))
                    .text_size(px(t.typography.size_xxl * 1.4))
                    .text_color(hsla(t.palette.text))
                    .child(stopwatch_text(ms)),
            )
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .child(go)
                    .child(lap)
                    .child(reset),
            )
            .child(list)
    }
}

// ───────────────────────────── tap tempo ─────────────────────────────

struct TapState {
    t0: Instant,
    taps: Vec<f64>,
    bpm: Option<f32>,
    playing: Option<Instant>,
}

/// Measure the tempo of music: tap along and read the BPM, then press Play to
/// hear/see it back as a visual metronome (a bar of beat lights, beat 1 accented).
#[derive(IntoElement)]
pub struct TapTempo {
    id: ElementId,
    beats: usize,
    on_change: Option<Cb<f32>>,
}

impl TapTempo {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            beats: 4,
            on_change: None,
        }
    }
    /// Beats per bar in the metronome lights (default 4).
    pub fn beats(mut self, n: usize) -> Self {
        self.beats = n.clamp(1, 12);
        self
    }
    crate::callback!(
        /// Called with the new BPM after a tap or a ± adjustment.
        on_change, f32
    );
}

impl RenderOnce for TapTempo {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| TapState {
            t0: Instant::now(),
            taps: vec![],
            bpm: None,
            playing: None,
        });
        let (bpm, taps, playing) = {
            let s = st.read(cx);
            (s.bpm, s.taps.clone(), s.playing)
        };
        let id = self.id.clone();
        let on_change = self.on_change.clone();
        let (s1, s2, s3, s4, s5) = (st.clone(), st.clone(), st.clone(), st.clone(), st.clone());

        let nudge = |label: &'static str, key: &'static str, d: f32, s: gpui::Entity<TapState>| {
            let f = on_change.clone();
            Button::new(id2(&id, key), label)
                .variant(ButtonVariant::Outline)
                .disabled(bpm.is_none())
                .on_click(move |_, w, cx| {
                    let nb = s.update(cx, |s, cx| {
                        let b = (s.bpm.unwrap_or(120.0) + d).clamp(20.0, 300.0);
                        s.bpm = Some(b);
                        cx.notify();
                        b
                    });
                    if let Some(f) = &f {
                        f(nb, w, cx);
                    }
                })
        };
        let f = on_change.clone();
        let tap = Button::new(id2(&id, "tap"), "Tap")
            .variant(ButtonVariant::Primary)
            .refine(|d| d.w(px(120.0)).h(px(56.0)))
            .on_click(move |_, w, cx| {
                let nb = s1.update(cx, |s, cx| {
                    let now = s.t0.elapsed().as_secs_f64();
                    if s.taps.last().is_some_and(|l| now - l > 2.5) {
                        s.taps.clear();
                    }
                    s.taps.push(now);
                    if s.taps.len() > 12 {
                        s.taps.remove(0);
                    }
                    if let Some(b) = tap_bpm(&s.taps) {
                        s.bpm = Some(b.clamp(20.0, 300.0));
                    }
                    cx.notify();
                    s.bpm
                });
                if let (Some(b), Some(f)) = (nb, &f) {
                    f(b, w, cx);
                }
            });
        let reset = Button::new(id2(&id, "reset"), "Reset")
            .variant(ButtonVariant::Outline)
            .disabled(bpm.is_none())
            .on_click(move |_, _, cx| {
                s2.update(cx, |s, cx| {
                    s.taps.clear();
                    s.bpm = None;
                    s.playing = None;
                    cx.notify();
                })
            });
        let play = Button::new(
            id2(&id, "play"),
            if playing.is_some() { "Stop" } else { "Play" },
        )
        .variant(if playing.is_some() {
            ButtonVariant::Danger
        } else {
            ButtonVariant::Secondary
        })
        .disabled(bpm.is_none())
        .on_click(move |_, _, cx| {
            s3.update(cx, |s, cx| {
                s.playing = if s.playing.is_some() {
                    None
                } else {
                    Some(Instant::now())
                };
                cx.notify();
            })
        });

        // Beat lights.
        let (cur, frac) = match (playing, bpm) {
            (Some(at), Some(b)) => {
                window.request_animation_frame();
                let (i, f) = beat_phase(at.elapsed().as_secs_f64(), b);
                (Some((i % self.beats as u64) as usize), f)
            }
            _ => (None, 1.0),
        };
        let mut lights = div().flex().gap(px(t.spacing.sm)).items_center();
        for i in 0..self.beats {
            let base = if i == 0 {
                t.palette.primary
            } else {
                t.palette.secondary
            };
            let lit = if cur == Some(i) {
                (1.0 - frac).powi(2)
            } else {
                0.0
            };
            let c = crate::motion::mix(hsla(t.palette.border), hsla(base), 0.15 + 0.85 * lit);
            lights = lights.child(div().w(px(22.0)).h(px(22.0)).rounded_full().bg(c));
        }

        let readout = match bpm {
            Some(b) => format!("{}", b.round() as u32),
            None => "—".into(),
        };
        let note = match (bpm, tap_jitter(&taps)) {
            (Some(b), Some(j)) => format!(
                "{} · {} taps · {}",
                tempo_name(b),
                taps.len(),
                if j < 0.04 {
                    "steady"
                } else if j < 0.1 {
                    "fairly steady"
                } else {
                    "uneven"
                }
            ),
            (Some(b), None) => format!("{} · {} taps", tempo_name(b), taps.len()),
            _ => "Tap along with the beat".into(),
        };
        let nb = nudge("−1", "m1", -1.0, s4.clone());
        let pb = nudge("+1", "p1", 1.0, s5.clone());
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.md))
            .child(
                div()
                    .flex()
                    .items_end()
                    .gap(px(t.spacing.sm))
                    .child(
                        div()
                            .font_family(first_family(t.typography.display))
                            .text_size(px(t.typography.size_xxl * 1.8))
                            .text_color(hsla(t.palette.text))
                            .child(readout),
                    )
                    .child(
                        div()
                            .pb(px(8.0))
                            .text_size(px(t.typography.size_md))
                            .text_color(hsla(t.palette.text_muted))
                            .child("BPM"),
                    ),
            )
            .child(
                div()
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(note),
            )
            .child(lights)
            .child(
                div()
                    .flex()
                    .gap(px(t.spacing.sm))
                    .items_center()
                    .child(nb)
                    .child(tap)
                    .child(pb),
            )
            .child(div().flex().gap(px(t.spacing.sm)).child(play).child(reset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(hms(3725), "01:02:05");
        assert_eq!(hms(86_400 + 5), "00:00:05");
        assert_eq!(stopwatch_text(61_230), "01:01.23");
        assert_eq!(stopwatch_text(3_661_000), "1:01:01.00");
        assert_eq!(countdown_text(0), "00:00");
        assert_eq!(countdown_text(1), "00:01");
        assert_eq!(countdown_text(60_000), "01:00");
        assert_eq!(countdown_text(3_600_000), "01:00:00");
    }

    #[test]
    fn hands() {
        let (h, m, s) = hand_angles(3.0 * 3600.0);
        assert!((h - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert!(m.abs() < 1e-5 && s.abs() < 1e-5);
        let (_, m, s) = hand_angles(30.0 * 60.0 + 15.0);
        assert!((s - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        assert!(m > std::f32::consts::PI);
        assert!((hand_angles(-1.0).2 - hand_angles(86_399.0).2).abs() < 1e-5);
    }

    #[test]
    fn splits() {
        assert_eq!(lap_splits(&[1000, 2500, 2600]), vec![1000, 1500, 100]);
        assert!(lap_splits(&[]).is_empty());
    }

    #[test]
    fn bpm_from_taps() {
        assert_eq!(tap_bpm(&[1.0]), None);
        let taps: Vec<f64> = (0..6).map(|i| i as f64 * 0.5).collect();
        assert!((tap_bpm(&taps).unwrap() - 120.0).abs() < 0.01);
        // only the last 8 intervals count: slow start is forgotten
        let mut v = vec![0.0, 3.0];
        v.extend((0..9).map(|i| 3.0 + (i + 1) as f64 * 0.6));
        assert!((tap_bpm(&v).unwrap() - 100.0).abs() < 0.01);
    }

    #[test]
    fn jitter() {
        let even: Vec<f64> = (0..5).map(|i| i as f64 * 0.5).collect();
        assert!(tap_jitter(&even).unwrap() < 1e-6);
        assert!(tap_jitter(&[0.0, 0.5, 1.5, 1.6]).unwrap() > 0.3);
        assert_eq!(tap_jitter(&[0.0, 1.0]), None);
    }

    #[test]
    fn tempo_and_beats() {
        assert_eq!(tempo_name(60.0), "Adagio");
        assert_eq!(tempo_name(128.0), "Allegro");
        assert_eq!(tempo_name(200.0), "Presto");
        let (i, f) = beat_phase(1.25, 120.0);
        assert_eq!(i, 2);
        assert!((f - 0.5).abs() < 1e-5);
    }

    #[test]
    fn auto_face_follows_shape_not_theme_id() {
        use dodai_core::ShapeKind::*;
        assert!(ClockFace::Auto.is_digital(Chamfered));
        assert!(!ClockFace::Auto.is_digital(Rounded));
        assert!(!ClockFace::Auto.is_digital(Square));
        assert!(ClockFace::Digital.is_digital(Rounded));
        assert!(!ClockFace::Analog.is_digital(Chamfered));
    }

    #[test]
    fn seven_segments() {
        assert_eq!(segments(8), 0b1111111);
        assert_eq!(segments(1).count_ones(), 2);
        assert_eq!(segments(7).count_ones(), 3);
        assert_eq!(segments(13), segments(3)); // wraps
    }

    #[test]
    fn dates() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(19_723), (2024, 1, 1));
        assert_eq!(civil_from_days(19_782), (2024, 2, 29));
        assert_eq!(civil_from_days(-1), (1969, 12, 31));
        assert_eq!(weekday_name(0), "THU");
        assert_eq!(weekday_name(19_723), "MON"); // 2024-01-01
        assert_eq!(date_text(86_400.0 * 19_723.0 + 5.0, 0), "MON · 1 JAN 2024");
        assert_eq!(date_text(86_400.0 * 19_723.0, -60), "SUN · 31 DEC 2023");
    }

    #[test]
    fn day_seconds() {
        assert_eq!(day_secs(86_400.0 * 3.0 + 10.0, 0), 10.0);
        assert_eq!(day_secs(10.0, -1), 86_400.0 - 50.0);
        assert_eq!(day_secs(0.0, 60), 3600.0);
    }
}
