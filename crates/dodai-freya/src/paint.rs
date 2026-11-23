//! Token paint → Freya values. The only place that turns [`crate::style`] data into
//! Freya types, so the rest of the adapter stays free of conversions.

use crate::style::{chamfer_svg, ShapeStyle, TextStyle};
use dodai_core::{Color as Rgba, Theme};
use freya::prelude::*;

/// Token colour → Freya colour (straight alpha, `0..=255`).
/// The app root: full size, the canvas colour, and for a design with a gradient canvas (Dodai)
/// its mesh stretched behind whatever children the caller adds. This is the default way to
/// start a window.
pub fn root(theme: &Theme) -> Rect {
    let base = rect()
        .width(Size::fill())
        .height(Size::fill())
        .background(color(theme.palette.background));
    match dodai_core::backdrop_svg(theme) {
        Some(svg) => base.child(
            rect()
                .position(Position::new_absolute())
                .width(Size::fill())
                .height(Size::fill())
                .child(
                    SvgViewer::new(Bytes::from(svg.into_bytes()))
                        .show_loader(false)
                        .width(Size::fill())
                        .height(Size::fill()),
                ),
        ),
        None => base,
    }
}

pub fn color(c: Rgba) -> Color {
    Color::from_argb(c.a, c.r, c.g, c.b)
}

/// Border of the rect path (`None` for borderless or chamfered boxes).
fn border(style: &ShapeStyle) -> Option<Border> {
    style.border().map(|(c, w)| {
        Border::new()
            .fill(color(c))
            .width(w)
            .alignment(BorderAlignment::Inner)
    })
}

/// A label painted from a resolved [`TextStyle`]; `fg` overrides the role colour.
pub fn text(ts: &TextStyle, fg: Option<Rgba>, s: &str) -> Label {
    label()
        .text(ts.apply(s))
        .color(color(fg.unwrap_or(ts.color)))
        .font_size(ts.size)
        .font_family(ts.family)
        .font_weight(i32::from(ts.weight))
        .letter_spacing(ts.letter_spacing)
}

/// A shaped box of `w`×`h` with `child` inside (centred, or top-left when `vertical`).
///
/// `Rounded` and `Square` themes paint the rect itself; `Chamfered` themes put the
/// outline polygon from [`dodai_core::Shape::outline`] behind the content.
pub fn shaped(
    theme: &Theme,
    style: &ShapeStyle,
    w: f32,
    h: f32,
    padding: f32,
    vertical: bool,
    child: impl IntoElement,
) -> Rect {
    let (direction, main, cross) = if vertical {
        (Direction::Vertical, Alignment::Start, Alignment::Start)
    } else {
        (Direction::Horizontal, Alignment::Center, Alignment::Center)
    };
    let boxed = rect()
        .width(Size::px(w))
        .height(Size::px(h))
        .padding(Gaps::new_all(padding))
        .direction(direction)
        .main_align(main)
        .cross_align(cross);
    let boxed = if style.is_chamfered() {
        let pad = style.svg_pad();
        let svg = chamfer_svg(theme, style, w, h).into_bytes();
        // Behind the content: the polygon is rasterised separately and must not cover it.
        boxed.child(
            rect()
                .position(Position::new_absolute().top(-pad).left(-pad))
                .layer(-1)
                .child(
                    SvgViewer::new(Bytes::from(svg))
                        .show_loader(false)
                        .width(Size::px(w + 2.0 * pad))
                        .height(Size::px(h + 2.0 * pad)),
                ),
        )
    } else {
        let boxed = match style.fill {
            Some(c) => boxed.background(color(c)),
            None => boxed,
        };
        let boxed = match style.shadow() {
            Some((x, y, blur, c)) => {
                boxed.shadow(Shadow::new().x(x).y(y).blur(blur).color(color(c)))
            }
            None => boxed,
        };
        let boxed = match style.blur {
            Some(b) => boxed.backdrop_blur(b),
            None => boxed,
        };
        boxed
            .border(border(style))
            .corner_radius(style.corner_radius())
    };
    boxed.child(child)
}
