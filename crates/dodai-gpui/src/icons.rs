//! Vector [`Icon`]s drawn with gpui paths (no asset files, any colour/size).
//!
//! [`Icon`] follows the theme: chamfered/square themes get the stroked,
//! curve-free Cyberpunk pack; rounded themes (Material 3) get the matching
//! Material Symbols glyph (feature `material-icons`). Force a pack with
//! [`Icon::pack`]. [`MaterialIcon`] draws any catalogue icon by name.

use crate::paint::stroke;
pub use dodai_cyberpunk_icons::IconName;
use gpui::{canvas, div, px, Bounds, Hsla, IntoElement, ParentElement, Pixels, RenderOnce, Styled};

/// Which icon pack an [`Icon`] draws from.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconPack {
    /// Follow the theme: Material for rounded themes, stroked otherwise.
    #[default]
    Auto,
    /// The curve-free stroked Cyberpunk pack.
    Stroked,
    /// Material Symbols (needs feature `material-icons`; falls back to stroked).
    Material,
}

impl IconPack {
    /// Resolve `Auto` against the theme's shape kind.
    #[cfg_attr(not(feature = "material-icons"), allow(dead_code))]
    pub(crate) fn resolve(self, rounded: bool) -> Self {
        match self {
            Self::Auto if rounded && cfg!(feature = "material-icons") => Self::Material,
            Self::Auto => Self::Stroked,
            Self::Material if !cfg!(feature = "material-icons") => Self::Stroked,
            p => p,
        }
    }
}

/// Vector icon. Defaults: 16 px, white-ish — set `.color(..)`.
#[derive(IntoElement)]
pub struct Icon {
    name: IconName,
    size: f32,
    color: Hsla,
    weight: f32,
    pack: IconPack,
    filled: bool,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: 16.0,
            color: gpui::white(),
            weight: 1.75,
            pack: IconPack::Auto,
            filled: false,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = c;
        self
    }
    /// Stroke width in px at 24 px size (scales with `size`; stroked pack only).
    pub fn weight(mut self, w: f32) -> Self {
        self.weight = w;
        self
    }
    /// Force an icon pack instead of following the theme.
    pub fn pack(mut self, pack: IconPack) -> Self {
        self.pack = pack;
        self
    }
    /// Use the filled variant (Material pack only).
    pub fn filled(mut self, on: bool) -> Self {
        self.filled = on;
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let (size, color, weight) = (self.size, self.color, self.weight);
        let rounded = crate::theme_of(cx).shape.kind() == dodai_core::ShapeKind::Rounded;
        #[cfg(feature = "material-icons")]
        if self.pack.resolve(rounded) == IconPack::Material {
            let name = dodai_material_icons::resolve(self.name.name());
            if let Some(m) = material(name, self.filled) {
                return glyph(size, color, m, false).into_any_element();
            }
        }
        let _ = rounded;
        let shapes = self.name.shapes();
        let k = size / 24.0;
        div()
            .size(px(size))
            .flex_none()
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
            .into_any_element()
    }
}

/// Catalogue lookup: `name` or its `-fill` variant (falls back to outline).
#[cfg(feature = "material-icons")]
fn material(name: &str, filled: bool) -> Option<&'static dodai_material_icons::Icon> {
    use dodai_material_icons::outlined;
    if filled {
        if let Some(i) = outlined::get(&format!("{name}-fill")) {
            return Some(i);
        }
    }
    outlined::get(name)
}

/// A filled glyph: all subpaths in one path, non-zero rule (holes wind opposite).
#[cfg(feature = "material-icons")]
fn glyph(
    size: f32,
    color: Hsla,
    icon: &'static dodai_material_icons::Icon,
    faceted: bool,
) -> impl IntoElement {
    let src = if faceted {
        icon.faceted()
    } else {
        icon.polygons()
    };
    let polys: Vec<Vec<(f32, f32)>> = src
        .into_iter()
        .map(|p| {
            p.into_iter()
                .map(|(x, y)| (x * size / 24.0, y * size / 24.0))
                .collect()
        })
        .collect();
    div().size(px(size)).flex_none().child(
        canvas(
            |_, _, _| (),
            move |b: Bounds<Pixels>, _, w, _| {
                crate::paint::fill_batch(w, b.origin, &polys, color);
            },
        )
        .size_full(),
    )
}

/// Any Material Symbols icon by catalogue name (`"home"`, `"arrow_back"`, ...).
/// Unknown names draw nothing.
#[cfg(feature = "material-icons")]
#[derive(IntoElement)]
pub struct MaterialIcon {
    name: String,
    size: f32,
    color: Hsla,
    filled: bool,
    faceted: Option<bool>,
}

#[cfg(feature = "material-icons")]
impl MaterialIcon {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 24.0,
            color: gpui::white(),
            filled: false,
            faceted: None,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = c;
        self
    }
    /// Reduce the glyph to straight facets (`true`). Opt-in: the default is
    /// the smooth Material drawing in every theme.
    pub fn faceted(mut self, on: bool) -> Self {
        self.faceted = Some(on);
        self
    }
    /// Use the `-fill` variant when the catalogue has one.
    pub fn filled(mut self, on: bool) -> Self {
        self.filled = on;
        self
    }
}

#[cfg(feature = "material-icons")]
impl RenderOnce for MaterialIcon {
    fn render(self, _: &mut gpui::Window, _: &mut gpui::App) -> impl IntoElement {
        let name = dodai_material_icons::resolve(&self.name);
        let faceted = self.faceted.unwrap_or(false);
        match material(name, self.filled) {
            Some(m) => glyph(self.size, self.color, m, faceted).into_any_element(),
            None => div().size(px(self.size)).flex_none().into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_pack_follows_the_shape() {
        let mat = cfg!(feature = "material-icons");
        assert_eq!(
            IconPack::Auto.resolve(true),
            if mat {
                IconPack::Material
            } else {
                IconPack::Stroked
            }
        );
        assert_eq!(IconPack::Auto.resolve(false), IconPack::Stroked);
        assert_eq!(IconPack::Stroked.resolve(true), IconPack::Stroked);
    }

    #[cfg(feature = "material-icons")]
    #[test]
    fn every_stroked_icon_has_a_material_counterpart() {
        for &i in dodai_cyberpunk_icons::ALL {
            let n = dodai_material_icons::resolve(i.name());
            assert!(material(n, false).is_some(), "{} -> {n}", i.name());
            assert!(material(n, true).is_some(), "{} -> {n}-fill", i.name());
        }
    }
}
