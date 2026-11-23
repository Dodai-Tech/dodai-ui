//! Resolving a theme into the colours and timing a surface effect is painted with.

use crate::{Color, ResolvedSurface, SurfaceClass, Texture, Theme};
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

fn paint_with(
    theme: &Theme,
    mut fx: ResolvedSurface,
    pressable: bool,
    container: bool,
) -> SurfacePaint {
    let p = &theme.palette;
    // The layout (cards, bars) keeps the design's gradient; what sits on top of it (controls,
    // popups) is the blurred backdrop only, so its wash fades out as the frost comes in.
    let wash = if container {
        fx.wash
    } else {
        fx.wash * (1.0 - fx.frost.clamp(0.0, 1.0))
    };
    fx.wash = (wash * theme.surface_style.gradient).min(1.0);
    fx.rim = (fx.rim * theme.surface_style.gradient).min(1.0);
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
    active: ResolvedSurface,
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
        !class.takes_mix_frost(),
    ))
}

/// A preview of one `texture` (for pickers); `neon` is the app-wide lit-line amount.
pub fn swatch_paint(theme: &Theme, texture: Texture, neon: f32) -> SurfacePaint {
    let fx = ResolvedSurface {
        texture,
        strength: 0.8,
        scale: 1.0,
        neon,
        ..ResolvedSurface::NONE
    };
    paint_with(theme, fx, false, false)
}

/// One frame of the mix `active` (with a component's own `over` on top) for a `class` surface of
/// `w`×`h` px, as an SVG document: for toolkits that show images or CSS backgrounds rather than
/// draw (web, Slint). `seed` varies the random-looking details between surfaces, `time` picks
/// the moment of an animated effect. `None` when the design draws nothing of it. The second
/// value says the effect animates.
pub fn surface_svg(
    theme: &Theme,
    class: SurfaceClass,
    over: &Surface,
    active: ResolvedSurface,
    (w, h): (f32, f32),
    (seed, time): (i32, f32),
) -> Option<(String, bool)> {
    let paint = surface_paint(theme, class, over, active)?;
    let shape = theme.shape.clip();
    let frame = dodai_fx::Frame {
        w,
        h,
        shape: &shape,
        seed,
        time,
        presses: &[],
    };
    dodai_fx::surface_svg(&frame, &paint)
}

#[cfg(all(test, feature = "cyberpunk", feature = "material"))]
mod tests {
    use super::*;
    use crate::SurfaceTargets;

    /// The textures `t` draws (not the empty one).
    fn textured(t: &Theme) -> Vec<Texture> {
        t.surface_textures()
            .iter()
            .copied()
            .filter(|x| *x != Texture::None)
            .collect()
    }

    /// A mix of the first texture `t` owns.
    fn owned(t: &Theme) -> ResolvedSurface {
        ResolvedSurface {
            texture: textured(t)[0],
            strength: 0.8,
            targets: SurfaceTargets::ALL,
            ..ResolvedSurface::NONE
        }
    }

    #[test]
    fn a_surface_becomes_an_svg_document() {
        let t = crate::themes::cyberpunk::yellow();
        let (svg, animated) = surface_svg(
            &t,
            SurfaceClass::Button,
            &Surface::INHERIT,
            owned(&t),
            (160.0, 80.0),
            (1, 0.0),
        )
        .expect("cyberpunk draws its own texture");
        assert!(svg.contains("viewBox=\"0 0 160 80\"") && svg.contains("<rect "));
        assert!(!animated);
    }

    #[test]
    fn nothing_owned_or_nothing_set_is_none() {
        let t = crate::themes::cyberpunk::yellow();
        let size = (160.0, 80.0);
        let none = |t: &Theme, fx| {
            surface_svg(
                t,
                SurfaceClass::Button,
                &Surface::INHERIT,
                fx,
                size,
                (1, 0.0),
            )
            .is_none()
        };
        assert!(none(&t, ResolvedSurface::NONE));
        // A design draws only the textures it owns.
        let m = crate::themes::material::light();
        let foreign = textured(&t)
            .into_iter()
            .find(|x| !m.surface_textures().contains(x))
            .expect("the designs own different textures");
        let theirs = ResolvedSurface {
            texture: foreign,
            ..owned(&t)
        };
        assert!(none(&m, theirs));
    }
}
