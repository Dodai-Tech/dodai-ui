//! Renko bricks: fixed-size price blocks that ignore time.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Brick {
    /// Bottom edge.
    pub lo: f32,
    /// Top edge.
    pub hi: f32,
    pub up: bool,
}

/// Convert closes into bricks of `size`. A reversal needs two bricks of
/// movement (the usual Renko rule). Pure.
pub fn renko(closes: &[f32], size: f32) -> Vec<Brick> {
    let mut out: Vec<Brick> = Vec::new();
    if size <= 0.0 || closes.is_empty() {
        return out;
    }
    let mut level = (closes[0] / size).floor() * size;
    for &c in &closes[1..] {
        loop {
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
