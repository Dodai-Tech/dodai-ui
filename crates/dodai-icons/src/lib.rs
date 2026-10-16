//! Framework-agnostic icon geometry: polylines on a 24×24 grid.

/// Built-in icon set (24×24 design grid).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    Grid,
    Layers,
    Sliders,
    Play,
    Menu,
    Table,
    BarChart,
    Sparkle,
    Search,
    Check,
    Close,
    ChevronRight,
    ChevronDown,
    Plus,
    Dot,
}

pub(crate) fn rect(x0: f32, y0: f32, x1: f32, y1: f32) -> Vec<(f32, f32)> {
    vec![(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
}

impl IconName {
    /// Polylines `(points, closed)` on the 24×24 grid. Pure.
    pub fn shapes(self) -> Vec<(Vec<(f32, f32)>, bool)> {
        let l = |p: &[(f32, f32)]| (p.to_vec(), false);
        let c = |p: Vec<(f32, f32)>| (p, true);
        match self {
            IconName::Grid => vec![
                c(rect(3.0, 3.0, 10.0, 10.0)),
                c(rect(14.0, 3.0, 21.0, 10.0)),
                c(rect(3.0, 14.0, 10.0, 21.0)),
                c(rect(14.0, 14.0, 21.0, 21.0)),
            ],
            IconName::Layers => vec![
                c(vec![(12.0, 3.0), (21.0, 8.0), (12.0, 13.0), (3.0, 8.0)]),
                l(&[(3.0, 12.0), (12.0, 17.0), (21.0, 12.0)]),
                l(&[(3.0, 16.0), (12.0, 21.0), (21.0, 16.0)]),
            ],
            IconName::Sliders => vec![
                l(&[(4.0, 7.0), (20.0, 7.0)]),
                l(&[(4.0, 12.0), (20.0, 12.0)]),
                l(&[(4.0, 17.0), (20.0, 17.0)]),
                c(rect(7.0, 5.0, 11.0, 9.0)),
                c(rect(13.0, 10.0, 17.0, 14.0)),
                c(rect(6.0, 15.0, 10.0, 19.0)),
            ],
            IconName::Play => vec![c(vec![(7.0, 4.0), (20.0, 12.0), (7.0, 20.0)])],
            IconName::Menu => vec![
                l(&[(4.0, 6.0), (20.0, 6.0)]),
                l(&[(4.0, 12.0), (20.0, 12.0)]),
                l(&[(4.0, 18.0), (20.0, 18.0)]),
            ],
            IconName::Table => vec![
                c(rect(3.0, 5.0, 21.0, 19.0)),
                l(&[(3.0, 10.0), (21.0, 10.0)]),
                l(&[(10.0, 10.0), (10.0, 19.0)]),
            ],
            IconName::BarChart => vec![
                c(rect(4.0, 13.0, 8.0, 20.0)),
                c(rect(10.0, 8.0, 14.0, 20.0)),
                c(rect(16.0, 4.0, 20.0, 20.0)),
            ],
            IconName::Sparkle => vec![c(vec![
                (12.0, 2.0),
                (14.0, 10.0),
                (22.0, 12.0),
                (14.0, 14.0),
                (12.0, 22.0),
                (10.0, 14.0),
                (2.0, 12.0),
                (10.0, 10.0),
            ])],
            IconName::Search => vec![
                c((0..16)
                    .map(|i| {
                        let a = i as f32 / 16.0 * std::f32::consts::TAU;
                        (10.0 + 6.0 * a.cos(), 10.0 + 6.0 * a.sin())
                    })
                    .collect()),
                l(&[(14.5, 14.5), (21.0, 21.0)]),
            ],
            IconName::Check => vec![l(&[(4.0, 12.5), (9.5, 18.0), (20.0, 6.0)])],
            IconName::Close => vec![
                l(&[(5.0, 5.0), (19.0, 19.0)]),
                l(&[(19.0, 5.0), (5.0, 19.0)]),
            ],
            IconName::ChevronRight => vec![l(&[(9.0, 5.0), (16.0, 12.0), (9.0, 19.0)])],
            IconName::ChevronDown => vec![l(&[(5.0, 9.0), (12.0, 16.0), (19.0, 9.0)])],
            IconName::Plus => vec![
                l(&[(12.0, 4.0), (12.0, 20.0)]),
                l(&[(4.0, 12.0), (20.0, 12.0)]),
            ],
            IconName::Dot => vec![c(rect(10.0, 10.0, 14.0, 14.0))],
        }
    }
}

/// Every icon, for galleries and tests.
pub const ALL: [IconName; 15] = [
    IconName::Grid,
    IconName::Layers,
    IconName::Sliders,
    IconName::Play,
    IconName::Menu,
    IconName::Table,
    IconName::BarChart,
    IconName::Sparkle,
    IconName::Search,
    IconName::Check,
    IconName::Close,
    IconName::ChevronRight,
    IconName::ChevronDown,
    IconName::Plus,
    IconName::Dot,
];

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shapes_inside_grid() {
        for n in ALL {
            let s = n.shapes();
            assert!(!s.is_empty());
            for (p, _) in s {
                assert!(p
                    .iter()
                    .all(|&(x, y)| (0.0..=24.0).contains(&x) && (0.0..=24.0).contains(&y)));
            }
        }
    }
}
