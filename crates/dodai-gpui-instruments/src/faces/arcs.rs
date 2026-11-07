//! The arc layout of the heads-up, segment LCD, race and line looks: a half ring over the top
//! with a large figure under it, each drawn in its own look (the halo's arc is in `layouts`).
//! Heads-up: a fan of hairlines lit to the value and a glowing needle between corner brackets.
//! Segment LCD: an arc of segments on an LCD module with a seven-segment figure. Race: an arc of
//! blocks lit in the zone colours on a carbon plate, the figure big and leaning. Line: one
//! hairline with a thin arc to the value and a dot riding on it. Every colour comes from the
//! theme.

use super::layouts::layer;
use super::line::hair;
use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::dash::text_box;
use crate::draw::{polar, turn_arc};
use crate::frames::{ghost, lcd, seg_arc, seg_text};
use crate::race::{figure, plate_in, tag, tag_text};
use crate::shapes::{band, circle, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::prelude::FluentBuilder;
use gpui::{div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, Pixels, Styled};

/// Height of an arc face, as a share of its width.
pub(crate) const ARC_H: f32 = 0.58;

/// Start and span of the arc in turns: from nine o'clock over the top to three o'clock.
const SWEEP: (f32, f32) = (0.75, 0.5);

/// The centre and radius of the arc on a face `s` wide: centred across, low enough that the
/// figure fits under the crown and the end figures under the ends. Pure.
fn arc_of(s: f32) -> (f32, f32, f32) {
    (s / 2.0, s * 0.47, s * 0.4)
}

/// The turn of the arc at `f` of the scale. Pure.
fn turn(f: f32) -> f32 {
    SWEEP.0 + SWEEP.1 * f.clamp(0.0, 1.0)
}

impl GaugeFace {
    /// The labels of the majors set inside the arc at radius `r`, `size` high, in `col`.
    fn arc_labels(&self, r: f32, size: f32, col: gpui::Hsla, italic: bool) -> Vec<AnyElement> {
        let (cx, cy, _) = arc_of(self.size);
        major_labels(self)
            .into_iter()
            .map(|(f, txt)| {
                let (x, y) = polar(cx, cy, r, turn(f));
                text_box(x, y - size * 0.6, size * 4.0)
                    .text_size(px(size))
                    .text_color(col)
                    .when(italic, |d| d.italic())
                    .child(txt)
                    .into_any_element()
            })
            .collect()
    }

    /// The heads-up arc: corner brackets, a fan of hairlines over the top lit up to the value
    /// (bright at their outer ends), the majors as ticks outside it with their figures inside,
    /// a glowing needle across the fan and the figure under the crown, its unit above and the
    /// caption below in the primary colour.
    pub(super) fn hud_arc(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * ARC_H;
        let (ax, ay, r) = arc_of(s);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            // Corner brackets frame the field of view.
            let (k, e) = (s * 0.06, s * 0.015);
            for (x, y, dx, dy) in [
                (e, e, 1.0, 1.0),
                (s - e, e, -1.0, 1.0),
                (e, h - e, 1.0, -1.0),
                (s - e, h - e, -1.0, -1.0),
            ] {
                stroke(
                    w,
                    o,
                    &[(x + k * dx, y), (x, y), (x, y + k * dy)],
                    hl * 1.5,
                    scale_alpha(ink.primary, 0.55),
                    false,
                );
            }
            let fine = 120usize;
            for i in 0..=fine {
                let f = i as f32 / fine as f32;
                let on = f <= frac;
                let c2 = if on { col } else { ink.text };
                let (a_in, a_out) = if on { (0.25, 0.9) } else { (0.06, 0.22) };
                let mid = r * 0.9;
                let a = turn(f);
                stroke(
                    w,
                    o,
                    &[polar(ax, ay, r * 0.8, a), polar(ax, ay, mid, a)],
                    hl,
                    scale_alpha(c2, a_in),
                    false,
                );
                stroke(
                    w,
                    o,
                    &[polar(ax, ay, mid, a), polar(ax, ay, r, a)],
                    hl,
                    scale_alpha(c2, a_out),
                    false,
                );
            }
            for f in &majors {
                stroke(
                    w,
                    o,
                    &[
                        polar(ax, ay, r * 1.03, turn(*f)),
                        polar(ax, ay, r * 1.09, turn(*f)),
                    ],
                    hl * 2.0,
                    ink.muted,
                    false,
                );
            }
            let needle = [
                polar(ax, ay, r * 0.72, turn(frac)),
                polar(ax, ay, r * 1.1, turn(frac)),
            ];
            crate::mfd::glow(w, o, &needle, (s * 0.006).max(1.5), col, false);
        };
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(self.arc_labels(r * 0.68, s * 0.03, ink.muted, false))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(ay - r * 0.48))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(s * 0.028))
                            .text_color(ink.muted)
                            .child(self.unit.to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.15))
                            .line_height(px(s * 0.16))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.028))
                            .text_color(ink.primary)
                            .child(self.caption().to_uppercase()),
                    ),
            )
            .into_any_element()
    }

    /// The segment arc: an LCD module with an arc of segments over the top lit to the value (a
    /// zone's colour past a limit, the unlit ones faint), the printed zones and numerals inside
    /// it, the seven-segment figure under the crown and the caption and unit under the figure.
    pub(super) fn segment_arc(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * ARC_H;
        let (ax, ay, r) = arc_of(s);
        let col = self.value_colour(&t, &ink);
        let lit = self.lit(&t, 28, ink.primary);
        let off = ghost(ink.primary, t.is_dark());
        let zones = zone_colours(&self, &t);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, h), s, ink);
            seg_arc(w, o, (ax, ay), (r * 0.8, r * 0.95), SWEEP, &lit, off);
            for (a, z, zc) in &zones {
                fill(
                    w,
                    o,
                    &band(ax, ay, r * 0.75, r * 0.77, turn(*a), turn(*z)),
                    *zc,
                );
            }
        };
        let fig = self.figure();
        let fh = s * 0.12;
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(self.arc_labels(r * 0.64, s * 0.028, ink.muted, false))
            .child(seg_text(&fig, ax, ay - r * 0.45, fh, col))
            .child(
                text_box(ax, ay - r * 0.45 + fh + s * 0.035, s * 0.5)
                    .gap(px(s * 0.012))
                    .text_size(px(s * 0.03))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(ink.primary)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(div().text_color(ink.muted).child(self.lcd_unit())),
            )
            .into_any_element()
    }

    /// The race arc: a carbon plate, an arc of blocks over the top lit to the value in the zone
    /// colours (the dark blocks faint), the scale's figures inside it, the figure big and
    /// leaning under the crown and the caption on a slanted tag under it.
    pub(super) fn race_arc(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let h = s * ARC_H;
        let (ax, ay, r) = arc_of(s);
        let frac = self.frac();
        let col = self.limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let n = 22usize;
        let lit = crate::race::lit_count(frac, n);
        let head = (ax - s * 0.14, ay - s * 0.035, s * 0.28, s * 0.05);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            plate_in(w, o, (s, h), s, ink);
            let colour = |at: f32| {
                zones
                    .iter()
                    .find(|(a, z, _)| at >= *a && at <= *z)
                    .map_or(ink.ok, |z| z.2)
            };
            for i in 0..n {
                let (f0, f1) = (i as f32 / n as f32, (i as f32 + 0.8) / n as f32);
                // The blocks thicken towards the top of the scale, like a rev bar.
                let r0 = r * (0.86 - 0.08 * f0);
                let p = band(ax, ay, r0, r * 0.97, turn(f0), turn(f1));
                if i < lit {
                    let c = colour((f0 + f1) / 2.0);
                    stroke(w, o, &p, s * 0.01, scale_alpha(c, 0.18), true);
                    fill(w, o, &p, c);
                } else {
                    fill(w, o, &p, scale_alpha(ink.text, 0.07));
                }
            }
            tag(w, o, head, ink.primary);
        };
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(self.arc_labels(r * 0.66, s * 0.03, ink.muted, true))
            .child(
                text_box(ax, ay - r * 0.62, s * 0.7)
                    .flex()
                    .justify_center()
                    .child(figure(self.figure(), s * 0.15, col)),
            )
            .child(tag_text(head, ink.on_primary, self.caption()))
            .child(
                text_box(ax, ay + s * 0.035, s * 0.4)
                    .text_size(px(s * 0.03))
                    .italic()
                    .text_color(ink.muted)
                    .child(self.unit.to_uppercase()),
            )
            .into_any_element()
    }

    /// The line arc: one hairline over the top with dots at the majors, the zones as a finer
    /// line inside it, a thin arc up to the value with a dot riding on its end, the scale's end
    /// figures under the ends and a large light figure under the crown.
    pub(super) fn line_arc(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * ARC_H;
        let (ax, ay, r) = arc_of(s);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hl = hair(s);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            stroke(
                w,
                o,
                &turn_arc(ax, ay, r, turn(0.0), turn(1.0)),
                hl,
                scale_alpha(ink.text, 0.28),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &turn_arc(ax, ay, r - s * 0.02, turn(*a), turn(*z)),
                    hl * 1.6,
                    scale_alpha(*zc, 0.9),
                    false,
                );
            }
            for f in &majors {
                let (x, y) = polar(ax, ay, r - s * 0.02, turn(*f));
                fill(w, o, &circle(x, y, hl * 1.3), ink.muted);
            }
            if frac > 0.0 {
                stroke(
                    w,
                    o,
                    &turn_arc(ax, ay, r, turn(0.0), turn(frac)),
                    (s * 0.007).max(1.5),
                    col,
                    false,
                );
            }
            let (x, y) = polar(ax, ay, r, turn(frac));
            fill(w, o, &circle(x, y, s * 0.02), ink.ground);
            fill(w, o, &circle(x, y, s * 0.012), col);
        };
        let ends = self.end_figures().map(move |(f, txt)| {
            let (x, y) = polar(ax, ay, r, turn(f));
            text_box(x, y + s * 0.04, s * 0.14)
                .text_size(px(s * 0.03))
                .text_color(ink.muted)
                .child(txt)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(ends)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(ay - r * 0.55))
                    .w(px(s))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(s * 0.17))
                            .line_height(px(s * 0.18))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.028))
                            .text_color(ink.muted)
                            .child(self.unit.to_uppercase()),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arc_fits_its_face() {
        let s = 300.0;
        let (cx, cy, r) = arc_of(s);
        assert!(cx - r > 0.0 && cx + r < s, "inside the width");
        assert!(cy - r > 0.0, "the crown below the top");
        assert!(cy < s * ARC_H, "the ends above the foot");
    }

    #[test]
    fn the_arc_runs_from_nine_over_the_top_to_three() {
        assert_eq!(turn(0.0), 0.75);
        assert_eq!(turn(0.5), 1.0);
        assert_eq!(turn(1.0), 1.25);
        assert_eq!(turn(2.0), 1.25, "clamped");
    }
}
