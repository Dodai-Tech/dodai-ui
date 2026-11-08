//! Two layouts taken over from the single-piece clusters: the heads-up cut (a cut-corner frame
//! with a fan of hairlines that light up to the value) and the halo capsule (a stadium frame
//! with a band of light running from the primary to the accent colour). Every colour comes from
//! the theme.

use super::layouts::layer;
use super::rich::major_labels;
use super::round::ROUND;
use super::GaugeFace;
use crate::gpui::dash::text_box;
use crate::gpui::draw::{polar, turn_arc};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill, fill_circle, scale_alpha, stroke};
use dodai_gpui::theme_of;
use gpui::{div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, Pixels, Styled};

/// Height of the halo capsule, as a share of its width.
const CAPSULE_H: f32 = 0.5;

/// The eight corners of a rectangle `(x, y, w, h)` with its corners cut by `k`. Pure.
fn cut(x: f32, y: f32, w: f32, h: f32, k: f32) -> Vec<(f32, f32)> {
    vec![
        (x + k, y),
        (x + w - k, y),
        (x + w, y + k),
        (x + w, y + h - k),
        (x + w - k, y + h),
        (x + k, y + h),
        (x, y + h - k),
        (x, y + k),
    ]
}

/// A stadium (a rectangle with round ends) `w` by `h` from the origin. Pure.
fn stadium(w: f32, h: f32) -> Vec<(f32, f32)> {
    let r = h / 2.0;
    let left = (0..=32).map(|i| polar(r, r, r, 0.5 + 0.5 * i as f32 / 32.0));
    let right = (0..=32).map(|i| polar(w - r, r, r, 0.5 * i as f32 / 32.0));
    left.chain(right).collect()
}

impl GaugeFace {
    /// The heads-up cut: a cut-corner frame of hairlines, a fan of 180 hairlines lit up to the
    /// value, a glowing needle across the fan and the figure large in the middle.
    pub(super) fn hud_cut(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let c = s / 2.0;
        let rd = s * 0.34;
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f.clamp(0.0, 1.0);
        let frac = self.frac();
        let col = self.value_colour(&t, &ink);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let majors: Vec<f32> = major_labels(&self).into_iter().map(|(f, _)| f).collect();
        let hair = (s * 0.003).max(0.6);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let k = s * 0.16;
            let frame = cut(1.0, 1.0, s - 2.0, s - 2.0, k);
            fill(w, o, &frame, ink.ground);
            stroke(
                w,
                o,
                &frame,
                hair * 1.5,
                scale_alpha(ink.primary, 0.5),
                true,
            );
            let inner = cut(s * 0.025, s * 0.025, s * 0.95, s * 0.95, k * 0.9);
            stroke(w, o, &inner, hair, scale_alpha(ink.text, 0.12), true);
            let fine = 180usize;
            let mut fine_lines = crate::gpui::draw::Strokes::default();
            for i in 0..=fine {
                let f = i as f32 / fine as f32;
                let on = f <= frac;
                let c2 = if on { col } else { ink.text };
                let (a_in, a_out) = if on { (0.25, 0.9) } else { (0.06, 0.25) };
                let mid = rd * 0.78;
                fine_lines.add(
                    vec![polar(c, c, rd * 0.5, at(f)), polar(c, c, mid, at(f))],
                    hair,
                    scale_alpha(c2, a_in),
                );
                fine_lines.add(
                    vec![polar(c, c, mid, at(f)), polar(c, c, rd, at(f))],
                    hair,
                    scale_alpha(c2, a_out),
                );
            }
            fine_lines.paint(w, o);
            for f in &majors {
                stroke(
                    w,
                    o,
                    &[
                        polar(c, c, rd * 1.02, at(*f)),
                        polar(c, c, rd * 1.1, at(*f)),
                    ],
                    hair * 2.0,
                    ink.muted,
                    false,
                );
            }
            let needle = [
                polar(c, c, rd * 0.62, at(frac)),
                polar(c, c, rd * 1.14, at(frac)),
            ];
            crate::gpui::mfd::glow(w, o, &needle, (s * 0.006).max(1.5), col, false);
        };
        let labels = major_labels(&self).into_iter().map(move |(f, txt)| {
            let (x, y) = polar(c, c, rd * 1.24, at(f));
            text_box(x, y - s * 0.02, s * 0.12)
                .text_size(px(s * 0.032))
                .text_color(ink.muted)
                .child(txt)
                .into_any_element()
        });
        div()
            .relative()
            .size(px(s))
            .child(layer(paint))
            .children(labels)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(s * 0.032))
                            .text_color(ink.muted)
                            .child(self.unit.to_uppercase()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.2))
                            .line_height(px(s * 0.22))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(s * 0.03))
                            .text_color(ink.primary)
                            .child(self.caption().to_uppercase()),
                    ),
            )
            .into_any_element()
    }

    /// The halo capsule: a stadium of hairline with a lit edge along its foot running from the
    /// primary to the accent colour, and a band of light over three quarters of a turn in the
    /// middle, coloured the same way up to the value, round the figure; the scale's ends stand at
    /// the two sides.
    pub(super) fn halo_capsule(self, cx: &App) -> AnyElement {
        let t = theme_of(cx);
        let ink = self.lit_ink(&t);
        let s = self.size;
        let h = s * CAPSULE_H;
        let (mx, my) = (s / 2.0, h / 2.0);
        let r_band = h * 0.4;
        let band = (h * 0.035).max(2.5);
        let (start, span) = ROUND;
        let at = move |f: f32| start + span * f.clamp(0.0, 1.0);
        let frac = self.frac();
        let alarm = self.limit(&t);
        let figure_col = self.display_limit(&t).unwrap_or(ink.text);
        let ticks: Vec<(f32, bool)> = self
            .scale
            .ticks()
            .into_iter()
            .map(|k| (k.frac, k.major))
            .collect();
        let hair = (s * 0.002).max(0.6);
        let (a0, a1) = (ink.primary, ink.accent);
        let paint = move |w: &mut gpui::Window, o: gpui::Point<Pixels>| {
            let outline = stadium(s - 1.0, h - 1.0);
            fill(w, o, &outline, scale_alpha(ink.ground, 0.6));
            stroke(w, o, &outline, hair, scale_alpha(ink.text, 0.3), true);
            let r = h / 2.0;
            let n = 40;
            for i in 0..n {
                let f = i as f32 / n as f32;
                let x0 = r + (s - 2.0 * r) * f;
                let x1 = r + (s - 2.0 * r) * (f + 1.0 / n as f32);
                stroke(
                    w,
                    o,
                    &[(x0, h - 1.5), (x1, h - 1.5)],
                    hair * 1.6,
                    scale_alpha(mix(a0, a1, f), 0.9),
                    false,
                );
            }
            stroke(
                w,
                o,
                &turn_arc(mx, my, r_band, at(0.0), at(1.0)),
                band,
                scale_alpha(ink.text, 0.08),
                false,
            );
            let n = 48;
            for i in 0..n {
                let (f0, f1) = (frac * i as f32 / n as f32, frac * (i + 1) as f32 / n as f32);
                let c = alarm.unwrap_or_else(|| mix(a0, a1, i as f32 / n as f32));
                stroke(
                    w,
                    o,
                    &turn_arc(mx, my, r_band, at(f0), at(f1)),
                    band,
                    c,
                    false,
                );
            }
            let knob = polar(mx, my, r_band, at(frac));
            fill_circle(w, o, knob, band * 1.3, alarm.unwrap_or(a1));
            fill_circle(w, o, knob, band * 0.7, ink.ground);
            for (f, major) in &ticks {
                let len = if *major { h * 0.04 } else { h * 0.02 };
                let ro = r_band - band - h * 0.012;
                stroke(
                    w,
                    o,
                    &[polar(mx, my, ro, at(*f)), polar(mx, my, ro - len, at(*f))],
                    if *major { hair * 1.6 } else { hair },
                    scale_alpha(ink.text, if *major { 0.8 } else { 0.35 }),
                    false,
                );
            }
        };
        let side = |left: bool, top: String, sub: gpui::SharedString| {
            let d = div()
                .absolute()
                .top(px(my - h * 0.07))
                .w(px(s * 0.22))
                .flex()
                .flex_col()
                .child(
                    div()
                        .text_size(px(h * 0.07))
                        .font_weight(FontWeight::LIGHT)
                        .text_color(ink.text)
                        .child(top),
                )
                .child(
                    div()
                        .text_size(px(h * 0.045))
                        .text_color(ink.muted)
                        .child(sub),
                );
            if left {
                d.left(px(h * 0.3)).items_start()
            } else {
                d.left(px(s - h * 0.3 - s * 0.22)).items_end()
            }
        };
        let (lo, hi) = (self.scale.min, self.scale.max);
        let ends = (
            crate::instrument::readout(lo, 0, ""),
            crate::instrument::readout(hi, 0, ""),
        );
        div()
            .relative()
            .w(px(s))
            .h(px(h))
            .child(layer(paint))
            .child(side(true, ends.0, "MIN".into()))
            .child(side(false, ends.1, "MAX".into()))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .size_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .child(
                        div()
                            .text_size(px(h * 0.18))
                            .line_height(px(h * 0.19))
                            .font_weight(FontWeight::LIGHT)
                            .text_color(figure_col)
                            .child(self.figure()),
                    )
                    .child(
                        div()
                            .text_size(px(h * 0.045))
                            .text_color(ink.muted)
                            .child(self.unit.clone()),
                    )
                    .child(
                        div()
                            .mt(px(h * 0.02))
                            .text_size(px(h * 0.04))
                            .text_color(a1)
                            .child(self.caption().to_uppercase()),
                    ),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{cut, stadium};

    #[test]
    fn a_cut_frame_has_eight_corners_inside_its_box() {
        let f = cut(0.0, 0.0, 100.0, 80.0, 10.0);
        assert_eq!(f.len(), 8);
        assert!(f
            .iter()
            .all(|&(x, y)| (0.0..=100.0).contains(&x) && (0.0..=80.0).contains(&y)));
    }

    #[test]
    fn a_stadium_spans_its_width_and_height() {
        let p = stadium(200.0, 50.0);
        let (min_x, max_x) = p
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), &(x, _)| (a.min(x), b.max(x)));
        assert!(min_x.abs() < 0.5 && (max_x - 200.0).abs() < 0.5);
    }
}
