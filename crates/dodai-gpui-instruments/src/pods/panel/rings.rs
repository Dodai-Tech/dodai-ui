//! The round layouts of a [`RowsPod`]: the rings (every level an open ring around the readings,
//! one inside the other) and, for the looks that are round, the strip and the column drawn as a
//! row or a stack of small round registers. Each sits in its look's own case: a polished bezel
//! for sport, the coin-edged chronograph case, a chrome ring on an aged face for vintage, the
//! heads-up dashes and brackets, the halo's glow and a single hairline for line.

use super::*;
use crate::frames::{halo, lit_segments, seg_arc};
use crate::shapes::{band, chrono_case, circle, hud_ring};

/// Whether `look` is drawn round, so its pod layouts keep to circles too. Pure.
pub(crate) fn round_look(look: Look) -> bool {
    matches!(
        look,
        Look::Sport | Look::Chronograph | Look::Vintage | Look::HeadsUp | Look::Halo | Look::Line
    )
}

/// Start and span of every ring's arc, in turns: open a quarter at the bottom.
const SWEEP: (f32, f32) = (0.625, 0.75);

/// The radius of the face inside `look`'s case for a disc `d` across. Pure.
pub(crate) fn face_r(look: Look, d: f32) -> f32 {
    let inset = match look {
        Look::Sport | Look::Line => 0.03,
        Look::Chronograph => 0.06,
        Look::Vintage => 0.055,
        Look::HeadsUp => 0.075,
        Look::Halo => 0.095,
        Look::Segment | Look::GlassCockpit | Look::Race => margin(look),
    };
    d / 2.0 - d * inset
}

/// Radii of `n` concentric rings inside a face of radius `r`, `step` apart, the first half a
/// step in. Pure.
pub(crate) fn ring_radii(r: f32, n: usize, step: f32) -> Vec<f32> {
    (0..n).map(|i| r - step * (i as f32 + 0.5)).collect()
}

/// Centres (from the middle of the face) and the diameter of `n` round registers laid out
/// inside a face of radius `r`: one in the middle, two side by side, three or more on a circle
/// (four in a square), each as large as keeps them clear of each other and of the rim. Pure.
pub(crate) fn register_spots(n: usize, r: f32) -> (Vec<(f32, f32)>, f32) {
    if n <= 1 {
        return (vec![(0.0, 0.0); n], r * 1.1);
    }
    let k = (std::f32::consts::PI / n as f32).sin() * 0.88;
    let rho = r * 0.9 / (1.0 + k);
    let start = match n {
        2 => 0.25,
        4 => 0.125,
        _ => 0.0,
    };
    let spots = (0..n)
        .map(|i| polar(0.0, 0.0, rho, start + i as f32 / n as f32))
        .collect();
    (spots, 2.0 * k * rho)
}

/// Paint `look`'s case for a disc `d` across with its top left at `at`.
fn case(w: &mut Window, o: gpui::Point<Pixels>, look: Look, at: (f32, f32), d: f32, ink: Ink) {
    let o = o + gpui::point(px(at.0), px(at.1));
    let c = d / 2.0;
    let hair = (d * 0.004).max(0.6);
    match look {
        Look::Sport => {
            fill(w, o, &circle(c, c, c), ink.ground);
            fill(w, o, &circle(c, c, c), scale_alpha(ink.text, 0.05));
            stroke(
                w,
                o,
                &circle(c, c, c),
                hair * 1.6,
                scale_alpha(ink.text, 0.5),
                true,
            );
            stroke(
                w,
                o,
                &circle(c, c, c - d * 0.018),
                hair,
                scale_alpha(ink.text, 0.16),
                true,
            );
        }
        Look::Chronograph => {
            chrono_case(w, o, d, ink);
        }
        Look::Vintage => {
            fill(w, o, &circle(c, c, c), ink.face);
            fill(w, o, &circle(c, c, c), scale_alpha(ink.text, 0.1));
            stroke(
                w,
                o,
                &circle(c, c, c),
                hair * 1.4,
                scale_alpha(ink.text, 0.45),
                true,
            );
            let r = c - d * 0.04;
            fill(w, o, &circle(c, c, r), aged(ink));
            stroke(
                w,
                o,
                &circle(c, c, r),
                hair,
                scale_alpha(ink.text, 0.3),
                true,
            );
        }
        Look::HeadsUp => {
            hud_ring(w, o, d, ink, ink.primary);
        }
        Look::Halo => {
            halo(w, o, d, ink);
        }
        Look::Line => stroke(w, o, &circle(c, c, c - hair), hair, ink.muted, true),
        Look::Segment | Look::GlassCockpit | Look::Race => frame(w, o, look, (d, d), d, ink),
    }
}

/// Paint one level as an open ring of radius `r` around `c`, `bw` thick and `f` full in `col`,
/// the look's way: blocks for race and segment, a glowing line for heads-up and halo, hairlines
/// for line and a solid band with round ends elsewhere.
#[allow(clippy::too_many_arguments)]
fn level_ring(
    w: &mut Window,
    o: gpui::Point<Pixels>,
    look: Look,
    c: (f32, f32),
    (r, bw): (f32, f32),
    f: f32,
    col: Hsla,
    ty: Type,
) {
    let f = pm::unit(f);
    let (a, span) = SWEEP;
    let hair = (bw * 0.12).max(0.6);
    match look {
        Look::Race | Look::Segment => {
            let lit = lit_segments(24, f, |_| col);
            seg_arc(w, o, c, (r - bw / 2.0, r + bw / 2.0), SWEEP, &lit, ty.track);
        }
        Look::HeadsUp | Look::Halo => {
            stroke(
                w,
                o,
                &turn_arc(c.0, c.1, r, a, a + span),
                hair,
                ty.track,
                false,
            );
            if f > 0.0 {
                glow(
                    w,
                    o,
                    &turn_arc(c.0, c.1, r, a, a + span * f),
                    bw * 0.45,
                    col,
                    false,
                );
            }
        }
        Look::Line => {
            stroke(
                w,
                o,
                &turn_arc(c.0, c.1, r, a, a + span),
                hair,
                ty.track,
                false,
            );
            if f > 0.0 {
                stroke(
                    w,
                    o,
                    &turn_arc(c.0, c.1, r, a, a + span * f),
                    hair * 2.4,
                    col,
                    false,
                );
                let (x, y) = polar(c.0, c.1, r, a + span * f);
                fill(w, o, &circle(x, y, hair * 3.0), col);
            }
        }
        Look::Sport | Look::Chronograph | Look::Vintage | Look::GlassCockpit => {
            let (r0, r1) = (r - bw / 2.0, r + bw / 2.0);
            fill(w, o, &band(c.0, c.1, r0, r1, a, a + span), ty.track);
            for end in [a, a + span] {
                let (x, y) = polar(c.0, c.1, r, end);
                fill(w, o, &circle(x, y, bw / 2.0), ty.track);
            }
            if f > 0.0 {
                fill(w, o, &band(c.0, c.1, r0, r1, a, a + span * f), col);
                for end in [a, a + span * f] {
                    let (x, y) = polar(c.0, c.1, r, end);
                    fill(w, o, &circle(x, y, bw / 2.0), col);
                }
            }
        }
    }
}

impl RowsPod {
    /// The pod as rings: every level an open ring, one inside the other, around the readings.
    /// With no levels to show, the readings are round registers inside the case instead.
    pub(super) fn ringed(self, t: &dodai_core::Theme) -> AnyElement {
        if self.rows.iter().all(|r| r.bar.is_none()) {
            let s = self.size;
            let (spots, d) = register_spots(self.rows.len(), face_r(self.look, s) - s * 0.02);
            let spots = spots
                .into_iter()
                .map(|(x, y)| (s / 2.0 + x - d / 2.0, s / 2.0 + y - d / 2.0))
                .collect();
            return self.dials(t, (s, s), spots, d, Heading::Middle);
        }
        let look = self.look;
        let ink = Ink::of(t);
        let ty = type_of(look, ink);
        let s = self.size;
        let tone = |tn: Tone| super::super::detailed::tone_or(t, tn, ink);
        let levels: Vec<(f32, Hsla)> = self
            .rows
            .iter()
            .filter_map(|r| r.bar.map(|(f, tn)| (f, tone(tn))))
            .take(4)
            .collect();
        let step = s * 0.062;
        let bw = s * 0.03;
        let radii = ring_radii(face_r(look, s) - s * 0.02, levels.len(), step);
        let inner = radii
            .last()
            .map_or(face_r(look, s) - s * 0.04, |r| r - step * 0.5 - s * 0.01);
        let c = s / 2.0;
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            case(w, o, look, (0.0, 0.0), s, ink);
            for (r, (f, col)) in radii.iter().zip(&levels) {
                level_ring(w, o, look, (c, c), (*r, bw), *f, *col, ty);
            }
        };
        let n = self.rows.len().max(1) as f32;
        let wd = inner * 1.5;
        let fs = (inner * 1.2 / n / 1.35).min(s * 0.055);
        let mut lit = self
            .rows
            .iter()
            .filter_map(|r| r.bar.map(|(_, tn)| tone(tn)));
        let list = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(fs * 0.25))
            .when(!self.heading.is_empty(), |d| {
                d.child(
                    div()
                        .text_size(px(fs * 0.6))
                        .text_color(ty.heading)
                        .font_weight(FontWeight::MEDIUM)
                        .child(if ty.caps {
                            SharedString::from(self.heading.to_uppercase())
                        } else {
                            self.heading.clone()
                        }),
                )
            })
            .children(self.rows.iter().map(|r| {
                let icon_col = if r.bar.is_some() {
                    lit.next().unwrap_or(ty.muted)
                } else {
                    ty.muted
                };
                let fs = fit_text(fs, wd - fs, &r.value).max(fs * 0.6);
                div()
                    .flex()
                    .items_center()
                    .gap(px(fs * 0.3))
                    .child(r.icon.element(fs * 0.75, icon_col))
                    .child(
                        div()
                            .text_size(px(fs))
                            .font_weight(ty.weight)
                            .text_color(ty.value)
                            .child(r.value.clone()),
                    )
            }));
        div()
            .relative()
            .size(px(s))
            .child(super::super::detailed::layer(paint))
            .child(
                div()
                    .absolute()
                    .left(px(c - wd / 2.0))
                    .top(px(c - inner))
                    .w(px(wd))
                    .h(px(inner * 2.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(list),
            )
            .into_any_element()
    }

    /// The pod as a strip or a column of round registers, one per reading, for a round look.
    pub(super) fn registers(self, t: &dodai_core::Theme, layout: PodLayout) -> AnyElement {
        let s = self.size;
        let (wd, h) = pod_box(layout, s);
        let head_h = if self.heading.is_empty() {
            0.0
        } else {
            s * 0.065
        };
        let n = self.rows.len().max(1);
        let across = layout == PodLayout::Strip;
        let (cell_w, cell_h) = if across {
            (wd / n as f32, h - head_h)
        } else {
            (wd, (h - head_h) / n as f32)
        };
        let d = cell_w.min(cell_h) * 0.94;
        let spots: Vec<(f32, f32)> = (0..n)
            .map(|i| {
                let (cx, cy) = if across {
                    (cell_w * (i as f32 + 0.5), head_h + cell_h / 2.0)
                } else {
                    (wd / 2.0, head_h + cell_h * (i as f32 + 0.5))
                };
                (cx - d / 2.0, cy - d / 2.0)
            })
            .collect();
        self.dials(t, (wd, h), spots, d, Heading::Top(head_h))
    }

    /// Paint one round register `d` across for every reading, top left at `spots`, in a box of
    /// `wd` by `h`; a [`Heading::Middle`] box is the look's case itself.
    fn dials(
        self,
        t: &dodai_core::Theme,
        (wd, h): (f32, f32),
        spots: Vec<(f32, f32)>,
        d: f32,
        head: Heading,
    ) -> AnyElement {
        let look = self.look;
        let ink = Ink::of(t);
        let ty = type_of(look, ink);
        let s = self.size;
        let tone = |tn: Tone| super::super::detailed::tone_or(t, tn, ink);
        let bars: Vec<Option<(f32, Hsla)>> = self
            .rows
            .iter()
            .map(|r| r.bar.map(|(f, tn)| (f, tone(tn))))
            .collect();
        let r_arc = face_r(look, d) - d * 0.08;
        let bw = d * 0.055;
        let painted = spots.clone();
        let paint = move |w: &mut Window, o: gpui::Point<Pixels>| {
            if head == Heading::Middle {
                case(w, o, look, (0.0, 0.0), wd, ink);
            }
            for (at, bar) in painted.iter().zip(&bars) {
                case(w, o, look, *at, d, ink);
                if let Some((f, col)) = bar {
                    let c = (at.0 + d / 2.0, at.1 + d / 2.0);
                    level_ring(w, o, look, c, (r_arc, bw), *f, *col, ty);
                }
            }
        };
        let inner = r_arc - bw;
        let fs = d * 0.17;
        let (head_top, head_h, head_fs) = match head {
            Heading::Top(hh) => (0.0, hh, s * 0.04),
            Heading::Middle => (h / 2.0 - s * 0.03, s * 0.06, s * 0.03),
        };
        let heading = (!self.heading.is_empty()).then(|| {
            div()
                .absolute()
                .left_0()
                .top(px(head_top))
                .w(px(wd))
                .h(px(head_h))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(head_fs))
                .text_color(ty.heading)
                .font_weight(FontWeight::MEDIUM)
                .child(if ty.caps {
                    SharedString::from(self.heading.to_uppercase())
                } else {
                    self.heading.clone()
                })
        });
        div()
            .relative()
            .w(px(wd))
            .h(px(h))
            .child(super::super::detailed::layer(paint))
            .when_some(heading, |e, hd| e.child(hd))
            .children(self.rows.iter().zip(&spots).map(|(r, at)| {
                // A long reading shrinks a little, then wraps onto a second line.
                let fs = fit_text(fs, inner * 1.5, &r.value).max(fs * 0.6);
                div()
                    .absolute()
                    .left(px(at.0 + d / 2.0 - inner))
                    .top(px(at.1 + d / 2.0 - inner))
                    .size(px(inner * 2.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(d * 0.02))
                    .child(r.icon.element(d * 0.11, ty.muted))
                    .child(
                        div()
                            .w(px(inner * 1.9))
                            .text_center()
                            .line_height(gpui::relative(1.05))
                            .text_size(px(fs))
                            .font_weight(ty.weight)
                            .text_color(ty.value)
                            .child(r.value.clone()),
                    )
            }))
            .into_any_element()
    }
}

/// Where a register pod puts its heading.
#[derive(Clone, Copy, PartialEq)]
enum Heading {
    /// Over the registers, in a band this tall.
    Top(f32),
    /// In the middle of the case the registers sit in.
    Middle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_round_looks_keep_their_layouts_round() {
        assert!(round_look(Look::Sport) && round_look(Look::Line));
        assert!(!round_look(Look::Race) && !round_look(Look::GlassCockpit));
    }

    #[test]
    fn rings_nest_inside_the_face() {
        let r = face_r(Look::Halo, 200.0);
        assert!(r < 100.0);
        let radii = ring_radii(r, 3, 12.0);
        assert_eq!(radii, vec![r - 6.0, r - 18.0, r - 30.0]);
        assert!(ring_radii(r, 0, 12.0).is_empty());
    }

    #[test]
    fn registers_stay_clear_of_each_other_and_the_rim() {
        for n in 2..=6 {
            let (spots, d) = register_spots(n, 100.0);
            assert_eq!(spots.len(), n);
            for (i, a) in spots.iter().enumerate() {
                assert!(a.0.hypot(a.1) + d / 2.0 <= 90.01);
                for b in &spots[i + 1..] {
                    assert!((a.0 - b.0).hypot(a.1 - b.1) >= d);
                }
            }
        }
        assert_eq!(register_spots(1, 100.0).0, vec![(0.0, 0.0)]);
    }
}
