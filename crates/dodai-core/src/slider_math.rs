//! Slider maths shared by every toolkit: fractions, pointer and key values, ticks, two-thumb
//! ranges. Pure.

/// Clamp to `0.0..=1.0` (NaN → 0).
pub fn clamp01(v: f32) -> f32 {
    if v.is_nan() {
        0.0
    } else {
        v.clamp(0.0, 1.0)
    }
}

/// Position of a value in `min..=max` as a 0..1 fraction.
pub fn slider_fraction(value: f32, min: f32, max: f32) -> f32 {
    if max <= min {
        0.0
    } else {
        clamp01((value - min) / (max - min))
    }
}

/// Tick positions (`0..=1`) for `min..=max` at multiples of `step`;
/// empty when there would be more than 64 (too dense to draw). Pure.
pub fn tick_fractions(min: f32, max: f32, step: Option<f32>) -> Vec<f32> {
    let Some(step) = step.filter(|s| *s > 0.0 && max > min) else {
        return Vec::new();
    };
    let n = ((max - min) / step).round() as usize;
    if n == 0 || n > 64 {
        return Vec::new();
    }
    (0..=n)
        .map(|i| ((i as f32 * step) / (max - min)).clamp(0.0, 1.0))
        .collect()
}

/// Map a pointer x to a slider value. `left`/`width` are the control's bounds,
/// `thumb` the thumb diameter (the thumb centre travels `width - thumb`).
pub fn slider_value(
    x: f32,
    left: f32,
    width: f32,
    thumb: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
) -> f32 {
    let span = (width - thumb).max(1.0);
    let f = clamp01((x - left - thumb / 2.0) / span);
    let mut v = min + f * (max - min);
    if let Some(s) = step.filter(|s| *s > 0.0) {
        v = min + ((v - min) / s).round() * s;
    }
    v.clamp(min.min(max), max.max(min))
}

/// The value a key moves a slider to: arrows by one `step` (a twentieth of the range without
/// one), Page Up/Down by ten, Home and End to the ends. `None` for any other key or when the
/// value would not change. Pure.
pub fn key_value(key: &str, v: f32, min: f32, max: f32, step: Option<f32>) -> Option<f32> {
    let (lo, hi) = (min.min(max), max.max(min));
    let one = step.filter(|s| *s > 0.0).unwrap_or((hi - lo) / 20.0);
    let to = match key {
        "right" | "up" => v + one,
        "left" | "down" => v - one,
        "pageup" => v + one * 10.0,
        "pagedown" => v - one * 10.0,
        "home" => lo,
        "end" => hi,
        _ => return None,
    };
    let to = to.clamp(lo, hi);
    ((to - v).abs() > f32::EPSILON * hi.abs().max(1.0)).then_some(to)
}

/// Which thumb (`0` = low, `1` = high) is closest to `v`; ties go to the
/// side that can still move toward `v`.
pub fn nearest_thumb(lo: f32, hi: f32, v: f32) -> usize {
    let (dl, dh) = ((v - lo).abs(), (v - hi).abs());
    if dl < dh || (dl == dh && v < lo) {
        0
    } else {
        1
    }
}

/// Move `thumb` to `v`, keeping `lo <= hi`.
pub fn range_update(thumb: usize, v: f32, lo: f32, hi: f32) -> (f32, f32) {
    if thumb == 0 {
        (v.min(hi), hi)
    } else {
        (lo, v.max(lo))
    }
}
