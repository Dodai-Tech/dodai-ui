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

/// How a button is cropped on this surface (see [`dodai_core::Theme::button_crop`]).
///
/// A cut corner draws through the chamfer shader, a ticket stub as a concave bite, tight
/// corners through per-corner radii. Ticks are drawn by the button's own children.
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct SurfaceCrop(pub Option<dodai_core::ButtonCrop>);

/// How the surface geometry is painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Paint {
    /// Native UI node with this corner radius (px, 0 = square).
    Native {
        /// Corner radius in px.
        radius: f32,
    },
    /// Native UI node with its own radius per corner `[top-left, top-right, bottom-right,
    /// bottom-left]` (px).
    NativeRadii {
        /// Corner radii in px.
        radii: [f32; 4],
    },
    /// Shader-drawn cut corners.
    Chamfer {
        /// Cut size in px.
        chamfer: f32,
        /// Which corners are cut.
        corners: Corners,
        /// Concave bites instead of straight cuts.
        scoop: bool,
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
    resolve_cropped(theme, s, None)
}

/// [`resolve`] for a surface whose end is cropped.
pub fn resolve_cropped(
    theme: &Theme,
    s: &Surface,
    crop: Option<dodai_core::ButtonCrop>,
) -> ResolvedSurface {
    use dodai_core::ButtonCrop as B;
    let sh = &theme.shape;
    let paint = match crop {
        Some(B::CutTicked { corners, .. }) => Paint::Chamfer {
            // Steep 45° cut: the largest the shape allows.
            chamfer: sh.chamfer.max(sh.control_height),
            corners,
            scoop: false,
        },
        Some(B::Scooped { corners, size }) => Paint::Chamfer {
            chamfer: size,
            corners,
            scoop: true,
        },
        Some(B::Squared {
            corners,
            radius,
            rest,
        }) => {
            let pick = |on: bool| if on { radius } else { rest };
            Paint::NativeRadii {
                radii: [
                    pick(corners.top_left),
                    pick(corners.top_right),
                    pick(corners.bottom_right),
                    pick(corners.bottom_left),
                ],
            }
        }
        Some(B::Slot { .. }) | None => base_paint(theme, s),
    };
    finish(theme, s, paint)
}

fn base_paint(theme: &Theme, s: &Surface) -> Paint {
    let sh = &theme.shape;
    match sh.kind() {
        ShapeKind::Square => Paint::Native { radius: 0.0 },
        ShapeKind::Rounded => Paint::Native {
            radius: if s.pill { 9999.0 } else { sh.radius },
        },
        ShapeKind::Chamfered => Paint::Chamfer {
            chamfer: sh.chamfer,
            corners: sh.corners,
            scoop: false,
        },
    }
}

fn finish(theme: &Theme, s: &Surface, paint: Paint) -> ResolvedSurface {
    let clear = Rgba::rgba(0, 0, 0, 0);
    let sh = &theme.shape;
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
/// `shaders/chamfer.wesl`; used to test the shader maths against
/// `Shape::outline`.
pub fn chamfer_distance(x: f32, y: f32, w: f32, h: f32, chamfer: f32, corners: Corners) -> f32 {
    cut_distance(x, y, w, h, chamfer, corners, false)
}

/// [`chamfer_distance`] with the corners cut straight or, with `scoop`, bitten out concavely.
pub fn cut_distance(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    chamfer: f32,
    corners: Corners,
    scoop: bool,
) -> f32 {
    const K: f32 = std::f32::consts::FRAC_1_SQRT_2;
    let c = chamfer.min(w.min(h) / 3.0);
    let mut d = x.min(w - x).min(y).min(h - y);
    if scoop {
        let bite = |on: bool, cx: f32, cy: f32| {
            if on {
                (x - cx).hypot(y - cy) - c
            } else {
                f32::MAX
            }
        };
        return d
            .min(bite(corners.top_left, 0.0, 0.0))
            .min(bite(corners.top_right, w, 0.0))
            .min(bite(corners.bottom_right, w, h))
            .min(bite(corners.bottom_left, 0.0, h));
    }
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
    glass: Res<crate::glass::DodaiGlass>,
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
        Option<Ref<SurfaceCrop>>,
        Option<Ref<crate::glass::UiGlass>>,
    )>,
) {
    let theme_changed = theme.is_changed() || glass.is_changed();
    for (e, s, mut node, mut bg, mut bc, mat, shadow, crop, ug) in &mut q {
        let crop_changed = crop.as_ref().is_some_and(|c| c.is_changed());
        let glass_changed = ug.as_ref().is_some_and(|g| g.is_changed());
        if !theme_changed && !s.is_changed() && !crop_changed && !glass_changed {
            continue;
        }
        let mut r = resolve_cropped(&theme, &s, crop.and_then(|c| c.0));
        if let Some(ug) = ug {
            // See-through fill; a frosted one is painted by its pane (fill over the blur).
            let g = glass.glass(&theme, &ug);
            r.fill = if g.frosted() && glass.has_backdrop() {
                r.fill.with_alpha(0.0)
            } else {
                g.color(r.fill)
            };
        }
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
                    commands
                        .entity(e)
                        .try_remove::<MaterialNode<ChamferMaterial>>();
                }
            }
            Paint::NativeRadii { radii } => {
                set(&mut bg.0, color(r.fill));
                let b = BorderColor::all(color(r.border));
                if *bc != b {
                    *bc = b;
                }
                let px = |i: usize| Val::Px(radii[i]);
                upd(
                    &mut node.border_radius,
                    BorderRadius {
                        top_left: px(0).into(),
                        top_right: px(1).into(),
                        bottom_right: px(2).into(),
                        bottom_left: px(3).into(),
                    },
                );
                if mat.is_some() {
                    commands
                        .entity(e)
                        .try_remove::<MaterialNode<ChamferMaterial>>();
                }
            }
            Paint::Chamfer {
                chamfer,
                corners,
                scoop,
            } => {
                set(&mut bg.0, Color::NONE);
                let b = BorderColor::all(Color::NONE);
                if *bc != b {
                    *bc = b;
                }
                upd(&mut node.border_radius, BorderRadius::ZERO);
                let m = ChamferMaterial {
                    fill: linear(r.fill),
                    border: linear(r.border),
                    params: Vec4::new(r.border_width, chamfer, scoop as u8 as f32, 0.0),
                    corners: Vec4::new(
                        corners.top_left as u8 as f32,
                        corners.top_right as u8 as f32,
                        corners.bottom_right as u8 as f32,
                        corners.bottom_left as u8 as f32,
                    ),
                };
                let found = mat.is_some_and(|h| {
                    materials.get_mut(&h.0).is_some_and(|mut existing| {
                        if *existing != m {
                            *existing = m.clone();
                        }
                        true
                    })
                });
                if !found {
                    commands
                        .entity(e)
                        .try_insert(MaterialNode(materials.add(m)));
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
                    commands.entity(e).try_insert(sh);
                }
            }
            (None, Some(_)) => {
                commands.entity(e).try_remove::<BoxShadow>();
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
    use dodai_core::themes;

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
    fn a_scooped_corner_is_bitten_out_and_the_rest_stays() {
        let (w, h, c) = (100.0, 40.0, 8.0);
        let left = Corners::LEFT;
        // Inside the bite at the top-left corner: outside the shape.
        assert!(cut_distance(2.0, 2.0, w, h, c, left, true) < 0.0);
        // Same spot on the uncut right end stays inside.
        assert!(cut_distance(w - 2.0, 2.0, w, h, c, left, true) > 0.0);
        // Just past the bite radius: inside.
        assert!(cut_distance(9.0, 1.0, w, h, c, left, true) > 0.0);
        // Centre stays at its distance to the nearest edge.
        assert_eq!(cut_distance(50.0, 20.0, w, h, c, left, true), 20.0);
    }

    #[test]
    fn crops_pick_their_paint() {
        use dodai_core::{ButtonCrop as B, CropEdge};
        let t = themes::all()
            .into_iter()
            .find(|t| t.shape.kind() == ShapeKind::Chamfered)
            .unwrap();
        let s = Surface::default();
        let cut = B::CutTicked {
            corners: Corners::TOP_LEFT,
            tick: CropEdge::Right,
        };
        match resolve_cropped(&t, &s, Some(cut)).paint {
            Paint::Chamfer {
                corners,
                scoop,
                chamfer,
            } => {
                assert_eq!(corners, Corners::TOP_LEFT);
                assert!(!scoop);
                assert!(chamfer >= t.shape.control_height);
            }
            p => panic!("{p:?}"),
        }
        let stub = B::Scooped {
            corners: Corners::RIGHT,
            size: 6.0,
        };
        assert!(matches!(
            resolve_cropped(&t, &s, Some(stub)).paint,
            Paint::Chamfer { scoop: true, .. }
        ));
        let tight = B::Squared {
            corners: Corners::LEFT,
            radius: 4.0,
            rest: 9999.0,
        };
        assert_eq!(
            resolve_cropped(&t, &s, Some(tight)).paint,
            Paint::NativeRadii {
                radii: [4.0, 9999.0, 9999.0, 4.0]
            }
        );
        assert_eq!(resolve_cropped(&t, &s, None), resolve(&t, &s));
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
