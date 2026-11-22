//! Press commands: buttons that set or step a stored value, pick a date range, and "select
//! all" toggles. A button's counter key carries the command, so every toolkit runs it through
//! [`crate::Action::Press`] (and a toggle's key through [`crate::Action::Set`]) unchanged.

use dodai_ui_core::calendar::{range_pick, Date, DateRange};

use crate::state::{Value, Values};

const SET: &str = "set:";
const STEP: &str = "step:";
const RANGE: &str = "range:";
const ALL: &str = "all:";
const FLIP: &str = "flip:";

/// A button key that stores `n` under `key`.
pub fn set(key: &str, n: usize) -> String {
    format!("{SET}{key}={n}")
}

/// A button key that moves `key` (`default` while unset) by `delta`, wrapping within `lo..=hi`.
pub fn step(key: &str, delta: i64, lo: i64, hi: i64, default: i64) -> String {
    format!("{STEP}{key}:{delta}:{lo}:{hi}:{default}")
}

/// A button key that picks `d` into the date range `key` (first press starts, second ends).
pub fn range(key: &str, d: Date) -> String {
    format!("{RANGE}{key}={}", pack(d))
}

/// A button key that flips the flag `key` (`default` while unset): a disclosure, a popover.
pub fn flip(key: &str, default: bool) -> String {
    format!("{FLIP}{key}={}", u8::from(default))
}

/// A toggle key that sets every one of `keys` at once.
pub fn all(keys: &[String]) -> String {
    format!("{ALL}{}", keys.join(","))
}

/// A date as one stored number (`yyyymmdd`; 0 is none).
pub fn pack(d: Date) -> usize {
    (d.year.max(0) as usize) * 10_000 + d.month as usize * 100 + d.day as usize
}

/// The date stored as `n` (`None` for 0).
pub fn unpack(n: usize) -> Option<Date> {
    (n > 0).then(|| {
        Date::new(
            (n / 10_000) as i32,
            (n / 100 % 100) as u32,
            (n % 100) as u32,
        )
    })
}

/// The date range stored under `key`.
pub fn date_range(values: &Values, key: &str) -> DateRange {
    DateRange {
        start: unpack(values.index_of(&format!("{key}-a"), 0)),
        end: unpack(values.index_of(&format!("{key}-b"), 0)),
    }
}

/// Run the press command `cmd`; `false` when it is not one.
pub(crate) fn press(values: &mut Values, cmd: &str) -> bool {
    if let Some(rest) = cmd.strip_prefix(SET) {
        if let Some((key, n)) = rest.split_once('=') {
            if let Ok(n) = n.parse() {
                values.set(key, Value::Index(n));
            }
        }
        return true;
    }
    if let Some(rest) = cmd.strip_prefix(STEP) {
        let mut parts = rest.rsplitn(5, ':');
        let nums: Vec<i64> = parts
            .by_ref()
            .take(4)
            .filter_map(|p| p.parse().ok())
            .collect();
        if let (Some(key), [default, hi, lo, delta]) = (parts.next(), nums.as_slice()) {
            let now = values.index_of(key, *default as usize) as i64;
            let span = (hi - lo + 1).max(1);
            let next = lo + (now - lo + delta).rem_euclid(span);
            values.set(key, Value::Index(next as usize));
        }
        return true;
    }
    if let Some(rest) = cmd.strip_prefix(FLIP) {
        if let Some((key, default)) = rest.split_once('=') {
            let now = values.flag(&crate::Bind::new(key, default == "1"));
            values.set(key, Value::Bool(!now));
        }
        return true;
    }
    if let Some(rest) = cmd.strip_prefix(RANGE) {
        if let Some((key, d)) = rest.split_once('=') {
            if let Some(d) = d.parse().ok().and_then(unpack) {
                let r = range_pick(date_range(values, key), d);
                values.set(format!("{key}-a"), Value::Index(r.start.map_or(0, pack)));
                values.set(format!("{key}-b"), Value::Index(r.end.map_or(0, pack)));
            }
        }
        return true;
    }
    false
}

/// Run the toggle command `cmd` turned `on`; `false` when it is not one.
pub(crate) fn toggle(values: &mut Values, cmd: &str, on: bool) -> bool {
    let Some(keys) = cmd.strip_prefix(ALL) else {
        return false;
    };
    for k in keys.split(',') {
        values.set(k, Value::Bool(on));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_and_step_store_numbers_and_steps_wrap() {
        let mut v = Values::default();
        assert!(press(&mut v, &set("day", 15)));
        assert_eq!(v.index_of("day", 0), 15);
        let up = step("h", 1, 0, 23, 23);
        assert!(press(&mut v, &up));
        assert_eq!(v.index_of("h", 9), 0, "23 + 1 wraps to 0");
        assert!(press(&mut v, &step("h", -1, 0, 23, 23)));
        assert_eq!(v.index_of("h", 9), 23);
        assert!(!press(&mut v, "plain-counter"));
    }

    #[test]
    fn a_range_starts_then_ends() {
        let mut v = Values::default();
        let d = |n| Date::new(2025, 6, n);
        press(&mut v, &range("r", d(10)));
        press(&mut v, &range("r", d(14)));
        assert_eq!(date_range(&v, "r").span(), Some((d(10), d(14))));
        assert_eq!(unpack(pack(d(3))), Some(d(3)));
    }

    #[test]
    fn flip_turns_a_flag_from_its_default() {
        let mut v = Values::default();
        assert!(press(&mut v, &flip("open", true)));
        assert!(!v.flag(&crate::Bind::new("open", true)));
        assert!(press(&mut v, &flip("open", true)));
        assert!(v.flag(&crate::Bind::new("open", false)));
    }

    #[test]
    fn select_all_sets_every_member() {
        let mut v = Values::default();
        let keys = vec!["a".to_string(), "b".to_string()];
        assert!(toggle(&mut v, &all(&keys), true));
        assert!(v.flag(&crate::Bind::new("a", false)) && v.flag(&crate::Bind::new("b", false)));
        assert!(!toggle(&mut v, "plain", true));
    }
}
