//! The rings, strip and column layouts of a [`RowsPod`] (see [`PodLayout`]), in every look: the
//! look's own frame painted behind the readings, its type and its level bars. Sport and
//! chronograph sit in a polished bezel, vintage on a screwed panel with an aged window, heads-up
//! between corner brackets, segment in an LCD module, halo in a glowing capsule, glass cockpit on
//! an avionics panel, race on a carbon plate with a slanted tag and line on nothing but hairlines.

use super::detailed::layer;
use super::*;
use crate::gpui::frames::{aged, lcd, screw};
use crate::gpui::look::PodLayout;
use crate::gpui::mfd::{glow, panel};
use crate::gpui::race::{plate_in, tag, tag_text};
use crate::gpui::shapes::{chamfered, rounded, Ink};
use dodai_gpui::paint::{fill_rect, stroke_rect};

mod rings;

/// Height of the strip, as a share of its width.
pub(crate) const STRIP_H: f32 = 0.46;
/// Width of the column, as a share of its height.
pub(crate) const COLUMN_W: f32 = 0.5;

/// The box of `layout` for a pod of `s`: its width and height. Pure.
pub(crate) fn pod_box(layout: PodLayout, s: f32) -> (f32, f32) {
    match layout {
        PodLayout::Round | PodLayout::Rings => (s, s),
        PodLayout::Strip => (s, s * STRIP_H),
        PodLayout::Column => (s * COLUMN_W, s),
    }
}

/// The text size for `text` at most `fs` that still fits `avail` px: a figure is about 0.56 of
/// its size wide. Pure.
fn fit_text(fs: f32, avail: f32, text: &str) -> f32 {
    let n = text.chars().count().max(1) as f32;
    fs.min(avail.max(0.0) / (0.56 * n))
}

/// The inner margin of a look's frame, as a share of the pod: wider where the frame has a
/// bezel, a panel or screws. Pure.
fn margin(look: Look) -> f32 {
    match look {
        Look::Vintage | Look::Segment | Look::GlassCockpit => 0.075,
        Look::Sport | Look::Chronograph | Look::Halo | Look::Race => 0.06,
        Look::HeadsUp => 0.055,
        Look::Line => 0.02,
    }
}

/// The type and colours of a look's readings.
#[derive(Clone, Copy)]
struct Type {
    /// The figures.
    value: Hsla,
    /// Icons and small print.
    muted: Hsla,
    /// The heading.
    heading: Hsla,
    /// The empty part of a level bar.
    track: Hsla,
    weight: FontWeight,
    /// Headings in capitals.
    caps: bool,
}

fn type_of(look: Look, ink: Ink) -> Type {
    let base = Type {
        value: ink.text,
        muted: ink.muted,
        heading: ink.muted,
        track: scale_alpha(ink.text, 0.12),
        weight: FontWeight::LIGHT,
        caps: true,
    };
    match look {
        Look::Vintage => Type {
            weight: FontWeight::NORMAL,
            track: scale_alpha(ink.text, 0.16),
            ..base
        },
        Look::HeadsUp => Type {
            value: ink.primary,
            muted: scale_alpha(ink.primary, 0.7),
            heading: ink.primary,
            track: scale_alpha(ink.primary, 0.14),
            weight: FontWeight::NORMAL,
            ..base
        },
        Look::Segment => Type {
            value: ink.primary,
            muted: scale_alpha(ink.primary, 0.65),
            heading: scale_alpha(ink.primary, 0.8),
            track: scale_alpha(ink.primary, 0.1),
            weight: FontWeight::MEDIUM,
            ..base
        },
        Look::Halo => Type {
            heading: ink.primary,
            ..base
        },
        Look::GlassCockpit => Type {
            heading: ink.info,
            weight: FontWeight::MEDIUM,
            ..base
        },
        Look::Race => Type {
            weight: FontWeight::BOLD,
            ..base
        },
        Look::Sport | Look::Chronograph | Look::Line => base,
    }
}

/// Paint `look`'s frame over a box `wd` by `h` of a pod of `s`.
fn frame(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    look: Look,
    (wd, h): (f32, f32),
    s: f32,
    ink: Ink,
) {
    let hair = (s * 0.003).max(0.6);
    match look {
        Look::Sport | Look::Chronograph => {
            // A polished bezel: a bright outer hairline, a shaded inner one and a dark glass.
            let r = if look == Look::Sport {
                s * 0.07
            } else {
                s * 0.1
            };
            let outer = rounded(0.0, 0.0, wd, h, r);
            fill(w, o, &outer, ink.ground);
            fill(w, o, &outer, scale_alpha(ink.text, 0.05));
            stroke(w, o, &outer, hair * 1.6, scale_alpha(ink.text, 0.5), true);
            let k = s * 0.018;
            let inner = rounded(k, k, wd - 2.0 * k, h - 2.0 * k, r - k);
            stroke(w, o, &inner, hair, scale_alpha(ink.text, 0.16), true);
            for i in 0..4 {
                fill(
                    w,
                    o,
                    &crate::gpui::mfd::sheen(
                        (k, k, wd - 2.0 * k, h - 2.0 * k),
                        0.25 + 0.1 * i as f32,
                    ),
                    scale_alpha(ink.text, 0.01),
                );
            }
            if look == Look::Chronograph {
                // A railway track just inside the bezel, along the top.
                let y = k * 2.2;
                let (x0, x1) = (r, wd - r);
                stroke(w, o, &[(x0, y), (x1, y)], hair * 0.6, ink.muted, false);
                let n = ((x1 - x0) / (s * 0.02)).max(1.0) as usize;
                for i in 0..=n {
                    let x = x0 + (x1 - x0) * i as f32 / n as f32;
                    let l = if i % 5 == 0 { s * 0.016 } else { s * 0.008 };
                    stroke(w, o, &[(x, y), (x, y + l)], hair * 0.7, ink.muted, false);
                }
            }
        }
        Look::Vintage => {
            let p = chamfered(0.0, 0.0, wd, h, s * 0.03);
            fill(w, o, &p, ink.face);
            fill(w, o, &p, scale_alpha(ink.text, 0.08));
            stroke(w, o, &p, hair, scale_alpha(ink.text, 0.3), true);
            let k = s * 0.05;
            let win = chamfered(k, k, wd - 2.0 * k, h - 2.0 * k, s * 0.02);
            fill(w, o, &win, aged(ink));
            stroke(w, o, &win, hair * 1.4, scale_alpha(ink.text, 0.35), true);
            let e = s * 0.025;
            for (i, at) in [(e, e), (wd - e, e), (e, h - e), (wd - e, h - e)]
                .into_iter()
                .enumerate()
            {
                screw(w, o, at, s * 0.012, 0.1 + 0.2 * i as f32, ink);
            }
        }
        Look::HeadsUp => {
            let (l, lw) = (s * 0.07, (s * 0.004).max(1.0));
            for (x, y, dx, dy) in [
                (0.0, 0.0, 1.0, 1.0),
                (wd, 0.0, -1.0, 1.0),
                (0.0, h, 1.0, -1.0),
                (wd, h, -1.0, -1.0),
            ] {
                let pts = [(x + dx * l, y), (x, y), (x, y + dy * l)];
                glow(w, o, &pts, lw, ink.primary, false);
            }
            fill_rect(
                w,
                o,
                (0.0, 0.0, wd, h),
                s * 0.02,
                scale_alpha(ink.primary, 0.03),
            );
        }
        Look::Segment => {
            lcd(w, o, (0.0, 0.0, wd, h), s, ink);
        }
        Look::Halo => {
            let r = (wd.min(h) * 0.5).min(s * 0.16);
            let p = rounded(0.0, 0.0, wd, h, r);
            for i in 1..=3 {
                let k = s * 0.01 * i as f32;
                stroke_rect(
                    w,
                    o,
                    (-k, -k, wd + 2.0 * k, h + 2.0 * k),
                    r + k,
                    hair,
                    scale_alpha(ink.primary, 0.05),
                );
            }
            fill(w, o, &p, ink.ground);
            fill(w, o, &p, scale_alpha(ink.primary, 0.04));
            glow(w, o, &p, hair, scale_alpha(ink.primary, 0.6), true);
        }
        Look::GlassCockpit => {
            panel(w, o, (wd, h), ink);
        }
        Look::Race => plate_in(w, o, (wd, h), s, ink),
        Look::Line => {}
    }
}

/// A level bar `f` full in `col`, `wd` wide (or filling its row) and `h` thick, drawn the look's
/// way: blocks on the race plate and the LCD, a glowing line on the halo, a hairline with a dot
/// for line, a slim rule elsewhere.
fn level(look: Look, f: f32, col: Hsla, ty: Type, h: f32) -> gpui::Div {
    let f = pm::unit(f);
    match look {
        Look::Race | Look::Segment => {
            let n = 12;
            let lit = crate::gpui::race::lit_count(f, n);
            div()
                .w_full()
                .h(px(h * 1.6))
                .flex()
                .gap(px(h * 0.4))
                .children((0..n).map(|i| {
                    div()
                        .flex_1()
                        .h_full()
                        .bg(if i < lit { col } else { ty.track })
                }))
        }
        Look::Line => div()
            .w_full()
            .h(px(h * 2.2))
            .relative()
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(h * 1.0))
                    .h(px((h * 0.25).max(0.6)))
                    .bg(ty.track),
            )
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(h * 0.9))
                    .h(px((h * 0.45).max(1.0)))
                    .w(gpui::relative(f))
                    .bg(col),
            ),
        _ => {
            let radius = if matches!(look, Look::Halo | Look::Sport | Look::Chronograph) {
                h
            } else {
                0.0
            };
            let glow_h = if look == Look::Halo { h * 0.6 } else { 0.0 };
            div()
                .w_full()
                .h(px(h))
                .rounded(px(radius))
                .bg(ty.track)
                .child(
                    div()
                        .h_full()
                        .w(gpui::relative(f))
                        .rounded(px(radius))
                        .bg(col)
                        .when(glow_h > 0.0, |d| {
                            d.shadow(vec![gpui::BoxShadow {
                                color: scale_alpha(col, 0.6),
                                offset: gpui::point(px(0.0), px(0.0)),
                                blur_radius: px(glow_h * 3.0),
                                spread_radius: px(0.0),
                            }])
                        }),
                )
        }
    }
}

impl RowsPod {
    /// The pod as rings, a strip or a column (see [`PodLayout`]) in its look.
    pub(super) fn laid_out(self, t: &dodai_core::Theme, layout: PodLayout) -> AnyElement {
        if layout == PodLayout::Rings {
            return self.ringed(t);
        }
        if rings::round_look(self.look) {
            return self.registers(t, layout);
        }
        let look = self.look;
        let ink = Ink::of(t);
        let ty = type_of(look, ink);
        let s = self.size;
        let (wd, h) = pod_box(layout, s);
        let m = s * margin(look);
        let tagged = look == Look::Race && !self.heading.is_empty();
        let race_tag = (m, m * 0.7, s * 0.4, s * 0.06);
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            frame(w, o, look, (wd, h), s, ink);
            if tagged {
                tag(w, o, race_tag, ink.primary);
            }
        };
        let bar_h = (s * 0.012).max(2.0);
        let tone = |tn: Tone| super::detailed::tone_or(t, tn, ink);
        let hair = (s * 0.003).max(0.6);
        let n = self.rows.len().max(1) as f32;
        let head_size = s * if layout == PodLayout::Strip {
            0.04
        } else {
            0.045
        };
        let head = (!self.heading.is_empty() && look != Look::Race).then(|| {
            div()
                .text_size(px(head_size))
                .text_color(ty.heading)
                .font_weight(FontWeight::MEDIUM)
                .child(if ty.caps {
                    SharedString::from(self.heading.to_uppercase())
                } else {
                    self.heading.clone()
                })
        });
        let rule = |vertical: bool| {
            let d = div().bg(scale_alpha(ty.muted, 0.25)).flex_none();
            if vertical {
                d.w(px(hair)).h_full()
            } else {
                d.h(px(hair)).w_full()
            }
        };
        let ruled = matches!(
            look,
            Look::Sport | Look::Chronograph | Look::Line | Look::Vintage
        );
        let body = match layout {
            PodLayout::Strip => {
                let fs = (s * 0.32 / n).min(s * 0.06);
                let cell_w = (wd - 2.0 * m) / n - s * 0.03;
                let mut row = div().flex().flex_1().items_center();
                for (i, r) in self.rows.iter().enumerate() {
                    // A long reading shrinks, but no further than three fifths; past that it
                    // wraps onto a second line.
                    let fs = fit_text(fs, cell_w, &r.value).max(fs * 0.6);
                    if i > 0 && ruled {
                        row = row.child(rule(true));
                    }
                    row = row.child(
                        div()
                            .flex_1()
                            .h_full()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .gap(px(s * 0.012))
                            .px(px(s * 0.015))
                            .child(r.icon.element(fs * 0.9, ty.muted))
                            .child(
                                div()
                                    .w_full()
                                    .text_center()
                                    .line_height(gpui::relative(1.1))
                                    .text_size(px(fs))
                                    .font_weight(ty.weight)
                                    .text_color(ty.value)
                                    .child(r.value.clone()),
                            )
                            .child(match r.bar {
                                Some((f, tn)) => {
                                    div()
                                        .w_full()
                                        .child(level(look, f, tone(tn), ty, bar_h * 0.8))
                                }
                                None => div().h(px(bar_h)),
                            }),
                    );
                }
                row
            }
            // The round pod and the rings never come here; they would read as the column.
            PodLayout::Round | PodLayout::Rings | PodLayout::Column => {
                let fs = (s * 0.42 / n).min(s * 0.085);
                let inner = wd - 2.0 * m;
                div().flex().flex_col().flex_1().justify_around().children(
                    self.rows.iter().enumerate().map(|(i, r)| {
                        let fs = fit_text(fs, inner - fs * 0.6 - s * 0.02, &r.value).max(fs * 0.55);
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(s * 0.01))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(s * 0.02))
                                    .child(r.icon.element(fs * 0.6, ty.muted))
                                    .child(
                                        div()
                                            .flex_1()
                                            .line_height(gpui::relative(1.1))
                                            .text_size(px(fs))
                                            .font_weight(ty.weight)
                                            .text_color(ty.value)
                                            .child(r.value.clone()),
                                    ),
                            )
                            .when_some(r.bar, |d, (f, tn)| {
                                d.child(level(look, f, tone(tn), ty, bar_h * 0.8))
                            })
                            .when(ruled && i + 1 < self.rows.len(), |d| d.child(rule(false)))
                    }),
                )
            }
        };
        let top_gap = s * 0.07;
        div()
            .relative()
            .w(px(wd))
            .h(px(h))
            .child(layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(m))
                    .top(px(m))
                    .w(px(wd - 2.0 * m))
                    .h(px(h - 2.0 * m))
                    .flex()
                    .flex_col()
                    .gap(px(s * 0.02))
                    .when_some(head, |d, hd| d.child(hd))
                    .when(tagged, |d| d.child(div().h(px(top_gap))))
                    .child(body),
            )
            .when(tagged, |d| {
                d.child(tag_text(race_tag, ink.on_primary, self.heading.clone()))
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strip_is_wide_and_a_column_tall() {
        let s = 200.0;
        assert_eq!(pod_box(PodLayout::Round, s), (s, s));
        let (w, h) = pod_box(PodLayout::Strip, s);
        assert!(w > h);
        let (w, h) = pod_box(PodLayout::Column, s);
        assert!(h > w);
    }

    #[test]
    fn long_readings_shrink_to_fit() {
        assert_eq!(fit_text(20.0, 1000.0, "63 %"), 20.0);
        let f = fit_text(20.0, 80.0, "42 min to 80 %");
        assert!(f < 20.0 && f * 0.56 * 14.0 <= 80.0 + 1e-3);
        assert_eq!(fit_text(20.0, -5.0, "x"), 0.0);
    }

    #[test]
    fn a_soft_tyre_shows_a_short_bar() {
        let l = tyre_levels([Some(2.4), Some(2.4), Some(1.6), None]);
        assert!(l[2] < l[0] && l[0] < 1.0);
        assert_eq!(l[3], 0.0);
        assert_eq!(tyre_levels([None; 4]), [0.0; 4]);
        assert_eq!(tyre_levels([Some(f32::NAN), None, None, None])[0], 0.0);
    }

    #[test]
    fn framed_looks_keep_their_readings_off_the_frame() {
        assert!(margin(Look::Vintage) > margin(Look::Line));
        assert!(Look::ALL.iter().all(|&l| margin(l) < 0.1));
    }
}
