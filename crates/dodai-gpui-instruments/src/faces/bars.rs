//! The bar and column layouts of the race and line looks (the halo's and the segment LCD's are
//! in `layouts` and `lcd`, the glass cockpit's in `glass`). Race: a carbon plate with a rising
//! row, or a stack, of blocks lit in the zone colours and the figure big and leaning. Line: an
//! upright hairline with a thin line up to the value and a dot riding on it. Every colour comes
//! from the theme.

use super::layouts::layer;
use super::line::hair;
use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::dash::text_box;
use crate::race::{figure, lit_count, plate_in, seg_bar, tag, tag_text, LEAN};
use crate::shapes::{circle, Ink};
use dodai_gpui::paint::{fill, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// Height of a race bar, as a share of its width.
pub(crate) const RACE_BAR_H: f32 = 0.42;
/// Width of a column, as a share of its height.
pub(crate) const COLUMN_W: f32 = 0.42;

/// The blocks of a column over `(x, y, wd, h)`: `n` leaning blocks with gaps from the foot up,
/// each wider than the one under it, from under half the width at the foot to all of it at the
/// top, like a rev bar stood on end. Pure.
pub(crate) fn column_blocks((x, y, wd, h): (f32, f32, f32, f32), n: usize) -> Vec<Vec<(f32, f32)>> {
    let n = n.max(1);
    let step = h / n as f32;
    let bh = step * 0.72;
    (0..n)
        .map(|i| {
            let bw = wd * (0.45 + 0.55 * i as f32 / (n - 1).max(1) as f32);
            let y1 = y + h - step * i as f32;
            let k = bh * LEAN;
            vec![
                (x + k, y1 - bh),
                (x + bw + k, y1 - bh),
                (x + bw, y1),
                (x, y1),
            ]
        })
        .collect()
}

impl GaugeFace {
    /// The zone colour at `at` of the scale, else the success colour: how race blocks light.
    fn race_colour(zones: &[(f32, f32, Hsla)], ink: Ink, at: f32) -> Hsla {
        zones
            .iter()
            .find(|(a, z, _)| at >= *a && at <= *z)
            .map_or(ink.ok, |z| z.2)
    }

    /// The race bar: a wide carbon plate, the caption on a slanted tag at the top left, the
    /// figure big and leaning at the top right, and a rising row of blocks lit to the value in
    /// the zone colours over the scale's figures.
    pub(super) fn race_bar(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let h = s * RACE_BAR_H;
        let frac = self.frac();
        let col = self.limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let bar = (s * 0.06, s * 0.2, s * 0.88, s * 0.12);
        let head = (s * 0.06, s * 0.08, s * 0.3, s * 0.06);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            plate_in(w, o, (s, h), s, ink);
            tag(w, o, head, ink.primary);
            seg_bar(
                w,
                o,
                bar,
                frac,
                24,
                true,
                |at| Self::race_colour(&zones, ink, at),
                ink,
            );
        };
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            text_box(bar.0 + bar.2 * f, bar.1 + bar.3 + s * 0.015, s * 0.12)
                .text_size(px(s * 0.03))
                .italic()
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .child(tag_text(head, ink.on_primary, self.caption()))
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.06))
                    .top(px(head.1 + head.3 + s * 0.005))
                    .text_size(px(s * 0.028))
                    .italic()
                    .text_color(ink.muted)
                    .child(self.unit.to_uppercase()),
            )
            .child(
                div()
                    .absolute()
                    .left(px(s * 0.45))
                    .top(px(s * 0.035))
                    .w(px(s * 0.49))
                    .flex()
                    .justify_end()
                    .child(figure(self.figure(), s * 0.13, col)),
            )
            .children(numerals)
            .into_any_element()
    }

    /// The race column: a narrow carbon plate, the caption on a slanted tag at the top, the
    /// figure big and leaning under it, and a stack of blocks widening upwards, lit to the value
    /// in the zone colours, with the scale's figures beside it.
    pub(super) fn race_column(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = Ink::of(&t);
        let s = self.size;
        let wd = s * COLUMN_W;
        let frac = self.frac();
        let col = self.limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let n = 16;
        let stack = (wd * 0.14, s * 0.4, wd * 0.5, s * 0.54);
        let head = (wd * 0.1, s * 0.08, wd * 0.8, s * 0.055);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            plate_in(w, o, (wd, s), s, ink);
            tag(w, o, head, ink.primary);
            let blocks = column_blocks(stack, n);
            let lit = lit_count(frac, n);
            for (i, b) in blocks.iter().enumerate() {
                if i < lit {
                    let c = Self::race_colour(&zones, ink, (i as f32 + 0.5) / n as f32);
                    stroke(w, o, b, s * 0.008, scale_alpha(c, 0.18), true);
                    fill(w, o, b, c);
                } else {
                    fill(w, o, b, scale_alpha(ink.text, 0.07));
                }
            }
        };
        let (_, sy, sw, sh) = stack;
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            div()
                .absolute()
                .left(px(stack.0 + sw + wd * 0.08))
                .top(px(sy + sh - sh * f - s * 0.02))
                .text_size(px(s * 0.03))
                .italic()
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(wd))
            .h(px(s))
            .child(layer(paint))
            .child(tag_text(head, ink.on_primary, self.caption()))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(s * 0.15))
                    .w(px(wd))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(figure(self.figure(), s * 0.12, col))
                    .child(
                        div()
                            .text_size(px(s * 0.028))
                            .italic()
                            .text_color(ink.muted)
                            .child(self.unit.to_uppercase()),
                    ),
            )
            .children(numerals)
            .into_any_element()
    }

    /// The line column: the figure light and large at the top with its unit, and under it one
    /// upright hairline with dots at the majors, the zones as a finer line beside it, a thin
    /// line up to the value with a dot riding on its end and the scale's end figures beside
    /// the ends.
    pub(super) fn line_column(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let wd = s * COLUMN_W;
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let zones = zone_colours(&self, &t);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hl = hair(s);
        let x = wd * 0.38;
        let (top, foot) = (s * 0.3, s * 0.94);
        let y_of = move |f: f32| foot - (foot - top) * f.clamp(0.0, 1.0);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            stroke(
                w,
                o,
                &[(x, top), (x, foot)],
                hl,
                scale_alpha(ink.text, 0.28),
                false,
            );
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &[(x + s * 0.02, y_of(*a)), (x + s * 0.02, y_of(*z))],
                    hl * 1.6,
                    scale_alpha(*zc, 0.9),
                    false,
                );
            }
            for f in &majors {
                fill(w, o, &circle(x + s * 0.02, y_of(*f), hl * 1.3), ink.muted);
            }
            let y = y_of(frac);
            if frac > 0.0 {
                stroke(w, o, &[(x, foot), (x, y)], (s * 0.006).max(1.5), col, false);
            }
            fill(w, o, &circle(x, y, s * 0.016), ink.ground);
            fill(w, o, &circle(x, y, s * 0.01), col);
        };
        let ends = self.end_figures().map(move |(f, txt)| {
            div()
                .absolute()
                .left(px(x + s * 0.045))
                .top(px(y_of(f) - s * 0.018))
                .text_size(px(s * 0.03))
                .text_color(ink.muted)
                .child(txt)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(wd))
            .h(px(s))
            .child(layer(paint))
            .children(ends)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(s * 0.04))
                    .w(px(wd))
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(s * 0.12))
                            .line_height(px(s * 0.13))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.026))
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
    fn column_blocks_widen_upwards_inside_their_box() {
        let b = column_blocks((10.0, 20.0, 40.0, 100.0), 8);
        assert_eq!(b.len(), 8);
        let width = |p: &Vec<(f32, f32)>| p[1].0 - p[0].0;
        assert!(b.windows(2).all(|p| width(&p[1]) > width(&p[0])));
        assert!((width(&b[7]) - 40.0).abs() < 1e-3);
        assert!(b
            .iter()
            .flatten()
            .all(|&(_, y)| (20.0..=120.0).contains(&y)));
        assert!(b[0][3].1 > b[7][3].1, "the first block sits at the foot");
    }
}
