//! Shape-driven border glyphs: [`ShapeKind`] picks the border set, and for
//! chamfered shapes only the corners the theme cuts get a diagonal glyph. A scooped corner
//! (`Shape::scoop`) is drawn like a straight cut: a terminal cell has no concave arc glyph.

use dodai_core::{Shape, ShapeKind};
use ratatui::symbols::border;

/// Square corners (`┌┐└┘`).
pub const SQUARE_BORDER: border::Set<'static> = border::PLAIN;
/// Rounded corners (`╭╮╰╯`).
pub const ROUNDED_BORDER: border::Set<'static> = border::ROUNDED;

/// Cut-corner border with only the bottom-right corner cut (`╱`).
pub const CHAMFER_BORDER: border::Set<'static> = border::Set {
    bottom_right: "╱",
    ..border::PLAIN
};

/// Border glyph set for a shape: plain / rounded / chamfered (honouring
/// which `corners` are cut: `╱` top-left & bottom-right, `╲` top-right & bottom-left).
pub fn border_set(shape: &Shape) -> border::Set<'static> {
    match shape.kind() {
        ShapeKind::Square => SQUARE_BORDER,
        ShapeKind::Rounded => ROUNDED_BORDER,
        ShapeKind::Chamfered => {
            let k = shape.corners;
            let mut s = border::PLAIN;
            if k.top_left {
                s.top_left = "╱";
            }
            if k.top_right {
                s.top_right = "╲";
            }
            if k.bottom_right {
                s.bottom_right = "╱";
            }
            if k.bottom_left {
                s.bottom_left = "╲";
            }
            s
        }
    }
}

/// True when the shape cuts its bottom-right corner (drives the `◤` button/tab tip).
pub fn cuts_bottom_right(shape: &Shape) -> bool {
    shape.kind() == ShapeKind::Chamfered && shape.corners.bottom_right
}
