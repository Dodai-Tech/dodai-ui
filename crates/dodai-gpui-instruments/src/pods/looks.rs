//! The card and strip looks of the pods: the same readings as the round pods, laid out in a
//! rounded rectangle (top to bottom) or a wide low strip (left to right). Colours from the theme.

use super::*;

/// A line of text.
fn text(s: impl Into<SharedString>, size: f32, k: f32, col: Hsla) -> gpui::Div {
    div()
        .text_size(px(size * k))
        .text_color(col)
        .whitespace_nowrap()
        .child(s.into())
}

/// A horizontal track with a fill from `from` to `to` (fractions of the width).
fn hbar(size: f32, w: f32, from: f32, to: f32, col: Hsla, track: Hsla) -> gpui::Div {
    let (a, b) = (from.min(to), from.max(to));
    div()
        .relative()
        .w(px(w))
        .h(px(size * 0.03))
        .rounded(px(size * 0.015))
        .bg(track)
        .child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .left(gpui::relative(a))
                .w(gpui::relative(b - a))
                .rounded(px(size * 0.015))
                .bg(col),
        )
}

impl TyrePod {
    pub(crate) fn rect(self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let track = scale_alpha(sk.text, 0.14);
        let col = |bar: Option<f32>| {
            if pm::is_low(bar, self.low) {
                sk.warn
            } else if bar.is_some_and(f32::is_finite) {
                sk.text
            } else {
                sk.muted
            }
        };
        let names = ["FL", "FR", "RL", "RR"];
        if self.look == PodLook::Strip {
            let mut row = sk
                .strip(s)
                .child(text(self.heading.clone(), s, 0.06, sk.muted));
            for (name, bar) in names.iter().zip(self.bars) {
                row = row.child(
                    div()
                        .flex()
                        .flex_col()
                        .items_center()
                        .child(text(*name, s, 0.05, sk.muted))
                        .child(text(pm::pressure_text(bar), s, 0.08, col(bar))),
                );
            }
            return row
                .child(text(self.unit.clone(), s, 0.05, sk.muted))
                .into_any_element();
        }
        let col_h = s * 0.34;
        let mut columns = div().flex().justify_between().items_end();
        for (name, bar) in names.iter().zip(self.bars) {
            let frac = pm::unit(pm::finite(bar.unwrap_or(0.0), 0.0) / 4.0);
            columns = columns.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.02))
                    .child(text(*name, s, 0.05, sk.muted))
                    .child(
                        div()
                            .relative()
                            .w(px(s * 0.09))
                            .h(px(col_h))
                            .rounded(px(s * 0.03))
                            .overflow_hidden()
                            .bg(track)
                            .child(
                                div()
                                    .absolute()
                                    .left_0()
                                    .bottom_0()
                                    .w_full()
                                    .h(gpui::relative(frac))
                                    .bg(col(bar)),
                            ),
                    )
                    .child(text(pm::pressure_text(bar), s, 0.07, col(bar))),
            );
        }
        sk.card(s)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(
                        text(self.heading.clone(), s, 0.06, sk.text)
                            .font_weight(FontWeight::MEDIUM),
                    )
                    .child(text(self.unit.clone(), s, 0.05, sk.muted)),
            )
            .child(columns)
            .when(!self.note.is_empty(), |d| {
                d.child(text(self.note.clone(), s, 0.05, sk.muted))
            })
            .into_any_element()
    }
}

impl RowsPod {
    pub(crate) fn rect(self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let track = scale_alpha(sk.text, 0.14);
        if self.look == PodLook::Strip {
            let mut row = sk.strip(s).justify_between();
            for r in &self.rows {
                let mut cell = div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(s * 0.02))
                    .child(r.icon.element(s * 0.08, sk.muted))
                    .child(text(r.value.clone(), s, 0.06, sk.text));
                if let Some((f, tone)) = r.bar {
                    cell = cell.child(hbar(s, s * 0.22, 0.0, f, tone.color(t), track));
                }
                row = row.child(cell);
            }
            return row.into_any_element();
        }
        let mut card = sk.card(s).gap(px(s * 0.02));
        if !self.heading.is_empty() {
            card = card.child(
                text(self.heading.clone(), s, 0.06, sk.text).font_weight(FontWeight::MEDIUM),
            );
        }
        let last = self.rows.len().saturating_sub(1);
        for (i, r) in self.rows.iter().enumerate() {
            let mut line = div()
                .flex()
                .items_center()
                .gap(px(s * 0.04))
                .py(px(s * 0.025))
                .when(i != last, |d| {
                    d.border_b_1().border_color(scale_alpha(sk.line, 0.7))
                })
                .child(r.icon.element(s * 0.07, sk.muted))
                .child(
                    div()
                        .flex_1()
                        .child(text(r.value.clone(), s, 0.06, sk.text)),
                );
            if let Some((f, tone)) = r.bar {
                line = line.child(hbar(s, s * 0.3, 0.0, f, tone.color(t), track));
            }
            card = card.child(line);
        }
        card.into_any_element()
    }
}

impl NoticePod {
    pub(crate) fn rect(self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let tone = if self.tone == Tone::Off {
            sk.info
        } else {
            self.tone.color(t)
        };
        let icon = self.icon.element(s * 0.13, tone);
        if self.look == PodLook::Strip {
            return sk
                .strip(s)
                .child(icon)
                .child(text(self.title.clone(), s, 0.07, sk.text).font_weight(FontWeight::MEDIUM))
                .child(text(self.text.clone(), s, 0.06, sk.muted))
                .into_any_element();
        }
        div()
            .flex()
            .w(px(s))
            .rounded(px(s * 0.07))
            .border_1()
            .border_color(sk.line)
            .bg(sk.face)
            .overflow_hidden()
            .child(
                div()
                    .w(px(s * 0.022))
                    .flex_none()
                    .my(px(s * 0.05))
                    .ml(px(s * 0.04))
                    .rounded_full()
                    .bg(tone),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(s * 0.05))
                    .p(px(s * 0.06))
                    .child(icon)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(s * 0.015))
                            .child(
                                text(self.title.clone(), s, 0.07, sk.text)
                                    .font_weight(FontWeight::MEDIUM),
                            )
                            .child(text(self.text.clone(), s, 0.055, sk.muted)),
                    ),
            )
            .into_any_element()
    }
}

impl GForcePod {
    pub(crate) fn rect(self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let track = scale_alpha(sk.text, 0.14);
        let ox = pm::g_offset(self.lateral, self.limit);
        let oy = pm::g_offset(self.longitudinal, self.limit);
        if self.look == PodLook::Strip {
            let row = |name: &str, off: f32, g: f32| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(s * 0.04))
                    .child(
                        div()
                            .w(px(s * 0.3))
                            .child(text(name.to_string(), s, 0.055, sk.muted)),
                    )
                    .child(hbar(s, s * 0.6, 0.5, 0.5 + off * 0.5, sk.primary, track))
                    .child(text(pm::g_text(g), s, 0.065, sk.text))
            };
            return sk
                .strip(s)
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(s * 0.025))
                        .child(row("Sideways", ox, self.lateral))
                        .child(row("Forward", oy, self.longitudinal)),
                )
                .into_any_element();
        }
        let b = s * 0.6;
        let d = s * 0.07;
        let line = scale_alpha(sk.text, 0.25);
        let plot = div()
            .relative()
            .w(px(b))
            .h(px(b))
            .rounded(px(s * 0.04))
            .border_1()
            .border_color(sk.line)
            .bg(scale_alpha(sk.text, 0.05))
            .child(
                div()
                    .absolute()
                    .left(px(b / 2.0))
                    .top_0()
                    .w(px(1.0))
                    .h_full()
                    .bg(line),
            )
            .child(
                div()
                    .absolute()
                    .top(px(b / 2.0))
                    .left_0()
                    .h(px(1.0))
                    .w_full()
                    .bg(line),
            )
            .child(
                div()
                    .absolute()
                    .left(px(b / 2.0 * (1.0 + ox) - d / 2.0))
                    .top(px(b / 2.0 * (1.0 - oy) - d / 2.0))
                    .size(px(d))
                    .rounded_full()
                    .bg(sk.primary),
            );
        sk.card(s)
            .items_center()
            .child(text("G-force", s, 0.06, sk.text).font_weight(FontWeight::MEDIUM))
            .child(plot)
            .child(
                div()
                    .flex()
                    .gap(px(s * 0.06))
                    .child(text(
                        format!("← → {}", pm::g_text(self.lateral)),
                        s,
                        0.055,
                        sk.muted,
                    ))
                    .child(text(
                        format!("↑ ↓ {}", pm::g_text(self.longitudinal)),
                        s,
                        0.055,
                        sk.muted,
                    )),
            )
            .into_any_element()
    }
}

impl MediaPod {
    pub(crate) fn rect(mut self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let track = scale_alpha(sk.text, 0.14);
        let side = if self.look == PodLook::Strip {
            s * 0.16
        } else {
            s * 0.3
        };
        let cover = self.cover.take().unwrap_or_else(|| {
            div()
                .flex()
                .items_center()
                .justify_center()
                .size_full()
                .child(Icon::new(IconId::Music).size(side * 0.5).color(sk.info))
                .into_any_element()
        });
        let cover = div()
            .flex_none()
            .size(px(side))
            .rounded(px(s * 0.03))
            .border_1()
            .border_color(sk.line)
            .overflow_hidden()
            .child(cover);
        if self.look == PodLook::Strip {
            return sk
                .strip(s)
                .child(cover)
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            text(self.title.clone(), s, 0.065, sk.info)
                                .font_weight(FontWeight::MEDIUM),
                        )
                        .child(text(self.artist.clone(), s, 0.05, sk.muted)),
                )
                .child(text(self.time.clone(), s, 0.05, sk.muted))
                .into_any_element();
        }
        sk.card(s)
            .child(
                div().flex().gap(px(s * 0.05)).child(cover).child(
                    div()
                        .flex()
                        .flex_col()
                        .justify_center()
                        .child(
                            text(self.title.clone(), s, 0.07, sk.info)
                                .font_weight(FontWeight::MEDIUM),
                        )
                        .child(text(self.artist.clone(), s, 0.055, sk.muted)),
                ),
            )
            .child(hbar(s, s * 0.88, 0.0, 0.4, sk.info, track))
            .child(text(self.time.clone(), s, 0.05, sk.muted))
            .into_any_element()
    }
}

impl CompassPod {
    pub(crate) fn rect(self, t: &dodai_core::Theme) -> AnyElement {
        let (sk, s) = (Skin::of(t), self.size);
        let h = self.heading;
        let name = pm::cardinal(h);
        if self.look == PodLook::Strip {
            let w = PodLook::Strip.width(s);
            let ppd = s * 0.016;
            let base = (h / 5.0).floor() * 5.0;
            let mut tape = div()
                .relative()
                .w(px(w))
                .h(px(s * 0.4))
                .rounded(px(s * 0.09))
                .border_1()
                .border_color(sk.line)
                .bg(sk.face)
                .overflow_hidden();
            for k in -16..=16 {
                let a = base + k as f32 * 5.0;
                let x = w / 2.0 + (a - h) * ppd;
                let deg = a.rem_euclid(360.0);
                let major = (deg % 45.0).abs() < 0.1;
                let mid = (deg % 15.0).abs() < 0.1;
                let tall = if major {
                    0.1
                } else if mid {
                    0.06
                } else {
                    0.03
                };
                tape = tape.child(
                    div()
                        .absolute()
                        .left(px(x))
                        .bottom(px(s * 0.03))
                        .w(px(1.0))
                        .h(px(s * tall))
                        .bg(scale_alpha(sk.text, if major { 0.9 } else { 0.4 })),
                );
                if major {
                    let idx = ((deg + 22.5) / 45.0) as usize % 8;
                    tape = tape.child(
                        div()
                            .absolute()
                            .left(px(x - s * 0.06))
                            .bottom(px(s * 0.12))
                            .w(px(s * 0.12))
                            .flex()
                            .justify_center()
                            .text_size(px(s * 0.06))
                            .text_color(sk.text)
                            .child(["N", "NE", "E", "SE", "S", "SW", "W", "NW"][idx]),
                    );
                }
            }
            return tape
                .child(
                    div()
                        .absolute()
                        .left(px(w / 2.0 - 1.0))
                        .top_0()
                        .w(px(2.0))
                        .h(px(s * 0.1))
                        .bg(sk.primary),
                )
                .child(
                    div()
                        .absolute()
                        .left(px(w / 2.0 + s * 0.03))
                        .top(px(s * 0.01))
                        .text_size(px(s * 0.06))
                        .text_color(sk.text)
                        .child(format!("{h:.0}°")),
                )
                .into_any_element();
        }
        let idx = ((h + 22.5) / 45.0) as usize % 8;
        let mut points = div().flex().justify_between();
        for (i, n) in ["N", "NE", "E", "SE", "S", "SW", "W", "NW"]
            .iter()
            .enumerate()
        {
            points = points.child(text(*n, s, 0.05, if i == idx { sk.info } else { sk.muted }));
        }
        sk.card(s)
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(s * 0.03))
                    .child(
                        text(format!("{h:.0}°"), s, 0.2, sk.text).font_weight(FontWeight::MEDIUM),
                    )
                    .child(text(name, s, 0.11, sk.info)),
            )
            .child(points)
            .when(!self.height.is_empty(), |d| {
                d.child(text(self.height.clone(), s, 0.055, sk.muted))
            })
            .into_any_element()
    }
}
