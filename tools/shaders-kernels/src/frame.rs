//! GPU-side `Frame`: a read-only view of a storage buffer, with the
//! sampling methods post effects use.

pub use crate::ctx::{Ctx, ShaderPalette};
use crate::math::*;
use crate::prelude::*;
use glam::Vec4;

pub struct Frame<'a> {
    pub w: u32,
    pub h: u32,
    data: &'a [Vec4],
}

impl<'a> Frame<'a> {
    pub fn new(w: u32, h: u32, data: &'a [Vec4]) -> Self {
        Self { w, h, data }
    }
    pub fn get(&self, x: i32, y: i32) -> Vec3 {
        let x = x.clamp(0, self.w as i32 - 1) as u32;
        let y = y.clamp(0, self.h as i32 - 1) as u32;
        let p = self.data[(y * self.w + x) as usize];
        vec3(p.x, p.y, p.z)
    }
    pub fn nearest(&self, uv: Vec2) -> Vec3 {
        self.get(
            (uv.x * self.w as f32).floor() as i32,
            (uv.y * self.h as f32).floor() as i32,
        )
    }
    pub fn sample(&self, uv: Vec2) -> Vec3 {
        let x = uv.x * self.w as f32 - 0.5;
        let y = uv.y * self.h as f32 - 0.5;
        let (x0, y0) = (x.floor(), y.floor());
        let (fx, fy) = (x - x0, y - y0);
        let (x0, y0) = (x0 as i32, y0 as i32);
        let top = self.get(x0, y0).mix(self.get(x0 + 1, y0), fx);
        let bot = self.get(x0, y0 + 1).mix(self.get(x0 + 1, y0 + 1), fx);
        top.mix(bot, fy)
    }
}
