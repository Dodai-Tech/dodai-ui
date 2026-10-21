//! Vector [`Icon`]s drawn with gpui paths (no asset files, any colour/size).
//!
//! [`Icon`] follows the theme: chamfered/square themes get the stroked,
//! curve-free Cyberpunk pack; rounded themes (Material 3) get the matching
//! Material Symbols glyph (feature `material`). The stroked pack is feature `cyberpunk`; with it
//! off, every icon is drawn by the Material pack. Force a pack with
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
    /// Material Symbols (needs feature `material`; falls back to stroked).
    Material,
    /// The heavy Neo-Brutalism pack (needs feature `neobrutalism`; falls back to stroked).
    Bold,
}

impl IconPack {
    /// Resolve `Auto` against the theme's shape kind.
    #[cfg_attr(not(feature = "material"), allow(dead_code))]
    pub(crate) fn resolve(self, set: dodai_core::IconSet) -> Self {
        let (mat, stroked) = (cfg!(feature = "material"), cfg!(feature = "cyberpunk"));
        let bold = cfg!(feature = "neobrutalism");
        match self {
            Self::Auto if set == dodai_core::IconSet::Bold && bold => Self::Bold,
            Self::Bold if !bold => Self::Stroked,
            Self::Auto if set == dodai_core::IconSet::Symbols && mat => Self::Material,
            // The stroked pack is not compiled in: the only pack left is Material.
            Self::Auto | Self::Stroked if !stroked && mat => Self::Material,
            Self::Auto => Self::Stroked,
            Self::Material if !mat => Self::Stroked,
            p => p,
        }
    }
}

/// Vector icon. Defaults: 16 px, the theme text colour; set `.color(..)` to override.
#[cfg_attr(
    not(all(feature = "cyberpunk", feature = "material")),
    allow(dead_code)
)]
#[derive(IntoElement)]
pub struct Icon {
    name: Option<IconName>,
    /// Shapes resolved by name ([`Icon::named`]), for variants.
    custom: Option<Vec<dodai_cyberpunk_icons::Shape>>,
    /// The name given to [`Icon::named`], for the Material pack.
    label: Option<String>,
    size: f32,
    color: Option<Hsla>,
    weight: f32,
    pack: IconPack,
    filled: bool,
}

impl Icon {
    pub fn new(name: IconName) -> Self {
        Self::base(Some(name), None)
    }

    /// An icon by name, including derived variants: `"heart-add"`,
    /// `"wifi_off"`, a Material spelling like `"arrow_back"`. Unknown names
    /// draw nothing. Always the stroked Cyberpunk drawing.
    pub fn named(name: &str) -> Self {
        #[cfg(all(feature = "cyberpunk", feature = "material"))]
        let shapes = dodai_cyberpunk_icons::lookup_with(name, &dodai_material_icons::alias_of)
            .unwrap_or_default();
        #[cfg(all(feature = "cyberpunk", not(feature = "material")))]
        let shapes = dodai_cyberpunk_icons::lookup(name).unwrap_or_default();
        #[cfg(not(feature = "cyberpunk"))]
        let shapes: Vec<dodai_cyberpunk_icons::Shape> = Vec::new();
        let mut icon = Self::base(None, Some(shapes)).pack(IconPack::Stroked);
        icon.label = Some(name.to_string());
        icon
    }

    fn base(name: Option<IconName>, custom: Option<Vec<dodai_cyberpunk_icons::Shape>>) -> Self {
        Self {
            name,
            custom,
            label: None,
            size: 16.0,
            color: None,
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
        self.color = Some(c);
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
        let theme = crate::theme_of(cx);
        let color = self
            .color
            .unwrap_or_else(|| crate::hsla(theme.palette.text));
        let (size, weight) = (self.size, self.weight);
        let set = theme.look().icons;
        let pack = self.pack.resolve(set);
        #[cfg(feature = "neobrutalism")]
        if pack == IconPack::Bold {
            let wanted = self.name.map(|n| n.name().to_string()).or(self.label);
            if let Some(w) = wanted {
                let alt = w.replace('_', "-");
                let hit = |n: &str| {
                    (self.filled)
                        .then(|| dodai_neobrutalism_icons::get(&format!("{n}-fill")))
                        .flatten()
                        .or_else(|| dodai_neobrutalism_icons::get(n))
                };
                if let Some(m) = hit(&w).or_else(|| hit(&alt)) {
                    return glyph(size, color, m, false).into_any_element();
                }
            }
            // A name the pack lacks draws nothing: no other pack stands in for it.
            return div().size(px(size)).flex_none().into_any_element();
        }
        #[cfg(feature = "material")]
        if pack == IconPack::Material {
            let wanted = self.name.map(|n| n.name().to_string()).or(self.label);
            if let Some(w) = wanted {
                if let Some(m) = material(dodai_material_icons::resolve(&w), self.filled) {
                    return glyph(size, color, m, false).into_any_element();
                }
                // Not in Material Symbols: the shared Phosphor vocabulary, drawn as is.
                if let Some(m) = phosphor(&w, self.filled) {
                    return glyph(size, color, m, false).into_any_element();
                }
            }
            return div().size(px(size)).flex_none().into_any_element();
        }
        #[cfg(feature = "cyberpunk")]
        let shapes = self
            .custom
            .unwrap_or_else(|| self.name.map(|n| n.shapes()).unwrap_or_default());
        // No hand-drawn icon: the shared Phosphor vocabulary, faceted into the angular style.
        #[cfg(feature = "cyberpunk")]
        if shapes.is_empty() {
            if let Some(m) = self.label.as_deref().and_then(|w| phosphor(w, self.filled)) {
                return glyph(size, color, m, true).into_any_element();
            }
        }
        #[cfg(not(feature = "cyberpunk"))]
        let shapes: Vec<dodai_cyberpunk_icons::Shape> = Vec::new();
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
#[cfg(feature = "material")]
fn material(name: &str, filled: bool) -> Option<&'static dodai_material_icons::Icon> {
    use dodai_material_icons::outlined;
    if filled {
        if let Some(i) = outlined::get(&format!("{name}-fill")) {
            return Some(i);
        }
    }
    outlined::get(name)
}

/// A Phosphor icon by its own name (`"acorn"`, kebab or snake case).
fn phosphor(name: &str, filled: bool) -> Option<&'static dodai_phosphor_icons::Icon> {
    let n = name.replace('_', "-");
    filled
        .then(|| dodai_phosphor_icons::get(&format!("{n}-fill")))
        .flatten()
        .or_else(|| dodai_phosphor_icons::get(&n))
}

/// A filled glyph: all subpaths in one path, non-zero rule (holes wind opposite).
#[allow(dead_code)]
fn glyph(
    size: f32,
    color: Hsla,
    icon: &'static dodai_phosphor_icons::Icon,
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
#[cfg(feature = "material")]
#[derive(IntoElement)]
pub struct MaterialIcon {
    name: String,
    size: f32,
    color: Option<Hsla>,
    filled: bool,
    faceted: Option<bool>,
}

#[cfg(feature = "material")]
impl MaterialIcon {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            size: 24.0,
            color: None,
            filled: false,
            faceted: None,
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
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

#[cfg(feature = "material")]
impl RenderOnce for MaterialIcon {
    fn render(self, _: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let name = dodai_material_icons::resolve(&self.name);
        let faceted = self.faceted.unwrap_or(false);
        let color = self
            .color
            .unwrap_or_else(|| crate::hsla(crate::theme_of(cx).palette.text));
        match material(name, self.filled) {
            Some(m) => glyph(self.size, color, m, faceted).into_any_element(),
            None => div().size(px(self.size)).flex_none().into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_pack_follows_the_shape() {
        let (mat, stroked) = (cfg!(feature = "material"), cfg!(feature = "cyberpunk"));
        let want = |m: bool| {
            if m {
                IconPack::Material
            } else {
                IconPack::Stroked
            }
        };
        use dodai_core::IconSet::{Bold, Stroked, Symbols};
        assert_eq!(IconPack::Auto.resolve(Symbols), want(mat));
        assert_eq!(IconPack::Auto.resolve(Stroked), want(mat && !stroked));
        assert_eq!(IconPack::Stroked.resolve(Symbols), want(mat && !stroked));
        // A design that names the bold pack never gets another pack's glyphs.
        let bold = if cfg!(feature = "neobrutalism") {
            IconPack::Bold
        } else {
            want(mat && !stroked)
        };
        assert_eq!(IconPack::Auto.resolve(Bold), bold);
    }

    #[cfg(feature = "material")]
    #[test]
    fn every_stroked_icon_has_a_material_counterpart() {
        for &i in dodai_cyberpunk_icons::ALL {
            let n = dodai_material_icons::resolve(i.name());
            assert!(material(n, false).is_some(), "{} -> {n}", i.name());
            assert!(material(n, true).is_some(), "{} -> {n}-fill", i.name());
        }
    }
}
