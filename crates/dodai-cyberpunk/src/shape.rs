//! Cyberpunk's shape language: chamfered ("cut") corners on a few chosen sides.

use dodai_core::{
    CornerStyle, Corners, DefaultShape, ShapePreset, ShapeScale, ShapeSize, ShapeSystem,
};

/// Cyberpunk cut sizes.
pub const CHAMFER_SCALE: ShapeScale = ShapeScale {
    xs: 4.0,
    sm: 6.0,
    md: 10.0,
    lg: 14.0,
    xl: 20.0,
};

fn cut(name: &'static str, corners: Corners) -> ShapeSystem {
    ShapeSystem {
        name,
        scale: CHAMFER_SCALE,
        roles: std::borrow::Cow::Borrowed(&[]),
        default: DefaultShape {
            style: CornerStyle::Cut,
            size: ShapeSize::Medium,
            sides: corners,
        },
    }
}

/// "Slab": bottom-right cut on everything.
pub fn slab() -> ShapeSystem {
    cut("Slab", Corners::BOTTOM_RIGHT)
}
/// "Diagonal": top-left and bottom-right.
pub fn diagonal() -> ShapeSystem {
    cut("Diagonal", Corners::DIAGONAL)
}
/// "Tab": both top corners.
pub fn tab() -> ShapeSystem {
    cut("Tab", Corners::TOP)
}
/// "Bracket": both left corners.
pub fn bracket() -> ShapeSystem {
    cut("Bracket", Corners::LEFT)
}

/// Shape presets offered by Cyberpunk.
pub fn shape_presets() -> Vec<ShapePreset> {
    vec![
        ShapePreset::new("slab", "Slab", slab()),
        ShapePreset::new("diagonal", "Diagonal", diagonal()),
        ShapePreset::new("tab", "Tab", tab()),
        ShapePreset::new("bracket", "Bracket", bracket()),
        ShapePreset::new("sharp", "Sharp", ShapeSystem::sharp()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use dodai_core::ShapeRole;

    #[test]
    fn presets_cut_expected_corners() {
        let slab = slab().spec(ShapeRole::Button);
        assert_eq!(slab.sides(), Corners::BOTTOM_RIGHT);
        assert!(!slab.has_curves());
    }
}
