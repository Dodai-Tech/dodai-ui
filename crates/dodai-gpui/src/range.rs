//! [`RangeSlider`]: two-thumb slider selecting `lo..=hi`.

use crate::controls::{clamp01, slider_fraction, slider_value, tick_fractions, Direction};
use crate::hsla;
use crate::paint::{round_shape, shaped, ShapeStyle};
use crate::theme_of;
use dodai_core::ControlState;
use gpui::{
    canvas, div, px, relative, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, Styled, Window,
};
use std::rc::Rc;

/// Which thumb (`0` = low, `1` = high) is closest to `v`; ties go to the
/// side that can still move toward `v`.
pub fn nearest_thumb(lo: f32, hi: f32, v: f32) -> usize {
    let (dl, dh) = ((v - lo).abs(), (v - hi).abs());
    if dl < dh || (dl == dh && v < lo) {
        0
    } else {
        1
    }
}

/// Move `thumb` to `v`, keeping `lo <= hi`.
pub fn range_update(thumb: usize, v: f32, lo: f32, hi: f32) -> (f32, f32) {
    if thumb == 0 {
        (v.min(hi), hi)
    } else {
        (lo, v.max(lo))
    }
}

#[derive(Default)]
struct RangeState {
    bounds: Bounds<Pixels>,
    active: Option<usize>,
}

type RangeHandler = Rc<dyn Fn(f32, f32, &mut Window, &mut App)>;

/// Two-thumb slider (controlled), horizontal or vertical.
#[derive(IntoElement)]
pub struct RangeSlider {
    id: ElementId,
    direction: Direction,
    length: f32,
    lo: f32,
    hi: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    ticks: bool,
    show_value: bool,
    on_change: Option<RangeHandler>,
    corners: Option<dodai_core::Corners>,
}

crate::corners_setter!(RangeSlider);

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, lo: f32, hi: f32) -> Self {
        Self {
            corners: None,
            id: id.into(),
            direction: Direction::Horizontal,
            length: 160.0,
            lo,
            hi,
            min: 0.0,
            max: 1.0,
            step: None,
            ticks: false,
            show_value: false,
            on_change: None,
        }
    }
    /// Horizontal (default) or vertical (fixed [`RangeSlider::length`]).
    pub fn direction(mut self, d: Direction) -> Self {
        self.direction = d;
        self
    }
    /// Height in px of a vertical slider (default 160).
    pub fn length(mut self, px: f32) -> Self {
        self.length = px;
        self
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, s: f32) -> Self {
        self.step = Some(s);
        self
    }
    /// Tick mark at every `step`.
    pub fn ticks(mut self, on: bool) -> Self {
        self.ticks = on;
        self
    }
    /// Show `lo – hi` readout.
    pub fn show_value(mut self, on: bool) -> Self {
        self.show_value = on;
        self
    }
    crate::callback!(on_change, f32, f32);
}

impl RangeSlider {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let t = crate::apply_corners(t, self.corners);
        let thumb_size = t.sizes().slider_thumb;
        let vertical = self.direction == Direction::Vertical;
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| RangeState::default());
        let (min, max, step, lo, hi) = (self.min, self.max, self.step, self.lo, self.hi);
        let grow = crate::motion::grow(window, cx, "RangeSlider");
        let (fl, fh) = (
            slider_fraction(lo, min, max) * grow,
            slider_fraction(hi, min, max) * grow,
        );
        let apply = {
            let st = st.clone();
            let f = self.on_change.clone();
            move |p: gpui::Point<Pixels>, pick: bool, w: &mut Window, cx: &mut App| {
                let b = st.read(cx).bounds;
                let (pos, len): (f32, f32) = if vertical {
                    (
                        (b.origin.y + b.size.height - p.y).into(),
                        b.size.height.into(),
                    )
                } else {
                    ((p.x - b.origin.x).into(), b.size.width.into())
                };
                let v = slider_value(pos, 0.0, len, thumb_size, min, max, step);
                if pick {
                    st.update(cx, |s, _| s.active = Some(nearest_thumb(lo, hi, v)));
                }
                if let (Some(f), Some(a)) = (&f, st.read(cx).active) {
                    let (l, h) = range_update(a, v, lo, hi);
                    f(l, h, w, cx);
                }
            }
        };
        let (a1, a2) = (apply.clone(), apply);
        let rec = st.clone();
        let (mv, up, up2) = (st.clone(), st.clone(), st.clone());
        let k = t.toggle(true, ControlState::Idle);
        let thumb_style = ShapeStyle::from_tokens(&t, Some(k.track), Some(k.border), true, false);
        let thumb_shape = round_shape(&t.shape, thumb_size);
        let pk = t.progress(None);
        let sz = t.sizes();
        let tick_off = (thumb_size + t.spacing.xs - sz.tick_len) / 2.0;
        let track_shape = crate::paint::round_shape(&t.shape, sz.slider_track);
        let fill_style = ShapeStyle::from_tokens(&t, Some(pk.fill), None, true, false);
        let fill = shaped(&track_shape, fill_style).absolute();
        let fill = if vertical {
            fill.left_0()
                .w_full()
                .bottom(relative(clamp01(fl)))
                .h(relative(clamp01(fh - fl)))
        } else {
            fill.top_0()
                .h_full()
                .left(relative(clamp01(fl)))
                .w(relative(clamp01(fh - fl)))
        };
        let track = shaped(
            &track_shape,
            ShapeStyle::from_tokens(&t, Some(pk.track), Some(pk.border), false, false),
        );
        let track = if vertical {
            track.h_full().w(px(sz.slider_track))
        } else {
            track.w_full().h(px(sz.slider_track))
        }
        .child(fill);
        let thumb = |f: f32| {
            let th = shaped(&thumb_shape, thumb_style)
                .absolute()
                .w(px(thumb_size))
                .h(px(thumb_size));
            if vertical {
                th.left(px(t.spacing.xs / 2.0))
                    .top(relative(1.0 - f))
                    .mt(px(-thumb_size / 2.0))
            } else {
                th.top(px(t.spacing.xs / 2.0))
                    .left(relative(f))
                    .ml(px(-thumb_size / 2.0))
            }
        };
        let tick_c = hsla(t.palette.border_strong);
        let mut rail = div().absolute().top_0().bottom_0().left_0().right_0();
        rail = if vertical {
            rail.top(px(thumb_size / 2.0)).bottom(px(thumb_size / 2.0))
        } else {
            rail.left(px(thumb_size / 2.0)).right(px(thumb_size / 2.0))
        };
        if self.ticks {
            for f in tick_fractions(min, max, step) {
                let tk = div().absolute().bg(tick_c);
                rail = rail.child(if vertical {
                    tk.left(px(tick_off))
                        .w(px(sz.tick_len))
                        .h(px(sz.tick_width))
                        .top(relative(1.0 - f))
                } else {
                    tk.top(px(tick_off))
                        .h(px(sz.tick_len))
                        .w(px(sz.tick_width))
                        .left(relative(f))
                        .ml(px(-sz.tick_width / 2.0))
                });
            }
        }
        rail = rail.child(thumb(fl)).child(thumb(fh));
        let base = div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .cursor_pointer();
        let base = if vertical {
            base.w(px(thumb_size + 4.0))
                .h(px(self.length))
                .justify_center()
        } else {
            base.w_full().h(px(thumb_size + 4.0))
        };
        let slider = base
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, _, cx| rec.update(cx, |s, _| s.bounds = b),
                )
                .absolute()
                .size_full(),
            )
            .child(track)
            .child(rail)
            .on_mouse_down(MouseButton::Left, move |e, w, cx| {
                a1(e.position, true, w, cx)
            })
            .on_mouse_move(move |e, w, cx| {
                if mv.read(cx).active.is_some() {
                    a2(e.position, false, w, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |s, _| s.active = None)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up2.update(cx, |s, _| s.active = None)
            });
        if !self.show_value {
            return slider.into_any_element();
        }
        let r = div()
            .font_family(crate::text::first_family(t.typography.mono))
            .text_size(px(t.typography.size_xs))
            .text_color(hsla(t.palette.text_muted))
            .child(format!(
                "{} – {}",
                dodai_charts_core::fmt_num((lo * 100.0).round() / 100.0),
                dodai_charts_core::fmt_num((hi * 100.0).round() / 100.0)
            ));
        if vertical {
            div()
                .flex()
                .flex_col()
                .items_center()
                .gap(px(t.spacing.xs))
                .child(r)
                .child(slider)
                .into_any_element()
        } else {
            div()
                .flex()
                .items_center()
                .gap(px(t.spacing.sm))
                .child(slider.flex_1())
                .child(r)
                .into_any_element()
        }
    }
}

crate::entry_render!(RangeSlider);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_nearest_thumb() {
        assert_eq!(nearest_thumb(0.2, 0.8, 0.1), 0);
        assert_eq!(nearest_thumb(0.2, 0.8, 0.9), 1);
        assert_eq!(nearest_thumb(0.5, 0.5, 0.2), 0);
        assert_eq!(nearest_thumb(0.5, 0.5, 0.9), 1);
    }

    #[test]
    fn thumbs_cannot_cross() {
        assert_eq!(range_update(0, 0.9, 0.2, 0.6), (0.6, 0.6));
        assert_eq!(range_update(1, 0.1, 0.2, 0.6), (0.2, 0.2));
        assert_eq!(range_update(0, 0.1, 0.2, 0.6), (0.1, 0.6));
    }
}
