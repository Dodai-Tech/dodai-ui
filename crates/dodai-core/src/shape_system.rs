//! **Shape system**: renderer-agnostic shape primitives, size scale and
//! per-role mapping, so each design system can define its own geometry
//! (Cyberpunk chamfers, Material 3 rounded scale, …).
//!
//! * [`Corner`] – one corner: `Square`, `Round`, `Cut` (chamfer) or `Scoop`.
//! * [`CornerSpec`] – four independent corners; produces an outline polygon,
//!   can be interpolated ([`CornerSpec::lerp`]) for morphing.
//! * [`ShapeSystem`] – a design system's scale + role table; components ask
//!   for a [`ShapeRole`] and get a [`CornerSpec`].
//!
//! ```
//! use dodai_core::{ShapeRole, ShapeSystem};
//! let m3 = ShapeSystem::material3();
//! assert!(m3.spec(ShapeRole::Button).has_curves());
//! let pts = m3.spec(ShapeRole::Card).outline(200.0, 100.0);
//! assert!(pts.len() > 4);
//! ```

use crate::Corners;

/// Segments used to flatten a quarter arc.
const ARC_STEPS: usize = 8;

/// Size used for "fully rounded" (clamped to half the shorter side).
pub const FULL: f32 = 9999.0;

/// One corner of a shape.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Corner {
    /// Hard 90° corner.
    Square,
    /// Convex arc of radius `r`.
    Round(f32),
    /// Straight chamfer cut of size `c`.
    Cut(f32),
    /// Concave arc of radius `r` (a "scooped" corner).
    Scoop(f32),
}

impl Corner {
    /// Size in px (0 for `Square`).
    pub fn size(self) -> f32 {
        match self {
            Corner::Square => 0.0,
            Corner::Round(v) | Corner::Cut(v) | Corner::Scoop(v) => v,
        }
    }

    fn same_kind(self, o: Corner) -> bool {
        std::mem::discriminant(&self) == std::mem::discriminant(&o)
    }

    fn with_size(self, s: f32) -> Corner {
        match self {
            Corner::Square => Corner::Square,
            Corner::Round(_) => Corner::Round(s),
            Corner::Cut(_) => Corner::Cut(s),
            Corner::Scoop(_) => Corner::Scoop(s),
        }
    }

    /// Blend two corners. Same kind: size interpolates. Different kinds shrink
    /// through `Square` (first half) and grow into the target (second half).
    pub fn lerp(a: Corner, b: Corner, t: f32) -> Corner {
        let t = t.clamp(0.0, 1.0);
        let mix = |x: f32, y: f32, t: f32| x + (y - x) * t;
        if a.same_kind(b) {
            return a.with_size(mix(a.size(), b.size(), t));
        }
        match (a, b) {
            (Corner::Square, _) => b.with_size(mix(0.0, b.size(), t)),
            (_, Corner::Square) => a.with_size(mix(a.size(), 0.0, t)),
            _ if t < 0.5 => a.with_size(mix(a.size(), 0.0, t * 2.0)),
            _ => b.with_size(mix(0.0, b.size(), (t - 0.5) * 2.0)),
        }
    }
}

/// Four independent corners, clockwise from top-left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerSpec {
    pub top_left: Corner,
    pub top_right: Corner,
    pub bottom_right: Corner,
    pub bottom_left: Corner,
}

impl CornerSpec {
    pub const SQUARE: Self = Self::uniform(Corner::Square);

    pub const fn uniform(c: Corner) -> Self {
        Self {
            top_left: c,
            top_right: c,
            bottom_right: c,
            bottom_left: c,
        }
    }

    /// `c` on the corners flagged in `sides`, `Square` elsewhere.
    pub fn on(sides: Corners, c: Corner) -> Self {
        let p = |on: bool| if on { c } else { Corner::Square };
        Self {
            top_left: p(sides.top_left),
            top_right: p(sides.top_right),
            bottom_right: p(sides.bottom_right),
            bottom_left: p(sides.bottom_left),
        }
    }

    /// Which corners are not square.
    pub fn sides(&self) -> Corners {
        let on = |c: Corner| c != Corner::Square && c.size() > 0.0;
        Corners {
            top_left: on(self.top_left),
            top_right: on(self.top_right),
            bottom_right: on(self.bottom_right),
            bottom_left: on(self.bottom_left),
        }
    }

    pub fn is_square(&self) -> bool {
        !self.sides().any()
    }

    /// Any arc (round / scoop) that needs more than straight segments.
    pub fn has_curves(&self) -> bool {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ]
        .iter()
        .any(|c| matches!(c, Corner::Round(_) | Corner::Scoop(_)) && c.size() > 0.0)
    }

    /// `Some(r)` when all four corners are the same round radius (maps
    /// directly to CSS `border-radius`, egui `Rounding`, iced radius…).
    pub fn uniform_radius(&self) -> Option<f32> {
        match (
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ) {
            (Corner::Round(a), Corner::Round(b), Corner::Round(c), Corner::Round(d))
                if a == b && b == c && c == d =>
            {
                Some(a)
            }
            _ => None,
        }
    }

    /// Per-corner interpolation (for morphing / animated preset switches).
    pub fn lerp(a: &Self, b: &Self, t: f32) -> Self {
        Self {
            top_left: Corner::lerp(a.top_left, b.top_left, t),
            top_right: Corner::lerp(a.top_right, b.top_right, t),
            bottom_right: Corner::lerp(a.bottom_right, b.bottom_right, t),
            bottom_left: Corner::lerp(a.bottom_left, b.bottom_left, t),
        }
    }

    /// Outline polygon, clockwise from top-left, for a `w`×`h` box. Arcs are
    /// flattened. Sizes are clamped: round / scoop to half the shorter side,
    /// cuts to a third of it (matches [`crate::Shape::outline`]).
    pub fn outline(&self, w: f32, h: f32) -> Vec<(f32, f32)> {
        let m = w.min(h).max(0.0);
        let clamp = |c: Corner| match c {
            Corner::Square => Corner::Square,
            Corner::Round(r) => Corner::Round(r.clamp(0.0, m / 2.0)),
            Corner::Scoop(r) => Corner::Scoop(r.clamp(0.0, m / 2.0)),
            Corner::Cut(r) => Corner::Cut(r.clamp(0.0, m / 3.0)),
        };
        let mut p = Vec::with_capacity(16);
        // (vertex, direction into the box along x, along y, entry-on-y-axis)
        corner_points(&mut p, clamp(self.top_left), (0.0, 0.0), (1.0, 1.0), true);
        corner_points(&mut p, clamp(self.top_right), (w, 0.0), (-1.0, 1.0), false);
        corner_points(&mut p, clamp(self.bottom_right), (w, h), (-1.0, -1.0), true);
        corner_points(
            &mut p,
            clamp(self.bottom_left),
            (0.0, h),
            (1.0, -1.0),
            false,
        );
        p
    }
}

fn corner_points(
    out: &mut Vec<(f32, f32)>,
    c: Corner,
    v: (f32, f32),
    s: (f32, f32),
    entry_on_y: bool,
) {
    let r = c.size();
    if c == Corner::Square || r <= 0.0 {
        out.push(v);
        return;
    }
    let on_y = (v.0, v.1 + s.1 * r);
    let on_x = (v.0 + s.0 * r, v.1);
    let (a, b) = if entry_on_y {
        (on_y, on_x)
    } else {
        (on_x, on_y)
    };
    match c {
        Corner::Cut(_) => {
            out.push(a);
            out.push(b);
        }
        Corner::Round(_) | Corner::Scoop(_) => {
            let centre = if matches!(c, Corner::Round(_)) {
                (v.0 + s.0 * r, v.1 + s.1 * r)
            } else {
                v
            };
            let a0 = (a.1 - centre.1).atan2(a.0 - centre.0);
            let a1 = (b.1 - centre.1).atan2(b.0 - centre.0);
            let mut d = a1 - a0;
            while d > std::f32::consts::PI {
                d -= std::f32::consts::TAU;
            }
            while d < -std::f32::consts::PI {
                d += std::f32::consts::TAU;
            }
            for i in 0..=ARC_STEPS {
                let ang = a0 + d * (i as f32 / ARC_STEPS as f32);
                out.push((centre.0 + r * ang.cos(), centre.1 + r * ang.sin()));
            }
        }
        Corner::Square => unreachable!(),
    }
}

/// Named size step on a design system's scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeSize {
    None,
    ExtraSmall,
    Small,
    Medium,
    Large,
    ExtraLarge,
    /// Fully rounded / maximal cut (pill).
    Full,
}

/// Pixel value for each [`ShapeSize`] step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeScale {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl ShapeScale {
    /// Material 3: 4 / 8 / 12 / 16 / 28.
    pub const MATERIAL3: Self = Self {
        xs: 4.0,
        sm: 8.0,
        md: 12.0,
        lg: 16.0,
        xl: 28.0,
    };
    /// Cyberpunk cut sizes.
    pub const CHAMFER: Self = Self {
        xs: 4.0,
        sm: 6.0,
        md: 10.0,
        lg: 14.0,
        xl: 20.0,
    };

    pub fn px(&self, s: ShapeSize) -> f32 {
        match s {
            ShapeSize::None => 0.0,
            ShapeSize::ExtraSmall => self.xs,
            ShapeSize::Small => self.sm,
            ShapeSize::Medium => self.md,
            ShapeSize::Large => self.lg,
            ShapeSize::ExtraLarge => self.xl,
            ShapeSize::Full => FULL,
        }
    }
}

/// What a component *is*, so a design system can shape it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeRole {
    Button,
    Fab,
    Chip,
    Tag,
    Badge,
    Input,
    Card,
    Panel,
    Menu,
    Dialog,
    Sheet,
    Tooltip,
}

/// How a corner is drawn across a whole design system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CornerStyle {
    Round,
    Cut,
    Scoop,
}

/// One role's mapping: size step + which corners get it.
pub type RoleShape = (ShapeRole, ShapeSize, Corners);

/// A design system's shape language.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeSystem {
    pub name: &'static str,
    pub style: CornerStyle,
    pub scale: ShapeScale,
    /// Per-role overrides of `default`.
    pub roles: &'static [RoleShape],
    /// Used for roles not listed.
    pub default: (ShapeSize, Corners),
}

const ALL: Corners = Corners {
    top_left: true,
    top_right: true,
    bottom_right: true,
    bottom_left: true,
};

const M3_ROLES: &[RoleShape] = &[
    (ShapeRole::Button, ShapeSize::Full, ALL),
    (ShapeRole::Fab, ShapeSize::Large, ALL),
    (ShapeRole::Chip, ShapeSize::Small, ALL),
    (ShapeRole::Tag, ShapeSize::Small, ALL),
    (ShapeRole::Badge, ShapeSize::Full, ALL),
    (ShapeRole::Input, ShapeSize::ExtraSmall, Corners::TOP),
    (ShapeRole::Card, ShapeSize::Medium, ALL),
    (ShapeRole::Panel, ShapeSize::Medium, ALL),
    (ShapeRole::Menu, ShapeSize::ExtraSmall, ALL),
    (ShapeRole::Dialog, ShapeSize::ExtraLarge, ALL),
    (ShapeRole::Sheet, ShapeSize::ExtraLarge, Corners::TOP),
    (ShapeRole::Tooltip, ShapeSize::ExtraSmall, ALL),
];

impl ShapeSystem {
    /// Material 3: rounded scale, pill buttons, top-rounded fields.
    pub const fn material3() -> Self {
        Self {
            name: "Material 3",
            style: CornerStyle::Round,
            scale: ShapeScale::MATERIAL3,
            roles: M3_ROLES,
            default: (ShapeSize::Medium, ALL),
        }
    }

    /// Cyberpunk "slab": bottom-right cut on everything.
    pub const fn cyberpunk_slab() -> Self {
        Self::cyberpunk("Slab", Corners::BOTTOM_RIGHT)
    }
    /// Cyberpunk "diagonal": top-left + bottom-right.
    pub const fn cyberpunk_diagonal() -> Self {
        Self::cyberpunk("Diagonal", Corners::DIAGONAL)
    }
    /// Cyberpunk "tab": both top corners.
    pub const fn cyberpunk_tab() -> Self {
        Self::cyberpunk("Tab", Corners::TOP)
    }
    /// Cyberpunk "bracket": both left corners.
    pub const fn cyberpunk_bracket() -> Self {
        Self::cyberpunk("Bracket", Corners::LEFT)
    }

    const fn cyberpunk(name: &'static str, corners: Corners) -> Self {
        Self {
            name,
            style: CornerStyle::Cut,
            scale: ShapeScale::CHAMFER,
            roles: &[],
            default: (ShapeSize::Medium, corners),
        }
    }

    /// No rounding or cutting anywhere.
    pub const fn sharp() -> Self {
        Self {
            name: "Sharp",
            style: CornerStyle::Round,
            scale: ShapeScale::MATERIAL3,
            roles: &[],
            default: (ShapeSize::None, Corners::NONE),
        }
    }

    /// Resolved corners for `role`.
    pub fn spec(&self, role: ShapeRole) -> CornerSpec {
        let (size, sides) = self
            .roles
            .iter()
            .find(|(r, _, _)| *r == role)
            .map(|(_, s, c)| (*s, *c))
            .unwrap_or(self.default);
        let px = self.scale.px(size);
        let corner = if px <= 0.0 {
            Corner::Square
        } else {
            match self.style {
                CornerStyle::Round => Corner::Round(px),
                CornerStyle::Cut => Corner::Cut(px),
                CornerStyle::Scoop => Corner::Scoop(px),
            }
        };
        CornerSpec::on(sides, corner)
    }
}

/// A named, user-selectable shape of a design family.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapePreset {
    pub id: &'static str,
    pub name: &'static str,
    pub system: ShapeSystem,
}

impl ShapePreset {
    /// Presets offered by Cyberpunk.
    pub const CYBERPUNK: &'static [ShapePreset] = &[
        ShapePreset {
            id: "slab",
            name: "Slab",
            system: ShapeSystem::cyberpunk_slab(),
        },
        ShapePreset {
            id: "diagonal",
            name: "Diagonal",
            system: ShapeSystem::cyberpunk_diagonal(),
        },
        ShapePreset {
            id: "tab",
            name: "Tab",
            system: ShapeSystem::cyberpunk_tab(),
        },
        ShapePreset {
            id: "bracket",
            name: "Bracket",
            system: ShapeSystem::cyberpunk_bracket(),
        },
        ShapePreset {
            id: "sharp",
            name: "Sharp",
            system: ShapeSystem::sharp(),
        },
    ];
    /// Presets offered by Material 3.
    pub const MATERIAL3: &'static [ShapePreset] = &[
        ShapePreset {
            id: "m3",
            name: "Material",
            system: ShapeSystem::material3(),
        },
        ShapePreset {
            id: "sharp",
            name: "Sharp",
            system: ShapeSystem::sharp(),
        },
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cut_outline_matches_legacy_shape_outline() {
        let shape = crate::Shape {
            corners: Corners::DIAGONAL,
            radius: 0.0,
            chamfer: 10.0,
            border_width: 1.0,
            control_height: 32.0,
        };
        let spec = CornerSpec::on(Corners::DIAGONAL, Corner::Cut(10.0));
        assert_eq!(spec.outline(120.0, 40.0), shape.outline(120.0, 40.0));
    }

    #[test]
    fn round_outline_stays_in_box_and_pill_clamps() {
        let pill = CornerSpec::uniform(Corner::Round(FULL));
        for (x, y) in pill.outline(100.0, 40.0) {
            assert!((-0.01..=100.01).contains(&x) && (-0.01..=40.01).contains(&y));
        }
        // clamped to half height: leftmost arc point reaches x=0 at mid-height
        assert!(pill.outline(100.0, 40.0).iter().any(|p| p.0 < 0.01));
    }

    #[test]
    fn scoop_bulges_into_box() {
        let s = CornerSpec::on(Corners::TOP_LEFT, Corner::Scoop(10.0));
        let pts = s.outline(100.0, 100.0);
        // concave: every arc point is >= 10 from the (0,0) vertex is false; they
        // lie exactly on the circle around it
        let on_arc = pts.iter().filter(|p| (p.0.hypot(p.1) - 10.0).abs() < 0.01);
        assert!(on_arc.count() >= ARC_STEPS);
    }

    #[test]
    fn m3_roles_map_as_expected() {
        let m3 = ShapeSystem::material3();
        assert_eq!(m3.spec(ShapeRole::Button).uniform_radius(), Some(FULL));
        assert_eq!(m3.spec(ShapeRole::Card).uniform_radius(), Some(12.0));
        let input = m3.spec(ShapeRole::Input);
        assert_eq!(input.top_left, Corner::Round(4.0));
        assert_eq!(input.bottom_left, Corner::Square);
    }

    #[test]
    fn cyberpunk_presets_cut_expected_corners() {
        let slab = ShapeSystem::cyberpunk_slab().spec(ShapeRole::Button);
        assert_eq!(slab.sides(), Corners::BOTTOM_RIGHT);
        assert!(!slab.has_curves());
        assert!(ShapeSystem::sharp().spec(ShapeRole::Card).is_square());
    }

    #[test]
    fn lerp_morphs_between_kinds() {
        let a = CornerSpec::uniform(Corner::Round(16.0));
        let b = CornerSpec::SQUARE;
        assert_eq!(CornerSpec::lerp(&a, &b, 0.5).top_left, Corner::Round(8.0));
        let c = CornerSpec::uniform(Corner::Cut(10.0));
        assert_eq!(
            Corner::lerp(a.top_left, c.top_left, 0.0),
            Corner::Round(16.0)
        );
        assert_eq!(Corner::lerp(a.top_left, c.top_left, 1.0), Corner::Cut(10.0));
    }
}
