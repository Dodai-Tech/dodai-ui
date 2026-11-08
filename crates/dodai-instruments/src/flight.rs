//! Flight-instrument geometry: attitude horizon and pitch ladder, scrolling tapes, heading
//! labels and a synthetic sonar row. Pure, so every toolkit draws the same instruments.

use dodai_charts::math::polar;

/// Wraps a heading in degrees to `0..360`. Pure.
pub fn wrap_heading(deg: f32) -> f32 {
    deg.rem_euclid(360.0)
}

/// Label of a heading mark: `N`/`E`/`S`/`W` at the cardinals, otherwise the heading in tens
/// of degrees (`03` for 030°). Pure.
pub fn heading_label(deg: f32) -> String {
    let d = wrap_heading(deg).round() as i32 % 360;
    match d {
        0 => "N".into(),
        90 => "E".into(),
        180 => "S".into(),
        270 => "W".into(),
        _ => format!("{:02}", d / 10),
    }
}

/// One mark on a scrolling tape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TapeMark {
    /// Position relative to the readout: `-0.5..=0.5` of the visible span (negative = lower).
    pub offset: f32,
    /// The value the mark stands for.
    pub value: f32,
    /// Whether the mark is a labelled major one.
    pub major: bool,
}

/// Marks of a tape showing `span` units around `value`, one every `step`, every
/// `major_every`-th being major. Pure.
pub fn tape_marks(value: f32, span: f32, step: f32, major_every: u32) -> Vec<TapeMark> {
    if span <= 0.0 || step <= 0.0 {
        return Vec::new();
    }
    let first = ((value - span / 2.0) / step).ceil() as i64;
    let last = ((value + span / 2.0) / step).floor() as i64;
    (first..=last)
        .map(|i| TapeMark {
            offset: (i as f32 * step - value) / span,
            value: i as f32 * step,
            major: major_every > 0 && i.rem_euclid(major_every as i64) == 0,
        })
        .collect()
}

/// Marks of a heading tape: like [`tape_marks`] but the values wrap at 360°. Pure.
pub fn heading_marks(heading: f32, span: f32, step: f32, major_every: u32) -> Vec<TapeMark> {
    tape_marks(heading, span, step, major_every)
        .into_iter()
        .map(|m| TapeMark {
            value: wrap_heading(m.value),
            ..m
        })
        .collect()
}

/// A pitch-ladder rung.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rung {
    /// Left end.
    pub a: (f32, f32),
    /// Right end.
    pub b: (f32, f32),
    /// Pitch angle of the rung in degrees (positive above the horizon).
    pub pitch: i32,
    /// Whether it is a long, labelled rung.
    pub major: bool,
}

/// Direction along the horizon and the unit normal pointing at the ground, for a roll in
/// degrees (positive = right wing down). Pure.
fn axes(roll_deg: f32) -> ((f32, f32), (f32, f32)) {
    let a = roll_deg.to_radians();
    ((a.cos(), -a.sin()), (a.sin(), a.cos()))
}

/// Centre of the horizon line for a pitch in degrees, `ppd` pixels per degree. Pure.
pub fn horizon_center(cx: f32, cy: f32, pitch_deg: f32, roll_deg: f32, ppd: f32) -> (f32, f32) {
    let (_, n) = axes(roll_deg);
    (cx + n.0 * pitch_deg * ppd, cy + n.1 * pitch_deg * ppd)
}

/// The two halves of the attitude disc.
#[derive(Debug, Clone, PartialEq)]
pub struct Horizon {
    /// Polygon above the horizon line.
    pub sky: Vec<(f32, f32)>,
    /// Polygon below the horizon line.
    pub ground: Vec<(f32, f32)>,
}

/// Splits a disc of radius `r` at `(cx, cy)` along the horizon line. Pure.
pub fn horizon_halves(
    cx: f32,
    cy: f32,
    r: f32,
    pitch_deg: f32,
    roll_deg: f32,
    ppd: f32,
) -> Horizon {
    let (_, n) = axes(roll_deg);
    let h = horizon_center(cx, cy, pitch_deg, roll_deg, ppd);
    let disc: Vec<_> = (0..72).map(|i| polar(cx, cy, r, i as f32 / 72.0)).collect();
    let side = |keep_ground: bool| {
        let sign = if keep_ground { 1.0 } else { -1.0 };
        let dist = |p: (f32, f32)| ((p.0 - h.0) * n.0 + (p.1 - h.1) * n.1) * sign;
        let mut out = Vec::new();
        for i in 0..disc.len() {
            let (p, q) = (disc[i], disc[(i + 1) % disc.len()]);
            let (dp, dq) = (dist(p), dist(q));
            if dp >= 0.0 {
                out.push(p);
            }
            if (dp >= 0.0) != (dq >= 0.0) {
                let t = dp / (dp - dq);
                out.push((p.0 + (q.0 - p.0) * t, p.1 + (q.1 - p.1) * t));
            }
        }
        out
    };
    Horizon {
        sky: side(false),
        ground: side(true),
    }
}

/// Pitch-ladder rungs every 5° (long ones every 10°) that fall inside radius `r * 0.8`. Pure.
pub fn ladder(cx: f32, cy: f32, r: f32, pitch_deg: f32, roll_deg: f32, ppd: f32) -> Vec<Rung> {
    let (d, n) = axes(roll_deg);
    let h = horizon_center(cx, cy, pitch_deg, roll_deg, ppd);
    (-9..=9)
        .filter(|k| *k != 0)
        .filter_map(|k| {
            let p = k * 5;
            let c = (h.0 - n.0 * p as f32 * ppd, h.1 - n.1 * p as f32 * ppd);
            if ((c.0 - cx).powi(2) + (c.1 - cy).powi(2)).sqrt() > r * 0.8 {
                return None;
            }
            let major = p % 10 == 0;
            let half = r * if major { 0.3 } else { 0.16 };
            Some(Rung {
                a: (c.0 - d.0 * half, c.1 - d.1 * half),
                b: (c.0 + d.0 * half, c.1 + d.1 * half),
                pitch: p,
                major,
            })
        })
        .collect()
}

/// Roll-scale ticks as `(turns from the top, major)`: 10°, 20°, 30°, 45° and 60° each side. Pure.
pub fn roll_ticks() -> Vec<(f32, bool)> {
    [10.0f32, 20.0, 30.0, 45.0, 60.0]
        .iter()
        .flat_map(|d| {
            let major = matches!(*d as i32, 30 | 60);
            [(d / 360.0, major), (-d / 360.0, major)]
        })
        .collect()
}

/// One row of a synthetic sonar waterfall at time `t` seconds: `bins` intensities `0..=1` of
/// background noise plus two drifting tonal lines and one steady line. Pure and deterministic.
pub fn waterfall_row(t: f32, bins: usize) -> Vec<f32> {
    let noise = |i: usize, k: f32| {
        let x = (i as f32 * 12.9898 + (t * 20.0).floor() * 78.233 + k).sin() * 43_758.547;
        x - x.floor()
    };
    let tone = |i: f32, centre: f32, width: f32| (-((i - centre) / width).powi(2)).exp();
    let n = bins.max(1) as f32;
    (0..bins)
        .map(|i| {
            let f = i as f32;
            let a = tone(f, n * (0.3 + 0.08 * (t * 0.35).sin()), n * 0.02);
            let b = tone(f, n * (0.65 - 0.06 * (t * 0.22).cos()), n * 0.02);
            let c = tone(f, n * 0.85, n * 0.012);
            (0.12 + 0.28 * noise(i, 0.0) + 0.7 * a + 0.55 * b + 0.45 * c).clamp(0.0, 1.0)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_labels_use_cardinals_and_tens() {
        assert_eq!(heading_label(0.0), "N");
        assert_eq!(heading_label(-90.0), "W");
        assert_eq!(heading_label(30.0), "03");
        assert_eq!(wrap_heading(370.0), 10.0);
    }

    #[test]
    fn tape_centres_on_the_value() {
        let m = tape_marks(103.0, 100.0, 10.0, 5);
        assert!(m.iter().all(|k| (-0.5..=0.5).contains(&k.offset)));
        let c = m.iter().find(|k| k.value == 100.0).unwrap();
        assert!(c.major && (c.offset + 0.03).abs() < 1e-5);
    }

    #[test]
    fn level_flight_splits_the_disc_in_half() {
        let Horizon { sky, ground } = horizon_halves(0.0, 0.0, 10.0, 0.0, 0.0, 2.0);
        let top = sky.iter().all(|p| p.1 <= 1e-3);
        let bottom = ground.iter().all(|p| p.1 >= -1e-3);
        assert!(top && bottom);
    }

    #[test]
    fn climbing_pushes_the_horizon_down() {
        let h = horizon_center(0.0, 0.0, 10.0, 0.0, 2.0);
        assert!((h.1 - 20.0).abs() < 1e-4);
    }

    #[test]
    fn waterfall_rows_stay_in_range() {
        let r = waterfall_row(3.0, 40);
        assert_eq!(r.len(), 40);
        assert!(r.iter().all(|v| (0.0..=1.0).contains(v)));
    }
}
