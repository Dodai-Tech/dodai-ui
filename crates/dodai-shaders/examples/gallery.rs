//! `cargo run -p dodai-shaders --example gallery --release`
//!
//! Standalone demo of every shader in `dodai-shaders`, using GPUI directly.
//!
//! * Click a shader in the sidebar: generators become the source, post
//!   effects are toggled onto the chain.
//! * `Space` pause · `←/→` previous/next shader · `↑/↓` intensity · `G` grid
//!   of all generators · `P` cycle palette · `[` `]` pixel size.

use dodai_shaders::{catalog, Family, Player, ShaderInfo, ShaderPalette};
use gpui::{
    canvas, div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Corners,
    FocusHandle, KeyDownEvent, MouseButton, RenderImage, SharedString, Window, WindowBounds,
    WindowOptions,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Instant;

struct Gallery {
    sel: usize,
    fx: Vec<&'static str>,
    grid: bool,
    palette: usize,
    /// Set by a preset; cleared when the palette is cycled or a shader picked.
    pal_override: Option<ShaderPalette>,
    palettes: Vec<(SharedString, Option<ShaderPalette>)>,
    intensity: f32,
    pixel: f32,
    speed: f32,
    running: bool,
    base: f32,
    t0: Instant,
    focus: FocusHandle,
    prev_main: Prev,
    prev_grid: Vec<Prev>,
}

const BG: u32 = 0x0b0b12;
const PANEL: u32 = 0x14141f;
const LINE: u32 = 0x262638;
const TEXT: u32 = 0xe8e8f0;
const MUTED: u32 = 0x8888a0;
const ACCENT: u32 = 0xfcee0a;

impl Gallery {
    fn new(cx: &mut Context<Self>) -> Self {
        let mut palettes: Vec<(SharedString, Option<ShaderPalette>)> = vec![
            ("Family default".into(), None),
            ("Cyberpunk".into(), Some(ShaderPalette::cyberpunk())),
            ("Material 3".into(), Some(ShaderPalette::material3())),
            ("Matrix".into(), Some(ShaderPalette::matrix())),
            ("Amber terminal".into(), Some(ShaderPalette::amber())),
            ("Platformer".into(), Some(ShaderPalette::platformer())),
            ("Metroidvania".into(), Some(ShaderPalette::metroidvania())),
        ];
        for t in dodai::themes::all() {
            palettes.push((
                format!("Theme: {}", t.meta.name).into(),
                Some(ShaderPalette::from_theme(&t)),
            ));
        }
        let first_gen = catalog().iter().position(|s| s.is_generator()).unwrap_or(0);
        Self {
            sel: first_gen,
            fx: Vec::new(),
            grid: false,
            palette: 0,
            pal_override: None,
            palettes,
            intensity: 1.0,
            pixel: 2.0,
            speed: 1.0,
            running: true,
            base: 0.0,
            t0: Instant::now(),
            focus: cx.focus_handle(),
            prev_main: Default::default(),
            prev_grid: (0..catalog().len()).map(|_| Default::default()).collect(),
        }
    }

    /// Everything but time that changes the picture.
    fn key(&self) -> u64 {
        let mut h = (self.palette as u64) << 8 ^ self.intensity.to_bits() as u64;
        h = h.wrapping_mul(31).wrapping_add(self.sel as u64);
        for f in &self.fx {
            for b in f.bytes() {
                h = h.wrapping_mul(31).wrapping_add(b as u64);
            }
        }
        h
    }

    fn time(&self) -> f32 {
        self.base
            + if self.running {
                self.t0.elapsed().as_secs_f32() * self.speed
            } else {
                0.0
            }
    }

    fn toggle_run(&mut self) {
        self.base = self.time();
        self.t0 = Instant::now();
        self.running = !self.running;
    }

    fn palette_for(&self, family: Family) -> ShaderPalette {
        self.palettes[self.palette]
            .1
            .unwrap_or_else(|| family.palette())
    }

    fn apply_preset(&mut self, p: &'static dodai_shaders::Preset) {
        let chain = p.shaders();
        if let Some(i) = catalog().iter().position(|s| s.id == chain[0].id) {
            self.sel = i;
            self.fx = chain[1..].iter().map(|s| s.id).collect();
            self.pal_override = Some((p.palette)());
            self.grid = false;
        }
    }

    fn pick(&mut self, i: usize) {
        self.pal_override = None;
        let s = &catalog()[i];
        if s.is_generator() {
            self.sel = i;
            self.fx.clear();
        } else if let Some(p) = self.fx.iter().position(|x| *x == s.id) {
            self.fx.remove(p);
        } else {
            self.fx.push(s.id);
        }
    }

    fn step(&mut self, d: isize) {
        let n = catalog().len() as isize;
        let mut i = self.sel as isize;
        loop {
            i = (i + d).rem_euclid(n);
            if catalog()[i as usize].is_generator() {
                break;
            }
        }
        self.sel = i as usize;
    }

    /// Chain shown in the main view: the selected generator (or the family's
    /// default source when only effects were picked) plus effects.
    fn chain(&self) -> Vec<&'static ShaderInfo> {
        let mut c = vec![&catalog()[self.sel]];
        c.extend(self.fx.iter().filter_map(|id| dodai_shaders::by_id(id)));
        c
    }
}

/// Per-view state: the toolkit-neutral player plus the last uploaded texture.
#[derive(Default)]
struct View {
    player: Option<Player>,
    image: Option<Arc<RenderImage>>,
}
type Prev = Rc<RefCell<View>>;

#[allow(clippy::too_many_arguments)]
fn shader_canvas(
    chain: Vec<&'static ShaderInfo>,
    pal: ShaderPalette,
    pixel: f32,
    t: f32,
    intensity: f32,
    fps: f32,
    key: u64,
    view: Prev,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<gpui::Pixels>, _, w, _| {
            let (bw, bh): (f32, f32) = (b.size.width.into(), b.size.height.into());
            let visible = w.content_mask().bounds.intersects(&b);
            let mut v = view.borrow_mut();
            let pl = v
                .player
                .get_or_insert_with(|| Player::new(chain.clone(), pal));
            pl.chain.clone_from(&chain);
            pl.palette = pal;
            pl.intensity = intensity;
            pl.pixel = pixel;
            pl.max_fps = fps;
            if visible || v.image.is_none() {
                let pl = v.player.as_mut().expect("set");
                let (frame, fresh) = pl.tick(t, bw, bh, key);
                if fresh {
                    let buf = image::RgbaImage::from_raw(frame.w, frame.h, frame.to_bgra8())
                        .expect("size");
                    let img = Arc::new(RenderImage::new(vec![image::Frame::new(buf)]));
                    if let Some(old) = v.image.replace(img) {
                        let _ = w.drop_image(old);
                    }
                }
            }
            if let Some(img) = &v.image {
                let _ = w.paint_image(b, Corners::default(), img.clone(), 0, false);
            }
            if fps > 0.0 && visible {
                w.request_animation_frame();
            }
        },
    )
    .absolute()
    .size_full()
}

fn chip(label: impl Into<SharedString>, on: bool) -> gpui::Div {
    div()
        .px(px(10.0))
        .py(px(5.0))
        .border_1()
        .border_color(rgb(if on { ACCENT } else { LINE }))
        .text_color(rgb(if on { ACCENT } else { TEXT }))
        .text_size(px(12.0))
        .cursor_pointer()
        .child(label.into())
}

impl Render for Gallery {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.time();
        let cat = catalog();
        let cur = &cat[self.sel];

        // Sidebar
        let mut side = div()
            .id("side")
            .w(px(250.0))
            .h_full()
            .flex_shrink_0()
            .overflow_y_scroll()
            .bg(rgb(PANEL))
            .border_r_1()
            .border_color(rgb(LINE))
            .p(px(10.0))
            .flex()
            .flex_col()
            .gap(px(2.0));
        side = side.child(
            div()
                .mb(px(4.0))
                .text_size(px(11.0))
                .text_color(rgb(ACCENT))
                .child("PRESETS"),
        );
        for (pi, p) in dodai_shaders::presets::all().enumerate() {
            side = side.child(
                div()
                    .id(("preset", pi))
                    .px(px(8.0))
                    .py(px(4.0))
                    .text_size(px(13.0))
                    .text_color(rgb(TEXT))
                    .hover(|d| d.bg(rgb(LINE)))
                    .cursor_pointer()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |g, _, _, cx| {
                            g.apply_preset(p);
                            cx.notify();
                        }),
                    )
                    .child(format!("★ {}", p.name)),
            );
        }
        side = side.child(
            div()
                .mt(px(8.0))
                .text_size(px(11.0))
                .text_color(rgb(TEXT))
                .child("▣ source (pick one)   ◇ effect (stack)"),
        );
        for fam in Family::ALL {
            side = side.child(
                div()
                    .mt(px(10.0))
                    .mb(px(4.0))
                    .text_size(px(11.0))
                    .text_color(rgb(ACCENT))
                    .child(fam.name().to_uppercase()),
            );
            for (i, s) in cat.iter().enumerate().filter(|(_, s)| s.family == fam) {
                let active = if s.is_generator() {
                    i == self.sel
                } else {
                    self.fx.contains(&s.id)
                };
                let tag = if s.is_generator() { "▣" } else { "◇" };
                let idle = if s.is_generator() { TEXT } else { 0xe0a050 };
                side = side.child(
                    div()
                        .id(("sh", i))
                        .px(px(8.0))
                        .py(px(4.0))
                        .text_size(px(13.0))
                        .text_color(rgb(if active { BG } else { idle }))
                        .bg(rgb(if active { ACCENT } else { PANEL }))
                        .hover(|d| d.bg(rgb(LINE)))
                        .cursor_pointer()
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |g, _, _, cx| {
                                g.grid = false;
                                g.pick(i);
                                cx.notify();
                            }),
                        )
                        .child(format!("{tag} {}", s.name)),
                );
            }
        }

        // Controls
        let pal_name = self.palettes[self.palette].0.clone();
        let controls = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.0))
            .child(
                chip(
                    if self.running {
                        "⏸ Pause"
                    } else {
                        "▶ Play"
                    },
                    !self.running,
                )
                .id("run")
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|g, _, _, cx| {
                        g.toggle_run();
                        cx.notify();
                    }),
                ),
            )
            .child(
                chip(if self.grid { "▦ Grid" } else { "▢ Single" }, self.grid)
                    .id("grid")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|g, _, _, cx| {
                            g.grid = !g.grid;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                chip(format!("Palette: {pal_name}"), false)
                    .id("pal")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|g, _, _, cx| {
                            g.pal_override = None;
                            g.palette = (g.palette + 1) % g.palettes.len();
                            cx.notify();
                        }),
                    ),
            )
            .child(
                chip(format!("Intensity {:.0}%", self.intensity * 100.0), false)
                    .id("int")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|g, _, _, cx| {
                            g.intensity = if g.intensity >= 1.0 {
                                0.0
                            } else {
                                g.intensity + 0.25
                            };
                            cx.notify();
                        }),
                    ),
            )
            .child(
                chip(format!("Pixel {}px", self.pixel), false)
                    .id("px")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|g, _, _, cx| {
                            g.pixel = if g.pixel >= 8.0 { 1.0 } else { g.pixel + 1.0 };
                            cx.notify();
                        }),
                    ),
            )
            .child(
                chip(format!("Speed ×{:.1}", self.speed), false)
                    .id("spd")
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|g, _, _, cx| {
                            g.base = g.time();
                            g.t0 = Instant::now();
                            g.speed = if g.speed >= 2.0 { 0.25 } else { g.speed + 0.25 };
                            cx.notify();
                        }),
                    ),
            );

        // Main area
        let main_area: gpui::AnyElement = if self.grid {
            let mut g = div()
                .flex()
                .flex_wrap()
                .gap(px(10.0))
                .id("grid")
                .overflow_y_scroll()
                .flex_1();
            for (i, s) in cat.iter().enumerate().filter(|(_, s)| s.is_generator()) {
                let pal = self.palette_for(s.family);
                g = g.child(
                    div()
                        .w(px(230.0))
                        .flex()
                        .flex_col()
                        .gap(px(4.0))
                        .child(div().relative().w_full().h(px(130.0)).child(shader_canvas(
                            vec![s],
                            pal,
                            4.0,
                            t,
                            self.intensity,
                            15.0,
                            self.key(),
                            self.prev_grid[i].clone(),
                        )))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(TEXT))
                                .child(s.name),
                        ),
                );
            }
            g.into_any_element()
        } else {
            let chain = self.chain();
            let fam = cur.family;
            let pal = self.palette_for(fam);
            // Effects alone: put them over the family's default source.
            let mut shown = chain.clone();
            if !cur.is_generator() {
                shown.insert(0, dodai_shaders::by_id(fam.default_source()).unwrap());
            }
            let names: Vec<&str> = chain.iter().map(|s| s.name).collect();
            div()
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .child(
                    div()
                        .text_size(px(20.0))
                        .text_color(rgb(TEXT))
                        .child(names.join("  →  ")),
                )
                .child(
                    div()
                        .text_size(px(13.0))
                        .text_color(rgb(MUTED))
                        .child(cur.desc),
                )
                .child(
                    div()
                        .relative()
                        .flex_1()
                        .w_full()
                        .border_1()
                        .border_color(rgb(LINE))
                        .child(shader_canvas(
                            shown,
                            pal,
                            self.pixel,
                            t,
                            self.intensity,
                            30.0,
                            self.key(),
                            self.prev_main.clone(),
                        )),
                )
                .into_any_element()
        };

        let _ = window;
        div()
            .id("root")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|g, e: &KeyDownEvent, _, cx| {
                match e.keystroke.key.as_str() {
                    "space" => g.toggle_run(),
                    "right" => g.step(1),
                    "left" => g.step(-1),
                    "up" => g.intensity = (g.intensity + 0.1).min(1.0),
                    "down" => g.intensity = (g.intensity - 0.1).max(0.0),
                    "g" => g.grid = !g.grid,
                    "p" => g.palette = (g.palette + 1) % g.palettes.len(),
                    "]" => g.pixel = (g.pixel + 1.0).min(10.0),
                    "[" => g.pixel = (g.pixel - 1.0).max(1.0),
                    _ => return,
                }
                cx.notify();
            }))
            .size_full()
            .flex()
            .bg(rgb(BG))
            .text_color(rgb(TEXT))
            .child(side)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(10.0))
                    .p(px(14.0))
                    .min_w_0()
                    .child(controls)
                    .child(main_area),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        let win = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(Gallery::new),
            )
            .expect("window");
        win.update(cx, |g, window, cx| {
            window.focus(&g.focus);
            cx.activate(true);
        })
        .ok();
    });
}
