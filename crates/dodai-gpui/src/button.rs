//! [`Button`]: every [`ButtonVariant`], hover/press/disabled, shaped per theme.

use crate::interact::{track, use_interact, ClickHandler};
use crate::paint::{shaped_id, ShapeEdge, ShapeStyle};
use crate::text::{first_family, format_label};
use crate::{hsla, theme_of};
use dodai_core::{ButtonVariant, ControlState, Theme};
use gpui::{
    div, px, App, ClickEvent, ElementId, Hsla, IntoElement, ParentElement, SharedString, Stateful,
    StatefulInteractiveElement, Styled, Window,
};

/// Resolve a button's paint style + text colour from tokens (pure, no gpui context).
pub fn button_style(
    theme: &Theme,
    variant: ButtonVariant,
    state: ControlState,
) -> (ShapeStyle, Hsla) {
    let b = theme.button(variant, state);
    let lit = matches!(
        state,
        ControlState::Hovered | ControlState::Pressed | ControlState::Focused
    );
    let glows = lit && variant != ButtonVariant::Ghost;
    (
        ShapeStyle::from_tokens(
            theme,
            b.background,
            b.border,
            glows,
            state != ControlState::Disabled
                && variant != ButtonVariant::Scrim
                && b.background.is_some(),
        )
        .with_surface(theme, dodai_core::SurfaceClass::Button),
        hsla(b.foreground),
    )
}

type Refine = Box<dyn FnOnce(Stateful<gpui::Div>) -> Stateful<gpui::Div>>;

/// Which end of a [`Button`] is cropped (the Phantom Liberty cut-corner buttons). The cut
/// sits on one corner; a small tick mark on the opposite edge points at the neighbour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Crop {
    /// Top-left corner cut, tick on the right edge.
    Left,
    /// Bottom-right corner cut, tick on the left edge.
    Right,
    /// A shallow slot in the middle of the top edge; the corners stay as they are.
    Top,
    /// A shallow slot in the middle of the bottom edge; the corners stay as they are.
    Bottom,
}

#[derive(Default)]
struct CropDefaults {
    crop: Option<Crop>,
    gap: Option<f32>,
}
impl gpui::Global for CropDefaults {}

/// The crop every [`Button`] gets unless it sets its own with [`Button::crop`].
pub fn button_crop(cx: &App) -> Option<Crop> {
    cx.try_global::<CropDefaults>().and_then(|d| d.crop)
}

/// The tick gap (px) every cropped [`Button`] uses unless it sets [`Button::cut_gap`].
pub fn button_cut_gap(cx: &App) -> Option<f32> {
    cx.try_global::<CropDefaults>().and_then(|d| d.gap)
}

/// Crop every button (`None` = no crop) and repaint. Per-button `.crop(..)` still wins.
pub fn set_button_crop(cx: &mut App, crop: Option<Crop>) {
    cx.default_global::<CropDefaults>().crop = crop;
    cx.refresh_windows();
}

/// Set the tick gap for every cropped button (`None` = theme default) and repaint.
pub fn set_button_cut_gap(cx: &mut App, gap: Option<f32>) {
    cx.default_global::<CropDefaults>().gap = gap;
    cx.refresh_windows();
}

/// A resolved crop: which end, and the gap between the two bars of its edge tick.
#[derive(Clone, Copy)]
pub(crate) struct CropSpec {
    end: Crop,
    gap: f32,
}

/// The crop a control gets: its own, else the global default; only on designs with cut corners.
pub(crate) fn resolve_crop(
    t: &Theme,
    cx: &App,
    crop: Option<Crop>,
    gap: Option<f32>,
) -> Option<CropSpec> {
    let end = crop
        .or_else(|| button_crop(cx))
        .filter(|_| t.look().cut_corners)?;
    let gap = gap.or_else(|| button_cut_gap(cx)).unwrap_or(t.spacing.xs);
    Some(CropSpec { end, gap })
}

impl CropSpec {
    /// Cut the corner (or slot the edge), open the border, and drop the shadow.
    pub(crate) fn apply(&self, t: &Theme, shape: &mut dodai_core::Shape, style: &mut ShapeStyle) {
        let (edge, half, depth) = match self.end {
            Crop::Left => (ShapeEdge::Right, self.gap / 2.0, t.spacing.md),
            Crop::Right => (ShapeEdge::Left, self.gap / 2.0, t.spacing.md),
            // Superficial: a wide, shallow slot.
            Crop::Top => (ShapeEdge::Top, t.spacing.md, t.spacing.xs),
            Crop::Bottom => (ShapeEdge::Bottom, t.spacing.md, t.spacing.xs),
        };
        *style = style.with_edge_gap(edge, half, depth);
        // Flat: an offset shadow would sit over the cut and hide it.
        style.shadow = None;
        match self.end {
            Crop::Left => shape.corners = dodai_core::Corners::TOP_LEFT,
            Crop::Right => shape.corners = dodai_core::Corners::BOTTOM_RIGHT,
            Crop::Top | Crop::Bottom => return,
        }
        // Steep 45° cut: the largest the shape allows (a third of the height).
        shape.chamfer = shape.chamfer.max(t.shape.control_height);
    }

    /// Whether the tick sits on the leading (left) edge.
    pub(crate) fn tick_leads(&self) -> bool {
        self.end == Crop::Right
    }

    /// Extra width the tick needs inside the control.
    pub(crate) fn inset(&self, t: &Theme) -> f32 {
        match self.end {
            Crop::Left | Crop::Right => t.spacing.md,
            Crop::Top | Crop::Bottom => 0.0,
        }
    }

    /// The two-bar tick on the end opposite the cut.
    pub(crate) fn tick(&self, t: &Theme, colour: Hsla) -> Option<gpui::Div> {
        if matches!(self.end, Crop::Top | Crop::Bottom) {
            return None;
        }
        let bar = t.shape.border_width.max(1.0);
        let tick = div()
            .absolute()
            .top(gpui::relative(0.5))
            .mt(px(-(bar + self.gap / 2.0)))
            .w(px(t.spacing.md))
            .flex()
            .flex_col()
            .gap(px(self.gap))
            .child(div().h(px(bar)).bg(colour))
            .children((self.gap > 0.0).then(|| div().h(px(bar)).bg(colour)));
        Some(match self.end {
            Crop::Left | Crop::Top | Crop::Bottom => tick.right(px(0.0)),
            Crop::Right => tick.left(px(0.0)),
        })
    }
}

/// Themed button. Construct with an id so hover/press state can be tracked.
///
/// ```ignore
/// Button::new("go", "Hack").variant(ButtonVariant::Primary).on_click(|_, _, _| {})
/// ```
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    disabled: bool,
    on_click: Option<ClickHandler>,
    refine: Option<Refine>,
    shimmer: bool,
    corners: Option<dodai_core::Corners>,
    radii: Option<[f32; 4]>,
    elevated: bool,
    size: Option<dodai_core::ControlSize>,
    square: bool,
    crop: Option<Crop>,
    cut_gap: Option<f32>,
}

impl Button {
    /// Create a new [`Button`].
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Primary,
            disabled: false,
            shimmer: false,
            corners: None,
            radii: None,
            elevated: false,
            size: None,
            square: false,
            crop: None,
            cut_gap: None,
            on_click: None,
            refine: None,
        }
    }
    /// Choose the emphasis variant (default `Primary`).
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Disable interaction and use disabled tokens.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// Click callback (not called when disabled).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Box::new(f));
        self
    }
    /// Escape hatch for extra layout styling (width, margin…) on the root div.
    pub fn refine(
        mut self,
        f: impl FnOnce(Stateful<gpui::Div>) -> Stateful<gpui::Div> + 'static,
    ) -> Self {
        self.refine = Some(Box::new(f));
        self
    }
}

impl Button {
    /// Cut these corners instead of the theme's (e.g. `Corners::TOP_LEFT`,
    /// `Corners::DIAGONAL`, or any combination). Has an effect on themes with
    /// a chamfer size.
    pub fn corners(mut self, c: dodai_core::Corners) -> Self {
        self.corners = Some(c);
        self
    }
    /// Per-corner radii `[top-left, top-right, bottom-right, bottom-left]` in px on rounded
    /// themes (connected button groups, split buttons). Ignored by other shapes.
    pub fn radii(mut self, r: [f32; 4]) -> Self {
        self.radii = Some(r);
        self
    }
    /// Material elevated button: tonal surface lifted by a shadow (1 dp, 2 dp on hover).
    /// Other design systems draw the `Secondary` variant.
    pub fn elevated(mut self, on: bool) -> Self {
        self.elevated = on;
        self
    }
    /// One of the five sizes (XS 32 … XL 136 under Material); default is the theme's own
    /// control height.
    pub fn size(mut self, s: dodai_core::ControlSize) -> Self {
        self.size = Some(s);
        self
    }
    /// Square (rounded-rectangle) shape instead of the pill, on rounded designs.
    pub fn square(mut self, on: bool) -> Self {
        self.square = on;
        self
    }
    /// Crop the left or right end (cut corner plus an edge tick). Combine with
    /// [`Button::corners`] to cut other corners instead. Only designs with cut
    /// corners draw it; others ignore it.
    pub fn crop(mut self, c: Crop) -> Self {
        self.crop = Some(c);
        self
    }
    /// Space in px between the two bars of a cropped button's edge tick, left open in the border
    /// (default: the theme's `xs` spacing). `0.0` draws a single tick. Only used with [`Button::crop`].
    pub fn cut_gap(mut self, px: f32) -> Self {
        self.cut_gap = Some(px.max(0.0));
        self
    }
    /// Sweep a shimmer highlight over the button (e.g. while busy), clipped to its shape.
    pub fn shimmer(mut self, on: bool) -> Self {
        self.shimmer = on;
        self
    }
}

impl Button {
    fn build(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = theme_of(cx);
        let st = use_interact(&self.id, window, cx);
        let state = st.read(cx).control_state(self.disabled);
        let lifted = self.elevated && t.look().tonal;
        let resolve = |s: ControlState| {
            if self.elevated {
                let b = t.elevated_button(s);
                let mut st = ShapeStyle::from_tokens(&t, b.background, b.border, false, false)
                    .with_surface(&t, dodai_core::SurfaceClass::Button);
                if lifted && s != ControlState::Disabled {
                    let level = if s == ControlState::Idle { 1 } else { 2 };
                    st = st.with_lift(&t, level);
                }
                (st, hsla(b.foreground))
            } else {
                button_style(&t, self.variant, s)
            }
        };
        let (mut style, mut fg) = resolve(state);
        // Neo-brutalism: the button sinks into its hard shadow on hover.
        let mut sink = 0.0;
        if state != ControlState::Disabled {
            let lvl = crate::motion::smooth(
                window,
                cx,
                (self.id.clone(), "fx").into(),
                if state == ControlState::Idle {
                    0.0
                } else {
                    1.0
                },
            );
            let (idle, idle_fg) = resolve(ControlState::Idle);
            style.fill = crate::motion::mix_opt(idle.fill, style.fill, lvl);
            style.stroke = crate::motion::mix_opt(idle.stroke, style.stroke, lvl);
            fg = crate::motion::mix(idle_fg, fg, lvl);
            if let (true, Some((c, off))) = (t.look().sink, style.shadow) {
                sink = off * lvl;
                style.shadow = Some((c, off - sink));
            }
        }
        if self.shimmer {
            style = style.with_shimmer(crate::paint::Shimmer {
                // A light sweep on filled buttons (dark label text would smear),
                // the accent itself on outline/ghost ones.
                color: if style.fill.is_some() {
                    hsla(t.effects.highlight)
                } else {
                    style.stroke.unwrap_or(fg).opacity(0.25)
                },
                ms: 1400,
                playback: crate::paint::Playback::Loop,
            });
        }
        let pressed = state == ControlState::Pressed;
        // Hover lift: a spring in the theme's personality (gpui has no transforms, so only the
        // vertical offset of the pose is applied).
        let lift = if state == ControlState::Disabled || t.look().sink {
            0.0
        } else {
            crate::motion::hover_lift(
                window,
                cx,
                (self.id.clone(), "lift").into(),
                state != ControlState::Idle,
            )
        };
        let mut shape = t.role_shape(dodai_core::ShapeRole::Button);
        let crop = resolve_crop(&t, cx, self.crop, self.cut_gap);
        if let Some(c) = &crop {
            c.apply(&t, &mut shape, &mut style);
        }
        // Explicit corners win over the crop's default cut (the tick stays).
        if let Some(c) = self.corners {
            shape.corners = c;
        }
        let tick_colour = style.stroke.unwrap_or(fg);
        let m = self.size.map(|s| t.control_metrics(s));
        if let (true, Some(m), true) = (self.square, m, t.look().tonal) {
            shape.radius = m.square_radius;
        }
        let mut el = track(shaped_id(self.id, &shape, style), &st)
            .h(px(m.map_or(t.shape.control_height, |m| m.height)))
            .px(px(
                m.map_or(t.button_padding(), |m| m.padding) + crop.map_or(0.0, |c| c.inset(&t))
            ))
            .mt(px(lift + sink + if pressed { 1.0 } else { 0.0 }))
            .mb(px(-lift - sink + if pressed { -1.0 } else { 0.0 }))
            .ml(px(sink))
            .mr(px(-sink))
            .flex()
            .items_center()
            .justify_center()
            .font_family(first_family(t.typography.body))
            .text_size(px(m.map_or(t.typography.size_md, |m| m.text_size)))
            .text_color(fg)
            .child(div().child(format_label(&t.typography, &self.label)));
        if let Some(c) = &crop {
            el = el.relative().children(c.tick(&t, tick_colour));
        }
        if let (Some(r), true) = (self.radii, t.look().tonal) {
            el = el
                .rounded_tl(px(r[0]))
                .rounded_tr(px(r[1]))
                .rounded_br(px(r[2]))
                .rounded_bl(px(r[3]));
        }
        if !self.disabled {
            el = el.cursor_pointer();
            if let Some(h) = self.on_click {
                el = el.on_click(move |e, w, cx| h(e, w, cx));
            }
        }
        match self.refine {
            Some(f) => f(el),
            None => el,
        }
    }
}

crate::entry_render!(Button);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{light_rounded, square};
    use dodai::themes;

    #[test]
    fn colours_come_from_tokens() {
        for t in [light_rounded(), square(), themes::cyberpunk::yellow()] {
            let (s, fg) = button_style(&t, ButtonVariant::Primary, ControlState::Idle);
            assert_eq!(s.fill, Some(hsla(t.palette.primary)));
            assert_eq!(fg, hsla(t.palette.on_primary));
            assert!(s.glow.is_none(), "idle never glows");
        }
    }

    #[test]
    fn outline_has_border_and_hover_glow_follows_theme() {
        let c = themes::cyberpunk::yellow();
        let (s, _) = button_style(&c, ButtonVariant::Outline, ControlState::Hovered);
        assert!(s.stroke.is_some() && s.glow.is_some());
        let l = light_rounded();
        let (s, _) = button_style(&l, ButtonVariant::Outline, ControlState::Hovered);
        assert!(s.stroke.is_some() && s.glow.is_none());
    }

    #[test]
    fn disabled_and_ghost() {
        let t = themes::cyberpunk::yellow();
        let (s, _) = button_style(&t, ButtonVariant::Primary, ControlState::Disabled);
        assert!(s.glow.is_none() && s.shadow.is_none());
        let (s, _) = button_style(&t, ButtonVariant::Ghost, ControlState::Hovered);
        assert!(s.glow.is_none());
    }
}
