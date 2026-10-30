//! Vector [`Icon`]s drawn with gpui paths (no asset files, any colour/size).
//!
//! An [`Icon`] is named by writing it: `Icon::Heart`, `Icon::HouseLine.size(20.0)` (typed: a typo
//! is a compile error, and the editor completes it). It is drawn in the
//! style of the active design system: Cyberpunk's straight segments and chamfers, Material 3's
//! Material Symbols, Neo-Brutalism's bold Phosphor. Every icon draws in every design. Force a pack
//! with [`Icon::pack`].

pub use dodai_icons::cyberpunk::IconName;
pub use dodai_icons::IconId;
#[cfg(feature = "cyberpunk")]
pub use dodai_icons::Modifier;
use dodai_icons::{Glyph, Pack};
use gpui::{canvas, px, Bounds, Hsla, IntoElement, Pixels, RenderOnce, Styled};

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
        let key = MeshKey {
            id: self.id,
            pack: self.pack.resolve(theme.look().icons),
            filled: self.filled,
            #[cfg(feature = "cyberpunk")]
            modifier: self.modifier,
            size: self.size.to_bits(),
            weight: self.weight.to_bits(),
        };
        let mesh = Mesh::cached(key);
        // One sized canvas: a single layout node per icon (pages hold hundreds).
        canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                if let Some(mesh) = &mesh {
                    mesh.paint(w, b.origin, color);
                }
            },
        )
        .size(px(self.size))
        .flex_none()
    }
}

/// What an icon's triangles depend on: the drawing, its size and its stroke weight.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct MeshKey {
    id: IconId,
    pack: Pack,
    filled: bool,
    #[cfg(feature = "cyberpunk")]
    modifier: Option<Modifier>,
    size: u32,
    weight: u32,
}

/// Icon meshes kept per thread (a few screens of distinct icon, size and weight).
const MESH_LIMIT: usize = 2048;

/// An icon tessellated once, as triangles relative to its top-left corner. Painting only
/// offsets them: a page of icons used to re-tessellate every one of them on every scroll frame.
struct Mesh(Vec<[(f32, f32); 3]>);

impl Mesh {
    /// The mesh for `key`, built on first use; `None` when the pack has no such icon.
    fn cached(key: MeshKey) -> Option<std::rc::Rc<Self>> {
        // Bounded: sizes can animate or follow density, so keys keep coming; the least
        // recently drawn mesh goes first.
        thread_local! {
            static MESHES: std::cell::RefCell<crate::lru::Lru<MeshKey, Option<std::rc::Rc<Mesh>>>> =
                std::cell::RefCell::new(crate::lru::Lru::new(MESH_LIMIT));
        }
        MESHES.with(|m| {
            let mut m = m.borrow_mut();
            if let Some(v) = m.get(key) {
                return v;
            }
            let v = Self::build(key).map(std::rc::Rc::new);
            m.insert(key, v.clone());
            v
        })
    }

    /// Resolve the glyph and tessellate it: strokes at the weight, fills as one even-odd path
    /// (the options gpui's `PathBuilder` uses, so icons look as before).
    fn build(key: MeshKey) -> Option<Self> {
        use lyon_tessellation::math::point;
        use lyon_tessellation::path::Path;
        use lyon_tessellation::{
            BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions,
            StrokeTessellator, StrokeVertex, VertexBuffers,
        };
        #[cfg(feature = "cyberpunk")]
        let glyph = match key.modifier {
            Some(m) => dodai_icons::cyber_shapes(key.id, Some(m)).map(Glyph::Stroked),
            None => dodai_icons::glyph(key.id, key.pack, key.filled),
        };
        #[cfg(not(feature = "cyberpunk"))]
        let glyph = dodai_icons::glyph(key.id, key.pack, key.filled);
        let k = f32::from_bits(key.size) / 24.0;
        let weight = f32::from_bits(key.weight);
        let outline = |polys: &mut dyn Iterator<Item = (&[(f32, f32)], bool)>| {
            let mut b = Path::builder();
            for (p, closed) in polys {
                let Some(((x, y), rest)) = p.split_first() else {
                    continue;
                };
                if !crate::paint::finite(p) {
                    continue;
                }
                b.begin(point(x * k, y * k));
                for (x, y) in rest {
                    b.line_to(point(x * k, y * k));
                }
                b.end(closed);
            }
            b.build()
        };
        let mut buf: VertexBuffers<lyon_tessellation::math::Point, u32> = VertexBuffers::new();
        match glyph? {
            Glyph::Stroked(shapes) => {
                let width = (weight * k).max(1.0);
                let path = outline(&mut shapes.iter().map(|(p, c)| (p.as_slice(), *c)));
                StrokeTessellator::new()
                    .tessellate_path(
                        &path,
                        &StrokeOptions::default().with_line_width(width),
                        &mut BuffersBuilder::new(&mut buf, |v: StrokeVertex| v.position()),
                    )
                    .ok()?;
            }
            Glyph::Filled(polys) => {
                let path = outline(&mut polys.iter().map(|p| (p.as_slice(), true)));
                FillTessellator::new()
                    .tessellate_path(
                        &path,
                        &FillOptions::default(),
                        &mut BuffersBuilder::new(&mut buf, |v: FillVertex| v.position()),
                    )
                    .ok()?;
            }
        }
        let at = |i: u32| buf.vertices.get(i as usize).map(|p| (p.x, p.y));
        Some(Self(
            buf.indices
                .chunks_exact(3)
                .filter_map(|t| Some([at(t[0])?, at(t[1])?, at(t[2])?]))
                .collect(),
        ))
    }

    fn paint(&self, w: &mut gpui::Window, o: gpui::Point<Pixels>, color: Hsla) {
        let Some(first) = self.0.first() else {
            return;
        };
        let at = |(x, y): (f32, f32)| gpui::point(o.x + px(x), o.y + px(y));
        let solid = gpui::point(0.0, 1.0);
        let mut path = gpui::Path::new(at(first[0]));
        for [a, b, c] in &self.0 {
            path.push_triangle((at(*a), at(*b), at(*c)), (solid, solid, solid));
        }
        w.paint_path(path, color);
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
    fn icon_meshes_are_triangles_inside_the_icon_box() {
        for (pack, filled) in [(Pack::Material, true), (Pack::Stroked, false)] {
            let size = 20.0f32;
            let key = MeshKey {
                id: IconId::CalendarHeart,
                pack,
                filled,
                #[cfg(feature = "cyberpunk")]
                modifier: None,
                size: size.to_bits(),
                weight: 1.5f32.to_bits(),
            };
            let mesh = Mesh::cached(key).expect("the icon exists in this pack");
            assert!(!mesh.0.is_empty(), "{pack:?} drew nothing");
            // Half a stroke may stick out past the drawing's box.
            let slack = 1.5;
            for (x, y) in mesh.0.iter().flatten() {
                assert!((-slack..=size + slack).contains(x), "{pack:?} x {x}");
                assert!((-slack..=size + slack).contains(y), "{pack:?} y {y}");
            }
            // Asked again, the same mesh comes back without tessellating.
            assert!(std::rc::Rc::ptr_eq(&mesh, &Mesh::cached(key).unwrap()));
        }
    }

    #[test]
    fn a_hand_drawn_name_is_the_same_icon_as_its_id() {
        assert_eq!(IconId::from(IconName::Heart), IconId::Heart);
    }
}
