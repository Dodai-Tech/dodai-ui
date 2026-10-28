//! [`SelectionTrack`]: the indicator that travels between the items of a row of choices (tabs,
//! segmented controls, custom pickers), animated the way a [`SelectionMotion`] says.
//!
//! A component puts [`SelectionTrack::container`] in its row, [`SelectionTrack::item`] in each
//! item and [`SelectionTrack::indicator`] in the row; the track measures where the items sit and
//! moves a [`SelectionMarker`] between them. All the motion comes from `dodai_core::motion` ([`Glide`],
//! [`Stretch`], [`Glitch`]) at the app's pace.
//!
//! [`Glide`]: crate::motion::Glide
//! [`Stretch`]: crate::motion::Stretch

use crate::hsla;
use crate::motion::{Glitch, Motion, SelectionMotion};
use crate::paint::{bottom_band_slide, shaped_id, ShapeStyle};
use dodai_core::{Shape, Theme};
use gpui::{div, px, AnyElement, App, ElementId, Hsla, IntoElement, ParentElement, Styled, Window};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// What the indicator looks like.
pub struct SelectionMarker(Look);

enum Look {
    Fill(Box<(Shape, ShapeStyle)>),
    Underline(Hsla),
}

impl SelectionMarker {
    /// A filled shape behind the chosen item (a pill, a block).
    pub fn fill(shape: Shape, style: ShapeStyle) -> Self {
        Self(Look::Fill(Box::new((shape, style))))
    }
    /// The design's underline band at the foot of the chosen item.
    pub fn underline(colour: Hsla) -> Self {
        Self(Look::Underline(colour))
    }
    fn colour(&self, t: &Theme) -> Hsla {
        match &self.0 {
            Look::Fill(f) => f.1.fill.unwrap_or_else(|| hsla(t.palette.primary)),
            Look::Underline(c) => *c,
        }
    }
}

/// Where the items sit, relative to the row (`x, y, w, h`), and which one the selection left.
#[derive(Default)]
struct Layout {
    origin: Cell<(f32, f32)>,
    items: RefCell<Vec<[f32; 4]>>,
    /// `(active, the one before it)`.
    switch: Cell<Option<(usize, usize)>>,
}

/// The travelling indicator of one row of choices. Cheap to clone; keyed by its id.
#[derive(Clone)]
pub struct SelectionTrack {
    id: ElementId,
    layout: Rc<Layout>,
}

impl SelectionTrack {
    /// The track for the row keyed `id` (state survives across frames).
    pub fn new(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> Self {
        let id = id.into();
        let layout = window
            .use_keyed_state((id.clone(), "selection"), cx, |_, _| {
                Rc::new(Layout::default())
            })
            .read(cx)
            .clone();
        Self { id, layout }
    }

    /// Goes in the row (it must be `relative`): records where the row sits.
    pub fn container(&self) -> impl IntoElement {
        let l = self.layout.clone();
        gpui::canvas(
            move |b, _, _| l.origin.set((b.origin.x.into(), b.origin.y.into())),
            |_, _, _, _| (),
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    /// Goes in item `i` (it must be `relative`): records where the item sits.
    pub fn item(&self, i: usize) -> impl IntoElement {
        let l = self.layout.clone();
        gpui::canvas(
            move |b, _, _| {
                let mut r = l.items.borrow_mut();
                if r.len() <= i {
                    r.resize(i + 1, [0.0; 4]);
                }
                r[i] = [
                    b.origin.x.into(),
                    b.origin.y.into(),
                    b.size.width.into(),
                    b.size.height.into(),
                ];
            },
            |_, _, _, _| (),
        )
        .absolute()
        .top_0()
        .left_0()
        .size_full()
    }

    /// Item `i` relative to the row, once measured.
    fn rect(&self, i: usize) -> Option<[f32; 4]> {
        let (ox, oy) = self.layout.origin.get();
        let r = self.layout.items.borrow().get(i).copied()?;
        (r[2] > 0.0).then_some([r[0] - ox, r[1] - oy, r[2], r[3]])
    }

    /// The item the selection left when it last moved to `active`.
    fn left(&self, active: usize) -> usize {
        let from = match self.layout.switch.get() {
            Some((now, from)) if now == active => from,
            Some((now, _)) => now,
            None => active,
        };
        self.layout.switch.set(Some((active, from)));
        from
    }

    /// The marker on item `active`, travelling there the `kind` way at `motion`'s pace (eased
    /// or on a spring, as its travel says). [`SelectionMotion::Lift`] has nothing travelling: the
    /// marker is simply on the chosen item. Until the items are measured it asks for another
    /// frame and draws nothing.
    pub fn indicator(
        &self,
        active: usize,
        kind: SelectionMotion,
        motion: Motion,
        marker: SelectionMarker,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let t = crate::theme_of(cx);
        let from = self.left(active);
        let Some([x, y, w, h]) = self.rect(active) else {
            crate::motion::next_frame(window);
            return div().into_any_element();
        };
        let colour = marker.colour(&t);
        let key = |tag: &'static str| ElementId::from((self.id.clone(), tag));
        let body = match kind {
            SelectionMotion::Glitch => {
                // Twice the design's duration: long enough to read as a glitch.
                let clip = || Glitch::clip(&motion.speed(0.5));
                let old = self.rect(from).map(|r| (r[0], r[2]));
                match crate::motion::play_on_change(window, cx, key("glitch"), active, clip) {
                    Some(g) => glitch(&t, &self.id, &marker, g, (x, w), old, colour),
                    None => draw(&t, &self.id, &marker, (x, w), colour, "marker"),
                }
            }
            SelectionMotion::Stretch => {
                let at = crate::motion::stretch(window, cx, key("span"), (x, w), motion);
                draw(&t, &self.id, &marker, at, colour, "marker")
            }
            SelectionMotion::Slide => {
                let x = crate::motion::smooth_with(window, cx, key("x"), x, motion);
                let w = crate::motion::smooth_with(window, cx, key("w"), w, motion);
                draw(&t, &self.id, &marker, (x, w), colour, "marker")
            }
            SelectionMotion::Lift => draw(&t, &self.id, &marker, (x, w), colour, "marker"),
        };
        // The marker's band: the chosen item's height, the row's width.
        div()
            .absolute()
            .left_0()
            .w_full()
            .top(px(y))
            .h(px(h))
            .child(body)
            .into_any_element()
    }
}

/// The marker at `(x, w)` in `colour`; `tag` keeps the copies of a glitch apart.
fn draw(
    t: &Theme,
    id: &ElementId,
    marker: &SelectionMarker,
    (x, w): (f32, f32),
    colour: Hsla,
    tag: &'static str,
) -> AnyElement {
    match &marker.0 {
        Look::Fill(f) => {
            let mut st = f.1;
            st.fill = Some(colour);
            shaped_id((id.clone(), tag), &f.0, st)
                .absolute()
                .top_0()
                .left(px(x))
                .w(px(w))
                .h_full()
                .into_any_element()
        }
        Look::Underline(_) => {
            bottom_band_slide(&t.shape, t.shape.border_width, colour, x, w).into_any_element()
        }
    }
}

/// One frame of a glitching marker: colour-split ghosts, a flickering trace of the item it left
/// and, for a fill, the new shape in three slices that jitter sideways and settle (each slice
/// clips the whole contour, so cut corners stay cut).
fn glitch(
    t: &Theme,
    id: &ElementId,
    marker: &SelectionMarker,
    g: Glitch,
    (x, w): (f32, f32),
    old: Option<(f32, f32)>,
    colour: Hsla,
) -> AnyElement {
    const AMP: f32 = 14.0;
    let mut layer = div().absolute().top_0().left_0().size_full();
    let ghosts = [
        (t.palette.info, -0.5, "ghost-a"),
        (t.palette.danger, 0.5, "ghost-b"),
    ];
    for (n, (c, side, tag)) in ghosts.into_iter().enumerate() {
        let at = (x + g.amplitude(AMP) * side, w);
        layer = layer.child(draw(
            t,
            id,
            marker,
            at,
            hsla(c).opacity(g.ghost(n as u32)),
            tag,
        ));
    }
    if let (Some(o), true) = (old, g.trace() > 0.0) {
        layer = layer.child(draw(t, id, marker, o, colour.opacity(g.trace()), "trace"));
    }
    let Look::Fill(f) = &marker.0 else {
        let at = (x + g.slice_dx(0, AMP), w);
        return layer
            .child(draw(t, id, marker, at, colour, "marker"))
            .into_any_element();
    };
    for i in 0..3u32 {
        if g.slice_hidden(i) {
            continue;
        }
        let tag: &'static str = ["slice-0", "slice-1", "slice-2"][i as usize];
        let mut st = f.1;
        st.fill = Some(colour);
        layer = layer.child(
            div()
                .absolute()
                .top(gpui::relative(i as f32 / 3.0))
                .h(gpui::relative(1.0 / 3.0))
                .left(px(x + g.slice_dx(i, AMP)))
                .w(px(w))
                .overflow_hidden()
                .child(
                    shaped_id((id.clone(), tag), &f.0, st)
                        .absolute()
                        .top(gpui::relative(-(i as f32)))
                        .left_0()
                        .w_full()
                        .h(gpui::relative(3.0)),
                ),
        );
    }
    layer.into_any_element()
}
