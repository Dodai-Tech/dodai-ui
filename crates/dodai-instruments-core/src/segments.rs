//! Seven-segment display geometry: which segments a character lights and the polygon of each
//! segment. Pure, so every toolkit draws the same digits.

/// Segment bits: a top, b top-right, c bottom-right, d bottom, e bottom-left, f top-left, g middle.
const MASKS: [u8; 10] = [
    0b0111111, 0b0000110, 0b1011011, 0b1001111, 0b1100110, 0b1101101, 0b1111101, 0b0000111,
    0b1111111, 0b1101111,
];

/// Lit-segment bit mask of a character (digits, `-` and `°` as a raised ring; everything else is
/// blank). Pure.
pub fn mask(c: char) -> u8 {
    match c {
        '0'..='9' => MASKS[c as usize - '0' as usize],
        '-' => 0b1000000,
        '°' => 0b1100011,
        _ => 0,
    }
}

/// Horizontal advance of a character at digit height `h`: digits are `0.7 h`, dots and colons
/// `0.22 h`. Pure.
pub fn advance(c: char, h: f32) -> f32 {
    match c {
        ':' | '.' => h * 0.22,
        _ => h * 0.7,
    }
}

/// Width of the digit body (without the gap to the next digit) at height `h`. Pure.
pub fn digit_width(h: f32) -> f32 {
    h * 0.56
}

/// Segment thickness at height `h`. Pure.
pub fn thickness(h: f32) -> f32 {
    h * 0.12
}

/// Shear applied to a point so digits lean forward. Pure.
pub fn slant(p: (f32, f32), x0: f32, h: f32) -> (f32, f32) {
    (x0 + p.0 + (h - p.1) * 0.1, p.1)
}

fn hbar(x0: f32, x1: f32, y: f32, th: f32) -> Vec<(f32, f32)> {
    let h = th / 2.0;
    vec![
        (x0, y),
        (x0 + h, y - h),
        (x1 - h, y - h),
        (x1, y),
        (x1 - h, y + h),
        (x0 + h, y + h),
    ]
}

fn vbar(x: f32, y0: f32, y1: f32, th: f32) -> Vec<(f32, f32)> {
    let h = th / 2.0;
    vec![
        (x, y0),
        (x + h, y0 + h),
        (x + h, y1 - h),
        (x, y1),
        (x - h, y1 - h),
        (x - h, y0 + h),
    ]
}

/// The seven segment polygons (a..g) of one digit of height `h`, unslanted. Pure.
pub fn digit_segments(h: f32) -> [Vec<(f32, f32)>; 7] {
    let (dw, th) = (digit_width(h), thickness(h));
    let (xl, xr) = (th / 2.0, dw - th / 2.0);
    let (yt, ym, yb) = (th / 2.0, h / 2.0, h - th / 2.0);
    let p = th * 0.55;
    [
        hbar(xl + p, xr - p, yt, th),
        vbar(xr, yt + p, ym - p, th),
        vbar(xr, ym + p, yb - p, th),
        hbar(xl + p, xr - p, yb, th),
        vbar(xl, ym + p, yb - p, th),
        vbar(xl, yt + p, ym - p, th),
        hbar(xl + p, xr - p, ym, th),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_degree_sign_is_the_upper_ring() {
        // a (top), b (top right), f (top left) and g (middle): the ring in the upper half.
        let lit: Vec<u8> = (0..7).filter(|i| mask('°') >> i & 1 == 1).collect();
        assert_eq!(lit, [0, 1, 5, 6]);
    }

    #[test]
    fn eight_lights_everything_and_one_two_segments() {
        assert_eq!(mask('8'), 0b1111111);
        assert_eq!(mask('1').count_ones(), 2);
        assert_eq!(mask('x'), 0);
    }

    #[test]
    fn segments_stay_inside_the_digit_box() {
        for seg in digit_segments(40.0) {
            for (x, y) in seg {
                assert!((0.0..=digit_width(40.0)).contains(&x) && (0.0..=40.0).contains(&y));
            }
        }
    }
}
