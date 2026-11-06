//! The nixie look of the pods: glow tubes and neon on a riveted chassis. Tyre pressures in tubes
//! with a warning bulb each, rows of tubes over bargraph tubes, a large neon bulb for a notice, a
//! tuning eye for G-force, a bank of bargraph tubes for media and a ring of cathodes for the
//! compass. Every colour comes from the theme.

use super::detailed::{case, label, layer, tone_or};
use super::*;
use crate::shapes::{band, circle, Ink};
use crate::tubes::{bargraph, bulb, chassis, dekatron, glow, tubes, tubes_at};

/// The heading of a nixie pod: small capitals, like an engraved strip.
fn title(s: f32, ink: Ink, text: &str) -> gpui::Div {
    label(
        s / 2.0,
        s * 0.075,
        s * 0.7,
        s * 0.034,
        ink.muted,
        text.to_uppercase(),
    )
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    /// Four pressures in tubes, one per corner, each with a bulb that lights when it is low.
    pub(super) fn nixie(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let neon = glow(ink);
        let h = s * 0.13;
        let spots = [
            (c - s * 0.2, s * 0.2),
            (c + s * 0.2, s * 0.2),
            (c - s * 0.2, s * 0.52),
            (c + s * 0.2, s * 0.52),
        ];
        let lows: Vec<bool> = self.bars.iter().map(|b| pm::is_low(*b, self.low)).collect();
        let lows_p = lows.clone();
        let warn = ink.warn;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            for (i, &(x, y)) in spots.iter().enumerate() {
                let bx = if i % 2 == 0 {
                    x - s * 0.13
                } else {
                    x + s * 0.13
                };
                bulb(
                    w,
                    o,
                    (bx, y + h + s * 0.04),
                    s * 0.018,
                    warn,
                    lows_p[i],
                    ink,
                );
            }
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(title(s, ink, &self.heading));
        let names = ["FL", "FR", "RL", "RR"];
        for (i, &(x, y)) in spots.iter().enumerate() {
            el = el
                .child(tubes_at(
                    &pm::pressure_text(self.bars[i]),
                    x,
                    y,
                    h,
                    neon,
                    ink,
                ))
                .child(label(
                    x,
                    y + h + s * 0.025,
                    s * 0.2,
                    s * 0.03,
                    if lows[i] { ink.warn } else { ink.muted },
                    names[i],
                ));
        }
        let foot = if self.note.is_empty() {
            self.unit.to_uppercase()
        } else {
            self.note.to_uppercase()
        };
        el.child(label(c, s * 0.84, s * 0.8, s * 0.032, ink.muted, foot))
            .into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    /// Each row's figure in tubes with its unit beside, the icon glowing on the left and a bar as
    /// a bargraph tube under it.
    pub(super) fn nixie(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let neon = glow(ink);
        let n = self.rows.len().max(1);
        let has_heading = !self.heading.is_empty();
        let top = if has_heading { s * 0.13 } else { s * 0.09 };
        let pitch = ((s * 0.84 - top) / n as f32).min(s * 0.22);
        let (x0, x1) = (s * 0.1, s * 0.9);
        let bars: Vec<Option<(f32, Hsla)>> = self
            .rows
            .iter()
            .map(|r| r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            for (i, bar) in bars.iter().enumerate() {
                if let Some((f, col)) = bar {
                    let y = top + pitch * i as f32 + pitch * 0.72;
                    bargraph(
                        w,
                        o,
                        (x0 + s * 0.1, y, x1 - x0 - s * 0.1, pitch * 0.14),
                        *f,
                        false,
                        *col,
                        ink,
                    );
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if has_heading {
            el = el.child(title(s, ink, &self.heading));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let has_bar = r.bar.is_some();
            let h = (pitch * if has_bar { 0.55 } else { 0.7 }).min(s * 0.11);
            let (fig, rest) = pm::split_figure(&r.value);
            let value: AnyElement = if fig.is_empty() {
                div()
                    .text_size(px(s * 0.045))
                    .text_color(neon)
                    .child(r.value.to_uppercase())
                    .into_any_element()
            } else {
                div()
                    .flex()
                    .items_end()
                    .gap(px(s * 0.015))
                    .child(tubes(fig, h, neon, ink))
                    .child(
                        div()
                            .text_size(px(s * 0.03))
                            .text_color(ink.muted)
                            .child(rest.to_uppercase()),
                    )
                    .into_any_element()
            };
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(y + pitch * 0.04))
                    .w(px(x1 - x0))
                    .h(px(h))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(r.icon.element(s * 0.055, neon))
                    .child(value),
            );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    /// A large neon bulb in the tone with the symbol dark on it, the title and message under it
    /// and a row of small bulbs lit in turn.
    pub(super) fn nixie(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let (by, br) = (s * 0.27, s * 0.13);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            bulb(w, o, (c, by), br, col, true, ink);
            for i in 0..7 {
                let x = c + (i as f32 - 3.0) * s * 0.07;
                bulb(w, o, (x, s * 0.82), s * 0.016, col, i % 2 == 0, ink);
            }
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(c - br * 0.55))
                    .top(px(by - br * 0.55))
                    .child(self.icon.element(br * 1.1, ink.face)),
            )
            .child(label(
                c,
                s * 0.45,
                s * 0.8,
                s * 0.05,
                col,
                self.title.to_uppercase(),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.12))
                    .top(px(s * 0.54))
                    .w(px(s * 0.76))
                    .text_center()
                    .text_size(px(s * 0.036))
                    .text_color(ink.muted)
                    .child(self.text.clone()),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

/// The shadow of a tuning eye for a force of `total` of the limit: half its width in turns,
/// wide at rest and closing as the force grows. Pure.
pub(crate) fn eye_shadow(total: f32) -> f32 {
    0.16 * (1.0 - total.clamp(0.0, 1.0)) + 0.01
}

impl GForcePod {
    /// A tuning eye: a glowing disc whose dark wedge points the way of the force and closes as it
    /// grows, the total in tubes under it.
    pub(super) fn nixie(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (ey, er) = (s * 0.37, s * 0.21);
        let neon = glow(ink);
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let total = gx.hypot(gy).min(1.0);
        // Turn 0 is up, clockwise: forward force points up, rightward force to the right.
        let way = if total > 1e-3 {
            (gx.atan2(gy) / std::f32::consts::TAU).rem_euclid(1.0)
        } else {
            0.0
        };
        let half = eye_shadow(total);
        let eye = ink.ok;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            fill(w, o, &circle(c, ey, er * 1.12), ink.line);
            for k in 1..=4 {
                fill(
                    w,
                    o,
                    &circle(c, ey, er * (1.0 + 0.06 * k as f32)),
                    scale_alpha(eye, 0.05),
                );
            }
            fill(w, o, &circle(c, ey, er), scale_alpha(eye, 0.85));
            fill(
                w,
                o,
                &band(c, ey, er * 0.32, er, way - half, way + half),
                ink.face,
            );
            fill(w, o, &circle(c, ey, er * 0.32), ink.line);
            fill(w, o, &circle(c, ey, er * 0.18), scale_alpha(ink.text, 0.15));
        };
        let figure = pm::g_text(pm::g_total(self.lateral, self.longitudinal));
        case(s)
            .child(layer(paint))
            .child(title(s, ink, "G-force"))
            .child(tubes_at(&figure, c, s * 0.62, s * 0.13, neon, ink))
            .child(label(
                s * 0.22,
                s * 0.82,
                s * 0.26,
                s * 0.03,
                ink.muted,
                format!("LAT {}", pm::g_text(self.lateral.abs())),
            ))
            .child(label(
                s * 0.78,
                s * 0.82,
                s * 0.26,
                s * 0.03,
                ink.muted,
                format!("LON {}", pm::g_text(self.longitudinal.abs())),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    /// A bank of upright bargraph tubes as the spectrum, the title glowing, the elapsed time in
    /// tubes and the progress in a long bargraph tube.
    pub(super) fn nixie(self, ink: Ink) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let neon = glow(ink);
        let f = pm::played(&self.time);
        let bands = pm::spectrum(&self.title, 10);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            let (x0, y0, wd, h) = (s * 0.13, s * 0.1, s * 0.74, s * 0.22);
            let cw = wd / bands.len() as f32;
            for (i, b) in bands.iter().enumerate() {
                bargraph(
                    w,
                    o,
                    (x0 + cw * i as f32 + cw * 0.2, y0, cw * 0.6, h),
                    *b,
                    true,
                    neon,
                    ink,
                );
            }
            bargraph(
                w,
                o,
                (s * 0.12, s * 0.8, s * 0.76, s * 0.05),
                f,
                false,
                neon,
                ink,
            );
        };
        case(s)
            .child(layer(paint))
            .child(label(
                c,
                s * 0.36,
                s * 0.76,
                s * 0.045,
                neon,
                self.title.clone(),
            ))
            .child(label(
                c,
                s * 0.43,
                s * 0.76,
                s * 0.032,
                ink.muted,
                self.artist.clone(),
            ))
            .child(tubes_at(
                pm::elapsed(&self.time),
                c,
                s * 0.52,
                s * 0.15,
                neon,
                ink,
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

impl CompassPod {
    /// A ring of 36 glowing cathodes with the heading's one lit, the bearing in tubes in the
    /// middle and the cardinal under it.
    pub(super) fn nixie(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let cy = s * 0.45;
        let neon = glow(ink);
        let heading = pm::bearing(self.heading);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            chassis(w, o, s, ink);
            dekatron(w, o, (c, cy), s * 0.27, 36, heading / 360.0, neon, ink);
        };
        let mut el = case(s)
            .child(layer(paint))
            .child(tubes_at(
                &format!("{:03.0}", heading),
                c,
                cy - s * 0.1,
                s * 0.13,
                neon,
                ink,
            ))
            .child(label(
                c,
                cy + s * 0.06,
                s * 0.3,
                s * 0.05,
                neon,
                pm::cardinal(heading),
            ));
        for (d, name) in [(0.0, "N"), (90.0, "E"), (180.0, "S"), (270.0, "W")] {
            let (x, y) = polar(c, cy, s * 0.355, d / 360.0);
            el = el.child(label(x, y - s * 0.02, s * 0.08, s * 0.034, ink.muted, name));
        }
        el.child(label(
            c,
            s * 0.87,
            s * 0.7,
            s * 0.03,
            ink.muted,
            format!("ALT {}", self.height.to_uppercase()),
        ))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_eye_closes_as_the_force_grows() {
        assert!(eye_shadow(0.0) > eye_shadow(0.5));
        assert!(eye_shadow(0.5) > eye_shadow(1.0));
        assert_eq!(eye_shadow(2.0), eye_shadow(1.0));
        assert!(eye_shadow(1.0) > 0.0, "a sliver stays dark at the limit");
    }
}
