//! The telltale lamps and the speed-limit sign in every [`Look`] but the classic one: bezels,
//! lamp lenses, chamfered frames, LCD blocks, halos, annunciator capsules and neon bulbs. Colours
//! from the theme.

use super::*;
use crate::frames::{aged, ghost, glow_arc, glow_dot, seg_text};
use crate::mfd::rect;
use crate::shapes::{chamfered, circle, Ink};
use crate::tubes::{bulb, glow, tubes};
use dodai_gpui::motion::mix;

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
        Look::Digital if lit => ink.face,
        Look::Digital => ghost(ink.primary, dark),
        // A lit lens glows in the tone; the symbol on it reads in the text colour.
        Look::Vintage if lit => ink.text,
        // A lit bulb glows in the tone; the symbol sits dark on it.
        Look::Nixie if lit => ink.face,
        _ => col,
    };
    let paint = move |w: &mut Window, o: gpui::Point<Pixels>| match look {
        Look::Chrono => {
            fill(w, o, &circle(c, c, c * 0.96), scale_alpha(ink.text, 0.06));
            stroke(w, o, &circle(c, c, c * 0.96), d * 0.04, ink.line, true);
            stroke(
                w,
                o,
                &circle(c, c, c * 0.8),
                d * 0.015,
                scale_alpha(ink.text, 0.2),
                true,
            );
        }
        Look::Vintage => {
            fill(w, o, &circle(c, c, c), scale_alpha(ink.text, 0.25));
            fill(w, o, &circle(c, c, c * 0.82), ink.line);
            if lit {
                for k in 0..5 {
                    fill(
                        w,
                        o,
                        &circle(c, c, c * 0.82 * (1.0 - k as f32 * 0.15)),
                        scale_alpha(col, 0.2),
                    );
                }
            } else {
                fill(w, o, &circle(c, c, c * 0.82), scale_alpha(aged(ink), 0.5));
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
        Look::Digital => {
            let block = chamfered(d * 0.04, d * 0.04, d * 0.92, d * 0.92, d * 0.08);
            fill(
                w,
                o,
                &block,
                if lit { col } else { ghost(ink.primary, dark) },
            );
        }
        Look::Futuristic => {
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
            stroke(
                w,
                o,
                &circle(c, c, c * 0.92),
                d * 0.02,
                scale_alpha(col, if lit { 0.6 } else { 0.25 }),
                true,
            );
        }
        Look::Cockpit => {
            let frame = rect(d * 0.04, d * 0.14, d * 0.92, d * 0.72);
            fill(w, o, &frame, ink.face);
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
        Look::Nixie => bulb(w, o, (c, c), c * 0.92, col, lit, ink),
        Look::Classic => {}
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
                    .weight(2.0)
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
    let printed = matches!(look, Look::Chrono | Look::Vintage);
    let paint = move |w: &mut Window, o: gpui::Point<Pixels>| match look {
        Look::Chrono => {
            fill(w, o, &circle(c, c, c), scale_alpha(ink.text, 0.1));
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
            fill(w, o, &circle(c, c, c * 0.86), red);
            fill(w, o, &circle(c, c, c * 0.64), paper);
        }
        Look::Vintage => {
            fill(w, o, &circle(c, c, c), scale_alpha(ink.text, 0.3));
            fill(w, o, &circle(c, c, c * 0.92), paper);
            fill(w, o, &circle(c, c, c * 0.84), red);
            fill(w, o, &circle(c, c, c * 0.62), paper);
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
        Look::Digital => {
            let frame = chamfered(0.0, 0.0, d, d, d * 0.12);
            fill(w, o, &frame, ink.face);
            stroke(w, o, &frame, d * 0.06, red, true);
        }
        Look::Futuristic => {
            glow_arc(w, o, (c, c), c * 0.82, (0.0, 1.0), d * 0.05, (red, red));
            stroke(
                w,
                o,
                &circle(c, c, c * 0.62),
                d * 0.01,
                scale_alpha(red, 0.4),
                true,
            );
        }
        Look::Cockpit => {
            let frame = rect(d * 0.04, d * 0.04, d * 0.92, d * 0.92);
            fill(w, o, &frame, ink.face);
            fill(w, o, &rect(d * 0.04, d * 0.04, d * 0.92, d * 0.16), red);
            stroke(w, o, &frame, d * 0.06, red, true);
        }
        Look::Nixie => {
            fill(w, o, &circle(c, c, c), ink.face);
            glow_arc(w, o, (c, c), c * 0.86, (0.0, 1.0), d * 0.03, (red, red));
        }
        Look::Classic => {}
    };
    let figure: AnyElement = match look {
        Look::Digital => seg_text(&text, c, c - fs * 0.6, fs * 1.2, red).into_any_element(),
        Look::Nixie => div()
            .relative()
            .child(tubes(&text, fs * 1.25, glow(ink), ink))
            .into_any_element(),
        _ => div()
            .relative()
            .text_size(px(fs))
            .text_color(if printed {
                hsla(t.palette.background)
            } else {
                ink.text
            })
            .font_weight(if look == Look::Futuristic {
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
