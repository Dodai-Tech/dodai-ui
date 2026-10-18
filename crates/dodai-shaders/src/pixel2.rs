//! More pixel art: sunset sea, rainy city, pine forest, space shooter, torch-lit
//! dungeon, campfire, snow village and an underwater reef, plus outline,
//! posterize and duotone post effects. Everything snaps to the 144-line virtual
//! canvas, uses fixed hand-picked colours and Bayer dithering.

use crate::frame::{Ctx, Frame};
use crate::math::*;
use crate::pixel::{spr, sprite_at, vpix, VH};
#[allow(unused_imports)]
use crate::prelude::*;
use crate::{Family, ShaderInfo};

const F: Family = Family::Pixel;

pub const SHADERS: &[ShaderInfo] = &[
    ShaderInfo::generator(
        "px_ocean",
        "Sunset sea",
        F,
        "Striped retro sun, dithered sky, glittering sea and gulls",
        px_ocean,
    ),
    ShaderInfo::generator(
        "px_city",
        "Rainy skyline",
        F,
        "Three parallax skyline layers, blinking windows, moon and neon rain",
        px_city,
    ),
    ShaderInfo::generator(
        "px_forest",
        "Pine forest",
        F,
        "Scrolling pine layers at dawn with mist and fireflies",
        px_forest,
    ),
    ShaderInfo::generator(
        "px_space",
        "Space shooter",
        F,
        "Starfield, ringed planet, a ship firing at aliens",
        px_space,
    ),
    ShaderInfo::generator(
        "px_dungeon",
        "Torch dungeon",
        F,
        "Brick walls, tiles, flickering torch light, a knight and slimes",
        px_dungeon,
    ),
    ShaderInfo::generator(
        "px_campfire",
        "Campfire",
        F,
        "Night camp: pixel flames, sparks, tent and pines in warm light",
        px_campfire,
    ),
    ShaderInfo::generator(
        "px_snow",
        "Snow village",
        F,
        "Cosy lit houses, smoke, mountains and layered snowfall",
        px_snow,
    ),
    ShaderInfo::generator(
        "px_reef",
        "Underwater reef",
        F,
        "Caustics, light shafts, swaying weed, fish and bubbles",
        px_reef,
    ),
    ShaderInfo::post(
        "px_outline",
        "Pixel outline",
        F,
        "Dark outline on the pixel grid where a bright shape meets a darker one",
        px_outline,
    ),
    ShaderInfo::post(
        "px_posterize",
        "Posterize",
        F,
        "A few colour levels per channel with ordered dither",
        px_posterize,
    ),
    ShaderInfo::post(
        "px_duotone",
        "Palette ramp",
        F,
        "Maps brightness onto the theme's colours as a dithered 4-step ramp",
        px_duotone,
    ),
];

fn rgb(r: u32, g: u32, b: u32) -> Vec3 {
    rgb8(r, g, b)
}

/// 1 where `v` beats the Bayer threshold at this pixel.
fn dith(x: f32, y: f32, v: f32) -> f32 {
    step(bayer4(x as i32, y as i32), v)
}

fn pick4(v: f32, x: f32, y: f32, a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> Vec3 {
    let k = (v * 3.0 + bayer4(x as i32, y as i32))
        .floor()
        .clamp(0.0, 3.0);
    if k < 0.5 {
        a
    } else if k < 1.5 {
        b
    } else if k < 2.5 {
        c
    } else {
        d
    }
}

fn sc(ch: u32) -> Vec3 {
    if ch == b'R' as u32 {
        rgb(222, 60, 70)
    } else if ch == b'O' as u32 {
        rgb(255, 150, 50)
    } else if ch == b'Y' as u32 {
        rgb(255, 224, 90)
    } else if ch == b'G' as u32 {
        rgb(90, 200, 90)
    } else if ch == b'g' as u32 {
        rgb(140, 150, 170)
    } else if ch == b'B' as u32 {
        rgb(70, 120, 230)
    } else if ch == b'C' as u32 {
        rgb(110, 230, 240)
    } else if ch == b'W' as u32 {
        rgb(244, 246, 250)
    } else if ch == b'K' as u32 {
        rgb(24, 22, 36)
    } else if ch == b'P' as u32 {
        rgb(160, 80, 200)
    } else if ch == b'S' as u32 {
        rgb(246, 200, 160)
    } else {
        rgb(120, 76, 50)
    }
}

/// Sprite colour at world pixel `(x, y)` for a sprite whose top-left is `(ox, oy)`.
fn blit<const W: usize, const H: usize>(
    rows: [[u32; W]; H],
    x: f32,
    y: f32,
    ox: f32,
    oy: f32,
    flip: bool,
) -> Option<Vec3> {
    let mut sx = (x - ox.floor()) as i32;
    let sy = (y - oy.floor()) as i32;
    if flip {
        sx = W as i32 - 1 - sx;
    }
    sprite_at(rows, sx, sy).map(sc)
}

/// 1 inside a stylised pine (tiered triangle plus trunk).
fn pine(x: f32, y: f32, cx: f32, base: f32, ht: f32) -> f32 {
    let k = y - (base - ht);
    let tier = (k / 8.0).floor();
    let within = k - tier * 8.0;
    let hw = 1.0 + tier * 2.4 + within * 0.45;
    let dx = (x - cx).abs();
    let leaf = step(0.0, k) * step(k, ht - 4.0) * step(dx, hw);
    let trunk = step(base - 5.0, y) * step(y, base) * step(dx, 1.5);
    leaf.max(trunk)
}

const SHIP: [[u32; 13]; 7] = spr([
    "......CC.....",
    ".....CCCg....",
    "RRgggWWWWgg..",
    "RRRgWWWWWWWgg",
    "RRgggWWWWgg..",
    ".....CCCg....",
    "......CC.....",
]);

const ALIEN: [[u32; 9]; 7] = spr([
    "..G...G..",
    "...G.G...",
    "..GGGGG..",
    ".GG.G.GG.",
    "GGGGGGGGG",
    "G.GGGGG.G",
    "G.G...G.G",
]);

const KNIGHT: [[u32; 8]; 10] = spr([
    "..gggg..", ".gWWWWg.", ".gWKKWg.", ".gWWWWg.", "..RRRR..", ".RRRRRR.", ".SRRRRS.", "..BBBB..",
    "..B..B..", "..K..K..",
]);
const KNIGHT2: [[u32; 8]; 10] = spr([
    "..gggg..", ".gWWWWg.", ".gWKKWg.", ".gWWWWg.", "..RRRR..", ".RRRRRR.", ".SRRRRS.", "..BBBB..",
    ".B....B.", ".K....K.",
]);

const SLIME: [[u32; 9]; 6] = spr([
    "...GGG...",
    "..GGGGG..",
    ".GGWGWGG.",
    "GGGGGGGGG",
    "GGGGGGGGG",
    ".GGGGGGG.",
]);

const FISH: [[u32; 8]; 5] = spr(["...OO..O", "..OOOOOO", ".OKOOOO.", "..OOOOOO", "...OO..O"]);

const FISH2: [[u32; 8]; 5] = spr(["...CC..C", "..CCCCCC", ".CKCCCC.", "..CCCCCC", "...CC..C"]);

// ------------------------------------------------------------------ scenes --

pub fn px_ocean(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let yh = 84.0;
    let sc_ = vec2(vw * 0.5, yh - 10.0);
    let mut col;
    if y < yh {
        let idx = ((y / yh) * 5.0 + (bayer4(x as i32, y as i32) - 0.5) * 0.9)
            .floor()
            .clamp(0.0, 5.0);
        col = match idx as u32 {
            0 => rgb(38, 22, 82),
            1 => rgb(94, 38, 110),
            2 => rgb(170, 52, 112),
            3 => rgb(232, 92, 92),
            4 => rgb(255, 150, 80),
            _ => rgb(255, 206, 120),
        };
        // cloud streaks, lit on their lower edge
        let n = noise(vec2(x * 0.04 + t * 0.3 + y * 0.01, y * 0.55));
        let n2 = noise(vec2(
            x * 0.04 + t * 0.3 + (y + 2.0) * 0.01,
            (y + 2.0) * 0.55,
        ));
        let band = step(14.0, y) * step(y, 62.0);
        let cl = step(0.64, n) * band;
        let lit = cl * step(n2, 0.64);
        col = if lit > 0.5 {
            rgb(255, 170, 110)
        } else if cl > 0.5 {
            rgb(80, 34, 104)
        } else {
            col
        };
        // retro sun
        let d = (p - sc_).len();
        let rel = y - (sc_.y - 4.0);
        let hide = step(0.0, rel) * step(rel % 5.0, rel * 0.2);
        if d < 24.0 && hide < 0.5 {
            col = pick4(
                1.0 - (y - (sc_.y - 24.0)) / 48.0,
                x,
                y,
                rgb(240, 80, 110),
                rgb(255, 130, 80),
                rgb(255, 190, 90),
                rgb(255, 236, 140),
            );
        } else if d < 29.0 {
            col = col.mix(
                rgb(255, 190, 120),
                dith(x, y, 0.35 * (1.0 - (d - 24.0) / 5.0)) * 0.6,
            );
        }
        // gulls
        for i in 0..3 {
            let fi = i as f32;
            let bx = (t * 9.0 + fi * 73.0) % (vw + 24.0) - 12.0;
            let by = 20.0 + fi * 9.0 + (t * 0.8 + fi).sin() * 3.0;
            let rx = x - bx.floor();
            let ry = y - by.floor();
            let flap = if (t * 5.0 + fi * 2.0).sin() > 0.0 {
                1.0
            } else {
                -1.0
            };
            if rx.abs() <= 3.0 && ry == (rx.abs() * 0.6).floor() * flap {
                col = rgb(34, 16, 56);
            }
        }
    } else {
        let d = y - yh;
        col = pick4(
            d / 60.0,
            x,
            y,
            rgb(22, 30, 92),
            rgb(30, 52, 120),
            rgb(44, 88, 150),
            rgb(60, 120, 170),
        );
        let dx = (x - sc_.x).abs();
        let w = 3.0 + d * 0.55;
        let hsh = hash2(
            (x / 2.0).floor() as i32,
            y as i32 + (t * 2.0).floor() as i32,
        );
        if dx < w && hsh > 0.45 + 0.3 * dx / w {
            col = pick4(
                1.0 - d / 50.0,
                x,
                y,
                rgb(240, 100, 100),
                rgb(255, 150, 80),
                rgb(255, 200, 100),
                rgb(255, 238, 150),
            );
        }
        let ph = (x * 0.22 + t * 1.2 + d * 0.9 + (d * 0.4).sin() * 2.0).sin();
        if ph > 0.93 && hash2(y as i32, (x / 6.0).floor() as i32) > 0.4 {
            col = rgb(140, 190, 220);
        }
        if y == yh {
            col = rgb(255, 190, 120);
        }
    }
    col
}

pub fn px_city(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let yh = 118.0;
    let idx = ((y / yh) * 4.0 + (bayer4(x as i32, y as i32) - 0.5) * 0.9)
        .floor()
        .clamp(0.0, 4.0);
    let mut col = match idx as u32 {
        0 => rgb(6, 8, 28),
        1 => rgb(14, 18, 52),
        2 => rgb(30, 30, 84),
        3 => rgb(70, 50, 110),
        _ => rgb(150, 70, 120),
    };
    let sh = hash2(x as i32, y as i32);
    if sh > 0.993 && y < 70.0 && (t * 3.0 + sh * 500.0).sin() > -0.3 {
        col = rgb(220, 224, 255);
    }
    // moon
    let m = vec2(vw * 0.78, 28.0);
    let dm = (p - m).len();
    let cut = (p - (m + vec2(5.0, -3.0))).len();
    if dm < 11.0 && cut > 9.0 {
        col = rgb(250, 240, 200);
        if (p - (m + vec2(-4.0, 3.0))).len() < 2.5 {
            col = rgb(214, 200, 160);
        }
    } else if dm < 15.0 {
        col = col.mix(
            rgb(210, 190, 220),
            dith(x, y, 0.3 * (1.0 - (dm - 11.0) / 4.0)) * 0.5,
        );
    }
    for l in 0..3 {
        let fl = l as f32;
        let bw = 10.0 + fl * 5.0;
        let xs = x + t * (2.0 + fl * 3.0) + fl * 41.0;
        let cell = (xs / bw).floor();
        let ci = cell as i32;
        let h1 = hash2(ci, l * 7 + 3);
        let top = yh - (30.0 + h1 * (36.0 + fl * 14.0)) * (0.8 + fl * 0.2);
        let wx = xs - cell * bw;
        if wx < bw - 1.0 && y >= top {
            col = match l {
                0 => rgb(22, 26, 66),
                1 => rgb(14, 16, 46),
                _ => rgb(8, 9, 26),
            };
            if l >= 1 {
                let wy = y - top;
                let on = step(1.0, wx % 3.0)
                    * step(wx % 3.0, 1.5)
                    * step(3.0, wy)
                    * step(1.5, wx)
                    * step(wy % 4.0, 2.5)
                    * step(1.5, wy % 4.0);
                let hw = hash2(
                    ci * 13 + (wx / 3.0).floor() as i32,
                    (wy / 4.0).floor() as i32 + l * 50,
                );
                let blink = if hw > 0.93 {
                    step(0.0, (t * 2.0 + hw * 99.0).sin())
                } else {
                    1.0
                };
                if on > 0.5 && hw > 0.45 && blink > 0.5 {
                    col = if hw > 0.8 {
                        rgb(110, 230, 255)
                    } else {
                        rgb(255, 214, 110)
                    };
                }
            }
        }
        // antenna with a red lamp
        if h1 > 0.7 && (wx - bw * 0.5).floor() == 0.0 && y >= top - 9.0 && y < top {
            col = rgb(8, 9, 26);
            if y == (top - 9.0).floor() && (t * 1.5 + h1 * 9.0).sin() > 0.0 {
                col = rgb(255, 50, 60);
            }
        }
    }
    if y >= 130.0 {
        col = rgb(10, 10, 22);
        if y == 136.0 && (x + t * 30.0) % 16.0 < 8.0 {
            col = rgb(255, 60, 160);
        }
        if y > 136.0 && hash2(x as i32, y as i32) > 0.97 {
            col = rgb(40, 50, 90);
        }
    }
    let rn = hash2(x as i32, ((y - t * 80.0 - x * 7.0) / 5.0).floor() as i32);
    if rn > 0.985 {
        col = col.mix(rgb(150, 170, 230), 0.55);
    }
    col
}

pub fn px_forest(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let k = y / 120.0;
    let idx = (k * 4.0 + (bayer4(x as i32, y as i32) - 0.5) * 0.9)
        .floor()
        .clamp(0.0, 4.0);
    let mut col = match idx as u32 {
        0 => rgb(24, 40, 84),
        1 => rgb(56, 80, 128),
        2 => rgb(130, 120, 150),
        3 => rgb(240, 170, 140),
        _ => rgb(255, 216, 150),
    };
    let ds = (p - vec2(vw * 0.7, 76.0)).len();
    if ds < 8.0 {
        col = rgb(255, 236, 170);
    }
    for l in 0..3 {
        let fl = l as f32;
        let sp = 16.0 + fl * 4.0;
        let xs = x + t * (1.5 + fl * 2.5) + fl * 37.0;
        let cell = (xs / sp).floor();
        let h1 = hash2(cell as i32, l * 5 + 1);
        let cx = cell * sp + sp * 0.5 + (h1 - 0.5) * 4.0;
        let base = 96.0 + fl * 14.0;
        let ht = 36.0 + h1 * 20.0 + fl * 14.0;
        let gcol = match l {
            0 => rgb(44, 72, 96),
            1 => rgb(28, 56, 70),
            _ => rgb(12, 34, 42),
        };
        let lcol = match l {
            0 => rgb(50, 84, 112),
            1 => rgb(30, 64, 80),
            _ => rgb(14, 40, 50),
        };
        // mist behind this layer
        if y > base - 22.0 && y < base {
            col = col.mix(
                rgb(224, 206, 214),
                dith(x, y, 0.4 * (y - (base - 22.0)) / 22.0) * 0.55,
            );
        }
        if pine(xs, y, cx, base, ht) > 0.5 {
            col = lcol;
            let ky = y - (base - ht);
            let tier = (ky / 8.0).floor();
            let hw = 1.0 + tier * 2.4 + (ky - tier * 8.0) * 0.45;
            if xs - cx < -hw + 1.2 {
                col = lcol * 1.35;
            }
        }
        if y > base - 2.0 {
            col = gcol;
        }
    }
    for i in 0..12 {
        let fi = i as f32;
        let fx = hash1(fi * 3.7) * vw + (t * 0.7 + fi * 3.0).sin() * 8.0;
        let fy = 70.0 + hash1(fi * 9.1) * 50.0 + (t * 0.5 + fi).cos() * 6.0;
        if (t * 2.0 + fi * 5.0).sin() > 0.2 {
            if x == fx.floor() && y == fy.floor() {
                col = rgb(250, 250, 140);
            } else if (x - fx.floor()).abs() + (y - fy.floor()).abs() == 1.0 {
                col = col.mix(rgb(250, 250, 140), 0.25);
            }
        }
    }
    col
}

pub fn px_space(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let mut col = rgb(8, 6, 24).mix(rgb(24, 12, 52), dith(x, y, y / VH * 0.6));
    for l in 0..3 {
        let fl = l as f32;
        let xs = (x + t * (6.0 + fl * 14.0)).floor();
        let h = hash2(xs as i32 + l * 913, y as i32 + l * 71);
        if h > 0.988 {
            col = match l {
                0 => rgb(90, 90, 150),
                1 => rgb(170, 170, 220),
                _ => rgb(255, 255, 255),
            };
        }
    }
    // ringed planet
    let pc = vec2(vw * 0.78, 96.0);
    let dp = (p - pc).len();
    if dp < 30.0 {
        let lit = (p - pc).dot(vec2(-0.6, -0.7)) / 30.0 * 0.5 + 0.55;
        let stripes = ((y + (x * 0.1).sin() * 2.0) / 5.0).floor() % 3.0;
        let base = if stripes < 0.5 {
            rgb(150, 70, 90)
        } else if stripes < 1.5 {
            rgb(200, 110, 100)
        } else {
            rgb(120, 54, 90)
        };
        col = base
            * (0.35
                + 0.8
                    * (lit * 3.0 + bayer4(x as i32, y as i32))
                        .floor()
                        .clamp(0.0, 3.0)
                    / 3.0);
    }
    let rr = ((p.x - pc.x) / 1.0).powf(2.0) / 2400.0
        + ((p.y - pc.y - (p.x - pc.x) * 0.25) / 1.0).powf(2.0) / 40.0;
    if rr > 0.8 && rr < 1.15 && (dp > 30.0 || y > pc.y + (x - pc.x) * 0.25) {
        col = rgb(240, 200, 150);
    }
    // ship
    let sy = 70.0 + (t * 0.9).sin() * 18.0;
    let sx = vw * 0.16;
    // bullets
    for i in 0..4 {
        let fi = i as f32;
        let bx = sx + 12.0 + ((t * 70.0 + fi * 40.0) % 160.0);
        let by = sy + 3.0 + (((t * 0.9).sin() * 0.0) + 0.0);
        if y == by.floor() && x >= bx.floor() && x < bx.floor() + 4.0 {
            col = rgb(255, 240, 120);
        }
    }
    // aliens on sine paths, with a brief explosion as they cross the firing line
    for i in 0..3 {
        let fi = i as f32;
        let ph = (t * 0.22 + fi * 0.33) % 1.0;
        let ax = vw + 10.0 - ph * (vw * 0.8);
        let ay = 40.0 + fi * 28.0 + (t * 2.0 + fi * 2.0).sin() * 8.0;
        if ph < 0.9 {
            if let Some(cc) = blit(ALIEN, x, y, ax, ay, false) {
                col = cc;
            }
        } else {
            let e = (ph - 0.9) * 10.0;
            let d = (p - vec2(ax + 4.0, ay + 3.0)).len();
            if d < 3.0 + e * 9.0 && d > e * 6.0 - 2.0 {
                col = if hash2(x as i32, y as i32) > 0.5 {
                    rgb(255, 210, 80)
                } else {
                    rgb(240, 90, 50)
                };
            }
        }
    }
    if let Some(cc) = blit(SHIP, x, y, sx, sy, false) {
        col = cc;
    }
    // engine flame
    if y == (sy + 3.0).floor()
        && x < sx.floor()
        && x > sx.floor() - 3.0 - (t * 20.0).sin().abs() * 3.0
    {
        col = rgb(255, 160, 60);
    }
    col
}

pub fn px_dungeon(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let (tx, ty) = ((x / 12.0).floor(), (y / 12.0).floor());
    let mut col;
    if y < 30.0 {
        let row = (y / 6.0).floor();
        let bx = (x + (row % 2.0) * 6.0) % 12.0;
        col = rgb(96, 62, 74);
        let bh = hash2(((x + (row % 2.0) * 6.0) / 12.0).floor() as i32, row as i32);
        if bh > 0.5 {
            col = rgb(84, 54, 68);
        }
        if bx < 1.0 || y % 6.0 < 1.0 {
            col = rgb(44, 28, 40);
        }
        if y > 26.0 {
            col = col * 0.7;
        }
    } else {
        col = if (tx + ty) % 2.0 < 1.0 {
            rgb(62, 56, 80)
        } else {
            rgb(54, 48, 72)
        };
        if x % 12.0 < 1.0 || y % 12.0 < 1.0 {
            col = col * 0.82;
        }
        if hash2(x as i32, y as i32) > 0.992 {
            col = col * 0.7;
        }
    }
    // knight and slimes
    // the knight roams the whole room on a looping path, facing where he walks
    let kx = vw * 0.5 + (t * 0.45).sin() * vw * 0.38;
    let ky = 66.0 + ((t * 0.31 + 1.0).sin() * 0.5 + 0.5) * 52.0;
    let step = (t * 5.0) % 2.0 < 1.0;
    let kb = if step { 0.0 } else { -1.0 };
    let face_left = (t * 0.45).cos() < 0.0;
    let kn = if step { KNIGHT } else { KNIGHT2 };
    if let Some(cc) = blit(kn, x, y, kx, ky + kb, face_left) {
        col = cc;
    }
    for i in 0..2 {
        let fi = i as f32;
        let hop = -(t * 3.0 + fi * 2.0).sin().abs() * 8.0;
        let sx = vw * (0.2 + fi * 0.6) + (t * 0.4 + fi * 3.0).sin() * 22.0;
        if let Some(cc) = blit(SLIME, x, y, sx, 108.0 + hop, false) {
            col = cc;
        }
    }
    // torch light, quantised to four dithered steps
    let mut light: f32 = 0.0;
    for i in 0..2 {
        let fi = i as f32;
        let tp = vec2(vw * (0.25 + fi * 0.5), 24.0);
        let fl = 1.0 + 0.18 * (t * 11.0 + fi * 5.0).sin() + 0.1 * (t * 23.0 + fi).sin();
        let d2 = (p - tp).dot(p - tp);
        light += 1.3 / (1.0 + d2 / (1500.0 * fl));
        // flame sprite
        let fx = x - tp.x.floor();
        let fy = y - tp.y.floor();
        let h = 5.0 + (t * 14.0 + fi * 3.0).sin() * 1.5;
        if fx.abs() < 2.0 - (fy.abs() / 3.0).floor() * 0.0
            && fy < 1.0
            && fy > -h
            && fx.abs() <= (1.0 - (fy + h) / h * 0.0).max(0.0) * (2.0 + fy * 0.3).max(0.0)
        {
            col = if fy > -2.0 {
                rgb(255, 240, 150)
            } else {
                rgb(255, 150, 50)
            };
        }
        if fx.abs() <= 1.0 && fy >= 1.0 && fy < 5.0 {
            col = rgb(70, 44, 36);
        }
    }
    // the knight carries a lantern
    let kd = p - vec2(kx + 4.0, ky + 5.0);
    light += 0.9 / (1.0 + kd.dot(kd) / (260.0 * (1.0 + 0.15 * (t * 13.0).sin())));
    let lv = (light * 4.0 + bayer4(x as i32, y as i32))
        .floor()
        .clamp(0.0, 4.0)
        / 4.0;
    let tint = rgb(255, 190, 120).mix(Vec3::WHITE, 0.3);
    col * (rgb(70, 76, 130).mix(tint, lv) * (0.5 + lv * 0.7))
}

pub fn px_campfire(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let cx = vw * 0.5;
    let ground = 102.0;
    let idx = ((y / 100.0) * 3.0 + (bayer4(x as i32, y as i32) - 0.5) * 0.9)
        .floor()
        .clamp(0.0, 3.0);
    let mut col = match idx as u32 {
        0 => rgb(6, 8, 26),
        1 => rgb(12, 16, 44),
        2 => rgb(22, 28, 64),
        _ => rgb(40, 40, 84),
    };
    let sh = hash2(x as i32, y as i32);
    if sh > 0.99 && y < 80.0 && (t * 2.0 + sh * 700.0).sin() > -0.4 {
        col = rgb(230, 230, 255);
    }
    // pines
    for i in 0..9 {
        let fi = i as f32;
        let px = fi * (vw / 8.0) + (hash1(fi * 3.0) - 0.5) * 14.0;
        let hh = 40.0 + hash1(fi * 7.0) * 30.0;
        if pine(x, y, px, ground - 6.0 - (fi % 2.0) * 4.0, hh) > 0.5 {
            col = rgb(8, 14, 28);
        }
    }
    if y >= ground - 6.0 {
        col = if hash2(x as i32, y as i32) > 0.93 {
            rgb(30, 50, 40)
        } else {
            rgb(18, 28, 36)
        };
    }
    // tent
    let tx = cx + 38.0;
    let k = ground - y;
    if k > 0.0 && k < 22.0 && (x - tx).abs() < 17.0 - k * 0.75 {
        col = rgb(150, 60, 70);
        if (x - tx).abs() < 1.5 {
            col = rgb(50, 20, 34);
        }
        if x < tx - 1.0 && ((k as i32) + (x as i32)) % 5 == 0 {
            col = rgb(130, 50, 62);
        }
    }
    // light on the world around the fire
    let flick = 1.0 + 0.12 * (t * 9.0).sin() + 0.08 * (t * 17.0).sin();
    let d2 = (p - vec2(cx, ground - 8.0)).dot(p - vec2(cx, ground - 8.0));
    let lv = ((1.6 / (1.0 + d2 / (3200.0 * flick))) * 4.0 + bayer4(x as i32, y as i32))
        .floor()
        .clamp(0.0, 4.0)
        / 4.0;
    col = (col * (0.7 + lv * 1.3)).mix(rgb(255, 150, 60) * col.luma().max(0.2), lv * 0.4);
    // logs and flames
    let fx = x - cx;
    let fy = ground - y;
    if fy > -3.0 && fy <= 0.0 && fx.abs() < 11.0 && ((fx + fy * 2.0) as i32 % 6).abs() < 4 {
        col = rgb(96, 60, 40);
    }
    let hc = 24.0 + 7.0 * noise(vec2(x * 0.45, t * 6.0));
    let prof = hc * (1.0 - (fx / 9.0) * (fx / 9.0));
    if fy > 0.0 && fy < prof && fx.abs() < 9.0 {
        col = pick4(
            1.0 - fy / prof,
            x,
            y,
            rgb(190, 40, 40),
            rgb(245, 110, 40),
            rgb(255, 190, 60),
            rgb(255, 244, 170),
        );
    }
    for i in 0..10 {
        let fi = i as f32;
        let h = hash1(fi * 5.3);
        let age = (t * (10.0 + h * 8.0) + h * 50.0) % 60.0;
        let sx = cx + (t * 2.0 + h * 9.0).sin() * 6.0 * age / 60.0 + (h - 0.5) * 8.0;
        let sy = ground - 8.0 - age;
        if x == sx.floor() && y == sy.floor() {
            col = pick4(
                1.0 - age / 60.0,
                x,
                y,
                rgb(120, 40, 40),
                rgb(240, 110, 40),
                rgb(255, 180, 60),
                rgb(255, 230, 120),
            );
        }
    }
    col
}

pub fn px_snow(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let mut col = pick4(
        1.0 - y / 90.0,
        x,
        y,
        rgb(70, 96, 160),
        rgb(100, 140, 200),
        rgb(150, 190, 226),
        rgb(206, 228, 240),
    );
    // two mountain ranges
    for l in 0..2 {
        let fl = l as f32;
        let ridge = 66.0 + fl * 12.0
            - (x * 0.035 + fl * 2.0).sin() * 14.0
            - noise(vec2(x * 0.06 + fl * 9.0, 1.0)) * 12.0;
        let ridge = ridge.floor();
        if y >= ridge {
            let cap = y - ridge;
            col = if cap < 7.0 - fl * 2.0 + dith(x, y, 0.5) * 2.0 {
                rgb(240, 246, 252)
            } else if l == 0 {
                rgb(92, 104, 150)
            } else {
                rgb(64, 78, 124)
            };
            if cap > 7.0 && ((x + cap * 0.7) as i32) % 9 < 2 && l == 0 {
                col = rgb(76, 88, 134);
            }
        }
    }
    // ground
    if y >= 100.0 {
        col = if hash2(x as i32, y as i32) > 0.96 {
            rgb(190, 214, 236)
        } else {
            rgb(228, 238, 248)
        };
        if y > 128.0 {
            col = pick4(
                (y - 128.0) / 16.0,
                x,
                y,
                rgb(228, 238, 248),
                rgb(210, 224, 242),
                rgb(190, 208, 232),
                rgb(170, 190, 220),
            );
        }
    }
    // houses
    for i in 0..3 {
        let fi = i as f32;
        let hx = vw * (0.2 + fi * 0.3);
        let base = 118.0 + (fi % 2.0) * 4.0;
        let w = 24.0 + (fi % 2.0) * 4.0;
        let dx = x - hx;
        let k = base - y;
        if dx.abs() < w * 0.5 && k > 0.0 && k <= 15.0 {
            col = if ((x as i32) / 3) % 2 == 0 {
                rgb(142, 84, 66)
            } else {
                rgb(130, 76, 60)
            };
            // window
            if dx.abs() >= 3.0 && dx.abs() < 7.0 && k > 6.0 && k < 11.0 {
                col = rgb(255, 214, 110).mix(
                    rgb(255, 170, 70),
                    0.5 + 0.5 * (t * 6.0 + fi * 3.0).sin() * 0.3,
                );
            }
            if dx.abs() < 2.0 && k < 8.0 {
                col = rgb(70, 44, 40);
            }
        }
        let rk = k - 15.0;
        if rk > 0.0 && rk < 11.0 && dx.abs() < w * 0.5 + 3.0 - rk * 1.1 {
            col = if rk > 7.0 - dith(x, y, 0.5) * 2.0 {
                rgb(244, 248, 252)
            } else {
                rgb(90, 56, 70)
            };
        }
        // chimney smoke
        let sx = hx + w * 0.25 + (t * 2.0 + y * 0.1).sin() * 2.0;
        let sk = base - 26.0 - y;
        if sk > 0.0 && sk < 28.0 && (x - sx).abs() < 1.0 + sk * 0.1 {
            col = col.mix(
                rgb(220, 226, 240),
                dith(x, y, 0.55 * (1.0 - sk / 28.0)) * 0.8,
            );
        }
    }
    // pines in front
    for i in 0..5 {
        let fi = i as f32;
        let px = vw * (0.08 + fi * 0.21) + hash1(fi * 2.0) * 10.0;
        if pine(x, y, px, 134.0, 40.0 + hash1(fi * 4.0) * 12.0) > 0.5 {
            col = rgb(24, 62, 66);
            if (x - px) < -2.0 && dith(x, y, 0.5) > 0.5 {
                col = rgb(36, 84, 84);
            }
            if dith(x + 1.0, y * 2.0, 0.2) > 0.5 && y % 8.0 < 2.0 {
                col = rgb(228, 240, 248);
            }
        }
    }
    // snowfall, three depths
    for l in 0..3 {
        let fl = l as f32;
        let sp = 14.0 + fl * 12.0;
        let cell = 4.0 + (2.0 - fl);
        let sxp = ((x + (t * 1.5 + y * 0.12).sin() * 2.0) / cell).floor();
        let syp = ((y - t * sp) / cell).floor();
        if hash2(sxp as i32 + l * 137, syp as i32) > 0.965 - fl * 0.005
            && (x % cell) < 1.0 + (2.0 - fl) * 0.0
            && (y % cell) < 1.0
        {
            col = rgb(255, 255, 255);
        }
    }
    col
}

pub fn px_reef(uv: Vec2, c: &Ctx) -> Vec3 {
    let (p, vw) = vpix(uv, c);
    let (x, y, t) = (p.x, p.y, c.time);
    let mut col = pick4(
        1.0 - y / VH,
        x,
        y,
        rgb(6, 24, 70),
        rgb(14, 62, 120),
        rgb(26, 110, 170),
        rgb(70, 180, 210),
    );
    // light shafts
    let sh = (x + y * 0.5) * 0.12;
    if sh.sin() > 0.55 + 0.2 * (t * 0.5 + (x * 0.05).floor()).sin()
        && y < 100.0
        && dith(x, y, 1.0 - y / 100.0) > 0.5
    {
        col = col.mix(rgb(190, 240, 250), 0.25);
    }
    // caustics near the surface
    let ca =
        ((x * 0.3 + t * 1.2).sin() + (y * 0.45 - t * 0.9).sin() + ((x + y) * 0.2 + t).sin()) / 3.0;
    if ca > 0.62 && y < 80.0 {
        col = col.mix(rgb(210, 250, 255), 0.3);
    }
    // far whale
    let wx = (vw + 80.0) - ((t * 4.0) % (vw + 160.0));
    let wd = (vec2((x - wx) / 28.0, (y - 40.0) / 6.0)).len();
    if wd < 1.0 {
        col = col.mix(rgb(12, 36, 90), 0.8);
    }
    // seabed
    let sy = 122.0 + (x * 0.07).sin() * 3.0 + (x * 0.2).sin() * 1.0;
    if y >= sy.floor() {
        col = if hash2(x as i32, y as i32) > 0.9 {
            rgb(214, 190, 130)
        } else {
            rgb(232, 208, 150)
        };
        col = col.mix(rgb(100, 90, 80), dith(x, y, (y - sy) / 40.0) * 0.5);
    }
    // seaweed
    for i in 0..9 {
        let fi = i as f32;
        let bx = fi * (vw / 8.0) + 6.0;
        let hh = 20.0 + hash1(fi * 2.0) * 26.0;
        let up = (122.0 - y).max(0.0);
        let sway = (t * 1.5 + fi + up * 0.12).sin() * (2.0 + up * 0.06);
        if up > 0.0 && up < hh && (x - (bx + sway).floor()).abs() < 1.0 {
            col = if (x - (bx + sway)) < 0.0 {
                rgb(40, 150, 90)
            } else {
                rgb(24, 110, 70)
            };
        }
    }
    // coral
    for i in 0..4 {
        let fi = i as f32;
        let cx = vw * (0.12 + fi * 0.26) + 12.0;
        let up = 122.0 - y;
        let d = (x - cx).abs();
        if up > 0.0 && up < 12.0 && d < 7.0 - up * 0.4 && ((x as i32) + (up as i32) / 2) % 3 != 0 {
            col = if fi % 2.0 < 1.0 {
                rgb(240, 100, 130)
            } else {
                rgb(250, 160, 80)
            };
        }
    }
    // fish school
    for i in 0..6 {
        let fi = i as f32;
        let fx = (vw + 12.0) - ((t * (8.0 + (fi % 3.0) * 3.0) + fi * 40.0) % (vw + 24.0));
        let fy = 30.0 + fi * 11.0 + (t * 1.7 + fi * 2.0).sin() * 3.0;
        let s = if i % 2 == 0 { FISH } else { FISH2 };
        if let Some(cc) = blit(s, x, y, fx, fy, false) {
            col = cc;
        }
    }
    // bubbles
    for i in 0..10 {
        let fi = i as f32;
        let h = hash1(fi * 4.1);
        let by = 120.0 - ((t * (8.0 + h * 8.0) + h * 90.0) % 120.0);
        let bx = h * vw + (t * 2.0 + fi).sin() * 2.0;
        let d = (p - vec2(bx.floor(), by.floor())).len();
        if d > 1.0 && d < 2.2 {
            col = rgb(200, 240, 255);
        }
    }
    col
}

// -------------------------------------------------------------------- post --

fn snap(src: &Frame, uv: Vec2) -> (Vec2, Vec2, f32, f32) {
    let vh = VH;
    let vw = (vh * src.w as f32 / src.h as f32).round();
    let (gx, gy) = ((uv.x * vw).floor(), (uv.y * vh).floor());
    (vec2(gx, gy), vec2(vw, vh), gx, gy)
}

pub fn px_outline(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (g, dim, gx, gy) = snap(src, uv);
    let at = |dx: f32, dy: f32| vec2((gx + dx + 0.5) / dim.x, (gy + dy + 0.5) / dim.y);
    let s = src.sample(at(0.0, 0.0));
    let l = s.luma();
    let thr = 0.22 - 0.12 * c.intensity;
    let mut edge: f32 = 0.0;
    let n = [
        src.sample(at(-1.0, 0.0)).luma(),
        src.sample(at(1.0, 0.0)).luma(),
        src.sample(at(0.0, -1.0)).luma(),
        src.sample(at(0.0, 1.0)).luma(),
    ];
    let mut i = 0;
    while i < 4 {
        if n[i] > l + thr {
            edge = 1.0;
        }
        i += 1;
    }
    let _ = g;
    if edge > 0.5 {
        s * 0.25 + rgb(6, 6, 16) * 0.5
    } else {
        s
    }
}

pub fn px_posterize(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (_, dim, gx, gy) = snap(src, uv);
    let s = src.sample(vec2((gx + 0.5) / dim.x, (gy + 0.5) / dim.y));
    let lv = 9.0 - 5.0 * c.intensity;
    let d = (bayer4(gx as i32, gy as i32) - 0.5) * 0.5;
    let q = |v: f32| ((v * (lv - 1.0) + d + 0.5).floor() / (lv - 1.0)).clamp(0.0, 1.0);
    vec3(q(s.x), q(s.y), q(s.z))
}

pub fn px_duotone(src: &Frame, uv: Vec2, c: &Ctx) -> Vec3 {
    let (_, dim, gx, gy) = snap(src, uv);
    let s = src.sample(vec2((gx + 0.5) / dim.x, (gy + 0.5) / dim.y));
    let l = sat(s.luma().powf(0.9) * 1.1);
    let out = pick4(l, gx, gy, c.pal.bg, c.pal.a, c.pal.b, c.pal.fg);
    s.mix(out, c.intensity.clamp(0.3, 1.0))
}
