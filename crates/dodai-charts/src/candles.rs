//! OHLC data and Heikin-Ashi smoothing.

/// One price bar.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candle {
    pub open: f32,
    pub high: f32,
    pub low: f32,
    pub close: f32,
    pub volume: f32,
}

impl Candle {
    pub const fn new(open: f32, high: f32, low: f32, close: f32) -> Self {
        Self {
            open,
            high,
            low,
            close,
            volume: 0.0,
        }
    }
    pub const fn volume(mut self, v: f32) -> Self {
        self.volume = v;
        self
    }
    /// Close at or above open.
    pub fn bullish(&self) -> bool {
        self.close >= self.open
    }
    /// Body `(low, high)`.
    pub fn body(&self) -> (f32, f32) {
        (self.open.min(self.close), self.open.max(self.close))
    }
}

/// Heikin-Ashi transform: smoother candles, same length. Pure.
pub fn heikin_ashi(c: &[Candle]) -> Vec<Candle> {
    let mut out: Vec<Candle> = Vec::with_capacity(c.len());
    for k in c {
        let close = (k.open + k.high + k.low + k.close) / 4.0;
        let open = match out.last() {
            Some(p) => (p.open + p.close) / 2.0,
            None => (k.open + k.close) / 2.0,
        };
        out.push(Candle {
            open,
            high: k.high.max(open).max(close),
            low: k.low.min(open).min(close),
            close,
            volume: k.volume,
        });
    }
    out
}

/// `(min low, max high)` across candles, `None` when empty.
pub fn price_range(c: &[Candle]) -> Option<(f32, f32)> {
    c.iter().fold(None, |a, k| match a {
        None => Some((k.low, k.high)),
        Some((l, h)) => Some((l.min(k.low), h.max(k.high))),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heikin_ashi_first_and_chain() {
        let c = [
            Candle::new(10.0, 14.0, 9.0, 12.0),
            Candle::new(12.0, 16.0, 11.0, 15.0),
        ];
        let h = heikin_ashi(&c);
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].close, 11.25);
        assert_eq!(h[0].open, 11.0);
        assert_eq!(h[1].open, (h[0].open + h[0].close) / 2.0);
        assert!(h[1].high >= h[1].close && h[1].low <= h[1].open);
    }

    #[test]
    fn range_and_body() {
        let c = [
            Candle::new(1.0, 5.0, 0.5, 3.0),
            Candle::new(3.0, 4.0, 0.2, 2.0),
        ];
        assert_eq!(price_range(&c), Some((0.2, 5.0)));
        assert_eq!(price_range(&[]), None);
        assert!(c[0].bullish() && !c[1].bullish());
        assert_eq!(c[1].body(), (2.0, 3.0));
    }
}
