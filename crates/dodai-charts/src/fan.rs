//! Span fan layout: one spoke per item around an arc, its length the item's lifespan on a
//! shared scale, with an optional peak stretch and a magnitude.
//!
//! Lengths share one linear scale (the longest span reaches the rim), so spokes compare
//! honestly; magnitudes map to *area* (radius ∝ √magnitude) for the end markers.

/// One item: a lifespan `start..end`, an optional `peak` inside it and a `magnitude`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    /// Start of the lifespan.
    pub start: f32,
    /// End of the lifespan (spans with `end < start` are swapped).
    pub end: f32,
    /// The peak stretch; clipped to the lifespan.
    pub peak: Option<(f32, f32)>,
    /// A non-negative size (area, count, ...) for the end marker.
    pub magnitude: f32,
}

/// The order spokes go round the arc in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FanOrder {
    /// As given.
    #[default]
    Given,
    /// By start, earliest first.
    Start,
    /// By length, longest first.
    Length,
    /// By magnitude, largest first.
    Magnitude,
}

/// A laid-out spoke. Angles are turns (`0` = up, clockwise, as `math::polar`); lengths are
/// fractions `0..=1` of the ring between hub and rim.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spoke {
    /// Index of the item in the input.
    pub item: usize,
    /// Angle of the spoke, in turns.
    pub turn: f32,
    /// Length, `0..=1`.
    pub len: f32,
    /// The peak stretch along the spoke, `0..=1` from the hub, if any.
    pub peak: Option<(f32, f32)>,
    /// End marker size, `0..=1` (√ of the magnitude share, so the *area* compares).
    pub mark: f32,
}

fn clean(v: f32) -> f32 {
    if v.is_finite() {
        v
    } else {
        0.0
    }
}

/// Lays `items` out on an arc of `sweep` turns (`0..=1`) centred on the top, in `order`.
/// Pure; non-finite numbers count as 0.
pub fn layout(items: &[Span], sweep: f32, order: FanOrder) -> Vec<Spoke> {
    let sweep = if sweep.is_finite() {
        sweep.clamp(0.05, 1.0)
    } else {
        0.75
    };
    let spans: Vec<(f32, f32)> = items
        .iter()
        .map(|s| {
            let (a, b) = (clean(s.start), clean(s.end));
            (a.min(b), a.max(b))
        })
        .collect();
    let mags: Vec<f32> = items.iter().map(|s| clean(s.magnitude).max(0.0)).collect();
    let longest = spans.iter().map(|(a, b)| b - a).fold(0.0f32, f32::max);
    let biggest = mags.iter().copied().fold(0.0f32, f32::max);
    let mut idx: Vec<usize> = (0..items.len()).collect();
    let key = |i: &usize| match order {
        FanOrder::Given => 0.0,
        FanOrder::Start => spans[*i].0,
        FanOrder::Length => -(spans[*i].1 - spans[*i].0),
        FanOrder::Magnitude => -mags[*i],
    };
    idx.sort_by(|a, b| key(a).total_cmp(&key(b)));
    // A full circle leaves one step free so the first and last spokes do not meet.
    let n = items.len();
    let steps = if sweep >= 1.0 {
        n as f32
    } else {
        n.saturating_sub(1).max(1) as f32
    };
    let first = -sweep / 2.0;
    let frac = |v: f32| if longest > 0.0 { v / longest } else { 0.0 };
    idx.into_iter()
        .enumerate()
        .map(|(k, i)| {
            let (a, b) = spans[i];
            let turn = if n == 1 {
                0.0
            } else {
                first + sweep * k as f32 / steps
            };
            let peak = items[i].peak.and_then(|(p, q)| {
                let (p, q) = (clean(p).clamp(a, b), clean(q).clamp(a, b));
                let (p, q) = (p.min(q), p.max(q));
                (q > p).then(|| (frac(p - a), frac(q - a)))
            });
            Spoke {
                item: i,
                turn: turn.rem_euclid(1.0),
                len: frac(b - a),
                peak,
                mark: if biggest > 0.0 {
                    (mags[i] / biggest).sqrt()
                } else {
                    0.0
                },
            }
        })
        .collect()
}

/// The spoke nearest the pointer angle `turn`, if within half a step of one. `spokes` as
/// laid out by [`layout`] with `sweep`.
pub fn hit(spokes: &[Spoke], sweep: f32, turn: f32) -> Option<usize> {
    if spokes.is_empty() || !turn.is_finite() {
        return None;
    }
    let half = (sweep.clamp(0.05, 1.0) / spokes.len().max(2) as f32 / 2.0).max(0.002);
    let dist = |s: &Spoke| {
        let d = (s.turn - turn).rem_euclid(1.0);
        d.min(1.0 - d)
    };
    spokes
        .iter()
        .enumerate()
        .min_by(|a, b| dist(a.1).total_cmp(&dist(b.1)))
        .filter(|(_, s)| dist(s) <= half * 1.5)
        .map(|(k, _)| k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: f32, end: f32, mag: f32) -> Span {
        Span {
            start,
            end,
            peak: None,
            magnitude: mag,
        }
    }

    #[test]
    fn the_longest_span_reaches_the_rim_and_lengths_share_one_scale() {
        let s = layout(
            &[span(0.0, 100.0, 1.0), span(50.0, 100.0, 4.0)],
            0.5,
            FanOrder::Given,
        );
        assert_eq!(s[0].len, 1.0);
        assert_eq!(s[1].len, 0.5);
        assert_eq!(
            s[0].mark, 0.5,
            "area scale: a quarter the size, half the radius"
        );
        assert_eq!(s[1].mark, 1.0);
    }

    #[test]
    fn spokes_fill_the_sweep_centred_on_top() {
        let s = layout(&[span(0.0, 1.0, 1.0); 3], 0.5, FanOrder::Given);
        let turns: Vec<f32> = s.iter().map(|s| s.turn).collect();
        assert_eq!(turns, [0.75, 0.0, 0.25]);
        let one = layout(&[span(0.0, 1.0, 1.0)], 0.5, FanOrder::Given);
        assert_eq!(one[0].turn, 0.0);
    }

    #[test]
    fn order_sorts_and_keeps_item_indices() {
        let items = [span(10.0, 20.0, 5.0), span(0.0, 50.0, 1.0)];
        let by_start: Vec<usize> = layout(&items, 0.5, FanOrder::Start)
            .iter()
            .map(|s| s.item)
            .collect();
        assert_eq!(by_start, [1, 0]);
        let by_mag: Vec<usize> = layout(&items, 0.5, FanOrder::Magnitude)
            .iter()
            .map(|s| s.item)
            .collect();
        assert_eq!(by_mag, [0, 1]);
    }

    #[test]
    fn peaks_are_clipped_to_the_span() {
        let items = [Span {
            start: 0.0,
            end: 100.0,
            peak: Some((80.0, 150.0)),
            magnitude: 1.0,
        }];
        assert_eq!(
            layout(&items, 0.5, FanOrder::Given)[0].peak,
            Some((0.8, 1.0))
        );
        let outside = [Span {
            peak: Some((200.0, 300.0)),
            ..items[0]
        }];
        assert_eq!(layout(&outside, 0.5, FanOrder::Given)[0].peak, None);
    }

    #[test]
    fn degenerate_input_stays_finite() {
        let items = [
            span(f32::NAN, f32::INFINITY, f32::NAN),
            span(5.0, 5.0, -3.0),
            span(9.0, 1.0, 0.0),
        ];
        for sweep in [f32::NAN, 0.0, 1.0, 7.0] {
            for s in layout(&items, sweep, FanOrder::Length) {
                assert!(s.turn.is_finite() && (0.0..=1.0).contains(&s.len));
                assert!((0.0..=1.0).contains(&s.mark));
            }
        }
        assert!(layout(&[], 0.5, FanOrder::Start).is_empty());
    }

    #[test]
    fn hit_finds_the_nearest_spoke_only_close_by() {
        let s = layout(&[span(0.0, 1.0, 1.0); 5], 0.5, FanOrder::Given);
        assert_eq!(hit(&s, 0.5, 0.01), Some(2));
        assert_eq!(hit(&s, 0.5, 0.99), Some(2));
        assert_eq!(hit(&s, 0.5, 0.5), None);
        assert_eq!(hit(&s, 0.5, f32::NAN), None);
        assert_eq!(hit(&[], 0.5, 0.0), None);
    }
}
