//! The telltale lamps and the speed-limit sign in every [`Look`] but Sport: bezels,
//! lamp lenses, chamfered frames, LCD blocks, halos, annunciator capsules and slanted race
//! lenses. Colours from the theme.

use super::*;
use crate::gpui::frames::{aged, ghost, glow_arc, glow_dot, seg_text};
use crate::gpui::mfd::rect;
use crate::gpui::race::skew;
use crate::gpui::shapes::{chamfered, Ink};
use dodai_gpui::motion::mix;
use dodai_gpui::paint::{fill_circle, fill_rect, stroke_circle};

/// A canvas filling its parent.
fn layer(paint: impl Fn(&mut Window, gpui::Point<Pixels>) + 'static) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |b: Bounds<Pixels>, _, w, _| paint(w, b.origin),
    )
    .absolute()
    .size_full()
}

/// One telltale of icon size `sz` in `look`: the backing drawn for the look, the symbol on top.
pub(super) fn lamp(icon: Symbol, tone: Tone, sz: f32, look: Look, t: &Theme) -> AnyElement {
    let ink = Ink::of(t);
    let col = tone.color(t);
    let lit = tone != Tone::Off;
    let d = sz * 1.55;
    let c = d / 2.0;
    let dark = t.is_dark();
    let icon_col = match look {
        Look::Segment if lit => ink.face,
        Look::Segment => ghost(ink.primary, dark),
        // A lit lens glows in the tone; the symbol on it reads in the text colour.
        Look::Vintage if lit => ink.text,
        // A lit race lens is filled with the tone; the symbol sits dark on it.
        Look::Race if lit => ink.ground,
        // An unlit line lamp is a faint outline; a lit one is the symbol alone in its tone.
        Look::Line if !lit => scale_alpha(ink.muted, 0.45),
        _ => col,
    };
    let paint = move |w: &mut Window, o: gpui::Point<Pixels>| match look {
        Look::Chronograph => {
            fill_circle(w, o, (c, c), c * 0.96, scale_alpha(ink.text, 0.06));
            stroke_circle(w, o, (c, c), c * 0.96, d * 0.04, ink.line);
            stroke_circle(w, o, (c, c), c * 0.8, d * 0.015, scale_alpha(ink.text, 0.2));
        }
        Look::Vintage => {
            fill_circle(w, o, (c, c), c, scale_alpha(ink.text, 0.25));
            fill_circle(w, o, (c, c), c * 0.82, ink.line);
            if lit {
                for k in 0..5 {
                    fill_circle(
                        w,
                        o,
                        (c, c),
                        c * 0.82 * (1.0 - k as f32 * 0.15),
                        scale_alpha(col, 0.2),
                    );
                }
            } else {
                fill_circle(w, o, (c, c), c * 0.82, scale_alpha(aged(ink), 0.5));
            }
            let mut glint = turn_arc(c, c, c * 0.68, 0.8, 0.97);
            glint.extend(turn_arc(c, c, c * 0.5, 0.97, 0.8));
            fill(w, o, &glint, scale_alpha(ink.text, 0.3));
        }
        Look::HeadsUp => {
            let frame = chamfered(d * 0.04, d * 0.04, d * 0.92, d * 0.92, d * 0.22);
            if lit {
                fill(w, o, &frame, scale_alpha(col, 0.12));
            }
            stroke(
                w,
                o,
                &frame,
                d * 0.035,
                scale_alpha(col, if lit { 0.9 } else { 0.4 }),
                true,
            );
        }
        Look::Segment => {
            let block = chamfered(d * 0.04, d * 0.04, d * 0.92, d * 0.92, d * 0.08);
            fill(
                w,
                o,
                &block,
                if lit { col } else { ghost(ink.primary, dark) },
            );
        }
        Look::Halo => {
            if lit {
                glow_dot(
                    w,
                    o,
                    (c, c),
                    c * 0.5,
                    scale_alpha(col, 0.35),
                    scale_alpha(col, 0.0),
                );
            }
            stroke_circle(
                w,
                o,
                (c, c),
                c * 0.92,
                d * 0.02,
                scale_alpha(col, if lit { 0.6 } else { 0.25 }),
            );
        }
        Look::GlassCockpit => {
            let frame = rect(d * 0.04, d * 0.14, d * 0.92, d * 0.72);
            fill(w, o, &frame, ink.ground);
            if lit {
                fill(w, o, &frame, scale_alpha(col, 0.16));
            }
            stroke(
                w,
                o,
                &frame,
                d * 0.04,
                scale_alpha(col, if lit { 1.0 } else { 0.35 }),
                true,
            );
        }
        Look::Race => {
            let lens = skew(d * 0.02, d * 0.2, d * 0.96, d * 0.6);
            if lit {
                fill(w, o, &lens, col);
            } else {
                fill(w, o, &lens, scale_alpha(ink.text, 0.05));
                stroke(w, o, &lens, d * 0.03, scale_alpha(col, 0.35), true);
            }
        }
        Look::Sport | Look::Line | Look::Apex | Look::Daylight => {}
    };
    div()
        .relative()
        .w(px(d))
        .h(px(d))
        .flex()
        .items_center()
        .justify_center()
        .child(layer(paint))
        .child(
            div().relative().child(
                Icon::symbol(icon)
                    .size(sz * 0.72)
                    .weight(if look == Look::Line { 1.2 } else { 2.0 })
                    .color(icon_col),
            ),
        )
        .into_any_element()
}

/// The speed-limit sign of diameter `d` in `look`.
pub(super) fn sign(limit: u32, d: f32, look: Look, t: &Theme) -> AnyElement {
    let ink = Ink::of(t);
    let red = ink.danger;
    let c = d / 2.0;
    let text = limit.to_string();
    let fs = d * if limit >= 100 { 0.3 } else { 0.38 };
    // The printed signs keep a light sheet with a dark figure in either mode, as the classic one.
    let paper = match look {
        Look::Vintage => mix(ink.text, ink.warn, 0.12),
        _ => ink.text,
    };
    let printed = matches!(look, Look::Chronograph | Look::Vintage);
    let paint = move |w: &mut Window, o: gpui::Point<Pixels>| match look {
        Look::Chronograph => {
            fill_circle(w, o, (c, c), c, scale_alpha(ink.text, 0.1));
            for i in 0..48 {
                let a = i as f32 / 48.0;
                stroke(
                    w,
                    o,
                    &[polar(c, c, c * 0.88, a), polar(c, c, c, a)],
                    d * 0.012,
                    scale_alpha(ink.text, 0.3),
                    false,
                );
            }
            fill_circle(w, o, (c, c), c * 0.86, red);
            fill_circle(w, o, (c, c), c * 0.64, paper);
        }
        Look::Vintage => {
            fill_circle(w, o, (c, c), c, scale_alpha(ink.text, 0.3));
            fill_circle(w, o, (c, c), c * 0.92, paper);
            fill_circle(w, o, (c, c), c * 0.84, red);
            fill_circle(w, o, (c, c), c * 0.62, paper);
            let mut glint = turn_arc(c, c, c * 0.88, 0.78, 0.98);
            glint.extend(turn_arc(c, c, c * 0.7, 0.98, 0.78));
            fill(w, o, &glint, scale_alpha(ink.text, 0.15));
        }
        Look::HeadsUp => {
            let oct = chamfered(d * 0.04, d * 0.04, d * 0.92, d * 0.92, d * 0.27);
            stroke(w, o, &oct, d * 0.07, red, true);
            let inner = chamfered(d * 0.16, d * 0.16, d * 0.68, d * 0.68, d * 0.2);
            stroke(w, o, &inner, d * 0.015, scale_alpha(red, 0.6), true);
        }
        Look::Segment => {
            let frame = chamfered(0.0, 0.0, d, d, d * 0.12);
            fill(w, o, &frame, ink.face);
            stroke(w, o, &frame, d * 0.06, red, true);
        }
        Look::Halo => {
            glow_arc(w, o, (c, c), c * 0.82, (0.0, 1.0), d * 0.05, (red, red));
            stroke_circle(w, o, (c, c), c * 0.62, d * 0.01, scale_alpha(red, 0.4));
        }
        Look::GlassCockpit => {
            let frame = rect(d * 0.04, d * 0.04, d * 0.92, d * 0.92);
            fill(w, o, &frame, ink.ground);
            fill_rect(w, o, (d * 0.04, d * 0.04, d * 0.92, d * 0.16), 0.0, red);
            stroke(w, o, &frame, d * 0.06, red, true);
        }
        Look::Race => {
            let frame = skew(d * 0.02, d * 0.16, d * 0.96, d * 0.68);
            fill(w, o, &frame, ink.ground);
            stroke(w, o, &frame, d * 0.06, red, true);
        }
        Look::Line => {
            stroke_circle(w, o, (c, c), c * 0.94, (d * 0.04).max(1.5), red);
        }
        Look::Sport | Look::Apex | Look::Daylight => {}
    };
    let figure: AnyElement = match look {
        Look::Segment => seg_text(&text, c, c - fs * 0.6, fs * 1.2, red).into_any_element(),
        Look::Race => div()
            .relative()
            .child(crate::gpui::race::figure(text, fs * 0.95, ink.text))
            .into_any_element(),
        _ => div()
            .relative()
            .text_size(px(fs))
            .text_color(if printed {
                hsla(t.palette.background)
            } else {
                ink.text
            })
            .font_weight(if matches!(look, Look::Halo | Look::Line) {
                FontWeight::LIGHT
            } else {
                FontWeight::BOLD
            })
            .child(text)
            .into_any_element(),
    };
    div()
        .relative()
        .w(px(d))
        .h(px(d))
        .flex()
        .items_center()
        .justify_center()
        .child(layer(paint))
        .child(figure)
        .into_any_element()
}
