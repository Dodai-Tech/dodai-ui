//! Toolkit-independent surface textures and effects.
//!
//! The geometry of every Dodai surface effect (grain, halftone, hatch, grid, circuit traces,
//! scanlines, glitch, sheen, flicker, bloom, pulse, chase, click ripples, ...) lives here once.
//! A toolkit implements [`Painter`] (four primitives: quad, dot, stroked path, filled path) and
//! calls [`paint`] each frame; the effects are clipped to the shape, follow cut corners, and
//! never look at a theme id: they read a [`SurfaceFx`] and the colours of a [`SurfacePaint`].

mod canvas;
mod clip;
mod effects;
pub mod frost;
mod geom;
mod vocab;

/// Neutral distance in px between scanline bars.
pub const SCANLINE_PITCH: f32 = 3.0;

use canvas::Canvas;
pub use clip::{Clip, Corners, ShapeKind};
use effects::{
    brackets, chase, depth, edge_light, glitch, gloss, pop, press_frame, pulse, sheen, strip, tape,
    texture, tint, wash,
};
use geom::noise;
pub use geom::{poly_span, row_span};
pub use vocab::{
    EffectSet, PressStyle, Surface, SurfaceClass, SurfaceFx, SurfaceTargets, SurfaceTuning, Texture,
};

/// Smallest side (px) of a surface that gets a texture or effect.
const MIN_SIDE: f32 = 16.0;

/// A straight (non-premultiplied) sRGB colour with `f32` channels in `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    /// Red.
    pub r: f32,
    /// Green.
    pub g: f32,
    /// Blue.
    pub b: f32,
    /// Opacity.
    pub a: f32,
}

impl Rgba {
    /// Opaque white.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
}

impl Rgba {
    /// HSL lightness, `0..=1`.
    pub fn lightness(self) -> f32 {
        (self.r.max(self.g).max(self.b) + self.r.min(self.g).min(self.b)) / 2.0
    }
}

/// `c` with its opacity multiplied by `k` (clamped to `0..=1`).
pub fn scale_alpha(c: Rgba, k: f32) -> Rgba {
    Rgba {
        a: (c.a * k).clamp(0.0, 1.0),
        ..c
    }
}

/// What a toolkit implements to show the effects. Coordinates are px, local to the surface's
/// top-left corner; everything has already been clipped to the shape.
pub trait Painter {
    /// A filled rectangle.
    fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba);
    /// A filled circle of diameter `d` centred at `cx, cy`.
    fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba);
    /// A stroked path through `pts`, closed back to the first point when `closed`.
    fn stroke(&mut self, pts: &[(f32, f32)], closed: bool, width: f32, c: Rgba);
    /// A filled polygon.
    fn fill(&mut self, pts: &[(f32, f32)], c: Rgba);
}

/// The mix plus the colours resolved from a theme.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfacePaint {
    /// What to draw.
    pub fx: SurfaceFx,
    /// Pattern colour (the text colour).
    pub ink: Rgba,
    /// Surface colour, for holes and dimming.
    pub hole: Rgba,
    /// First glitch ghost.
    pub a: Rgba,
    /// Second glitch ghost.
    pub b: Rgba,
    /// Sheen band.
    pub light: Rgba,
    /// Colour of lit lines and lamps.
    pub neon: Rgba,
    /// Colour of inner shadows.
    pub shade: Rgba,
    /// The gradient of the `wash` effect, start to end.
    pub wash: [Rgba; 3],
    /// Timing and sizes from the theme.
    pub tune: SurfaceTuning,
    /// Whether a press on this surface plays the click ripple.
    pub pressable: bool,
    /// Lightness (0..1) of the fill underneath, when known (see [`Self::contrasting`]).
    pub fill_lightness: f32,
}

impl SurfacePaint {
    /// Draw the pattern in whichever of text and background contrasts with a fill of lightness
    /// `fill_lightness`, so it stays visible on bright fills (a primary button) as well as dark ones.
    #[must_use]
    pub fn contrasting(mut self, fill_lightness: f32) -> Self {
        self.fill_lightness = fill_lightness;
        let (text, back) = (self.ink, self.hole);
        if (text.lightness() - fill_lightness).abs() < (back.lightness() - fill_lightness).abs() {
            self.ink = back;
            self.hole = text;
        }
        self
    }
}

/// A click ripple in progress on the surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ripple {
    /// Touch point, relative to the surface.
    pub at: (f32, f32),
    /// Progress through the ripple, `0..1`.
    pub t: f32,
}

/// Where and when to paint.
#[derive(Debug, Clone, Copy)]
pub struct Frame<'a> {
    /// Surface width, px.
    pub w: f32,
    /// Surface height, px.
    pub h: f32,
    /// The surface's shape.
    pub shape: &'a Clip,
    /// Varies the random-looking details between surfaces (any stable per-surface number).
    pub seed: i32,
    /// Seconds on the animation clock.
    pub time: f32,
    /// Click ripples currently playing on this surface.
    pub presses: &'a [Ripple],
}

/// Paint the texture and effects of `sp` for one frame. Returns `true` when the surface is
/// animated and wants another frame.
pub fn paint(out: &mut dyn Painter, f: &Frame, sp: &SurfacePaint) -> bool {
    let (w, h, shape, seed) = (f.w, f.h, f.shape, f.seed);
    if w.min(h) < MIN_SIDE {
        return false;
    }
    let fx = sp.fx;
    let mut cv = Canvas {
        out,
        shape,
        pts: (shape.kind() == ShapeKind::Chamfered).then(|| shape.outline(w, h)),
        w,
        h,
        inset: shape.border_width,
        neon: fx.neon.clamp(0.0, 1.0),
        neon_c: sp.neon,
        halo: sp.tune.halo,
        clear: 0.0,
        time: f.time,
    };
    if fx.tint > 0.0 {
        tint(&mut cv, sp);
    }
    if fx.has_texture() {
        cv.clear = fx.clear.clamp(0.0, 1.0);
        texture(&mut cv, sp, seed);
        cv.clear = 0.0;
    }
    if fx.depth > 0.0 {
        depth(&mut cv, sp);
    }
    if fx.edge > 0.0 {
        edge_light(&mut cv, sp);
    }
    if fx.brackets > 0.0 {
        brackets(&mut cv, sp);
    }
    if fx.pop > 0.0 {
        pop(&mut cv, sp);
    }
    if fx.wash > 0.0 {
        wash(&mut cv, sp);
    }
    if fx.gloss > 0.0 {
        gloss(&mut cv, sp);
    }
    if fx.tape > 0.0 {
        tape(&mut cv, sp);
    }
    let mut animating = fx.animated();
    if fx.ripple > 0.0 && sp.pressable && !f.presses.is_empty() {
        animating = true;
        let k = fx.ripple.clamp(0.0, 1.0);
        for r in f.presses {
            press_frame(&mut cv, fx.press, sp.ink, sp.ink, k, r.at, r.t);
        }
    }
    if fx.strip > 0.0 {
        strip(&mut cv, sp);
    }
    if fx.pulse > 0.0 {
        pulse(&mut cv, sp);
    }
    if fx.chase > 0.0 {
        chase(&mut cv, sp, seed);
    }
    if fx.bloom > 0.0 {
        let n = 4;
        for k in (1..=n).rev() {
            let width = cv.inset + 2.0 * fx.bloom * sp.tune.bloom_spread * k as f32 / n as f32;
            cv.outline(0.0, width, scale_alpha(sp.a, 0.5 * fx.bloom / n as f32));
        }
    }
    if fx.sheen > 0.0 {
        sheen(&mut cv, sp);
    }
    if fx.flicker > 0.0 {
        let frame = (cv.time * sp.tune.flicker_rate) as i32;
        if noise(frame, seed) < fx.flicker * sp.tune.flicker_chance {
            cv.fill_shape(scale_alpha(sp.hole, 0.28 * fx.flicker.max(0.3)));
        }
    }
    if fx.glitch > 0.0 {
        glitch(&mut cv, sp, seed);
    }
    animating
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records every primitive so tests can inspect what an effect drew.
    #[derive(Default)]
    struct Rec {
        quads: Vec<(f32, f32, f32, f32, Rgba)>,
        dots: usize,
        paths: Vec<Vec<(f32, f32)>>,
    }

    impl Painter for Rec {
        fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
            self.quads.push((x, y, w, h, c));
        }
        fn dot(&mut self, _: f32, _: f32, _: f32, _: Rgba) {
            self.dots += 1;
        }
        fn stroke(&mut self, pts: &[(f32, f32)], _: bool, _: f32, _: Rgba) {
            self.paths.push(pts.to_vec());
        }
        fn fill(&mut self, pts: &[(f32, f32)], _: Rgba) {
            self.paths.push(pts.to_vec());
        }
    }

    /// Stand-in for a design: just the outline the effects clip to.
    struct Theme {
        shape: Clip,
    }

    impl Default for Theme {
        fn default() -> Self {
            Self {
                shape: Clip {
                    radius: 6.0,
                    border_width: 1.0,
                    ..Clip::default()
                },
            }
        }
    }

    fn run(sp: &SurfacePaint, theme: &Theme, presses: &[Ripple]) -> (Rec, bool) {
        let mut rec = Rec::default();
        let f = Frame {
            w: 160.0,
            h: 64.0,
            shape: &theme.shape,
            seed: 7,
            time: 1.3,
            presses,
        };
        let animating = paint(&mut rec, &f, sp);
        (rec, animating)
    }

    fn only(fx: SurfaceFx) -> SurfacePaint {
        let ink = Rgba {
            r: 0.1,
            g: 0.1,
            b: 0.1,
            a: 1.0,
        };
        SurfacePaint {
            fx,
            ink,
            hole: Rgba::WHITE,
            a: Rgba { r: 0.0, ..ink },
            b: Rgba { g: 0.0, ..ink },
            light: Rgba::WHITE,
            neon: Rgba { b: 0.9, ..ink },
            shade: ink,
            wash: [ink; 3],
            tune: SurfaceTuning::DEFAULT,
            pressable: true,
            fill_lightness: 0.0,
        }
    }

    #[test]
    fn every_texture_draws_something_inside_the_surface() {
        let t = Theme::default();
        for tex in Texture::ALL.into_iter().filter(|t| *t != Texture::None) {
            let sp = only(SurfaceFx {
                texture: tex,
                strength: 0.8,
                scale: 1.0,
                ..SurfaceFx::NONE
            });
            let (rec, _) = run(&sp, &t, &[]);
            let drew = rec.quads.len() + rec.dots + rec.paths.len();
            assert!(drew > 0, "{tex:?} drew nothing");
            for (x, y, w, h, c) in &rec.quads {
                assert!(
                    *x >= -0.5 && *y >= -0.5 && x + w <= 160.5 && y + h <= 64.5,
                    "{tex:?}"
                );
                assert!((0.0..=1.0).contains(&c.a), "{tex:?} alpha {}", c.a);
            }
        }
    }

    #[test]
    fn nothing_is_drawn_for_the_empty_mix_or_a_tiny_surface() {
        let t = Theme::default();
        let (rec, anim) = run(&only(SurfaceFx::NONE), &t, &[]);
        assert!(rec.quads.is_empty() && rec.paths.is_empty() && !anim);
        let sp = only(SurfaceFx {
            texture: Texture::Grid,
            strength: 1.0,
            ..SurfaceFx::NONE
        });
        let mut rec = Rec::default();
        let tiny = Frame {
            w: 10.0,
            h: 10.0,
            shape: &t.shape,
            seed: 0,
            time: 0.0,
            presses: &[],
        };
        assert!(!paint(&mut rec, &tiny, &sp) && rec.quads.is_empty());
    }

    #[test]
    fn painting_is_deterministic_and_ripples_ask_for_frames() {
        let t = Theme::default();
        let sp = only(SurfaceFx {
            texture: Texture::Grain,
            strength: 0.7,
            ripple: 1.0,
            ..SurfaceFx::NONE
        });
        let (a, _) = run(&sp, &t, &[]);
        let (b, anim_idle) = run(&sp, &t, &[]);
        assert_eq!(a.quads, b.quads);
        assert!(!anim_idle, "a static texture needs no more frames");
        let (_, anim) = run(
            &sp,
            &t,
            &[Ripple {
                at: (40.0, 20.0),
                t: 0.4,
            }],
        );
        assert!(anim, "a playing ripple needs the next frame");
    }

    #[test]
    fn a_huge_surface_costs_a_bounded_number_of_primitives() {
        let t = Theme::default();
        let big = |sp: &SurfacePaint| {
            let mut rec = Rec::default();
            let f = Frame {
                w: 2000.0,
                h: 1200.0,
                shape: &t.shape,
                seed: 3,
                time: 1.7,
                presses: &[],
            };
            paint(&mut rec, &f, sp);
            rec.quads.len() + rec.dots + rec.paths.len()
        };
        for tex in Texture::ALL {
            let n = big(&only(SurfaceFx {
                texture: tex,
                strength: 1.0,
                scale: 1.0,
                ..SurfaceFx::NONE
            }));
            assert!(n <= 2500, "{tex:?} drew {n} primitives");
        }
        for (name, set) in effects() {
            let mut fx = SurfaceFx::NONE;
            set(&mut fx);
            let n = big(&only(fx));
            assert!(n <= 2000, "{name} drew {n} primitives");
        }
    }

    fn finite(rec: &Rec, what: &str) {
        for (x, y, w, h, c) in &rec.quads {
            assert!(
                [x, y, w, h, &c.r, &c.g, &c.b, &c.a]
                    .iter()
                    .all(|v| v.is_finite()),
                "{what}: non-finite quad"
            );
        }
        for p in &rec.paths {
            assert!(
                p.iter().all(|(x, y)| x.is_finite() && y.is_finite()),
                "{what}: non-finite path"
            );
        }
    }

    type Setter = fn(&mut SurfaceFx);

    /// Every animated effect, switched on alone.
    fn effects() -> Vec<(&'static str, Setter)> {
        vec![
            ("glitch", |f| f.glitch = 1.0),
            ("sheen", |f| f.sheen = 1.0),
            ("flicker", |f| f.flicker = 1.0),
            ("bloom", |f| f.bloom = 1.0),
            ("pulse", |f| f.pulse = 1.0),
            ("chase", |f| f.chase = 1.0),
            ("edge", |f| f.edge = 1.0),
            ("brackets", |f| f.brackets = 1.0),
            ("depth", |f| f.depth = 1.0),
            ("tape", |f| f.tape = 1.0),
            ("gloss", |f| f.gloss = 1.0),
            ("strip", |f| f.strip = 1.0),
            ("wash", |f| f.wash = 1.0),
            ("tint", |f| f.tint = 1.0),
            ("pop", |f| f.pop = 1.0),
        ]
    }

    fn at(t: &Theme, sp: &SurfacePaint, time: f32) -> (Rec, bool) {
        let mut rec = Rec::default();
        let f = Frame {
            w: 160.0,
            h: 64.0,
            shape: &t.shape,
            seed: 7,
            time,
            presses: &[],
        };
        let anim = paint(&mut rec, &f, sp);
        (rec, anim)
    }

    #[test]
    fn every_effect_draws_at_some_moment_and_stays_finite() {
        let t = Theme::default();
        for (name, set) in effects() {
            let mut fx = SurfaceFx::NONE;
            set(&mut fx);
            let sp = only(fx);
            let mut drew = false;
            for step in 0..60 {
                let (rec, _) = at(&t, &sp, step as f32 * 0.137);
                finite(&rec, name);
                drew |= rec.quads.len() + rec.dots + rec.paths.len() > 0;
            }
            assert!(drew, "{name} never drew");
        }
    }

    #[test]
    fn animated_effects_ask_for_the_next_frame() {
        let t = Theme::default();
        for (name, set) in effects() {
            // Only the moving ones; the rest are static looks.
            if !matches!(name, "glitch" | "sheen" | "flicker" | "pulse" | "chase") {
                continue;
            }
            let mut fx = SurfaceFx::NONE;
            set(&mut fx);
            let (_, anim) = at(&t, &only(fx), 0.5);
            assert!(anim, "{name} is animated but did not ask for a frame");
        }
    }

    #[test]
    fn every_texture_and_effect_survives_every_outline_and_size() {
        let outlines = [
            Clip::default(),
            Clip {
                radius: 12.0,
                border_width: 1.0,
                ..Clip::default()
            },
            Clip {
                chamfer: 10.0,
                corners: Corners::ALL,
                border_width: 2.0,
                ..Clip::default()
            },
            Clip {
                chamfer: 10.0,
                corners: Corners::TOP_LEFT,
                scoop: true,
                ..Clip::default()
            },
        ];
        for clip in &outlines {
            for (w, h) in [(16.0, 16.0), (17.0, 400.0), (900.0, 20.0), (2000.0, 1200.0)] {
                for tex in Texture::ALL {
                    let mut fx = SurfaceFx {
                        texture: tex,
                        strength: 1.0,
                        scale: 1.0,
                        ..SurfaceFx::NONE
                    };
                    for (_, set) in effects() {
                        set(&mut fx);
                    }
                    let sp = only(fx);
                    let mut rec = Rec::default();
                    let f = Frame {
                        w,
                        h,
                        shape: clip,
                        seed: 3,
                        time: 1.7,
                        presses: &[],
                    };
                    paint(&mut rec, &f, &sp);
                    finite(&rec, &format!("{tex:?} {w}x{h}"));
                }
            }
        }
    }

    #[test]
    fn contrasting_swaps_ink_and_hole_to_stay_visible() {
        let sp = only(SurfaceFx::NONE);
        let on_dark = sp.contrasting(0.0);
        let on_light = sp.contrasting(1.0);
        assert!(on_dark.ink.lightness() >= on_light.ink.lightness());
        assert_ne!(on_dark.ink, on_light.ink);
    }

    #[test]
    fn wash_runs_through_the_three_colours_and_stays_inside() {
        let t = Theme::default();
        let mut sp = only(SurfaceFx {
            wash: 1.0,
            ..SurfaceFx::NONE
        });
        let red = Rgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let blue = Rgba {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        };
        sp.wash = [red, red, blue];
        let (rec, anim) = run(&sp, &t, &[]);
        assert!(!anim && !rec.quads.is_empty());
        for (x, y, w, h, c) in &rec.quads {
            assert!(*x >= -0.5 && *y >= -0.5 && x + w <= 160.5 && y + h <= 64.5);
            assert!(c.a > 0.0 && c.a <= 0.24 + 1e-4);
        }
        let first = rec.quads.first().unwrap().4;
        let last = rec.quads.last().unwrap().4;
        assert!(first.r > first.b, "starts red");
        assert!(last.b > last.r, "ends blue");
    }
}

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;
