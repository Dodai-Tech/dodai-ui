//! The [`Surface`] component: one themed box (fill + border + shape + glow).
//!
//! Widgets describe *what* a surface is (colours, glow wanted); this module
//! decides *how* it is drawn from the theme's [`ShapeKind`]:
//!
//! * `Square` / `Rounded` → native `BackgroundColor`/`BorderColor`/`BorderRadius`
//! * `Chamfered` → [`ChamferMaterial`] shader (same cuts as `Shape::outline`)
//! * glow → `BoxShadow` (rectangular halo) when `Effects::glow_enabled()`

use bevy::prelude::*;
use bevy::ui_render::prelude::MaterialNode;
use dodai_core::{Color as Rgba, Corners, ShapeKind, Theme};

use crate::material::ChamferMaterial;
use crate::theme::{color, DodaiTheme};

/// A themed box. Requires the native UI colour components it may drive.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
#[require(Node, BackgroundColor, BorderColor)]
pub struct Surface {
    /// `None` = transparent.
    pub fill: Option<Rgba>,
    /// `None` = no border.
    pub border: Option<Rgba>,
    /// Draw the theme glow around this surface.
    pub glow: bool,
    /// Fully round when the theme is `Rounded` (switch thumbs, radios).
    pub pill: bool,
}

/// How the surface geometry is painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// Native UI node with this corner radius (px, 0 = square).
    Native {
        /// Corner radius in px.
        radius: f32,
    },
    /// Shader-drawn cut corners.
    Chamfer {
        /// Cut size in px.
        chamfer: f32,
        /// Which corners are cut.
        corners: Corners,
    },
}

/// Glow halo parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlowSpec {
    /// Halo colour (alpha already scaled by intensity).
    pub color: Rgba,
    /// Blur radius px (already scaled by intensity).
    pub radius: f32,
}

/// A [`Surface`] resolved against a theme: pure data, no ECS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedSurface {
    /// Fill (transparent when none).
    pub fill: Rgba,
    /// Border (transparent when none).
    pub border: Rgba,
    /// Border width px (0 without border).
    pub border_width: f32,
    /// Geometry.
    pub paint: Paint,
    /// Glow, if enabled for this surface and theme.
    pub glow: Option<GlowSpec>,
}

/// Resolve a surface against a theme. This is the only place shape tokens are
/// interpreted; it never looks at theme ids.
pub fn resolve(theme: &Theme, s: &Surface) -> ResolvedSurface {
    let clear = Rgba::rgba(0, 0, 0, 0);
    let sh = &theme.shape;
    let paint = match sh.kind() {
        ShapeKind::Square => Paint::Native { radius: 0.0 },
        ShapeKind::Rounded => Paint::Native {
            radius: if s.pill { 9999.0 } else { sh.radius },
        },
        ShapeKind::Chamfered => Paint::Chamfer {
            chamfer: sh.chamfer,
            corners: sh.corners,
        },
    };
    let glow = (s.glow && theme.effects.glow_enabled()).then(|| GlowSpec {
        color: theme.effects.glow_color(),
        radius: theme.effects.glow_radius(),
    });
    ResolvedSurface {
        fill: s.fill.unwrap_or(clear),
        border: s.border.unwrap_or(clear),
        border_width: if s.border.is_some() {
            sh.border_width
        } else {
            0.0
        },
        paint,
        glow,
    }
}

/// Signed distance (px, positive inside) from `(x, y)` to the edge of a
/// `w`×`h` box with cuts of size `chamfer` at `corners`. Rust mirror of
/// `shaders/chamfer.wgsl`; used to test the shader maths against
/// `Shape::outline`.
pub fn chamfer_distance(x: f32, y: f32, w: f32, h: f32, chamfer: f32, corners: Corners) -> f32 {
    const K: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let c = chamfer.min(w.min(h) / 3.0);
    let mut d = x.min(w - x).min(y).min(h - y);
    if corners.top_left {
        d = d.min((x + y - c) * K);
    }
    if corners.top_right {
        d = d.min(((w - x) + y - c) * K);
    }
    if corners.bottom_right {
        d = d.min(((w - x) + (h - y) - c) * K);
    }
    if corners.bottom_left {
        d = d.min((x + (h - y) - c) * K);
    }
    d
}

/// Marker on entities that hold a chamfer material handle (managed here).
fn linear(c: Rgba) -> LinearRgba {
    color(c).to_linear()
}

/// Apply [`Surface`]s to the native components / material. Runs when the theme
/// changed or a surface changed.
#[allow(clippy::type_complexity)]
pub(crate) fn apply_surfaces(
    theme: Res<DodaiTheme>,
    mut materials: ResMut<Assets<ChamferMaterial>>,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        Ref<Surface>,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderColor,
        Option<&MaterialNode<ChamferMaterial>>,
        Option<&BoxShadow>,
    )>,
) {
    let theme_changed = theme.is_changed();
    for (e, s, mut node, mut bg, mut bc, mat, shadow) in &mut q {
        if !theme_changed && !s.is_changed() {
            continue;
        }
        let r = resolve(&theme, &s);
        let bw = Val::Px(r.border_width);
        let border = UiRect::all(bw);
        if node.border != border {
            node.border = border;
        }
        match r.paint {
            Paint::Native { radius } => {
                set(&mut bg.0, color(r.fill));
                let b = BorderColor::all(color(r.border));
                if *bc != b {
                    *bc = b;
                }
                let rad = BorderRadius::all(Val::Px(radius));
                upd(&mut node.border_radius, rad);
                if mat.is_some() {
                    commands.entity(e).remove::<MaterialNode<ChamferMaterial>>();
                }
            }
            Paint::Chamfer { chamfer, corners } => {
                set(&mut bg.0, Color::NONE);
                let b = BorderColor::all(Color::NONE);
                if *bc != b {
                    *bc = b;
                }
                upd(&mut node.border_radius, BorderRadius::ZERO);
                let m = ChamferMaterial {
                    fill: linear(r.fill),
                    border: linear(r.border),
                    params: Vec4::new(r.border_width, chamfer, 0.0, 0.0),
                    corners: Vec4::new(
                        corners.top_left as u8 as f32,
                        corners.top_right as u8 as f32,
                        corners.bottom_right as u8 as f32,
                        corners.bottom_left as u8 as f32,
                    ),
                };
                match mat.and_then(|h| materials.get_mut(&h.0)) {
                    Some(existing) => {
                        if *existing != m {
                            *existing = m;
                        }
                    }
                    None => {
                        commands.entity(e).insert(MaterialNode(materials.add(m)));
                    }
                }
            }
        }
        match (r.glow, shadow) {
            (Some(g), _) => {
                let sh = BoxShadow::new(
                    color(g.color),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(0.0),
                    Val::Px(g.radius),
                );
                if shadow != Some(&sh) {
                    commands.entity(e).insert(sh);
                }
            }
            (None, Some(_)) => {
                commands.entity(e).remove::<BoxShadow>();
            }
            (None, None) => {}
        }
    }
}

fn upd<T: PartialEq>(slot: &mut T, v: T) {
    if *slot != v {
        *slot = v;
    }
}

fn set(slot: &mut Color, v: Color) {
    if *slot != v {
        *slot = v;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{rounded_light, square_theme};
    use dodai::themes;

    fn point_in_poly(poly: &[(f32, f32)], x: f32, y: f32) -> bool {
        let mut inside = false;
        let mut j = poly.len() - 1;
        for i in 0..poly.len() {
            let (xi, yi) = poly[i];
            let (xj, yj) = poly[j];
            if (yi > y) != (yj > y) && x < (xj - xi) * (y - yi) / (yj - yi) + xi {
                inside = !inside;
            }
            j = i;
        }
        inside
    }

    #[test]
    fn paint_follows_shape_kind() {
        let s = Surface::default();
        for t in themes::all() {
            let p = resolve(&t, &s).paint;
            match t.shape.kind() {
                ShapeKind::Chamfered => assert!(matches!(p, Paint::Chamfer { .. }), "{}", t.id()),
                _ => assert!(matches!(p, Paint::Native { .. }), "{}", t.id()),
            }
        }
        assert_eq!(
            resolve(&rounded_light(), &s).paint,
            Paint::Native { radius: 8.0 }
        );
        assert_eq!(
            resolve(&square_theme(), &s).paint,
            Paint::Native { radius: 0.0 }
        );
    }

    #[test]
    fn pill_only_rounds_rounded_themes() {
        let s = Surface {
            pill: true,
            ..Default::default()
        };
        assert!(
            matches!(resolve(&rounded_light(), &s).paint, Paint::Native { radius } if radius > 100.0)
        );
        assert_eq!(
            resolve(&square_theme(), &s).paint,
            Paint::Native { radius: 0.0 }
        );
    }

    #[test]
    fn chamfer_distance_matches_outline() {
        for t in themes::all()
            .into_iter()
            .filter(|t| t.shape.kind() == ShapeKind::Chamfered)
        {
            let (w, h) = (120.0, 40.0);
            let poly = t.shape.outline(w, h);
            for xi in 0..=24 {
                for yi in 0..=8 {
                    let (x, y) = (xi as f32 * 5.0 + 0.3, yi as f32 * 5.0 + 0.3);
                    let d = chamfer_distance(x, y, w, h, t.shape.chamfer, t.shape.corners);
                    // Skip points within AA distance of the edge.
                    if d.abs() < 0.5 {
                        continue;
                    }
                    assert_eq!(d > 0.0, point_in_poly(&poly, x, y), "{} @ {x},{y}", t.id());
                }
            }
        }
    }

    #[test]
    fn glow_follows_effects_and_intensity() {
        let s = Surface {
            glow: true,
            ..Default::default()
        };
        assert!(resolve(&rounded_light(), &s).glow.is_none());
        for t in themes::all()
            .into_iter()
            .filter(|t| t.effects.glow_enabled())
        {
            let g1 = resolve(&t, &s).glow.unwrap();
            let g2 = resolve(&t.clone().with_glow_intensity(2.0), &s)
                .glow
                .unwrap();
            assert!(g2.radius > g1.radius);
            assert!(resolve(&t.clone().with_glow_intensity(0.0), &s)
                .glow
                .is_none());
            let off = Surface::default();
            assert!(resolve(&t, &off).glow.is_none());
        }
    }

    #[test]
    fn border_width_only_with_border() {
        let t = rounded_light();
        let none = resolve(&t, &Surface::default());
        assert_eq!(none.border_width, 0.0);
        let b = resolve(
            &t,
            &Surface {
                border: Some(t.palette.border),
                ..Default::default()
            },
        );
        assert_eq!(b.border_width, t.shape.border_width);
    }
}
