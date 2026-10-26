//! The outline surface effects are clipped to.

/// Which corners of a control are cut (chamfered).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Corners {
    pub top_left: bool,
    pub top_right: bool,
    pub bottom_right: bool,
    pub bottom_left: bool,
}

impl Corners {
    pub const NONE: Self = Self {
        top_left: false,
        top_right: false,
        bottom_right: false,
        bottom_left: false,
    };
    /// Only top-left (UL).
    pub const TOP_LEFT: Self = Self {
        top_left: true,
        ..Self::NONE
    };
    /// Only top-right (UR).
    pub const TOP_RIGHT: Self = Self {
        top_right: true,
        ..Self::NONE
    };
    /// Only bottom-left (LL).
    pub const BOTTOM_LEFT: Self = Self {
        bottom_left: true,
        ..Self::NONE
    };
    /// Both top corners.
    pub const TOP: Self = Self {
        top_left: true,
        top_right: true,
        ..Self::NONE
    };
    /// Both bottom corners.
    pub const BOTTOM: Self = Self {
        bottom_left: true,
        bottom_right: true,
        ..Self::NONE
    };
    /// Both left corners.
    pub const LEFT: Self = Self {
        top_left: true,
        bottom_left: true,
        ..Self::NONE
    };
    /// Both right corners.
    pub const RIGHT: Self = Self {
        top_right: true,
        bottom_right: true,
        ..Self::NONE
    };
    /// Top-right and bottom-left (the other diagonal).
    pub const ANTI_DIAGONAL: Self = Self {
        top_right: true,
        bottom_left: true,
        ..Self::NONE
    };
    /// Only bottom-right (LR).
    pub const BOTTOM_RIGHT: Self = Self {
        bottom_right: true,
        ..Self::NONE
    };
    /// Top-left and bottom-right (diagonal).
    pub const DIAGONAL: Self = Self {
        top_left: true,
        bottom_right: true,
        ..Self::NONE
    };
    pub const ALL: Self = Self {
        top_left: true,
        top_right: true,
        bottom_right: true,
        bottom_left: true,
    };

    /// Copy with the corner `top_left` / `top_right` / `bottom_right` /
    /// `bottom_left` (index 0..4, clockwise from top-left) set to `on`.
    pub fn set(mut self, corner: usize, on: bool) -> Self {
        match corner {
            0 => self.top_left = on,
            1 => self.top_right = on,
            2 => self.bottom_right = on,
            _ => self.bottom_left = on,
        }
        self
    }
    /// Whether corner `i` (clockwise from top-left) is cut.
    pub fn get(self, corner: usize) -> bool {
        [
            self.top_left,
            self.top_right,
            self.bottom_right,
            self.bottom_left,
        ][corner.min(3)]
    }

    pub fn any(self) -> bool {
        self.top_left || self.top_right || self.bottom_right || self.bottom_left
    }
}

/// How a theme draws control outlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// Hard 90° corners.
    Square,
    /// Rounded corners (`Shape::radius`).
    Rounded,
    /// Cut corners (`Shape::chamfer` on `Shape::corners`).
    Chamfered,
}

/// Segments in a scooped corner's arc.
const SCOOP_STEPS: usize = 8;

/// The outline an effect is clipped to: corner radius, cut corners and the border inset.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Clip {
    /// Which corners are chamfered when `chamfer > 0`.
    pub corners: Corners,
    /// Plain corner radius (0 for hard-edged outlines).
    pub radius: f32,
    /// Cut-corner size (0 = none).
    pub chamfer: f32,
    /// Draw the chamfered corners as concave quarter-circle arcs instead of straight cuts.
    pub scoop: bool,
    /// Border width: effects stay inside it.
    pub border_width: f32,
}

impl Clip {
    /// Outline style implied by the tokens.
    pub fn kind(&self) -> ShapeKind {
        if self.chamfer > 0.0 && self.corners.any() {
            ShapeKind::Chamfered
        } else if self.radius > 0.0 {
            ShapeKind::Rounded
        } else {
            ShapeKind::Square
        }
    }

    /// Outline polygon (clockwise from top-left) for a `w`×`h` box with this
    /// shape's chamfer, clamped to a third of the shorter side. Shared by every
    /// adapter so all frameworks cut identical corners. For non-chamfered shapes
    /// this is the plain rectangle.
    pub fn outline(&self, w: f32, h: f32) -> Vec<(f32, f32)> {
        let c = if self.kind() == ShapeKind::Chamfered {
            self.chamfer.min(w.min(h) / 3.0).max(0.0)
        } else {
            0.0
        };
        let k = self.corners;
        let mut p = Vec::with_capacity(8);
        // One corner: a straight cut between its two end points, or a concave arc about the
        // box corner `(cx, cy)` when scooped. `at(sin θ, cos θ)` runs from the first end point (θ = 0) to the second.
        let mut corner = |on: bool, plain: (f32, f32), at: &dyn Fn(f32, f32) -> (f32, f32)| {
            if !(on && c > 0.0) {
                p.push(plain);
            } else if self.scoop {
                for i in 0..=SCOOP_STEPS {
                    let a = std::f32::consts::FRAC_PI_2 * i as f32 / SCOOP_STEPS as f32;
                    p.push(at(a.sin(), a.cos().max(0.0)));
                }
            } else {
                p.push(at(0.0, 1.0));
                p.push(at(1.0, 0.0));
            }
        };
        corner(k.top_left, (0.0, 0.0), &|sn, cs| (c * sn, c * cs));
        corner(k.top_right, (w, 0.0), &|sn, cs| (w - c * cs, c * sn));
        corner(k.bottom_right, (w, h), &|sn, cs| (w - c * sn, h - c * cs));
        corner(k.bottom_left, (0.0, h), &|sn, cs| (c * cs, h - c * sn));
        p
    }
}
