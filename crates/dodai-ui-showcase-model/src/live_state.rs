//! The running widgets of Data display (countdown timer, stopwatch, tap tempo): their state lives
//! in the showcase values, so every toolkit drives the same model with plain button presses.
//! Times are seconds since the showcase started ([`crate::widgets::elapsed`]); every function
//! takes that `now`, so tests can drive the clock.

use crate::{Value, Values};
use dodai_ui_core::clock_math::{beat_phase, tap_bpm};

/// Prefix of every press and value key here.
pub const PREFIX: &str = "live:";

/// Timer: start / pause.
pub const TIMER_GO: &str = "live:timer-go";
/// Timer: back to the full time.
pub const TIMER_RESET: &str = "live:timer-reset";
/// Timer: one minute less.
pub const TIMER_MINUS: &str = "live:timer-minus";
/// Timer: one minute more.
pub const TIMER_PLUS: &str = "live:timer-plus";
/// Stopwatch: start / stop.
pub const SW_GO: &str = "live:sw-go";
/// Stopwatch: record a lap.
pub const SW_LAP: &str = "live:sw-lap";
/// Stopwatch: back to zero.
pub const SW_RESET: &str = "live:sw-reset";
/// Tap tempo: a tap.
pub const TAP: &str = "live:tap";
/// Tap tempo: one BPM less.
pub const TAP_MINUS: &str = "live:tap-minus";
/// Tap tempo: one BPM more.
pub const TAP_PLUS: &str = "live:tap-plus";
/// Tap tempo: play / stop the beat lights.
pub const TAP_PLAY: &str = "live:tap-play";
/// Tap tempo: forget the taps.
pub const TAP_RESET: &str = "live:tap-reset";

const T_MIN: &str = "live:t-min";
const T_ACC: &str = "live:t-acc";
const T_AT: &str = "live:t-at";
const S_ACC: &str = "live:s-acc";
const S_AT: &str = "live:s-at";
const S_LAPS: &str = "live:s-laps";
const P_TAPS: &str = "live:p-taps";
const P_BPM: &str = "live:p-bpm";
const P_PLAY: &str = "live:p-play";

/// Beat lights of the tap pad.
pub const BEATS: usize = 4;
/// Laps the stopwatch lists (newest first).
pub const MAX_LAPS: usize = 5;

fn num(v: &Values, k: &str) -> Option<f32> {
    match v.0.get(k) {
        Some(Value::Num(x)) => Some(*x),
        _ => None,
    }
}

fn put(v: &mut Values, k: &str, x: Option<f32>) {
    match x {
        Some(x) => v.set(k, Value::Num(x)),
        None => {
            v.0.remove(k);
        }
    }
}

fn list<T: std::str::FromStr>(v: &Values, k: &str) -> Vec<T> {
    match v.0.get(k) {
        Some(Value::Text(s)) => s.split(',').filter_map(|x| x.parse().ok()).collect(),
        _ => Vec::new(),
    }
}

fn put_list<T: ToString>(v: &mut Values, k: &str, xs: &[T]) {
    let s: Vec<String> = xs.iter().map(ToString::to_string).collect();
    v.set(k, Value::Text(s.join(",")));
}

fn ms(secs: f32) -> u64 {
    (secs.max(0.0) * 1000.0) as u64
}

/// What the countdown shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimerView {
    /// The set time.
    pub total_ms: u64,
    /// What is left.
    pub left_ms: u64,
    /// Counting down now.
    pub running: bool,
    /// Ran out.
    pub done: bool,
}

/// The countdown at `now`.
pub fn timer(v: &Values, now: f32) -> TimerView {
    let total_ms = u64::from(minutes(v)) * 60_000;
    let run = num(v, T_AT).map_or(0.0, |at| now - at);
    let gone = ms(num(v, T_ACC).unwrap_or(0.0) + run).min(total_ms);
    let done = gone >= total_ms;
    TimerView {
        total_ms,
        left_ms: total_ms - gone,
        running: num(v, T_AT).is_some() && !done,
        done,
    }
}

fn minutes(v: &Values) -> u32 {
    num(v, T_MIN).map_or(1, |m| m as u32)
}

/// What the stopwatch shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StopwatchView {
    /// Time on the watch.
    pub ms: u64,
    /// Counting now.
    pub running: bool,
    /// The time at each lap press.
    pub laps: Vec<u64>,
}

/// The stopwatch at `now`.
pub fn stopwatch(v: &Values, now: f32) -> StopwatchView {
    let run = num(v, S_AT).map_or(0.0, |at| now - at);
    StopwatchView {
        ms: ms(num(v, S_ACC).unwrap_or(0.0) + run),
        running: num(v, S_AT).is_some(),
        laps: list(v, S_LAPS),
    }
}

/// What the tap pad shows.
#[derive(Debug, Clone, PartialEq)]
pub struct TapView {
    /// The tempo, once two taps gave one.
    pub bpm: Option<f32>,
    /// The tap times.
    pub taps: Vec<f64>,
    /// While playing: the lit beat and how far through it.
    pub beat: Option<(usize, f32)>,
}

/// The tap pad at `now`.
pub fn tap(v: &Values, now: f32) -> TapView {
    let bpm = num(v, P_BPM);
    let beat = match (num(v, P_PLAY), bpm) {
        (Some(at), Some(b)) => {
            let (i, f) = beat_phase(f64::from(now - at), b);
            Some(((i % BEATS as u64) as usize, f))
        }
        _ => None,
    };
    TapView {
        bpm,
        taps: list(v, P_TAPS),
        beat,
    }
}

/// Whether something here moves on its own at `now` (the toolkit should keep redrawing).
pub fn running(v: &Values, now: f32) -> bool {
    timer(v, now).running || num(v, S_AT).is_some() || tap(v, now).beat.is_some()
}

/// Apply a press `cmd` at `now`; `false` when it is not one of these.
pub fn press(v: &mut Values, cmd: &str, now: f32) -> bool {
    match cmd {
        TIMER_GO => {
            let t = timer(v, now);
            if let Some(at) = num(v, T_AT) {
                put(v, T_ACC, Some(num(v, T_ACC).unwrap_or(0.0) + now - at));
                put(v, T_AT, None);
            } else if !t.done {
                put(v, T_AT, Some(now));
            }
        }
        TIMER_RESET => {
            put(v, T_ACC, None);
            put(v, T_AT, None);
        }
        TIMER_MINUS | TIMER_PLUS => {
            if !timer(v, now).running {
                let d: i32 = if cmd == TIMER_PLUS { 1 } else { -1 };
                let m = (minutes(v) as i32 + d).clamp(1, 99);
                put(v, T_MIN, Some(m as f32));
                put(v, T_ACC, None);
                put(v, T_AT, None);
            }
        }
        SW_GO => {
            if let Some(at) = num(v, S_AT) {
                put(v, S_ACC, Some(num(v, S_ACC).unwrap_or(0.0) + now - at));
                put(v, S_AT, None);
            } else {
                put(v, S_AT, Some(now));
            }
        }
        SW_LAP => {
            let s = stopwatch(v, now);
            if s.running {
                let mut laps = s.laps;
                laps.push(s.ms);
                put_list(v, S_LAPS, &laps);
            }
        }
        SW_RESET => {
            put(v, S_ACC, None);
            put(v, S_AT, None);
            v.0.remove(S_LAPS);
        }
        TAP => {
            let now = f64::from(now);
            let mut taps: Vec<f64> = list(v, P_TAPS);
            // A pause of more than 2.5 s starts over.
            if taps.last().is_some_and(|l| now - l > 2.5) {
                taps.clear();
            }
            taps.push(now);
            if taps.len() > 12 {
                taps.remove(0);
            }
            if let Some(b) = tap_bpm(&taps) {
                put(v, P_BPM, Some(b.clamp(20.0, 300.0)));
            }
            put_list(v, P_TAPS, &taps);
        }
        TAP_MINUS | TAP_PLUS => {
            if let Some(b) = num(v, P_BPM) {
                let d = if cmd == TAP_PLUS { 1.0 } else { -1.0 };
                put(v, P_BPM, Some((b + d).clamp(20.0, 300.0)));
            }
        }
        TAP_PLAY => {
            let playing = num(v, P_PLAY).is_some();
            put(v, P_PLAY, (!playing).then_some(now));
        }
        TAP_RESET => {
            for k in [P_TAPS, P_BPM, P_PLAY] {
                v.0.remove(k);
            }
        }
        _ => return false,
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_timer_counts_down_pauses_and_runs_out() {
        let mut v = Values::default();
        assert_eq!(timer(&v, 0.0).left_ms, 60_000);
        press(&mut v, TIMER_GO, 1.0);
        let t = timer(&v, 11.0);
        assert!(t.running && t.left_ms == 50_000);
        press(&mut v, TIMER_GO, 11.0);
        assert_eq!(timer(&v, 99.0).left_ms, 50_000);
        press(&mut v, TIMER_GO, 100.0);
        let t = timer(&v, 200.0);
        assert!(t.done && !t.running && t.left_ms == 0);
        press(&mut v, TIMER_PLUS, 200.0);
        assert_eq!(timer(&v, 200.0).total_ms, 120_000);
        press(&mut v, TIMER_MINUS, 200.0);
        press(&mut v, TIMER_MINUS, 200.0);
        assert_eq!(timer(&v, 200.0).total_ms, 60_000);
    }

    #[test]
    fn the_stopwatch_adds_up_runs_and_keeps_laps() {
        let mut v = Values::default();
        press(&mut v, SW_LAP, 0.0);
        assert!(stopwatch(&v, 0.0).laps.is_empty());
        press(&mut v, SW_GO, 1.0);
        press(&mut v, SW_LAP, 2.0);
        press(&mut v, SW_GO, 3.0);
        press(&mut v, SW_GO, 10.0);
        let s = stopwatch(&v, 11.0);
        assert_eq!((s.ms, s.laps.clone()), (3000, vec![1000]));
        assert!(running(&v, 11.0));
        press(&mut v, SW_RESET, 11.0);
        assert_eq!(stopwatch(&v, 12.0).ms, 0);
    }

    #[test]
    fn taps_give_a_tempo_and_play_lights_the_beats() {
        let mut v = Values::default();
        for i in 0..4 {
            press(&mut v, TAP, i as f32 * 0.5);
        }
        assert_eq!(tap(&v, 2.0).bpm.map(f32::round), Some(120.0));
        press(&mut v, TAP_PLUS, 2.0);
        assert_eq!(tap(&v, 2.0).bpm.map(f32::round), Some(121.0));
        press(&mut v, TAP_PLAY, 2.0);
        assert!(tap(&v, 2.1).beat.is_some_and(|(i, _)| i == 0));
        press(&mut v, TAP_RESET, 3.0);
        assert_eq!(
            tap(&v, 3.0),
            TapView {
                bpm: None,
                taps: vec![],
                beat: None
            }
        );
        assert!(!running(&v, 3.0));
    }
}
