//! [`ShiftLights`]: a row of lights over the engine speed that fills green, amber and red towards
//! the shift point and turns all blue at it, with the gear under it. Drawn in every [`Look`].

use crate::dash::{text_box, Tone};
use crate::frames::{aged, ghost, glow_dot, seg_text};
use crate::look::Look;
use crate::mfd::rect;
use crate::pod_math as pm;
use crate::race::{figure, segments, skew};
use crate::shapes::{chamfered, circle, rounded, Ink};
use dodai_gpui::follow::{follow, Follow};
use dodai_gpui::follow_api;
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{
    canvas, div, px, App, Bounds, FontWeight, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    SharedString, Styled, Window,
};

/// How many of `n` lights are lit at `rpm`: none up to `start`, all from `shift`, in between in
/// proportion, rounded up so the first light comes on just past `start`. Pure.
pub fn lit_lights(rpm: f32, start: f32, shift: f32, n: usize) -> usize {
    let rpm = pm::finite(rpm, 0.0);
    if rpm <= start {
        return 0;
    }
    if rpm >= shift || shift <= start {
        return n;
    }
    (((rpm - start) / (shift - start)) * n as f32).ceil() as usize
}

/// The tone of light `i` of `n` while climbing: the first four tenths green, the next three amber,
/// the rest red. Pure.
pub fn light_tone(i: usize, n: usize) -> Tone {
    let f = (i as f32 + 0.5) / n.max(1) as f32;
    if f < 0.4 {
        Tone::Success
    } else if f < 0.7 {
        Tone::Warning
    } else {
        Tone::Danger
    }
}

/// Shift lights: a row of lights over the engine speed and the gear under it.
///
/// Lights come on one by one from `start` (green, then amber, then red) and all turn blue at
/// `shift`, the moment to change up. Each [`Look`] draws its own lights: dots, lenses, frames,
/// segment blocks, glow, annunciator bars or a rising rev bar.
#[derive(IntoElement)]
pub struct ShiftLights {
    follow: Follow,
    rpm: f32,
    start: f32,
    shift: f32,
    count: usize,
    gear: Option<SharedString>,
    look: Look,
    width: f32,
}

impl ShiftLights {
    follow_api!();

    /// Lights for `rpm`, starting at 5 000 and shifting at 7 000.
    #[track_caller]
    pub fn new(rpm: f32) -> Self {
        Self {
            follow: Follow::here(),
            rpm: pm::finite(rpm, 0.0),
            start: 5000.0,
            shift: 7000.0,
            count: 10,
            gear: None,
            look: Look::Sport,
            width: 320.0,
        }
    }
    /// The engine speeds where the first light comes on and where all turn blue.
    pub fn range(mut self, start: f32, shift: f32) -> Self {
        self.start = pm::finite(start, 5000.0);
        self.shift = pm::finite(shift, 7000.0).max(self.start);
        self
    }
    /// How many lights (2 to 20, default 10).
    pub fn count(mut self, n: usize) -> Self {
        self.count = n.clamp(2, 20);
        self
    }
    /// The gear under the lights, e.g. `3` or `N`.
    pub fn gear(mut self, g: impl Into<SharedString>) -> Self {
        self.gear = Some(g.into());
        self
    }
    /// The look (see [`Look`]).
    pub fn look(mut self, look: Look) -> Self {
        self.look = look;
        self
    }
    /// Width of the row of lights in px (at least 120); the housing adds a margin and the
    /// height follows.
    pub fn width(mut self, w: f32) -> Self {
        self.width = pm::finite(w, 320.0).max(120.0);
        self
    }
}

/// Paint the strip the lights sit in, `wd` by `h`, as `look` builds it: none for the sport dash,
/// a bezel, a plate, brackets, an LCD, a glow line, a display frame or a race plate.
fn housing(w: &mut Window, o: gpui::Point<Pixels>, (wd, h): (f32, f32), look: Look, ink: Ink) {
    let k = h * 0.25;
    match look {
        Look::Sport => {}
        Look::Chronograph => {
            let p = rounded(0.0, 0.0, wd, h, h / 2.0);
            fill(w, o, &p, scale_alpha(ink.text, 0.06));
            stroke(w, o, &p, h * 0.06, ink.line, true);
        }
        Look::Vintage => {
            let p = chamfered(0.0, 0.0, wd, h, k * 0.5);
            fill(w, o, &p, aged(ink));
            stroke(w, o, &p, h * 0.05, scale_alpha(ink.text, 0.3), true);
        }
        Look::HeadsUp => {
            for (x, dx) in [(0.0, 1.0), (wd, -1.0)] {
                stroke(
                    w,
                    o,
                    &[(x + dx * k, 0.0), (x, 0.0), (x, h), (x + dx * k, h)],
                    h * 0.05,
                    scale_alpha(ink.primary, 0.6),
                    false,
                );
            }
        }
        Look::Segment => {
            crate::frames::lcd(w, o, (0.0, 0.0, wd, h), h * 0.8, ink);
        }
        Look::Halo => {
            stroke(
                w,
                o,
                &[(k, h - h * 0.08), (wd - k, h - h * 0.08)],
                h * 0.03,
                scale_alpha(ink.primary, 0.5),
                false,
            );
        }
        Look::GlassCockpit => {
            let p = rect(0.0, 0.0, wd, h);
            fill(w, o, &p, ink.ground);
            stroke(w, o, &p, h * 0.04, scale_alpha(ink.text, 0.35), true);
        }
        Look::Race => {
            let p = crate::race::plate_outline(wd, h, h * 0.35);
            fill(w, o, &p, ink.ground);
            stroke(w, o, &p, h * 0.04, ink.line, true);
            fill(
                w,
                o,
                &skew(h * 0.3, h * 0.86, wd * 0.3, h * 0.06),
                ink.primary,
            );
        }
    }
}

/// Paint one light of diameter `d` at `(x, y)` (its top left) in `look`.
#[allow(clippy::too_many_arguments)]
fn light(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    (x, y): (f32, f32),
    d: f32,
    col: Hsla,
    lit: bool,
    look: Look,
    ink: Ink,
    dark: bool,
) {
    let (cx, cy, r) = (x + d / 2.0, y + d / 2.0, d / 2.0);
    let off = scale_alpha(ink.text, 0.1);
    match look {
        Look::Sport => fill(w, o, &circle(cx, cy, r * 0.8), if lit { col } else { off }),
        Look::Chronograph => {
            stroke(w, o, &circle(cx, cy, r * 0.92), d * 0.08, ink.line, true);
            fill(w, o, &circle(cx, cy, r * 0.62), if lit { col } else { off });
        }
        Look::Vintage => {
            fill(w, o, &circle(cx, cy, r), scale_alpha(ink.text, 0.25));
            fill(
                w,
                o,
                &circle(cx, cy, r * 0.8),
                if lit {
                    col
                } else {
                    scale_alpha(aged(ink), 0.6)
                },
            );
            fill(
                w,
                o,
                &circle(cx - r * 0.25, cy - r * 0.25, r * 0.18),
                scale_alpha(ink.text, 0.4),
            );
        }
        Look::HeadsUp => {
            let frame = chamfered(x + d * 0.08, y + d * 0.08, d * 0.84, d * 0.84, d * 0.24);
            if lit {
                fill(w, o, &frame, scale_alpha(col, 0.25));
            }
            stroke(
                w,
                o,
                &frame,
                d * 0.07,
                scale_alpha(col, if lit { 1.0 } else { 0.3 }),
                true,
            );
        }
        Look::Segment => {
            let block = chamfered(x + d * 0.06, y + d * 0.2, d * 0.88, d * 0.6, d * 0.1);
            fill(w, o, &block, if lit { col } else { ghost(col, dark) });
        }
        Look::Halo => {
            if lit {
                glow_dot(w, o, (cx, cy), r * 0.4, col, scale_alpha(ink.text, 0.8));
            }
            stroke(
                w,
                o,
                &circle(cx, cy, r * 0.75),
                d * 0.04,
                scale_alpha(col, if lit { 0.7 } else { 0.25 }),
                true,
            );
        }
        Look::GlassCockpit => {
            let bar = rect(x + d * 0.08, y + d * 0.3, d * 0.84, d * 0.4);
            fill(w, o, &bar, if lit { col } else { ink.ground });
            stroke(
                w,
                o,
                &bar,
                d * 0.05,
                if lit { col } else { scale_alpha(ink.text, 0.4) },
                true,
            );
        }
        // Race lights are drawn as one rising bar by the caller.
        Look::Race => {}
    }
}

impl RenderOnce for ShiftLights {
    fn render(mut self, w: &mut Window, cx: &mut App) -> impl IntoElement {
        self.rpm = pm::finite(follow(w, cx, &self.follow, 0, self.rpm), 0.0);
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let dark = t.is_dark();
        let n = self.count;
        let lit = lit_lights(self.rpm, self.start, self.shift, n);
        let shift_now = lit == n && self.rpm >= self.shift;
        let cols: Vec<Hsla> = (0..n)
            .map(|i| {
                if shift_now {
                    Tone::Info.color(&t)
                } else {
                    light_tone(i, n).color(&t)
                }
            })
            .collect();
        let wd = self.width;
        let pitch = wd / n as f32;
        let d = pitch * 0.86;
        let look = self.look;
        let pad = d * 0.45;
        let (hw, hh) = (wd + 2.0 * pad, d + 2.0 * pad);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            housing(w, o, (hw, hh), look, ink);
            if look == Look::Race {
                // A rev bar: slanted blocks rising to the shift point.
                let segs = segments((pad, pad, wd, d), cols.len(), true);
                for (i, (seg, col)) in segs.iter().zip(&cols).enumerate() {
                    fill(
                        w,
                        o,
                        seg,
                        if i < lit {
                            *col
                        } else {
                            scale_alpha(ink.text, 0.1)
                        },
                    );
                }
                return;
            }
            for (i, col) in cols.iter().enumerate() {
                let x = pad + pitch * i as f32 + (pitch - d) / 2.0;
                light(w, o, (x, pad), d, *col, i < lit, look, ink, dark);
            }
        };
        let row = div().relative().w(px(hw)).h(px(hh)).child(
            canvas(
                |_, _, _| (),
                move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
            )
            .absolute()
            .size_full(),
        );
        let mut el = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(d * 0.3))
            .child(row);
        if let Some(g) = self.gear {
            let h = (wd * 0.2).max(28.0);
            let col = if shift_now {
                Tone::Info.color(&t)
            } else {
                ink.text
            };
            let gear = match look {
                Look::Segment => div()
                    .relative()
                    .w(px(h))
                    .h(px(h))
                    .child(seg_text(&g, h / 2.0, 0.0, h * 0.9, col))
                    .into_any_element(),
                Look::Race => {
                    let (fill_col, text_col) = if shift_now {
                        (col, ink.ground)
                    } else {
                        (ink.primary, ink.on_primary)
                    };
                    let tw = h * 1.2;
                    div()
                        .relative()
                        .w(px(tw))
                        .h(px(h))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            canvas(
                                |_, _, _| (),
                                move |b: Bounds<Pixels>, _, w, _| {
                                    fill(w, b.origin, &skew(0.0, 0.0, tw, h), fill_col)
                                },
                            )
                            .absolute()
                            .size_full(),
                        )
                        .child(div().relative().child(figure(g, h * 0.8, text_col)))
                        .into_any_element()
                }
                Look::GlassCockpit => div()
                    .relative()
                    .w(px(h * 0.9))
                    .h(px(h))
                    .border_2()
                    .border_color(ink.text)
                    .child(
                        text_box(h * 0.45, h * 0.08, h * 0.9)
                            .text_size(px(h * 0.7))
                            .text_color(col)
                            .child(g),
                    )
                    .into_any_element(),
                _ => div()
                    .text_size(px(h))
                    .line_height(px(h))
                    .font_weight(match look {
                        Look::Halo => FontWeight::LIGHT,
                        Look::Sport => FontWeight::MEDIUM,
                        _ => FontWeight::BOLD,
                    })
                    .text_color(col)
                    .child(g)
                    .into_any_element(),
            };
            el = el.child(gear);
        }
        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lights_fill_from_start_to_shift() {
        assert_eq!(lit_lights(4000.0, 5000.0, 7000.0, 10), 0);
        assert_eq!(lit_lights(5000.0, 5000.0, 7000.0, 10), 0);
        assert_eq!(lit_lights(5001.0, 5000.0, 7000.0, 10), 1);
        assert_eq!(lit_lights(6000.0, 5000.0, 7000.0, 10), 5);
        assert_eq!(lit_lights(7000.0, 5000.0, 7000.0, 10), 10);
        assert_eq!(lit_lights(9000.0, 5000.0, 7000.0, 10), 10);
        assert_eq!(lit_lights(f32::NAN, 5000.0, 7000.0, 10), 0);
        assert_eq!(lit_lights(6000.0, 6000.0, 6000.0, 4), 0);
        assert_eq!(lit_lights(6001.0, 6000.0, 6000.0, 4), 4);
    }

    #[test]
    fn lights_run_green_amber_red() {
        let tones: Vec<Tone> = (0..10).map(|i| light_tone(i, 10)).collect();
        assert_eq!(tones[0], Tone::Success);
        assert_eq!(tones[3], Tone::Success);
        assert_eq!(tones[4], Tone::Warning);
        assert_eq!(tones[6], Tone::Warning);
        assert_eq!(tones[7], Tone::Danger);
        assert_eq!(tones[9], Tone::Danger);
    }
}
