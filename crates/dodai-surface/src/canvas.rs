//! The clipped drawing surface the effects paint on: every call is clipped to the shape (and
//! inside its border) before it reaches the [`Painter`].

use crate::geom::{outline_of, poly_span, row_span};
use crate::{scale_alpha, Painter, Rgba};
use dodai_core::Shape;

pub(crate) struct Canvas<'a> {
    pub out: &'a mut dyn Painter,
    pub shape: &'a Shape,
    pub pts: Option<Vec<(f32, f32)>>,
    pub w: f32,
    pub h: f32,
    pub inset: f32,
    pub neon: f32,
    pub neon_c: Rgba,
    pub halo: f32,
    /// Texture fade towards the centre (`0` = even); set only while a texture is drawn.
    pub clear: f32,
    /// Seconds on the animation clock.
    pub time: f32,
}

impl Canvas<'_> {
    pub fn span(&self, y: f32) -> Option<(f32, f32)> {
        match &self.pts {
            Some(p) => poly_span(p, y.clamp(0.0, self.h)),
            None => row_span(self.shape, self.w, self.h, y.clamp(0.0, self.h)),
        }
    }

    /// Alpha factor for the texture at `x, y`: 1 at the edges, down to `1 - clear` in the middle.
    fn keep(&self, x: f32, y: f32) -> f32 {
        if self.clear <= 0.0 {
            return 1.0;
        }
        let nx = ((x - self.w / 2.0) / (self.w / 2.0).max(1.0)).abs();
        let ny = ((y - self.h / 2.0) / (self.h / 2.0).max(1.0)).abs();
        let t = ((nx.max(ny) - 0.3) / 0.7).clamp(0.0, 1.0);
        let t = t * t * (3.0 - 2.0 * t);
        1.0 - self.clear * (1.0 - t)
    }

    /// Paint `x, y, w, h` clipped to the shape (and inside the border).
    pub fn quad(&mut self, x: f32, y: f32, w: f32, h: f32, c: Rgba) {
        let c = scale_alpha(c, self.keep(x + w / 2.0, y + h / 2.0));
        let (y0, y1) = (y.max(self.inset), (y + h).min(self.h - self.inset));
        if y1 <= y0 {
            return;
        }
        let (Some(a), Some(b)) = (self.span(y0), self.span(y1)) else {
            return;
        };
        let lo = a.0.max(b.0).max(self.inset);
        let hi = a.1.min(b.1).min(self.w - self.inset);
        let (x0, x1) = (x.max(lo), (x + w).min(hi));
        if x1 > x0 {
            self.out.quad(x0, y0, x1 - x0, y1 - y0, c);
        }
    }

    /// A thin line segment: flat ink, or lit (halo plus bright core) when neon is on.
    pub fn line(&mut self, x: f32, y: f32, w: f32, h: f32, ink: Rgba) {
        if self.neon <= 0.0 {
            self.quad(x, y, w, h, ink);
            return;
        }
        let (n, c) = (self.neon, self.neon_c);
        let o = self.halo;
        self.quad(
            x - o,
            y - o,
            w + 2.0 * o,
            h + 2.0 * o,
            scale_alpha(c, 0.10 * n),
        );
        let i = o / 2.0;
        self.quad(
            x - i,
            y - i,
            w + 2.0 * i,
            h + 2.0 * i,
            scale_alpha(c, 0.22 * n),
        );
        self.quad(
            x,
            y,
            w,
            h,
            scale_alpha(c, (ink.a * 2.0).min(1.0) * (0.5 + 0.5 * n)),
        );
    }

    /// A round dot of diameter `d` centred at `cx, cy`, kept fully inside the shape.
    pub fn dot(&mut self, cx: f32, cy: f32, d: f32, c: Rgba) {
        let c = scale_alpha(c, self.keep(cx, cy));
        let r = d / 2.0;
        let (Some(a), Some(b)) = (self.span(cy - r), self.span(cy + r)) else {
            return;
        };
        if cx - r < a.0.max(b.0).max(self.inset) || cx + r > a.1.min(b.1).min(self.w - self.inset) {
            return;
        }
        if cy - r < self.inset || cy + r > self.h - self.inset {
            return;
        }
        self.out.dot(cx, cy, d, c);
    }

    fn shape_points(&self) -> Vec<(f32, f32)> {
        self.pts
            .clone()
            .unwrap_or_else(|| outline_of(self.shape, self.w, self.h))
    }

    /// The shape's outline shifted `dx` px right, stroked `width` px wide.
    pub fn outline(&mut self, dx: f32, width: f32, c: Rgba) {
        let pts: Vec<_> = self
            .shape_points()
            .into_iter()
            .map(|(x, y)| (x + dx, y))
            .collect();
        self.out.stroke(&pts, true, width, c);
    }

    /// The shape's outline pulled in by `d` px, stroked `width` px wide (inner glow and shadow).
    pub fn outline_inset(&mut self, d: f32, width: f32, c: Rgba) {
        let (w, h) = (self.w - 2.0 * d, self.h - 2.0 * d);
        if w < 4.0 || h < 4.0 {
            return;
        }
        let mut sh = self.shape.clone();
        sh.radius = (sh.radius - d).max(0.0);
        let pts: Vec<_> = outline_of(&sh, w, h)
            .into_iter()
            .map(|(x, y)| (x + d, y + d))
            .collect();
        self.out.stroke(&pts, true, width, c);
    }

    /// An open polyline through `pts` (shape-local px).
    pub fn polyline(&mut self, pts: &[(f32, f32)], width: f32, c: Rgba) {
        self.out.stroke(pts, false, width, c);
    }

    /// A circle outline.
    pub fn ring(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: Rgba) {
        let n = 48;
        let pts: Vec<(f32, f32)> = (0..=n)
            .map(|i| {
                let a = std::f32::consts::TAU * i as f32 / n as f32;
                (cx + r * a.cos(), cy + r * a.sin())
            })
            .collect();
        self.polyline(&pts, width, c);
    }

    /// A filled polygon (shape-local px), for strips and tape.
    pub fn polygon(&mut self, pts: &[(f32, f32)], c: Rgba) {
        self.out.fill(pts, c);
    }

    /// A circle outline clipped to the surface rectangle (inside the border).
    pub fn ring_clipped(&mut self, cx: f32, cy: f32, r: f32, width: f32, c: Rgba) {
        let n = ((r * 0.6) as usize).clamp(24, 96);
        let (y0, y1) = (self.inset + 1.0, self.h - self.inset - 1.0);
        let mut run: Vec<(f32, f32)> = Vec::new();
        for i in 0..=n {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            let (x, y) = (cx + r * a.cos(), cy + r * a.sin());
            let inside = y >= y0
                && y <= y1
                && self.span(y).is_some_and(|(a, b)| {
                    x >= a.max(self.inset + 1.0) && x <= b.min(self.w - self.inset - 1.0)
                });
            if inside {
                run.push((x, y));
            } else if !run.is_empty() {
                self.polyline(&run, width, c);
                run.clear();
            }
        }
        if run.len() > 1 {
            self.polyline(&run, width, c);
        }
    }

    pub fn fill_shape(&mut self, c: Rgba) {
        let pts = self.shape_points();
        self.out.fill(&pts, c);
    }
}
