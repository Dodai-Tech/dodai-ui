//! The chronograph and heads-up looks of the pods, and the dispatch to the others.
//! [`Look::Chronograph`] puts the reading in a watch case: a knurled bezel, fine scales, tapered
//! needles and a glass. [`Look::HeadsUp`] draws it as a heads-up display: a ring of dashes with
//! brackets, chamfered frames and segment bars. Both take every colour from the theme.

use super::*;
use crate::shapes::{
    band, cells, chamfered, chrono_case, chrono_glass, circle, hud_ring, needle, star, Ink,
};

/// A canvas over the whole pod.
pub(super) fn layer(
    paint: impl Fn(&mut Window, gpui::Point<Pixels>) + 'static,
) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

/// The square box of a pod of `s`.
pub(super) fn case(s: f32) -> gpui::Div {
    div().relative().w(px(s)).h(px(s))
}

/// A line of text centred on `x`, its top at `y`.
pub(super) fn label(
    x: f32,
    y: f32,
    w: f32,
    size: f32,
    col: Hsla,
    text: impl Into<SharedString>,
) -> gpui::Div {
    text_box(x, y, w)
        .text_size(px(size))
        .text_color(col)
        .child(text.into())
}

/// A tone's colour, the accent for [`Tone::Off`].
pub(super) fn tone_or(t: &dodai_core::Theme, tone: Tone, ink: Ink) -> Hsla {
    match tone {
        Tone::Off => ink.primary,
        other => other.color(t),
    }
}

// ───────────────────────────────── TyrePod ─────────────────────────────────

impl TyrePod {
    pub(super) fn detailed(self, t: &dodai_core::Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Classic | Look::Chronograph => self.chrono(t),
        }
    }

    /// Four small dials in a watch case, the low band hatched in the warning colour.
    fn chrono(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = crate::gauges::SWEEP;
        let low_f = pm::pressure_frac(Some(self.low), self.low).unwrap_or(0.5);
        let r = s * 0.125;
        let centres = [
            (c - s * 0.14, c - s * 0.07),
            (c + s * 0.14, c - s * 0.07),
            (c - s * 0.14, c + s * 0.21),
            (c + s * 0.14, c + s * 0.21),
        ];
        let reads: Vec<(Option<f32>, bool)> = self
            .bars
            .iter()
            .map(|b| (pm::pressure_frac(*b, self.low), pm::is_low(*b, self.low)))
            .collect();
        let reads_paint = reads.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for (&(x, y), (f, low)) in centres.iter().zip(&reads_paint) {
                fill(w, o, &circle(x, y, r), scale_alpha(ink.text, 0.05));
                stroke(w, o, &circle(x, y, r), s * 0.005, ink.line, true);
                // The low band, hatched.
                let (ta, tz) = (start, start + span * low_f);
                fill(
                    w,
                    o,
                    &band(x, y, r * 0.72, r * 0.9, ta, tz),
                    scale_alpha(ink.warn, 0.2),
                );
                let n = ((tz - ta) * 120.0).ceil().max(2.0) as usize;
                for k in 0..n {
                    let u = ta + (tz - ta) * k as f32 / n as f32;
                    stroke(
                        w,
                        o,
                        &[
                            polar(x, y, r * 0.72, u),
                            polar(x, y, r * 0.9, (u + 0.012).min(tz)),
                        ],
                        s * 0.004,
                        ink.warn,
                        false,
                    );
                }
                for i in 0..=20 {
                    let a = start + span * i as f32 / 20.0;
                    let r0 = if i % 5 == 0 { r * 0.7 } else { r * 0.8 };
                    stroke(
                        w,
                        o,
                        &[polar(x, y, r0, a), polar(x, y, r * 0.92, a)],
                        if i % 5 == 0 { s * 0.006 } else { s * 0.003 },
                        if i % 5 == 0 { ink.text } else { ink.muted },
                        false,
                    );
                }
                if let Some(f) = f {
                    let col = if *low { ink.warn } else { ink.primary };
                    fill(
                        w,
                        o,
                        &needle(x, y, start + span * f, r * 0.85, r * 0.2, s * 0.012),
                        col,
                    );
                }
                fill(w, o, &circle(x, y, s * 0.016), ink.text);
            }
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint)).child(label(
            c,
            s * 0.15,
            s * 0.6,
            s * 0.045,
            ink.muted,
            self.heading.clone(),
        ));
        let names = ["FL", "FR", "RL", "RR"];
        for ((&(x, y), (_, low)), (name, bar)) in
            centres.iter().zip(&reads).zip(names.iter().zip(self.bars))
        {
            let side = if x < c { -1.0 } else { 1.0 };
            el = el
                .child(label(
                    x + side * (r + s * 0.04),
                    y - s * 0.02,
                    s * 0.08,
                    s * 0.032,
                    ink.muted,
                    *name,
                ))
                .child(label(
                    x,
                    y + r * 0.42,
                    r * 2.0,
                    s * 0.045,
                    if *low { ink.warn } else { ink.text },
                    pm::pressure_text(bar),
                ));
        }
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                s * 0.83,
                s * 0.7,
                s * 0.032,
                ink.muted,
                self.note.clone(),
            ));
        }
        el.into_any_element()
    }

    /// Four chamfered boxes on a heads-up ring, each with a segment bar.
    fn hud(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let (bw, bh) = (s * 0.27, s * 0.19);
        let boxes = [
            (c - s * 0.01 - bw, c - s * 0.13),
            (c + s * 0.01, c - s * 0.13),
            (c - s * 0.01 - bw, c + s * 0.08),
            (c + s * 0.01, c + s * 0.08),
        ];
        let reads: Vec<(Option<f32>, bool)> = self
            .bars
            .iter()
            .map(|b| (pm::pressure_frac(*b, self.low), pm::is_low(*b, self.low)))
            .collect();
        let reads_paint = reads.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            for (&(x, y), (f, low)) in boxes.iter().zip(&reads_paint) {
                let col = if *low { ink.warn } else { ink.primary };
                let frame = chamfered(x, y, bw, bh, s * 0.025);
                fill(w, o, &frame, scale_alpha(col, 0.1));
                stroke(w, o, &frame, s * 0.005, scale_alpha(col, 0.8), true);
                let lit = f.map_or(0, |f| pm::segments_lit(f, 8));
                cells(
                    w,
                    o,
                    (x + bw * 0.1, y + bh * 0.74, bw * 0.8, bh * 0.12),
                    8,
                    lit,
                    col,
                    scale_alpha(ink.text, 0.15),
                );
            }
            // Crosshair between the boxes.
            let k = s * 0.025;
            stroke(
                w,
                o,
                &[(c - k, c - s * 0.025), (c + k, c - s * 0.025)],
                s * 0.004,
                ink.muted,
                false,
            );
            stroke(
                w,
                o,
                &[(c, c - s * 0.025 - k), (c, c - s * 0.025 + k)],
                s * 0.004,
                ink.muted,
                false,
            );
        };
        let mut el = case(s).child(layer(paint)).child(label(
            c,
            s * 0.19,
            s * 0.6,
            s * 0.038,
            ink.primary,
            self.heading.to_uppercase(),
        ));
        let names = ["FL", "FR", "RL", "RR"];
        for ((&(x, y), (_, low)), (name, bar)) in
            boxes.iter().zip(&reads).zip(names.iter().zip(self.bars))
        {
            el = el.child(
                div()
                    .absolute()
                    .left(px(x + bw * 0.1))
                    .top(px(y + bh * 0.08))
                    .w(px(bw * 0.8))
                    .flex()
                    .items_baseline()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(s * 0.03))
                            .text_color(ink.muted)
                            .child(*name),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.062))
                            .text_color(if *low { ink.warn } else { ink.text })
                            .font_weight(FontWeight::MEDIUM)
                            .child(pm::pressure_text(bar)),
                    ),
            );
        }
        if !self.note.is_empty() {
            el = el.child(label(
                c,
                s * 0.8,
                s * 0.6,
                s * 0.03,
                ink.muted,
                self.note.clone(),
            ));
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── RowsPod ─────────────────────────────────

impl RowsPod {
    pub(super) fn detailed(self, t: &dodai_core::Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Classic | Look::Chronograph => self.chrono(t),
        }
    }

    /// Each row on a fine ruler with a pointer, or a dotted leader when it has no level.
    fn chrono(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let pitch = (s * 0.5 / n as f32).min(s * 0.13);
        let top = c - pitch * n as f32 / 2.0
            + if self.heading.is_empty() {
                0.0
            } else {
                s * 0.03
            };
        let (x0, x1) = (c - s * 0.3, c + s * 0.3);
        let (rx0, rx1) = (x0 + s * 0.09, x1 - s * 0.16);
        let rows: Vec<(Option<(f32, Hsla)>, bool)> = self
            .rows
            .iter()
            .map(|r| (r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))), r.span))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for (i, (bar, span)) in rows.iter().enumerate() {
                let y = top + pitch * (i as f32 + 0.5);
                if i > 0 {
                    stroke(
                        w,
                        o,
                        &[(x0, y - pitch / 2.0), (x1, y - pitch / 2.0)],
                        s * 0.003,
                        scale_alpha(ink.text, 0.1),
                        false,
                    );
                }
                match bar {
                    Some((f, col)) => {
                        let base = y + pitch * 0.12;
                        for k in 0..=20 {
                            let x = rx0 + (rx1 - rx0) * k as f32 / 20.0;
                            let h = if k % 5 == 0 {
                                pitch * 0.28
                            } else {
                                pitch * 0.14
                            };
                            stroke(
                                w,
                                o,
                                &[(x, base - h), (x, base)],
                                s * 0.003,
                                ink.muted,
                                false,
                            );
                        }
                        let fx = rx0 + (rx1 - rx0) * f;
                        stroke(
                            w,
                            o,
                            &[(rx0, base + s * 0.008), (fx, base + s * 0.008)],
                            s * 0.008,
                            *col,
                            false,
                        );
                        let k = s * 0.016;
                        fill(
                            w,
                            o,
                            &[
                                (fx, base - pitch * 0.05),
                                (fx - k, base - pitch * 0.05 - k * 1.4),
                                (fx + k, base - pitch * 0.05 - k * 1.4),
                            ],
                            ink.text,
                        );
                        if *span {
                            stroke(
                                w,
                                o,
                                &[(rx0, base - pitch * 0.3), (rx0, base + s * 0.012)],
                                s * 0.008,
                                ink.info,
                                false,
                            );
                            stroke(
                                w,
                                o,
                                &[(rx1, base - pitch * 0.3), (rx1, base + s * 0.012)],
                                s * 0.008,
                                ink.danger,
                                false,
                            );
                        }
                    }
                    None => {
                        let mut x = rx0;
                        while x < rx1 + s * 0.05 {
                            fill(w, o, &circle(x, y + pitch * 0.08, s * 0.004), ink.muted);
                            x += s * 0.022;
                        }
                    }
                }
            }
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(label(
                c,
                top - s * 0.075,
                s * 0.6,
                s * 0.042,
                ink.muted,
                self.heading.clone(),
            ));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * (i as f32 + 0.5);
            el = el
                .child(
                    div()
                        .absolute()
                        .left(px(x0))
                        .top(px(y - s * 0.035))
                        .child(r.icon.element(s * 0.06, ink.muted)),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(x1 - s * 0.15))
                        .top(px(y - s * 0.03))
                        .w(px(s * 0.15))
                        .flex()
                        .justify_end()
                        .whitespace_nowrap()
                        .text_size(px(s * 0.045))
                        .text_color(ink.text)
                        .child(r.value.clone()),
                );
        }
        el.into_any_element()
    }

    /// Each row in a chamfered strip with a segment bar.
    fn hud(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let n = self.rows.len().max(1);
        let pitch = (s * 0.52 / n as f32).min(s * 0.13);
        let top = c - pitch * n as f32 / 2.0
            + if self.heading.is_empty() {
                0.0
            } else {
                s * 0.03
            };
        let (x0, x1) = (c - s * 0.31, c + s * 0.31);
        let rows: Vec<(Option<(f32, Hsla)>, bool)> = self
            .rows
            .iter()
            .map(|r| (r.bar.map(|(f, tone)| (f, tone_or(t, tone, ink))), r.span))
            .collect();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            for (i, (bar, span)) in rows.iter().enumerate() {
                let y = top + pitch * i as f32;
                let h = pitch * 0.82;
                let frame = chamfered(x0, y, x1 - x0, h, h * 0.3);
                fill(w, o, &frame, scale_alpha(ink.primary, 0.07));
                stroke(
                    w,
                    o,
                    &frame,
                    s * 0.004,
                    scale_alpha(ink.primary, 0.55),
                    true,
                );
                let (bx0, bx1) = (x0 + s * 0.1, x1 - s * 0.17);
                match bar {
                    Some((f, col)) => {
                        cells(
                            w,
                            o,
                            (bx0, y + h * 0.36, bx1 - bx0, h * 0.28),
                            12,
                            pm::segments_lit(*f, 12),
                            *col,
                            scale_alpha(ink.text, 0.14),
                        );
                        if *span {
                            stroke(
                                w,
                                o,
                                &[
                                    (bx0 - s * 0.01, y + h * 0.25),
                                    (bx0 - s * 0.01, y + h * 0.75),
                                ],
                                s * 0.006,
                                ink.info,
                                false,
                            );
                            stroke(
                                w,
                                o,
                                &[
                                    (bx1 + s * 0.012, y + h * 0.25),
                                    (bx1 + s * 0.012, y + h * 0.75),
                                ],
                                s * 0.006,
                                ink.danger,
                                false,
                            );
                        }
                    }
                    None => {
                        let mut x = bx0;
                        while x < bx1 {
                            stroke(
                                w,
                                o,
                                &[(x, y + h * 0.5), (x + s * 0.012, y + h * 0.5)],
                                s * 0.004,
                                ink.muted,
                                false,
                            );
                            x += s * 0.022;
                        }
                    }
                }
            }
        };
        let mut el = case(s).child(layer(paint));
        if !self.heading.is_empty() {
            el = el.child(label(
                c,
                top - s * 0.07,
                s * 0.6,
                s * 0.036,
                ink.primary,
                self.heading.to_uppercase(),
            ));
        }
        for (i, r) in self.rows.iter().enumerate() {
            let y = top + pitch * i as f32;
            let h = pitch * 0.82;
            el = el.child(
                div()
                    .absolute()
                    .left(px(x0 + s * 0.03))
                    .top(px(y))
                    .w(px(x1 - x0 - s * 0.06))
                    .h(px(h))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(r.icon.element(s * 0.05, ink.text))
                    .child(
                        div()
                            .whitespace_nowrap()
                            .text_size(px(s * 0.042))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(r.value.clone()),
                    ),
            );
        }
        el.into_any_element()
    }
}

// ───────────────────────────────── NoticePod ─────────────────────────────────

impl NoticePod {
    pub(super) fn detailed(self, t: &dodai_core::Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Classic | Look::Chronograph => self.chrono(t),
        }
    }

    /// The symbol in a double ring of the tone, minute marks around, title and advice under it.
    fn chrono(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let (ix, iy, ir) = (c, c - s * 0.12, s * 0.11);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for i in 0..60 {
                let a = i as f32 / 60.0;
                let long = i % 5 == 0;
                stroke(
                    w,
                    o,
                    &[
                        polar(c, c, r_face - if long { s * 0.05 } else { s * 0.025 }, a),
                        polar(c, c, r_face - s * 0.008, a),
                    ],
                    if long { s * 0.008 } else { s * 0.003 },
                    if long { col } else { ink.muted },
                    false,
                );
            }
            fill(w, o, &circle(ix, iy, ir), scale_alpha(col, 0.12));
            stroke(w, o, &circle(ix, iy, ir), s * 0.008, col, true);
            stroke(
                w,
                o,
                &circle(ix, iy, ir * 1.18),
                s * 0.003,
                scale_alpha(col, 0.6),
                true,
            );
            chrono_glass(w, o, s, r_face, ink);
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(ix - ir * 0.6))
                    .top(px(iy - ir * 0.6))
                    .child(self.icon.element(ir * 1.2, col)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - s * 0.3))
                    .top(px(c + s * 0.04))
                    .w(px(s * 0.6))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.015))
                    .child(
                        div()
                            .text_size(px(s * 0.058))
                            .text_color(ink.text)
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.title.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.04))
                            .text_color(ink.muted)
                            .text_center()
                            .child(self.text.clone()),
                    ),
            )
            .into_any_element()
    }

    /// A hazard band, the symbol in a chamfered frame and the title in capitals.
    fn hud(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let col = tone_or(t, self.tone, ink);
        let (hx, hy, hw, hh) = (c - s * 0.24, s * 0.2, s * 0.48, s * 0.055);
        let (fx, fy, fs) = (c - s * 0.09, s * 0.3, s * 0.18);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, col);
            // Hazard stripes inside a chamfered band.
            let frame = chamfered(hx, hy, hw, hh, hh * 0.5);
            fill(w, o, &frame, scale_alpha(col, 0.12));
            let step = hh * 0.9;
            let mut x = hx + hh * 0.5;
            while x + step * 0.5 + hh < hx + hw - hh * 0.5 {
                fill(
                    w,
                    o,
                    &[
                        (x + hh, hy),
                        (x + hh + step * 0.5, hy),
                        (x + step * 0.5, hy + hh),
                        (x, hy + hh),
                    ],
                    col,
                );
                x += step;
            }
            stroke(w, o, &frame, s * 0.004, col, true);
            let f = chamfered(fx, fy, fs, fs, fs * 0.22);
            fill(w, o, &f, scale_alpha(col, 0.12));
            stroke(w, o, &f, s * 0.007, col, true);
            for (x, sx) in [(fx - s * 0.035, -1.0_f32), (fx + fs + s * 0.035, 1.0)] {
                stroke(
                    w,
                    o,
                    &[(x, fy + fs * 0.5), (x - sx * s * 0.04, fy + fs * 0.5)],
                    s * 0.005,
                    ink.muted,
                    false,
                );
            }
        };
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(fx + fs * 0.2))
                    .top(px(fy + fs * 0.2))
                    .child(self.icon.element(fs * 0.6, col)),
            )
            .child(
                div()
                    .absolute()
                    .left(px(c - s * 0.3))
                    .top(px(fy + fs + s * 0.04))
                    .w(px(s * 0.6))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.015))
                    .child(
                        div()
                            .text_size(px(s * 0.05))
                            .text_color(col)
                            .font_weight(FontWeight::BOLD)
                            .child(self.title.to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.038))
                            .text_color(ink.text)
                            .text_center()
                            .child(self.text.clone()),
                    ),
            )
            .into_any_element()
    }
}

// ───────────────────────────────── GForcePod ─────────────────────────────────

impl GForcePod {
    pub(super) fn detailed(self, t: &dodai_core::Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Classic | Look::Chronograph => self.chrono(t),
        }
    }

    /// A friction circle: rings with their g, a fine cross, the vector and a haloed dot.
    fn chrono(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let reach = s * 0.3;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let (dx, dy) = (c + gx * reach, c - gy * reach);
        let limit = self.limit;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for k in 1..=3 {
                stroke(
                    w,
                    o,
                    &circle(c, c, reach * k as f32 / 3.0),
                    if k == 3 { s * 0.006 } else { s * 0.003 },
                    if k == 3 {
                        ink.text
                    } else {
                        scale_alpha(ink.text, 0.3)
                    },
                    true,
                );
            }
            for (ax, ay) in [(1.0_f32, 0.0_f32), (0.0, 1.0)] {
                stroke(
                    w,
                    o,
                    &[
                        (c - ax * reach, c - ay * reach),
                        (c + ax * reach, c + ay * reach),
                    ],
                    s * 0.003,
                    ink.muted,
                    false,
                );
                for i in -15i32..=15 {
                    if i == 0 {
                        continue;
                    }
                    let d = reach * i as f32 / 15.0;
                    let h = if i % 5 == 0 { s * 0.02 } else { s * 0.01 };
                    let (px_, py_) = (c + ax * d, c + ay * d);
                    stroke(
                        w,
                        o,
                        &[(px_ - ay * h, py_ - ax * h), (px_ + ay * h, py_ + ax * h)],
                        s * 0.003,
                        ink.muted,
                        false,
                    );
                }
            }
            stroke(
                w,
                o,
                &[(c, c), (dx, dy)],
                s * 0.008,
                scale_alpha(ink.primary, 0.7),
                false,
            );
            for (k, a) in [(3.0_f32, 0.12_f32), (2.0, 0.25)] {
                fill(
                    w,
                    o,
                    &circle(dx, dy, s * 0.018 * k),
                    scale_alpha(ink.primary, a),
                );
            }
            fill(w, o, &circle(dx, dy, s * 0.02), ink.primary);
            fill(w, o, &circle(dx, dy, s * 0.008), ink.text);
            fill(w, o, &circle(c, c, s * 0.012), ink.text);
            chrono_glass(w, o, s, r_face, ink);
        };
        let g = pm::g_text;
        let total = (self.lateral.powi(2) + self.longitudinal.powi(2)).sqrt();
        let mut el = case(s).child(layer(paint));
        for k in 1..=3 {
            let (x, y) = polar(c, c, reach * k as f32 / 3.0, 0.125);
            el = el.child(label(
                x + s * 0.02,
                y - s * 0.035,
                s * 0.1,
                s * 0.028,
                ink.muted,
                format!("{:.1}", limit * k as f32 / 3.0),
            ));
        }
        let side = |x: f32, y: f32, v: f32| label(x, y, s * 0.12, s * 0.04, ink.text, g(v));
        el.child(side(c, c - reach - s * 0.075, self.longitudinal.max(0.0)))
            .child(side(c, c + reach + s * 0.01, (-self.longitudinal).max(0.0)))
            .child(side(
                c - reach - s * 0.06,
                c - s * 0.025,
                (-self.lateral).max(0.0),
            ))
            .child(side(
                c + reach + s * 0.06,
                c - s * 0.025,
                self.lateral.max(0.0),
            ))
            .child(
                div()
                    .absolute()
                    .left(px(c - s * 0.075))
                    .top(px(c + reach + s * 0.065))
                    .px(px(s * 0.015))
                    .rounded(px(s * 0.01))
                    .border_1()
                    .border_color(ink.line)
                    .bg(scale_alpha(ink.text, 0.06))
                    .text_size(px(s * 0.04))
                    .text_color(ink.primary)
                    .child(format!("{total:.2} g")),
            )
            .into_any_element()
    }

    /// A dashed grid with brackets, edge markers for each axis and a diamond target.
    fn hud(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let h = s * 0.25;
        let (gx, gy) = (
            pm::g_offset(self.lateral, self.limit),
            pm::g_offset(self.longitudinal, self.limit),
        );
        let (dx, dy) = (c + gx * h, c - gy * h);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            for i in -4i32..=4 {
                let d = h * i as f32 / 4.0;
                let a = if i == 0 { 0.5 } else { 0.18 };
                let mut x = c - h;
                while x < c + h {
                    let e = (x + s * 0.012).min(c + h);
                    stroke(
                        w,
                        o,
                        &[(x, c + d), (e, c + d)],
                        s * 0.003,
                        scale_alpha(ink.text, a),
                        false,
                    );
                    stroke(
                        w,
                        o,
                        &[(c + d, x), (c + d, e)],
                        s * 0.003,
                        scale_alpha(ink.text, a),
                        false,
                    );
                    x += s * 0.022;
                }
            }
            // Corner brackets of the grid.
            let k = s * 0.05;
            for (sx, sy) in [(-1.0_f32, -1.0_f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let (x, y) = (c + sx * h, c + sy * h);
                stroke(
                    w,
                    o,
                    &[(x - sx * k, y), (x, y), (x, y - sy * k)],
                    s * 0.008,
                    ink.primary,
                    false,
                );
            }
            // Edge markers: lateral on the bottom edge, longitudinal on the right.
            let m = s * 0.02;
            fill(
                w,
                o,
                &[
                    (dx, c + h + s * 0.01),
                    (dx - m, c + h + s * 0.01 + m * 1.5),
                    (dx + m, c + h + s * 0.01 + m * 1.5),
                ],
                ink.text,
            );
            fill(
                w,
                o,
                &[
                    (c + h + s * 0.01, dy),
                    (c + h + s * 0.01 + m * 1.5, dy - m),
                    (c + h + s * 0.01 + m * 1.5, dy + m),
                ],
                ink.text,
            );
            stroke(
                w,
                o,
                &[(dx, dy), (dx, c + h)],
                s * 0.003,
                scale_alpha(ink.primary, 0.6),
                false,
            );
            stroke(
                w,
                o,
                &[(dx, dy), (c + h, dy)],
                s * 0.003,
                scale_alpha(ink.primary, 0.6),
                false,
            );
            let d = s * 0.03;
            let diamond = [(dx, dy - d), (dx + d, dy), (dx, dy + d), (dx - d, dy)];
            fill(w, o, &diamond, scale_alpha(ink.primary, 0.3));
            stroke(w, o, &diamond, s * 0.006, ink.primary, true);
        };
        let total = (self.lateral.powi(2) + self.longitudinal.powi(2)).sqrt();
        case(s)
            .child(layer(paint))
            .child(label(
                c,
                c - h - s * 0.075,
                s * 0.4,
                s * 0.04,
                ink.primary,
                format!("{total:.2} G"),
            ))
            .child(label(
                c - h * 0.5,
                c + h + s * 0.045,
                s * 0.3,
                s * 0.032,
                ink.muted,
                format!("LAT {}", pm::g_text(self.lateral)),
            ))
            .child(label(
                c + h * 0.5,
                c + h + s * 0.045,
                s * 0.3,
                s * 0.032,
                ink.muted,
                format!("LON {}", pm::g_text(self.longitudinal)),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── MediaPod ─────────────────────────────────

impl MediaPod {
    pub(super) fn detailed(mut self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let cover = self.cover.take();
        match self.look {
            Look::HeadsUp => self.hud(ink, cover),
            Look::Vintage => self.vintage(ink),
            Look::Segment => self.digital(ink, t.is_dark()),
            Look::Halo => self.futuristic(ink, cover),
            Look::GlassCockpit => self.cockpit(ink),
            Look::Race => self.race(ink),
            Look::Classic | Look::Chronograph => self.chrono(ink, cover),
        }
    }

    /// The cover as the label of a record, grooves around it, and the progress on the bezel.
    fn chrono(self, ink: Ink, cover: Option<AnyElement>) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let (vx, vy, vr) = (c, c - s * 0.08, s * 0.2);
        let lr = s * 0.085;
        let f = pm::played(&self.time);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            let rp = r_face - s * 0.025;
            for i in 0..=60 {
                let a = i as f32 / 60.0;
                stroke(
                    w,
                    o,
                    &[
                        polar(c, c, rp - if i % 5 == 0 { s * 0.02 } else { s * 0.01 }, a),
                        polar(c, c, rp, a),
                    ],
                    s * 0.003,
                    ink.muted,
                    false,
                );
            }
            stroke(
                w,
                o,
                &turn_arc(c, c, rp + s * 0.008, 0.0, f.max(0.002)),
                s * 0.01,
                ink.primary,
                false,
            );
            let (tip, b1, b2) = (
                polar(c, c, rp - s * 0.03, f),
                polar(c, c, rp - s * 0.065, f - 0.012),
                polar(c, c, rp - s * 0.065, f + 0.012),
            );
            fill(w, o, &[tip, b1, b2], ink.primary);
            fill(w, o, &circle(vx, vy, vr), scale_alpha(ink.text, 0.1));
            fill(w, o, &circle(vx, vy, lr), scale_alpha(ink.primary, 0.35));
            for k in 0..9 {
                stroke(
                    w,
                    o,
                    &circle(vx, vy, lr + (vr - lr) * (k as f32 + 0.5) / 9.0),
                    s * 0.002,
                    scale_alpha(ink.text, 0.18),
                    true,
                );
            }
            // A sheen across the grooves.
            fill(
                w,
                o,
                &band(vx, vy, lr, vr, 0.05, 0.17),
                scale_alpha(ink.text, 0.06),
            );
            fill(
                w,
                o,
                &band(vx, vy, lr, vr, 0.55, 0.67),
                scale_alpha(ink.text, 0.06),
            );
            chrono_glass(w, o, s, r_face, ink);
        };
        let cover = cover.unwrap_or_else(|| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(Icon::new(IconId::Music).size(lr * 0.8).color(ink.text))
                .into_any_element()
        });
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(vx - lr * 0.62))
                    .top(px(vy - lr * 0.62))
                    .w(px(lr * 1.24))
                    .h(px(lr * 1.24))
                    .overflow_hidden()
                    .child(cover),
            )
            .child(label(
                c,
                c + s * 0.15,
                s * 0.6,
                s * 0.05,
                ink.primary,
                self.title.clone(),
            ))
            .child(label(
                c,
                c + s * 0.215,
                s * 0.6,
                s * 0.038,
                ink.text,
                self.artist.clone(),
            ))
            .child(label(
                c,
                c + s * 0.27,
                s * 0.6,
                s * 0.032,
                ink.muted,
                self.time.clone(),
            ))
            .into_any_element()
    }

    /// The cover in brackets over a spectrum, the title in capitals and a segment progress bar.
    fn hud(self, ink: Ink, cover: Option<AnyElement>) -> AnyElement {
        let s = self.size;
        let c = s / 2.0;
        let side = s * 0.2;
        let (cx0, cy0) = (c - side / 2.0, s * 0.17);
        let f = pm::played(&self.time);
        let bars = pm::spectrum(&self.title, 18);
        let (sx0, sx1, sy) = (c - s * 0.27, c + s * 0.27, cy0 + side + s * 0.12);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            let k = s * 0.03;
            let g = s * 0.012;
            for (x, y, sx, sy) in [
                (cx0 - g, cy0 - g, 1.0_f32, 1.0_f32),
                (cx0 + side + g, cy0 - g, -1.0, 1.0),
                (cx0 - g, cy0 + side + g, 1.0, -1.0),
                (cx0 + side + g, cy0 + side + g, -1.0, -1.0),
            ] {
                stroke(
                    w,
                    o,
                    &[(x + sx * k, y), (x, y), (x, y + sy * k)],
                    s * 0.007,
                    ink.primary,
                    false,
                );
            }
            let n = bars.len();
            let bw = (sx1 - sx0) / n as f32;
            for (i, b) in bars.iter().enumerate() {
                let x = sx0 + bw * i as f32;
                let hgt = s * 0.08 * b;
                let lit = (i as f32 + 0.5) / n as f32 <= f.max(0.02);
                fill(
                    w,
                    o,
                    &[
                        (x + bw * 0.15, sy - hgt),
                        (x + bw * 0.85, sy - hgt),
                        (x + bw * 0.85, sy),
                        (x + bw * 0.15, sy),
                    ],
                    if lit {
                        ink.primary
                    } else {
                        scale_alpha(ink.text, 0.25)
                    },
                );
                fill(
                    w,
                    o,
                    &[
                        (x + bw * 0.15, sy + s * 0.008),
                        (x + bw * 0.85, sy + s * 0.008),
                        (x + bw * 0.85, sy + s * 0.008 + hgt * 0.3),
                        (x + bw * 0.15, sy + s * 0.008 + hgt * 0.3),
                    ],
                    scale_alpha(if lit { ink.primary } else { ink.text }, 0.15),
                );
            }
            cells(
                w,
                o,
                (sx0, c + s * 0.29, sx1 - sx0, s * 0.02),
                20,
                pm::segments_lit(f, 20),
                ink.primary,
                scale_alpha(ink.text, 0.15),
            );
        };
        let cover = cover.unwrap_or_else(|| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .bg(scale_alpha(ink.primary, 0.2))
                .child(
                    Icon::new(IconId::Music)
                        .size(side * 0.45)
                        .color(ink.primary),
                )
                .into_any_element()
        });
        case(s)
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(cx0))
                    .top(px(cy0))
                    .w(px(side))
                    .h(px(side))
                    .overflow_hidden()
                    .child(cover),
            )
            .child(label(
                c,
                sy + s * 0.04,
                s * 0.62,
                s * 0.045,
                ink.text,
                self.title.to_uppercase(),
            ))
            .child(label(
                c,
                sy + s * 0.1,
                s * 0.6,
                s * 0.035,
                ink.muted,
                self.artist.clone(),
            ))
            .child(label(
                c,
                c + s * 0.325,
                s * 0.5,
                s * 0.03,
                ink.primary,
                self.time.clone(),
            ))
            .into_any_element()
    }
}

// ───────────────────────────────── CompassPod ─────────────────────────────────

impl CompassPod {
    pub(super) fn detailed(self, t: &dodai_core::Theme) -> AnyElement {
        match self.look {
            Look::HeadsUp => self.hud(t),
            Look::Vintage => self.vintage(t),
            Look::Segment => self.digital(t),
            Look::Halo => self.futuristic(t),
            Look::GlassCockpit => self.cockpit(t),
            Look::Race => self.race(t),
            Look::Classic | Look::Chronograph => self.chrono(t),
        }
    }

    /// A turning rose: degree ring, numbers every 30°, an eight-point star and a fixed lubber
    /// mark, the bearing in a window.
    fn chrono(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let turn = self.heading / 360.0;
        let r_ring = c - s * 0.085;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            let r_face = chrono_case(w, o, s, ink);
            for i in 0..72 {
                let a = i as f32 / 72.0 - turn;
                let (len, wd) = if i % 6 == 0 {
                    (s * 0.045, s * 0.006)
                } else {
                    (s * 0.022, s * 0.003)
                };
                stroke(
                    w,
                    o,
                    &[polar(c, c, r_ring - len, a), polar(c, c, r_ring, a)],
                    wd,
                    ink.text,
                    false,
                );
            }
            stroke(
                w,
                o,
                &circle(c, c, r_ring - s * 0.13),
                s * 0.003,
                scale_alpha(ink.text, 0.3),
                true,
            );
            // The rose: a long star and a short one between it, each point half lit.
            let (long, short) = (s * 0.17, s * 0.032);
            fill(
                w,
                o,
                &star(c, c, long * 0.62, short, 0.125 - turn, 4),
                scale_alpha(ink.muted, 0.6),
            );
            fill(
                w,
                o,
                &star(c, c, long, short, -turn, 4),
                scale_alpha(ink.text, 0.25),
            );
            for q in 0..4 {
                let a = q as f32 / 4.0 - turn;
                let tip = polar(c, c, long, a);
                let side = polar(c, c, short, a + 0.125);
                fill(
                    w,
                    o,
                    &[(c, c), tip, side],
                    if q == 0 { ink.danger } else { ink.text },
                );
            }
            fill(w, o, &circle(c, c, s * 0.018), ink.face);
            stroke(w, o, &circle(c, c, s * 0.018), s * 0.004, ink.text, true);
            // Lubber mark, fixed at the top.
            let y = c - r_face + s * 0.004;
            fill(
                w,
                o,
                &[(c, y + s * 0.05), (c - s * 0.022, y), (c + s * 0.022, y)],
                ink.primary,
            );
            chrono_glass(w, o, s, r_face, ink);
        };
        let mut el = case(s).child(layer(paint));
        for i in 0..12 {
            let deg = i as f32 * 30.0;
            let txt: SharedString = match i {
                0 => "N".into(),
                3 => "E".into(),
                6 => "S".into(),
                9 => "W".into(),
                _ => format!("{}", i * 3).into(),
            };
            let (x, y) = polar(c, c, r_ring - s * 0.085, deg / 360.0 - turn);
            let col = if i == 0 {
                ink.danger
            } else if i % 3 == 0 {
                ink.text
            } else {
                ink.muted
            };
            el = el.child(label(
                x,
                y - s * 0.025,
                s * 0.1,
                if i % 3 == 0 { s * 0.045 } else { s * 0.032 },
                col,
                txt,
            ));
        }
        el.child(
            div()
                .absolute()
                .left(px(c - s * 0.13))
                .top(px(c + s * 0.185))
                .w(px(s * 0.26))
                .flex()
                .justify_center()
                .gap(px(s * 0.02))
                .rounded(px(s * 0.012))
                .border_1()
                .border_color(ink.line)
                .bg(ink.face)
                .text_size(px(s * 0.045))
                .child(
                    div()
                        .text_color(ink.text)
                        .child(format!("{:03.0}°", self.heading)),
                )
                .child(
                    div()
                        .text_color(ink.primary)
                        .child(pm::cardinal(self.heading)),
                ),
        )
        .into_any_element()
    }

    /// A heading tape across the top, a turning ring of dashes with a north marker, and the
    /// bearing framed in the middle.
    fn hud(self, t: &dodai_core::Theme) -> AnyElement {
        let ink = Ink::of(t);
        let s = self.size;
        let c = s / 2.0;
        let heading = self.heading;
        let turn = heading / 360.0;
        let (tx0, tx1, ty) = (c - s * 0.26, c + s * 0.26, s * 0.25);
        let per = (tx1 - tx0) / 90.0;
        let r = s * 0.2;
        let (fw, fh) = (s * 0.26, s * 0.12);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            hud_ring(w, o, s, ink, ink.primary);
            // The tape: 45° either side, a mark every 5°.
            let first = ((heading - 45.0) / 5.0).ceil() as i32;
            let last = ((heading + 45.0) / 5.0).floor() as i32;
            for k in first..=last {
                let deg = k as f32 * 5.0;
                let x = c + (deg - heading) * per;
                let h = if k % 2 == 0 { s * 0.035 } else { s * 0.018 };
                stroke(w, o, &[(x, ty), (x, ty + h)], s * 0.004, ink.text, false);
            }
            stroke(w, o, &[(tx0, ty), (tx1, ty)], s * 0.003, ink.muted, false);
            fill(
                w,
                o,
                &[
                    (c, ty - s * 0.005),
                    (c - s * 0.018, ty - s * 0.035),
                    (c + s * 0.018, ty - s * 0.035),
                ],
                ink.primary,
            );
            // The turning ring.
            for i in 0..36 {
                let a = i as f32 / 36.0 - turn;
                stroke(
                    w,
                    o,
                    &turn_arc(c, c + s * 0.08, r, a - 0.008, a + 0.008),
                    s * 0.012,
                    scale_alpha(ink.text, 0.35),
                    false,
                );
            }
            let tip = polar(c, c + s * 0.08, r + s * 0.035, -turn);
            let (b1, b2) = (
                polar(c, c + s * 0.08, r - s * 0.005, -turn - 0.02),
                polar(c, c + s * 0.08, r - s * 0.005, -turn + 0.02),
            );
            fill(w, o, &[tip, b1, b2], ink.danger);
            let frame = chamfered(c - fw / 2.0, c + s * 0.08 - fh / 2.0, fw, fh, s * 0.025);
            fill(w, o, &frame, scale_alpha(ink.primary, 0.1));
            stroke(w, o, &frame, s * 0.005, ink.primary, true);
        };
        let mut el = case(s).child(layer(paint));
        let first = ((heading - 40.0) / 10.0).ceil() as i32;
        let last = ((heading + 40.0) / 10.0).floor() as i32;
        for k in first..=last {
            let deg = (k * 10).rem_euclid(360);
            let x = c + (k as f32 * 10.0 - heading) * per;
            let (txt, col): (SharedString, Hsla) = match deg {
                0 => ("N".into(), ink.danger),
                90 => ("E".into(), ink.text),
                180 => ("S".into(), ink.text),
                270 => ("W".into(), ink.text),
                d => (format!("{}", d / 10).into(), ink.muted),
            };
            el = el.child(label(x, ty + s * 0.04, s * 0.08, s * 0.035, col, txt));
        }
        el.child(
            div()
                .absolute()
                .left(px(c - fw / 2.0))
                .top(px(c + s * 0.08 - fh / 2.0))
                .w(px(fw))
                .h(px(fh))
                .flex()
                .items_center()
                .justify_center()
                .gap(px(s * 0.015))
                .child(
                    div()
                        .text_size(px(s * 0.06))
                        .text_color(ink.text)
                        .font_weight(FontWeight::MEDIUM)
                        .child(format!("{heading:03.0}")),
                )
                .child(
                    div()
                        .text_size(px(s * 0.035))
                        .text_color(ink.primary)
                        .child(pm::cardinal(heading)),
                ),
        )
        .child(label(
            c,
            c + s * 0.34,
            s * 0.4,
            s * 0.032,
            ink.muted,
            self.height.clone(),
        ))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_off_takes_the_accent() {
        let t = dodai_core::switcher().current().clone();
        let ink = Ink::of(&t);
        assert_eq!(tone_or(&t, Tone::Off, ink), ink.primary);
        assert_eq!(tone_or(&t, Tone::Danger, ink), Tone::Danger.color(&t));
        let (x, y) = crate::shapes::dir(0.25);
        assert!((x - 1.0).abs() < 1e-4 && y.abs() < 1e-4);
    }
}
