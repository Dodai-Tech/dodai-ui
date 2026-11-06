//! **Button crops**: one end of a button reshaped to point at a neighbour or an edge.
//!
//! The app picks *which end* ([`Crop`]); the design decides *how it looks* ([`ButtonCrop`])
//! through [`crate::DesignSpec::button_crop`], so a crop always speaks the shape language of
//! the design it is drawn in. An end a design has no crop for is drawn uncropped.

use crate::{Corners, Theme};

/// Which end of a button is cropped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crop {
    /// The left end (the top-left corner, for designs that crop one corner).
    Left,
    /// The right end (the bottom-right corner, for designs that crop one corner).
    Right,
    /// The top edge.
    Top,
    /// The bottom edge.
    Bottom,
}

impl Crop {
    /// Every end, in picker order.
    pub const ALL: [Crop; 4] = [Crop::Left, Crop::Right, Crop::Top, Crop::Bottom];

    /// Display name.
    pub fn name(self) -> &'static str {
        match self {
            Crop::Left => "Left",
            Crop::Right => "Right",
            Crop::Top => "Top",
            Crop::Bottom => "Bottom",
        }
    }
}

/// A side of a button, where a tick or a slot sits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CropEdge {
    Left,
    Right,
    Top,
    Bottom,
}

/// How a design draws one cropped end, in its own shape language.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ButtonCrop {
    /// `corners` cut straight at 45°, and a double tick opening the border on `tick`.
    CutTicked { corners: Corners, tick: CropEdge },
    /// A wide, shallow slot of `depth` px in the middle of `edge`; the corners stay.
    Slot { edge: CropEdge, depth: f32 },
    /// `corners` drawn with a tight `radius` px rounding and the others with `rest` px
    /// ([`crate::FULL`] for a pill end).
    Squared {
        corners: Corners,
        radius: f32,
        rest: f32,
    },
    /// Concave quarter-circle bites of `size` px out of `corners` (a ticket stub).
    Scooped { corners: Corners, size: f32 },
}

impl ButtonCrop {
    /// What the crop looks like, for pickers and hints.
    pub fn describe(&self) -> &'static str {
        match self {
            ButtonCrop::CutTicked { .. } => "a cut corner, with a double tick on the far end",
            ButtonCrop::Slot { .. } => "a shallow slot in the edge",
            ButtonCrop::Squared { .. } => "tight corners where the other ends stay rounded",
            ButtonCrop::Scooped { .. } => "concave bites out of the corners, like a ticket stub",
        }
    }

    /// Whether the crop draws a tick (and so uses the tick gap).
    pub fn has_tick(&self) -> bool {
        matches!(self, ButtonCrop::CutTicked { .. })
    }
}

impl Theme {
    /// How this theme's design crops `end` of a button, or `None` when it has no crop there.
    pub fn button_crop(&self, end: Crop) -> Option<ButtonCrop> {
        self.spec.get().and_then(|s| s.button_crop(self, end))
    }

    /// The ends this theme's design can crop, in picker order.
    pub fn button_crops(&self) -> Vec<Crop> {
        Crop::ALL
            .into_iter()
            .filter(|c| self.button_crop(*c).is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_theme_without_a_design_crops_nothing() {
        let t = crate::ThemeSwitcher::default().current().clone();
        assert!(t.button_crops().is_empty());
        assert_eq!(t.button_crop(Crop::Left), None);
    }

    #[test]
    fn only_cut_crops_have_a_tick() {
        let cut = ButtonCrop::CutTicked {
            corners: Corners::TOP_LEFT,
            tick: CropEdge::Right,
        };
        let leaf = ButtonCrop::Squared {
            corners: Corners::DIAGONAL,
            radius: 4.0,
            rest: crate::FULL,
        };
        assert!(cut.has_tick());
        assert!(!leaf.has_tick());
        assert_ne!(cut.describe(), leaf.describe());
    }

    #[cfg(all(
        feature = "cyberpunk",
        feature = "dodai",
        feature = "material",
        feature = "neobrutalism"
    ))]
    #[test]
    fn every_design_crops_in_its_own_way() {
        use crate::themes;
        let kind = |t: &Theme| std::mem::discriminant(&t.button_crop(Crop::Left).unwrap());
        let designs = [
            themes::cyberpunk::yellow(),
            themes::dodai::light(),
            themes::material::light(),
            themes::neobrutalism::light(),
        ];
        assert_eq!(designs[0].button_crops(), Crop::ALL.to_vec());
        assert_eq!(designs[1].button_crops(), vec![Crop::Left, Crop::Right]);
        // No design borrows the cut-and-tick crop.
        for t in &designs[1..] {
            assert!(!t.button_crop(Crop::Left).unwrap().has_tick());
        }
        assert_ne!(kind(&designs[0]), kind(&designs[3]));
        assert_ne!(kind(&designs[1]), kind(&designs[3]));
        // Dodai's leaf tightens the diagonal, Material's connected end one side.
        assert_ne!(
            designs[1].button_crop(Crop::Left),
            designs[2].button_crop(Crop::Left)
        );
    }
}
