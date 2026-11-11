//! The look every vehicle instrument shares ([`Look`], defined with the designs so each can name
//! its own), and how big a face is drawn in each layout.

pub use dodai_core::vehicle_look::{AtLimit, Layout, Look, PodLayout};

/// The look an instrument draws in: the one it was given, else the design's own
/// ([`ChartStyle::vehicle`](dodai_core::ChartStyle::vehicle)).
pub(crate) fn resolve(chosen: Option<Look>, cx: &gpui::App) -> Look {
    chosen.unwrap_or_else(|| dodai_gpui::theme_of(cx).chart_style().vehicle)
}

/// The proportions of `look`'s face in `layout` (after [`Look::fit`]). Pure.
pub(crate) fn shape(look: Look, layout: Layout) -> Shape {
    match (look, look.fit(layout)) {
        (Look::HeadsUp, Layout::Tape)
        | (
            _,
            Layout::Dial
            | Layout::Bezel
            | Layout::Sweep
            | Layout::Track
            | Layout::Sector
            | Layout::Ribbed
            | Layout::Cut
            | Layout::Band
            | Layout::Lit
            | Layout::Fill
            | Layout::Ring
            | Layout::Pointer,
        ) => Shape::Square,
        (_, Layout::Tape | Layout::Column) => Shape::Tall,
        _ => Shape::Wide,
    }
}

/// The layout a single-reading pod draws in `layout`: the look's own, except that the side
/// displays of [`Look::Daylight`] are its dark ring dials, not the white dial of the tachometer.
/// Pure.
pub(crate) fn side(look: Look, layout: Layout) -> Layout {
    match (look, look.fit(layout)) {
        (Look::Daylight, Layout::Dial) => Layout::Ring,
        (_, layout) => layout,
    }
}

/// The proportions of a face: square as a dial, wider than tall, or taller than wide.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Shape {
    /// As wide as tall (heads-up tapes fill the square too).
    Square,
    /// As wide as the size, flatter than a dial.
    Wide,
    /// As tall as the size and under half as wide.
    Tall,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tapes_and_columns_stand_upright_and_the_rest_lie_flat() {
        assert_eq!(shape(Look::HeadsUp, Layout::Tape), Shape::Square);
        assert_eq!(shape(Look::GlassCockpit, Layout::Tape), Shape::Tall);
        assert_eq!(shape(Look::Segment, Layout::Column), Shape::Tall);
        assert_eq!(shape(Look::Chronograph, Layout::Bezel), Shape::Square);
        assert_eq!(shape(Look::Chronograph, Layout::Register), Shape::Wide);
        assert_eq!(shape(Look::Sport, Layout::Track), Shape::Square);
        assert_eq!(shape(Look::Chronograph, Layout::Sector), Shape::Square);
        assert_eq!(shape(Look::Vintage, Layout::Bezel), Shape::Square);
        assert_eq!(shape(Look::Halo, Layout::Bar), Shape::Wide);
        assert_eq!(shape(Look::Race, Layout::Arc), Shape::Wide);
        assert_eq!(shape(Look::Line, Layout::Column), Shape::Tall);
        assert_eq!(
            shape(Look::Race, Layout::Tape),
            Shape::Square,
            "falls back to the dial"
        );
    }
}
