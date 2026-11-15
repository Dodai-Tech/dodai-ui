//! See-through fills: frosted glass (from the surface mix) and a component's own fill opacity,
//! resolved once so every toolkit makes a fill exactly as transparent.

use crate::{SurfaceClass, SurfaceFx, Theme};
use dodai_fx::Surface;

/// How see-through one surface's fill is, and the frosted glass under it.
///
/// Apply it to a fill alpha with [`alpha`](Self::alpha); when [`frosted`](Self::frosted), paint a
/// blurred copy of the backdrop (`blur` px, `saturation`) at `amount` opacity under the fill.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Glass {
    /// Frost amount, `0.0..=1.0` (`0.0` = no glass).
    pub amount: f32,
    /// The alpha a fully frosted fill drops to: the design's glass tint alpha.
    pub clear: f32,
    /// How much of its own alpha a frosted fill keeps on top of `clear` (`0.0` for containers;
    /// `0.5` for controls, so their labels keep the contrast of their fill).
    pub keep: f32,
    /// The component's own fill opacity (`1.0` = as designed).
    pub opacity: f32,
    /// Blur of the backdrop under the glass, px.
    pub blur: f32,
    /// Saturation of the blurred backdrop (`1.0` = unchanged).
    pub saturation: f32,
}

impl Glass {
    /// An opaque fill with no glass.
    pub const SOLID: Self = Self {
        amount: 0.0,
        clear: 1.0,
        keep: 0.0,
        opacity: 1.0,
        blur: 0.0,
        saturation: 1.0,
    };

    /// The glass of a `class` surface of `theme` at frost `amount`, fill `opacity`.
    pub fn new(theme: &Theme, class: SurfaceClass, amount: f32, opacity: f32) -> Self {
        let f = theme.frost();
        Self {
            amount: amount.clamp(0.0, 1.0),
            clear: f32::from(f.tint.a) / 255.0,
            keep: if class.is_container() { 0.0 } else { 0.5 },
            opacity: opacity.clamp(0.0, 1.0),
            blur: f.blur,
            saturation: f.saturation,
        }
    }

    /// Whether a blurred backdrop goes under the fill.
    pub fn frosted(&self) -> bool {
        self.amount > 0.0
    }

    /// Whether the fill is drawn as designed (no glass, full opacity).
    pub fn is_solid(&self) -> bool {
        !self.frosted() && self.opacity >= 1.0
    }

    /// The alpha a fill of alpha `a` is drawn with.
    pub fn alpha(&self, a: f32) -> f32 {
        let clear = self.clear + (a - self.clear) * self.keep;
        (a + (clear.min(a) - a) * self.amount) * self.opacity
    }

    /// `c` with its alpha made see-through by [`alpha`](Self::alpha).
    pub fn color(&self, c: crate::Color) -> crate::Color {
        if self.is_solid() {
            return c;
        }
        c.with_alpha(self.alpha(f32::from(c.a) / 255.0))
    }
}

/// The glass of a `class` surface: frost from the app-wide mix `active` with the component's own
/// `over` on top (only where the mix reaches it), opacity from `over`.
pub fn surface_glass(
    theme: &Theme,
    class: SurfaceClass,
    over: &Surface,
    active: SurfaceFx,
) -> Glass {
    let amount = crate::surface_paint(theme, class, over, active).map_or(0.0, |p| p.fx.frost);
    Glass::new(theme, class, amount, over.opacity_value().unwrap_or(1.0))
}

/// Size of the canvas mesh picture toolkits draw stretched over the window (wider than tall;
/// the blobs are round at this aspect).
pub const MESH_SIZE: (usize, usize) = (96, 60);

/// Blob centres `(x, y, radius)` in unit coordinates, one per wash colour, plus a faint fourth.
const BLOBS: [(f32, f32, f32); 4] = [
    (0.10, 0.05, 0.70),
    (0.92, 0.30, 0.65),
    (0.45, 1.00, 0.75),
    (0.60, 0.10, 0.40),
];

/// A showcase picture with hard edges (so the blur is visible), in the theme's colours: a dusk
/// sky with a low sun, stars and three ridges of mountains, each layer darker than the last.
pub fn frost_scene(p: &crate::Palette, w: usize, h: usize) -> Vec<u8> {
    let (top, horizon) = (p.primary.mix(p.text, 0.25), p.warning.mix(p.danger, 0.3));
    let sun = p.warning.mix(p.surface_raised, 0.6);
    // Far to near: lighter and cooler first, darkest last.
    let ridges = [
        (0.56, 0.12, 1.7, 0.4, p.info.mix(p.primary, 0.4)),
        (0.68, 0.14, 2.6, 2.1, p.primary.mix(p.success, 0.35)),
        (0.84, 0.12, 3.9, 4.0, p.success.mix(p.text, 0.55)),
    ];
    let mut out = vec![0u8; w * h * 4];
    let (fw, fh) = (w as f32, h as f32);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / fw, y as f32 / fh);
            // Sky: a vertical blend, pulled toward the horizon.
            let mut c = top.mix(horizon, v.powf(1.4));
            // Hard-edged sun and a few stars (a hash picks the cells that hold one).
            if ((u - 0.5) * fw / fh).hypot(v - 0.42) < 0.13 {
                c = sun;
            }
            let (cx, cy) = (x / 14, y / 14);
            let hash = (cx * 73_856_093) ^ (cy * 19_349_663);
            if v < 0.4 && hash % 17 == 0 && (x % 14).abs_diff(7) < 2 && (y % 14).abs_diff(7) < 2 {
                c = p.surface_raised;
            }
            // Ridges: sharp peaks from folded sines.
            for (base, amp, freq, phase, col) in ridges {
                let peak = 1.0 - (u * freq * std::f32::consts::TAU + phase).sin().abs();
                let wobble = (u * freq * 3.1 + phase).sin() * 0.25 + 0.75;
                if v > base - amp * peak * wobble {
                    c = col;
                }
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    out
}

/// The canvas mesh of a design whose `surface_style.backdrop` is above zero, as `w`×`h` RGBA
/// pixels: the canvas with the three wash colours (and the second again, fainter)
/// blended in with a smooth falloff of `strength`. Tiny and smooth: scaled up, it is its own
/// blur, so translucent surfaces over it need no further blurring.
pub fn mesh_pixels(
    canvas: crate::Color,
    wash: [crate::Color; 3],
    strength: f32,
    w: usize,
    h: usize,
) -> Vec<u8> {
    let blobs = mesh_blobs(wash, strength);
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let mut c = canvas;
            for b in &blobs {
                // The picture is wider than tall: keep the blobs round.
                let d = ((u - b.x) * (w as f32 / h as f32)).hypot(v - b.y) / b.r;
                c = c.mix(b.color, b.alpha_at(d));
            }
            let o = (y * w + x) * 4;
            out[o..o + 4].copy_from_slice(&[c.r, c.g, c.b, 255]);
        }
    }
    out
}

/// One soft blob of the canvas mesh, laid over the canvas in order.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshBlob {
    /// Centre in unit coordinates of the picture.
    pub x: f32,
    pub y: f32,
    /// Radius in picture heights.
    pub r: f32,
    pub color: crate::Color,
    /// Opacity at the centre.
    pub weight: f32,
}

impl MeshBlob {
    /// Opacity at distance `d` from the centre, in radii (smooth falloff, 0 from 1 on).
    pub fn alpha_at(&self, d: f32) -> f32 {
        let f = (1.0 - d).clamp(0.0, 1.0);
        (f * f * (3.0 - 2.0 * f) * self.weight).min(1.0)
    }
}

/// The blobs of a mesh of the three `wash` colours (and the second again, fainter) at
/// `strength`, bottom first.
pub fn mesh_blobs(wash: [crate::Color; 3], strength: f32) -> [MeshBlob; 4] {
    let colours = [wash[0], wash[1], wash[2], wash[1]];
    std::array::from_fn(|i| {
        let (x, y, r) = BLOBS[i];
        MeshBlob {
            x,
            y,
            r,
            color: colours[i],
            weight: strength * if i == 3 { 0.5 } else { 1.0 },
        }
    })
}

/// [`mesh_pixels`] of `theme`'s canvas, `None` for a design with a flat canvas.
pub fn backdrop_mesh(theme: &Theme, w: usize, h: usize) -> Option<Vec<u8>> {
    let style = &theme.surface_style;
    (style.backdrop > 0.0)
        .then(|| mesh_pixels(theme.palette.background, style.wash, style.backdrop, w, h))
}

/// A picture with hard edges in the palette's colours to put behind frosted glass, so the blur
/// shows (`w`×`h` RGBA): bands of the
/// accents across the canvas and a few discs.
pub fn glass_picture(p: &crate::Palette, w: usize, h: usize) -> Vec<u8> {
    let bands = [p.primary, p.info, p.success, p.warning, p.danger];
    let discs = [
        (0.2, 0.3, 0.12, p.warning),
        (0.7, 0.6, 0.18, p.primary),
        (0.45, 0.85, 0.1, p.success),
    ];
    let mut out = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        for x in 0..w {
            let (u, v) = (x as f32 / w as f32, y as f32 / h as f32);
            let band = ((u + v * 0.6) * 6.0) as usize % bands.len();
            let mut c = bands[band].mix(p.background, 0.35);
            for (cx, cy, r, colour) in discs {
                if ((u - cx) * w as f32 / h as f32).hypot(v - cy) < r {
                    c = colour;
                }
            }
            out.extend([c.r, c.g, c.b, 255]);
        }
    }
    out
}

#[cfg(all(test, feature = "dodai", feature = "material"))]
mod tests {
    use super::*;

    fn theme(id: &str) -> Theme {
        crate::themes::all()
            .into_iter()
            .find(|t| t.id() == id)
            .expect("known theme")
    }

    #[test]
    fn no_frost_and_full_opacity_keep_the_fill() {
        let t = theme("material-dark");
        let g = surface_glass(&t, SurfaceClass::Card, &Surface::INHERIT, SurfaceFx::NONE);
        assert!(g.is_solid());
        assert_eq!(g.alpha(1.0), 1.0);
        assert_eq!(g.color(t.palette.surface), t.palette.surface);
    }

    #[test]
    fn frost_drops_containers_to_the_glass_and_fields_halfway() {
        let t = theme("material-dark");
        let fx = SurfaceFx {
            frost: 1.0,
            targets: crate::SurfaceTargets::ALL,
            ..SurfaceFx::NONE
        };
        let clear = f32::from(t.frost().tint.a) / 255.0;
        let card = surface_glass(&t, SurfaceClass::Card, &Surface::INHERIT, fx);
        assert!(card.frosted());
        assert!((card.alpha(1.0) - clear).abs() < 1e-5);
        // The mix frosts containers and inputs; a button frosts itself.
        let field = surface_glass(&t, SurfaceClass::Input, &Surface::INHERIT, fx);
        assert!(field.frosted());
        let mixed = surface_glass(&t, SurfaceClass::Button, &Surface::INHERIT, fx);
        assert!(!mixed.frosted());
        let own = surface_glass(
            &t,
            SurfaceClass::Button,
            &Surface::new().frost(1.0),
            SurfaceFx::NONE,
        );
        assert!((own.alpha(1.0) - (clear + 1.0) / 2.0).abs() < 1e-5);
        // A fill already clearer than the glass stays as it is.
        assert!((card.alpha(clear / 2.0) - clear / 2.0).abs() < 1e-5);
    }

    #[test]
    fn opacity_works_without_the_mix_and_on_any_design() {
        for t in crate::themes::all() {
            let g = surface_glass(
                &t,
                SurfaceClass::Button,
                &Surface::new().opacity(0.5),
                SurfaceFx::NONE,
            );
            assert!(!g.frosted() && !g.is_solid(), "{}", t.id());
            assert_eq!(g.alpha(0.8), 0.4);
        }
    }

    #[test]
    fn every_design_frosts() {
        for t in crate::themes::all() {
            let g = surface_glass(
                &t,
                SurfaceClass::Panel,
                &Surface::new().frost(1.0),
                SurfaceFx::NONE,
            );
            assert!(g.frosted() && g.blur > 0.0, "{}", t.id());
            assert!(g.alpha(1.0) < 1.0, "{}", t.id());
        }
    }
}

#[cfg(test)]
mod mesh_tests {
    use super::*;
    use crate::Color;
    const W: usize = 96;
    const H: usize = 60;

    #[test]
    fn colour_where_a_blob_reaches_and_a_flat_canvas_at_zero() {
        let canvas = Color::rgba(250, 250, 252, 255);
        let wash = [
            Color::rgba(120, 60, 220, 255),
            Color::rgba(40, 80, 230, 255),
            Color::rgba(20, 190, 190, 255),
        ];
        let px = mesh_pixels(canvas, wash, 0.5, W, H);
        let o = (2 * W + 2) * 4;
        assert_ne!(px[o..o + 3], [250, 250, 252]);
        let flat = mesh_pixels(canvas, wash, 0.0, W, H);
        assert!(flat.chunks_exact(4).all(|p| p[..3] == [250, 250, 252]));
    }
}
