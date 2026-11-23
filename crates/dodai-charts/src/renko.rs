//! Renko bricks: fixed-size price blocks that ignore time.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brick {
    /// Bottom edge.
    pub lo: f32,
    /// Top edge.
    pub hi: f32,
    pub up: bool,
}

/// Most bricks one call returns.
pub const MAX_BRICKS: usize = 10_000;

/// Convert closes into bricks of `size`. A reversal needs two bricks of
/// movement (the usual Renko rule). Pure.
pub fn renko(closes: &[f32], size: f32) -> Vec<Brick> {
    let mut out: Vec<Brick> = Vec::new();
    let mut closes = closes.iter().copied().filter(|c| c.is_finite());
    let (true, Some(first)) = (size.is_finite() && size > 0.0, closes.next()) else {
        return out;
    };
    let mut level = (first / size).floor() * size;
    if !level.is_finite() {
        return out;
    }
    for c in closes {
        // A tiny brick against a big move would never finish; stop at a sane count.
        while out.len() < MAX_BRICKS {
            let last_up = out.last().map(|b| b.up);
            if c >= level + size * if last_up == Some(false) { 2.0 } else { 1.0 } {
                let lo = if last_up == Some(false) {
                    level + size
                } else {
                    level
                };
                out.push(Brick {
                    lo,
                    hi: lo + size,
                    up: true,
                });
                level = lo + size;
            } else if c <= level - size * if last_up == Some(true) { 2.0 } else { 1.0 } {
                let hi = if last_up == Some(true) {
                    level - size
                } else {
                    level
                };
                out.push(Brick {
                    lo: hi - size,
                    hi,
                    up: false,
                });
                level = hi - size;
            } else {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_input_is_bounded() {
        assert!(renko(&[0.0, 1e30], 1e-30).len() <= MAX_BRICKS);
        assert!(renko(&[1.0, 2.0], f32::NAN).is_empty());
        assert!(renko(&[f32::NAN, f32::INFINITY], 1.0).is_empty());
    }

    #[test]
    fn rises_make_up_bricks() {
        let b = renko(&[10.0, 13.5], 1.0);
        assert_eq!(b.len(), 3);
        assert!(b.iter().all(|k| k.up));
        assert_eq!(b[0].lo, 10.0);
        assert_eq!(b[2].hi, 13.0);
    }

    #[test]
    fn reversal_needs_two_bricks() {
        let b = renko(&[10.0, 12.0, 11.0], 1.0);
        assert_eq!(b.len(), 2); // 11.0 is one brick below top: no reversal
        let b = renko(&[10.0, 12.0, 10.0], 1.0);
        assert!(!b.last().unwrap().up);
    }

    #[test]
    fn degenerate_inputs() {
        assert!(renko(&[], 1.0).is_empty());
        assert!(renko(&[1.0, 5.0], 0.0).is_empty());
    }
}
