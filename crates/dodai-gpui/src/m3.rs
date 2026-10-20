//! Material 3 (expressive) components with no equivalent elsewhere: [`ButtonGroup`],
//! [`SplitButton`] and [`LoadingIndicator`]. They render on every theme; the Material look
//! (pill corners, morphing shapes, tonal colours) keys off `Shape::kind() == Rounded`.

use crate::interact::ClickHandler;
use crate::{hsla, theme_of, Button};
use dodai_core::ButtonVariant;
use gpui::{
    canvas, div, px, AnimationExt, App, ClickEvent, ElementId, IntoElement, ParentElement,
    PathBuilder, RenderOnce, SharedString, Styled, Window,
};
use std::rc::Rc;
use std::time::Duration;

const FULL: f32 = 999.0;
/// Inner corner of a connected group / split button, in px.
pub const INNER_CORNER: f32 = 8.0;

/// Corner radii `[tl, tr, br, bl]` of item `i` of `n` in a button group. A standard
/// (spaced) group is all pills; a connected one is a pill on the outer ends only, and its
/// selected item is a full pill.
pub fn group_radii(i: usize, n: usize, connected: bool, selected: bool) -> [f32; 4] {
    if !connected || selected {
        return [FULL; 4];
    }
    let (first, last) = (i == 0, i + 1 == n);
    let l = if first { FULL } else { INNER_CORNER };
    let r = if last { FULL } else { INNER_CORNER };
    [l, r, r, l]
}

type IndexHandler = Rc<dyn Fn(usize, &mut Window, &mut App)>;

/// A row of related buttons. `connected` joins them with a 2 px gap and small inner corners
/// (the selected one morphs to a pill); otherwise they are separate pills 8 px apart.
/// Selection is controlled: pass `selected` and update it in `on_select`.
#[derive(IntoElement)]
pub struct ButtonGroup {
    id: ElementId,
    labels: Vec<SharedString>,
    selected: Option<usize>,
    connected: bool,
    on_select: Option<IndexHandler>,
}

impl ButtonGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            labels: Vec::new(),
            selected: None,
            connected: false,
            on_select: None,
        }
    }
    pub fn labels<S: Into<SharedString>>(mut self, labels: impl IntoIterator<Item = S>) -> Self {
        self.labels = labels.into_iter().map(Into::into).collect();
        self
    }
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    pub fn connected(mut self, c: bool) -> Self {
        self.connected = c;
        self
    }
    crate::callback!(on_select, usize);
}

impl RenderOnce for ButtonGroup {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let rounded = t.look().tonal;
        let n = self.labels.len();
        let gap = if self.connected && rounded { 2.0 } else { 8.0 };
        let mut row = div().flex().items_center().gap(px(gap));
        for (i, label) in self.labels.iter().enumerate() {
            let on = self.selected == Some(i);
            let mut b = Button::new((self.id.clone(), i.to_string()), label.clone())
                .variant(if on {
                    ButtonVariant::Primary
                } else if rounded {
                    ButtonVariant::Secondary
                } else {
                    ButtonVariant::Outline
                })
                .radii(group_radii(i, n, self.connected, on));
            if let Some(f) = self.on_select.clone() {
                b = b.on_click(move |_, w, cx| f(i, w, cx));
            }
            row = row.child(b);
        }
        row
    }
}

/// A button with a trailing menu toggle: `[ Label | ▾ ]`. `on_click` runs the main action,
/// `on_toggle` opens or closes the menu (rendered by the caller, e.g. a `DropdownMenu`).
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    open: bool,
    on_click: Option<ClickHandler>,
    on_toggle: Option<ClickHandler>,
}

impl SplitButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Primary,
            open: false,
            on_click: None,
            on_toggle: None,
        }
    }
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Draw the toggle as open (chevron up).
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
    pub fn on_toggle(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }
}

impl RenderOnce for SplitButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let rounded = t.look().tonal;
        let mut lead = Button::new((self.id.clone(), "lead"), self.label)
            .variant(self.variant)
            .radii([FULL, 4.0, 4.0, FULL]);
        if let Some(f) = self.on_click {
            lead = lead.on_click(move |e, w, cx| f(e, w, cx));
        }
        let mut menu = Button::new((self.id, "menu"), if self.open { "▴" } else { "▾" })
            .variant(self.variant)
            .radii([4.0, FULL, FULL, 4.0])
            .refine(|d| d.w(px(48.0)).px(px(0.0)));
        if let Some(f) = self.on_toggle {
            menu = menu.on_click(move |e, w, cx| f(e, w, cx));
        }
        div()
            .flex()
            .items_center()
            .gap(px(if rounded { 2.0 } else { 4.0 }))
            .child(lead)
            .child(menu)
    }
}

/// Outline radius multiplier (relative to the base radius) of a morph shape at angle `a`.
/// Shape `k` is a lobed circle: `1 + amp * cos(lobes * a)`.
fn lobed(k: usize, a: f32) -> f32 {
    // (lobes, amplitude): soft burst, cookie, pentagon, oval, sunny, square cookie, pill.
    let (n, amp) = match k % 7 {
        0 => (12.0, 0.08),
        1 => (9.0, 0.07),
        2 => (5.0, 0.07),
        3 => (2.0, 0.16),
        4 => (8.0, 0.09),
        5 => (4.0, 0.11),
        _ => (2.0, 0.11),
    };
    1.0 + amp * (n * a).cos()
}

/// Outline points of the loading indicator at animation phase `phase` (0..1 per full
/// sequence), as `(x, y)` offsets from the centre for a base radius of 1. The shape morphs to
/// the next of seven while turning one revolution per sequence.
pub fn loading_outline(phase: f32, samples: usize) -> Vec<(f32, f32)> {
    let p = phase.rem_euclid(1.0) * 7.0;
    let k = p.floor() as usize;
    // Hold each shape for a moment, then ease into the next.
    let s = ((p - p.floor()) * 1.6 - 0.3).clamp(0.0, 1.0);
    let s = s * s * (3.0 - 2.0 * s);
    let spin = phase * std::f32::consts::TAU * 3.0;
    (0..samples)
        .map(|i| {
            let a = i as f32 / samples as f32 * std::f32::consts::TAU;
            let r = lobed(k, a) * (1.0 - s) + lobed(k + 1, a) * s;
            let ang = a + spin;
            (r * ang.cos(), r * ang.sin())
        })
        .collect()
}

/// Material 3 expressive loading indicator: a 48 px container with a 38 px shape that morphs
/// and turns. `contained` draws it on a filled pill/circle.
#[derive(IntoElement)]
pub struct LoadingIndicator {
    id: ElementId,
    contained: bool,
    motion: Option<crate::motion::Motion>,
}

impl LoadingIndicator {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            contained: false,
            motion: None,
        }
    }
    pub fn contained(mut self, c: bool) -> Self {
        self.contained = c;
        self
    }
    /// Override the sequence duration (default 4.2 s, linear).
    pub fn motion(mut self, m: crate::motion::Motion) -> Self {
        self.motion = Some(m);
        self
    }
}

impl RenderOnce for LoadingIndicator {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        use crate::motion::MotionExt;
        let t = theme_of(cx);
        let r = &t.palette.roles;
        let (container, ink) = if self.contained {
            (Some(r.primary_container), r.on_primary_container)
        } else {
            (None, t.palette.primary)
        };
        let ink = hsla(ink);
        let m = self.motion.unwrap_or_else(|| {
            crate::motion::Motion::new(Duration::from_millis(4200), crate::motion::Easing::Linear)
        });
        let mut base = div().w(px(48.0)).h(px(48.0)).rounded_full().flex_none();
        if let Some(c) = container {
            base = base.bg(hsla(c));
        }
        base.with_animation(self.id, m.looping(), move |el, phase| {
            let pts = loading_outline(phase, 72);
            el.child(
                canvas(
                    |_, _, _| (),
                    move |b, _, window, _| {
                        let c = b.center();
                        let rad = 19.0 / 1.1;
                        let poly: Vec<_> = pts
                            .iter()
                            .map(|(x, y)| gpui::point(c.x + px(x * rad), c.y + px(y * rad)))
                            .collect();
                        let mut pb = PathBuilder::fill();
                        pb.add_polygon(&poly, true);
                        if let Ok(p) = pb.build() {
                            window.paint_path(p, ink);
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_groups_are_pills() {
        for i in 0..3 {
            assert_eq!(group_radii(i, 3, false, false), [FULL; 4]);
        }
    }

    #[test]
    fn connected_groups_round_only_the_outer_ends() {
        assert_eq!(
            group_radii(0, 3, true, false),
            [FULL, INNER_CORNER, INNER_CORNER, FULL]
        );
        assert_eq!(group_radii(1, 3, true, false), [INNER_CORNER; 4]);
        assert_eq!(
            group_radii(2, 3, true, false),
            [INNER_CORNER, FULL, FULL, INNER_CORNER]
        );
        // The selected item is a pill wherever it is.
        assert_eq!(group_radii(1, 3, true, true), [FULL; 4]);
        // A single item has both ends outer.
        assert_eq!(group_radii(0, 1, true, false), [FULL; 4]);
    }

    #[test]
    fn loading_outline_is_closed_bounded_and_morphs() {
        let a = loading_outline(0.0, 48);
        assert_eq!(a.len(), 48);
        for (x, y) in &a {
            let r = (x * x + y * y).sqrt();
            assert!((0.8..=1.2).contains(&r), "radius {r}");
        }
        // Different phases give different outlines; phase wraps.
        assert_ne!(loading_outline(0.0, 48), loading_outline(0.5, 48));
        let (p0, p1) = (loading_outline(0.0, 48), loading_outline(1.0, 48));
        for (u, v) in p0.iter().zip(&p1) {
            assert!((u.0 - v.0).abs() < 1e-4 && (u.1 - v.1).abs() < 1e-4);
        }
    }
}
