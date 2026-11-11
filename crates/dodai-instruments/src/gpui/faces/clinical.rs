//! The clinical looks: [`Look::Apex`](crate::gpui::look::Look::Apex), a dark dial with big white
//! numerals, fine track rings, a red blade and a pill naming the mode (its band layout floods the
//! scale with red), and [`Look::Daylight`](crate::gpui::look::Look::Daylight), a white-faced dial
//! with black numerals beside dark ring dials with a hairline track and a red marker. Both print
//! their scale in heavy type and keep their red for the blade, the marker and the redline; the
//! dark and the white follow the theme's text and background, inverted in a light theme so the
//! dark dials stay dark. Every colour comes from the theme.

use super::layouts::layer;
use super::rich::{major_labels, zone_colours, FINE};
use super::round::ROUND;
use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::draw::{polar, turn_arc};
use crate::gpui::shapes::{band, Hand, Ink};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke, stroke_circle};
use dodai_gpui::theme_of;
use gpui::prelude::FluentBuilder;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// The two colours a clinical dial is printed with: the white of its paper and the dark of its
/// print, taken from the theme and swapped in a light one. Pure.
pub(super) fn paper_and_print(dark: bool, ink: &Ink) -> (Hsla, Hsla) {
    if dark {
        (ink.text, ink.ground)
    } else {
        (ink.ground, ink.text)
    }
}

/// A numeral or figure centred on `(x, y)`.
pub(super) fn figure_at(
    (x, y): (f32, f32),
    size: f32,
    col: Hsla,
    weight: FontWeight,
    text: impl Into<gpui::SharedString>,
) -> AnyElement {
    text_box(x, y - size * 0.65, size * 5.0)
        .text_size(px(size))
        .line_height(px(size * 1.3))
        .font_weight(weight)
        .text_color(col)
        .child(text.into())
        .into_any_element()
}

/// A pill of `width` by `height` centred on `(x, y)` with a short caption in it.
#[allow(clippy::too_many_arguments)]
pub(super) fn pill(
    (x, y): (f32, f32),
    (width, height): (f32, f32),
    text: impl Into<gpui::SharedString>,
    fg: Hsla,
    bg: Hsla,
    edge: Hsla,
) -> AnyElement {
    div()
        .absolute()
        .left(px(x - width / 2.0))
        .top(px(y - height / 2.0))
        .w(px(width))
        .h(px(height))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(height))
        .border_1()
        .border_color(edge)
        .bg(bg)
        .text_size(px(height * 0.5))
        .font_weight(FontWeight::BOLD)
        .text_color(fg)
        .child(text.into())
        .into_any_element()
}

/// The pivot of a blade: a turned metal cap with a bright rim and a dark dot.
pub(super) fn cap(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    c: (f32, f32),
    r: f32,
    paper: Hsla,
    print: Hsla,
) {
    fill_circle(w, o, c, r, mix(paper, print, 0.55));
    fill_circle(
        w,
        o,
        (c.0 - r * 0.12, c.1 - r * 0.15),
        r * 0.78,
        mix(paper, print, 0.3),
    );
    stroke_circle(w, o, c, r, (r * 0.1).max(0.6), scale_alpha(paper, 0.7));
    fill_circle(w, o, c, r * 0.28, print);
}

/// The light on a glass case: a faint arc along the top left of the rim and a fainter one
/// opposite.
pub(super) fn sheen(w: &mut gpui::Window, o: gpui::Point<Pixels>, c: f32, r: f32, paper: Hsla) {
    let wd = (r * 0.012).max(0.8);
    stroke(
        w,
        o,
        &turn_arc(c, c, r, 0.80, 0.97),
        wd,
        scale_alpha(paper, 0.2),
        false,
    );
    stroke(
        w,
        o,
        &turn_arc(c, c, r, 0.30, 0.42),
        wd,
        scale_alpha(paper, 0.07),
        false,
    );
}

/// A lit arc of `r` between `a` and `b` turns, `wd` thick, with a bead on its end: the way a
/// dial without a hand shows its value.
#[allow(clippy::too_many_arguments)]
pub(super) fn lit_arc(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    c: f32,
    r: f32,
    (a, b): (f32, f32),
    wd: f32,
    (col, bead): (Hsla, Hsla),
) {
    if b > a {
        fill(w, o, &band(c, c, r - wd / 2.0, r + wd / 2.0, a, b), col);
    }
    let (x, y) = polar(c, c, r, b);
    fill_circle(w, o, (x, y), wd * 0.95, col);
    fill_circle(w, o, (x, y), wd * 0.55, bead);
}

/// The fine marks of a scale between two radii, `FINE` to every major.
pub(super) fn minute_ticks(
    w: &mut gpui::Window,
    o: gpui::Point<Pixels>,
    c: f32,
    (r0, r1): (f32, f32),
    n: usize,
    width: f32,
    col: Hsla,
) {
    let (start, span) = ROUND;
    for i in 0..=n {
        let a = start + span * i as f32 / n.max(1) as f32;
        stroke(
            w,
            o,
            &[polar(c, c, r0, a), polar(c, c, r1, a)],
            width,
            col,
            false,
        );
    }
}

impl GaugeFace {
    /// The scale's majors as `(fraction, label)`.
    pub(super) fn clinical_majors(&self) -> Vec<(f32, String)> {
        major_labels(self)
    }

    /// The caption of a mode pill: the label, else the unit, in capitals.
    pub(super) fn pill_text(&self) -> String {
        self.caption().to_uppercase()
    }

    /// The width of a mode pill for its caption, in px. Pure.
    pub(super) fn pill_width(&self) -> f32 {
        let n = self.pill_text().chars().count() as f32;
        self.size * (0.08 + 0.022 * n).clamp(0.22, 0.56)
    }

    /// The dark clinical dial: a near-black face, fine track rings inside big white numerals, a
    /// minute track outside them, the redline as a red block, the figure under the cap and a
    /// pill naming the mode. With a `needle` a red blade on a metal cap shows the value; without
    /// one a lit red arc fills the innermost track ring up to it, a bead on its end, and the
    /// numerals past it dim.
    pub(super) fn apex_dial(self, cx: &App, needle: bool) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let (paper, print) = paper_and_print(t.is_dark(), &ink);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let frac = self.frac();
        let red = ink.danger;
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let hl = (s * 0.003).max(0.6);
        let base = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            fill_circle(w, o, (c, c), c - s * 0.01, print);
            fill_circle(w, o, (c, c), c - s * 0.01, scale_alpha(paper, 0.035));
            stroke_circle(
                w,
                o,
                (c, c),
                c - s * 0.01,
                s * 0.012,
                scale_alpha(paper, 0.22),
            );
            sheen(w, o, c, c - s * 0.01, paper);
            // The railway-track rings inside the numerals.
            for k in 0..4 {
                let r = s * (0.255 + 0.016 * k as f32);
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    hl,
                    scale_alpha(paper, 0.42 - 0.07 * k as f32),
                    false,
                );
            }
            minute_ticks(
                w,
                o,
                c,
                (s * 0.452, s * 0.466),
                marks,
                hl,
                scale_alpha(paper, 0.5),
            );
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(
                        c,
                        c,
                        s * 0.43,
                        s * 0.475,
                        start + span * a,
                        start + span * z,
                    ),
                    *zc,
                );
            }
            if !needle {
                lit_arc(
                    w,
                    o,
                    c,
                    s * 0.285,
                    (start, start + span * frac),
                    s * 0.026,
                    (red, paper),
                );
            }
        };
        let blade = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            Hand {
                centre: (c, c),
                turn: start + span * frac,
                reach: (s * 0.09, s * 0.36),
                half: s * 0.012,
                colours: (red, paper),
                shadow: Some(scale_alpha(ink.ground, 0.5)),
            }
            .paint(w, o);
            cap(w, o, (c, c), s * 0.05, paper, print);
        };
        let numerals = self.clinical_majors().into_iter().map(|(f, label)| {
            let on = if needle || f <= frac + 1e-4 {
                paper
            } else {
                scale_alpha(paper, 0.4)
            };
            figure_at(
                polar(c, c, s * 0.375, start + span * f),
                s * 0.085,
                on,
                FontWeight::BOLD,
                label,
            )
        });
        let col = self.limit(&t).unwrap_or(paper);
        let (fig, fs, unit_at) = if needle {
            ((c, c + s * 0.17), s * 0.085, c + s * 0.225)
        } else {
            ((c, c - s * 0.02), s * 0.15, c + s * 0.075)
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(base))
            .children(numerals)
            .when(needle, |d| d.child(layer(blade)))
            .child(figure_at(fig, fs, col, FontWeight::BOLD, self.figure()))
            .child(figure_at(
                (c, unit_at),
                s * 0.032,
                scale_alpha(paper, 0.6),
                FontWeight::NORMAL,
                self.unit.to_string(),
            ))
            .child(pill(
                (c, c + s * 0.4),
                (self.pill_width(), s * 0.062),
                self.pill_text(),
                paper,
                print,
                scale_alpha(paper, 0.85),
            ))
            .into_any_element()
    }

    /// The dark clinical dial in its track mode: the scale flooded with a thick red band (lit up
    /// to the value, dim after it) with the numerals printed on it and a pill on the red. With a
    /// `needle` a bright blade on a cap shows the value and the figure sits beside the cap;
    /// without one the figure stands big in the middle.
    pub(super) fn apex_band(self, cx: &App, needle: bool) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let (paper, print) = paper_and_print(t.is_dark(), &ink);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let frac = self.frac();
        let red = ink.danger;
        let (r0, r1) = (s * 0.3, s * 0.475);
        let base = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            fill_circle(w, o, (c, c), c - s * 0.01, print);
            fill_circle(w, o, (c, c), c - s * 0.01, scale_alpha(paper, 0.035));
            stroke_circle(
                w,
                o,
                (c, c),
                c - s * 0.01,
                s * 0.012,
                scale_alpha(paper, 0.22),
            );
            sheen(w, o, c, c - s * 0.01, paper);
            fill(
                w,
                o,
                &band(c, c, r0, r1, start, start + span),
                scale_alpha(red, 0.3),
            );
            if frac > 0.0 {
                fill(w, o, &band(c, c, r0, r1, start, start + span * frac), red);
                // A bright edge where the lit part ends.
                let a = start + span * frac;
                stroke(
                    w,
                    o,
                    &[polar(c, c, r0, a), polar(c, c, r1, a)],
                    (s * 0.006).max(1.0),
                    scale_alpha(paper, 0.85),
                    false,
                );
            }
            let hl = (s * 0.003).max(0.6);
            for r in [r0 - s * 0.012, r0 - s * 0.024] {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    hl,
                    scale_alpha(paper, 0.25),
                    false,
                );
            }
        };
        let blade = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            Hand {
                centre: (c, c),
                turn: start + span * frac,
                reach: (s * 0.09, r1 - s * 0.01),
                half: s * 0.011,
                colours: (mix(red, paper, 0.55), paper),
                shadow: Some(scale_alpha(ink.ground, 0.6)),
            }
            .paint(w, o);
            cap(w, o, (c, c), s * 0.05, paper, print);
        };
        let numerals = self.clinical_majors().into_iter().map(|(f, label)| {
            let on = if f <= frac + 1e-4 { print } else { paper };
            figure_at(
                polar(c, c, (r0 + r1) / 2.0, start + span * f),
                s * 0.075,
                on,
                FontWeight::BOLD,
                label,
            )
        });
        let col = self.limit(&t).unwrap_or(paper);
        let (fig, fs) = if needle {
            ((c + s * 0.175, c - s * 0.005), s * 0.12)
        } else {
            ((c, c - s * 0.045), s * 0.17)
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(base))
            .children(numerals)
            .when(needle, |d| d.child(layer(blade)))
            .child(figure_at(fig, fs, col, FontWeight::BOLD, self.figure()))
            .child(pill(
                if needle {
                    (c - s * 0.02, c + s * 0.15)
                } else {
                    (c, c + s * 0.125)
                },
                (self.pill_width(), s * 0.055),
                self.pill_text(),
                paper,
                red,
                red,
            ))
            .into_any_element()
    }

    /// The white-faced dial: a white face under black numerals in a dark case, a minute track,
    /// the redline as a red block, a dark centre disc and a dark pill naming the reading. With a
    /// `needle` a red blade on a cap shows the value and the figure sits under the cap; without
    /// one a lit red arc fills the inner track up to it and the figure stands in the middle.
    pub(super) fn daylight_dial(self, cx: &App, needle: bool) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let (paper, print) = paper_and_print(t.is_dark(), &ink);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let frac = self.frac();
        let red = ink.danger;
        let zones = zone_colours(&self, &t);
        let marks = self.scale.majors.max(1) * FINE;
        let hl = (s * 0.003).max(0.6);
        let base = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            fill_circle(w, o, (c, c), c - s * 0.01, print);
            stroke_circle(
                w,
                o,
                (c, c),
                c - s * 0.01,
                s * 0.012,
                scale_alpha(paper, 0.3),
            );
            sheen(w, o, c, c - s * 0.01, paper);
            let face = c - s * 0.045;
            fill_circle(w, o, (c, c), face, paper);
            stroke_circle(w, o, (c, c), face, s * 0.006, scale_alpha(print, 0.5));
            for k in 0..3 {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, s * (0.285 + 0.014 * k as f32), start, start + span),
                    hl,
                    scale_alpha(print, 0.6 - 0.12 * k as f32),
                    false,
                );
            }
            minute_ticks(
                w,
                o,
                c,
                (s * 0.44, s * 0.458),
                marks,
                hl,
                scale_alpha(print, 0.7),
            );
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(
                        c,
                        c,
                        s * 0.425,
                        s * 0.47,
                        start + span * a,
                        start + span * z,
                    ),
                    *zc,
                );
            }
            if !needle {
                lit_arc(
                    w,
                    o,
                    c,
                    s * 0.3,
                    (start, start + span * frac),
                    s * 0.026,
                    (red, print),
                );
            }
            // The dark centre disc the readout sits in.
            fill_circle(w, o, (c, c), s * 0.235, print);
            stroke_circle(w, o, (c, c), s * 0.235, hl * 1.5, scale_alpha(paper, 0.35));
        };
        let blade = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            Hand {
                centre: (c, c),
                turn: start + span * frac,
                reach: (s * 0.07, s * 0.44),
                half: s * 0.011,
                colours: (red, paper),
                shadow: Some(scale_alpha(print, 0.25)),
            }
            .paint(w, o);
            cap(w, o, (c, c), s * 0.045, paper, print);
        };
        let numerals = self.clinical_majors().into_iter().map(|(f, label)| {
            let on = if needle || f <= frac + 1e-4 {
                print
            } else {
                scale_alpha(print, 0.4)
            };
            figure_at(
                polar(c, c, s * 0.365, start + span * f),
                s * 0.085,
                on,
                FontWeight::BOLD,
                label,
            )
        });
        let col = self.limit(&t).unwrap_or(paper);
        let (fig, fs) = if needle {
            ((c, c + s * 0.13), s * 0.075)
        } else {
            ((c, c - s * 0.03), s * 0.14)
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(base))
            .children(numerals)
            .when(needle, |d| d.child(layer(blade)))
            .child(figure_at(fig, fs, col, FontWeight::BOLD, self.figure()))
            .when(!needle, |d| {
                d.child(figure_at(
                    (c, c + s * 0.085),
                    s * 0.034,
                    scale_alpha(paper, 0.6),
                    FontWeight::NORMAL,
                    self.unit.to_string(),
                ))
            })
            .child(pill(
                (c, c + s * 0.385),
                (self.pill_width(), s * 0.062),
                self.pill_text(),
                paper,
                print,
                print,
            ))
            .into_any_element()
    }

    /// The dark ring dial: a hairline double ring over three quarters of a turn, the zones as
    /// red runs on it, a red marker riding on it at the value, small numerals inside and the
    /// figure bold in the middle over its unit. With a `needle` a short red hand from a small
    /// cap points at the value too and the figure moves under it.
    pub(super) fn daylight_ring(self, cx: &App, needle: bool) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let (paper, print) = paper_and_print(t.is_dark(), &ink);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let frac = self.frac();
        let red = ink.danger;
        let zones = zone_colours(&self, &t);
        let r = s * 0.4;
        let base = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            fill_circle(w, o, (c, c), c - s * 0.01, print);
            fill_circle(w, o, (c, c), c - s * 0.01, scale_alpha(paper, 0.03));
            stroke_circle(
                w,
                o,
                (c, c),
                c - s * 0.01,
                s * 0.012,
                scale_alpha(paper, 0.25),
            );
            sheen(w, o, c, c - s * 0.01, paper);
            let arc = |w: &mut gpui::Window, r: f32, wd: f32, col: Hsla| {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start, start + span),
                    wd,
                    col,
                    false,
                );
            };
            arc(w, r, s * 0.006, scale_alpha(paper, 0.85));
            arc(
                w,
                r - s * 0.03,
                (s * 0.003).max(0.6),
                scale_alpha(paper, 0.35),
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(c, c, r, start + span * a, start + span * z),
                    s * 0.014,
                    *zc,
                    false,
                );
            }
            let a = start + span * frac;
            stroke(
                w,
                o,
                &[polar(c, c, r - s * 0.045, a), polar(c, c, r + s * 0.03, a)],
                s * 0.018,
                red,
                false,
            );
        };
        let blade = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            Hand {
                centre: (c, c),
                turn: start + span * frac,
                reach: (s * 0.06, r - s * 0.125),
                half: s * 0.009,
                colours: (red, paper),
                shadow: Some(scale_alpha(ink.ground, 0.5)),
            }
            .paint(w, o);
            cap(w, o, (c, c), s * 0.038, paper, print);
        };
        let numerals = self.clinical_majors().into_iter().map(|(f, label)| {
            figure_at(
                polar(c, c, r - s * 0.085, start + span * f),
                s * 0.052,
                scale_alpha(paper, 0.8),
                FontWeight::NORMAL,
                label,
            )
        });
        let col = self.limit(&t).unwrap_or(paper);
        let (fig, fs, cap_at) = if needle {
            ((c, c + s * 0.15), s * 0.13, c + s * 0.255)
        } else {
            ((c, c - s * 0.02), s * 0.17, c + s * 0.12)
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(base))
            .children(numerals)
            .when(needle, |d| d.child(layer(blade)))
            .child(figure_at(fig, fs, col, FontWeight::BOLD, self.figure()))
            .child(figure_at(
                (c, cap_at),
                s * 0.04,
                scale_alpha(paper, 0.6),
                FontWeight::NORMAL,
                self.caption().to_uppercase(),
            ))
            .into_any_element()
    }
}
