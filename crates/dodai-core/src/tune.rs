//! App-wide shape tuning: how round the corners are and how borders look, on top of whatever a
//! design defines. Like the surface mix it is a runtime setting, not part of a design.

use crate::{ShapeKind, ShapeSize, ShapeSystem, Theme};

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

/// Shape tuning applied to every component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShapeTune {
    /// Multiplier on rounded corners: 0 square, 1 as designed, above 1 rounder.
    pub roundness: f32,
    /// Border width in px; `None` keeps each design's own.
    pub border_width: Option<f32>,
    /// Border colour.
    pub tint: BorderTint,
    /// Give components that have no border one.
    pub outline_all: bool,
}

impl ShapeTune {
    /// The design as shipped.
    pub const DEFAULT: Self = Self {
        roundness: 1.0,
        border_width: None,
        tint: BorderTint::Design,
        outline_all: false,
    };

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

/// Radius used for "fully rounded" once the roundness is turned down.
const FULL_AT_ONE: f32 = 28.0;

impl ShapeSystem {
    /// This shape language with every rounded corner scaled by `f`: 0 is square, 1 unchanged.
    /// Cut corners keep their size. Fully rounded corners stay pills until `f` drops below 1.
    pub fn with_roundness(&self, f: f32) -> Self {
        use crate::CornerStyle::Cut;
        let f = f.max(0.0);
        let size = |s: ShapeSize| match s {
            ShapeSize::Px(v) => ShapeSize::Px(v * f),
            ShapeSize::Full if f < 1.0 => ShapeSize::Px(FULL_AT_ONE * f),
            other => other,
        };
        let mut out = self.clone();
        out.scale.xs *= f;
        out.scale.sm *= f;
        out.scale.md *= f;
        out.scale.lg *= f;
        out.scale.xl *= f;
        out.default.size = size(out.default.size);
        out.roles = out
            .roles
            .iter()
            .map(|r| {
                let mut r = *r;
                if r.style != Cut {
                    r.size = size(r.size);
                }
                r
            })
            .collect();
        out
    }
}

impl Theme {
    /// This theme with `tune` applied. Chamfered designs keep their cuts; roundness only
    /// affects rounded corners.
    pub fn tuned(&self, tune: &ShapeTune) -> Theme {
        let mut t = self.clone();
        if (tune.roundness - 1.0).abs() > f32::EPSILON && t.shape.kind() != ShapeKind::Chamfered {
            t.shape.radius *= tune.roundness.max(0.0);
            t.shape_system = Some(self.shape_system().with_roundness(tune.roundness));
        }
        if let Some(w) = tune.border_width {
            t.shape.border_width = w.max(0.0);
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
        let sq = sys.with_roundness(0.0);
        assert_eq!(
            sq.spec(crate::ShapeRole::Button).top_left.size(),
            0.0,
            "scaled to nothing"
        );
        assert!(sys.spec(crate::ShapeRole::Button).top_left.size() > 0.0);
    }

    #[test]
    fn pills_stay_pills_at_full_roundness_and_shrink_below() {
        let sys = ShapeSystem::from_shape(&crate::Shape {
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
        );
        assert!(
            sys.with_roundness(1.0)
                .spec(crate::ShapeRole::Chip)
                .top_left
                .size()
                > 1000.0
        );
        assert!(
            sys.with_roundness(0.5)
                .spec(crate::ShapeRole::Chip)
                .top_left
                .size()
                < 20.0
        );
    }
}
