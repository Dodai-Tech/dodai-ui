//! The segment LCD's extra layouts: the bar, a wide module with the figure over a row of
//! segments, and the column, a narrow module with the segments stacked upright. Both light
//! their segments in the zone colours past a limit and leave the unlit ones faintly visible.
//! Every colour comes from the theme.

use super::layouts::{along, layer};
use super::rich::{major_labels, zone_colours};
use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::dial::severity_color;
use crate::gpui::frames::{ghost, lcd, lit_segments, seg_text, seg_width};
use crate::gpui::pod_math as pm;
use crate::instrument::Severity;
use dodai_gpui::paint::{fill, stroke};
use dodai_gpui::theme_of;
use gpui::{
    div, px, AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, Pixels, Styled,
};

/// Height of the segment bar, as a share of its width.
pub(crate) const SEG_BAR_H: f32 = 0.42;
/// Width of the segment column, as a share of its height.
pub(crate) const SEG_COLUMN_W: f32 = 0.42;

/// A segment `(x, y, wd, h)` leaning forward by `lean`, as LCD bar graphs draw them. Pure.
pub(crate) fn slanted((x, y, wd, h): (f32, f32, f32, f32), lean: f32) -> Vec<(f32, f32)> {
    vec![
        (x + lean, y),
        (x + wd + lean, y),
        (x + wd, y + h),
        (x, y + h),
    ]
}

impl GaugeFace {
    /// The colours of `n` segments lit to the value: the accent, a zone's colour past a limit.
    pub(super) fn lit(&self, t: &dodai_core::Theme, n: usize, on: Hsla) -> Vec<Option<Hsla>> {
        let spans = self.spans();
        let tt = t.clone();
        lit_segments(n, self.frac(), move |f| match pm::severity_at(&spans, f) {
            Severity::Normal => on,
            sev => severity_color(&tt, sev),
        })
    }

    /// The unit in capitals, else the label.
    pub(super) fn lcd_unit(&self) -> String {
        if self.unit.is_empty() {
            self.label.to_uppercase()
        } else {
            self.unit.to_uppercase()
        }
    }

    /// A wide LCD module: the caption and unit at the top left, the seven-segment figure at the
    /// top right, and a row of leaning segments over the printed zones and numerals.
    pub(super) fn segment_bar(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * SEG_BAR_H;
        let col = self.value_colour(&t, &ink);
        let n = 32;
        let lit = self.lit(&t, n, ink.primary);
        let off = ghost(ink.primary, t.is_dark());
        let zones = zone_colours(&self, &t);
        let k = s * 0.035;
        let (x0, x1) = (k + s * 0.035, s - k - s * 0.045);
        let (by, bh) = (k + s * 0.17, s * 0.065);
        let majors = self.scale.majors.max(1);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, s, h), s, ink);
            let step = (x1 - x0) / n as f32;
            for (i, c) in lit.iter().enumerate() {
                let x = x0 + step * i as f32;
                fill(
                    w,
                    o,
                    &slanted((x, by, step - s * 0.006, bh), s * 0.01),
                    c.unwrap_or(off),
                );
            }
            for (a, z, zc) in &zones {
                let y = by + bh + s * 0.012;
                stroke(
                    w,
                    o,
                    &[(along(x0, x1, *a), y), (along(x0, x1, *z), y)],
                    s * 0.006,
                    *zc,
                    false,
                );
            }
            for i in 0..=majors {
                let x = along(x0, x1, i as f32 / majors as f32);
                let y = by + bh + s * 0.02;
                stroke(
                    w,
                    o,
                    &[(x, y), (x, y + s * 0.014)],
                    s * 0.005,
                    ink.muted,
                    false,
                );
            }
        };
        let fig = self.figure();
        let fh = s * 0.11;
        let right = s - k - s * 0.035;
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            text_box(along(x0, x1, f), by + bh + s * 0.036, s * 0.12)
                .text_size(px(s * 0.03))
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .children(numerals)
            .child(
                div()
                    .absolute()
                    .left(px(x0))
                    .top(px(k + s * 0.03))
                    .flex()
                    .flex_col()
                    .gap(px(s * 0.008))
                    .child(
                        div()
                            .text_size(px(s * 0.036))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(ink.primary)
                            .child(self.caption().to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.03))
                            .text_color(ink.muted)
                            .child(self.lcd_unit()),
                    ),
            )
            .child(seg_text(
                &fig,
                right - seg_width(&fig, fh) / 2.0,
                k + s * 0.025,
                fh,
                col,
            ))
            .into_any_element()
    }

    /// A narrow upright LCD module: the caption, the seven-segment figure and the unit at the
    /// top, and a stack of segments rising to the value beside the printed zones and numerals.
    pub(super) fn segment_column(self, cx: &mut App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let wd = s * SEG_COLUMN_W;
        let col = self.value_colour(&t, &ink);
        let n = 24;
        let lit = self.lit(&t, n, ink.primary);
        let off = ghost(ink.primary, t.is_dark());
        let zones = zone_colours(&self, &t);
        let (yb, yt) = (s * 0.9, s * 0.3);
        let (bx, bw) = (s * 0.085, s * 0.15);
        let zx = bx + bw + s * 0.022;
        let y_of = move |f: f32| yb - (yb - yt) * f.clamp(0.0, 1.0);
        let majors = self.scale.majors.max(1);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            lcd(w, o, (0.0, 0.0, wd, s), s, ink);
            let step = (yb - yt) / n as f32;
            for (i, c) in lit.iter().enumerate() {
                let y = yb - step * (i as f32 + 1.0);
                fill(
                    w,
                    o,
                    &slanted((bx, y + s * 0.003, bw, step - s * 0.006), s * 0.006),
                    c.unwrap_or(off),
                );
            }
            for (a, z, zc) in &zones {
                stroke(
                    w,
                    o,
                    &[(zx, y_of(*a)), (zx, y_of(*z))],
                    s * 0.007,
                    *zc,
                    false,
                );
            }
            for i in 0..=majors {
                let y = y_of(i as f32 / majors as f32);
                stroke(
                    w,
                    o,
                    &[(zx + s * 0.008, y), (zx + s * 0.024, y)],
                    s * 0.005,
                    ink.muted,
                    false,
                );
            }
        };
        let fig = self.figure();
        let fh = s * 0.1;
        let numerals = major_labels(&self).into_iter().map(move |(f, label)| {
            div()
                .absolute()
                .left(px(zx + s * 0.03))
                .top(px(y_of(f) - s * 0.017))
                .text_size(px(s * 0.028))
                .text_color(ink.muted)
                .child(label)
                .into_any_element()
        });
        div()
            .relative()
            .w(px(wd))
            .h(px(s))
            .child(layer(paint))
            .children(numerals)
            .child(
                text_box(wd / 2.0, s * 0.06, wd * 0.9)
                    .text_size(px(s * 0.032))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(ink.primary)
                    .child(self.caption().to_uppercase()),
            )
            .child(seg_text(&fig, wd / 2.0, s * 0.11, fh, col))
            .child(
                text_box(wd / 2.0, s * 0.235, wd * 0.9)
                    .text_size(px(s * 0.028))
                    .text_color(ink.muted)
                    .child(self.lcd_unit()),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_segment_leans_forward_by_its_lean() {
        let p = slanted((10.0, 20.0, 5.0, 8.0), 2.0);
        assert_eq!(p[0], (12.0, 20.0), "the top starts further right");
        assert_eq!(p[3], (10.0, 28.0), "the foot stays put");
        assert_eq!(p[1].0 - p[0].0, 5.0);
    }
}
