//! `cargo run --release -p dodai-shaders --example radiance -- [out.ppm]`
//!
//! Fireworks over a car on a phone screen at a quarter of its size, lit with radiance cascades:
//! the car blocks the bursts' light, the tiles below catch it on the edges facing the bursts.

use dodai_shaders::radiance::{solve, Cascades, Cell, Scene};
use dodai_shaders::{vec3, Vec3};
use std::f32::consts::TAU;
use std::time::Instant;

const W: u32 = 270;
const H: u32 = 600;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "radiance.ppm".into());
    let scene = scene();
    let started = Instant::now();
    let light = solve(&scene, &Cascades::default());
    println!(
        "{W}x{H}, {} cascades, {:?}",
        Cascades::default().count(W, H),
        started.elapsed()
    );

    let sky = vec3(0.02, 0.03, 0.07);
    let mut out = format!("P6\n{W} {H}\n255\n").into_bytes();
    for y in 0..H {
        for x in 0..W {
            let cell = scene.cell(x, y);
            let base = match cell.absorption {
                a if a > 10.0 && cell.source == Vec3::default() => vec3(0.16, 0.17, 0.2),
                a if a > 0.0 => cell.source,
                _ => sky,
            };
            // no light gets into a wall: its edge shows the light just outside it
            let lit = if cell.absorption > 10.0 {
                rim(&light, x, y) * 0.8
            } else {
                at(&light, x, y)
            };
            let c = base + lit * 1.2;
            out.extend([c.x, c.y, c.z].map(|v| (v / (1.0 + v) * 1.3 * 255.0).min(255.0) as u8));
        }
    }
    std::fs::write(&path, out).expect("write the picture");
    println!("wrote {path}");
}

fn at(light: &[Vec3], x: u32, y: u32) -> Vec3 {
    light[(y.min(H - 1) * W + x.min(W - 1)) as usize]
}

/// The brightest of the four neighbours.
fn rim(light: &[Vec3], x: u32, y: u32) -> Vec3 {
    let n = [
        at(light, x.saturating_sub(1), y),
        at(light, x + 1, y),
        at(light, x, y.saturating_sub(1)),
        at(light, x, y + 1),
    ];
    n.into_iter().fold(
        Vec3::default(),
        |a, b| if b.luma() > a.luma() { b } else { a },
    )
}

fn scene() -> Scene {
    let mut s = Scene::new(W, H);
    // the car: body, cabin, wheels
    s.fill_rect(30.0, 150.0, 240.0, 205.0, Cell::wall());
    s.fill_rect(75.0, 118.0, 200.0, 152.0, Cell::wall());
    s.fill_disc(70.0, 205.0, 18.0, Cell::wall());
    s.fill_disc(200.0, 205.0, 18.0, Cell::wall());
    // header buttons and tiles
    s.fill_rect(12.0, 22.0, 84.0, 44.0, Cell::wall());
    for (x0, y0, x1, y1) in [
        (12, 252, 122, 292),
        (130, 252, 240, 292),
        (12, 299, 122, 386),
        (130, 299, 240, 386),
        (12, 393, 240, 480),
    ] {
        s.fill_rect(x0 as f32, y0 as f32, x1 as f32, y1 as f32, Cell::wall());
    }
    burst(&mut s, 160.0, 80.0, vec3(1.0, 0.7, 0.3), 1);
    burst(&mut s, 22.0, 80.0, vec3(0.3, 1.0, 0.5), 2);
    burst(&mut s, 250.0, 95.0, vec3(0.8, 0.4, 1.0), 3);
    s
}

/// A bright core with sparks around it, each a small patch of glowing gas.
fn burst(s: &mut Scene, cx: f32, cy: f32, colour: Vec3, seed: u32) {
    s.fill_disc(cx, cy, 3.0, Cell::light(colour * 4.0));
    for i in 0..48 {
        let a = TAU * i as f32 / 48.0 + seed as f32;
        let r = 14.0 + 16.0 * hash(i * 7 + seed);
        s.fill_disc(
            cx + a.cos() * r,
            cy + a.sin() * r + r * r / 200.0,
            1.6,
            Cell::glow(colour * 3.0, 0.8),
        );
    }
}

fn hash(n: u32) -> f32 {
    let n = n.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    ((n >> 9) as f32 / (1u32 << 23) as f32).fract()
}
