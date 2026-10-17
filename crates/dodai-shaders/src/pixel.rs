//! Pixel-art shaders: side-scrolling platformer worlds, metroidvania caves
//! and castles, sprites, plus palette-limiting post effects.
//!
//! Generators render on a small *virtual* canvas (144 px tall) and snap to
//! it, so every output pixel is a crisp art pixel whatever the window size.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Pixel;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "platformer",
        "Platformer world",
        F,
        "Scrolling hills, clouds, bricks, ? blocks, a pipe, coins and a running hero",
        platformer,
    ),
    ShaderInfo::generator(
        "metroid_cave",
        "Metroidvania cave",
        F,
        "Parallax brick caverns lit by flickering torches",
        metroid_cave,
    ),
    ShaderInfo::generator(
        "castle_hall",
        "Castle hall",
        F,
        "Gothic arches, stained glass and moon shafts",
        castle_hall,
    ),
    ShaderInfo::generator(
        "pixel_mountains",
        "Night mountains",
        F,
        "Four parallax ridge layers under stars and a moon",
        pixel_mountains,
    ),
    ShaderInfo::generator(
        "pixel_items",
        "Item sprites",
        F,
        "Heart, coin, key, mushroom, ghost and sword sprites bobbing in a row",
        pixel_items,
    ),
    ShaderInfo::post(
        "pixelate",
        "Pixelate",
        F,
        "Blocky low-resolution look",
        pixelate,
    ),
    ShaderInfo::post(
        "gameboy",
        "Game Boy",
        F,
        "Four green shades with ordered dither",
        gameboy,
    ),
    ShaderInfo::post(
        "pico16",
        "16-colour palette",
        F,
        "Snaps to a fixed 16-colour console palette with dither",
        pico16,
    ),
];

const VH: f32 = 144.0;

/// Virtual pixel coordinates (whole numbers, y down) and the canvas width.
fn vpix(uv: Vec2, c: &Ctx) -> (Vec2, f32) {
    let vw = (VH * c.aspect()).round();
    (vec2((uv.x * vw).floor(), (uv.y * VH).floor()), vw)
}

fn rgb(r: u8, g: u8, b: u8) -> Vec3 {
    Vec3::from_u8([r, g, b])
}

/// Character at `(x, y)` of a sprite given as rows of text; `None` outside.
pub fn sprite_at<const N: usize>(rows: &[&str; N], x: i32, y: i32) -> Option<u8> {
    if x < 0 || y < 0 || y as usize >= N {
        return None;
    }
    rows[y as usize]
        .as_bytes()
        .get(x as usize)
        .copied()
        .filter(|c| *c != b'.')
}

const HERO: [[&str; 16]; 3] = [
    [
        "....RRRRR...",
        "...RRRRRRRRR",
        "...BBBSSSBS.",
        "..BSBSSSBSSS",
        "..BSBBSSSBSS",
        "..BBSSSSBBBB",
        "....SSSSSSS.",
        "...RRBRRR...",
        "..RRRBRRBRRR",
        ".RRRRBBBBRRR",
        ".SSRBYBBYBRS",
        ".SSSBBBBBBSS",
        ".SSBBBBBBBSS",
        "...BBB..BBB.",
        "..BBB....BBB.",
        ".BBBB....BBBB",
    ],
    [
        "....RRRRR...",
        "...RRRRRRRRR",
        "...BBBSSSBS.",
        "..BSBSSSBSSS",
        "..BSBBSSSBSS",
        "..BBSSSSBBBB",
        "....SSSSSSS.",
        "....RRRBRR..",
        "...RRRRBRRR.",
        "...RRRBBBRRS",
        "..SSBBYBBYSS",
        "..SSBBBBBBS.",
        "...BBBBBBB..",
        "....BBBBB...",
        "....BB.BB...",
        "....BBB.BBB.",
    ],
    [
        "....RRRRR...",
        "...RRRRRRRRR",
        "...BBBSSSBS.",
        "..BSBSSSBSSS",
        "..BSBBSSSBSS",
        "..BBSSSSBBBB",
        "....SSSSSSS.",
        "..SRRRBRRRS.",
        ".SSRRRBRRRSS",
        ".SS.RRBBRR.S",
        "....BBYBBY..",
        "...BBBBBBBB.",
        "..BBB.BB.BBB",
        "..BB...BB.BB",
        "..B.....B..B",
        "............",
    ],
];

fn hero_color(ch: u8) -> Vec3 {
    match ch {
        b'R' => rgb(216, 40, 0),
        b'S' => rgb(252, 188, 176),
        b'B' => rgb(136, 80, 24),
        b'Y' => rgb(252, 216, 80),
        _ => Vec3::BLACK,
    }
}

const QBLOCK: [&str; 8] = [
    "..XXXX..", ".X....X.", "......X.", "....XX..", "...X....", "........", "...X....", "........",
];

pub fn platformer(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let scroll = (c.time * 36.0).floor();
    let wx = p.x + scroll;
    let ground = VH - 28.0;
    let sky = rgb(92, 148, 252);
    let mut col = sky;

    // clouds, parallax 0.25
    let cx = fmod(wx * 0.25 + 40.0, 150.0) - 75.0;
    let cy = 30.0 + 14.0 * hash1((((wx * 0.25 + 40.0) / 150.0).floor()) * 3.0);
    let blob = |ox: f32, oy: f32, r: f32| sd_circle(vec2(cx - ox, p.y - cy - oy), r);
    let cloud = blob(-9.0, 2.0, 6.0)
        .min(blob(0.0, -2.0, 8.0))
        .min(blob(10.0, 2.0, 6.0))
        .min(blob(0.0, 5.0, 8.0));
    if cloud < 0.0 && p.y < cy + 9.0 {
        col = if cloud > -2.0 || p.y > cy + 5.0 {
            rgb(176, 220, 252)
        } else {
            Vec3::WHITE
        };
    }

    // hills, parallax 0.5
    let hx = fmod(wx * 0.5, 220.0) - 110.0;
    let hh = 46.0 * (1.0 - (hx / 100.0).powi(2)).max(0.0).sqrt();
    let hill_top = ground - hh;
    if p.y >= hill_top && hh > 0.0 {
        col = if p.y < hill_top + 2.0 {
            rgb(0, 100, 0)
        } else {
            rgb(0, 168, 0)
        };
        // eyes-dot texture
        if (p.y as i32 + (wx * 0.5) as i32 / 3) % 9 == 0 && (wx as i32) % 7 == 0 {
            col = rgb(0, 120, 0);
        }
    }
    // bush
    let bx = fmod(wx + 60.0, 190.0) - 95.0;
    let bush = (1.0 - (bx / 22.0).powi(2)).max(0.0).sqrt() * 12.0;
    if p.y >= ground - bush && p.y < ground && bush > 0.0 {
        col = if p.y < ground - bush + 2.0 {
            rgb(124, 252, 0)
        } else {
            rgb(0, 190, 0)
        };
    }

    // pipe
    let px_ = fmod(wx + 20.0, 330.0);
    if (130.0..162.0).contains(&px_) && p.y >= ground - 34.0 && p.y < ground {
        let lip = p.y < ground - 34.0 + 12.0;
        let (x0, x1) = if lip { (130.0, 162.0) } else { (132.0, 160.0) };
        if px_ >= x0 && px_ < x1 {
            let edge = px_ - x0 < 3.0
                || p.y == (ground - 34.0).floor()
                || (lip && p.y == (ground - 22.0).floor());
            let hi = px_ - x0 > 6.0 && px_ - x0 < 9.0;
            col = if edge {
                rgb(0, 40, 0)
            } else if hi {
                rgb(172, 252, 124)
            } else {
                rgb(0, 168, 0)
            };
        }
    }

    // floating ? / brick row
    let bt = 16.0;
    let by = ground - 62.0;
    let row_x = fmod(wx, 256.0);
    if p.y >= by && p.y < by + bt && (96.0..176.0).contains(&row_x) {
        let tile = ((row_x - 96.0) / bt).floor() as i32;
        let (lx, ly) = (fmod(row_x, bt), p.y - by);
        let question = tile % 2 == 1;
        let bounce = if question && ((c.time * 1.5).floor() as i32 + tile) % 4 == 0 {
            -(fract(c.time * 1.5) * std::f32::consts::PI).sin() * 6.0
        } else {
            0.0
        };
        let ly = ly - bounce;
        if (0.0..bt).contains(&ly) {
            if question {
                let border = lx < 1.0 || lx > 14.0 || ly < 1.0 || ly > 14.0;
                let ch = sprite_at(&QBLOCK, (lx / 2.0) as i32, (ly / 2.0) as i32);
                col = if border || ch.is_some() {
                    rgb(136, 80, 24)
                } else {
                    rgb(252, 188, 60)
                };
            } else {
                let mortar =
                    ly % 8.0 < 1.0 || fmod(lx + if ly >= 8.0 { 4.0 } else { 0.0 }, 8.0) < 1.0;
                col = if mortar {
                    rgb(60, 20, 0)
                } else {
                    rgb(200, 76, 12)
                };
            }
        }
        // coin above the ? blocks
        if question {
            let phase = fract(c.time * 1.5 + tile as f32 * 0.25);
            let cy = by - 8.0 - phase * 12.0;
            let w = (c.time * 10.0).sin().abs() * 3.5 + 0.8;
            let ccx = 96.0 + (tile as f32) * bt + 8.0;
            if ((row_x - ccx) / w).powi(2) + ((p.y - cy) / 5.0).powi(2) < 1.0 && phase < 0.8 {
                col = rgb(252, 216, 80);
            }
        }
    }

    // ground
    if p.y >= ground {
        let ly = p.y - ground;
        let row = (ly / 8.0).floor();
        let mortar =
            ly % 8.0 < 1.0 || fmod(wx + if row as i32 % 2 == 1 { 8.0 } else { 0.0 }, 16.0) < 1.0;
        col = if mortar {
            rgb(60, 20, 0)
        } else if ly < 1.0 {
            rgb(252, 160, 68)
        } else {
            rgb(200, 76, 12)
        };
    }

    // hero
    let hero_x = (vw * 0.28).floor();
    let t_jump = c.time % 2.6;
    let jump = if t_jump < 0.7 {
        (t_jump / 0.7 * std::f32::consts::PI).sin() * 34.0
    } else {
        0.0
    };
    let frame = if jump > 0.5 {
        2
    } else {
        ((c.time * 8.0) as usize) % 2
    };
    let hx_ = p.x - hero_x;
    let hy_ = p.y - (ground - 16.0 - jump.floor());
    if let Some(ch) = sprite_at(&HERO[frame], hx_ as i32, hy_ as i32) {
        col = hero_color(ch);
    }
    col
}

pub fn metroid_cave(uv: Vec2, c: &Ctx) -> Vec3 {
    let pal = &c.pal;
    let (p, _) = vpix(uv, c);
    let t = c.time;
    let tile = 8.0;

    // torches: world positions, scroll slow
    let scroll = (t * 10.0).floor();
    let mut light = 0.22;
    let period = 120.0;
    let wx = p.x + scroll;
    let ti = ((wx + period / 2.0) / period).floor();
    let mut torch_px = None;
    for k in -1..=1 {
        let tx = (ti + k as f32) * period + 0.0 - scroll;
        let ty = VH * 0.38;
        let fl = 0.85 + 0.15 * (t * 11.0 + tx * 0.7).sin() * (t * 7.3 + tx).cos();
        let d = (p - vec2(tx, ty)).len();
        light += (1.0 - d / (110.0 * fl)).max(0.0).powi(2) * 1.2;
        if k == 0 {
            torch_px = Some((vec2(tx, ty), fl));
        }
    }
    // posterised light, retro banding
    let lit = (sat(light) * 5.0).floor() / 5.0;

    // layers: far bricks (dim), near wall bricks
    let layer = |par: f32, scale: f32| {
        let x = p.x + (t * 10.0 * par).floor();
        let ty = (p.y / tile).floor();
        let off = if (ty as i32) % 2 == 0 {
            0.0
        } else {
            tile / 2.0
        };
        let bx = ((x + off) / (tile * 2.0)).floor();
        let (lx, ly) = (fmod(x + off, tile * 2.0), fmod(p.y, tile));
        let mortar = lx < 1.0 || ly < 1.0;
        let h = hash2(bx as i32, ty as i32);
        let base = pal.a.mix(pal.bg, 0.35 - 0.1 * scale) * (0.8 + 0.5 * h);
        if mortar {
            pal.bg * 0.5
        } else {
            base
        }
    };
    let mut col;
    // central cavern opening: darker, with near-layer bricks at the edges
    let cave_y = VH * 0.18
        + 18.0 * ((p.x + scroll * 0.6) * 0.03).sin()
        + 9.0 * ((p.x + scroll * 0.6) * 0.09).sin();
    let floor_y = VH * 0.8 + 6.0 * (((p.x + scroll) / 16.0).floor() * 1.3).sin().round();
    if p.y > cave_y && p.y < floor_y {
        col = layer(0.6, 1.0) * 0.7;
        // arches hint: dark recesses
        let ax = fmod(wx, period) - period / 2.0;
        if ax.abs() < 30.0 && p.y > VH * 0.5 && p.y < floor_y - 6.0 {
            col = col * 0.4;
        }
    } else {
        col = layer(1.0, 2.0);
    }
    // platforms
    let plat_y = VH * 0.62;
    let px_ = fmod(wx + 40.0, 160.0);
    if (24.0..88.0).contains(&px_) && p.y >= plat_y && p.y < plat_y + 6.0 {
        col = if p.y < plat_y + 2.0 {
            pal.b
        } else {
            pal.a * 0.55
        };
        col = col * (0.8 + 0.2 * hash2((px_ / 4.0) as i32, p.y as i32));
    }
    // moss on floor edge
    if p.y >= floor_y && p.y < floor_y + 3.0 {
        col = pal.b * (0.7 + 0.3 * hash2(p.x as i32, 4));
    }
    // chains
    let chain_x = fmod(wx + 13.0, 53.0);
    if chain_x < 2.0
        && p.y < cave_y + 22.0 + 6.0 * hash1(((wx + 13.0) / 53.0).floor())
        && (p.y as i32) % 4 < 3
    {
        col = pal.fg * 0.35;
    }
    col = col * (0.4 + 0.9 * lit)
        + pal.c * 0.35 * (lit - 0.2).max(0.0).powi(2) * hash2(p.x as i32, p.y as i32).max(0.6);

    // torch sprite + flame
    if let Some((tp, fl)) = torch_px {
        let d = p - tp;
        if d.x.abs() < 2.0 && d.y >= 0.0 && d.y < 10.0 {
            col = rgb(90, 56, 28);
        }
        let flame = ((d.x / 3.2).powi(2)
            + ((d.y + 5.0 + 3.0 * (t * 12.0).sin()) / (6.0 * fl)).powi(2))
            < 1.0;
        if flame && d.y < 1.0 {
            col = if d.y < -5.0 {
                rgb(255, 240, 150)
            } else {
                pal.c
            };
        }
    }
    // fog bands near the floor
    let fog = fbm(vec2((p.x + t * 6.0) * 0.03, p.y * 0.12), 3);
    col.mix(
        pal.fg * 0.6,
        smoothstep(floor_y - 24.0, floor_y, p.y) * fog * 0.35 * lit.max(0.25),
    )
}

pub fn castle_hall(uv: Vec2, c: &Ctx) -> Vec3 {
    let pal = &c.pal;
    let (p, _) = vpix(uv, c);
    let t = c.time;
    let scroll = (t * 6.0).floor();
    let wx = p.x + scroll;
    let bay = 96.0;
    let lx = fmod(wx, bay) - bay / 2.0;
    let bi = (wx / bay).floor() as i32;
    let floor_y = VH - 22.0;
    let mut col = pal.bg.mix(pal.a, 0.18) * (0.7 + 0.3 * (1.0 - p.y / VH));
    // brick back wall texture
    let brick = (p.y / 6.0).floor();
    let off = if brick as i32 % 2 == 0 { 0.0 } else { 6.0 };
    if p.y % 6.0 < 1.0 || fmod(wx + off, 12.0) < 1.0 {
        col = col * 0.7;
    }
    // arch window (stained glass)
    let wy = floor_y - 54.0;
    let win_w = 22.0;
    let arch_d = {
        let q = vec2(lx, p.y - wy);
        let rect = sd_box(q - vec2(0.0, 18.0), vec2(win_w, 22.0));
        let top = sd_circle(q, win_w);
        rect.min(top)
    };
    if arch_d < 0.0 {
        let cell = vec2((lx / 7.0).floor(), ((p.y - wy) / 7.0).floor());
        let h = hash3(cell.x as i32, cell.y as i32, bi);
        let tone = if h < 0.33 {
            pal.a
        } else if h < 0.66 {
            pal.b
        } else {
            pal.c
        };
        let lead = fmod(lx, 7.0) < 1.0 || fmod(p.y - wy, 7.0) < 1.0;
        col = if lead {
            pal.bg * 0.3
        } else {
            tone * (0.55 + 0.45 * (t * 0.8 + h * 6.0).sin().abs())
        };
    } else if arch_d < 2.0 {
        col = pal.fg * 0.3;
    }
    // pillars at bay edges
    let pe = fmod(wx + bay / 2.0, bay);
    if pe < 10.0 || pe > bay - 10.0 {
        let shade = if pe < 3.0 || pe > bay - 3.0 {
            0.45
        } else {
            0.7
        };
        col = pal.a.mix(pal.bg, 0.55) * shade * (0.8 + 0.2 * hash2(p.y as i32 / 4, 1));
        if p.y > floor_y - 6.0 || p.y < 10.0 {
            col = col * 1.3;
        }
    }
    // moon shafts, diagonal
    let shaft = ((p.x + p.y * 0.8 + scroll * 0.0) / 14.0 + t * 0.05).sin();
    let sh = smoothstep(0.55, 0.95, shaft) * step(p.y, floor_y) * 0.18;
    col = col + pal.fg * sh * vec3(0.7, 0.8, 1.0);
    // floor: checker
    if p.y >= floor_y {
        let chk = ((fmod(wx, 16.0) < 8.0) as i32 + ((p.y - floor_y) / 8.0) as i32) % 2 == 0;
        col = if chk {
            pal.a.mix(pal.bg, 0.6)
        } else {
            pal.bg * 1.6
        };
        if p.y - floor_y < 1.0 {
            col = pal.fg * 0.5;
        }
    }
    // dust motes
    let mote = hash3((wx / 5.0) as i32, (p.y / 5.0) as i32, (t * 1.2) as i32);
    if mote > 0.996 {
        col = col + pal.fg * 0.5;
    }
    col
}

pub fn pixel_mountains(uv: Vec2, c: &Ctx) -> Vec3 {
    let pal = &c.pal;
    let (p, _) = vpix(uv, c);
    let t = c.time;
    let mut col = pal
        .bg
        .mix(pal.a * 0.5, smoothstep(0.0, 1.0, p.y / VH).powi(2));
    // stars
    let sh = hash2(p.x as i32, p.y as i32);
    if sh > 0.994 && p.y < VH * 0.55 {
        col = col + pal.fg * (0.4 + 0.6 * (t * 3.0 + sh * 80.0).sin().abs());
    }
    // moon
    let moon = vec2(VH * 1.15, 28.0);
    let d = (p - moon).len();
    if d < 12.0 {
        col = pal.fg
            * if (p - moon - vec2(3.0, -2.0)).len() < 10.0 {
                0.9
            } else {
                1.0
            };
        if (p - moon - vec2(-3.0, 2.0)).len() < 2.5 {
            col = pal.fg * 0.75;
        }
    }
    // ridges
    for i in 0..4 {
        let f = i as f32;
        let speed = 3.0 + f * 6.0;
        let x = (p.x + (t * speed).floor()) * (0.012 + 0.006 * f);
        let h = VH * (0.45 + f * 0.1) - 46.0 * fbm(vec2(x, f * 7.3), 4);
        if p.y >= h.floor() {
            let tone = pal.bg.mix(pal.a, 0.25 + 0.15 * f).mix(pal.b, 0.1 * f);
            col = tone * (0.35 + 0.22 * f);
            if p.y < h.floor() + 1.0 {
                col = tone * (0.7 + 0.1 * f);
            }
            // pines on the closest layer
            if i == 3 {
                let tx = fmod(p.x + (t * speed).floor(), 14.0) - 7.0;
                let tree = (p.y - (h.floor() - 14.0)) / 14.0;
                if (0.0..1.0).contains(&tree) && tx.abs() < tree * 5.0 {
                    col = pal.bg * 0.5;
                }
            }
        }
    }
    col
}

const ITEMS: [[&str; 8]; 6] = [
    [
        "..RR.RR.", ".RRRRRRR", ".RRWRRRR", ".RRRRRRR", "..RRRRR.", "...RRR..", "....R...",
        "........",
    ],
    [
        "..YYYY..", ".YWYYYY.", "YYWYYOYY", "YYWYYOYY", "YYWYYOYY", "YYYYYOYY", ".YYYOOY.",
        "..YYYY..",
    ],
    [
        "....YY..", "...YYYY.", "...YYYY.", "....YY..", "....Y...", "...YY...", "....Y...",
        "...YY...",
    ],
    [
        "..RRRR..", ".RWWRRR.", "RRWWRRWR", "RRRRRRRR", "..WWWW..", "..WSSW..", "..WSSW..",
        "..WWWW..",
    ],
    [
        "..WWWW..", ".WWWWWW.", "WWKWWKWW", "WWKWWKWW", "WWWWWWWW", "WWWWWWWW", "WW.WW.WW",
        "W..W..W.",
    ],
    [
        "......WW",
        ".....WW.",
        "....WW..",
        "WW.WW...",
        ".WWW....",
        "..W.....",
        ".B.W....",
        "B........",
    ],
];

fn item_color(ch: u8, pal: &crate::ShaderPalette) -> Vec3 {
    match ch {
        b'R' => rgb(228, 52, 52),
        b'W' => Vec3::WHITE,
        b'Y' => rgb(252, 208, 60),
        b'O' => rgb(200, 130, 20),
        b'S' => rgb(130, 90, 60),
        b'K' => rgb(20, 20, 30),
        b'B' => rgb(140, 90, 40),
        _ => pal.fg,
    }
}

pub fn pixel_items(uv: Vec2, c: &Ctx) -> Vec3 {
    let pal = &c.pal;
    let (p, vw) = vpix(uv, c);
    let mut col = pal.bg.mix(pal.a, 0.12 + 0.1 * (p.y / VH));
    // checker background
    if ((p.x / 8.0).floor() + (p.y / 8.0).floor()) as i32 % 2 == 0 {
        col = col * 0.92;
    }
    let n = 6;
    let scale = 4.0;
    let slot = vw / n as f32;
    let i = ((p.x / slot) as usize).min(n - 1);
    let cx = slot * (i as f32 + 0.5);
    let bob = ((c.time * 3.0 + i as f32 * 0.9).sin() * 3.0).round();
    let ox = ((p.x - cx) / scale + 4.0).floor();
    let oy = ((p.y - VH * 0.5 - bob) / scale + 4.0).floor();
    if let Some(ch) = sprite_at(&ITEMS[i], ox as i32, oy as i32) {
        col = item_color(ch, pal);
    }
    // shadow
    let sh = ((p.x - cx) / 14.0).powi(2) + ((p.y - (VH * 0.5 + 22.0)) / (3.0 - bob * 0.1)).powi(2);
    if sh < 1.0 {
        col = col * 0.7;
    }
    col
}

// ───────────────────────────── post effects ─────────────────────────────

pub fn pixelate(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let block = 2.0 + 6.0 * c.intensity;
    let (w, h) = (src.w as f32, src.h as f32);
    let u = vec2(
        ((uv.x * w / block).floor() + 0.5) * block / w,
        ((uv.y * h / block).floor() + 0.5) * block / h,
    );
    src.sample(u)
}

pub fn gameboy(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    const SHADES: [[u8; 3]; 4] = [[15, 56, 15], [48, 98, 48], [139, 172, 15], [155, 188, 15]];
    let block = 2.0;
    let (w, h) = (src.w as f32, src.h as f32);
    let (bx, by) = ((uv.x * w / block).floor(), (uv.y * h / block).floor());
    let s = src.sample(vec2((bx + 0.5) * block / w, (by + 0.5) * block / h));
    let l = sat(s.luma().powf(0.8) * 1.15);
    let d = bayer4(bx as i32, by as i32) - 0.5;
    let q = ((l * 3.0 + d * 0.9 * c.intensity).round().clamp(0.0, 3.0)) as usize;
    Vec3::from_u8(SHADES[q])
}

const P16: [[u8; 3]; 16] = [
    [0, 0, 0],
    [29, 43, 83],
    [126, 37, 83],
    [0, 135, 81],
    [171, 82, 54],
    [95, 87, 79],
    [194, 195, 199],
    [255, 241, 232],
    [255, 0, 77],
    [255, 163, 0],
    [255, 236, 39],
    [0, 228, 54],
    [41, 173, 255],
    [131, 118, 156],
    [255, 119, 168],
    [255, 204, 170],
];

pub fn pico16(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (w, h) = (src.w as f32, src.h as f32);
    let (x, y) = ((uv.x * w) as i32, (uv.y * h) as i32);
    let s = src.sample(uv);
    let d = (bayer4(x, y) - 0.5) * 0.18 * c.intensity;
    let t = s + Vec3::splat(d);
    let mut best = (f32::MAX, P16[0]);
    for pc in P16 {
        let v = Vec3::from_u8(pc);
        let dist = (v.powf(2.2) - t.clamp01().powf(2.2)).len();
        if dist < best.0 {
            best = (dist, pc);
        }
    }
    Vec3::from_u8(best.1)
}
