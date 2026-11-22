//! Display components: [`CircularProgress`], [`Banner`], [`ScrollArea`],
//! [`Image`] and [`AvatarGroup`].

use crate::data::Avatar;
use crate::feedback::alert_style;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_ui_core::Tone;
use gpui::{
    canvas, div, img, point, px, AnyElement, App, Axis, ElementId, InteractiveElement, IntoElement,
    MouseButton, ObjectFit, ParentElement, PathBuilder, RenderOnce, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, StyledImage, Window,
};
use std::path::PathBuf;
use std::rc::Rc;

// ───────────────────────────── circular progress ─────────────────────────────

/// Points of an arc from `start` rad sweeping `sweep` rad, radius `r` around `(cx, cy)`.
pub fn arc_points(cx: f32, cy: f32, r: f32, start: f32, sweep: f32) -> Vec<(f32, f32)> {
    let n = ((sweep.abs() / std::f32::consts::TAU) * 96.0)
        .ceil()
        .max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = start + sweep * i as f32 / n as f32;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// Like [`arc_points`] but the radius oscillates by `amp` with `waves` crests over the full
/// circle (Material 3 Expressive wavy circular indicator). `phase` travels the wave.
pub fn wavy_arc_points(
    c: (f32, f32),
    r: f32,
    start: f32,
    sweep: f32,
    amp: f32,
    waves: f32,
    phase: f32,
) -> Vec<(f32, f32)> {
    let n = ((sweep.abs() / std::f32::consts::TAU) * 192.0)
        .ceil()
        .max(2.0) as usize;
    (0..=n)
        .map(|i| {
            let a = start + sweep * i as f32 / n as f32;
            let rr = r + amp * (a * waves - phase).sin();
            (c.0 + rr * a.cos(), c.1 + rr * a.sin())
        })
        .collect()
}

pub(crate) fn elapsed_secs() -> f32 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs_f32()
}

/// Ring progress. `new(0.0..=1.0)` is determinate; [`CircularProgress::indeterminate`] spins.
#[derive(IntoElement)]
pub struct CircularProgress {
    value: Option<f32>,
    size: Option<f32>,
    thickness: Option<f32>,
    tone: Option<Tone>,
    label: bool,
    wavy: bool,
}

impl CircularProgress {
    /// Create a new [`CircularProgress`].
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            size: None,
            thickness: None,
            tone: None,
            label: true,
            wavy: false,
        }
    }
    /// Show the indeterminate state.
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            label: false,
            ..Self::new(0.0)
        }
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = Some(s);
        self
    }
    /// Stroke thickness in px.
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = Some(t);
        self
    }
    /// Semantic tone (colour).
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
        self
    }
    /// Material 3 Expressive wavy ring. Ignored by other design systems.
    pub fn wavy(mut self, on: bool) -> Self {
        self.wavy = on;
        self
    }
    /// Show the percentage in the middle (determinate only; default on).
    pub fn label(mut self, on: bool) -> Self {
        self.label = on;
        self
    }
}

impl RenderOnce for CircularProgress {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let k = t.progress(self.tone);
        let (track, fill) = (hsla(k.track), hsla(k.fill));
        // M3: 48 px indicator with a 4 px stroke, and a gap between the active arc and the
        // track; other design systems keep their 56 / 5 px ring.
        let m3 = t.look().tonal;
        let (size, th) = if m3 { (48.0, 4.0) } else { (56.0, 5.0) };
        let size = self.size.unwrap_or(size);
        let th = self.thickness.unwrap_or(th);
        let value = self.value;
        let wavy = self.wavy && m3;
        let mut d = div()
            .relative()
            .flex_none()
            .w(px(size))
            .h(px(size))
            .flex()
            .items_center()
            .justify_center()
            .font_family(first_family(t.typography.display))
            .text_size(px(size * 0.26))
            .text_color(hsla(t.palette.text))
            .child(
                canvas(
                    |_, _, _| (),
                    move |b, _, window, _| {
                        let (o, r) = (b.origin, size / 2.0 - th / 2.0);
                        let pt = |(x, y): (f32, f32)| {
                            point(o.x + px(x + th / 2.0), o.y + px(y + th / 2.0))
                        };
                        let mut ring = |pts: Vec<(f32, f32)>, c, closed: bool| {
                            let mut pb = PathBuilder::stroke(px(th));
                            let v: Vec<_> = pts.into_iter().map(pt).collect();
                            pb.add_polygon(&v, closed);
                            if let Ok(p) = pb.build() {
                                window.paint_path(p, c);
                            }
                        };
                        let full = std::f32::consts::TAU;
                        let (c0, c1) = (size / 2.0 - th / 2.0, size / 2.0 - th / 2.0);
                        let start = -std::f32::consts::FRAC_PI_2;
                        match value {
                            Some(v) if m3 && v > 0.0 && v < 1.0 => {
                                // Track resumes one stroke-width past the active arc's end.
                                let gap = (th * 1.5 / r).min(0.5);
                                let a = full * v;
                                ring(
                                    arc_points(
                                        c0,
                                        c1,
                                        r,
                                        start + a + gap,
                                        (full - a - 2.0 * gap).max(0.0),
                                    ),
                                    track,
                                    false,
                                );
                            }
                            _ => ring(arc_points(c0, c1, r, 0.0, full), track, true),
                        }
                        match value {
                            Some(v) if v > 0.0 => {
                                let start = -std::f32::consts::FRAC_PI_2;
                                let sweep = full * v.min(0.9999);
                                let pts = if wavy {
                                    let ph = elapsed_secs() * 3.0;
                                    wavy_arc_points((c0, c1), r - 1.5, start, sweep, 1.2, 7.0, ph)
                                } else {
                                    arc_points(c0, c1, r, start, sweep)
                                };
                                ring(pts, fill, false)
                            }
                            Some(_) => {}
                            None => {
                                let a = elapsed_secs() * 4.0;
                                let pts = if wavy {
                                    wavy_arc_points(
                                        (c0, c1),
                                        r - 1.5,
                                        a,
                                        full * 0.4,
                                        1.2,
                                        7.0,
                                        a * 2.0,
                                    )
                                } else {
                                    arc_points(c0, c1, r, a, full * 0.27)
                                };
                                ring(pts, fill, false);
                                crate::motion::next_frame(window);
                            }
                        }
                        if wavy && value.is_some_and(|v| v > 0.0) {
                            crate::motion::next_frame(window);
                        }
                    },
                )
                .absolute()
                .size_full(),
            );
        if let (true, Some(v)) = (self.label, value) {
            d = d.child(format!("{}%", (v * 100.0).round() as u32));
        }
        d
    }
}

// ───────────────────────────── banner ─────────────────────────────

type VoidFn = Rc<dyn Fn(&mut Window, &mut App)>;

/// Full-width message strip with an optional action and a dismiss ✕.
#[derive(IntoElement)]
pub struct Banner {
    id: ElementId,
    tone: Tone,
    message: SharedString,
    action: Option<SharedString>,
    on_action: Option<VoidFn>,
    on_dismiss: Option<VoidFn>,
    dismissible: bool,
    corners: Option<dodai_ui_core::Corners>,
    surface: dodai_ui_core::Surface,
}

crate::corners_setter!(Banner);
crate::surface_setter!(Banner);

impl Banner {
    /// Create a new [`Banner`].
    pub fn new(id: impl Into<ElementId>, tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            tone,
            message: message.into(),
            action: None,
            on_action: None,
            on_dismiss: None,
            dismissible: true,
            corners: None,
            surface: dodai_ui_core::Surface::INHERIT,
        }
    }
    /// Text button on the right, e.g. "Undo".
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some(label.into());
        self.on_action = Some(Rc::new(f));
        self
    }
    /// Show a dismiss button.
    pub fn dismissible(mut self, on: bool) -> Self {
        self.dismissible = on;
        self
    }
    crate::callback!(
        /// Called when the surface is dismissed.
        on_dismiss
    );
}

impl RenderOnce for Banner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = crate::apply_corners(theme_of(cx), self.corners);
        let gone = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        if *gone.read(cx) {
            return div().into_any_element();
        }
        let (style, fg) = alert_style(&t, self.tone, true);
        let style = style.with_own_surface(&t, dodai_ui_core::SurfaceClass::Bar, &self.surface);
        let mut row = shaped(&t.shape, style)
            .w_full()
            .px(px(t.spacing.md))
            .py(px(t.spacing.sm))
            .flex()
            .items_center()
            .gap(px(t.spacing.md))
            .font_family(first_family(t.typography.body))
            .text_size(px(t.typography.size_md))
            .text_color(fg)
            .child(div().flex_1().child(self.message.clone()));
        if let Some(label) = self.action.clone() {
            let f = self.on_action.clone();
            row = row.child(
                div()
                    .id((self.id.clone(), "act"))
                    .cursor_pointer()
                    .underline()
                    .on_click(move |_, w, cx| {
                        if let Some(f) = &f {
                            f(w, cx);
                        }
                    })
                    .child(label),
            );
        }
        if self.dismissible {
            let f = self.on_dismiss.clone();
            row = row.child(
                div()
                    .id((self.id.clone(), "x"))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .child(crate::surfaces::glyph_el("✕".into(), t.sizes().icon_sm, fg))
                    .on_click(move |_, w, cx| {
                        gone.update(cx, |g, cx| {
                            *g = true;
                            cx.notify();
                        });
                        if let Some(f) = &f {
                            f(w, cx);
                        }
                    }),
            );
        }
        row.into_any_element()
    }
}

// ───────────────────────────── scroll area ─────────────────────────────

/// `(thumb length, thumb offset)` in px for a track of `track` px, a viewport of
/// `view` px over `view + max` px of content scrolled by `scrolled` (0..=max).
pub fn thumb_geometry(track: f32, view: f32, max: f32, scrolled: f32) -> (f32, f32) {
    if max <= 0.0 || view <= 0.0 {
        return (0.0, 0.0);
    }
    let len = (track * view / (view + max)).clamp(24.0_f32.min(track), track);
    let pos = (scrolled / max).clamp(0.0, 1.0) * (track - len);
    (len, pos)
}

/// Scroll container with thin themed scrollbars (drag a thumb or use the wheel). Scrolls
/// vertically; [`horizontal`](Self::horizontal) adds the other axis.
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    height: Option<f32>,
    max_height: Option<f32>,
    horizontal: bool,
    bars: bool,
    children: Vec<AnyElement>,
}

/// Thickness of a scrollbar track and its gap to the edge, in px.
const BAR: f32 = 6.0;
const BAR_GAP: f32 = 2.0;

impl ScrollArea {
    /// Create a new [`ScrollArea`] (240 px tall unless sized otherwise).
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            height: Some(240.0),
            max_height: None,
            horizontal: false,
            bars: true,
            children: Vec::new(),
        }
    }
    /// Viewport height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h);
        self.max_height = None;
        self
    }
    /// Grow with the content up to `h` px, then scroll.
    pub fn max_height(mut self, h: f32) -> Self {
        self.max_height = Some(h);
        self.height = None;
        self
    }
    /// Also scroll sideways when the content is wider than the viewport.
    pub fn horizontal(mut self, on: bool) -> Self {
        self.horizontal = on;
        self
    }
    /// Draw the scrollbars (default on). Off keeps wheel and trackpad scrolling.
    pub fn scrollbar(mut self, on: bool) -> Self {
        self.bars = on;
        self
    }
    /// Add a child.
    pub fn child(mut self, c: impl IntoElement) -> Self {
        self.children.push(c.into_any_element());
        self
    }
}

/// Scrollbar geometry along both axes, as `(x, y)` pairs.
struct ScrollMetrics {
    track: (f32, f32),
    len: (f32, f32),
    max: (f32, f32),
}

type DragState = gpui::Entity<Option<(Axis, f32, f32)>>;

/// Move the scroll offset while a thumb is dragged.
fn drag_scroll(
    m: &ScrollMetrics,
    handle: &ScrollHandle,
    drag: &DragState,
    pos: gpui::Point<gpui::Pixels>,
    window: &mut Window,
    cx: &mut App,
) {
    let Some((axis, p0, off0)) = *drag.read(cx) else {
        return;
    };
    let o = handle.offset();
    match axis {
        Axis::Vertical => {
            let span = (m.track.1 - m.len.1).max(1.0);
            let new = (off0 + (f32::from(pos.y) - p0) * m.max.1 / span).clamp(0.0, m.max.1);
            handle.set_offset(point(o.x, px(-new)));
        }
        Axis::Horizontal => {
            let span = (m.track.0 - m.len.0).max(1.0);
            let new = (off0 + (f32::from(pos.x) - p0) * m.max.0 / span).clamp(0.0, m.max.0);
            handle.set_offset(point(px(-new), o.y));
        }
    }
    window.refresh();
}

/// One scrollbar: its rail and draggable thumb. `g` is `[len, pos, track, start offset]`.
fn scroll_bar(
    t: &dodai_ui_core::Theme,
    id: &ElementId,
    drag: &DragState,
    axis: Axis,
    g: [f32; 4],
    active: bool,
) -> (gpui::Div, gpui::Stateful<gpui::Div>) {
    let [len, pos, track, start] = g;
    let vertical = axis == Axis::Vertical;
    let thumb_color = if active {
        t.palette.primary
    } else {
        t.palette.border_strong
    };
    let id = (id.clone(), if vertical { "thumb-y" } else { "thumb-x" });
    let mut thumb = shaped_id(
        id,
        &crate::paint::scaled_shape(&t.shape, BAR),
        ShapeStyle::from_tokens(t, Some(thumb_color), None, false, false),
    )
    .absolute()
    .cursor_pointer();
    let mut rail = div()
        .absolute()
        .bg(hsla(t.palette.surface_raised.with_alpha(0.5)));
    if vertical {
        thumb = thumb
            .top(px(BAR_GAP + pos))
            .right(px(BAR_GAP))
            .w(px(BAR))
            .h(px(len));
        rail = rail
            .top(px(BAR_GAP))
            .right(px(BAR_GAP))
            .w(px(BAR))
            .h(px(track));
    } else {
        thumb = thumb
            .left(px(BAR_GAP + pos))
            .bottom(px(BAR_GAP))
            .h(px(BAR))
            .w(px(len));
        rail = rail
            .left(px(BAR_GAP))
            .bottom(px(BAR_GAP))
            .h(px(BAR))
            .w(px(track));
    }
    let drag = drag.clone();
    let thumb = thumb.on_mouse_down(MouseButton::Left, move |e, _, cx| {
        let p = f32::from(if vertical { e.position.y } else { e.position.x });
        drag.update(cx, |d, cx| {
            *d = Some((axis, p, start));
            cx.notify();
        });
        cx.stop_propagation();
    });
    (rail, thumb)
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let hs = window.use_keyed_state(self.id.clone(), cx, |_, _| ScrollHandle::new());
        let handle = hs.read(cx).clone();
        let drag =
            window.use_keyed_state(
                (self.id.clone(), "drag"),
                cx,
                |_, _| None::<(Axis, f32, f32)>,
            );
        let bounds = handle.bounds().size;
        let (view_h, view_w) = (f32::from(bounds.height), f32::from(bounds.width));
        let max = handle.max_offset();
        let (max_y, max_x) = (f32::from(max.height), f32::from(max.width));
        let off = handle.offset();
        let (off_y, off_x) = (-f32::from(off.y), -f32::from(off.x));
        if view_h <= 0.0 {
            crate::motion::next_frame(window); // first frame: bounds unknown yet
        }
        let both = self.horizontal && max_x > 0.0 && max_y > 0.0;
        let corner = if both { BAR + BAR_GAP } else { 0.0 };
        let track_y = (view_h - 2.0 * BAR_GAP - corner).max(1.0);
        let track_x = (view_w - 2.0 * BAR_GAP - corner).max(1.0);
        let (len_y, pos_y) = thumb_geometry(track_y, view_h, max_y, off_y);
        let (len_x, pos_x) = if self.horizontal {
            thumb_geometry(track_x, view_w, max_x, off_x)
        } else {
            (0.0, 0.0)
        };
        let dragging = *drag.read(cx);
        let pad = if self.bars { BAR + 2.0 * BAR_GAP } else { 0.0 };
        let mut view = div()
            .id((self.id.clone(), "view"))
            .track_scroll(&handle)
            .pr(px(if len_y > 0.0 { pad } else { 0.0 }))
            .pb(px(if len_x > 0.0 { pad } else { 0.0 }));
        view = match self.height {
            Some(_) => view.size_full(),
            None => view.max_h(px(self.max_height.unwrap_or(240.0))),
        };
        view = if self.horizontal {
            view.overflow_scroll()
        } else {
            view.overflow_y_scroll()
        };
        let mut root = div()
            .id((self.id.clone(), "root"))
            .relative()
            .w_full()
            .overflow_hidden();
        if let Some(h) = self.height {
            root = root.h(px(h));
        }
        root = root.child(view.children(self.children));
        let bar = |axis: Axis, len: f32, pos: f32, track: f32| {
            let start = if axis == Axis::Vertical { off_y } else { off_x };
            let active = dragging.is_some_and(|d| d.0 == axis);
            scroll_bar(&t, &self.id, &drag, axis, [len, pos, track, start], active)
        };
        if self.bars && len_y > 0.0 {
            let (rail, thumb) = bar(Axis::Vertical, len_y, pos_y, track_y);
            root = root.child(rail).child(thumb);
        }
        if self.bars && len_x > 0.0 {
            let (rail, thumb) = bar(Axis::Horizontal, len_x, pos_x, track_x);
            root = root.child(rail).child(thumb);
        }
        let m = ScrollMetrics {
            track: (track_x, track_y),
            len: (len_x, len_y),
            max: (max_x, max_y),
        };
        root.on_mouse_move({
            let (drag, handle) = (drag.clone(), handle.clone());
            move |e, window, cx| drag_scroll(&m, &handle, &drag, e.position, window, cx)
        })
        .on_mouse_up(MouseButton::Left, {
            let drag = drag.clone();
            move |_, _, cx| {
                drag.update(cx, |d, cx| {
                    *d = None;
                    cx.notify();
                })
            }
        })
        .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
            drag.update(cx, |d, cx| {
                *d = None;
                cx.notify();
            })
        })
    }
}

// ───────────────────────────── image ─────────────────────────────

/// How an [`Image`] fills its frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageFit {
    /// Fill the frame, cropping the overflow.
    #[default]
    Cover,
    /// Whole image visible, letterboxed.
    Contain,
    /// Stretch to the frame.
    Fill,
}

#[derive(Clone)]
enum Source {
    Path(PathBuf),
    Url(SharedString),
}

/// Image in a themed frame, with an optional caption and a play badge for video posters.
#[derive(IntoElement)]
pub struct Image {
    source: Source,
    width: f32,
    height: f32,
    fit: ImageFit,
    caption: Option<SharedString>,
    video: bool,
    on_image: bool,
}

impl Image {
    /// Load the picture from a file path.
    pub fn path(p: impl Into<PathBuf>) -> Self {
        Self::from(Source::Path(p.into()))
    }
    /// Load the picture from a URL.
    pub fn url(u: impl Into<SharedString>) -> Self {
        Self::from(Source::Url(u.into()))
    }
    /// Width and height in px.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
    /// How the picture fills its box.
    pub fn fit(mut self, f: ImageFit) -> Self {
        self.fit = f;
        self
    }
    /// Caption text.
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
        self
    }
    /// Lay the caption over the picture on the design's dark fade instead of below it.
    pub fn caption_on_image(mut self, on: bool) -> Self {
        self.on_image = on;
        self
    }
    /// Overlay a ▶ badge (a video poster).
    pub fn video(mut self, on: bool) -> Self {
        self.video = on;
        self
    }
}

impl From<Source> for Image {
    fn from(source: Source) -> Self {
        Self {
            source,
            width: 240.0,
            height: 150.0,
            fit: ImageFit::Cover,
            caption: None,
            video: false,
            on_image: false,
        }
    }
}

impl RenderOnce for Image {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let fit = match self.fit {
            ImageFit::Cover => ObjectFit::Cover,
            ImageFit::Contain => ObjectFit::Contain,
            ImageFit::Fill => ObjectFit::Fill,
        };
        let picture = match self.source {
            Source::Path(p) => img(p),
            Source::Url(u) => img(u),
        }
        .object_fit(fit)
        .size_full()
        .with_fallback({
            let c = hsla(t.palette.text_muted);
            move || {
                div()
                    .size_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(c)
                    .child("image unavailable")
                    .into_any_element()
            }
        });
        let style = ShapeStyle::from_tokens(
            &t,
            Some(t.palette.surface_raised),
            Some(t.palette.border),
            false,
            false,
        );
        let mut frame = shaped(&t.shape, style)
            .relative()
            .w(px(self.width))
            .h(px(self.height))
            .overflow_hidden()
            .child(picture);
        if self.video {
            let d = (self.height * 0.28).clamp(28.0, 56.0);
            let badge = shaped(
                &crate::paint::round_shape(&t.shape, d),
                ShapeStyle::from_tokens(
                    &t,
                    Some(t.palette.background.with_alpha(0.75)),
                    Some(t.palette.primary),
                    true,
                    false,
                ),
            )
            .w(px(d))
            .h(px(d))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(d * 0.45))
            .text_color(hsla(t.palette.primary))
            .child("▶");
            frame = frame.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(badge),
            );
        }
        let mut col = div()
            .flex()
            .flex_col()
            .gap(px(t.spacing.xs))
            .w(px(self.width));
        if let (true, Some(c)) = (self.on_image, self.caption.clone()) {
            let fade_h = (self.height * 0.5).max(t.sizes().touch_target);
            let ink = hsla(t.on(t.palette.roles.shadow.with_alpha(1.0)));
            frame = frame.child(crate::paint::fade_overlay(&t, fade_h)).child(
                div()
                    .absolute()
                    .left(px(t.spacing.md))
                    .right(px(t.spacing.md))
                    .bottom(px(t.spacing.sm))
                    .font_family(first_family(t.typography.body))
                    .text_size(px(t.typography.size_sm))
                    .text_color(ink)
                    .child(c),
            );
            return col.child(frame);
        }
        col = col.child(frame);
        if let Some(c) = self.caption {
            col = col.child(
                div()
                    .font_family(first_family(t.typography.body))
                    .text_size(px(t.typography.size_sm))
                    .text_color(hsla(t.palette.text_muted))
                    .child(c),
            );
        }
        col
    }
}

// ───────────────────────────── avatar group ─────────────────────────────

/// How many avatars show and how many hide behind the `+N` chip.
pub fn avatar_split(total: usize, max: usize) -> (usize, usize) {
    let max = max.max(1);
    if total <= max {
        (total, 0)
    } else {
        (max, total - max)
    }
}

/// Overlapping avatars with a `+N` overflow chip.
#[derive(IntoElement)]
pub struct AvatarGroup {
    names: Vec<SharedString>,
    max: usize,
    size: f32,
}

impl AvatarGroup {
    /// Create a new [`AvatarGroup`].
    pub fn new<S: Into<SharedString>>(initials: impl IntoIterator<Item = S>) -> Self {
        Self {
            names: initials.into_iter().map(Into::into).collect(),
            max: 4,
            size: 36.0,
        }
    }
    /// Avatars shown before the `+N` chip.
    pub fn max(mut self, m: usize) -> Self {
        self.max = m;
        self
    }
    /// Size in px.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
}

impl RenderOnce for AvatarGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let (shown, hidden) = avatar_split(self.names.len(), self.max);
        let overlap = -self.size * 0.2;
        let tones = [Tone::Info, Tone::Success, Tone::Warning, Tone::Danger];
        let mut row = div().flex().items_center();
        for (i, n) in self.names.iter().take(shown).enumerate() {
            let mut a = div().child(
                Avatar::new(n.clone())
                    .size(self.size)
                    .solid(true)
                    .tone(tones[i % tones.len()]),
            );
            if i > 0 {
                a = a.ml(px(overlap));
            }
            row = row.child(a);
        }
        if hidden > 0 {
            row = row.child(
                div().ml(px(if shown > 0 { overlap } else { 0.0 })).child(
                    Avatar::new(format!("+{hidden}"))
                        .size(self.size)
                        .solid(true),
                ),
            );
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wavy_arc_stays_within_amplitude_of_the_radius() {
        let p = wavy_arc_points((0.0, 0.0), 20.0, 0.0, std::f32::consts::TAU, 2.0, 9.0, 0.3);
        assert!(p.len() > 100);
        for (x, y) in p {
            let r = (x * x + y * y).sqrt();
            assert!((r - 20.0).abs() <= 2.0 + 1e-3, "{r}");
        }
    }

    #[test]
    fn arc_runs_from_start_by_sweep() {
        let p = arc_points(0.0, 0.0, 10.0, 0.0, std::f32::consts::PI);
        let (a, b) = (p[0], p[p.len() - 1]);
        assert!((a.0 - 10.0).abs() < 1e-4 && a.1.abs() < 1e-4);
        assert!((b.0 + 10.0).abs() < 1e-3 && b.1.abs() < 1e-3);
        assert!(p.len() > 8);
    }

    #[test]
    fn thumb_geometry_tracks_scroll() {
        assert_eq!(thumb_geometry(200.0, 200.0, 0.0, 0.0), (0.0, 0.0));
        let (len, pos) = thumb_geometry(200.0, 200.0, 200.0, 0.0);
        assert_eq!((len, pos), (100.0, 0.0));
        let (len, pos) = thumb_geometry(200.0, 200.0, 200.0, 200.0);
        assert_eq!(pos + len, 200.0);
        let (len, _) = thumb_geometry(200.0, 100.0, 100_000.0, 0.0);
        assert_eq!(len, 24.0); // never smaller than a grab handle
    }

    #[test]
    fn avatar_overflow() {
        assert_eq!(avatar_split(3, 4), (3, 0));
        assert_eq!(avatar_split(7, 4), (4, 3));
        assert_eq!(avatar_split(2, 0), (1, 1));
    }
}
