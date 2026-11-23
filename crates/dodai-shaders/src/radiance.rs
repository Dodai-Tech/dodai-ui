//! # Radiance cascades: 2D light transport with shadows, in a few passes
//!
//! A solver for how light spreads through a flat scene: lights shine, walls block, coloured gas
//! glows and dims what is behind it. For every pixel it returns the **mean intensity** (the light
//! arriving from all directions, averaged), which is what a surface at that pixel would be lit by.
//! Soft shadows, light around corners of an opening and lit rims on the edges facing a light all
//! come out of it without being asked for.
//!
//! It runs on the CPU (all cores) and is the reference for GPU ports: every step below is one
//! full-screen pass, so it maps onto fragment or compute shaders (see "Porting to a GPU").
//!
//! ## Source
//!
//! C. M. J. Osborne and A. Sannikov, *Radiance Cascades: A Novel High-Resolution Formal Solution
//! for Multidimensional Non-LTE Radiative Transfer*, RAS Techniques and Instruments (2024),
//! [arXiv:2408.14425](https://arxiv.org/abs/2408.14425),
//! [doi:10.1093/rasti/rzae062](https://doi.org/10.1093/rasti/rzae062).
//! The method is A. Sannikov's (first used for real-time global illumination in games); the paper
//! states it as a formal solution of the radiative transfer equation, which is the form used here.
//!
//! ## The problem
//!
//! Along a ray, radiance `I` changes by emission and absorption (the transfer equation):
//!
//! ```text
//! dI/ds = χ (S − I)
//! ```
//!
//! `χ` is how strongly the medium absorbs per pixel of path ([`Cell::absorption`](crate::radiance::Cell::absorption)) and `S` the
//! colour it glows with ([`Cell::source`](crate::radiance::Cell::source)). Across a stretch of length `Δs` with constant `χ`
//! and `S` this solves exactly to
//!
//! ```text
//! I_out = I_in · e^(−χΔs) + S · (1 − e^(−χΔs))
//! ```
//!
//! A wall is a cell with a large `χ` and `S = 0`, a light a large `χ` and `S` = its colour, a glow
//! (smoke, a firework's sparks) a small `χ`. The wanted result is, at every point `x`,
//!
//! ```text
//! J(x) = 1/2π ∮ I(x, ω) dω
//! ```
//!
//! **The cost problem.** Casting `N` rays from each of `P` pixels to the edge of the scene costs
//! `P · N · L` steps (`L` the scene size), and `N` must grow with `L`: a light of size `r` at
//! distance `d` covers an angle of about `r/d`, so far lights fall between the rays ("ray
//! effects", the spokes of a brute-force render). Radiance cascades get the same quality for a
//! cost that does not depend on `N`.
//!
//! ## The observation it rests on (the penumbra condition)
//!
//! Near a point, light changes quickly with **position** but slowly with **direction** (one
//! nearby light fills a wide angle, and a step to the side changes it a lot). Far away it is the
//! other way round: distant lights are small, so direction matters a lot, but moving a little
//! barely changes what is seen. So:
//!
//! * light from **near** needs many sample positions and few directions,
//! * light from **far** needs few sample positions and many directions.
//!
//! A single grid of rays cannot be both; a cascade of grids, each responsible for one distance
//! band, can.
//!
//! ## The cascades
//!
//! Cascade `i` is a grid of **probes**. Each probe casts rays only over its own band of
//! distances, the **interval** `[tᵢ, tᵢ₊₁)`. Going up one cascade:
//!
//! | | cascade 0 | cascade i |
//! |---|---|---|
//! | probe spacing | `Δ₀` | `Δ₀ · 2ⁱ` |
//! | directions per probe | `N₀` | `N₀ · 4ⁱ` |
//! | interval start `tᵢ` | `0` | `l₀ · (4ⁱ − 1) / 3` |
//! | interval length | `l₀` | `l₀ · 4ⁱ` |
//!
//! The probes get 4× fewer (half as many per axis) while the directions get 4× more, so **every
//! cascade holds the same number of rays**. Each band is 4× longer but starts 4× further out, so
//! the angle a light covers relative to the angle between rays stays the same in every band:
//! that is the penumbra condition, met at every distance. Enough cascades are used for the last
//! band to reach the far corner of the scene ([`Cascades::count`](crate::radiance::Cascades::count)).
//!
//! Directions are spread evenly: ray `k` of `Nᵢ` points at `θ = 2π (k + ½) / Nᵢ`. The four rays
//! `4k … 4k+3` of cascade `i+1` split the angle of ray `k` of cascade `i` into quarters, which is
//! what makes merging possible.
//!
//! ## The algorithm
//!
//! 1. **Distance field** (once per scene). For every pixel, the distance to the nearest pixel that
//!    absorbs anything (two-pass chamfer transform). Rays use it to jump through empty space
//!    instead of stepping one pixel at a time.
//! 2. **Trace each cascade.** For each probe and direction, march the ray over the probe's interval
//!    only, applying the formula above step by step. Keep two numbers: the light gathered on the
//!    interval, `Rᵢ`, and how much of what lies behind still gets through, the transmittance `Tᵢ`
//!    (1 in empty space, 0 behind a wall).
//! 3. **Merge from the top down.** The highest cascade has nothing behind it. For every other
//!    cascade, the light beyond the end of a ray is what cascade `i+1` saw from there on:
//!
//!    ```text
//!    far      = Σ_bilinear  w_q · ¼ Σ_{c=0..3}  Iᵢ₊₁(q, 4k + c)
//!    Iᵢ(p, k) = Rᵢ(p, k) + Tᵢ(p, k) · far
//!    ```
//!
//!    `q` runs over the four cascade-`i+1` probes around probe `p`, with bilinear weights `w_q`;
//!    the inner sum averages the four finer directions inside ray `k`. After the merge, `Iᵢ` is
//!    the light along the whole ray to the edge of the scene, not only over its interval.
//!    Because merging goes top-down, a cascade can be traced and merged in one pass and only
//!    two cascades are ever in memory.
//! 4. **Mean intensity.** At cascade 0 every probe averages its `N₀` merged directions:
//!    `J(p) = 1/N₀ Σₖ I₀(p, k)`. Pixels between probes take the bilinear mix (with `Δ₀ = 1`,
//!    one probe per pixel, there is nothing to mix).
//!
//! **Cost.** With `P` pixels, each cascade casts `P · N₀ / Δ₀²` rays, and the march costs little in
//! empty space thanks to the distance field. `C` cascades cost `C · P · N₀ / Δ₀²` rays, with `C`
//! growing only with `log₄` of the scene size. A brute-force solve of the same quality needs many
//! hundreds of rays per pixel, each running to the edge of the scene ([`reference`](crate::radiance::reference)).
//!
//! **Known limit.** Plain bilinear merging lets a little light leak past thin walls and can ring
//! slightly next to them: a cascade-`i` ray ends at its own probe's position while the
//! cascade-`i+1` ray continuing it starts from a probe up to `Δᵢ₊₁` away. The "bilinear fix"
//! (tracing from each cascade-`i` probe to each of the four neighbouring cascade-`i+1` interval
//! starts, then interpolating) removes it at 4× the tracing cost; walls a few pixels thick
//! do not need it.
//!
//! ## Porting to a GPU
//!
//! Each cascade is one texture of `(w/Δᵢ) × (h/Δᵢ)` probes, each `Nᵢ` texels (lay the directions
//! out as a `√Nᵢ × √Nᵢ` block per probe: all cascades then have the same texture size).
//!
//! * One pass builds the distance field: jump flooding, about `log₂(size)` passes, and only when
//!   the scene changes.
//! * One pass per cascade, from the top down, traces and merges, reading the cascade above it.
//! * One last pass averages cascade 0 into the light map that surfaces sample.
//!
//! At a quarter of the screen size this is about 5 cascades, so 5–6 full-screen passes a frame.
//! The light map can be refreshed at a lower rate than the screen.
//!
//! ```
//! use dodai_shaders::radiance::{solve, Cascades, Cell, Scene};
//! use dodai_shaders::vec3;
//!
//! let mut scene = Scene::new(64, 64);
//! scene.fill_disc(12.0, 32.0, 4.0, Cell::light(vec3(1.0, 0.8, 0.4)));
//! scene.fill_rect(30.0, 16.0, 34.0, 48.0, Cell::wall());
//! let light = solve(&scene, &Cascades::default());
//! assert_eq!(light.len(), 64 * 64);
//! // behind the wall is darker than at the same distance in the open
//! assert!(light[32 * 64 + 52].x < light[(32 - 20) * 64 + 28].x);
//! ```

use crate::math::{vec3, Vec3};
use std::f32::consts::TAU;

/// Absorption of [`Cell::wall`] and [`Cell::light`] per pixel: `e^(−64)`, so nothing gets through.
const OPAQUE: f32 = 64.0;
/// Below this transmittance a ray is treated as fully blocked.
const BLOCKED: f32 = 1e-4;
/// The chamfer distance (steps of 1 and √2) overestimates the true distance by up to 8.24 %.
const CHAMFER_SAFE: f32 = 0.92;
/// Half the diagonal of a pixel twice over: from a point to its cell centre, and from a
/// matter cell's centre to its edge.
const CELL_SLACK: f32 = 1.5;
/// Cascades are capped here; `4^16` directions would not fit any memory anyway.
const MAX_CASCADES: u32 = 16;

/// One pixel of the medium light travels through.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Cell {
    /// How strongly the cell absorbs, per pixel of path (`χ`). `0` is empty space.
    pub absorption: f32,
    /// The colour the cell glows with (`S`): what a ray inside it tends to when it is thick.
    pub source: Vec3,
}

impl Cell {
    /// Empty space.
    pub const EMPTY: Cell = Cell {
        absorption: 0.0,
        source: vec3(0.0, 0.0, 0.0),
    };

    /// Opaque and black: blocks light and casts shadows.
    pub const fn wall() -> Cell {
        Cell {
            absorption: OPAQUE,
            source: vec3(0.0, 0.0, 0.0),
        }
    }

    /// Opaque and shining with `colour`.
    pub const fn light(colour: Vec3) -> Cell {
        Cell {
            absorption: OPAQUE,
            source: colour,
        }
    }

    /// Partly transparent glow (smoke, sparks): `absorption` per pixel, around `0.05`–`0.5`.
    pub const fn glow(colour: Vec3, absorption: f32) -> Cell {
        Cell {
            absorption,
            source: colour,
        }
    }
}

/// A grid of [`Cell`]s, one per pixel, row by row. Cell `(x, y)` covers `[x, x+1) × [y, y+1)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Scene {
    /// An empty scene of `width × height` pixels.
    pub fn new(width: u32, height: u32) -> Scene {
        Scene {
            width,
            height,
            cells: vec![Cell::EMPTY; width as usize * height as usize],
        }
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// The cell at `(x, y)`, or [`Cell::EMPTY`] outside the scene.
    pub fn cell(&self, x: u32, y: u32) -> Cell {
        if x < self.width && y < self.height {
            self.cells[self.index(x, y)]
        } else {
            Cell::EMPTY
        }
    }

    /// Sets the cell at `(x, y)`; outside the scene nothing happens.
    pub fn set(&mut self, x: u32, y: u32, cell: Cell) {
        if x < self.width && y < self.height {
            let i = self.index(x, y);
            self.cells[i] = cell;
        }
    }

    /// Sets every cell whose centre lies in `[x0, x1) × [y0, y1)`.
    pub fn fill_rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, cell: Cell) {
        self.fill(|x, y| x >= x0 && x < x1 && y >= y0 && y < y1, cell);
    }

    /// Sets every cell whose centre lies within `r` of `(cx, cy)`.
    pub fn fill_disc(&mut self, cx: f32, cy: f32, r: f32, cell: Cell) {
        self.fill(|x, y| (x - cx).hypot(y - cy) <= r, cell);
    }

    fn fill(&mut self, inside: impl Fn(f32, f32) -> bool, cell: Cell) {
        for y in 0..self.height {
            for x in 0..self.width {
                if inside(x as f32 + 0.5, y as f32 + 0.5) {
                    self.set(x, y, cell);
                }
            }
        }
    }

    fn index(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }
}

/// How the cascades are laid out. The defaults suit scenes up to a few thousand pixels across.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cascades {
    /// Probe spacing of cascade 0 in pixels (`Δ₀`); `1` is one probe per pixel. Each cascade
    /// doubles it.
    pub spacing: u32,
    /// Ray directions per probe in cascade 0 (`N₀`). Each cascade has 4× as many.
    pub directions: u32,
    /// Ray length of cascade 0 in pixels (`l₀`). Each cascade's interval is 4× longer.
    pub interval: f32,
    /// March step in pixels inside matter. Empty space is crossed in larger jumps.
    pub step: f32,
}

impl Default for Cascades {
    fn default() -> Cascades {
        Cascades {
            spacing: 1,
            directions: 4,
            interval: 1.0,
            step: 0.5,
        }
    }
}

impl Cascades {
    /// Number of cascades for a `width × height` scene: enough for the last interval to end
    /// beyond the scene's diagonal.
    pub fn count(&self, width: u32, height: u32) -> u32 {
        let c = self.sanitized();
        let diagonal = (width as f32).hypot(height as f32);
        let mut n = 1;
        while n < MAX_CASCADES && interval_start(c.interval, n) < diagonal {
            n += 1;
        }
        n
    }

    /// Replaces values that would break the solve (zero, negative, not a number) with defaults.
    fn sanitized(&self) -> Cascades {
        let d = Cascades::default();
        let positive = |v: f32, fallback: f32| {
            if v.is_finite() && v > 0.0 {
                v
            } else {
                fallback
            }
        };
        Cascades {
            spacing: self.spacing.max(1),
            directions: self.directions.max(1),
            interval: positive(self.interval, d.interval),
            step: positive(self.step, d.step).clamp(0.05, 4.0),
        }
    }
}

/// Mean intensity `J` (light from all directions, averaged) at every pixel of `scene`, row by
/// row, `width × height` long. See the module docs for the algorithm.
pub fn solve(scene: &Scene, cascades: &Cascades) -> Vec<Vec3> {
    let (w, h) = (scene.width, scene.height);
    if w == 0 || h == 0 {
        return Vec::new();
    }
    let c = cascades.sanitized();
    let field = distance_field(scene);
    let mut upper: Option<(Level, Vec<Vec3>)> = None;
    for i in (0..c.count(w, h)).rev() {
        let level = Level::new(&c, i, w, h);
        let mut rays = vec![Vec3::default(); level.len()];
        par_fill(&mut rays, |n| {
            let (probe, k) = (n / level.dirs as usize, n % level.dirs as usize);
            let origin = level.probe_centre(probe);
            let angle = TAU * (k as f32 + 0.5) / level.dirs as f32;
            let dir = (angle.cos(), angle.sin());
            let (near, through) = march(scene, &field, origin, dir, level.start, level.end, c.step);
            match &upper {
                Some((up, light)) if through > 0.0 => {
                    near + far_light(up, light, origin, k) * through
                }
                _ => near,
            }
        });
        upper = Some((level, rays));
    }
    let Some((level, rays)) = upper else {
        return vec![Vec3::default(); w as usize * h as usize];
    };
    let probes: Vec<Vec3> = (0..level.probes())
        .map(|p| {
            let dirs = level.dirs as usize;
            let sum = rays[p * dirs..(p + 1) * dirs]
                .iter()
                .fold(Vec3::default(), |a, &b| a + b);
            sum / dirs as f32
        })
        .collect();
    let mut out = vec![Vec3::default(); w as usize * h as usize];
    par_fill(&mut out, |n| {
        let (x, y) = ((n % w as usize) as f32 + 0.5, (n / w as usize) as f32 + 0.5);
        level
            .bilinear((x, y))
            .iter()
            .fold(Vec3::default(), |a, &(p, wt)| a + probes[p] * wt)
    });
    out
}

/// Brute force for comparison: `rays` evenly spread rays from every pixel centre, each marched to
/// the edge of the scene. Slow; costs `rays` times the scene size per pixel.
pub fn reference(scene: &Scene, rays: u32) -> Vec<Vec3> {
    let (w, h) = (scene.width, scene.height);
    let rays = rays.max(1);
    let field = distance_field(scene);
    let step = Cascades::default().step;
    let end = (w as f32).hypot(h as f32) + 1.0;
    let mut out = vec![Vec3::default(); w as usize * h as usize];
    par_fill(&mut out, |n| {
        let origin = ((n % w as usize) as f32 + 0.5, (n / w as usize) as f32 + 0.5);
        let mut sum = Vec3::default();
        for k in 0..rays {
            let angle = TAU * (k as f32 + 0.5) / rays as f32;
            sum = sum
                + march(
                    scene,
                    &field,
                    origin,
                    (angle.cos(), angle.sin()),
                    0.0,
                    end,
                    step,
                )
                .0;
        }
        sum / rays as f32
    });
    out
}

/// Where interval `i` starts: `l₀ (4ⁱ − 1) / 3`, the sum of the lengths before it.
fn interval_start(l0: f32, i: u32) -> f32 {
    l0 * (4f32.powi(i as i32) - 1.0) / 3.0
}

/// The layout of one cascade.
struct Level {
    spacing: f32,
    cols: u32,
    rows: u32,
    dirs: u32,
    start: f32,
    end: f32,
}

impl Level {
    fn new(c: &Cascades, i: u32, w: u32, h: u32) -> Level {
        let spacing = c.spacing << i;
        Level {
            spacing: spacing as f32,
            cols: w.div_ceil(spacing),
            rows: h.div_ceil(spacing),
            dirs: c.directions * 4u32.pow(i),
            start: interval_start(c.interval, i),
            end: interval_start(c.interval, i + 1),
        }
    }

    fn probes(&self) -> usize {
        self.cols as usize * self.rows as usize
    }

    fn len(&self) -> usize {
        self.probes() * self.dirs as usize
    }

    fn probe_centre(&self, probe: usize) -> (f32, f32) {
        let (x, y) = (probe % self.cols as usize, probe / self.cols as usize);
        (
            (x as f32 + 0.5) * self.spacing,
            (y as f32 + 0.5) * self.spacing,
        )
    }

    /// The four probes around `pos` with their bilinear weights (clamped at the edges).
    fn bilinear(&self, pos: (f32, f32)) -> [(usize, f32); 4] {
        let axis = |v: f32, n: u32| {
            let g = (v / self.spacing - 0.5).clamp(0.0, (n - 1) as f32);
            let i0 = g.floor() as u32;
            (i0, (i0 + 1).min(n - 1), g - i0 as f32)
        };
        let (x0, x1, fx) = axis(pos.0, self.cols);
        let (y0, y1, fy) = axis(pos.1, self.rows);
        let at = |x: u32, y: u32| y as usize * self.cols as usize + x as usize;
        [
            (at(x0, y0), (1.0 - fx) * (1.0 - fy)),
            (at(x1, y0), fx * (1.0 - fy)),
            (at(x0, y1), (1.0 - fx) * fy),
            (at(x1, y1), fx * fy),
        ]
    }
}

/// Light beyond the end of ray `k` of a probe at `pos`: the four finer rays inside it, averaged,
/// at the four probes of the cascade above, mixed bilinearly.
fn far_light(up: &Level, light: &[Vec3], pos: (f32, f32), k: usize) -> Vec3 {
    let dirs = up.dirs as usize;
    up.bilinear(pos)
        .iter()
        .fold(Vec3::default(), |acc, &(p, wt)| {
            let base = p * dirs + 4 * k;
            let quarter = light[base] + light[base + 1] + light[base + 2] + light[base + 3];
            acc + quarter * (wt * 0.25)
        })
}

/// Marches from `origin + dir·t0` to `origin + dir·t1`. Returns the light gathered on the way and
/// the transmittance left for whatever lies beyond.
fn march(
    scene: &Scene,
    field: &[f32],
    origin: (f32, f32),
    dir: (f32, f32),
    t0: f32,
    t1: f32,
    step: f32,
) -> (Vec3, f32) {
    let (w, h) = (scene.width as f32, scene.height as f32);
    let mut light = Vec3::default();
    let mut through = 1.0;
    let mut t = t0;
    while t < t1 {
        let (x, y) = (origin.0 + dir.0 * t, origin.1 + dir.1 * t);
        // A straight ray that has left the rectangle never comes back.
        if x < 0.0 || y < 0.0 || x >= w || y >= h {
            break;
        }
        let skip = field[scene.index(x as u32, y as u32)] * CHAMFER_SAFE - CELL_SLACK;
        if skip > step {
            t += skip;
            continue;
        }
        let ds = step.min(t1 - t);
        let (mx, my) = (x + dir.0 * ds * 0.5, y + dir.1 * ds * 0.5);
        let cell = scene.cell(mx.max(0.0) as u32, my.max(0.0) as u32);
        if cell.absorption > 0.0 {
            let absorbed = 1.0 - (-cell.absorption * ds).exp();
            light = light + cell.source * (absorbed * through);
            through *= 1.0 - absorbed;
            if through < BLOCKED {
                return (light, 0.0);
            }
        }
        t += ds;
    }
    (light, through)
}

/// Distance in pixels from each cell centre to the nearest absorbing cell centre (chamfer
/// 1/√2, two passes). Never less than the true distance, at most 8.24 % more.
fn distance_field(scene: &Scene) -> Vec<f32> {
    let (w, h) = (scene.width as usize, scene.height as usize);
    let far = (w + h) as f32 * 2.0;
    let mut d: Vec<f32> = scene
        .cells
        .iter()
        .map(|c| if c.absorption > 0.0 { 0.0 } else { far })
        .collect();
    let diag = std::f32::consts::SQRT_2;
    let mut relax = |i: usize, j: usize, cost: f32| {
        let v = d[j] + cost;
        if v < d[i] {
            d[i] = v;
        }
    };
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            if x > 0 {
                relax(i, i - 1, 1.0);
            }
            if y > 0 {
                relax(i, i - w, 1.0);
                if x > 0 {
                    relax(i, i - w - 1, diag);
                }
                if x + 1 < w {
                    relax(i, i - w + 1, diag);
                }
            }
        }
    }
    for y in (0..h).rev() {
        for x in (0..w).rev() {
            let i = y * w + x;
            if x + 1 < w {
                relax(i, i + 1, 1.0);
            }
            if y + 1 < h {
                relax(i, i + w, 1.0);
                if x + 1 < w {
                    relax(i, i + w + 1, diag);
                }
                if x > 0 {
                    relax(i, i + w - 1, diag);
                }
            }
        }
    }
    d
}

/// Fills `out[n] = f(n)`, one contiguous chunk per core.
fn par_fill<T: Send>(out: &mut [T], f: impl Fn(usize) -> T + Sync) {
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get());
    let chunk = out.len().div_ceil(threads).max(1);
    std::thread::scope(|s| {
        for (ci, part) in out.chunks_mut(chunk).enumerate() {
            let f = &f;
            s.spawn(move || {
                for (i, v) in part.iter_mut().enumerate() {
                    *v = f(ci * chunk + i);
                }
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const N: u32 = 64;
    const WARM: Vec3 = vec3(1.0, 0.6, 0.3);

    fn at(light: &[Vec3], x: u32, y: u32) -> Vec3 {
        light[(y * N + x) as usize]
    }

    fn lamp() -> Scene {
        let mut s = Scene::new(N, N);
        s.fill_disc(12.0, 32.0, 4.0, Cell::light(WARM));
        s
    }

    #[test]
    fn empty_scene_is_dark() {
        let light = solve(&Scene::new(N, N), &Cascades::default());
        assert_eq!(light.len(), (N * N) as usize);
        assert!(light.iter().all(|c| *c == Vec3::default()));
    }

    #[test]
    fn zero_sized_scene_gives_no_pixels() {
        assert!(solve(&Scene::new(0, 10), &Cascades::default()).is_empty());
        assert!(solve(&Scene::new(10, 0), &Cascades::default()).is_empty());
    }

    #[test]
    fn inside_an_opaque_light_the_mean_intensity_is_its_colour() {
        let light = solve(&lamp(), &Cascades::default());
        let c = at(&light, 12, 32);
        assert!((c - WARM).len() < 1e-3, "{c:?}");
    }

    #[test]
    fn light_falls_off_with_distance() {
        let light = solve(&lamp(), &Cascades::default());
        let mut last = f32::MAX;
        for x in [20, 26, 34, 44, 58] {
            let v = at(&light, x, 32).x;
            assert!(v > 0.0 && v < last, "x {x}: {v} after {last}");
            last = v;
        }
    }

    #[test]
    fn matches_brute_force_in_open_space() {
        let scene = lamp();
        let fast = solve(&scene, &Cascades::default());
        let slow = reference(&scene, 1024);
        let (mut err, mut total) = (0.0, 0.0);
        for y in 0..N {
            for x in 0..N {
                let d = (x as f32 + 0.5 - 12.0).hypot(y as f32 + 0.5 - 32.0);
                if d > 8.0 {
                    err += (at(&fast, x, y).x - at(&slow, x, y).x).abs();
                    total += at(&slow, x, y).x;
                }
            }
        }
        assert!(err / total < 0.08, "mean relative error {}", err / total);
    }

    #[test]
    fn a_wall_casts_a_shadow() {
        let open = solve(&lamp(), &Cascades::default());
        let mut walled = lamp();
        walled.fill_rect(28.0, 20.0, 32.0, 44.0, Cell::wall());
        let shaded = solve(&walled, &Cascades::default());
        let behind = at(&shaded, 44, 32).x;
        assert!(behind < 0.15 * at(&open, 44, 32).x, "{behind}");
        // past the wall's end most of the light gets through
        assert!(at(&shaded, 44, 2).x > 0.5 * at(&open, 44, 2).x);
        // in between, the lamp is partly hidden: a soft edge
        let edge = at(&shaded, 44, 6).x;
        assert!(
            edge > 0.2 * at(&open, 44, 6).x && edge < 0.8 * at(&open, 44, 6).x,
            "{edge}"
        );
        // brute force agrees, in the full shadow and in the soft edge
        let slow = reference(&walled, 1024);
        assert!((behind - at(&slow, 44, 32).x).abs() < 0.1 * at(&open, 44, 32).x);
        assert!((edge - at(&slow, 44, 6).x).abs() < 0.15 * at(&open, 44, 6).x);
    }

    #[test]
    fn the_face_towards_a_light_is_lit_and_the_far_face_is_not() {
        let mut s = lamp();
        s.fill_rect(28.0, 20.0, 32.0, 44.0, Cell::wall());
        let light = solve(&s, &Cascades::default());
        // the empty pixels touching the wall: in front of it, inside it (nothing gets in), behind it
        assert!(at(&light, 27, 32).x > 0.05);
        assert!(at(&light, 29, 32).x < 1e-4);
        assert!(at(&light, 32, 32).x < 1e-4);
    }

    #[test]
    fn glow_dims_less_than_a_wall() {
        let open = at(&solve(&lamp(), &Cascades::default()), 44, 32).x;
        let behind = |cell| {
            let mut s = lamp();
            s.fill_rect(28.0, 20.0, 32.0, 44.0, cell);
            at(&solve(&s, &Cascades::default()), 44, 32).x
        };
        let wall = behind(Cell::wall());
        let haze = behind(Cell::glow(vec3(0.0, 0.0, 0.0), 0.2));
        assert!(wall < haze && haze < open, "{wall} {haze} {open}");
    }

    #[test]
    fn glow_shines_in_its_own_colour() {
        let mut s = Scene::new(N, N);
        s.fill_disc(32.0, 32.0, 8.0, Cell::glow(vec3(0.0, 1.0, 0.0), 0.3));
        let c = at(&solve(&s, &Cascades::default()), 32, 32);
        assert!(c.y > 0.3 && c.x == 0.0 && c.z == 0.0, "{c:?}");
    }

    #[test]
    fn coarser_probes_still_light_the_scene() {
        let cascades = Cascades {
            spacing: 2,
            ..Cascades::default()
        };
        let coarse = solve(&lamp(), &cascades);
        let fine = solve(&lamp(), &Cascades::default());
        let (a, b) = (at(&coarse, 30, 32).x, at(&fine, 30, 32).x);
        assert!((a - b).abs() < 0.2 * b, "{a} vs {b}");
    }

    #[test]
    fn cascade_count_covers_the_diagonal() {
        let c = Cascades::default();
        for (w, h) in [(1, 1), (64, 64), (300, 20), (1080, 2400)] {
            let n = c.count(w, h);
            let diagonal = (w as f32).hypot(h as f32);
            assert!(interval_start(c.interval, n) >= diagonal, "{w}x{h}: {n}");
            assert!(
                n == 1 || interval_start(c.interval, n - 1) < diagonal,
                "{w}x{h}: {n}"
            );
        }
    }

    #[test]
    fn broken_settings_fall_back_to_working_ones() {
        let broken = Cascades {
            spacing: 0,
            directions: 0,
            interval: -1.0,
            step: f32::NAN,
        };
        let light = solve(&lamp(), &broken);
        assert!(light.iter().all(|c| c.x.is_finite() && c.y.is_finite()));
        assert!(at(&light, 30, 32).x > 0.0);
    }

    #[test]
    fn cells_outside_the_scene_are_ignored() {
        let mut s = Scene::new(4, 4);
        s.set(9, 9, Cell::wall());
        assert_eq!(s.cell(9, 9), Cell::EMPTY);
        s.fill_disc(-10.0, -10.0, 2.0, Cell::wall());
        assert!(s.cells.iter().all(|c| *c == Cell::EMPTY));
    }

    #[test]
    fn distance_field_is_never_shorter_than_the_true_distance() {
        let mut s = Scene::new(N, N);
        s.set(10, 20, Cell::wall());
        let d = distance_field(&s);
        for y in 0..N {
            for x in 0..N {
                let truth = (x as f32 - 10.0).hypot(y as f32 - 20.0);
                let got = d[(y * N + x) as usize];
                assert!(
                    got >= truth - 1e-4 && got <= truth * 1.0825 + 1e-4,
                    "({x},{y}) {got} {truth}"
                );
            }
        }
    }
}
