//! Hexagonal binning for dense scatter plots. Pure.

/// One occupied hexagon: its centre and how many points fell in it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hex {
    /// Centre x.
    pub x: f32,
    /// Centre y.
    pub y: f32,
    /// Points inside.
    pub count: usize,
}

/// Bins points (already in drawing units) into pointy-top hexagons of circumradius `r`.
/// Non-finite points and a non-positive `r` are ignored. Ordered by row, then column. Pure.
pub fn hexbin(points: &[(f32, f32)], r: f32) -> Vec<Hex> {
    if !(r > 0.0 && r.is_finite()) {
        return Vec::new();
    }
    let s3 = 3f32.sqrt();
    let mut cells: std::collections::BTreeMap<(i32, i32), usize> = Default::default();
    for &(x, y) in points
        .iter()
        .filter(|(x, y)| x.is_finite() && y.is_finite())
    {
        // Axial coordinates, rounded through cube coordinates.
        let q = (s3 / 3.0 * x - y / 3.0) / r;
        let rr = (2.0 / 3.0 * y) / r;
        let (cx, cz) = (q, rr);
        let cy = -cx - cz;
        let (mut rx, ry, mut rz) = (cx.round(), cy.round(), cz.round());
        let (dx, dy, dz) = ((rx - cx).abs(), (ry - cy).abs(), (rz - cz).abs());
        if dx > dy && dx > dz {
            rx = -ry - rz;
        } else if dy <= dz {
            rz = -rx - ry;
        }
        *cells.entry((rz as i32, rx as i32)).or_default() += 1;
    }
    cells
        .into_iter()
        .map(|((row, q), count)| Hex {
            x: r * s3 * (q as f32 + row as f32 / 2.0),
            y: r * 1.5 * row as f32,
            count,
        })
        .collect()
}

/// The six corners of a pointy-top hexagon of circumradius `r` around `(x, y)`. Pure.
pub fn hex_corners(x: f32, y: f32, r: f32) -> [(f32, f32); 6] {
    std::array::from_fn(|i| {
        let a = std::f32::consts::FRAC_PI_3 * i as f32 - std::f32::consts::FRAC_PI_6;
        (x + r * a.cos(), y + r * a.sin())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn points_land_in_the_hexagon_around_them() {
        let h = hexbin(
            &[(0.0, 0.0), (0.1, 0.2), (100.0, 100.0), (f32::NAN, 0.0)],
            10.0,
        );
        assert_eq!(h.len(), 2);
        assert_eq!(h[0].count, 2);
        assert_eq!((h[0].x, h[0].y), (0.0, 0.0));
        let far = h[1];
        assert!(((far.x - 100.0).powi(2) + (far.y - 100.0).powi(2)).sqrt() <= 10.0);
        assert!(hexbin(&[(1.0, 1.0)], 0.0).is_empty());
    }

    #[test]
    fn corners_sit_on_the_circumradius() {
        for (x, y) in hex_corners(3.0, 4.0, 2.0) {
            assert!(((x - 3.0).powi(2) + (y - 4.0).powi(2) - 4.0).abs() < 1e-4);
        }
    }
}
