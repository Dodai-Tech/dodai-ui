//! Display components: [`CircularProgress`], [`Banner`], [`ScrollArea`],
//! [`Image`] and [`AvatarGroup`].

use crate::data::Avatar;
use crate::feedback::alert_style;
use crate::paint::{shaped, shaped_id, ShapeStyle};
use crate::text::first_family;
use crate::{hsla, theme_of};
use dodai_core::Tone;
use gpui::{
    canvas, div, img, point, px, AnyElement, App, ElementId, InteractiveElement, IntoElement,
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

fn elapsed_secs() -> f32 {
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
    size: f32,
    thickness: f32,
    tone: Option<Tone>,
    label: bool,
}

impl CircularProgress {
    pub fn new(value: f32) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            size: 56.0,
            thickness: 5.0,
            tone: None,
            label: true,
        }
    }
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            label: false,
            ..Self::new(0.0)
        }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    pub fn thickness(mut self, t: f32) -> Self {
        self.thickness = t;
        self
    }
    pub fn tone(mut self, t: Tone) -> Self {
        self.tone = Some(t);
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
        let (size, th, value) = (self.size, self.thickness, self.value);
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
                        ring(arc_points(c0, c1, r, 0.0, full), track, true);
                        match value {
                            Some(v) if v > 0.0 => {
                                let start = -std::f32::consts::FRAC_PI_2;
                                ring(
                                    arc_points(c0, c1, r, start, full * v.min(0.9999)),
                                    fill,
                                    false,
                                )
                            }
                            Some(_) => {}
                            None => {
                                let a = elapsed_secs() * 4.0;
                                ring(arc_points(c0, c1, r, a, full * 0.27), fill, false);
                                window.request_animation_frame();
                            }
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
}

impl Banner {
    pub fn new(id: impl Into<ElementId>, tone: Tone, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            tone,
            message: message.into(),
            action: None,
            on_action: None,
            on_dismiss: None,
            dismissible: true,
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
    pub fn dismissible(mut self, on: bool) -> Self {
        self.dismissible = on;
        self
    }
    pub fn on_dismiss(mut self, f: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Banner {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let gone = window.use_keyed_state(self.id.clone(), cx, |_, _| false);
        if *gone.read(cx) {
            return div().into_any_element();
        }
        let (style, fg) = alert_style(&t, self.tone, true);
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
                    .child("✕")
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

/// Vertical scroll container with a thin themed scrollbar (drag the thumb or use the wheel).
#[derive(IntoElement)]
pub struct ScrollArea {
    id: ElementId,
    height: f32,
    children: Vec<AnyElement>,
}

impl ScrollArea {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            height: 240.0,
            children: Vec::new(),
        }
    }
    /// Viewport height in px.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    pub fn child(mut self, c: impl IntoElement) -> Self {
        self.children.push(c.into_any_element());
        self
    }
}

impl RenderOnce for ScrollArea {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let hs = window.use_keyed_state(self.id.clone(), cx, |_, _| ScrollHandle::new());
        let handle = hs.read(cx).clone();
        let drag = window.use_keyed_state((self.id.clone(), "drag"), cx, |_, _| None::<(f32, f32)>);
        let view = f32::from(handle.bounds().size.height);
        let max = f32::from(handle.max_offset().height);
        let off = -f32::from(handle.offset().y);
        if view <= 0.0 {
            window.request_animation_frame(); // first frame: bounds unknown yet
        }
        let track = (self.height - 4.0).max(1.0);
        let (len, pos) = thumb_geometry(track, view, max, off);
        let dragging = drag.read(cx).is_some();
        let mut root = div()
            .id((self.id.clone(), "root"))
            .relative()
            .w_full()
            .h(px(self.height))
            .overflow_hidden()
            .child(
                div()
                    .id((self.id.clone(), "view"))
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&handle)
                    .pr(px(10.0))
                    .children(self.children),
            );
        if len > 0.0 {
            let thumb_color = if dragging {
                t.palette.primary
            } else {
                t.palette.border_strong
            };
            let thumb = shaped_id(
                (self.id.clone(), "thumb"),
                &crate::paint::scaled_shape(&t.shape, 6.0),
                ShapeStyle::from_tokens(&t, Some(thumb_color), None, false, false),
            )
            .absolute()
            .top(px(2.0 + pos))
            .right(px(2.0))
            .w(px(6.0))
            .h(px(len))
            .cursor_pointer()
            .on_mouse_down(MouseButton::Left, {
                let drag = drag.clone();
                move |e, _, cx| {
                    let y = f32::from(e.position.y);
                    drag.update(cx, |d, cx| {
                        *d = Some((y, off));
                        cx.notify();
                    });
                    cx.stop_propagation();
                }
            });
            root = root
                .child(
                    div()
                        .absolute()
                        .top(px(2.0))
                        .right(px(2.0))
                        .w(px(6.0))
                        .h(px(track))
                        .bg(hsla(t.palette.surface_raised.with_alpha(0.5))),
                )
                .child(thumb);
        }
        root.on_mouse_move({
            let (drag, handle) = (drag.clone(), handle.clone());
            move |e, window, cx| {
                let Some((y0, off0)) = *drag.read(cx) else {
                    return;
                };
                let span = (track - len).max(1.0);
                let new = (off0 + (f32::from(e.position.y) - y0) * max / span).clamp(0.0, max);
                let o = handle.offset();
                handle.set_offset(point(o.x, px(-new)));
                window.refresh();
            }
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
}

impl Image {
    pub fn path(p: impl Into<PathBuf>) -> Self {
        Self::from(Source::Path(p.into()))
    }
    pub fn url(u: impl Into<SharedString>) -> Self {
        Self::from(Source::Url(u.into()))
    }
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
    pub fn fit(mut self, f: ImageFit) -> Self {
        self.fit = f;
        self
    }
    pub fn caption(mut self, c: impl Into<SharedString>) -> Self {
        self.caption = Some(c.into());
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
