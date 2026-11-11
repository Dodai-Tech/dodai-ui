//! [`Look::Glow`](crate::gpui::look::Look::Glow): the instrument as a car draws it at night. A
//! black dial, a pale mark every step on the outer ring, the numerals standing in a bundle of
//! fine concentric lines that is cut out around them (its start slanted, the outer lines
//! first), the bundle continued in the redline's colour over the zones, a thin inner circle and
//! a long thin red needle on a small hub. Without a needle the bundle itself is the reading: it
//! lights up to the value and the rest stays dim. Every colour comes from the theme.

use super::clinical::{cap, figure_at, minute_ticks, paper_and_print, pill, sheen};
use super::layouts::layer;
use super::rich::{zone_colours, FINE};
use super::round::ROUND;
use super::GaugeFace;
use crate::gpui::draw::{polar, turn_arc};
use crate::gpui::shapes::{Hand, Ink};
use dodai_gpui::paint::{fill_circle, scale_alpha, stroke, stroke_circle};
use dodai_gpui::theme_of;
use gpui::prelude::FluentBuilder;
use gpui::{div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, Pixels, Styled};

/// The lines in the bundle.
const LINES: usize = 6;
/// The bundle's slant: each line further in starts this many turns later.
const SLANT: f32 = 2.5 / 360.0;

impl GaugeFace {
    /// The night instrument: see the module. With a `needle` the bundle is lit all along and
    /// the red needle points at the value; without one the bundle lights up to the value.
    pub(super) fn glow_dial(self, cx: &App, needle: bool) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let (paper, print) = paper_and_print(t.is_dark(), &ink);
        let s = self.size;
        let c = s / 2.0;
        let (start, span) = ROUND;
        let frac = self.frac();
        let red = ink.danger;
        let tone = ink.ok;
        let zones = zone_colours(&self, &t);
        // The bundle ends where the first zone starts.
        let lit_to = zones.first().map_or(1.0, |z| z.0);
        let marks = self.scale.majors.max(1) * FINE;
        let hl = (s * 0.0035).max(0.6);
        let labels = self.clinical_majors();
        let base = {
            let labels = labels.clone();
            move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
                fill_circle(w, o, (c, c), c - s * 0.01, print);
                stroke_circle(
                    w,
                    o,
                    (c, c),
                    c - s * 0.01,
                    s * 0.008,
                    scale_alpha(paper, 0.18),
                );
                sheen(w, o, c, c - s * 0.01, paper);
                minute_ticks(
                    w,
                    o,
                    c,
                    (s * 0.442, s * 0.4725),
                    marks,
                    hl,
                    scale_alpha(paper, 0.7),
                );
                for k in 0..LINES {
                    let r = s * (0.4 - 0.085 * k as f32 / (LINES - 1) as f32);
                    let a = start + SLANT * k as f32 + span * 0.02;
                    let b = start + span * lit_to - span * 0.01;
                    let lit_b = if needle {
                        b
                    } else {
                        (start + span * frac).min(b)
                    };
                    if b > a {
                        stroke(
                            w,
                            o,
                            &turn_arc(c, c, r, a, b),
                            hl * 1.6,
                            scale_alpha(tone, 0.2),
                            false,
                        );
                    }
                    if lit_b > a {
                        stroke(w, o, &turn_arc(c, c, r, a, lit_b), hl * 1.6, tone, false);
                    }
                    for (z0, z1, zc) in &zones {
                        stroke(
                            w,
                            o,
                            &turn_arc(c, c, r, start + span * z0, start + span * z1),
                            hl * 1.6,
                            *zc,
                            false,
                        );
                    }
                }
                // The numerals cut out of the bundle.
                for (f, _) in &labels {
                    fill_circle(
                        w,
                        o,
                        polar(c, c, s * 0.357, start + span * f),
                        s * 0.05,
                        print,
                    );
                }
                stroke_circle(w, o, (c, c), s * 0.21, hl, scale_alpha(paper, 0.22));
            }
        };
        let numerals = labels.into_iter().map(|(f, label)| {
            figure_at(
                polar(c, c, s * 0.357, start + span * f),
                s * 0.08,
                if needle || f <= frac + 1e-4 {
                    paper
                } else {
                    scale_alpha(paper, 0.55)
                },
                FontWeight::MEDIUM,
                label,
            )
        });
        let blade = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            Hand {
                centre: (c, c),
                turn: start + span * frac,
                reach: (s * 0.0, s * 0.45),
                half: s * 0.009,
                colours: (red, red),
                shadow: Some(scale_alpha(print, 0.6)),
            }
            .paint(w, o);
            cap(w, o, (c, c), s * 0.03, paper, print);
        };
        let bead = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let p = polar(c, c, s * 0.42, start + span * frac);
            fill_circle(w, o, p, s * 0.014, red);
        };
        let col = self.limit(&t).unwrap_or(paper);
        let fig = if needle {
            (c, c + s * 0.1)
        } else {
            (c, c - s * 0.01)
        };
        div()
            .relative()
            .w(px(s))
            .h(px(s))
            .child(layer(base))
            .children(numerals)
            .when(!needle, |d| d.child(layer(bead)))
            .when(needle, |d| d.child(layer(blade)))
            .child(figure_at(
                (c, c - s * 0.115),
                s * 0.03,
                scale_alpha(tone, 0.9),
                FontWeight::NORMAL,
                self.unit.to_string(),
            ))
            .child(figure_at(
                fig,
                s * 0.11,
                col,
                FontWeight::MEDIUM,
                self.figure(),
            ))
            .child(pill(
                (c, c + s * 0.31),
                (self.pill_width(), s * 0.055),
                self.pill_text(),
                paper,
                print,
                scale_alpha(paper, 0.7),
            ))
            .into_any_element()
    }
}
