//! Small building blocks shared by the component modules.

use bevy::prelude::*;

use crate::widgets::{text, TextRole};

/// A column of children.
pub(crate) fn column() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        ..default()
    }
}

/// A row of vertically centred children with a gap.
pub(crate) fn row(gap: f32) -> Node {
    Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(gap),
        ..default()
    }
}

/// Full-width block.
pub(crate) fn wide(mut n: Node) -> Node {
    n.width = Val::Percent(100.0);
    n
}

/// The glyphs the components draw: an icon from `dodai-icons` (in the active design's pack) with
/// the `icons` feature, a plain character without it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Glyph {
    ChevronRight,
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    Star,
    Plus,
    Minus,
    Close,
    Search,
    Menu,
    Upload,
}

impl Glyph {
    #[cfg(feature = "icons")]
    pub(crate) fn id(self) -> dodai_icons::IconId {
        use dodai_icons::IconId as I;
        match self {
            Glyph::ChevronRight => I::ChevronRight,
            Glyph::ChevronDown => I::ChevronDown,
            Glyph::ChevronUp => I::ChevronUp,
            Glyph::ChevronLeft => I::ChevronLeft,
            Glyph::Star => I::Star,
            Glyph::Plus => I::Plus,
            Glyph::Minus => I::Minus,
            Glyph::Close => I::Close,
            Glyph::Search => I::Search,
            Glyph::Menu => I::Menu,
            Glyph::Upload => I::Upload,
        }
    }

    #[cfg(not(feature = "icons"))]
    fn fallback(self) -> &'static str {
        match self {
            Glyph::ChevronRight => ">",
            Glyph::ChevronDown => "v",
            Glyph::ChevronUp => "^",
            Glyph::ChevronLeft => "<",
            Glyph::Star => "*",
            Glyph::Plus => "+",
            Glyph::Minus => "-",
            Glyph::Close => "x",
            Glyph::Search => "?",
            Glyph::Menu => "=",
            Glyph::Upload => "^",
        }
    }
}

/// `g` at `px`.
#[cfg(feature = "icons")]
pub(crate) fn glyph(g: Glyph, px: u32) -> impl Bundle {
    crate::widgets::icon(g.id(), px)
}

/// `g` at `px` (the `icons` feature is off: a character).
#[cfg(not(feature = "icons"))]
pub(crate) fn glyph(g: Glyph, _px: u32) -> impl Bundle {
    text(g.fallback(), TextRole::Control)
}

/// The glyph a [`glyph`] entity should show; [`crate::interact`] redraws it when this changes.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct GlyphSlot(pub Glyph);

/// A [`glyph`] whose shape can change later through its [`GlyphSlot`].
pub(crate) fn glyph_slot(g: Glyph, px: u32) -> impl Bundle {
    (glyph(g, px), GlyphSlot(g))
}

/// A caption-sized label.
pub(crate) fn caption(s: impl Into<String>) -> impl Bundle {
    text(s, TextRole::Caption)
}

/// Whether a star of a rating is lit.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StarFill(pub bool);
