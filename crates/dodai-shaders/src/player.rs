//! Toolkit-neutral playback: frame pacing, adaptive resolution and caching.
//!
//! A toolkit adapter only has to call [`Player::tick`] every frame and upload
//! the returned [`Frame`] when it is `fresh` (see `Frame::to_rgba8`,
//! `to_bgra8` and `to_half_blocks`).

use crate::frame::{Ctx, Frame, ShaderPalette};
use crate::{render, ShaderInfo};

/// Plays a shader chain at a bounded cost.
pub struct Player {
    pub chain: Vec<&'static ShaderInfo>,
    pub palette: ShaderPalette,
    pub intensity: f32,
    /// Time multiplier.
    pub speed: f32,
    /// Max renders per second; `0` renders once per [`key`](Self::tick) change.
    pub max_fps: f32,
    /// Wanted device pixels per shader pixel.
    pub pixel: f32,
    /// Render-time budget in ms; above it the picture gets chunkier.
    pub budget_ms: f32,
    frame: Option<Frame>,
    last_render: f32,
    last_key: u64,
    last_size: (u32, u32),
    auto: f32,
    pub last_cost_ms: f32,
}

impl Player {
    pub fn new(chain: Vec<&'static ShaderInfo>, palette: ShaderPalette) -> Self {
        Self {
            chain,
            palette,
            intensity: 1.0,
            speed: 1.0,
            max_fps: 30.0,
            pixel: 3.0,
            budget_ms: 8.0,
            frame: None,
            last_render: f32::NEG_INFINITY,
            last_key: 0,
            last_size: (0, 0),
            auto: 1.0,
            last_cost_ms: 0.0,
        }
    }

    /// Current extra coarsening applied to stay inside the budget (`1.0` = none).
    pub fn auto_scale(&self) -> f32 {
        self.auto
    }

    /// Advance to `now` seconds for a surface of `w × h` device pixels.
    /// `key` identifies everything else that affects the picture (palette,
    /// chain, intensity); change it to force a re-render of a static view.
    /// Returns the frame and whether it was freshly rendered.
    pub fn tick(&mut self, now: f32, w: f32, h: f32, key: u64) -> (&Frame, bool) {
        let p = (self.pixel * self.auto).max(1.0);
        let size = (
            ((w / p).round() as u32).clamp(16, 640),
            ((h / p).round() as u32).clamp(9, 400),
        );
        let animated = self.max_fps > 0.0;
        let due = match &self.frame {
            None => true,
            Some(_) if size != self.last_size || key != self.last_key => true,
            Some(_) => animated && now - self.last_render >= 1.0 / self.max_fps,
        };
        if due {
            let t = if animated { now * self.speed } else { 1.5 };
            let ctx = Ctx::new(size.0, size.1, t, self.palette).intensity(self.intensity);
            let start = std::time::Instant::now();
            self.frame = Some(render(&self.chain, &ctx));
            let ms = start.elapsed().as_secs_f32() * 1000.0;
            self.last_cost_ms = ms;
            self.last_render = now;
            self.last_size = size;
            self.last_key = key;
            // Adapt: too slow → coarser; comfortably fast → back toward wanted.
            if ms > self.budget_ms {
                self.auto = (self.auto * 1.2).min(4.0);
            } else if ms < self.budget_ms * 0.4 && self.auto > 1.0 {
                self.auto = (self.auto / 1.1).max(1.0);
            }
        }
        (self.frame.as_ref().expect("rendered"), due)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::by_id;

    fn player() -> Player {
        Player::new(vec![by_id("plasma").unwrap()], ShaderPalette::default())
    }

    #[test]
    fn caps_frame_rate() {
        let mut p = player();
        p.max_fps = 10.0;
        assert!(p.tick(0.0, 320.0, 180.0, 1).1);
        assert!(!p.tick(0.05, 320.0, 180.0, 1).1);
        assert!(p.tick(0.11, 320.0, 180.0, 1).1);
    }

    #[test]
    fn static_renders_once_until_key_changes() {
        let mut p = player();
        p.max_fps = 0.0;
        assert!(p.tick(0.0, 320.0, 180.0, 1).1);
        assert!(!p.tick(9.0, 320.0, 180.0, 1).1);
        assert!(p.tick(9.0, 320.0, 180.0, 2).1);
        assert!(p.tick(9.0, 400.0, 180.0, 2).1);
    }

    #[test]
    fn slow_renders_get_coarser() {
        let mut p = player();
        p.budget_ms = 0.0;
        p.tick(0.0, 320.0, 180.0, 1);
        assert!(p.auto_scale() > 1.0);
    }
}
