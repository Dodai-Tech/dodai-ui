//! [`Clock`]: analog or digital.

use super::helpers::*;
use crate::paint::shaped;
use crate::text::first_family;
use crate::{hsla, theme_of};
use gpui::{
    canvas, div, point, px, App, IntoElement, ParentElement, PathBuilder, RenderOnce, Styled,
    Window,
};

// ───────────────────────────── clock ─────────────────────────────

/// Which clock face to draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ClockFace {
    /// Follow the theme: cut-corner themes get the futuristic digital face,
    /// the others the analog dial.
    #[default]
    Auto,
    /// A dial with hands.
    Analog,
    /// Glowing seven-segment readout with ghost segments and a seconds bar.
    Digital,
}

impl ClockFace {
    /// Whether this face resolves to the digital readout for a theme shape.
    pub fn is_digital(self, kind: dodai_ui_core::ShapeKind) -> bool {
        match self {
            ClockFace::Digital => true,
            ClockFace::Analog => false,
            ClockFace::Auto => kind == dodai_ui_core::ShapeKind::Chamfered,
        }
    }
}

/// Live clock: analog dial, or a futuristic seven-segment face (see [`ClockFace`]).
///
/// ```ignore
/// Clock::new().size(180.0).local()
/// ```
#[derive(IntoElement)]
pub struct Clock {
    face: ClockFace,
    size: f32,
    seconds: bool,
    digital: bool,
    offset_min: i32,
}

impl Clock {
    /// Create a new [`Clock`].
    pub fn new() -> Self {
        Self {
            face: ClockFace::Auto,
            size: 180.0,
            seconds: true,
            digital: true,
            offset_min: 0,
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Pick the face (default [`ClockFace::Auto`]: follows the theme).
    pub fn face(mut self, f: ClockFace) -> Self {
        self.face = f;
        self
    }
    /// Show the second hand (default on).
    pub fn seconds(mut self, on: bool) -> Self {
        self.seconds = on;
        self
    }
    /// Show `HH:MM:SS` under the face (default on).
    pub fn digital(mut self, on: bool) -> Self {
        self.digital = on;
        self
    }
    /// Offset from UTC in minutes (default 0 = UTC).
    pub fn utc_offset_minutes(mut self, m: i32) -> Self {
        self.offset_min = m;
        self
    }
    /// Use this machine's time zone.
    pub fn local(self) -> Self {
        let m = local_utc_offset_minutes();
        self.utc_offset_minutes(m)
    }
}

impl RenderOnce for Clock {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let digital = self.face.is_digital(t.shape.kind());
        if digital {
            return digital_face(&self, &t).into_any_element();
        }
        let (size, seconds, off) = (self.size, self.seconds, self.offset_min);
        let (ink, accent, tick) = (
            hsla(t.palette.text),
            hsla(t.palette.primary),
            hsla(t.palette.border_strong),
        );
        let now = day_secs(unix_now(), off);
        let face = div().relative().flex_none().w(px(size)).h(px(size)).child(
            canvas(
                |_, _, _| (),
                move |b, _, window, _| {
                    let r = size / 2.0;
                    let (o, c) = (b.origin, (size / 2.0, size / 2.0));
                    let (hour, min, sec) = hand_angles(day_secs(unix_now(), off));
                    let pt = |a: f32, len: f32| {
                        // 0 rad = 12 o'clock, clockwise.
                        let (s, co) = a.sin_cos();
                        point(o.x + px(c.0 + len * s), o.y + px(c.1 - len * co))
                    };
                    let mut line = |a: f32, from: f32, to: f32, w: f32, col| {
                        let mut pb = PathBuilder::stroke(px(w));
                        pb.add_polygon(&[pt(a, from), pt(a, to)], false);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, col);
                        }
                    };
                    for i in 0..60 {
                        let a = i as f32 * std::f32::consts::TAU / 60.0;
                        if i % 5 == 0 {
                            line(a, r * 0.84, r * 0.96, 2.0, ink);
                        } else {
                            line(a, r * 0.92, r * 0.96, 1.0, tick);
                        }
                    }
                    line(hour, 0.0, r * 0.5, 4.0, ink);
                    line(min, 0.0, r * 0.74, 3.0, ink);
                    if seconds {
                        line(sec, -r * 0.14, r * 0.86, 1.5, accent);
                    }
                    line(0.0, 0.0, 0.0, 7.0, accent); // hub dot
                    crate::motion::next_frame(window);
                },
            )
            .absolute()
            .size_full(),
        );
        let mut col = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(t.spacing.sm))
            .child(face);
        if self.digital {
            let s = now as u64;
            let txt = if seconds {
                hms(s)
            } else {
                hms(s)[..5].to_string()
            };
            col = col.child(
                div()
                    .font_family(first_family(t.typography.display))
                    .text_size(px(t.typography.size_xl))
                    .text_color(ink)
                    .child(txt),
            );
        }
        col.into_any_element()
    }
}

/// One seven-segment digit; `slant` shears it forward for a sporty look.
#[allow(clippy::too_many_arguments)]
fn paint_digit(
    window: &mut Window,
    o: gpui::Point<gpui::Pixels>,
    (x, y, w, h, th): (f32, f32, f32, f32, f32),
    digit: u8,
    on: gpui::Hsla,
    off: gpui::Hsla,
    halo: f32,
) {
    let slant = 0.12;
    let pt = |px_: f32, py: f32| point(o.x + px(px_ + (h - py) * slant), o.y + px(py));
    let mut poly = |pts: [(f32, f32); 6], c: gpui::Hsla| {
        let v: Vec<_> = pts.iter().map(|&(a, b)| pt(a, b)).collect();
        let mut pb = PathBuilder::fill();
        pb.add_polygon(&v, true);
        if let Ok(p) = pb.build() {
            window.paint_path(p, c);
        }
    };
    let (g, t2) = (th * 0.16, th * 0.5);
    let (l, r, top, mid, bot) = (x + t2, x + w - t2, y + t2, y + h / 2.0, y + h - t2);
    let horiz = |yc: f32, grow: f32| {
        let (a, b, t3) = (l + g - grow, r - g + grow, t2 + grow);
        [
            (a, yc),
            (a + t3, yc - t3),
            (b - t3, yc - t3),
            (b, yc),
            (b - t3, yc + t3),
            (a + t3, yc + t3),
        ]
    };
    let vert = |xc: f32, y0: f32, y1: f32, grow: f32| {
        let (a, b, t3) = (y0 + g - grow, y1 - g + grow, t2 + grow);
        [
            (xc, a),
            (xc + t3, a + t3),
            (xc + t3, b - t3),
            (xc, b),
            (xc - t3, b - t3),
            (xc - t3, a + t3),
        ]
    };
    let bits = segments(digit);
    let shapes = |grow: f32| {
        [
            horiz(top, grow),
            vert(r, top, mid, grow),
            vert(r, mid, bot, grow),
            horiz(bot, grow),
            vert(l, mid, bot, grow),
            vert(l, top, mid, grow),
            horiz(mid, grow),
        ]
    };
    for (i, s) in shapes(0.0).into_iter().enumerate() {
        if bits & (1 << i) == 0 {
            poly(s, off);
        }
    }
    if halo > 0.0 {
        let mut c = on;
        c.a *= 0.22 * halo;
        for (i, s) in shapes(th * 0.35).into_iter().enumerate() {
            if bits & (1 << i) != 0 {
                poly(s, c);
            }
        }
    }
    for (i, s) in shapes(0.0).into_iter().enumerate() {
        if bits & (1 << i) != 0 {
            poly(s, on);
        }
    }
}

/// Futuristic face: glowing seven-segment `HH:MM` + `SS`, ghost segments, a
/// 60-cell seconds bar and the date, in the theme's panel shape.
fn digital_face(c: &Clock, t: &dodai_ui_core::Theme) -> gpui::Div {
    let (off_min, seconds) = (c.offset_min, c.seconds);
    let width = c.size * 2.0;
    let unit = width / 7.1;
    let (dw, dh) = (unit, unit * 1.9);
    let bar_h = (dh * 0.09).max(4.0);
    let height = dh + bar_h * 2.6;
    let (hi, lo) = (hsla(t.palette.primary), hsla(t.palette.secondary));
    let mut ghost = hsla(t.palette.border);
    ghost.a *= 0.55;
    let halo = 0.4 + t.effects.glow_intensity;
    let now = unix_now();
    let date = format!(
        "{}  ·  UTC{}{:02}:{:02}",
        date_text(now, off_min),
        if off_min < 0 { '-' } else { '+' },
        off_min.abs() / 60,
        off_min.abs() % 60
    );
    let display = div()
        .relative()
        .flex_none()
        .w(px(width))
        .h(px(height))
        .child(
            canvas(
                |_, _, _| (),
                move |b, _, window, _| {
                    let o = b.origin;
                    let secs_f = day_secs(unix_now(), off_min);
                    let s = secs_f as u64;
                    let (hh, mm, ss) = ((s / 3600) as u8, (s / 60 % 60) as u8, (s % 60) as u8);
                    let th = dw * 0.17;
                    let gap = dw * 0.18;
                    let mut x = 0.0;
                    for d in [hh / 10, hh % 10] {
                        paint_digit(window, o, (x, 0.0, dw, dh, th), d, hi, ghost, halo);
                        x += dw + gap;
                    }
                    // blinking colon
                    let lit = secs_f.fract() < 0.5;
                    let slant = |y: f32| (dh - y) * 0.12;
                    for y in [dh * 0.32, dh * 0.68] {
                        let r = th * 0.55;
                        let cx = x + dw * 0.1 + slant(y);
                        let v = [
                            point(o.x + px(cx - r), o.y + px(y)),
                            point(o.x + px(cx), o.y + px(y - r)),
                            point(o.x + px(cx + r), o.y + px(y)),
                            point(o.x + px(cx), o.y + px(y + r)),
                        ];
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&v, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, if lit { hi } else { ghost });
                        }
                    }
                    x += dw * 0.4;
                    for d in [mm / 10, mm % 10] {
                        paint_digit(window, o, (x, 0.0, dw, dh, th), d, hi, ghost, halo);
                        x += dw + gap;
                    }
                    if seconds {
                        x += gap;
                        let (sw, sh) = (dw * 0.55, dh * 0.55);
                        let y0 = dh - sh;
                        for d in [ss / 10, ss % 10] {
                            paint_digit(window, o, (x, y0, sw, sh, th * 0.6), d, lo, ghost, halo);
                            x += sw + gap * 0.6;
                        }
                    }
                    // seconds bar: 60 cells, filled up to the current second
                    let cell = width / 60.0;
                    for i in 0..60u8 {
                        let x0 = i as f32 * cell;
                        let c = if i <= ss {
                            if i == ss {
                                lo
                            } else {
                                hi
                            }
                        } else {
                            ghost
                        };
                        let y0 = dh + bar_h * 1.3;
                        let v = [
                            point(o.x + px(x0 + 1.0), o.y + px(y0)),
                            point(o.x + px(x0 + cell - 1.0), o.y + px(y0)),
                            point(o.x + px(x0 + cell - 1.0), o.y + px(y0 + bar_h)),
                            point(o.x + px(x0 + 1.0), o.y + px(y0 + bar_h)),
                        ];
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&v, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, c);
                        }
                    }
                    crate::motion::next_frame(window);
                },
            )
            .absolute()
            .size_full(),
        );
    let mut frame = shaped(&t.shape, crate::panel::panel_style(t, false))
        .p(px(t.spacing.lg))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(t.spacing.sm))
        .child(display);
    if c.digital {
        frame = frame.child(
            div()
                .font_family(first_family(t.typography.mono))
                .text_size(px(t.typography.size_sm))
                .text_color(hsla(t.palette.text_muted))
                .child(date),
        );
    }
    frame
}

impl Default for Clock {
    fn default() -> Self {
        Self::new()
    }
}
