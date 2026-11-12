//! The actions a page sends to the server: one `|`-separated line, kind first.
//!
//! | line | meaning |
//! |---|---|
//! | `P|key` | a counting / command button was pressed |
//! | `I|key|n` | index `n` was picked |
//! | `B|key|true` | a flag changed |
//! | `Z|key|n` | an integer changed |
//! | `T|key|text` | a text field changed (the text is last, so it may hold `|`) |
//! | `N|key|v|min|max|step` | a slider value, snapped to the step (`-` = none) |
//! | `L|lo|hi|end|v|min|max|step` | a range slider thumb (`end` 0 = low, 1 = high) |
//! | `KN|key|min|max|v0|dx|dy` | a knob dragged by `dx`, `dy` px from `v0` |
//! | `GO|yaw|pitch|y0|p0|dx|dy` | the gizmo dragged (orbit) |
//! | `GC|yaw|pitch|size|y0|p0|x|y` | the gizmo clicked at `x`, `y` |
//! | `C|n` | page `n` (empty = All) |
//! | `S|title` | a section was picked in the sidebar |
//! | `V|top|title` | the page scrolled to this section (`top` = 1 at the very top); only the sidebar changes |
//! | `F|text` | the sidebar filter |
//! | `D|n`, `M|n`, `K|n` | design, theme of the design, colour |
//! | `A|b`, `E|n`, `R|b`, `Y|b` | animate, speed, right-to-left, patterns |

use dodai_showcase_model::{catalog, Action, Bind, Showcase, Value, CATEGORIES};

/// A command button.
pub fn press(key: &str) -> String {
    format!("P|{key}")
}

/// A picked index (`n` may be the placeholder `{v}`).
pub fn index(key: &str, n: impl std::fmt::Display) -> String {
    format!("I|{key}|{n}")
}

/// A flag (`on` may be the placeholder `{v}`).
pub fn flag(key: &str, on: impl std::fmt::Display) -> String {
    format!("B|{key}|{on}")
}

/// An integer.
pub fn int(key: &str, n: i32) -> String {
    format!("Z|{key}|{n}")
}

/// A text value.
pub fn text(key: &str, v: &str) -> String {
    format!("T|{key}|{v}")
}

fn step_text(step: Option<f32>) -> String {
    step.map_or("-".to_string(), |s| s.to_string())
}

/// A slider value snapped to `step`.
pub fn snapped(key: &str, v: &str, min: f32, max: f32, step: Option<f32>) -> String {
    format!("N|{key}|{v}|{min}|{max}|{}", step_text(step))
}

/// One thumb of a range slider.
pub fn range(
    lo: &str,
    hi: &str,
    high: bool,
    v: &str,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> String {
    format!(
        "L|{lo}|{hi}|{}|{v}|{min}|{max}|{}",
        u8::from(high),
        step_text(step)
    )
}

fn num(s: &str) -> Option<f32> {
    s.trim().parse().ok()
}

fn snap(v: f32, min: f32, max: f32, step: Option<f32>) -> f32 {
    let v = v.clamp(min, max);
    match step {
        Some(s) if s > 0.0 => (min + ((v - min) / s).round() * s).clamp(min, max),
        _ => v,
    }
}

fn truth(s: &str) -> bool {
    matches!(s, "1" | "true")
}

fn section(title: &str) -> Option<&'static str> {
    catalog::sections()
        .iter()
        .find(|s| s.title == title)
        .map(|s| s.title)
}

/// Apply one line to the model. `false` when the line means nothing.
pub fn apply(m: &mut Showcase, line: &str) -> bool {
    let mut p = line.split('|');
    let Some(kind) = p.next() else {
        return false;
    };
    let rest: Vec<&str> = p.collect();
    let set = |m: &mut Showcase, k: &str, v: Value| m.update(Action::Set(k.to_string(), v));
    match (kind, rest.as_slice()) {
        ("P", [..]) => m.update(Action::Press(rest.join("|"))),
        ("I", [k, n]) => match n.parse() {
            Ok(n) => set(m, k, Value::Index(n)),
            Err(_) => return false,
        },
        ("B", [k, b]) => set(m, k, Value::Bool(truth(b))),
        ("Z", [k, n]) => match n.parse() {
            Ok(n) => set(m, k, Value::Int(n)),
            Err(_) => return false,
        },
        ("T", [..]) => {
            let mut it = line.splitn(3, '|').skip(1);
            let (Some(k), v) = (it.next(), it.next().unwrap_or("")) else {
                return false;
            };
            set(m, k, Value::Text(v.to_string()));
        }
        ("N", [k, v, min, max, step]) => {
            let (Some(v), Some(min), Some(max)) = (num(v), num(min), num(max)) else {
                return false;
            };
            set(m, k, Value::Num(snap(v, min, max, num(step))));
        }
        ("L", [lo, hi, end, v, min, max, step]) => {
            let (Some(v), Some(min), Some(max)) = (num(v), num(min), num(max)) else {
                return false;
            };
            let v = snap(v, min, max, num(step));
            let now = |k: &str, d: f32| m.values.num(&Bind::new(k, d));
            let (a, b) = (now(lo, min), now(hi, max));
            if *end == "1" {
                set(m, hi, Value::Num(v.max(a)));
            } else {
                set(m, lo, Value::Num(v.min(b)));
            }
        }
        ("KN", [k, min, max, v0, dx, dy]) => {
            let (Some(min), Some(max), Some(v0), Some(dx), Some(dy)) =
                (num(min), num(max), num(v0), num(dx), num(dy))
            else {
                return false;
            };
            set(
                m,
                k,
                Value::Num(dodai_core::knob_value(v0, dx, dy, min, max, None)),
            );
        }
        ("GO", [yk, pk, y0, p0, dx, dy]) => {
            let (Some(y0), Some(p0), Some(dx), Some(dy)) = (num(y0), num(p0), num(dx), num(dy))
            else {
                return false;
            };
            let (y, p) = dodai_core::orientation::orbit(y0, p0, dx, dy);
            set(m, yk, Value::Num(y));
            set(m, pk, Value::Num(p));
        }
        ("GC", [yk, pk, size, y0, p0, x, y]) => {
            let (Some(size), Some(y0), Some(p0), Some(x), Some(y)) =
                (num(size), num(y0), num(p0), num(x), num(y))
            else {
                return false;
            };
            if let Some((ny, np)) = gizmo_click(size, y0, p0, [x, y]) {
                set(m, yk, Value::Num(ny));
                set(m, pk, Value::Num(np));
            }
        }
        ("C", [n]) => m.update(Action::Category(
            n.parse().ok().filter(|c| *c < CATEGORIES.len()),
        )),
        ("C", []) => m.update(Action::Category(None)),
        ("S", [..]) => match section(&rest.join("|")) {
            Some(t) => m.update(Action::Section(t)),
            None => return false,
        },
        ("V", [top, ..]) => match section(&rest[1..].join("|")) {
            Some(t) => {
                m.set_at_top(*top == "1");
                m.update(Action::Visible(t));
            }
            None => return false,
        },
        ("F", _) => m.update(Action::Filter(rest.join("|"))),
        ("D", [n]) => m.update(Action::Design(n.parse().unwrap_or(0))),
        ("M", [n]) => m.update(Action::Variant(n.parse().unwrap_or(0))),
        ("K", [n]) => match n
            .parse::<usize>()
            .ok()
            .and_then(|i| m.colours().get(i).copied())
        {
            Some(c) => m.update(Action::Colour(c)),
            None => return false,
        },
        ("CT", [b]) => m.update(Action::CycleTheme(truth(b))),
        ("A", [b]) => m.update(Action::Animate(truth(b))),
        ("E", [n]) => m.update(Action::Speed(n.parse().unwrap_or(1))),
        ("R", [b]) => m.update(Action::Rtl(truth(b))),
        ("Y", [b]) => m.update(Action::Patterns(truth(b))),
        _ => return false,
    }
    true
}

/// Where a click on the gizmo goes (a ball looks along its axis, home goes home).
pub fn gizmo_click(size: f32, yaw: f32, pitch: f32, at: [f32; 2]) -> Option<(f32, f32)> {
    use dodai_core::orientation::{hit, place, AxisId, Basis, GizmoLayout, Hit};
    let layout = GizmoLayout::new(size);
    let placed = place(&Basis::from_orbit(yaw, pitch), &layout, &AxisId::DEFAULT);
    match hit(&layout, &placed, at) {
        Hit::Axis(a) => {
            let (y, p) = a.view_angles();
            Some((y.unwrap_or(yaw), p))
        }
        Hit::Home => Some((0.6, 0.45)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn num_of(m: &Showcase, k: &str) -> f32 {
        m.values.num(&Bind::new(k, f32::NAN))
    }

    #[test]
    fn values_land_under_their_key() {
        let mut m = Showcase::default();
        assert!(apply(&mut m, &index("k", 3)));
        assert_eq!(m.values.index_of("k", 0), 3);
        assert!(apply(&mut m, &flag("f", true)));
        assert!(m.values.flag(&Bind::new("f", false)));
        assert!(apply(&mut m, &int("z", 7)));
        assert_eq!(m.values.int(&Bind::new("z", 0)), 7);
    }

    #[test]
    fn text_keeps_a_bar_inside() {
        let mut m = Showcase::default();
        assert!(apply(&mut m, &text("t", "a|b")));
        assert_eq!(m.values.text(&Bind::new("t", String::new())), "a|b");
    }

    #[test]
    fn a_slider_value_snaps_to_its_step_and_range() {
        let mut m = Showcase::default();
        assert!(apply(&mut m, &snapped("s", "7", 0.0, 10.0, Some(5.0))));
        assert_eq!(num_of(&m, "s"), 5.0);
        assert!(apply(&mut m, &snapped("s", "99", 0.0, 10.0, None)));
        assert_eq!(num_of(&m, "s"), 10.0);
    }

    #[test]
    fn range_thumbs_do_not_cross() {
        let mut m = Showcase::default();
        assert!(apply(&mut m, &range("a", "b", true, "8", 0.0, 10.0, None)));
        assert!(apply(&mut m, &range("a", "b", false, "9", 0.0, 10.0, None)));
        assert_eq!((num_of(&m, "a"), num_of(&m, "b")), (8.0, 8.0));
    }

    #[test]
    fn chrome_actions_change_the_page() {
        let mut m = Showcase::default();
        assert!(apply(&mut m, "C|1"));
        assert_eq!(m.category(), Some(1));
        let t = m.visible()[1].title;
        assert!(apply(&mut m, &format!("S|{t}")));
        assert_eq!(m.active_section(), Some(t));
        assert!(!apply(&mut m, "S|no such section"));
        assert!(!apply(&mut m, "nonsense"));
    }
}
