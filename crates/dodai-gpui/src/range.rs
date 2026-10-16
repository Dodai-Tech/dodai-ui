//! [`RangeSlider`]: two-thumb slider selecting `lo..=hi`.

use crate::controls::{clamp01, slider_fraction, slider_value};
use crate::paint::{round_shape, scaled_shape, shaped, ShapeStyle};
use crate::theme_of;
use dodai_core::ControlState;
use gpui::{
    canvas, div, px, relative, App, Bounds, ElementId, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Pixels, RenderOnce, Styled, Window,
};
use std::rc::Rc;

const THUMB: f32 = 16.0;

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

/// Horizontal two-thumb slider (controlled).
#[derive(IntoElement)]
pub struct RangeSlider {
    id: ElementId,
    lo: f32,
    hi: f32,
    min: f32,
    max: f32,
    step: Option<f32>,
    on_change: Option<RangeHandler>,
}

impl RangeSlider {
    pub fn new(id: impl Into<ElementId>, lo: f32, hi: f32) -> Self {
        Self {
            id: id.into(),
            lo,
            hi,
            min: 0.0,
            max: 1.0,
            step: None,
            on_change: None,
        }
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
    pub fn on_change(mut self, f: impl Fn(f32, f32, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for RangeSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = window.use_keyed_state(self.id.clone(), cx, |_, _| RangeState::default());
        let (min, max, step, lo, hi) = (self.min, self.max, self.step, self.lo, self.hi);
        let (fl, fh) = (slider_fraction(lo, min, max), slider_fraction(hi, min, max));
        let apply = {
            let st = st.clone();
            let f = self.on_change.clone();
            move |x: Pixels, pick: bool, w: &mut Window, cx: &mut App| {
                let b = st.read(cx).bounds;
                let v = slider_value(
                    (x - b.origin.x).into(),
                    0.0,
                    b.size.width.into(),
                    THUMB,
                    min,
                    max,
                    step,
                );
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
        let thumb_shape = round_shape(&t.shape, THUMB);
        let pk = t.progress(None);
        let track_shape = scaled_shape(&t.shape, 6.0);
        let track = shaped(
            &track_shape,
            ShapeStyle::from_tokens(&t, Some(pk.track), Some(pk.border), false, false),
        )
        .w_full()
        .h(px(6.0))
        .child(
            shaped(
                &track_shape,
                ShapeStyle::from_tokens(&t, Some(pk.fill), None, true, false),
            )
            .absolute()
            .top_0()
            .h_full()
            .left(relative(clamp01(fl)))
            .w(relative(clamp01(fh - fl))),
        );
        let thumb = |f: f32| {
            shaped(&thumb_shape, thumb_style)
                .absolute()
                .w(px(THUMB))
                .h(px(THUMB))
                .top(px(2.0))
                .left(relative(f))
                .ml(px(-THUMB / 2.0))
        };
        div()
            .id(self.id)
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(THUMB + 4.0))
            .cursor_pointer()
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, _, cx| rec.update(cx, |s, _| s.bounds = b),
                )
                .absolute()
                .size_full(),
            )
            .child(track)
            .child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(px(THUMB / 2.0))
                    .right(px(THUMB / 2.0))
                    .child(thumb(fl))
                    .child(thumb(fh)),
            )
            .on_mouse_down(MouseButton::Left, move |e, w, cx| {
                a1(e.position.x, true, w, cx)
            })
            .on_mouse_move(move |e, w, cx| {
                if mv.read(cx).active.is_some() {
                    a2(e.position.x, false, w, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |s, _| s.active = None)
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up2.update(cx, |s, _| s.active = None)
            })
    }
}

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
