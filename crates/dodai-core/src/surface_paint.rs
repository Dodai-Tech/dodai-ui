//! Resolving a theme into the colours and timing a surface effect is painted with.

use crate::{Color, SurfaceClass, SurfaceFx, Texture, Theme};
use dodai_fx::{Rgba, Surface, SurfacePaint};

impl From<Color> for Rgba {
    fn from(c: Color) -> Self {
        let f = |v: u8| f32::from(v) / 255.0;
        Self {
            r: f(c.r),
            g: f(c.g),
            b: f(c.b),
            a: f(c.a),
        }
    }
}

fn paint_with(theme: &Theme, fx: SurfaceFx, pressable: bool) -> SurfacePaint {
    let p = &theme.palette;
    SurfacePaint {
        fx,
        ink: p.text.into(),
        hole: p.background.into(),
        a: p.link.into(),
        b: p.danger.into(),
        light: p.text.into(),
        neon: p.accent.into(),
        shade: p.roles.shadow.into(),
        wash: theme.surface_style.wash.map(Rgba::from),
        tune: theme.surface_style.tuning,
        pressable,
        fill_lightness: 0.0,
    }
}

/// Resolve the paint for a `class` surface: `active` is the app-wide mix, `over` the component's
/// own [`Surface`] on top. `None` when nothing would be drawn.
pub fn surface_paint(
    theme: &Theme,
    class: SurfaceClass,
    over: &Surface,
    active: SurfaceFx,
) -> Option<SurfacePaint> {
    if over.is_off() {
        return None;
    }
    let owned = (theme.surface_textures(), theme.surface_effects());
    let base = active.limited(owned.0, owned.1).for_class(class);
    let fx = over.over(base).limited(owned.0, owned.1);
    if fx.is_none() || !(fx.targets.contains(class) || over.forces_on()) {
        return None;
    }
    Some(paint_with(
        theme,
        fx,
        matches!(class, SurfaceClass::Button | SurfaceClass::Tag),
    ))
}

/// A preview of one `texture` (for pickers); `neon` is the app-wide lit-line amount.
pub fn swatch_paint(theme: &Theme, texture: Texture, neon: f32) -> SurfacePaint {
    let fx = SurfaceFx {
        texture,
        strength: 0.8,
        scale: 1.0,
        neon,
        ..SurfaceFx::NONE
    };
    paint_with(theme, fx, false)
}
