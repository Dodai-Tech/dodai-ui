//! Vector [`Icon`]s drawn with gpui paths (no asset files, any colour/size).
//!
//! An [`Icon`] is named by writing it: `Icon::Heart`, `Icon::HouseLine.size(20.0)` (typed: a typo
//! is a compile error, and the editor completes it). It is drawn in the
//! style of the active design system: Cyberpunk's straight segments and chamfers, Material 3's
//! Material Symbols, Neo-Brutalism's bold Phosphor. Every icon draws in every design. Force a pack
//! with [`Icon::pack`].

use crate::paint::stroke;
pub use dodai_icons::cyberpunk::IconName;
pub use dodai_icons::IconId;
#[cfg(feature = "cyberpunk")]
pub use dodai_icons::Modifier;
use dodai_icons::{Glyph, Pack};
use gpui::{canvas, div, px, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled};

/// Which icon pack an [`Icon`] draws from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconPack {
    /// Follow the theme's design system.
    #[default]
    Auto,
    /// Cyberpunk: hand-drawn straight segments (Phosphor, faceted, where it has none).
    Stroked,
    /// Material Symbols (Phosphor, as is, where Material has none).
    Material,
    /// Neo-Brutalism: bold Phosphor.
    Bold,
}

impl IconPack {
    /// Resolve `Auto` against the design system the theme declares.
    pub(crate) fn resolve(self, set: dodai_core::IconSet) -> Pack {
        match self {
            Self::Auto => set.into(),
            Self::Stroked => Pack::Stroked,
            Self::Material => Pack::Material,
            Self::Bold => Pack::Bold,
        }
    }
}

/// Vector icon. Defaults: 16 px, the theme text colour; set `.color(..)` to override.
#[derive(IntoElement)]
pub struct Icon {
    id: IconId,
    /// A derived Cyberpunk variant (`heart` + `Add`); always the stroked drawing.
    #[cfg(feature = "cyberpunk")]
    modifier: Option<Modifier>,
    size: f32,
    color: Option<Hsla>,
    weight: f32,
    pack: IconPack,
    filled: bool,
}

impl Icon {
    /// An icon from a value chosen at run time (`Icon::new(id)`); written out by hand, name it
    /// directly: `Icon::Heart`, `Icon::HouseLine`. A hand-drawn [`IconName`] converts too.
    pub fn new(id: impl Into<IconId>) -> Self {
        Self::of(id.into())
    }

    /// `const` constructor behind the generated `Icon::House`-style constants.
    pub(crate) const fn of(id: IconId) -> Self {
        Self {
            id,
            #[cfg(feature = "cyberpunk")]
            modifier: None,
            size: 16.0,
            color: None,
            weight: 1.75,
            pack: IconPack::Auto,
            filled: false,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Colour (default: the theme's).
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
    /// Stroke width in px at 24 px size (scales with `size`; stroked drawings only).
    pub fn weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
    /// Force an icon pack instead of following the theme.
    pub fn pack(mut self, pack: IconPack) -> Self {
        self.pack = pack;
        self
    }
    /// Use the filled form where the pack has one.
    pub fn filled(mut self, on: bool) -> Self {
        self.filled = on;
        self
    }
    /// A derived variant of a hand-drawn Cyberpunk icon (`Modifier::Add` on a heart is
    /// "add to favourites"). Always the stroked drawing, whatever the theme.
    #[cfg(feature = "cyberpunk")]
    pub fn modifier(mut self, m: Modifier) -> Self {
        self.modifier = Some(m);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = crate::theme_of(cx);
        let color = self
            .color
            .unwrap_or_else(|| crate::hsla(theme.palette.text));
        let (size, weight) = (self.size, self.weight);
        let pack = self.pack.resolve(theme.look().icons);
        #[cfg(feature = "cyberpunk")]
        let glyph = match self.modifier {
            Some(m) => dodai_icons::cyber_shapes(self.id, Some(m)).map(Glyph::Stroked),
            None => dodai_icons::glyph(self.id, pack, self.filled),
        };
        #[cfg(not(feature = "cyberpunk"))]
        let glyph = dodai_icons::glyph(self.id, pack, self.filled);
        draw(glyph, size, weight, color)
    }
}

/// Draw a resolved glyph: strokes at `weight`, fills as one non-zero path batch.
fn draw(glyph: Option<Glyph>, size: f32, weight: f32, color: Hsla) -> gpui::AnyElement {
    let k = size / 24.0;
    let blank = || div().size(px(size)).flex_none();
    let Some(glyph) = glyph else {
        return blank().into_any_element();
    };
    match glyph {
        Glyph::Stroked(shapes) => blank()
            .child(
                canvas(
                    |_, _, _| (),
                    move |b: Bounds<Pixels>, _, w, _| {
                        for (p, closed) in &shapes {
                            let p: Vec<(f32, f32)> =
                                p.iter().map(|&(x, y)| (x * k, y * k)).collect();
                            stroke(w, b.origin, &p, (weight * k).max(1.0), color, *closed);
                        }
                    },
                )
                .size_full(),
            )
            .into_any_element(),
        Glyph::Filled(polys) => {
            let polys: Vec<Vec<(f32, f32)>> = polys
                .into_iter()
                .map(|p| p.into_iter().map(|(x, y)| (x * k, y * k)).collect())
                .collect();
            blank()
                .child(
                    canvas(
                        |_, _, _| (),
                        move |b: Bounds<Pixels>, _, w, _| {
                            crate::paint::fill_batch(w, b.origin, &polys, color);
                        },
                    )
                    .size_full(),
                )
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_pack_follows_the_design_system() {
        use dodai_core::IconSet::{Bold, Stroked, Symbols};
        assert_eq!(IconPack::Auto.resolve(Symbols), Pack::Material);
        assert_eq!(IconPack::Auto.resolve(Stroked), Pack::Stroked);
        assert_eq!(IconPack::Auto.resolve(Bold), Pack::Bold);
        assert_eq!(IconPack::Stroked.resolve(Symbols), Pack::Stroked);
    }

    #[test]
    fn a_hand_drawn_name_is_the_same_icon_as_its_id() {
        assert_eq!(IconId::from(IconName::Heart), IconId::Heart);
    }
}
