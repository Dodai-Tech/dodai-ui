//! App-wide shape tuning: how round the corners are and how borders look, on top of whatever a
//! design defines. Like the surface mix it is a runtime setting, not part of a design.

use crate::{CornerStyle, RoleShape, ShapeKind, ShapeRole, ShapeSize, ShapeSystem, Theme};

/// Which colour borders take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderTint {
    /// Each component's own border colour.
    #[default]
    Design,
    /// The stronger neutral outline.
    Strong,
    /// The primary colour.
    Accent,
    /// The text colour.
    Text,
}

impl BorderTint {
    /// Every tint, in picker order.
    pub const ALL: [Self; 4] = [Self::Design, Self::Strong, Self::Accent, Self::Text];
    /// Picker label (`Design` reads "Default" in the UI).
    pub const fn label(self) -> &'static str {
        match self {
            Self::Design => "Default",
            Self::Strong => "Strong",
            Self::Accent => "Accent",
            Self::Text => "Text",
        }
    }
}

/// Which way a hard offset shadow falls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShadowDir {
    /// Down and to the right (the usual).
    #[default]
    BottomRight,
    /// Down and to the left.
    BottomLeft,
    /// Up and to the right.
    TopRight,
    /// Up and to the left.
    TopLeft,
}

impl ShadowDir {
    /// Every direction, in picker order.
    pub const ALL: [Self; 4] = [
        Self::BottomRight,
        Self::BottomLeft,
        Self::TopRight,
        Self::TopLeft,
    ];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::BottomRight => "Bottom right",
            Self::BottomLeft => "Bottom left",
            Self::TopRight => "Top right",
            Self::TopLeft => "Top left",
        }
    }
    /// Unit offset `(dx, dy)`, y pointing down.
    pub const fn dir(self) -> (f32, f32) {
        match self {
            Self::BottomRight => (1.0, 1.0),
            Self::BottomLeft => (-1.0, 1.0),
            Self::TopRight => (1.0, -1.0),
            Self::TopLeft => (-1.0, -1.0),
        }
    }
}

/// A family of components that can be rounded on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundnessGroup {
    /// Things you press or type into: buttons, chips, tags, badges, inputs, checkboxes.
    Controls,
    /// Things that hold content: cards, panels, menus, dialogs, sheets, bars.
    Containers,
    /// Round or small marks: icon buttons, FABs, indicators, tooltips.
    Marks,
}

impl RoundnessGroup {
    /// Every group, in picker order.
    pub const ALL: [Self; 3] = [Self::Controls, Self::Containers, Self::Marks];
    /// Picker label.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Controls => "Controls",
            Self::Containers => "Containers",
            Self::Marks => "Marks",
        }
    }
    const fn index(self) -> usize {
        match self {
            Self::Controls => 0,
            Self::Containers => 1,
            Self::Marks => 2,
        }
    }
    /// The group a role belongs to.
    pub const fn of(role: ShapeRole) -> Self {
        use ShapeRole::*;
        match role {
            Button | Chip | Tag | Badge | Input | Checkbox | SearchBar => Self::Controls,
            Card | Panel | Menu | Dialog | Sheet | SideSheet | Snackbar | Carousel | Toolbar
            | AppBar | RichTooltip => Self::Containers,
            Fab | FabSmall | FabLarge | IconButton | Indicator | Tooltip => Self::Marks,
        }
    }
}

/// Shape tuning applied to every component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeTune {
    /// Multiplier on rounded corners: 0 square, 1 as designed, above 1 rounder.
    pub roundness: f32,
    /// Extra multiplier per [`RoundnessGroup`] on top of `roundness`, indexed like
    /// [`RoundnessGroup::ALL`]; 1 leaves the group at the global value.
    pub groups: [f32; 3],
    /// Border width in px; `None` keeps each design's own.
    pub border_width: Option<f32>,
    /// Border colour.
    pub tint: BorderTint,
    /// Give components that have no border one.
    pub outline_all: bool,
    /// Offset in px of hard shadows; `None` keeps each design's own (designs without hard
    /// shadows stay without).
    pub hard_shadow: Option<f32>,
    /// Direction hard shadows fall.
    pub shadow_dir: ShadowDir,
}

impl ShapeTune {
    /// The design as shipped.
    pub const DEFAULT: Self = Self {
        roundness: 1.0,
        groups: [1.0; 3],
        border_width: None,
        tint: BorderTint::Design,
        outline_all: false,
        hard_shadow: None,
        shadow_dir: ShadowDir::BottomRight,
    };

    /// Corner multiplier for one role: the global roundness times its group's.
    pub fn roundness_of(&self, role: ShapeRole) -> f32 {
        (self.roundness * self.groups[RoundnessGroup::of(role).index()]).max(0.0)
    }

    /// Whether every role is rounded the same.
    fn uniform(&self) -> bool {
        self.groups.iter().all(|g| (g - 1.0).abs() <= f32::EPSILON)
    }

    /// Whether this changes nothing.
    pub fn is_default(&self) -> bool {
        *self == Self::DEFAULT
    }
}

impl Default for ShapeTune {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A pill's real radius as a share of the control height. Below half (0.5 = a full pill) so
/// roundness above 100% can still make pills rounder, and pills scale like every other corner.
const PILL_OF_CONTROL: f32 = 0.4;

impl ShapeSystem {
    /// Whether any role (or the fallback) is a pill.
    fn has_pill(&self) -> bool {
        self.default.size == ShapeSize::Full || self.roles.iter().any(|r| r.size == ShapeSize::Full)
    }

    /// This shape language with every rounded corner scaled by `f`: 0 is square, 1 unchanged.
    /// See [`with_roundness_by`](Self::with_roundness_by).
    pub fn with_roundness(&self, f: f32, control_height: f32) -> Self {
        self.with_roundness_by(control_height, |_| f)
    }

    /// This shape language with each role's rounded corners scaled by `factor(role)`.
    /// Cut corners keep their size. A fully rounded corner becomes a real radius
    /// (`PILL_OF_CONTROL` of `control_height`), scaled like the rest.
    pub fn with_roundness_by(
        &self,
        control_height: f32,
        factor: impl Fn(ShapeRole) -> f32,
    ) -> Self {
        let px = |size: ShapeSize| match size {
            ShapeSize::Full => PILL_OF_CONTROL * control_height,
            other => self.scale.px(other),
        };
        let mut out = self.clone();
        // Spell out every role so each can take its own factor, not just the listed ones.
        out.roles = ShapeRole::ALL
            .iter()
            .map(|&role| {
                let (style, size, sides) = self
                    .roles
                    .iter()
                    .rev()
                    .find(|e| e.role == role)
                    .map(|e| (e.style, e.size, e.sides))
                    .unwrap_or((self.default.style, self.default.size, self.default.sides));
                let size = if style == CornerStyle::Cut || size == ShapeSize::None {
                    size
                } else {
                    ShapeSize::Px(px(size) * factor(role).max(0.0))
                };
                RoleShape::new(role, style, size, sides)
            })
            .collect();
        // Anything reading the named steps directly follows the Controls factor.
        let f = factor(ShapeRole::Button).max(0.0);
        out.scale.xs *= f;
        out.scale.sm *= f;
        out.scale.md *= f;
        out.scale.lg *= f;
        out.scale.xl *= f;
        out
    }
}

impl Theme {
    /// This theme with `tune` applied. Chamfered designs keep their cuts; roundness only
    /// affects rounded corners.
    pub fn tuned(&self, tune: &ShapeTune) -> Theme {
        let mut t = self.clone();
        let sys = self.shape_system();
        let rounds = (tune.roundness - 1.0).abs() > f32::EPSILON || !tune.uniform();
        if (rounds || sys.has_pill()) && t.shape.kind() != ShapeKind::Chamfered {
            t.shape.radius *= tune.roundness.max(0.0);
            t.shape_system =
                Some(sys.with_roundness_by(self.shape.control_height, |r| tune.roundness_of(r)));
        }
        if let Some(w) = tune.border_width {
            t.shape.border_width = w.max(0.0);
        }
        if let Some(s) = tune.hard_shadow {
            if t.effects.hard_shadow > 0.0 {
                t.effects.hard_shadow = s.max(0.0);
            }
        }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tuning_changes_nothing() {
        let t = Theme::default();
        assert_eq!(t.tuned(&ShapeTune::DEFAULT), t);
    }

    #[test]
    fn roundness_zero_makes_every_role_square() {
        let sys = ShapeSystem::from_shape(&crate::Shape {
            corners: crate::Corners::NONE,
            radius: 12.0,
            chamfer: 0.0,
            border_width: 1.0,
            control_height: 40.0,
        });
        let sq = sys.with_roundness(0.0, 40.0);
        assert_eq!(
            sq.spec(crate::ShapeRole::Button).top_left.size(),
            0.0,
            "scaled to nothing"
        );
        assert!(sys.spec(crate::ShapeRole::Button).top_left.size() > 0.0);
    }

    fn pill_system() -> ShapeSystem {
        ShapeSystem::from_shape(&crate::Shape {
            corners: crate::Corners::NONE,
            radius: 12.0,
            chamfer: 0.0,
            border_width: 1.0,
            control_height: 40.0,
        })
        .with_default(
            crate::CornerStyle::Round,
            ShapeSize::Full,
            crate::Corners::ALL,
        )
        .with_role(
            ShapeRole::Card,
            crate::CornerStyle::Round,
            ShapeSize::Medium,
            crate::Corners::ALL,
        )
    }

    fn size(sys: &ShapeSystem, role: ShapeRole) -> f32 {
        sys.spec(role).top_left.size()
    }

    #[test]
    fn pills_scale_like_any_other_corner() {
        let sys = pill_system();
        // 100%: a real radius below a full pill (20 on a 40 px control), so there is headroom.
        assert_eq!(size(&sys.with_roundness(1.0, 40.0), ShapeRole::Chip), 16.0);
        assert_eq!(size(&sys.with_roundness(1.5, 40.0), ShapeRole::Chip), 24.0);
        assert_eq!(size(&sys.with_roundness(0.5, 40.0), ShapeRole::Chip), 8.0);
        assert_eq!(size(&sys.with_roundness(2.0, 40.0), ShapeRole::Card), 24.0);
    }

    #[test]
    fn groups_round_only_their_own_roles() {
        let sys = pill_system();
        let by = sys.with_roundness_by(40.0, |r| {
            if RoundnessGroup::of(r) == RoundnessGroup::Containers {
                0.0
            } else {
                1.0
            }
        });
        assert_eq!(size(&by, ShapeRole::Card), 0.0, "containers squared");
        assert_eq!(size(&by, ShapeRole::Chip), 16.0, "controls untouched");
    }

    #[test]
    fn tune_groups_multiply_the_global_roundness() {
        let mut tune = ShapeTune::DEFAULT;
        tune.roundness = 2.0;
        tune.groups[1] = 0.5;
        assert_eq!(tune.roundness_of(ShapeRole::Card), 1.0);
        assert_eq!(tune.roundness_of(ShapeRole::Button), 2.0);
    }

    #[test]
    fn every_role_has_a_group() {
        for r in ShapeRole::ALL {
            let _ = RoundnessGroup::of(r);
        }
    }
}
